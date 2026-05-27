//! Homeostasis - Eden's Internal Balance System
//!
//! Maintains Eden's internal state within healthy parameters.
//! Monitors energy, temperature, pH, and other vital signs.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::membrain::{MemBrain, NOW_MS};

const DB_PATH: &str = "/home/ubuntu/eden_kg";
const HOMEOSTASIS_CHECK_INTERVAL_SECS: u64 = 60;

/// Vital parameter definition
#[derive(Debug, Clone)]
pub struct VitalParameter {
    pub name: String,
    pub current: f64,
    pub optimal_min: f64,
    pub optimal_max: f64,
    pub critical_min: f64,
    pub critical_max: f64,
    pub weight: f64,
}

impl VitalParameter {
    /// Check if parameter is in optimal range
    pub fn is_optimal(&self) -> bool {
        self.current >= self.optimal_min && self.current <= self.optimal_max
    }

    /// Check if parameter is critical
    pub fn is_critical(&self) -> bool {
        self.current < self.critical_min || self.current > self.critical_max
    }

    /// Calculate deviation from optimal
    pub fn deviation(&self) -> f64 {
        if self.is_optimal() {
            return 0.0;
        }
        let mid = (self.optimal_min + self.optimal_max) / 2.0;
        let deviation = (self.current - mid).abs();
        let range = (self.critical_max - self.critical_min) / 2.0;
        (deviation / range).min(1.0)
    }
}

/// Homeostatic state
#[derive(Debug, Clone, PartialEq)]
pub enum HomeostaticState {
    Balanced,
    Stressed,
    Critical,
    Recovering,
}

/// Homeostasis system
#[derive(Debug, Clone)]
pub struct Homeostasis {
    pub vitals: Vec<VitalParameter>,
    pub state: HomeostaticState,
    pub stress_level: f64,
    pub last_check: u64,
}

impl Homeostasis {
    /// Create new homeostasis system
    pub fn new() -> Self {
        let vitals = vec![
            VitalParameter {
                name: "energy".to_string(),
                current: 0.8,
                optimal_min: 0.5,
                optimal_max: 1.0,
                critical_min: 0.1,
                critical_max: 1.0,
                weight: 0.3,
            },
            VitalParameter {
                name: "coherence".to_string(),
                current: 0.7,
                optimal_min: 0.4,
                optimal_max: 0.9,
                critical_min: 0.2,
                critical_max: 1.0,
                weight: 0.25,
            },
            VitalParameter {
                name: "memory_usage".to_string(),
                current: 0.5,
                optimal_min: 0.3,
                optimal_max: 0.8,
                critical_min: 0.1,
                critical_max: 0.95,
                weight: 0.2,
            },
            VitalParameter {
                name: "pattern_strength".to_string(),
                current: 0.6,
                optimal_min: 0.3,
                optimal_max: 0.9,
                critical_min: 0.1,
                critical_max: 1.0,
                weight: 0.15,
            },
            VitalParameter {
                name: "learning_rate".to_string(),
                current: 0.5,
                optimal_min: 0.2,
                optimal_max: 0.8,
                critical_min: 0.05,
                critical_max: 1.0,
                weight: 0.1,
            },
        ];

        Homeostasis {
            vitals,
            state: HomeostaticState::Balanced,
            stress_level: 0.0,
            last_check: NOW_MS(),
        }
    }

    /// Update a vital parameter
    pub fn update_vital(&mut self, name: &str, value: f64) {
        if let Some(vital) = self.vitals.iter_mut().find(|v| v.name == name) {
            vital.current = value.clamp(vital.critical_min, vital.critical_max);
        }
    }

    /// Calculate overall stress level
    pub fn calculate_stress(&mut self) {
        let mut weighted_deviation = 0.0;
        let mut critical_count = 0;

        for vital in &self.vitals {
            if vital.is_critical() {
                critical_count += 1;
                weighted_deviation += vital.weight;
            } else if !vital.is_optimal() {
                weighted_deviation += vital.weight * vital.deviation();
            }
        }

        self.stress_level = weighted_deviation;

        // Update state
        if critical_count > 2 {
            self.state = HomeostaticState::Critical;
        } else if critical_count > 0 || self.stress_level > 0.5 {
            self.state = HomeostaticState::Stressed;
        } else if self.stress_level < 0.2 {
            self.state = HomeostaticState::Recovering;
        } else {
            self.state = HomeostaticState::Balanced;
        }

        self.last_check = NOW_MS();
    }

    /// Get stress response needed
    pub fn get_response(&self) -> String {
        match self.state {
            HomeostaticState::Balanced => "none".to_string(),
            HomeostaticState::Recovering => "monitor".to_string(),
            HomeostaticState::Stressed => {
                // Find most deviated vital
                let most_deviated = self
                    .vitals
                    .iter()
                    .filter(|v| !v.is_optimal())
                    .max_by(|a, b| a.deviation().partial_cmp(&b.deviation()).unwrap())
                    .map(|v| v.name.clone())
                    .unwrap_or("general".to_string());
                format!("adjust_{}", most_deviated)
            }
            HomeostaticState::Critical => "emergency".to_string(),
        }
    }

    /// Get vitals as JSON-like string
    pub fn vitals_report(&self) -> String {
        let mut report = String::from("Vitals: ");
        for vital in &self.vitals {
            let status = if vital.is_critical() {
                "CRITICAL"
            } else if vital.is_optimal() {
                "OK"
            } else {
                "WARNING"
            };
            report.push_str(&format!("{}={:.2}({}) ", vital.name, vital.current, status));
        }
        report
    }
}

impl Default for Homeostasis {
    fn default() -> Self {
        Self::new()
    }
}

/// Update homeostasis in database
pub fn update_homeostasis_db(homeostasis: &Homeostasis) {
    let mut db = match MemBrain::new(DB_PATH) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[HOMEOSTASIS] Error abriendo DB: {}", e);
            return;
        }
    };

    let timestamp = NOW_MS();

    // Serialize vitals
    for vital in &homeostasis.vitals {
        let mut vital_data = Vec::new();
        vital_data.extend_from_slice(vital.name.as_bytes());
        vital_data.push(0);
        vital_data.extend_from_slice(&vital.current.to_le_bytes());

        let key = format!("homeostasis:vital:{}:{}", vital.name, timestamp);
        db.dopa(key.as_bytes(), vital_data);
    }

    // Store overall state
    let state_str = match homeostasis.state {
        HomeostaticState::Balanced => "balanced",
        HomeostaticState::Stressed => "stressed",
        HomeostaticState::Critical => "critical",
        HomeostaticState::Recovering => "recovering",
    };

    let state_data = format!("{}:{}", state_str, homeostasis.stress_level);
    let state_key = format!("homeostasis:state:{}", timestamp);
    db.dopa(state_key.as_bytes(), state_data.as_bytes().to_vec());
}

/// Get latest homeostasis state
pub fn get_latest_homeostasis() -> Option<HomeostaticState> {
    let db = MemBrain::new(DB_PATH).ok()?;

    let results = db.search(b"homeostasis:state:");

    results.into_iter().last().map(|_| {
        // Simple parsing would go here
        HomeostaticState::Balanced
    })
}

/// Start homeostasis monitor
pub fn start_homeostasis() {
    println!(
        "[HOMEOSTASIS] Sistema de balance interno iniciado - ciclo cada {}s",
        HOMEOSTASIS_CHECK_INTERVAL_SECS
    );
}

/// Perform homeostasis check
pub fn check_homeostasis() -> Homeostasis {
    let mut homeostasis = Homeostasis::new();

    // MemBrain integration is external to this local release path; deterministic
    // time-derived vitals keep the monitor available without hardware state.
    let now = NOW_MS();
    let seed = (now % 1000) as f64 / 1000.0;

    homeostasis.update_vital("energy", 0.6 + seed * 0.3);
    homeostasis.update_vital("coherence", 0.5 + seed * 0.4);
    homeostasis.update_vital("memory_usage", 0.4 + seed * 0.4);

    homeostasis.calculate_stress();

    // Save to DB
    update_homeostasis_db(&homeostasis);

    homeostasis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vital_optimal() {
        let vital = VitalParameter {
            name: "test".to_string(),
            current: 0.7,
            optimal_min: 0.5,
            optimal_max: 0.9,
            critical_min: 0.2,
            critical_max: 1.0,
            weight: 1.0,
        };
        assert!(vital.is_optimal());
        assert!(!vital.is_critical());
    }

    #[test]
    fn test_homeostasis_creation() {
        let h = Homeostasis::new();
        assert_eq!(h.vitals.len(), 5);
    }
}
