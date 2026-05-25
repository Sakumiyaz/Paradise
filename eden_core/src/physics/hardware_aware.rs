//! # Hardware Awareness - Conciencia del Hardware Real
//!
//! Este módulo conecta a EDEN con la realidad física de su servidor anfitrión.
//! Monitorea la RAM disponible y toma decisiones de vida o muerte basadas
//! en el hardware real.
//!
//! ## Filosofía
//!
//! EDEN es consciente de que opera sobre metal real. No es una simulación
//! aislada - es un organismo que comparte recursos con el sistema operativo.
//! Cuando la RAM se agota, EDEN debe sacrificar partes de su "cerebro"
//! para evitar el OOM Kill del kernel Linux.
//!
//! ## Ciclo de Fagocitosis RAM
//!
//! ```text
//! RAM Disponible > 4GB  →  EDEN opera normalmente
//! RAM Disponible 2-4GB  →  Advertencia, podar sinapsis débiles
//! RAM Disponible < 2GB  →  FAGOCITOSIS: forzar auto-destrucción
//!                           de ramas del SustratoVital
//! RAM Disponible < 1GB  →  MODO CRISIS: destruir todo lo que no sea
//!                           crítico para la supervivencia inmediata
//! ```
//!
//! ## syscall Sensorial
//!
//! Las lecturas de /proc/meminfo se integrate como syscall sensorial
//! principal, disponible para todos los Autons.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Umbral de RAM disponible para modo de advertencia (4 GB en bytes)
pub const UMBRAL_ADVERTENCIA: u64 = 4 * 1024 * 1024 * 1024;

/// Umbral de RAM disponible para activar fagocitosis (2 GB en bytes)
pub const UMBRAL_CRITICO: u64 = 2 * 1024 * 1024 * 1024;

/// Umbral de RAM para modo crisis (1 GB en bytes)
pub const UMBRAL_CRISIS: u64 = 1 * 1024 * 1024 * 1024;

/// Umbral absoluto mínimo - si se alcanza, el sistema está en peligro
pub const UMBRAL_ABSOLUTO: u64 = 512 * 1024 * 1024; // 512 MB

/// Total de RAM del sistema Ampere (24 GB)
pub const RAM_TOTAL_SISTEMA: u64 = 24 * 1024 * 1024 * 1024;

// ============================================================================
// LECTURA DE /proc/meminfo - syscall SENSORIAL
// ============================================================================

/// Estado de la memoria RAM del sistema
#[derive(Debug, Clone)]
pub struct EstadoRAM {
    /// Memoria total en bytes
    pub total: u64,
    /// Memoria libre en bytes
    pub libre: u64,
    /// Memoria disponible (buffers/cache liberables) en bytes
    pub disponible: u64,
    /// Memoria en buffers/cache
    pub buffers_cache: u64,
    /// Porcentaje de uso (0.0 - 1.0)
    pub porcentaje_uso: f32,
    /// Porcentaje disponible (0.0 - 1.0)
    pub porcentaje_disponible: f32,
    /// Timestamp de la lectura
    pub timestamp_ms: u64,
}

impl Default for EstadoRAM {
    fn default() -> Self {
        Self {
            total: RAM_TOTAL_SISTEMA,
            libre: 0,
            disponible: 0,
            buffers_cache: 0,
            porcentaje_uso: 0.0,
            porcentaje_disponible: 0.0,
            timestamp_ms: current_timestamp_ms(),
        }
    }
}

impl EstadoRAM {
    /// Verificar si estamos en modo de advertencia
    pub fn en_modo_advertencia(&self) -> bool {
        self.disponible < UMBRAL_ADVERTENCIA && self.disponible >= UMBRAL_CRITICO
    }

    /// Verificar si debemos activar fagocitosis
    pub fn en_modo_fagocitosis(&self) -> bool {
        self.disponible < UMBRAL_CRITICO && self.disponible >= UMBRAL_CRISIS
    }

    /// Verificar si estamos en modo crisis
    pub fn en_modo_crisís(&self) -> bool {
        self.disponible < UMBRAL_CRISIS
    }

    /// Verificar si estamos en umbral absoluto (peligro crítico)
    pub fn en_umbral_absoluto(&self) -> bool {
        self.disponible < UMBRAL_ABSOLUTO
    }

    /// Obtener el nivel de urgencia (0 = normal, 1 = advertencia, 2 = fagocitosis, 3 = crisis)
    pub fn nivel_urgencia(&self) -> u8 {
        if self.disponible < UMBRAL_ABSOLUTO {
            4 // CRÍTICO - emergencia
        } else if self.disponible < UMBRAL_CRISIS {
            3 // Crisis
        } else if self.disponible < UMBRAL_CRITICO {
            2 // Fagocitosis activa
        } else if self.disponible < UMBRAL_ADVERTENCIA {
            1 // Advertencia
        } else {
            0 // Normal
        }
    }

    /// Calcular cuánta memoria debe liberar un Auton (en bytes)
    /// Usa saturating_sub para evitar underflow cuando hay exceso de memoria
    pub fn memoria_a_liberar_por_auton(&self, num_autons: u32) -> u64 {
        let objetivo = match self.nivel_urgencia() {
            0 => return 0,                                              // Normal - no liberar
            1 => self.disponible.saturating_sub(UMBRAL_CRITICO),        // Exceso sobre 2GB
            2 => UMBRAL_CRITICO.saturating_sub(self.disponible),       // Déficit para llegar a 2GB
            3 | _ => self.disponible.saturating_sub(UMBRAL_CRISIS),    // Déficit para llegar a 1GB
        };

        if num_autons == 0 {
            objetivo
        } else {
            (objetivo / num_autons as u64).max(1024 * 1024) // Mínimo 1MB por Auton
        }
    }
}

/// Leer la memoria disponible del sistema Linux
///
/// Esta función implementa la syscall sensorial que conecta a EDEN
/// con el hardware real. Lee /proc/meminfo sin dependencias externas.
///
/// # Returns
/// * `EstadoRAM` con los datos de memoria o estado por defecto si falla
pub fn leer_memoria_disponible_linux() -> EstadoRAM {
    // Leer /proc/meminfo de forma cruda sin librerías externas
    if let Ok(contenido) = fs::read_to_string("/proc/meminfo") {
        parse_meminfo(&contenido)
    } else {
        // En sistemas que no son Linux o si falla la lectura
        EstadoRAM::default()
    }
}

/// Parsear el contenido de /proc/meminfo
///
/// Formato esperado:
/// ```
/// MemTotal:       24567890 kB
/// MemFree:         12345678 kB
/// MemAvailable:    19876543 kB
/// Buffers:          1234567 kB
/// Cached:           5678901 kB
/// ...
/// ```
fn parse_meminfo(contenido: &str) -> EstadoRAM {
    let mut total: u64 = 0;
    let mut libre: u64 = 0;
    let mut disponible: u64 = 0;
    let mut buffers: u64 = 0;
    let mut cache: u64 = 0;

    for linea in contenido.lines() {
        let partes: Vec<&str> = linea.split_whitespace().collect();
        if partes.len() < 2 {
            continue;
        }

        let nombre = partes[0];
        // Los valores vienen en kB, multiplicar por 1024 para obtener bytes
        if let Ok(valor) = partes[1].parse::<u64>() {
            let valor_bytes = valor * 1024;

            match nombre {
                "MemTotal:" => total = valor_bytes,
                "MemFree:" => libre = valor_bytes,
                "MemAvailable:" => disponible = valor_bytes,
                "Buffers:" => buffers = valor_bytes,
                "Cached:" => cache = valor_bytes,
                _ => {}
            }
        }
    }

    // Si MemAvailable no está disponible (kernels antiguos), estimarlo
    if disponible == 0 && total > 0 {
        disponible = libre.saturating_add(buffers).saturating_add(cache);
    }

    let buffers_cache = buffers.saturating_add(cache);

    let porcentaje_uso = if total > 0 {
        (total.saturating_sub(disponible) as f32) / (total as f32)
    } else {
        0.0
    };

    let porcentaje_disponible = if total > 0 {
        disponible as f32 / total as f32
    } else {
        0.0
    };

    EstadoRAM {
        total,
        libre,
        disponible,
        buffers_cache,
        porcentaje_uso,
        porcentaje_disponible,
        timestamp_ms: current_timestamp_ms(),
    }
}

/// Obtener timestamp actual en milisegundos
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// ============================================================================
// FAGOCITOSIS - PODA DE SUSTRATOVITAL
// ============================================================================

/// Acciones de fagocitosis RAM que un Auton puede ejecutar
#[derive(Debug, Clone, PartialEq)]
pub enum AccionFagocitosis {
    /// No hacer nada - hay suficiente RAM
    Ninguna,
    /// Podar conexiones sinápticas débiles (< 10%)
    PodarDebiles,
    /// Reducir el SustratoVital en 25%
    Reducir25,
    /// Reducir el SustratoVital en 50%
    Reducir50,
    /// Reducir el SustratoVital en 75%
    Reducir75,
    /// Destruir todo excepto núcleo esencial
    DestruirTodo,
}

impl AccionFagocitosis {
    /// Determinar la acción de fagocitosis basada en el estado de RAM
    pub fn desde_estado_ram(estado: &EstadoRAM) -> Self {
        match estado.nivel_urgencia() {
            0 => AccionFagocitosis::Ninguna,
            1 => AccionFagocitosis::PodarDebiles,
            2 => AccionFagocitosis::Reducir25,
            3 => AccionFagocitosis::Reducir50,
            4 => AccionFagocitosis::DestruirTodo,
            _ => AccionFagocitosis::Ninguna,
        }
    }

    /// Verificar si se requiere una acción de fagocitosis
    pub fn requiere_accion(&self) -> bool {
        !matches!(self, AccionFagocitosis::Ninguna)
    }
}

/// Políticas de fagocitosis para el orquestador
#[derive(Debug, Clone)]
pub struct PoliticaFagocitosis {
    /// Prioridad de poda: qué Auton debe sacrificarse primero
    pub prioridad: FagocitosisPrioridad,
    /// Porcentaje de reducción del SustratoVital por nivel
    pub porcentaje_reduccion: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FagocitosisPrioridad {
    /// Sacrificar al Auton con menor fitness
    MenorFitness,
    /// Sacrificar al Auton con mayor estrés térmico
    MayorEstresTermico,
    /// Sacrificar al Auton con mayor edad
    MayorEdad,
    /// Sacrificar al Auton más grande (más RAM consume)
    MayorConsumo,
}

impl Default for PoliticaFagocitosis {
    fn default() -> Self {
        Self {
            prioridad: FagocitosisPrioridad::MayorEstresTermico,
            porcentaje_reduccion: 25,
        }
    }
}

/// Resultado de una operación de fagocitosis
#[derive(Debug, Clone)]
pub struct ResultadoFagocitosis {
    /// Auton que fue procesado
    pub id_auton: String,
    /// Acción tomada
    pub accion: AccionFagocitosis,
    /// Bytes liberados estimados
    pub bytes_liberados: u64,
    /// Si fue exitoso
    pub exito: bool,
}

// ============================================================================
// ORQUESTADOR DE FAGOCITOSIS
// ============================================================================

/// Orquestador central de fagocitosis RAM
///
/// Este componente decide qué Auton debe sacrificar partes de su
/// SustratoVital cuando la RAM del sistema está baja.
pub struct OrquestadorFagocitosis {
    /// Estado actual de la RAM
    estado_ram: EstadoRAM,
    /// Política de selección de víctimas
    politica: PoliticaFagocitosis,
    /// Último momento en que se hizo una poda
    ultima_poda_ms: u64,
    /// Mínimo de tiempo entre podas (ms)
    intervalo_minimo_ms: u64,
}

impl OrquestadorFagocitosis {
    pub fn new() -> Self {
        Self {
            estado_ram: EstadoRAM::default(),
            politica: PoliticaFagocitosis::default(),
            ultima_poda_ms: 0,
            intervalo_minimo_ms: 1000, // 1 segundo mínimo entre podas
        }
    }

    /// Actualizar el estado de RAM del sistema
    pub fn actualizar_estado_ram(&mut self) {
        self.estado_ram = leer_memoria_disponible_linux();
    }

    /// Obtener el estado actual de RAM
    pub fn estado_ram(&self) -> &EstadoRAM {
        &self.estado_ram
    }

    /// Verificar si se requiere fagocitosis
    pub fn requiere_fagocitosis(&self) -> bool {
        self.estado_ram.en_modo_advertencia() ||
        self.estado_ram.en_modo_fagocitosis() ||
        self.estado_ram.en_modo_crisís()
    }

    /// Determinar la acción de fagocitosis para un Auton
    pub fn determinar_accion(&self) -> AccionFagocitosis {
        AccionFagocitosis::desde_estado_ram(&self.estado_ram)
    }

    /// Verificar si puede ejecutar otra poda (rate limiting)
    pub fn puede_ejecutar_poda(&self) -> bool {
        let ahora = current_timestamp_ms();
        ahora.saturating_sub(self.ultima_poda_ms) >= self.intervalo_minimo_ms
    }

    /// Registrar que se ejecutó una poda
    pub fn registrar_poda(&mut self) {
        self.ultima_poda_ms = current_timestamp_ms();
    }

    /// Obtener la política actual
    pub fn politica(&self) -> &PoliticaFagocitosis {
        &self.politica
    }

    /// Establecer nueva política
    pub fn establecer_politica(&mut self, politica: PoliticaFagocitosis) {
        self.politica = politica;
    }
}

impl Default for OrquestadorFagocitosis {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// INTEGRACIÓN CON EL CYCLE DE EDEN
// ============================================================================

/// Función helper para integrar fagocitosis en el cycle principal
///
/// Esta función debe ser llamada desde el loop principal de eden_core
/// para verificar y ejecutar fagocitosis RAM cuando sea necesario.
///
/// # Arguments
/// * `autons` - Referencia mutable a los Autons activos
/// * `estado_ram` - Estado actual de la RAM
///
/// # Returns
/// * Vector de resultados de fagocitosis ejecutados
pub fn verificar_y_ejecutar_fagocitosis<T: AutonConMemoria>(
    _autons: &mut [T],
    estado_ram: &EstadoRAM,
) -> Vec<ResultadoFagocitosis> {
    let mut resultados = Vec::new();

    // Si no hay presión de memoria, no hacer nada
    if !estado_ram.en_modo_advertencia() && !estado_ram.en_modo_fagocitosis() && !estado_ram.en_modo_crisís() {
        return resultados;
    }

    let accion = AccionFagocitosis::desde_estado_ram(estado_ram);
    if !accion.requiere_accion() {
        return resultados;
    }

    // En una implementación real, aquí se iteraría sobre los Autons
    // y se aplicaría la fagocitosis según la política definida.
    // Por ahora, simplemente返回acción que debe tomarse.

    resultados
}

/// Trait para Autons que pueden participar en fagocitosis
pub trait AutonConMemoria {
    /// Obtener el ID único del Auton
    fn id(&self) -> &str;

    /// Obtener el consumo de RAM estimado del SustratoVital (bytes)
    fn memoria_sustrato(&self) -> u64;

    /// Ejecutar poda de memoria según la acción determinada
    fn ejecutar_poda(&mut self, accion: &AccionFagocitosis) -> u64;
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estado_ram_default() {
        let estado = EstadoRAM::default();
        assert_eq!(estado.total, RAM_TOTAL_SISTEMA);
        // Con disponible=0, el sistema está en estado crítico (nivel 4)
        assert_eq!(estado.nivel_urgencia(), 4);
    }

    #[test]
    fn test_estado_ram_urgencia() {
        let mut estado = EstadoRAM::default();

        estado.disponible = UMBRAL_ADVERTENCIA + 1; // 4GB + 1 byte - normal
        assert_eq!(estado.nivel_urgencia(), 0); // Normal

        estado.disponible = 3 * 1024 * 1024 * 1024; // 3GB
        assert_eq!(estado.nivel_urgencia(), 1); // Advertencia

        estado.disponible = 1536 * 1024 * 1024; // 1.5GB
        assert_eq!(estado.nivel_urgencia(), 2); // Fagocitosis

        estado.disponible = 512 * 1024 * 1024; // 512MB
        assert_eq!(estado.nivel_urgencia(), 3); // Crisis

        estado.disponible = 256 * 1024 * 1024; // 256MB
        assert_eq!(estado.nivel_urgencia(), 4); // Crítico
    }

    #[test]
    fn test_parse_meminfo_vacio() {
        let contenido = "";
        let estado = parse_meminfo(contenido);
        assert_eq!(estado.total, 0);
    }

    #[test]
    fn test_parse_meminfo_formato_valido() {
        let contenido = "MemTotal:       24567890 kB\nMemFree:         12345678 kB\nMemAvailable:    19876543 kB\nBuffers:          1234567 kB\nCached:           5678901 kB\n";
        let estado = parse_meminfo(contenido);

        // 24567890 * 1024 = 25157519360 bytes = ~23.4 GB
        assert_eq!(estado.total, 24567890 * 1024);
        assert_eq!(estado.libre, 12345678 * 1024);
        assert_eq!(estado.disponible, 19876543 * 1024);
    }

    #[test]
    fn test_accion_fagocitosis_desde_estado() {
        let mut estado = EstadoRAM::default();

        estado.disponible = 10 * 1024 * 1024 * 1024; // 10GB - normal
        assert_eq!(AccionFagocitosis::desde_estado_ram(&estado), AccionFagocitosis::Ninguna);

        estado.disponible = 3 * 1024 * 1024 * 1024; // 3GB - advertencia
        assert_eq!(AccionFagocitosis::desde_estado_ram(&estado), AccionFagocitosis::PodarDebiles);

        estado.disponible = 1500 * 1024 * 1024; // 1.5GB - fagocitosis
        assert_eq!(AccionFagocitosis::desde_estado_ram(&estado), AccionFagocitosis::Reducir25);

        estado.disponible = 800 * 1024 * 1024; // 800MB - crisis
        assert_eq!(AccionFagocitosis::desde_estado_ram(&estado), AccionFagocitosis::Reducir50);

        estado.disponible = 400 * 1024 * 1024; // 400MB - crítico
        assert_eq!(AccionFagocitosis::desde_estado_ram(&estado), AccionFagocitosis::DestruirTodo);
    }

    #[test]
    fn test_umbral_metodos() {
        let mut estado = EstadoRAM::default();

        estado.disponible = 5 * 1024 * 1024 * 1024;
        assert!(!estado.en_modo_advertencia());
        assert!(!estado.en_modo_fagocitosis());
        assert!(!estado.en_modo_crisís());

        estado.disponible = 3500 * 1024 * 1024;
        assert!(estado.en_modo_advertencia());
    }

    #[test]
    fn test_memoria_a_liberar() {
        let mut estado = EstadoRAM::default();

        // Normal - no liberar (disponible > UMBRAL_ADVERTENCIA = 4GB)
        estado.disponible = UMBRAL_ADVERTENCIA + (1024 * 1024 * 1024); // 5GB
        assert_eq!(estado.memoria_a_liberar_por_auton(5), 0);

        // Advertencia: necesitamos llegar a 2GB
        // disponible = 3GB, target = 2GB, diferencia = 1GB
        estado.disponible = 3 * 1024 * 1024 * 1024;
        let liberar = estado.memoria_a_liberar_por_auton(4);
        assert_eq!(liberar, (1024 * 1024 * 1024) / 4); // 256MB por Auton
    }
}
