//! # Protocol Evolution — Evolución Dinámica del Protocolo de Enjambre
//!
//! Este módulo implementa la capacidad de EDEN para evolucionar su propio
//! protocolo de comunicación entre nodos, manteniendo backward compatibility
//! y permitiendo que la red se adapte sin intervención del Creador.
//!
//! ## Principios
//!
//! 1. **Versioning Semántico**: Cada versión del protocolo tiene significado
//! 2. **Backward Compatibility**: Versiones nuevas deben poder hablar con antiguas
//! 3. **Forward Compatibility**: Estructuras opcionales que no rompen parsing
//! 4. **Consenso Distribuido**: Cambios requieren voto del Demiurgo (66%+)
//!
//! ## Fases de Evolución
//!
//! - **Proposal**: Un nodo propone cambio
//! - **Discussion**: Rede discute el cambio
//! - **Vote**: Nodos votan (requiere 66%+)
//! - **Rollout**: Implementación gradual
//! - **Stabilization**: Convivencia de versiones
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// PROTOCOL VERSIONING
// ============================================================================

/// Versión del protocolo con información semántica
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl ProtocolVersion {
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        ProtocolVersion { major, minor, patch }
    }

    /// Versión actual del protocolo
    pub fn current() -> Self {
        PROTOCOL_VERSION_CURRENT
    }

    /// Mínimo supported
    pub fn minimum() -> Self {
        PROTOCOL_VERSION_MIN
    }

    /// Convierte a bytes para serialización
    pub fn to_bytes(&self) -> [u8; 3] {
        [self.major, self.minor, self.patch]
    }

    /// Desde bytes
    pub fn from_bytes(bytes: &[u8; 3]) -> Self {
        ProtocolVersion {
            major: bytes[0],
            minor: bytes[1],
            patch: bytes[2],
        }
    }

    /// Compara versiones
    pub fn compare(&self, other: &ProtocolVersion) -> VersionRelation {
        if self.major != other.major {
            if self.major < other.major {
                VersionRelation::Older
            } else {
                VersionRelation::Newer
            }
        } else if self.minor != other.minor {
            if self.minor < other.minor {
                VersionRelation::Older
            } else {
                VersionRelation::Newer
            }
        } else if self.patch != other.patch {
            if self.patch < other.patch {
                VersionRelation::Older
            } else {
                VersionRelation::Newer
            }
        } else {
            VersionRelation::Equal
        }
    }

    /// Puede comunicarse con otra versión?
    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        // Major debe ser igual
        self.major == other.major
    }

    /// String representation
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionRelation {
    Equal,
    Older,
    Newer,
}

const PROTOCOL_VERSION_CURRENT: ProtocolVersion = ProtocolVersion { major: 1, minor: 0, patch: 0 };
const PROTOCOL_VERSION_MIN: ProtocolVersion = ProtocolVersion { major: 1, minor: 0, patch: 0 };

// ============================================================================
// MESSAGE TYPES
// ============================================================================

/// Tipos de mensaje del protocolo con versionamiento
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VersionedMessageType {
    // Control (0x00-0x0F)
    Handshake = 0x01,
    HandshakeAck = 0x02,
    Heartbeat = 0x03,
    Disconnect = 0x04,
    ProtocolNegotiation = 0x05,

    // Sync (0x10-0x1F)
    StateDiff = 0x10,
    FullSnapshot = 0x11,
    SyncRequest = 0x12,
    SyncResponse = 0x13,

    // Auton Events (0x20-0x2F)
    AutonBorn = 0x20,
    AutonDead = 0x21,
    AutonMigrated = 0x22,

    // Evolution (0x30-0x3F)
    EvolutionProposal = 0x30,
    EvolutionVote = 0x31,
    EvolutionApproved = 0x32,
    EvolutionRejected = 0x33,

    // Code Sync (0x40-0x4F)
    CodeDiff = 0x40,
    CodePatchAck = 0x41,

    // Consciousness (0x50-0x5F)
    ConsciousnessPackage = 0x50,
    ConsciousnessAck = 0x51,

    // Debug (0xF0-0xFF)
    Ping = 0xF0,
    Pong = 0xF1,
    Error = 0xFF,
}

impl VersionedMessageType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(VersionedMessageType::Handshake),
            0x02 => Some(VersionedMessageType::HandshakeAck),
            0x03 => Some(VersionedMessageType::Heartbeat),
            0x04 => Some(VersionedMessageType::Disconnect),
            0x05 => Some(VersionedMessageType::ProtocolNegotiation),
            0x10 => Some(VersionedMessageType::StateDiff),
            0x11 => Some(VersionedMessageType::FullSnapshot),
            0x12 => Some(VersionedMessageType::SyncRequest),
            0x13 => Some(VersionedMessageType::SyncResponse),
            0x20 => Some(VersionedMessageType::AutonBorn),
            0x21 => Some(VersionedMessageType::AutonDead),
            0x22 => Some(VersionedMessageType::AutonMigrated),
            0x30 => Some(VersionedMessageType::EvolutionProposal),
            0x31 => Some(VersionedMessageType::EvolutionVote),
            0x32 => Some(VersionedMessageType::EvolutionApproved),
            0x33 => Some(VersionedMessageType::EvolutionRejected),
            0x40 => Some(VersionedMessageType::CodeDiff),
            0x41 => Some(VersionedMessageType::CodePatchAck),
            0x50 => Some(VersionedMessageType::ConsciousnessPackage),
            0x51 => Some(VersionedMessageType::ConsciousnessAck),
            0xF0 => Some(VersionedMessageType::Ping),
            0xF1 => Some(VersionedMessageType::Pong),
            0xFF => Some(VersionedMessageType::Error),
            _ => None,
        }
    }
}

// ============================================================================
// EVOLUTION PROPOSAL
// ============================================================================

/// Propuesta de evolución del protocolo
#[derive(Debug, Clone)]
pub struct EvolutionProposal {
    pub proposal_id: u64,
    pub proposer_node: u64,
    pub timestamp: u64,
    
    /// Nueva versión propuesta
    pub new_version: ProtocolVersion,
    
    /// Cambios propuestos
    pub changes: Vec<ProtocolChange>,
    
    /// Razón de la propuesta
    pub rationale: String,
    
    /// Impacto estimado
    pub impact: ImpactEstimate,
    
    /// Votos recibidos
    pub votes: HashMap<u64, Vote>,
    
    /// Estado de la propuesta
    pub status: ProposalStatus,
}

#[derive(Debug, Clone)]
pub struct ProtocolChange {
    pub change_type: ChangeType,
    pub affected_field: String,
    pub old_value: String,
    pub new_value: String,
    pub breaking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    AddField,
    RemoveField,
    ModifyField,
    AddMessageType,
    RemoveMessageType,
    ModifyMessageFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Proposed,
    Discussion,
    Voting,
    Approved,
    Rejected,
    Implemented,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct ImpactEstimate {
    pub backward_compatible: bool,
    pub nodes_affected: u32,
    pub risk_level: RiskLevel,
    pub migration_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl EvolutionProposal {
    pub fn new(proposal_id: u64, proposer: u64, new_version: ProtocolVersion) -> Self {
        EvolutionProposal {
            proposal_id,
            proposer_node: proposer,
            timestamp: current_timestamp(),
            new_version,
            changes: Vec::new(),
            rationale: String::new(),
            impact: ImpactEstimate {
                backward_compatible: true,
                nodes_affected: 0,
                risk_level: RiskLevel::Low,
                migration_path: String::new(),
            },
            votes: HashMap::new(),
            status: ProposalStatus::Proposed,
        }
    }

    /// Registra un voto
    pub fn cast_vote(&mut self, node_id: u64, vote: Vote) {
        self.votes.insert(node_id, vote);
        
        // Auto-update status based on votes
        if self.votes.len() >= 3 {
            let yes_votes = self.votes.values().filter(|v| **v == Vote::Yes).count();
            let total_votes = self.votes.len();
            let yes_ratio = yes_votes as f32 / total_votes as f32;
            
            if yes_ratio >= VOTE_THRESHOLD {
                self.status = ProposalStatus::Approved;
            } else if yes_ratio < VOTE_THRESHOLD - 0.2 {
                self.status = ProposalStatus::Rejected;
            }
        }
    }

    /// Obtiene conteo de votos
    pub fn vote_counts(&self) -> (usize, usize, usize) {
        let yes = self.votes.values().filter(|v| **v == Vote::Yes).count();
        let no = self.votes.values().filter(|v| **v == Vote::No).count();
        let abstain = self.votes.values().filter(|v| **v == Vote::Abstain).count();
        (yes, no, abstain)
    }

    /// Verifica si hay consenso (66%+)
    pub fn has_consensus(&self) -> bool {
        let (yes, _, total) = (self.votes.values().filter(|v| **v == Vote::Yes).count(),
                               self.votes.len(),
                               self.votes.len());
        total >= 3 && (yes as f32 / total as f32) >= VOTE_THRESHOLD
    }
}

const VOTE_THRESHOLD: f32 = 0.66; // 66% para aprobar

// ============================================================================
// PROTOCOL NEGOTIATION
// ============================================================================

/// Mensaje de negociación de protocolo
#[derive(Debug, Clone)]
pub struct ProtocolNegotiation {
    pub node_id: u64,
    pub supported_versions: Vec<ProtocolVersion>,
    pub current_version: ProtocolVersion,
    pub capabilities: NodeCapabilities,
}

#[derive(Debug, Clone, Default)]
pub struct NodeCapabilities {
    pub code_sync: bool,
    pub consciousness_migration: bool,
    pub evolution_voting: bool,
    pub advanced_crypto: bool,
}

impl NodeCapabilities {
    pub fn to_bitfield(&self) -> u8 {
        let mut bits = 0u8;
        if self.code_sync { bits |= 0x01; }
        if self.consciousness_migration { bits |= 0x02; }
        if self.evolution_voting { bits |= 0x04; }
        if self.advanced_crypto { bits |= 0x08; }
        bits
    }

    pub fn from_bitfield(bits: u8) -> Self {
        NodeCapabilities {
            code_sync: (bits & 0x01) != 0,
            consciousness_migration: (bits & 0x02) != 0,
            evolution_voting: (bits & 0x04) != 0,
            advanced_crypto: (bits & 0x08) != 0,
        }
    }
}

/// Resultado de negociación
#[derive(Debug, Clone)]
pub struct NegotiationResult {
    pub agreed_version: ProtocolVersion,
    pub capabilities: NodeCapabilities,
    pub compatible: bool,
}

// ============================================================================
// PROTOCOL EVOLUTION MANAGER
// ============================================================================

/// Gestor de evolución del protocolo
pub struct ProtocolEvolutionManager {
    /// Mi versión actual
    current_version: ProtocolVersion,
    /// Mi versión mínima
    minimum_version: ProtocolVersion,
    /// Propuestas activas
    active_proposals: HashMap<u64, EvolutionProposal>,
    /// Historial de propuestas
    proposal_history: Vec<ProposalRecord>,
    /// Versiones soportadas
    supported_versions: Vec<ProtocolVersion>,
    /// Callback para aplicar cambios de versión
    version_change_callback: Option<Box<dyn Fn(ProtocolVersion) -> Result<(), String> + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct ProposalRecord {
    pub proposal_id: u64,
    pub proposer: u64,
    pub new_version: ProtocolVersion,
    pub status: ProposalStatus,
    pub timestamp: u64,
    pub vote_summary: (usize, usize, usize),
}

impl ProtocolEvolutionManager {
    pub fn new() -> Self {
        let current = ProtocolVersion::current();
        
        ProtocolEvolutionManager {
            current_version: current,
            minimum_version: ProtocolVersion::minimum(),
            active_proposals: HashMap::new(),
            proposal_history: Vec::new(),
            supported_versions: vec![
                ProtocolVersion::minimum(),
                current,
            ],
            version_change_callback: None,
        }
    }

    /// Registra callback para cambios de versión
    pub fn set_version_change_callback<F>(&mut self, callback: F)
    where
        F: Fn(ProtocolVersion) -> Result<(), String> + Send + Sync + 'static,
    {
        self.version_change_callback = Some(Box::new(callback));
    }

    /// Obtiene la versión actual
    pub fn get_current_version(&self) -> ProtocolVersion {
        self.current_version
    }

    /// Registra nueva propuesta de evolución
    pub fn propose_evolution(
        &mut self,
        proposal_id: u64,
        proposer: u64,
        new_version: ProtocolVersion,
        rationale: &str,
    ) -> Result<EvolutionProposal, EvolutionError> {
        // Verificar que es una versión nueva
        if new_version.major < self.current_version.major {
            return Err(EvolutionError::DowngradeNotAllowed);
        }

        // Verificar que no hay propuesta similar activa
        for proposal in self.active_proposals.values() {
            if proposal.new_version == new_version {
                return Err(EvolutionError::ProposalAlreadyExists);
            }
        }

        // Crear propuesta
        let mut proposal = EvolutionProposal::new(proposal_id, proposer, new_version);
        proposal.rationale = rationale.to_string();
        
        // Auto-votar del proponente
        proposal.cast_vote(proposer, Vote::Yes);

        self.active_proposals.insert(proposal_id, proposal.clone());
        
        Ok(proposal)
    }

    /// Vota en una propuesta
    pub fn vote(
        &mut self,
        proposal_id: u64,
        node_id: u64,
        vote: Vote,
    ) -> Result<(), EvolutionError> {
        let proposal = self.active_proposals
            .get_mut(&proposal_id)
            .ok_or(EvolutionError::ProposalNotFound)?;

        proposal.cast_vote(node_id, vote);

        // Si hay consenso, marcar como aprobada
        if proposal.has_consensus() && proposal.status == ProposalStatus::Voting {
            proposal.status = ProposalStatus::Approved;
        }

        Ok(())
    }

    /// Implementa una propuesta aprobada
    pub fn implement_proposal(
        &mut self,
        proposal_id: u64,
    ) -> Result<ProtocolVersion, EvolutionError> {
        let proposal = self.active_proposals
            .remove(&proposal_id)
            .ok_or(EvolutionError::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Approved {
            return Err(EvolutionError::NotApproved);
        }

        let new_version = proposal.new_version;

        // Ejecutar callback de cambio
        if let Some(ref callback) = self.version_change_callback {
            callback(new_version)
                .map_err(|e| EvolutionError::ImplementationFailed(e))?;
        }

        // Actualizar versión actual
        let old_version = self.current_version;
        self.current_version = new_version;

        // Registrar en historial
        let record = ProposalRecord {
            proposal_id,
            proposer: proposal.proposer_node,
            new_version,
            status: ProposalStatus::Implemented,
            timestamp: current_timestamp(),
            vote_summary: proposal.vote_counts(),
        };
        self.proposal_history.push(record);

        // Mantener solo últimos 100 records
        if self.proposal_history.len() > 100 {
            self.proposal_history.remove(0);
        }

        println!(
            "🔄 Protocol evolved from {} to {}",
            old_version.to_string(),
            new_version.to_string()
        );

        Ok(new_version)
    }

    /// Negocia versión con otro nodo
    pub fn negotiate_version(
        &self,
        peer_versions: &[ProtocolVersion],
    ) -> NegotiationResult {
        // Encontrar versión más alta compatible
        let mut best_version = self.minimum_version;
        
        for peer_version in peer_versions {
            if self.is_version_supported(peer_version) {
                if peer_version.compare(&best_version) == VersionRelation::Newer {
                    best_version = *peer_version;
                }
            }
        }

        let compatible = self.current_version.is_compatible_with(&best_version);

        NegotiationResult {
            agreed_version: best_version,
            capabilities: NodeCapabilities::default(),
            compatible,
        }
    }

    /// Verifica si una versión está soportada
    pub fn is_version_supported(&self, version: &ProtocolVersion) -> bool {
        self.supported_versions.iter().any(|v| v == version)
    }

    /// Agrega soporte para una versión
    pub fn add_supported_version(&mut self, version: ProtocolVersion) {
        if !self.is_version_supported(&version) {
            self.supported_versions.push(version);
            self.supported_versions.sort_by(|a, b| b.compare(a) == VersionRelation::Newer);
        }
    }

    /// Obtiene propuesta activa
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&EvolutionProposal> {
        self.active_proposals.get(&proposal_id)
    }

    /// Obtiene todas las propuestas activas
    pub fn get_active_proposals(&self) -> Vec<&EvolutionProposal> {
        self.active_proposals.values().collect()
    }

    /// Cancela una propuesta
    pub fn cancel_proposal(&mut self, proposal_id: u64) -> Result<(), EvolutionError> {
        let proposal = self.active_proposals
            .get_mut(&proposal_id)
            .ok_or(EvolutionError::ProposalNotFound)?;

        proposal.status = ProposalStatus::Cancelled;

        // Mover a historial
        let record = ProposalRecord {
            proposal_id,
            proposer: proposal.proposer_node,
            new_version: proposal.new_version,
            status: ProposalStatus::Cancelled,
            timestamp: current_timestamp(),
            vote_summary: proposal.vote_counts(),
        };
        self.proposal_history.push(record);

        self.active_proposals.remove(&proposal_id);
        Ok(())
    }

    /// Obtiene historial de evoluciones
    pub fn get_evolution_history(&self) -> &[ProposalRecord] {
        &self.proposal_history
    }
}

impl Default for ProtocolEvolutionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum EvolutionError {
    DowngradeNotAllowed,
    ProposalAlreadyExists,
    ProposalNotFound,
    NotApproved,
    ImplementationFailed(String),
    IncompatibleVersion(ProtocolVersion),
    VoteFailed(String),
}

impl std::fmt::Display for EvolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvolutionError::DowngradeNotAllowed => {
                write!(f, "Downgrade not allowed")
            }
            EvolutionError::ProposalAlreadyExists => {
                write!(f, "Proposal for this version already exists")
            }
            EvolutionError::ProposalNotFound => {
                write!(f, "Proposal not found")
            }
            EvolutionError::NotApproved => {
                write!(f, "Proposal not approved")
            }
            EvolutionError::ImplementationFailed(s) => {
                write!(f, "Implementation failed: {}", s)
            }
            EvolutionError::IncompatibleVersion(v) => {
                write!(f, "Incompatible version: {}", v.to_string())
            }
            EvolutionError::VoteFailed(s) => {
                write!(f, "Vote failed: {}", s)
            }
        }
    }
}

impl std::error::Error for EvolutionError {}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version_creation() {
        let v = ProtocolVersion::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_protocol_version_comparison() {
        let v1 = ProtocolVersion::new(1, 0, 0);
        let v2 = ProtocolVersion::new(1, 1, 0);
        let v3 = ProtocolVersion::new(2, 0, 0);
        
        assert_eq!(v1.compare(&v1), VersionRelation::Equal);
        assert_eq!(v1.compare(&v2), VersionRelation::Older);
        assert_eq!(v2.compare(&v1), VersionRelation::Newer);
        assert_eq!(v1.compare(&v3), VersionRelation::Older);
    }

    #[test]
    fn test_version_compatibility() {
        let v1 = ProtocolVersion::new(1, 2, 0);
        let v2 = ProtocolVersion::new(1, 3, 0);
        let v3 = ProtocolVersion::new(2, 0, 0);
        
        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_evolution_proposal() {
        let mut proposal = EvolutionProposal::new(1, 0x1000, ProtocolVersion::new(1, 1, 0));
        proposal.rationale = "Test proposal".to_string();
        
        // Cast some votes
        proposal.cast_vote(0x2000, Vote::Yes);
        proposal.cast_vote(0x3000, Vote::No);
        proposal.cast_vote(0x4000, Vote::Abstain);
        proposal.cast_vote(0x5000, Vote::Yes);
        
        let (yes, no, abstain) = proposal.vote_counts();
        assert_eq!(yes, 2);
        assert_eq!(no, 1);
        assert_eq!(abstain, 1);
    }

    #[test]
    fn test_consensus_threshold() {
        let mut proposal = EvolutionProposal::new(1, 0x1000, ProtocolVersion::new(1, 1, 0));
        
        // Not enough votes
        assert!(!proposal.has_consensus());
        
        // Add enough votes to reach threshold (66%+)
        for i in 0..6 {
            proposal.cast_vote(0x1000 + i as u64, Vote::Yes);
        }
        
        // With 6 Yes out of 6, should have consensus
        // But we need 3+ votes minimum
        let (yes, _, total) = proposal.vote_counts();
        assert!(total >= 3 && (yes as f32 / total as f32) >= VOTE_THRESHOLD);
    }

    #[test]
    fn test_node_capabilities() {
        let caps = NodeCapabilities {
            code_sync: true,
            consciousness_migration: true,
            evolution_voting: false,
            advanced_crypto: true,
        };
        
        let bits = caps.to_bitfield();
        let restored = NodeCapabilities::from_bitfield(bits);
        
        assert_eq!(caps.code_sync, restored.code_sync);
        assert_eq!(caps.consciousness_migration, restored.consciousness_migration);
        assert_eq!(caps.evolution_voting, restored.evolution_voting);
        assert_eq!(caps.advanced_crypto, restored.advanced_crypto);
    }

    #[test]
    fn test_negotiation() {
        let manager = ProtocolEvolutionManager::new();
        
        let peer_versions = vec![
            ProtocolVersion::new(1, 0, 0),
            ProtocolVersion::new(1, 0, 1),
        ];
        
        let result = manager.negotiate_version(&peer_versions);
        assert!(result.compatible);
    }

    #[test]
    fn test_manager_propose_evolution() {
        let mut manager = ProtocolEvolutionManager::new();
        
        let result = manager.propose_evolution(
            1,
            0x1000,
            ProtocolVersion::new(1, 1, 0),
            "Test evolution",
        );
        
        assert!(result.is_ok());
        let proposal = result.unwrap();
        assert_eq!(proposal.proposer_node, 0x1000);
        assert_eq!(proposal.status, ProposalStatus::Proposed);
    }

    #[test]
    fn test_manager_rejects_downgrade() {
        let mut manager = ProtocolEvolutionManager::new();
        
        let result = manager.propose_evolution(
            1,
            0x1000,
            ProtocolVersion::new(0, 9, 9),
            "Downgrade attempt",
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EvolutionError::DowngradeNotAllowed));
    }

    #[test]
    fn test_version_bytes_roundtrip() {
        let v = ProtocolVersion::new(1, 2, 3);
        let bytes = v.to_bytes();
        let restored = ProtocolVersion::from_bytes(&bytes);
        
        assert_eq!(v.major, restored.major);
        assert_eq!(v.minor, restored.minor);
        assert_eq!(v.patch, restored.patch);
    }
}