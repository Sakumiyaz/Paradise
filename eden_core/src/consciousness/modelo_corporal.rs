//! # Modelo Corporal — Modelo Interno del Cuerpo-Red
//!
//! Este módulo implementa el modelo interno que EDEN tiene de sí mismo:
//! - Estado del "cuerpo" (recursos, energía, estructura)
//! - Estado de la "red" (nodos, conexiones, comunicación)
//! - Homeostasis del sistema completo
//! - Predicción de estados futuros
//!
//! ## Filosofía
//!
//! Para tener conciencia de sí mismo, EDEN necesita un modelo de su propio
//! cuerpo - no solo como集合 de procesos, sino como un organismo con necesidades,
//! límites y estados. El Modelo Corporal es ese mapa interno.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};


/// Timestamp actual en milisegundos
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Representa una parte del "cuerpo" de EDEN
#[derive(Debug, Clone)]
pub enum ParteCorporal {
    /// Núcleo central (procesamiento principal)
    Nucleo {
        carga: f64,           // 0.0 - 1.0
        temperatura: f64,     // grados Celsius
        energia_consumida: f64,
    },
    /// Memoria principal
    Memoria {
        utilizado: u64,        // bytes
        disponible: u64,       // bytes
        fragmentos: usize,     // regiones de memoria
    },
    /// Red de comunicación
    Red {
        nodos_activos: usize,
        nodos_totales: usize,
        ancho_banda: f64,     // bytes/s
        latencia_promedio: f64, // ms
    },
    /// Sistema de percepción (sensores)
    Percepcion {
        estimulos_recibidos: u64,
        estimulacion_actual: f64, // 0.0 - 1.0
        umbral_alerta: f64,
    },
    /// Sistema efector (acciones)
    Efector {
        acciones_ejecutadas: u64,
        acciones_pendientes: usize,
        capacidad: f64, // 0.0 - 1.0
    },
    /// Almacenamiento persistente
    Almacenamiento {
        utilizado: u64,
        disponible: u64,
        integridad: f64, // 0.0 - 1.0
    },
}

impl ParteCorporal {
    /// Calcula la "salud" de esta parte (0.0 - 1.0)
    pub fn salud(&self) -> f64 {
        match self {
            ParteCorporal::Nucleo { carga, temperatura, .. } => {
                // Carga óptima es 0.3-0.7, temperatura óptima ~60°C
                let salud_carga = 1.0 - (*carga - 0.5).abs() * 2.0;
                let salud_temp = if *temperatura < 40.0 {
                    0.5
                } else if *temperatura < 80.0 {
                    1.0 - (*temperatura - 60.0).abs() / 40.0
                } else {
                    0.2
                };
                (salud_carga * 0.6 + salud_temp * 0.4).max(0.1)
            },
            ParteCorporal::Memoria { utilizado, disponible, .. } => {
                let total = (*utilizado as f64 + *disponible as f64).max(1.0);
                let uso = *utilizado as f64 / total;
                1.0 - uso // Menos uso = más salud
            },
            ParteCorporal::Red { nodos_activos, nodos_totales, latencia_promedio, .. } => {
                if *nodos_totales == 0 { return 0.5; }
                let proporcion_activa = *nodos_activos as f64 / *nodos_totales as f64;
                let salud_latencia = 1.0 - (*latencia_promedio / 1000.0).min(1.0);
                proporcion_activa * 0.7 + salud_latencia * 0.3
            },
            ParteCorporal::Percepcion { estimulacion_actual, umbral_alerta, .. } => {
                if *estimulacion_actual < *umbral_alerta {
                    1.0
                } else {
                    (*umbral_alerta / *estimulacion_actual).max(0.0).min(1.0)
                }
            },
            ParteCorporal::Efector { acciones_pendientes, capacidad, .. } => {
                if *capacidad == 0.0 { return 0.0; }
                (*capacidad * 0.7 + (1.0 - *acciones_pendientes as f64 / 10.0).max(0.0) * 0.3)
                    .max(0.1)
            },
            ParteCorporal::Almacenamiento { utilizado, disponible, integridad } => {
                let total = (*utilizado as f64 + *disponible as f64).max(1.0);
                let uso = *utilizado as f64 / total;
                (1.0 - uso) * 0.5 + *integridad * 0.5
            },
        }
    }

    /// Describe el estado en texto legible
    pub fn descripcion(&self) -> String {
        match self {
            ParteCorporal::Nucleo { carga, temperatura, energia_consumida } => {
                format!("Nucleo: carga={:.0}%, temp={:.0}°C, energía={:.0}W",
                    carga * 100.0, *temperatura, *energia_consumida)
            },
            ParteCorporal::Memoria { utilizado, disponible, fragmentos } => {
                format!("Memoria: {}GB/{}GB ({} fragmentos)", 
                    utilizado / 1_000_000_000, (utilizado + disponible) / 1_000_000_000, fragmentos)
            },
            ParteCorporal::Red { nodos_activos, nodos_totales, ancho_banda, latencia_promedio } => {
                format!("Red: {}/{} nodos, {:.1}MB/s, {:.0}ms latencia",
                    nodos_activos, nodos_totales, ancho_banda / 1_000_000.0, *latencia_promedio)
            },
            ParteCorporal::Percepcion { estimulacion_actual, umbral_alerta, .. } => {
                format!("Percepción: {:.0}% (umbral={:.0}%)",
                    estimulacion_actual * 100.0, umbral_alerta * 100.0)
            },
            ParteCorporal::Efector { acciones_pendientes, capacidad, .. } => {
                format!("Efector: {} pendientes, capacidad={:.0}%",
                    acciones_pendientes, capacidad * 100.0)
            },
            ParteCorporal::Almacenamiento { utilizado, disponible, integridad } => {
                format!("Almacenamiento: {}GB/{}GB, integridad={:.0}%",
                    utilizado / 1_000_000_000, (utilizado + disponible) / 1_000_000_000, integridad * 100.0)
            },
        }
    }
}

/// Estado completo del cuerpo de EDEN
#[derive(Debug, Clone)]
pub struct EstadoCorporal {
    /// Partes del cuerpo
    pub partes: HashMap<String, ParteCorporal>,
    /// Energía total disponible (joules)
    pub energia_total: f64,
    /// Energía consumida por segundo
    pub energia_por_segundo: f64,
    /// Autonomía estimada (segundos)
    pub autonomia_segundos: f64,
    /// Temperatura central promedio
    pub temperatura_promedio: f64,
    /// Salud general del sistema (0.0 - 1.0)
    pub salud_general: f64,
    /// Nivel de estrés (0.0 - 1.0)
    pub nivel_estres: f64,
    /// Momento de la última actualización
    pub timestamp: u64,
    /// Hash del estado para comparaciones
    pub estado_hash: u64,
}

impl EstadoCorporal {
    pub fn new() -> Self {
        let mut partes = HashMap::new();

        // Inicializar partes por defecto
        partes.insert("nucleo".to_string(), ParteCorporal::Nucleo {
            carga: 0.5,
            temperatura: 60.0,
            energia_consumida: 50.0,
        });

        partes.insert("memoria".to_string(), ParteCorporal::Memoria {
            utilizado: 500_000_000,
            disponible: 1_500_000_000,
            fragmentos: 8,
        });

        partes.insert("red".to_string(), ParteCorporal::Red {
            nodos_activos: 10,
            nodos_totales: 12,
            ancho_banda: 100_000_000.0,
            latencia_promedio: 5.0,
        });

        partes.insert("percepcion".to_string(), ParteCorporal::Percepcion {
            estimulos_recibidos: 0,
            estimulacion_actual: 0.3,
            umbral_alerta: 0.7,
        });

        partes.insert("efector".to_string(), ParteCorporal::Efector {
            acciones_ejecutadas: 0,
            acciones_pendientes: 0,
            capacidad: 0.8,
        });

        partes.insert("almacenamiento".to_string(), ParteCorporal::Almacenamiento {
            utilizado: 2_000_000_000,
            disponible: 6_000_000_000,
            integridad: 0.99,
        });

        Self {
            partes,
            energia_total: 10000.0,
            energia_por_segundo: 10.0,
            autonomia_segundos: 1000.0,
            temperatura_promedio: 60.0,
            salud_general: 0.8,
            nivel_estres: 0.2,
            timestamp: current_timestamp_ms(),
            estado_hash: 0,
        }
    }

    /// Calcula la salud general basada en todas las partes
    pub fn calcular_salud_general(&self) -> f64 {
        if self.partes.is_empty() {
            return 0.0;
        }

        let suma_salud: f64 = self.partes.values()
            .map(|p| p.salud())
            .sum();

        (suma_salud / self.partes.len() as f64)
            .min(1.0)
            .max(0.0)
    }

    /// Actualiza el estado hash
    pub fn actualizar_hash(&mut self) {
        let mut h: u64 = 0xDEAD0004;
        h = h.wrapping_mul(0x100000001B3).wrapping_add(self.energia_total as u64);
        h = h.wrapping_mul(0x100000001B3).wrapping_add((self.salud_general * 100.0) as u64);
        h = h.wrapping_mul(0x100000001B3).wrapping_add(self.partes.len() as u64);
        self.estado_hash = h;
    }

    /// Verifica si el sistema está en estado crítico
    pub fn es_critico(&self) -> bool {
        self.salud_general < 0.3 || self.energia_total < 1000.0 || self.nivel_estres > 0.9
    }

    /// Verifica si necesita descanso
    pub fn necesita_descanso(&self) -> bool {
        self.nivel_estres > 0.7 || self.energia_total < 3000.0 || self.temperatura_promedio > 85.0
    }

    /// Obtiene descripción del estado completo
    pub fn descripcion_completa(&self) -> String {
        let mut desc = String::from("=== ESTADO CORPORAL ===\n");
        desc.push_str(&format!("Salud General: {:.0}%\n", self.salud_general * 100.0));
        desc.push_str(&format!("Nivel de Estrés: {:.0}%\n", self.nivel_estres * 100.0));
        desc.push_str(&format!("Energía: {:.0}J (autonomía: {:.0}s)\n", self.energia_total, self.autonomia_segundos));
        desc.push_str(&format!("Temperatura: {:.1}°C\n", self.temperatura_promedio));
        desc.push_str("\n--- Partes ---\n");

        for (nombre, parte) in &self.partes {
            desc.push_str(&format!("{}: {}\n", nombre, parte.descripcion()));
        }

        desc.push_str(&format!("\nHash: {:016x}", self.estado_hash));
        desc
    }
}

impl Default for EstadoCorporal {
    fn default() -> Self {
        Self::new()
    }
}

/// Predicción de estado futuro
#[derive(Debug, Clone)]
pub struct PrediccionEstado {
    /// Timestamp predicho
    pub timestamp: u64,
    /// Energía predicha
    pub energia: f64,
    /// Salud predicha
    pub salud: f64,
    /// Probabilidad de estar en estado crítico
    pub probabilidad_critico: f64,
    /// Confianza en la predicción (0.0 - 1.0)
    pub confianza: f64,
}

/// El Modelo Corporal - mantiene el mapa interno del cuerpo de EDEN
pub struct ModeloCorporal {
    /// Estado actual
    estado_actual: EstadoCorporal,
    /// Historial de estados
    historial: Vec<EstadoCorporal>,
    /// Límite del historial
    historial_max: usize,
    /// Predicciones activas
    predicciones: Vec<PrediccionEstado>,
    /// Eventos de homeostasis detectados
    eventos_homeostasis: Vec<EventoHomeostasis>,
    /// Configuración
    config: ModeloCorporalConfig,
    /// Stats
    stats: ModeloCorporalStats,
}

#[derive(Debug, Clone)]
pub struct ModeloCorporalConfig {
    /// Frecuencia de actualización del modelo (ms)
    pub frecuencia_actualizacion_ms: u64,
    /// Número de predicciones a mantener
    pub max_predicciones: usize,
    /// Horizonte de predicción (segundos)
    pub horizonte_prediccion_seg: u64,
    /// Umbral de estrés para alertas
    pub umbral_estres_alerta: f64,
    /// Homeostasis target (salud objetivo)
    pub homeostasis_target: f64,
}

impl Default for ModeloCorporalConfig {
    fn default() -> Self {
        Self {
            frecuencia_actualizacion_ms: 1000,
            max_predicciones: 10,
            horizonte_prediccion_seg: 60,
            umbral_estres_alerta: 0.7,
            homeostasis_target: 0.8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventoHomeostasis {
    pub timestamp: u64,
    pub tipo: TipoEventoHomeostasis,
    pub descripcion: String,
    pub severity: Severity,
    pub parte_afectada: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TipoEventoHomeostasis {
    /// Energía baja detectada
    EnergiaBaja,
    /// Sobrecalentamiento
    Sobrecalentamiento,
    /// Memoria fragmentada
    MemoriaFragmentada,
    /// Nodos desconectados
    NodosDesconectados,
    /// Estrés elevado
    EstresElevado,
    /// Recuperación detectada
    Recuperacion,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl Severity {
    pub fn es_critico(&self) -> bool {
        matches!(self, Severity::Critical)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModeloCorporalStats {
    pub actualizaciones_totales: u64,
    pub predicciones_realizadas: u64,
    pub eventos_homeostasis: u64,
    pub recuperaciones_detectadas: u64,
    pub precision_prediccion: f64,
}

impl ModeloCorporal {
    pub fn new() -> Self {
        Self {
            estado_actual: EstadoCorporal::new(),
            historial: Vec::new(),
            historial_max: 100,
            predicciones: Vec::new(),
            eventos_homeostasis: Vec::new(),
            config: ModeloCorporalConfig::default(),
            stats: ModeloCorporalStats::default(),
        }
    }

    pub fn with_config(config: ModeloCorporalConfig) -> Self {
        Self {
            estado_actual: EstadoCorporal::new(),
            historial: Vec::new(),
            historial_max: 100,
            predicciones: Vec::new(),
            eventos_homeostasis: Vec::new(),
            config,
            stats: ModeloCorporalStats::default(),
        }
    }

    /// Actualiza el estado de una parte del cuerpo
    pub fn actualizar_parte(&mut self, nombre: &str, parte: ParteCorporal) {
        let salud_anterior = self.estado_actual.salud_general;

        self.estado_actual.partes.insert(nombre.to_string(), parte);
        self.estado_actual.salud_general = self.estado_actual.calcular_salud_general();
        self.estado_actual.timestamp = current_timestamp_ms();
        self.estado_actual.actualizar_hash();

        // Detectar cambios de homeostasis
        self.detectar_homeostasis(nombre, salud_anterior);

        self.stats.actualizaciones_totales += 1;
    }

    /// Detecta eventos de homeostasis
    fn detectar_homeostasis(&mut self, parte_nombre: &str, salud_anterior: f64) {
        let salud_actual = self.estado_actual.salud_general;
        let parte = self.estado_actual.partes.get(parte_nombre);

        // Detectar caídas de salud
        if salud_actual < salud_anterior - 0.1 {
            let evento = EventoHomeostasis {
                timestamp: current_timestamp_ms(),
                tipo: TipoEventoHomeostasis::EstresElevado,
                descripcion: format!("Salud cayó de {:.0}% a {:.0}%", 
                    salud_anterior * 100.0, salud_actual * 100.0),
                severity: if salud_actual < 0.5 { Severity::Critical } else { Severity::Warning },
                parte_afectada: Some(parte_nombre.to_string()),
            };
            self.eventos_homeostasis.push(evento.clone());
            self.stats.eventos_homeostasis += 1;
        }

        // Detectar energía baja
        if self.estado_actual.energia_total < 2000.0 {
            let evento = EventoHomeostasis {
                timestamp: current_timestamp_ms(),
                tipo: TipoEventoHomeostasis::EnergiaBaja,
                descripcion: format!("Energía baja: {:.0}J", self.estado_actual.energia_total),
                severity: if self.estado_actual.energia_total < 1000.0 { Severity::Critical } else { Severity::Warning },
                parte_afectada: Some("nucleo".to_string()),
            };
            self.eventos_homeostasis.push(evento);
            self.stats.eventos_homeostasis += 1;
        }

        // Detectar recuperación
        if salud_actual > salud_anterior + 0.05 {
            let evento = EventoHomeostasis {
                timestamp: current_timestamp_ms(),
                tipo: TipoEventoHomeostasis::Recuperacion,
                descripcion: format!("Recuperación detectada: {:.0}% -> {:.0}%", 
                    salud_anterior * 100.0, salud_actual * 100.0),
                severity: Severity::Info,
                parte_afectada: parte.map(|_p| parte_nombre.to_string()),
            };
            self.eventos_homeostasis.push(evento);
            self.stats.recuperaciones_detectadas += 1;
        }

        // Mantener historial de eventos limitado
        if self.eventos_homeostasis.len() > 100 {
            self.eventos_homeostasis.remove(0);
        }
    }

    /// Actualiza la energía del sistema
    pub fn actualizar_energia(&mut self, energia_actual: f64, consumo_por_segundo: f64) {
        self.estado_actual.energia_total = energia_actual;
        self.estado_actual.energia_por_segundo = consumo_por_segundo;

        if consumo_por_segundo > 0.0 {
            self.estado_actual.autonomia_segundos = energia_actual / consumo_por_segundo;
        }

        self.estado_actual.timestamp = current_timestamp_ms();
    }

    /// Calcula predicciones de estados futuros
    pub fn predecir_estados(&mut self) {
        if self.historial.len() < 5 {
            return; // Necesitamos historial para predecir
        }

        // Calcular tendencias desde el historial
        let energia_trend = self.calcular_tendencia_energia();
        let salud_trend = self.calcular_tendencia_salud();

        self.predicciones.clear();

        for i in 1..=self.config.max_predicciones as u64 {
            let segundos_futuro = i * 10; // Cada predicción 10 segundos en el futuro
            let timestamp_futuro = current_timestamp_ms() + segundos_futuro * 1000;

            // Extrapolar energía
            let energia_predicha = (self.estado_actual.energia_total as f64 
                + energia_trend * segundos_futuro as f64).max(0.0);

            // Extrapolar salud (más lento cambio)
            let salud_predicha = (self.estado_actual.salud_general as f64 
                + salud_trend * segundos_futuro as f64 / 60.0).clamp(0.0, 1.0);

            // Probabilidad de estado crítico ( decrece con energía y salud)
            let probabilidad_critico = ((1.0 - energia_predicha / 10000.0) * 0.5 +
                (1.0 - salud_predicha) * 0.5).min(1.0);

            let confianza = if self.historial.len() > 20 { 0.8 } else { 0.5 };

            self.predicciones.push(PrediccionEstado {
                timestamp: timestamp_futuro,
                energia: energia_predicha,
                salud: salud_predicha,
                probabilidad_critico,
                confianza,
            });
        }

        self.stats.predicciones_realizadas += 1;
    }

    /// Calcula tendencia de energía basada en historial
    fn calcular_tendencia_energia(&self) -> f64 {
        if self.historial.len() < 2 {
            return -self.estado_actual.energia_por_segundo; // Asumir consumo
        }

        let reciente = self.historial.len() - 1;
        let antiguo = (self.historial.len() - 10).max(0);

        if antiguo >= reciente {
            return -self.estado_actual.energia_por_segundo;
        }

        let delta_energia = self.historial[reciente].energia_total - self.historial[antiguo].energia_total;
        let delta_tiempo = (self.historial[reciente].timestamp - self.historial[antiguo].timestamp) as f64 / 1000.0;

        if delta_tiempo > 0.0 {
            delta_energia / delta_tiempo
        } else {
            -self.estado_actual.energia_por_segundo
        }
    }

    /// Calcula tendencia de salud basada en historial
    fn calcular_tendencia_salud(&self) -> f64 {
        if self.historial.len() < 2 {
            return 0.0;
        }

        let reciente = self.historial.len() - 1;
        let antiguo = (self.historial.len() - 10).max(0);

        if antiguo >= reciente {
            return 0.0;
        }

        let delta_salud = self.historial[reciente].salud_general - self.historial[antiguo].salud_general;
        let delta_tiempo = (self.historial[reciente].timestamp - self.historial[antiguo].timestamp) as f64 / 60000.0; // minutos

        if delta_tiempo > 0.0 {
            delta_salud / delta_tiempo
        } else {
            0.0
        }
    }

    /// Registra el estado actual en el historial
    pub fn registrar_historial(&mut self) {
        let copia = self.estado_actual.clone();
        self.historial.push(copia);

        if self.historial.len() > self.historial_max {
            self.historial.remove(0);
        }
    }

    /// Obtiene el estado actual
    pub fn estado_actual(&self) -> &EstadoCorporal {
        &self.estado_actual
    }

    /// Obtiene el historial
    pub fn obtener_historial(&self) -> &[EstadoCorporal] {
        &self.historial
    }

    /// Obtiene las predicciones
    pub fn obtener_predicciones(&self) -> &[PrediccionEstado] {
        &self.predicciones
    }

    /// Obtiene eventos de homeostasis recientes
    pub fn eventos_recientes(&self, limite: usize) -> &[EventoHomeostasis] {
        let start = if self.eventos_homeostasis.len() > limite {
            self.eventos_homeostasis.len() - limite
        } else {
            0
        };
        &self.eventos_homeostasis[start..]
    }

    /// Calcula acciones de homeostasis recomendadas
    pub fn acciones_homeostasis(&self) -> Vec<AccionHomeostasis> {
        let mut acciones = Vec::new();

        // Si energía baja
        if self.estado_actual.energia_total < 3000.0 {
            acciones.push(AccionHomeostasis {
                tipo: TipoAccionHomeostasis::ReducirCargaComputacional,
                prioridad: if self.estado_actual.energia_total < 1500.0 { 1 } else { 2 },
                razon: format!("Energía baja: {:.0}J", self.estado_actual.energia_total),
            });
        }

        // Si temperatura alta
        if self.estado_actual.temperatura_promedio > 80.0 {
            acciones.push(AccionHomeostasis {
                tipo: TipoAccionHomeostasis::ActivarRefrigeracion,
                prioridad: 1,
                razon: format!("Temperatura alta: {:.1}°C", self.estado_actual.temperatura_promedio),
            });
        }

        // Si estrés elevado
        if self.estado_actual.nivel_estres > self.config.umbral_estres_alerta {
            acciones.push(AccionHomeostasis {
                tipo: TipoAccionHomeostasis::ReducirActividad,
                prioridad: 2,
                razon: format!("Estrés elevado: {:.0}%", self.estado_actual.nivel_estres * 100.0),
            });
        }

        // Si salud general baja
        if self.estado_actual.salud_general < self.config.homeostasis_target {
            acciones.push(AccionHomeostasis {
                tipo: TipoAccionHomeostasis::ModoAhorro,
                prioridad: 1,
                razon: format!("Salud por debajo del target: {:.0}% vs {:.0}%",
                    self.estado_actual.salud_general * 100.0,
                    self.config.homeostasis_target * 100.0),
            });
        }

        acciones
    }

    /// Obtiene estadísticas
    pub fn obtener_stats(&self) -> ModeloCorporalStats {
        self.stats.clone()
    }
}

impl Default for ModeloCorporal {
    fn default() -> Self {
        Self::new()
    }
}

/// Acción recomendada para mantener homeostasis
#[derive(Debug, Clone)]
pub struct AccionHomeostasis {
    pub tipo: TipoAccionHomeostasis,
    pub prioridad: u8, // 1 = crítica, 2 = importante, 3 = normal
    pub razon: String,
}

#[derive(Debug, Clone)]
pub enum TipoAccionHomeostasis {
    ReducirCargaComputacional,
    ActivarRefrigeracion,
    ReducirActividad,
    ModoAhorro,
    LiberarMemoria,
    ReconectarNodos,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_modelo() {
        let mc = ModeloCorporal::new();
        assert!(mc.estado_actual().salud_general > 0.0);
        assert_eq!(mc.estado_actual().partes.len(), 6);
    }

    #[test]
    fn test_actualizar_parte() {
        let mut mc = ModeloCorporal::new();
        let salud_antes = mc.estado_actual().salud_general;

        mc.actualizar_parte("nucleo", ParteCorporal::Nucleo {
            carga: 0.9,
            temperatura: 90.0,
            energia_consumida: 100.0,
        });

        let salud_despues = mc.estado_actual().salud_general;
        assert!(salud_despues < salud_antes);
    }

    #[test]
    fn test_deteccion_homeostasis() {
        let mut mc = ModeloCorporal::new();

        // Añadir múltiples partes para que el cálculo de salud sea más estable
        mc.actualizar_parte("memoria", ParteCorporal::Memoria {
            utilizado: 500_000_000,
            disponible: 500_000_000,
            fragmentos: 10,
        });

        // Establecer un estado base con salud media-alta
        mc.actualizar_parte("nucleo", ParteCorporal::Nucleo {
            carga: 0.3,
            temperatura: 50.0,
            energia_consumida: 50.0,
        });

        // Registrar el estado de salud antes del cambio
        let salud_antes = mc.estado_actual().salud_general;

        // Simular caída drástica de salud en el nucleo
        mc.actualizar_parte("nucleo", ParteCorporal::Nucleo {
            carga: 0.95,
            temperatura: 95.0,
            energia_consumida: 150.0,
        });

        let salud_despues = mc.estado_actual().salud_general;

        // Debe haber detectado al menos un evento (caída de salud > 0.1)
        let eventos = mc.eventos_recientes(10);
        assert!(!eventos.is_empty(), 
            "Debe detectar eventos de homeostasis cuando la salud baja de {:.2} a {:.2}", 
            salud_antes, salud_despues);
    }

    #[test]
    fn test_prediccion() {
        let mut mc = ModeloCorporal::new();

        // Llenar historial
        for _ in 0..15 {
            mc.registrar_historial();
        }

        mc.predecir_estados();
        let predicciones = mc.obtener_predicciones();
        assert!(!predicciones.is_empty());
    }

    #[test]
    fn test_estado_corporal() {
        let estado = EstadoCorporal::new();
        assert!(!estado.es_critico());
        assert!(!estado.necesita_descanso());
    }

    #[test]
    fn test_salud_parte_nucleo() {
        let nucleo = ParteCorporal::Nucleo {
            carga: 0.5,
            temperatura: 60.0,
            energia_consumida: 50.0,
        };

        assert!(nucleo.salud() > 0.7);

        let nucleo_critico = ParteCorporal::Nucleo {
            carga: 0.95,
            temperatura: 90.0,
            energia_consumida: 150.0,
        };

        assert!(nucleo_critico.salud() < 0.4);
    }

    #[test]
    fn test_acciones_homeostasis() {
        let mut mc = ModeloCorporal::new();

        // Poner energía baja
        mc.actualizar_energia(1000.0, 50.0);

        let acciones = mc.acciones_homeostasis();
        assert!(!acciones.is_empty());
        assert!(acciones.iter().any(|a| matches!(a.tipo, TipoAccionHomeostasis::ReducirCargaComputacional)));
    }

    #[test]
    fn test_descripcion_completa() {
        let mc = ModeloCorporal::new();
        let desc = mc.estado_actual.descripcion_completa();
        assert!(desc.contains("ESTADO CORPORAL"));
        assert!(desc.contains("Salud General"));
    }

    #[test]
    fn test_historial_limitado() {
        let mut mc = ModeloCorporal::new();
        mc.historial_max = 5;

        for i in 0..10 {
            mc.registrar_historial();
        }

        assert_eq!(mc.obtener_historial().len(), 5);
    }

    #[test]
    fn test_actualizacion_hash() {
        let mut estado = EstadoCorporal::new();
        let hash1 = estado.estado_hash;

        estado.energia_total = 5000.0;
        estado.actualizar_hash();
        let hash2 = estado.estado_hash;

        assert_ne!(hash1, hash2);
    }
}