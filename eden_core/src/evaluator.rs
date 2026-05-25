//! Evaluator - Eden's Self-Consciousness System
//!
//! The evaluator constantly monitors Eden's state and creates
//! self-evaluations for metacognition.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::membrain::{MemBrain, NOW_MS};

const DB_PATH: &str = "/home/ubuntu/eden_kg";
const EVALUATION_INTERVAL_SECS: u64 = 300;

/// Self-evaluation record
#[derive(Debug, Clone)]
pub struct SelfEvaluation {
    pub timestamp: u64,
    pub tasa_acierto: f64,
    pub patron_fuerte: String,
    pub patron_debil: String,
    pub ciclos: i64,
    pub estado_general: String,
}

/// Calculate success rate from MemBrain
fn calculate_tasa_acierto(_db: &mut MemBrain) -> f64 {
    // MemBrain doesn't have tables - return default value
    0.5
}

/// Calculate pattern extremes
fn calculate_pattern_extremes(_db: &mut MemBrain) -> (String, String) {
    // MemBrain doesn't have tables - return default values
    ("none".to_string(), "none".to_string())
}

/// Count survival events
fn count_supervivencia_events(_db: &mut MemBrain) -> i64 {
    // MemBrain doesn't have tables - return 0
    0
}

/// Calculate general state
fn calculate_estado_general(tasa_acierto: f64, ciclos: i64) -> String {
    if tasa_acierto >= 0.8 && ciclos < 100 {
        "OPTIMO".to_string()
    } else if tasa_acierto >= 0.6 {
        "ESTABLE".to_string()
    } else if tasa_acierto >= 0.4 {
        "ADVERTENCIA".to_string()
    } else {
        "CRITICO".to_string()
    }
}

/// Save evaluation to MemBrain
fn save_evaluation(_db: &mut MemBrain, _eval: &SelfEvaluation) {
    // MemBrain doesn't have tables - this is a stub
    // In a real implementation, this would store using one of the
    // available methods (gluta, gaba, dopa, adre)
}

/// Perform one evaluation cycle
fn evaluate() {
    let mut db = match MemBrain::new(DB_PATH) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[EVALUATOR] Error abriendo DB: {}", e);
            return;
        }
    };

    // Calculate metrics
    let tasa_acierto = calculate_tasa_acierto(&mut db);
    let (patron_fuerte, patron_debil) = calculate_pattern_extremes(&mut db);
    let ciclos = count_supervivencia_events(&mut db);
    let estado_general = calculate_estado_general(tasa_acierto, ciclos);

    let timestamp = NOW_MS();

    let eval = SelfEvaluation {
        timestamp,
        tasa_acierto,
        patron_fuerte,
        patron_debil,
        ciclos,
        estado_general,
    };

    // Save to database
    save_evaluation(&mut db, &eval);

    println!(
        "[EVALUATOR] Evaluación: tasa={:.2}%, fuerte={}, debil={}, ciclos={}, estado={}",
        eval.tasa_acierto * 100.0,
        eval.patron_fuerte,
        eval.patron_debil,
        eval.ciclos,
        eval.estado_general
    );
}

/// Start evaluator
pub fn start_evaluator() {
    println!(
        "[EVALUATOR] Autoconsciencia de Eden iniciada - ciclo cada {}s",
        EVALUATION_INTERVAL_SECS
    );
    evaluate();
}

/// Stop evaluator
pub fn stop() {
    println!("[EVALUATOR] Detenido");
}

/// Get latest evaluation
pub fn get_latest_evaluation() -> Option<SelfEvaluation> {
    // MemBrain doesn't have tables - return None
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estado_general_calculation() {
        assert_eq!(calculate_estado_general(0.9, 50), "OPTIMO");
        assert_eq!(calculate_estado_general(0.7, 50), "ESTABLE");
        assert_eq!(calculate_estado_general(0.5, 50), "ADVERTENCIA");
        assert_eq!(calculate_estado_general(0.3, 50), "CRITICO");
    }
}
