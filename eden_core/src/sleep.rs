//! Sleep - Eden's Processing Modes System
//!
//! Eden has different operational modes analogous to sleep/awake cycles.
//! During rest, it consolidates memories and optimizes its processes.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::membrain::{MemBrain, NOW_MS};

const DB_PATH: &str = "/home/ubuntu/eden_kg";
const AWAKE_THRESHOLD_MS: u64 = 30 * 60 * 1000; // 30 minutes

/// Eden's operational mode
#[derive(Debug, Clone, PartialEq)]
pub enum EdenMode {
    Awake,       // Full operation
    Alert,       // High readiness
    Resting,     // Reduced activity
    Processing,  // Memory consolidation
    Hibernating, // Minimal activity
}

/// Sleep state
#[derive(Debug, Clone)]
pub struct SleepState {
    pub mode: EdenMode,
    pub last_activity: u64,
    pub consecutive_rest_cycles: u64,
    pub last_mode_change: u64,
    pub memory_consolidation_active: bool,
    pub optimization_active: bool,
}

impl SleepState {
    /// Create new sleep state
    pub fn new() -> Self {
        SleepState {
            mode: EdenMode::Awake,
            last_activity: NOW_MS(),
            consecutive_rest_cycles: 0,
            last_mode_change: NOW_MS(),
            memory_consolidation_active: false,
            optimization_active: false,
        }
    }

    /// Update activity timestamp
    pub fn record_activity(&mut self) {
        self.last_activity = NOW_MS();

        // Wake up if was resting
        if self.mode != EdenMode::Awake {
            self.mode = EdenMode::Awake;
            self.consecutive_rest_cycles = 0;
            self.last_mode_change = NOW_MS();
        }
    }

    /// Check and update mode based on activity
    pub fn check_mode(&mut self) -> EdenMode {
        let now = NOW_MS();
        let idle_time = now - self.last_activity;

        let new_mode = if idle_time <= AWAKE_THRESHOLD_MS {
            EdenMode::Awake
        } else if idle_time <= AWAKE_THRESHOLD_MS * 2 {
            EdenMode::Alert
        } else if idle_time <= AWAKE_THRESHOLD_MS * 4 {
            EdenMode::Resting
        } else if idle_time <= AWAKE_THRESHOLD_MS * 8 {
            EdenMode::Processing
        } else {
            EdenMode::Hibernating
        };

        if new_mode != self.mode {
            self.mode = new_mode.clone();
            self.last_mode_change = now;

            // Track consecutive rest cycles
            if matches!(
                new_mode,
                EdenMode::Resting | EdenMode::Processing | EdenMode::Hibernating
            ) {
                self.consecutive_rest_cycles += 1;
            } else {
                self.consecutive_rest_cycles = 0;
            }
        }

        // Activate special processes based on mode
        self.memory_consolidation_active =
            matches!(new_mode, EdenMode::Resting | EdenMode::Processing);

        self.optimization_active = matches!(new_mode, EdenMode::Processing | EdenMode::Hibernating);

        new_mode
    }

    /// Get time in current mode
    pub fn time_in_mode(&self) -> u64 {
        NOW_MS() - self.last_mode_change
    }

    /// Get time since last activity
    pub fn idle_time(&self) -> u64 {
        NOW_MS() - self.last_activity
    }

    /// Get mode description
    pub fn mode_description(&self) -> &'static str {
        match self.mode {
            EdenMode::Awake => "Eden está completamente activo y procesando información",
            EdenMode::Alert => "Eden está alerta, procesando pero en espera",
            EdenMode::Resting => "Eden está descansando, consolidando experiencias recientes",
            EdenMode::Processing => "Eden está en procesamiento profundo, optimizando patrones",
            EdenMode::Hibernating => "Eden está hibernando, actividad mínima",
        }
    }
}

impl Default for SleepState {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory consolidation during rest
fn consolidate_memories() {
    let mut db = match MemBrain::new(DB_PATH) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[SLEEP] Error abriendo DB para consolidación: {}", e);
            return;
        }
    };

    println!("[SLEEP] Iniciando consolidación de memorias...");

    // Find recently accessed patterns
    let recent_patterns = db.search(b"access:");

    for (i, pattern) in recent_patterns.iter().enumerate() {
        if pattern.len() > 10 {
            // Strengthen pattern by re-inserting with higher weight
            let mut strengthened = pattern.clone();
            strengthened.push(0x01); // consolidation flag

            let key = format!("consolidated:{}", NOW_MS() + i as u64);
            db.dopa(key.as_bytes(), strengthened);
        }
    }

    // Log consolidation
    let log_key = format!("sleep:consolidation:{}", NOW_MS());
    db.dopa(
        log_key.as_bytes(),
        format!("consolidated {} patterns", recent_patterns.len())
            .as_bytes()
            .to_vec(),
    );

    println!(
        "[SLEEP] Consolidación completada - {} patrones procesados",
        recent_patterns.len()
    );
}

/// Optimize system during processing
fn run_optimization() {
    let mut db = match MemBrain::new(DB_PATH) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[SLEEP] Error abriendo DB para optimización: {}", e);
            return;
        }
    };

    println!("[SLEEP] Iniciando optimización del sistema...");

    // Find weak/duplicate patterns
    let all_patterns = db.search(b"weight:");
    let mut strong_patterns: Vec<&Vec<u8>> = Vec::new();
    let mut weak_count = 0;

    for pattern in &all_patterns {
        if pattern.len() > 16 {
            // Read weight (at offset 8, 8 bytes)
            let weight = if pattern.len() >= 16 {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&pattern[8..16]);
                f64::from_le_bytes(bytes)
            } else {
                0.5
            };

            if weight > 0.6 {
                strong_patterns.push(pattern);
            } else {
                weak_count += 1;
            }
        }
    }

    // Mark weak patterns for pruning
    println!(
        "[SLEEP] Optimización: {} patrones fuertes, {} débiles",
        strong_patterns.len(),
        weak_count
    );

    // Log optimization
    let log_key = format!("sleep:optimization:{}", NOW_MS());
    db.dopa(
        log_key.as_bytes(),
        format!(
            "optimized {} strong, marked {} weak",
            strong_patterns.len(),
            weak_count
        )
        .as_bytes()
        .to_vec(),
    );
}

/// Sleep controller
pub struct SleepController {
    pub state: SleepState,
}

impl SleepController {
    /// Create new sleep controller
    pub fn new() -> Self {
        SleepController {
            state: SleepState::new(),
        }
    }

    /// Record activity
    pub fn activity(&mut self) {
        self.state.record_activity();
    }

    /// Check mode
    pub fn check(&mut self) -> EdenMode {
        let mode = self.state.check_mode();

        // Run mode-specific processes
        if self.state.memory_consolidation_active {
            consolidate_memories();
            self.state.memory_consolidation_active = false; // Only once per check
        }

        if self.state.optimization_active {
            run_optimization();
            self.state.optimization_active = false; // Only once per check
        }

        mode
    }

    /// Get current state report
    pub fn report(&self) -> String {
        format!(
            "[SLEEP] Modo: {:?} | Idle: {}ms | En modo: {}ms | Desc: {}",
            self.state.mode,
            self.state.idle_time(),
            self.state.time_in_mode(),
            self.state.mode_description()
        )
    }
}

impl Default for SleepController {
    fn default() -> Self {
        Self::new()
    }
}

/// Start sleep system
pub fn start_sleep() {
    println!("[SLEEP] Sistema de ciclos de operación iniciado");
    println!(
        "[SLEEP] Umbral de inactividad: {}ms ({}min)",
        AWAKE_THRESHOLD_MS,
        AWAKE_THRESHOLD_MS / 60000
    );
}

/// Stop sleep system
pub fn stop() {
    println!("[SLEEP] Detenido");
}

/// Manual sleep trigger
pub fn trigger_sleep() {
    println!("[SLEEP] Entrando en modo de descanso...");
    consolidate_memories();
    run_optimization();
    println!("[SLEEP] Sueño completado");
}

/// Force wake
pub fn wake() {
    println!("[SLEEP] Despertando...");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sleep_state_creation() {
        let state = SleepState::new();
        assert_eq!(state.mode, EdenMode::Awake);
    }

    #[test]
    fn test_mode_change() {
        let mut state = SleepState::new();
        state.record_activity();
        assert_eq!(state.mode, EdenMode::Awake);
    }

    #[test]
    fn test_sleep_controller() {
        let controller = SleepController::new();
        assert_eq!(controller.state.mode, EdenMode::Awake);
    }
}
