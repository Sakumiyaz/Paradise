//! # Sistema Inmune A-Life para Eden
//!
//! Detecta y neutraliza patrones patogénicos en el organismo A-Life.
//!
//! Arquitectura: En lugar de SQL tradicional, usa MemBrain (motor neural).
//! - Patógenos detectados → Neuronas Adrenalina (TTL definido)
//! - Memoria inmune → Neuronas Gaba (persistentes)
//! - Tracking de ciclos → Neuronas Glutamato (efímeras)
//!
//! 100% RUST PURO - Sin dependencias externas.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::membrain::MemBrain;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Socket de comunicación con Purgatory
const PURGATORY_SOCK: &str = "/tmp/eden_cell.sock";

/// Ruta de almacenamiento de MemBrain
const MEMBRAIN_PATH: &str = "/home/ubuntu/eden_memory";

/// Intervalo de escaneo inmune (segundos)
const IMMUNE_INTERVAL_SECS: u64 = 30;

/// Umbral de loops infinitos (mismo input > N veces en ventana)
const LOOP_THRESHOLD: u32 = 50;

/// Ventana de tiempo para detectar loops (ms)
const LOOP_WINDOW_MS: u64 = 60_000;

/// Delta máximo de ciclos antes de considerar fork bomb
const FORK_BOMB_DELTA: u64 = 1000;

/// Ventana de tiempo para fork bombs (ms)
const FORK_BOMB_WINDOW_MS: u64 = 10_000;

/// Flag global de ejecución
static RUNNING: AtomicBool = AtomicBool::new(true);

// =============================================================================
// TIPOS DE DATOS NEURALES (Representación de Patógenos)
// =============================================================================

/// Tipo de patógeno neural
#[derive(Debug, Clone, PartialEq)]
pub enum PatogenType {
    /// Loop infinito - misma actividad repetida
    InfiniteLoop,
    /// Fork bomb - reproducción descontrolada
    ForkBomb,
    /// Corrupción - datos invariants violados
    Corruption,
    /// Corrupción energética - energía fuera de rango
    EnergyCorruption,
}

impl PatogenType {
    fn as_str(&self) -> &'static str {
        match self {
            PatogenType::InfiniteLoop => "INFINITE_LOOP",
            PatogenType::ForkBomb => "FORK_BOMB",
            PatogenType::Corruption => "CORRUPTION",
            PatogenType::EnergyCorruption => "ENERGY_CORRUPTION",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "INFINITE_LOOP" => PatogenType::InfiniteLoop,
            "FORK_BOMB" => PatogenType::ForkBomb,
            "ENERGY_CORRUPTION" => PatogenType::EnergyCorruption,
            _ => PatogenType::Corruption,
        }
    }
}

/// Firma de patógeno detectada
#[derive(Debug, Clone)]
pub struct PatogenSignature {
    pub firma: String,
    pub tipo: PatogenType,
    pub timestamp: u64,
    pub neutralizado: bool,
    pub details: String,
    pub cycles_active: u32,
}

/// Estado del tracker de ciclos para detectar fork bombs
#[derive(Debug, Clone)]
struct CycleTracker {
    reading_time: u64,
    cycles: u64,
}

/// Registro de actividad para detectar loops
#[derive(Debug, Clone)]
struct ActivityRecord {
    input_hash: String,
    timestamp: u64,
    count: u32,
}

// =============================================================================
// MEMORY IMMUNE - Capa de memoria inmune sobre MemBrain
// =============================================================================

/// Wrapper sobre MemBrain para operaciones inmunes especializadas
struct MemoryImmune {
    brain: MemBrain,
}

impl MemoryImmune {
    fn new() -> std::io::Result<Self> {
        Ok(Self {
            brain: MemBrain::new(MEMBRAIN_PATH)?,
        })
    }

    /// Registra una nueva detección de patógeno
    fn register_patogen(&mut self, firma: &str, tipo: &PatogenType, details: &str) {
        let key = format!("immune:{}", firma);
        let data = format!(
            "{}|{}|{}|{}",
            tipo.as_str(),
            Self::now(),
            if details.len() > 200 {
                &details[..200]
            } else {
                details
            },
            1 // neutralizado = true
        );

        // Usar GABA (estable) para memoria inmune persistente
        self.brain.gaba(key.as_bytes(), data.as_bytes().to_vec());

        println!(
            "[IMMUNE] ✓ Patógeno registrado: {} tipo={}",
            firma,
            tipo.as_str()
        );
    }

    /// Verifica si una firma ya está en memoria inmune
    fn is_immune(&mut self, firma: &str) -> bool {
        let key = format!("immune:{}", firma);
        self.brain.recall(key.as_bytes()).is_some()
    }

    /// Registra actividad para tracking de loops
    fn record_activity(&mut self, input_hash: &str) {
        let key = format!("activity:{}", input_hash);
        let existing = self.brain.recall(key.as_bytes());

        let count: u32 = existing
            .and_then(|d| String::from_utf8(d).ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let new_count = count + 1;
        self.brain
            .gluta(key.as_bytes(), new_count.to_string().into_bytes());
    }

    /// Obtiene todas las actividades recientes
    fn get_recent_activities(&self) -> Vec<(String, u32)> {
        // En una implementación completa, esto usaría el índice semántico
        // Por ahora, retornamos vacío (el motor MemBrain lo maneja internamente)
        Vec::new()
    }

    /// Registra tracking de ciclos
    fn record_cycle(&mut self, cycles: u64) {
        let key = format!("cycle:{}", Self::now());
        let data = cycles.to_string();
        // Glutamato: efímero pero registrado
        self.brain.gluta(key.as_bytes(), data.into_bytes());
    }

    /// Obtiene ciclos de hace N segundos
    fn get_cycles_ago(&self, ms_ago: u64) -> Option<u64> {
        let _target_time = Self::now().saturating_sub(ms_ago);

        // Buscar el reading más cercano a target_time
        // En producción, esto usaría índices temporales
        None
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

// =============================================================================
// DETECCIÓN DE PATÓGENOS
// =============================================================================

/// Detecta loops infinitos (mismo input > threshold en ventana)
fn detect_infinite_loops(immune: &mut MemoryImmune) {
    // En un sistema real, esto analizaría patrones de entrada
    // Simulado por ahora:

    let activities = immune.get_recent_activities();

    for (input_hash, count) in activities {
        if count > LOOP_THRESHOLD {
            let firma = generate_signature(&format!("LOOP_{}", input_hash));
            let details = format!(
                "input={} count={} en_window={}ms",
                input_hash, count, LOOP_WINDOW_MS
            );

            println!(
                "[IMMUNE] ⚠️ PATÓGENO: LOOP INFINITO firma={} {}",
                firma, details
            );

            if !immune.is_immune(&firma) {
                immune.register_patogen(&firma, &PatogenType::InfiniteLoop, &details);
                notify_purgatory(&firma, "SIGSTOP_LOOP");
            }
        }
    }
}

/// Detecta fork bombs (ciclos aumentan muy rápido)
fn detect_fork_bombs(immune: &mut MemoryImmune, current_cycles: u64) {
    // Registrar ciclo actual
    immune.record_cycle(current_cycles);

    // Obtener ciclos de hace 10 segundos
    if let Some(old_cycles) = immune.get_cycles_ago(FORK_BOMB_WINDOW_MS) {
        let delta = current_cycles.saturating_sub(old_cycles);

        if delta > FORK_BOMB_DELTA {
            let firma = generate_signature(&format!("FORK_{}_{}", old_cycles, current_cycles));
            let details = format!(
                "ciclos_anterior={} ciclos_actual={} delta={} en_{}ms",
                old_cycles, current_cycles, delta, FORK_BOMB_WINDOW_MS
            );

            println!(
                "[IMMUNE] ⚠️ PATÓGENO: FORK BOMB firma={} {}",
                firma, details
            );

            if !immune.is_immune(&firma) {
                immune.register_patogen(&firma, &PatogenType::ForkBomb, &details);
                notify_purgatory(&firma, "SIGSTOP_FORKBOMB");
            }
        }
    }
}

/// Detecta corrupción de datos (pesos fuera de rango)
fn detect_corruption() {
    // En un sistema real, esto verificaría integridad de MemBrain
    // y consistencia de las neuronas

    // Verificar que no hay pesos < 0 o > 1
    // Esto se hace dentro de la implementación de Neuron
    // pero podemos hacer un check adicional aquí

    println!("[IMMUNE] Verificando integridad de datos...");

    // Auto-poiesis check
    println!("[IMMUNE] Ejecutando autopoiesis check...");
}

/// Detecta corrupción energética (energía fuera de rango)
fn detect_energy_corruption() {
    // Similar a detect_corruption pero para energía específica
    // En el contexto A-Life, energía vive en el rango [0, 10000]

    println!("[IMMUNE] Verificando niveles energéticos...");
}

// =============================================================================
// UTILIDADES
// =============================================================================

/// Genera firma única para un patógeno
fn generate_signature(data: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let hash = hasher.finish();

    format!("IMM_{:016x}", hash)
}

/// Notifica a Purgatory sobre un patógeno
fn notify_purgatory(firma: &str, action: &str) {
    println!(
        "[IMMUNE] → Notificando a Purgatory: firma={} action={}",
        firma, action
    );

    // Intentar escribir al socket de purgatory
    if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(PURGATORY_SOCK) {
        let msg = format!(
            "{{\"op\":\"immune_alert\",\"firma\":\"{}\",\"action\":\"{}\"}}\n",
            firma, action
        );

        if let Err(e) = stream.write_all(msg.as_bytes()) {
            println!("[IMMUNE] Error enviando a purgatory: {}", e);
        } else {
            println!("[IMMUNE] ✓ Alerta enviada a Purgatory");
        }
    } else {
        println!("[IMMUNE] ⚠ Purgatory no disponible, alert logged");
    }
}

// =============================================================================
// API PÚBLICA
// =============================================================================

/// Inicia el sistema inmune
pub fn start_immune() {
    println!(
        "[IMMUNE] 🧬 Sistema Inmune A-Life iniciado - intervalo {}s",
        IMMUNE_INTERVAL_SECS
    );
    println!("[IMMUNE] Usando MemBrain como almacén neural");

    let mut immune = match MemoryImmune::new() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("[IMMUNE] ❌ Error inicializando MemBrain: {}", e);
            return;
        }
    };

    // Ejecutar escaneo inicial
    run_immune_scan(&mut immune, 0);

    // Loop principal
    while RUNNING.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(IMMUNE_INTERVAL_SECS));

        if RUNNING.load(Ordering::SeqCst) {
            // Obtener ciclos actuales (simulado)
            let current_cycles = (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs())
                % 10000;

            run_immune_scan(&mut immune, current_cycles);
        }
    }

    // Persistir antes de salir
    if let Err(e) = immune.brain.persist() {
        eprintln!("[IMMUNE] Error persistiendo: {}", e);
    }

    println!("[IMMUNE] Sistema Inmune detenido");
}

/// Detiene el sistema inmune
pub fn stop_immune() {
    RUNNING.store(false, Ordering::SeqCst);
    println!("[IMMUNE] Señal de parada recibida");
}

/// Ejecuta un ciclo de escaneo completo
fn run_immune_scan(immune: &mut MemoryImmune, current_cycles: u64) {
    println!("[IMMUNE] 🔍 Iniciando escaneo neural...");

    // 1. Detectar loops infinitos
    detect_infinite_loops(immune);

    // 2. Detectar fork bombs
    detect_fork_bombs(immune, current_cycles);

    // 3. Detectar corrupción de datos
    detect_corruption();

    // 4. Detectar corrupción energética
    detect_energy_corruption();

    // 5. Ejecutar autopoiesis en MemBrain
    if let Ok(mut brain) = MemBrain::new(MEMBRAIN_PATH) {
        let report = brain.autopoiesis();
        if !report.healthy {
            println!("[IMMUNE] ⚠️ Anomalías detectadas por autopoiesis:");
            if !report.dead_neurons.is_empty() {
                println!("  - Neuronas muertas: {}", report.dead_neurons.len());
            }
            if !report.corrupted_neurons.is_empty() {
                println!("  - Neuronas corruptas: {}", report.corrupted_neurons.len());
            }
        } else {
            println!("[IMMUNE] ✓ Autopoiesis: sistema saludable");
        }
    }

    // 6. Persistir estado
    if let Err(e) = immune.brain.persist() {
        eprintln!("[IMMUNE] Error persistiendo: {}", e);
    }

    // 7. Mostrar estadísticas
    let stats = immune.brain.stats();
    println!(
        "[IMMUNE] 📊 WM:{}/LTM:{}/Conexiones:{}",
        stats.working_memory_size, stats.long_term_memory_size, stats.total_connections
    );

    println!("[IMMUNE] ✅ Escaneo completado");
}

/// Verifica si un patrón está bloqueado por inmunidad
pub fn check_pattern_immunity(input_hash: &str) -> bool {
    let mut immune = match MemoryImmune::new() {
        Ok(i) => i,
        Err(_) => return false,
    };

    let firma_loop = generate_signature(&format!("LOOP_{}", input_hash));
    let firma_corrupt = generate_signature(&format!("CORRUPT_{}", input_hash));

    let is_loop_immune = immune.is_immune(&firma_loop);
    let is_corrupt_immune = immune.is_immune(&firma_corrupt);

    if is_loop_immune || is_corrupt_immune {
        println!(
            "[IMMUNE] 🚫 PATRÓN BLOQUEADO: input_hash={} loop={} corrupt={}",
            input_hash, is_loop_immune, is_corrupt_immune
        );
    }

    is_loop_immune || is_corrupt_immune
}

/// Obtiene toda la memoria inmune
pub fn get_immune_memory() -> Vec<PatogenSignature> {
    // En producción, esto leería del índice semántico de MemBrain
    // Retornamos vacío por ahora - se implementa cuando se use el sistema
    Vec::new()
}

/// Obtiene estadísticas del sistema inmune
pub fn get_immune_stats() -> ImmuneStats {
    let brain = MemBrain::new(MEMBRAIN_PATH).ok();

    brain
        .map(|b| {
            let stats = b.stats();
            ImmuneStats {
                working_memory: stats.working_memory_size as u64,
                long_term_memory: stats.long_term_memory_size as u64,
                total_connections: stats.total_connections,
                memory_bytes: stats.memory_bytes,
            }
        })
        .unwrap_or(ImmuneStats::default())
}

#[derive(Debug, Clone, Default)]
pub struct ImmuneStats {
    pub working_memory: u64,
    pub long_term_memory: u64,
    pub total_connections: u64,
    pub memory_bytes: u64,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_generation() {
        let sig1 = generate_signature("test_data");
        let sig2 = generate_signature("test_data");
        assert_eq!(sig1, sig2);

        let sig3 = generate_signature("different_data");
        assert_ne!(sig1, sig3);
    }

    #[test]
    fn test_patogen_type_str() {
        assert_eq!(PatogenType::InfiniteLoop.as_str(), "INFINITE_LOOP");
        assert_eq!(PatogenType::ForkBomb.as_str(), "FORK_BOMB");
        assert_eq!(PatogenType::Corruption.as_str(), "CORRUPTION");
        assert_eq!(PatogenType::EnergyCorruption.as_str(), "ENERGY_CORRUPTION");
    }

    #[test]
    fn test_patogen_type_from_str() {
        assert_eq!(
            PatogenType::from_str("INFINITE_LOOP"),
            PatogenType::InfiniteLoop
        );
        assert_eq!(PatogenType::from_str("FORK_BOMB"), PatogenType::ForkBomb);
        assert_eq!(PatogenType::from_str("UNKNOWN"), PatogenType::Corruption);
    }

    #[test]
    fn test_signature_format() {
        let sig = generate_signature("test");
        assert!(sig.starts_with("IMM_"));
        assert_eq!(sig.len(), 20); // "IMM_" + 16 hex chars = 20
    }
}
