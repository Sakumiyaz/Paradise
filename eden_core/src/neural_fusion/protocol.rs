//! # Fusion Protocol - Handshake and state management

#![allow(dead_code)]

use super::{BrainWave, ConsciousnessMetadata, FusionConfig};

/// Versión del protocolo de fusión
#[derive(Debug, Clone, Copy)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl ProtocolVersion {
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    pub fn current() -> Self {
        Self { major: 1, minor: 0 }
    }

    pub fn is_compatible(&self, other: &ProtocolVersion) -> bool {
        self.major == other.major
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::current()
    }
}

/// Estado del proceso de fusión
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FusionState {
    /// Sin fusión activa
    Idle,
    /// Iniciando handshake
    HandshakeInit,
    /// Esperando respuesta del otro
    HandshakeWait,
    /// Handshake completado, iniciando sincronización
    Synchronizing,
    /// Sincronizando ondas cerebrales
    BrainWaveSync,
    /// Transfiriendo memorias
    MemoryTransfer,
    /// Fusionando patrones
    PatternMerge,
    /// Fusión completa
    Fused,
    /// En proceso de separación
    Separating,
    /// Fusión fallida
    Failed,
}

impl FusionState {
    pub fn is_active(&self) -> bool {
        !matches!(self, FusionState::Idle | FusionState::Failed)
    }
}

/// Handshake de fusión
#[derive(Debug, Clone)]
pub struct FusionHandshake {
    pub version: ProtocolVersion,
    pub initiator_id: String,
    pub target_id: String,
    pub config: FusionConfig,
    pub metadata: ConsciousnessMetadata,
    pub timestamp_ms: u64,
}

impl FusionHandshake {
    /// Crea nuevo handshake
    pub fn new(initiator: &ConsciousnessMetadata, target_id: &str, config: FusionConfig) -> Self {
        Self {
            version: ProtocolVersion::current(),
            initiator_id: initiator.id.clone(),
            target_id: target_id.to_string(),
            config,
            metadata: initiator.clone(),
            timestamp_ms: current_timestamp_ms(),
        }
    }

    /// Verifica si el handshake es válido
    pub fn is_valid(&self) -> bool {
        self.version.major == ProtocolVersion::current().major
    }

    /// Responde al handshake
    pub fn accept(&self, metadata: &ConsciousnessMetadata) -> FusionAcceptance {
        if !self.is_valid() {
            return FusionAcceptance::Rejected("Version mismatch".to_string());
        }

        let compatibility = self.metadata.compatibility(metadata);

        if compatibility < 0.3 {
            return FusionAcceptance::Rejected("Low compatibility".to_string());
        }

        FusionAcceptance::Accepted {
            compatibility,
            counter_config: Some(FusionConfig::default()),
        }
    }
}

/// Respuesta a un handshake
#[derive(Debug, Clone)]
pub enum FusionAcceptance {
    Accepted {
        compatibility: f32,
        counter_config: Option<FusionConfig>,
    },
    Rejected(String),
}

/// Mensaje del protocolo de fusión
#[derive(Debug, Clone)]
pub enum FusionMessage {
    HandshakeRequest(FusionHandshake),
    HandshakeResponse {
        accepted: bool,
        acceptance: Option<FusionAcceptance>,
    },
    SyncRequest {
        wave: BrainWave,
        phase: f32,
    },
    SyncResponse {
        wave: BrainWave,
        phase: f32,
        locked: bool,
    },
    MemoryChunk {
        data: Vec<u8>,
        checksum: u32,
    },
    PatternChunk {
        data: Vec<u8>,
        checksum: u32,
    },
    SeparationRequest {
        preserve_identity: Option<String>,
    },
    SeparationComplete {
        resulting_state: FusionState,
    },
}

impl FusionMessage {
    /// Serializa mensaje
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();

        match self {
            FusionMessage::HandshakeRequest(h) => {
                result.push(0x01);
                result.extend_from_slice(&h.initiator_id.as_bytes());
                result.push(0);
                result.extend_from_slice(&h.target_id.as_bytes());
                result.push(0);
                result.extend_from_slice(&h.config.entity_a_weight.to_le_bytes());
            }
            FusionMessage::HandshakeResponse { accepted, .. } => {
                result.push(0x02);
                result.push(if *accepted { 1 } else { 0 });
            }
            FusionMessage::SyncRequest { wave, phase } => {
                result.push(0x10);
                result.push(*wave as u8);
                result.extend_from_slice(&phase.to_le_bytes());
            }
            FusionMessage::SyncResponse {
                wave,
                phase,
                locked,
            } => {
                result.push(0x11);
                result.push(*wave as u8);
                result.extend_from_slice(&phase.to_le_bytes());
                result.push(if *locked { 1 } else { 0 });
            }
            _ => {
                result.push(0xFF);
            }
        }

        result
    }
}

/// Timestamp actual en milisegundos
fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_validity() {
        let handshake = FusionHandshake {
            version: ProtocolVersion::current(),
            initiator_id: "test".to_string(),
            target_id: "other".to_string(),
            config: FusionConfig::default(),
            metadata: ConsciousnessMetadata {
                id: "test".to_string(),
                name: "Test".to_string(),
                age_cycles: 100,
                dominant_wave: BrainWave::Alpha,
                coherence_level: 0.8,
                memory_count: 50,
                pattern_count: 25,
            },
            timestamp_ms: 0,
        };

        assert!(handshake.is_valid());
    }

    #[test]
    fn test_message_serialize() {
        let msg = FusionMessage::SyncRequest {
            wave: BrainWave::Alpha,
            phase: 0.5,
        };

        let bytes = msg.serialize();
        assert!(!bytes.is_empty());
    }
}
