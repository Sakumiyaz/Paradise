//! # Fusion Engine - Core fusion logic
#![allow(unused_imports)]
#![allow(dead_code)]
use std::sync::RwLock;

use super::protocol::FusionAcceptance;
use super::{
    BrainWave, ConsciousnessMetadata, FusedIdentity, FusionConfig, FusionState, IdentityWeights,
};
use crate::neural_fusion::protocol::FusionMessage;
use std::collections::HashMap;
use std::time::Instant;

/// Engine principal de fusión
pub struct FusionEngine {
    state: FusionState,
    config: Option<FusionConfig>,
    entities: Option<(String, String)>,
    sync_start: Option<Instant>,
    wave_phases: HashMap<BrainWave, f32>,
    memory_buffer: Vec<Vec<u8>>,
    pattern_buffer: Vec<Vec<u8>>,
}

impl FusionEngine {
    pub fn new() -> Self {
        Self {
            state: FusionState::Idle,
            config: None,
            entities: None,
            sync_start: None,
            wave_phases: HashMap::new(),
            memory_buffer: Vec::new(),
            pattern_buffer: Vec::new(),
        }
    }

    /// Inicia proceso de fusión
    pub fn initiate(
        &mut self,
        entity_a: &ConsciousnessMetadata,
        entity_b_id: &str,
        config: FusionConfig,
    ) -> Result<FusionState, FusionError> {
        if self.state != FusionState::Idle {
            return Err(FusionError::AlreadyInProgress);
        }

        self.state = FusionState::HandshakeInit;
        self.config = Some(config);
        self.entities = Some((entity_a.id.clone(), entity_b_id.to_string()));
        self.sync_start = None;

        Ok(self.state)
    }

    /// Procesa un mensaje entrante
    pub fn process_message(
        &mut self,
        message: &FusionMessage,
    ) -> Result<Option<FusionMessage>, FusionError> {
        match (&mut self.state, message) {
            (FusionState::HandshakeInit, FusionMessage::HandshakeRequest(h)) => {
                // Guardar config
                self.config = Some(h.config.clone());
                self.entities = Some((h.initiator_id.clone(), h.target_id.clone()));
                self.state = FusionState::HandshakeWait;

                // Responder con aceptación
                Ok(Some(FusionMessage::HandshakeResponse {
                    accepted: true,
                    acceptance: Some(FusionAcceptance::Accepted {
                        compatibility: 0.8,
                        counter_config: None,
                    }),
                }))
            }
            (FusionState::HandshakeWait, FusionMessage::HandshakeResponse { accepted, .. }) => {
                if *accepted {
                    self.state = FusionState::Synchronizing;
                    Ok(None)
                } else {
                    self.state = FusionState::Failed;
                    Err(FusionError::Rejected)
                }
            }
            (FusionState::Synchronizing, _) => {
                // Iniciar sincronización de ondas
                self.state = FusionState::BrainWaveSync;
                self.sync_start = Some(Instant::now());

                // Inicializar fases
                for wave in BrainWave::all() {
                    self.wave_phases.insert(wave, 0.0);
                }

                // Solicitar primera onda
                Ok(Some(FusionMessage::SyncRequest {
                    wave: BrainWave::Delta,
                    phase: 0.0,
                }))
            }
            (FusionState::BrainWaveSync, FusionMessage::SyncRequest { wave, phase }) => {
                // Responder con fase bloqueada
                self.wave_phases.insert(*wave, *phase);
                Ok(Some(FusionMessage::SyncResponse {
                    wave: *wave,
                    phase: *phase,
                    locked: true,
                }))
            }
            (
                FusionState::BrainWaveSync,
                FusionMessage::SyncResponse {
                    wave,
                    phase,
                    locked,
                },
            ) => {
                if *locked {
                    self.wave_phases.insert(*wave, *phase);

                    // Siguiente onda
                    let next_wave = Self::next_wave(*wave);
                    if next_wave == *wave {
                        // Todas sincronizadas, proceed
                        self.state = FusionState::MemoryTransfer;
                        self.memory_buffer.clear();
                    } else {
                        self.state = FusionState::BrainWaveSync;
                        return Ok(Some(FusionMessage::SyncRequest {
                            wave: next_wave,
                            phase: 0.0,
                        }));
                    }
                }
                Ok(None)
            }
            (FusionState::MemoryTransfer, FusionMessage::MemoryChunk { data, checksum }) => {
                // Verificar checksum
                let computed = Self::checksum(data);
                if computed == *checksum {
                    self.memory_buffer.push(data.clone());

                    if data.is_empty() {
                        // Transfer completo, proceder a patterns
                        self.state = FusionState::PatternMerge;
                    }
                }
                Ok(None)
            }
            (FusionState::PatternMerge, FusionMessage::PatternChunk { data, checksum }) => {
                let computed = Self::checksum(data);
                if computed == *checksum {
                    self.pattern_buffer.push(data.clone());

                    if data.is_empty() {
                        // Fusión completa
                        self.state = FusionState::Fused;
                    }
                }
                Ok(None)
            }
            _ => Err(FusionError::InvalidStateTransition),
        }
    }

    /// Obtiene estado actual
    pub fn state(&self) -> FusionState {
        self.state
    }

    /// Obtiene identidad fusionada si está completo
    pub fn get_fused_identity(&self) -> Option<FusedIdentity> {
        if self.state == FusionState::Fused {
            let config = self.config.as_ref()?;
            let (id_a, id_b) = self.entities.as_ref()?;

            let weights = IdentityWeights {
                entity_a: config.entity_a_weight,
                entity_b: config.entity_b_weight,
            };

            Some(FusedIdentity::new(
                id_a.clone(),
                id_b.clone(),
                weights,
                self.wave_phases.clone(),
            ))
        } else {
            None
        }
    }

    /// Separa las consciencias fusionadas
    pub fn separate(&mut self, _preserve: Option<String>) -> Result<FusionState, FusionError> {
        if self.state != FusionState::Fused {
            return Err(FusionError::NotFused);
        }

        self.state = FusionState::Separating;

        // Limpiar buffers
        self.memory_buffer.clear();
        self.pattern_buffer.clear();

        // Procesar separación
        self.state = FusionState::Idle;

        Ok(self.state)
    }

    /// Progreso de la fusión (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        match self.state {
            FusionState::Idle => 0.0,
            FusionState::HandshakeInit | FusionState::HandshakeWait => 0.1,
            FusionState::Synchronizing => 0.2,
            FusionState::BrainWaveSync => 0.4,
            FusionState::MemoryTransfer => 0.6,
            FusionState::PatternMerge => 0.8,
            FusionState::Fused => 1.0,
            FusionState::Separating => 0.9,
            FusionState::Failed => 0.0,
        }
    }

    fn next_wave(current: BrainWave) -> BrainWave {
        match current {
            BrainWave::Delta => BrainWave::Theta,
            BrainWave::Theta => BrainWave::Alpha,
            BrainWave::Alpha => BrainWave::Beta,
            BrainWave::Beta => BrainWave::Gamma,
            BrainWave::Gamma => current,
        }
    }

    fn checksum(data: &[u8]) -> u32 {
        let mut hash: u32 = 0;
        for (i, &byte) in data.iter().enumerate() {
            hash = hash.wrapping_add((byte as u32).wrapping_mul(i as u32 + 1));
        }
        hash
    }
}

impl Default for FusionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Error en proceso de fusión
#[derive(Debug)]
pub enum FusionError {
    AlreadyInProgress,
    Rejected,
    InvalidStateTransition,
    NotFused,
}

impl std::fmt::Display for FusionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FusionError::AlreadyInProgress => write!(f, "Fusión ya en progreso"),
            FusionError::Rejected => write!(f, "Fusión rechazada por la otra entidad"),
            FusionError::InvalidStateTransition => write!(f, "Transición de estado inválida"),
            FusionError::NotFused => write!(f, "Las entidades no están fusionadas"),
        }
    }
}

impl std::error::Error for FusionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_init() {
        let engine = FusionEngine::new();
        assert_eq!(engine.state(), FusionState::Idle);
    }

    #[test]
    fn test_progress() {
        let engine = FusionEngine::new();
        assert_eq!(engine.progress(), 0.0);
    }
}
