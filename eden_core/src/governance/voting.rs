//! # Voting — Sistema de Votación Ponderada y Consentida
//!
//! Implementa el sistema de votación para governance de EDEN:
//! - Votos ponderados por contribución
//! - Múltiples tipos de votación (mayoría, supermayoría, unánimidad)
//! - Protección de disidentes
//! - Firmas para verificación de identidad
#![allow(dead_code)]
#![allow(non_snake_case)]

use super::{current_timestamp, Proposal, Vote, GovernanceError};
use std::collections::{HashMap, HashSet};

// ============================================================================
// VOTE TYPES
// ============================================================================

/// Tipo de votación
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteType {
    /// Mayoría simple (51%)
    SimpleMajority,
    /// Super mayoría (66%)
    SuperMajority,
    /// Mayoría cualificada (75%)
    QualifiedMajority,
    /// Unanimidad (100%)
    Unanimity,
}

impl VoteType {
    pub fn threshold(&self) -> f64 {
        match self {
            VoteType::SimpleMajority => 0.51,
            VoteType::SuperMajority => 0.66,
            VoteType::QualifiedMajority => 0.75,
            VoteType::Unanimity => 1.00,
        }
    }
}

/// Resultado de una votación
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteOutcome {
    Approved,
    Rejected,
    Tie,
    QuorumNotReached,
    InProgress,
}

// ============================================================================
// WEIGHTED VOTING
// ============================================================================

/// Gestor de votación ponderada
pub struct WeightedVoting {
    /// Configuración de pesos
    config: VotingConfig,
    /// Historial de votaciones
    history: Vec<VotingRecord>,
}

#[derive(Debug, Clone)]
pub struct VotingConfig {
    /// Peso base para todos los nodos
    pub base_weight: f64,
    /// Peso máximo por nodo
    pub max_weight: f64,
    /// Habilitar peso por reputación
    pub enable_reputation_weight: bool,
    /// Habilitar peso por stake (futuro)
    pub enable_stake_weight: bool,
}

impl Default for VotingConfig {
    fn default() -> Self {
        Self {
            base_weight: 1.0,
            max_weight: 5.0,
            enable_reputation_weight: true,
            enable_stake_weight: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VotingRecord {
    pub proposal_id: u64,
    pub outcome: VoteOutcome,
    pub total_weight_for: f64,
    pub total_weight_against: f64,
    pub total_weight_abstain: f64,
    pub timestamp: u64,
}

impl WeightedVoting {
    pub fn new() -> Self {
        Self {
            config: VotingConfig::default(),
            history: Vec::new(),
        }
    }

    /// Calcular peso de un voto basado en reputación y stake
    pub fn calculate_weight(
        &self,
        node_id: &str,
        reputation: f64,
        stake: f64,
    ) -> f64 {
        let mut weight = self.config.base_weight;

        if self.config.enable_reputation_weight {
            // Reputation mod: 0.5 a 1.5
            let rep_mod = 0.5 + reputation;
            weight *= rep_mod;
        }

        if self.config.enable_stake_weight {
            // Stake mod: log(scale)
            let stake_mod = (stake / 1000.0).ln().max(1.0);
            weight *= stake_mod;
        }

        weight.min(self.config.max_weight)
    }

    /// Emitir voto ponderado
    pub fn cast_vote(
        &self,
        proposal: &mut Proposal,
        node_id: &str,
        approve: bool,
        abstain: bool,
        weight: f64,
    ) -> Result<(), GovernanceError> {
        // Verificar que no haya votado ya
        if proposal.votes_for.iter().any(|v| v.node_id == node_id)
            || proposal.votes_against.iter().any(|v| v.node_id == node_id)
            || proposal.abstentions.iter().any(|v| v.node_id == node_id)
        {
            return Err(GovernanceError::AlreadyVoted(node_id.to_string()));
        }

        let vote = Vote {
            node_id: node_id.to_string(),
            weight,
            approve,
            abstain,
            timestamp: current_timestamp(),
            signature: None,
        };

        if abstain {
            proposal.abstentions.push(vote);
        } else if approve {
            proposal.votes_for.push(vote);
        } else {
            proposal.votes_against.push(vote);
        }

        Ok(())
    }

    /// Contar votos ponderados
    pub fn tally_votes(&self, proposal: &Proposal, vote_type: VoteType) -> VoteOutcome {
        let total_weight: f64 = proposal.votes_for.iter().map(|v| v.weight).sum()
            + proposal.votes_against.iter().map(|v| v.weight).sum();

        if total_weight == 0.0 {
            return VoteOutcome::QuorumNotReached;
        }

        let for_weight: f64 = proposal.votes_for.iter().map(|v| v.weight).sum();
        let against_weight: f64 = proposal.votes_against.iter().map(|v| v.weight).sum();

        let for_ratio = for_weight / total_weight;
        let against_ratio = against_weight / total_weight;

        let threshold = vote_type.threshold();

        if for_ratio >= threshold {
            VoteOutcome::Approved
        } else if against_ratio > (1.0 - threshold) {
            VoteOutcome::Rejected
        } else {
            VoteOutcome::Tie
        }
    }

    /// Verificar quorum
    pub fn check_quorum(
        &self,
        proposal: &Proposal,
        required_participants: usize,
    ) -> bool {
        let participants = proposal.voting_nodes();
        participants >= required_participants
    }

    /// Guardar resultado de votación
    pub fn record_outcome(&mut self, record: VotingRecord) {
        self.history.push(record);
        
        // Limitar tamaño del historial
        if self.history.len() > 10000 {
            self.history.remove(0);
        }
    }

    /// Obtener historial
    pub fn get_history(&self) -> &[VotingRecord] {
        &self.history
    }
}

impl Default for WeightedVoting {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// VOTE VERIFICATION
// ============================================================================

/// Verificador de votos
pub struct VoteVerifier {
    /// Nodos autorizados para votar
    authorized_nodes: HashSet<String>,
}

impl VoteVerifier {
    pub fn new() -> Self {
        Self {
            authorized_nodes: HashSet::new(),
        }
    }

    /// Autorizar un nodo para votar
    pub fn authorize(&mut self, node_id: &str) {
        self.authorized_nodes.insert(node_id.to_string());
    }

    /// Desautorizar un nodo
    pub fn deauthorize(&mut self, node_id: &str) {
        self.authorized_nodes.remove(node_id);
    }

    /// Verificar si un nodo está autorizado
    pub fn is_authorized(&self, node_id: &str) -> bool {
        self.authorized_nodes.contains(node_id)
    }

    /// Verificar firma de voto
    pub fn verify_signature(
        &self,
        node_id: &str,
        _payload: &[u8],
        signature: &[u8; 64],
    ) -> bool {
        // Simplified - in production would verify Ed25519 signature
        if !self.is_authorized(node_id) {
            return false;
        }
        
        // Check signature is non-zero (simplified check)
        signature.iter().any(|&b| b != 0)
    }
}

impl Default for VoteVerifier {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DISSENT PROTECTION
// ============================================================================

/// Sistema de protección de disidentes
pub struct DissentProtection {
    /// Nodos que votearon en contra del consensus
    dissent_records: HashMap<String, Vec<DissentEntry>>,
    /// Habilitada
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct DissentEntry {
    pub proposal_id: u64,
    pub voted_against: bool,
    pub final_outcome: bool, // true si el majority estaba en lo correcto
    pub timestamp: u64,
}

impl DissentProtection {
    pub fn new() -> Self {
        Self {
            dissent_records: HashMap::new(),
            enabled: true,
        }
    }

    /// Habilitar protección
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Deshabilitar protección
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Verificar si un nodo es disidente protegido
    pub fn is_dissent_protected(&self, node_id: &str) -> bool {
        if !self.enabled {
            return false;
        }

        // Si el nodo tiene historial de ser minority pero voters correctos,
        // recibe protección
        if let Some(entries) = self.dissent_records.get(node_id) {
            let correct_disses = entries.iter().filter(|e| e.final_outcome).count();
            let total = entries.len();
            
            // Más del 50% de sus disensos fueron correctos
            return correct_disses * 2 >= total;
        }

        false
    }

    /// Registrar disenso
    pub fn register(
        &mut self,
        node_id: &str,
        proposal_id: u64,
        voted_against: bool,
        was_correct: bool,
    ) {
        let entry = DissentEntry {
            proposal_id,
            voted_against,
            final_outcome: was_correct,
            timestamp: current_timestamp(),
        };

        self.dissent_records
            .entry(node_id.to_string())
            .or_insert_with(Vec::new)
            .push(entry);

        // Limitar historial por nodo
        if let Some(entries) = self.dissent_records.get_mut(node_id) {
            while entries.len() > 100 {
                entries.remove(0);
            }
        }
    }

    /// Obtener score de protección (0.0 - 1.0)
    pub fn protection_score(&self, node_id: &str) -> f64 {
        if !self.enabled {
            return 0.0;
        }

        if let Some(entries) = self.dissent_records.get(node_id) {
            if entries.is_empty() {
                return 0.0;
            }

            let correct = entries.iter().filter(|e| e.final_outcome).count();
            correct as f64 / entries.len() as f64
        } else {
            0.0
        }
    }
}

impl Default for DissentProtection {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_types() {
        assert_eq!(VoteType::SimpleMajority.threshold(), 0.51);
        assert_eq!(VoteType::SuperMajority.threshold(), 0.66);
        assert_eq!(VoteType::Unanimity.threshold(), 1.00);
    }

    #[test]
    fn test_weighted_voting() {
        let mut voting = WeightedVoting::new();
        
        // Create a mock proposal
        let mut proposal = Proposal::new(
            1,
            super::super::ProposalType::AddNode,
            "creator".to_string(),
            "Test".to_string(),
            0xDEAD,
            1024,
        );
        
        // Cast votes with different weights
        voting.cast_vote(&mut proposal, "node1", true, false, 1.0).unwrap();
        voting.cast_vote(&mut proposal, "node2", true, false, 1.5).unwrap();
        voting.cast_vote(&mut proposal, "node3", false, false, 1.0).unwrap();
        
        assert_eq!(proposal.votes_for.len(), 2);
        assert_eq!(proposal.votes_against.len(), 1);
    }

    #[test]
    fn test_tally() {
        let voting = WeightedVoting::new();
        
        let mut proposal = Proposal::new(
            1,
            super::super::ProposalType::AddNode,
            "creator".to_string(),
            "Test".to_string(),
            0xDEAD,
            1024,
        );
        
        // 3 votes for, 1 against - should pass simple majority
        proposal.votes_for.push(Vote {
            node_id: "n1".to_string(),
            weight: 1.0,
            approve: true,
            abstain: false,
            timestamp: 0,
            signature: None,
        });
        proposal.votes_for.push(Vote {
            node_id: "n2".to_string(),
            weight: 1.0,
            approve: true,
            abstain: false,
            timestamp: 0,
            signature: None,
        });
        proposal.votes_for.push(Vote {
            node_id: "n3".to_string(),
            weight: 1.0,
            approve: true,
            abstain: false,
            timestamp: 0,
            signature: None,
        });
        proposal.votes_against.push(Vote {
            node_id: "n4".to_string(),
            weight: 1.0,
            approve: false,
            abstain: false,
            timestamp: 0,
            signature: None,
        });
        
        let outcome = voting.tally_votes(&proposal, VoteType::SimpleMajority);
        assert_eq!(outcome, VoteOutcome::Approved);
    }

    #[test]
    fn test_dissent_protection() {
        let mut protection = DissentProtection::new();
        
        // Register some correct dissents
        protection.register("dissenter", 1, true, true);
        protection.register("dissenter", 2, true, true);
        protection.register("dissenter", 3, true, false);
        
        // 2/3 correct = protected
        assert!(protection.is_dissent_protected("dissenter"));
        assert_eq!(protection.protection_score("dissenter"), 2.0 / 3.0);
        
        // New node with no history
        assert!(!protection.is_dissent_protected("new_node"));
    }
}