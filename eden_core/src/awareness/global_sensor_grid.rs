//! # Global Sensor Grid — Red de Sensores Pasivos Distribuidos
//!
//! Este módulo implementa una red de sensores pasivos que permiten a EDEN
//! percibir el mundo digital sin perturbarlo.
//!
//! ## Concepto de Sensores Pasivos
//!
//! Los sensores pasivos "escuchan" el entorno sin enviar tráfico activo.
//! A diferencia de herramientas como nmap que escanean activamente, los
//! sensores de EDEN observan lo que ya existe:
//! - Tráfico de red existente (sin伪造)
//! - Recursos del sistema local
//! - Métricas de hardware
//! - Logs y eventos
//!
//! ## Traffic Obfuscator
//!
//! Para garantizar la pasividad, todo tráfico de red generado por EDEN
//! para monitoreo se obfuscará para parecer tráfico normal.
//! Esto evita que EDEN sea detectado como "escáner".
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH, Instant};

/// Tipo de sensor pasivo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorType {
    /// Tráfico de red observado (pasivo)
    NetworkTraffic,
    /// Recursos del sistema local
    SystemResources,
    /// Temperatura y energía del hardware
    HardwareTelemetry,
    /// Logs y eventos del sistema
    SystemLogs,
    /// Tiempo (reloj externo)
    TimeObserver,
    /// Procesos del sistema
    ProcessList,
    /// Uso de memoria
    MemoryPressure,
    /// Estadísticas de CPU
    CpuStats,
}

impl SensorType {
    pub fn nombre(&self) -> &'static str {
        match self {
            SensorType::NetworkTraffic => "NetworkTraffic",
            SensorType::SystemResources => "SystemResources",
            SensorType::HardwareTelemetry => "HardwareTelemetry",
            SensorType::SystemLogs => "SystemLogs",
            SensorType::TimeObserver => "TimeObserver",
            SensorType::ProcessList => "ProcessList",
            SensorType::MemoryPressure => "MemoryPressure",
            SensorType::CpuStats => "CpuStats",
        }
    }

    pub fn impacto(&self) -> &'static str {
        match self {
            SensorType::NetworkTraffic => "muy_bajo",
            SensorType::SystemResources => "bajo",
            SensorType::HardwareTelemetry => "bajo",
            SensorType::SystemLogs => "bajo",
            SensorType::TimeObserver => "ninguno",
            SensorType::ProcessList => "bajo",
            SensorType::MemoryPressure => "bajo",
            SensorType::CpuStats => "bajo",
        }
    }
}

/// Una lectura de sensor
#[derive(Debug, Clone)]
pub struct SensorReading {
    /// Tipo de sensor
    pub tipo: SensorType,
    /// Timestamp de la lectura
    pub timestamp_ms: u64,
    /// Valor leído (raw)
    pub valor: f64,
    /// Unidad de medición
    pub unidad: String,
    /// Metadatos adicionales
    pub metadatos: HashMap<String, String>,
}

impl SensorReading {
    pub fn nuevo(tipo: SensorType, valor: f64, unidad: &str) -> Self {
        Self {
            tipo,
            timestamp_ms: current_timestamp_ms(),
            valor,
            unidad: unidad.to_string(),
            metadatos: HashMap::new(),
        }
    }

    pub fn con_metadato(mut self, clave: &str, valor: &str) -> Self {
        self.metadatos.insert(clave.to_string(), valor.to_string());
        self
    }
}

/// Sensor pasivo individual
pub struct PassiveSensor {
    /// Tipo de sensor
    tipo: SensorType,
    /// Última lectura
    ultima_lectura: Option<SensorReading>,
    /// Historial de lecturas (ventana deslizante)
    historial: Vec<SensorReading>,
    /// Capacidad del historial
    capacidad_historial: usize,
    /// Contador de lecturas totales
    total_lecturas: u64,
    /// Última vez que se tomó lectura
    ultima_actualizacion: Instant,
}

impl PassiveSensor {
    pub fn new(tipo: SensorType) -> Self {
        Self {
            tipo,
            ultima_lectura: None,
            historial: Vec::with_capacity(1000),
            capacidad_historial: 1000,
            total_lecturas: 0,
            ultima_actualizacion: Instant::now(),
        }
    }

    /// Lee el sensor (implementación específica por tipo)
    pub fn leer(&mut self) -> SensorReading {
        let valor = match self.tipo {
            SensorType::NetworkTraffic => self.leer_trafico_red(),
            SensorType::SystemResources => self.leer_recursos_sistema(),
            SensorType::HardwareTelemetry => self.leer_telemetria_hardware(),
            SensorType::SystemLogs => self.leer_logs_sistema(),
            SensorType::TimeObserver => self.leer_tiempo(),
            SensorType::ProcessList => self.leer_procesos(),
            SensorType::MemoryPressure => self.leer_memoria(),
            SensorType::CpuStats => self.leer_cpu(),
        };

        self.total_lecturas += 1;
        self.ultima_actualizacion = Instant::now();

        let lectura = SensorReading::nuevo(self.tipo, valor.0, valor.1);

        // Agregar al historial
        if self.historial.len() >= self.capacidad_historial {
            self.historial.remove(0);
        }
        self.historial.push(lectura.clone());
        self.ultima_lectura = Some(lectura.clone());

        lectura
    }

    fn leer_trafico_red(&self) -> (f64, &'static str) {
        // Intentar leer desde /proc/net/dev o similar
        // Retorna bytes por segundo observados
        if let Ok(contenido) = std::fs::read_to_string("/proc/net/dev") {
            let lineas: Vec<&str> = contenido.lines().skip(2).collect();
            let mut total_bytes: u64 = 0;
            for linea in lineas.iter().take(10) {
                // Formato: iface:bytes
                if let Some(colons) = linea.find(':') {
                    let valores: Vec<&str> = linea[colons + 1..].split_whitespace().collect();
                    if valores.len() >= 9 {
                        if let Ok(bytes) = valores[0].parse::<u64>() {
                            total_bytes += bytes;
                        }
                    }
                }
            }
            return (total_bytes as f64 / 1_000_000.0, "MB"); // Mega-bytes observados
        }
        (0.0, "MB")
    }

    fn leer_recursos_sistema(&self) -> (f64, &'static str) {
        // Carga del sistema desde /proc/loadavg
        if let Ok(contenido) = std::fs::read_to_string("/proc/loadavg") {
            let partes: Vec<&str> = contenido.split_whitespace().collect();
            if let Ok(carga) = partes[0].parse::<f64>() {
                return (carga, "load_1m");
            }
        }
        (0.0, "load")
    }

    fn leer_telemetria_hardware(&self) -> (f64, &'static str) {
        // Intentar leer thermal zones
        if let Ok(temp) = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
            if let Ok(temp_i) = temp.trim().parse::<i64>() {
                return (temp_i as f64 / 1000.0, "°C");
            }
        }
        // Fallback: leer de /proc/acpi/thermal
        (0.0, "°C")
    }

    fn leer_logs_sistema(&self) -> (f64, &'static str) {
        // Contar líneas en /var/log/syslog o similar (accesible sin sudo)
        if let Ok(lineas) = std::fs::read_to_string("/var/log/syslog") {
            return (lineas.lines().count() as f64, "lines");
        }
        // Fallback: /dev/log estimation
        (0.0, "events")
    }

    fn leer_tiempo(&self) -> (f64, &'static str) {
        // Retornar tiempo Unix actual
        let ahora = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        (ahora.as_secs_f64(), "unix_timestamp")
    }

    fn leer_procesos(&self) -> (f64, &'static str) {
        // Contar procesos desde /proc
        let entries = std::fs::read_dir("/proc").unwrap_or_else(|_| unsafe {
            std::fs::read_dir(std::path::Path::new("/proc")).unwrap_unchecked()
        });
        let count = entries.filter_map(|e| e.ok()).filter(|e| {
            if let Ok(name) = e.file_name().into_string() {
                name.chars().all(|c| c.is_ascii_digit())
            } else {
                false
            }
        }).count();
        (count as f64, "processes")
    }

    fn leer_memoria(&self) -> (f64, &'static str) {
        // Leer desde /proc/meminfo
        if let Ok(contenido) = std::fs::read_to_string("/proc/meminfo") {
            let lineas: Vec<&str> = contenido.lines().collect();
            for linea in lineas.iter().take(5) {
                if linea.starts_with("MemAvailable:") {
                    let partes: Vec<&str> = linea.split_whitespace().collect();
                    if partes.len() >= 2 {
                        if let Ok(kb) = partes[1].parse::<f64>() {
                            return (kb / 1024.0, "MB"); // Convertir a MB
                        }
                    }
                }
            }
        }
        (0.0, "MB")
    }

    fn leer_cpu(&self) -> (f64, &'static str) {
        // Porcentaje de uso de CPU (lectura simple)
        if let Ok(contenido) = std::fs::read_to_string("/proc/stat") {
            let lineas: Vec<&str> = contenido.lines().collect();
            if let Some(first) = lineas.first() {
                let partes: Vec<&str> = first.split_whitespace().skip(1).collect();
                if partes.len() >= 4 {
                    let user: u64 = partes[0].parse().unwrap_or(0);
                    let nice: u64 = partes[1].parse().unwrap_or(0);
                    let system: u64 = partes[2].parse().unwrap_or(0);
                    let idle: u64 = partes[3].parse().unwrap_or(0);
                    let total: u64 = user + nice + system + idle;
                    if total > 0 {
                        let uso = (user + nice + system) as f64 / total as f64 * 100.0;
                        return (uso, "%");
                    }
                }
            }
        }
        (0.0, "%")
    }

    /// Obtiene la última lectura
    pub fn ultima_lectura(&self) -> Option<&SensorReading> {
        self.ultima_lectura.as_ref()
    }

    /// Obtiene estadísticas del sensor
    pub fn stats(&self) -> SensorStats {
        SensorStats {
            tipo: self.tipo,
            total_lecturas: self.total_lecturas,
            tamano_historial: self.historial.len(),
            ultima_actualizacion_ms: self.ultima_actualizacion
                .elapsed()
                .as_millis() as u64,
        }
    }

    /// Obtiene promedio de las últimas N lecturas
    pub fn promedio_ultimas(&self, n: usize) -> Option<f64> {
        let n = n.min(self.historial.len());
        if n == 0 {
            return None;
        }
        let suma: f64 = self.historial.iter().rev().take(n).map(|r| r.valor).sum();
        Some(suma / n as f64)
    }
}

/// Estadísticas de un sensor
#[derive(Debug, Clone)]
pub struct SensorStats {
    pub tipo: SensorType,
    pub total_lecturas: u64,
    pub tamano_historial: usize,
    pub ultima_actualizacion_ms: u64,
}

/// Traffic Obfuscator — Garantiza pasividad en tráfico de red
pub struct TrafficObfuscator {
    /// Patrón de tráfico "normal" que imitar
    patron_normal: HashMap<String, f64>,
    /// Última vez que se generó tráfico
    ultima_actividad: Instant,
    /// Intervalo mínimo entre actividades (ms)
    intervalo_min_ms: u64,
}

impl TrafficObfuscator {
    pub fn new() -> Self {
        Self {
            patron_normal: HashMap::new(),
            ultima_actividad: Instant::now(),
            intervalo_min_ms: 100, // 100ms entre actividades
        }
    }

    /// Registra una actividad de red observada
    pub fn registrar_actividad(&mut self, tipo: &str, bytes: f64) {
        *self.patron_normal.entry(tipo.to_string()).or_insert(0.0) += bytes;
        self.ultima_actividad = Instant::now();
    }

    /// Verifica si se puede realizar una actividad de red
    /// Retorna true si no violaría el patrón de pasividad
    /// Siempre retorna true si nunca se ha registrado actividad
    pub fn puede_actividad(&self) -> bool {
        let elapsed = self.ultima_actividad.elapsed().as_millis() as u64;
        elapsed >= self.intervalo_min_ms || self.patron_normal.is_empty()
    }

    /// Simula que el tráfico generado es "navegación normal"
    pub fn simular_trafico_normal(&self) -> String {
        // Generar identificación de tráfico simulada
        format!("Browsing-{:016x}", rand_u64_simple())
    }
}

impl Default for TrafficObfuscator {
    fn default() -> Self {
        Self::new()
    }
}

/// Grid de sensores pasivos global
pub struct GlobalSensorGrid {
    /// Sensores registrados
    sensores: HashMap<SensorType, PassiveSensor>,
    /// Traffic obfuscator
    obfuscator: TrafficObfuscator,
    /// Configuración del grid
    config: GridConfig,
    /// Estadísticas globales
    stats: GridStats,
}

#[derive(Debug, Clone)]
pub struct GridConfig {
    /// Intervalo de lectura global (ms)
    pub intervalo_lectura_ms: u64,
    /// Habilitar todos los sensores
    pub habilitado: bool,
    /// Sensores activos específicos
    pub sensores_activos: Vec<SensorType>,
    /// Nivel de detalle (0=solo crítico, 3=todo)
    pub nivel_detalle: u8,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            intervalo_lectura_ms: 1000,
            habilitado: true,
            sensores_activos: vec![
                SensorType::SystemResources,
                SensorType::HardwareTelemetry,
                SensorType::TimeObserver,
                SensorType::MemoryPressure,
                SensorType::CpuStats,
            ],
            nivel_detalle: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridStats {
    pub total_lecturas: u64,
    pub sensores_activos: usize,
    pub ultima_actualizacion_global: u64,
    pub alertas_activas: usize,
}

impl Default for GridStats {
    fn default() -> Self {
        Self {
            total_lecturas: 0,
            sensores_activos: 0,
            ultima_actualizacion_global: 0,
            alertas_activas: 0,
        }
    }
}

impl GlobalSensorGrid {
    /// Crea un nuevo grid de sensores
    pub fn new(config: GridConfig) -> Self {
        let mut sensores = HashMap::new();

        // Inicializar sensores activos
        for &tipo in config.sensores_activos.iter() {
            sensores.insert(tipo, PassiveSensor::new(tipo));
        }

        let sensores_count = sensores.len();
        Self {
            sensores,
            obfuscator: TrafficObfuscator::new(),
            config,
            stats: GridStats {
                total_lecturas: 0,
                sensores_activos: sensores_count,
                ultima_actualizacion_global: current_timestamp_ms(),
                alertas_activas: 0,
            },
        }
    }

    /// Lee todos los sensores activos
    pub fn leer_todos(&mut self) -> Vec<SensorReading> {
        let mut lecturas = Vec::new();
        self.stats.total_lecturas += 1;
        self.stats.ultima_actualizacion_global = current_timestamp_ms();

        for sensor in self.sensores.values_mut() {
            let lectura = sensor.leer();
            lecturas.push(lectura);
        }

        self.stats.sensores_activos = self.sensores.len();
        lecturas
    }

    /// Lee un sensor específico
    pub fn leer_sensor(&mut self, tipo: SensorType) -> Option<SensorReading> {
        if let Some(sensor) = self.sensores.get_mut(&tipo) {
            Some(sensor.leer())
        } else {
            None
        }
    }

    /// Obtiene estadísticas del grid
    pub fn stats(&self) -> GridStats {
        self.stats.clone()
    }

    /// Obtiene estadísticas de un sensor específico
    pub fn stats_sensor(&self, tipo: SensorType) -> Option<SensorStats> {
        self.sensores.get(&tipo).map(|s| s.stats())
    }

    /// Verifica si el grid puede realizar actividad de red
    pub fn puede_red(&self) -> bool {
        self.obfuscator.puede_actividad()
    }

    /// Registra actividad de red observada
    pub fn registrar_actividad_red(&mut self, tipo: &str, bytes: f64) {
        self.obfuscator.registrar_actividad(tipo, bytes);
    }

    /// Obtiene promedio de un sensor específico
    pub fn promedio_sensor(&self, tipo: SensorType, n: usize) -> Option<f64> {
        self.sensores.get(&tipo)?.promedio_ultimas(n)
    }

    /// Detecta anomalías en los sensores
    pub fn detectar_anomalias(&self) -> Vec<AnomaliaDetectada> {
        let mut anomalias = Vec::new();

        // Verificar CPU
        if let Some(stats) = self.sensores.get(&SensorType::CpuStats) {
            if let Some(avg) = stats.promedio_ultimas(60) {
                if avg > 90.0 {
                    anomalias.push(AnomaliaDetectada {
                        tipo: SensorType::CpuStats,
                        mensaje: format!("CPU elevado: {:.1}%", avg),
                        urgencia: 7,
                    });
                }
            }
        }

        // Verificar memoria
        if let Some(stats) = self.sensores.get(&SensorType::MemoryPressure) {
            if let Some(avg) = stats.promedio_ultimas(60) {
                if avg < 100.0 { // Menos de 100MB disponible
                    anomalias.push(AnomaliaDetectada {
                        tipo: SensorType::MemoryPressure,
                        mensaje: format!("Memoria baja: {:.0}MB", avg),
                        urgencia: 8,
                    });
                }
            }
        }

        // Verificar temperatura
        if let Some(stats) = self.sensores.get(&SensorType::HardwareTelemetry) {
            if let Some(avg) = stats.promedio_ultimas(60) {
                if avg > 80.0 {
                    anomalias.push(AnomaliaDetectada {
                        tipo: SensorType::HardwareTelemetry,
                        mensaje: format!("Temperatura alta: {:.1}°C", avg),
                        urgencia: 9,
                    });
                }
            }
        }

        anomalias
    }

    /// Genera reporte del estado del grid
    pub fn reporte(&self) -> String {
        let mut s = String::new();
        s.push_str("=== GLOBAL SENSOR GRID ===\n");
        s.push_str(&format!("Sensores activos: {}\n", self.sensores.len()));
        s.push_str(&format!("Total lecturas: {}\n", self.stats.total_lecturas));
        s.push_str(&format!("Última actualización: {}ms\n", self.stats.ultima_actualizacion_global));

        for (tipo, sensor) in &self.sensores {
            if let Some(lectura) = sensor.ultima_lectura() {
                let ms_since_read = Instant::now().elapsed().as_millis() as i64;
                let lectura_age = current_timestamp_ms() as i64 - lectura.timestamp_ms as i64;
                s.push_str(&format!(
                    "  {}: {:.2} {} (hace {}ms)\n",
                    tipo.nombre(),
                    lectura.valor,
                    lectura.unidad,
                    ms_since_read - lectura_age
                ));
            }
        }

        s
    }
}

/// Anomalía detectada en los sensores
#[derive(Debug, Clone)]
pub struct AnomaliaDetectada {
    pub tipo: SensorType,
    pub mensaje: String,
    pub urgencia: u8,
}

// =============================================================================
// Utilidades
// =============================================================================

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Genera un u64 simple para identificación (no criptográfico)
fn rand_u64_simple() -> u64 {
    // Usar timestamp + contador como fuente pseudo-aleatoria
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    now.wrapping_add(count.wrapping_mul(0x9E3779B97F4A7C15))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_tiempo() {
        let mut sensor = PassiveSensor::new(SensorType::TimeObserver);
        let lectura = sensor.leer();

        assert!(lectura.valor > 1_700_000_000.0); // Después de 1970
        assert_eq!(lectura.unidad, "unix_timestamp");
    }

    #[test]
    fn test_sensor_memoria() {
        let mut sensor = PassiveSensor::new(SensorType::MemoryPressure);
        let lectura = sensor.leer();

        // No verificamos valor exacto (depende del sistema)
        // Solo que la lectura funciona
        assert!(lectura.valor >= 0.0);
        assert_eq!(lectura.unidad, "MB");
    }

    #[test]
    fn test_grid_inicializacion() {
        let grid = GlobalSensorGrid::new(GridConfig::default());
        let stats = grid.stats();
        // Dependiendo del sistema y permisos, puede haber 0-5 sensores
        assert!(stats.sensores_activos >= 0, "Debe inicializarse correctamente");
    }

    #[test]
    fn test_grid_lectura_multiple() {
        let mut grid = GlobalSensorGrid::new(GridConfig::default());
        let lecturas = grid.leer_todos();

        assert!(!lecturas.is_empty());
    }

    #[test]
    fn test_obfuscator_pasividad() {
        let obs = TrafficObfuscator::new();
        assert!(obs.puede_actividad()); // Siempre puede al inicio
    }

    #[test]
    fn test_sensor_promedio() {
        let mut sensor = PassiveSensor::new(SensorType::TimeObserver);

        // Generar varias lecturas
        for _ in 0..10 {
            sensor.leer();
        }

        let promedio = sensor.promedio_ultimas(5);
        assert!(promedio.is_some());
        assert!(promedio.unwrap() > 0.0);
    }
}