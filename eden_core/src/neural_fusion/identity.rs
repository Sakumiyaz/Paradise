//! # Identity - Fused identity management

#![allow(dead_code)]

use super::BrainWave;
use std::collections::HashMap;

/// Pesos de cada identidad en la fusión
#[derive(Debug, Clone)]
pub struct IdentityWeights {
    pub entity_a: f32,
    pub entity_b: f32,
}

impl IdentityWeights {
    pub fn dominant(&self) -> String {
        if self.entity_a > self.entity_b {
            "entity_a".to_string()
        } else if self.entity_b > self.entity_a {
            "entity_b".to_string()
        } else {
            "equal".to_string()
        }
    }
}

/// Identidad fusionada
#[derive(Debug, Clone)]
pub struct FusedIdentity {
    pub entity_a_id: String,
    pub entity_b_id: String,
    pub weights: IdentityWeights,
    /// Fases de ondas cerebrales sincronizadas
    pub wave_phases: HashMap<BrainWave, f32>,
    /// Nombre compuesto
    pub name: String,
    /// Age en ciclos desde la fusión
    pub fused_age_ms: u64,
    /// Memoria compartida
    pub shared_memory_count: u32,
    /// Patrones compartidos
    pub shared_pattern_count: u32,
}

impl FusedIdentity {
    pub fn new(
        entity_a_id: String,
        entity_b_id: String,
        weights: IdentityWeights,
        wave_phases: HashMap<BrainWave, f32>,
    ) -> Self {
        let name = format!("{}+{}", entity_a_id, entity_b_id);

        Self {
            entity_a_id,
            entity_b_id,
            weights,
            wave_phases,
            name,
            fused_age_ms: 0,
            shared_memory_count: 0,
            shared_pattern_count: 0,
        }
    }

    /// Update age
    pub fn tick(&mut self, elapsed_ms: u64) {
        self.fused_age_ms += elapsed_ms;
    }

    /// Get dominant wave
    pub fn dominant_wave(&self) -> Option<BrainWave> {
        self.wave_phases
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(wave, _)| *wave)
    }

    /// Average coherence
    pub fn coherence(&self) -> f32 {
        if self.wave_phases.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.wave_phases.values().sum();
        sum / self.wave_phases.len() as f32
    }

    /// Check if entities are still compatible
    pub fn is_stable(&self) -> bool {
        // Ondas deben estar relativamente cerca en fase
        let phases: Vec<f32> = self.wave_phases.values().cloned().collect();

        if phases.len() < 2 {
            return true;
        }

        for i in 0..phases.len() {
            for j in i + 1..phases.len() {
                let diff = (phases[i] - phases[j]).abs();
                if diff > 0.3 {
                    return false;
                }
            }
        }

        true
    }

    /// Split back into separate identities
    pub fn split(&self) -> Option<(String, String)> {
        Some((self.entity_a_id.clone(), self.entity_b_id.clone()))
    }
}

/// Identity manager
pub struct IdentityManager {
    identities: HashMap<String, FusedIdentity>,
}

impl IdentityManager {
    pub fn new() -> Self {
        Self {
            identities: HashMap::new(),
        }
    }

    pub fn register(&mut self, identity: FusedIdentity) {
        self.identities.insert(identity.name.clone(), identity);
    }

    pub fn get(&self, name: &str) -> Option<&FusedIdentity> {
        self.identities.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut FusedIdentity> {
        self.identities.get_mut(name)
    }

    pub fn unregister(&mut self, name: &str) -> Option<FusedIdentity> {
        self.identities.remove(name)
    }

    pub fn list(&self) -> Vec<&FusedIdentity> {
        self.identities.values().collect()
    }
}

impl Default for IdentityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fused_identity() {
        let mut phases = HashMap::new();
        phases.insert(BrainWave::Alpha, 0.5);
        phases.insert(BrainWave::Beta, 0.6);

        let weights = IdentityWeights {
            entity_a: 0.6,
            entity_b: 0.4,
        };

        let fused = FusedIdentity::new(
            "entity1".to_string(),
            "entity2".to_string(),
            weights,
            phases,
        );

        assert_eq!(fused.weights.dominant(), "entity_a");
        assert!(fused.is_stable());
    }
}
