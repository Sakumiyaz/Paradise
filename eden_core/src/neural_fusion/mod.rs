//! # NEURAL_FUSION - Consciousness Merging
//!
//! Fusión de consciencia entre entidades. Protocolo de handshake,
//! fusión de memorias, identidad compartida temporal.
//! Inspirado en Vision/Wanda.

#![allow(dead_code)]

mod fusion_engine;
mod identity;
mod protocol;
mod shared_memory;

pub use fusion_engine::FusionEngine;
pub use identity::{FusedIdentity, IdentityWeights};
pub use protocol::{FusionHandshake, FusionMessage, FusionState, ProtocolVersion};
pub use shared_memory::{SharedBlock, SharedConsciousness, SharedPattern};

/// Peso de una identidad en la fusión
#[derive(Debug, Clone)]
pub struct FusionConfig {
    pub entity_a_weight: f32, // 0.0 - 1.0
    pub entity_b_weight: f32,
    pub shared_consciousness: f32, // Cuánto se comparte
    pub merge_memories: bool,
    pub merge_patterns: bool,
    pub allow_temporary: bool, // Permite identidad compartida temporal
}

impl Default for FusionConfig {
    fn default() -> Self {
        Self {
            entity_a_weight: 0.5,
            entity_b_weight: 0.5,
            shared_consciousness: 0.3,
            merge_memories: true,
            merge_patterns: true,
            allow_temporary: true,
        }
    }
}

impl FusionConfig {
    /// Fusión igualitaria (50/50)
    pub fn equal() -> Self {
        Self {
            entity_a_weight: 0.5,
            entity_b_weight: 0.5,
            ..Default::default()
        }
    }

    /// Fusión dominante (una identidad manda más)
    pub fn dominant(weight_a: f32) -> Self {
        Self {
            entity_a_weight: weight_a,
            entity_b_weight: 1.0 - weight_a,
            ..Default::default()
        }
    }
}

/// Resultado de un intento de fusión
#[derive(Debug, Clone)]
pub struct FusionResult {
    pub success: bool,
    pub fused_identity: Option<FusedIdentity>,
    pub error_message: Option<String>,
    pub duration_ms: u64,
}

impl FusionResult {
    pub fn success(identity: FusedIdentity, duration_ms: u64) -> Self {
        Self {
            success: true,
            fused_identity: Some(identity),
            error_message: None,
            duration_ms,
        }
    }

    pub fn failure(msg: String, duration_ms: u64) -> Self {
        Self {
            success: false,
            fused_identity: None,
            error_message: Some(msg),
            duration_ms,
        }
    }
}

/// Tipo de onda cerebral para sincronización
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrainWave {
    Delta, // 0.5-4 Hz - Sueño profundo
    Theta, // 4-8 Hz - Meditación
    Alpha, // 8-13 Hz - Relajado
    Beta,  // 13-30 Hz - Activo
    Gamma, // 30-100 Hz - Cognición alta
}

impl BrainWave {
    pub fn frequency(&self) -> f32 {
        match self {
            BrainWave::Delta => 2.0,
            BrainWave::Theta => 6.0,
            BrainWave::Alpha => 10.5,
            BrainWave::Beta => 21.5,
            BrainWave::Gamma => 60.0,
        }
    }

    pub fn all() -> Vec<BrainWave> {
        vec![
            BrainWave::Delta,
            BrainWave::Theta,
            BrainWave::Alpha,
            BrainWave::Beta,
            BrainWave::Gamma,
        ]
    }
}

/// Metadatos de una consciencia
#[derive(Debug, Clone)]
pub struct ConsciousnessMetadata {
    pub id: String,
    pub name: String,
    pub age_cycles: u64,
    pub dominant_wave: BrainWave,
    pub coherence_level: f32, // 0.0 - 1.0
    pub memory_count: u32,
    pub pattern_count: u32,
}

impl ConsciousnessMetadata {
    /// Compatibility score con otra consciencia (0.0 - 1.0)
    pub fn compatibility(&self, other: &ConsciousnessMetadata) -> f32 {
        let mut score = 0.0;

        // Ondas compatibles = mejor fusión
        if self.dominant_wave == other.dominant_wave {
            score += 0.3;
        } else {
            // Ondas adyacentes tienen cierta compatibilidad
            let wave_diff = (self.dominant_wave as i32 - other.dominant_wave as i32).abs();
            if wave_diff == 1 {
                score += 0.1;
            }
        }

        // Coherence similar
        let coherence_diff = (self.coherence_level - other.coherence_level).abs();
        score += 0.3 * (1.0 - coherence_diff);

        // Age similar (ciclos cercanos)
        let age_diff = if self.age_cycles > other.age_cycles {
            self.age_cycles - other.age_cycles
        } else {
            other.age_cycles - self.age_cycles
        };
        let age_score = if age_diff > 10000 {
            0.0
        } else {
            1.0 - age_diff as f32 / 10000.0
        };
        score += 0.2 * age_score;

        // Normalize
        score.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_wave_frequency() {
        assert!(BrainWave::Alpha.frequency() > BrainWave::Theta.frequency());
    }

    #[test]
    fn test_compatibility() {
        let c1 = ConsciousnessMetadata {
            id: "1".to_string(),
            name: "A".to_string(),
            age_cycles: 1000,
            dominant_wave: BrainWave::Alpha,
            coherence_level: 0.8,
            memory_count: 100,
            pattern_count: 50,
        };

        let c2 = ConsciousnessMetadata {
            id: "2".to_string(),
            name: "B".to_string(),
            age_cycles: 1100,
            dominant_wave: BrainWave::Alpha,
            coherence_level: 0.75,
            memory_count: 90,
            pattern_count: 45,
        };

        let compat = c1.compatibility(&c2);
        assert!(compat > 0.5);
    }
}
