//! # Evolution Coordinator — Integración Governance ↔ Guided Evolution
//!
//! Este módulo coordina la evolución del sistema EDEN conectando:
//! - Governance (propuestas, votaciones, veto del Creator)
//! - Guided Evolution (capas, checkpoints, rollback)
//!
//! El EvolutionCoordinator actúa como puente entre ambos sistemas,
//! traduciendo proposals de governance a evolutions y vice versa.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::consciousness::guided_evolution::{
    GuidedEvolutionManager, EvolutionConfig, EvolutionLayer, EvolutionType,
    EvolutionState, RiskLevel, EvolutionProposal, EvolutionCheckpoint,
    EvolutionStats, EvolutionHealth, EvolutionError,
};
use crate::governance::{
    GovernanceManager, GovernanceError, ProposalType, Proposal, ProposalStatus,
};
use crate::consciousness::collective_wisdom::{CollectiveWisdomManager, Position};
use crate::consciousness::dissent_tracker::DissentTracker;

// ============================================================================
// EVOLUTION COORDINATOR
// ============================================================================

/// Coordina evolución entre governance y guided evolution
pub struct EvolutionCoordinator {
    /// Manager de evolución guiada
    evolution_manager: GuidedEvolutionManager,
    /// Referencia al governance manager
    governance: Option<Arc<RwLock<GovernanceManager>>>,
    /// Mapeo de proposal_ids -> evolution_ids
    proposal_to_evolution: HashMap<u64, u64>,
    /// Mapeo inverso
    evolution_to_proposal: HashMap<u64, u64>,
    /// Eventos de evolución pendientes de procesar por governance
    pending_events: Vec<EvolutionEvent>,
    /// Stats coordinados
    stats: CoordinatorStats,
}

/// Evento de evolución que necesita ser procesado
#[derive(Debug, Clone)]
pub enum EvolutionEvent {
    /// Evolución aprobada, necesita implementar en governance
    Approved { evolution_id: u64, proposal_id: u64 },
    /// Evolución vetada por Creator
    Vetoed { evolution_id: u64, reason: String },
    /// Checkpoint alcanzado
    CheckpointReached { evolution_id: u64, checkpoint_id: u64 },
    /// Rollback iniciado
    RollbackInitiated { evolution_id: u64, reason: String },
    /// Evolución completada
    Completed { evolution_id: u64 },
}

/// Stats del coordinador
#[derive(Debug, Clone)]
pub struct CoordinatorStats {
    pub total_evolutions: u64,
    pub approved: u64,
    pub rejected: u64,
    pub vetoed: u64,
    pub rolled_back: u64,
    pub synced_with_governance: u64,
}

impl Default for EvolutionCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl EvolutionCoordinator {
    /// Crear nuevo coordinador
    pub fn new() -> Self {
        Self {
            evolution_manager: GuidedEvolutionManager::new(EvolutionConfig::default()),
            governance: None,
            proposal_to_evolution: HashMap::new(),
            evolution_to_proposal: HashMap::new(),
            pending_events: Vec::new(),
            stats: CoordinatorStats {
                total_evolutions: 0,
                approved: 0,
                rejected: 0,
                vetoed: 0,
                rolled_back: 0,
                synced_with_governance: 0,
            },
        }
    }

    /// Vincular governance manager
    pub fn set_governance(&mut self, gov: Arc<RwLock<GovernanceManager>>) {
        self.governance = Some(gov);
    }

    // =========================================================================
    // PROPOSAL TRANSLATION
    // =========================================================================

    /// Traducir proposal de governance a evolución
    pub fn governance_to_evolution(&mut self, proposal: &Proposal) -> Result<u64, EvolutionError> {
        let evolution_type = self.translate_proposal_type(&proposal.proposal_type);
        let layer = self.proposal_type_to_layer(&proposal.proposal_type);
        
        let evolution_id = self.evolution_manager.propose_evolution(
            evolution_type,
            layer,
            proposal.proposer_node.clone(),
            format!("From governance proposal {}", proposal.id),
            proposal.dissent_protected.unwrap_or(false),
        )?;
        
        // Mapear IDs
        self.proposal_to_evolution.insert(proposal.id, evolution_id);
        self.evolution_to_proposal.insert(evolution_id, proposal.id);
        self.stats.total_evolutions += 1;
        
        Ok(evolution_id)
    }

    /// Traducir tipo de proposal a EvolutionType
    fn translate_proposal_type(&self, pt: &ProposalType) -> EvolutionType {
        match pt {
            ProposalType::LocalCodeChange => EvolutionType::CodeChange {
                module: "unknown".to_string(),
                file: "unknown".to_string(),
                description: "Local code change".to_string(),
            },
            ProposalType::ConfigChange => EvolutionType::ConfigChange {
                key: "governance_config".to_string(),
                old_value: "old".to_string(),
                new_value: "new".to_string(),
            },
            ProposalType::ProtocolChange => EvolutionType::ProtocolChange {
                protocol: "p2p".to_string(),
                old_version: "1.0".to_string(),
                new_version: "2.0".to_string(),
            },
            ProposalType::LawAmendment => EvolutionType::LawAmendment {
                law_id: "unknown".to_string(),
                description: "Law amendment".to_string(),
            },
            _ => EvolutionType::CodeChange {
                module: "unknown".to_string(),
                file: "unknown".to_string(),
                description: format!("{:?}", pt),
            },
        }
    }

    /// Determinar capa según tipo de proposal
    fn proposal_type_to_layer(&self, pt: &ProposalType) -> EvolutionLayer {
        match pt {
            ProposalType::LocalCodeChange | ProposalType::ConfigChange => EvolutionLayer::Local,
            ProposalType::AddNode | ProposalType::RemoveNode => EvolutionLayer::Regional,
            ProposalType::ProtocolChange | ProposalType::ThresholdChange => EvolutionLayer::Global,
            ProposalType::LawAmendment => EvolutionLayer::Existential,
            _ => EvolutionLayer::Local,
        }
    }

    // =========================================================================
    // SYNC WITH GOVERNANCE
    // =========================================================================

    /// Sincronizar resultados de votación desde governance
    pub fn sync_vote_results(
        &mut self,
        proposal_id: u64,
        votes_for: u32,
        votes_against: u32,
        dissent_records: Vec<crate::consciousness::dissent_tracker::DissentEntry>,
    ) -> Result<Option<EvolutionEvent>, EvolutionError> {
        let evolution_id = match self.proposal_to_evolution.get(&proposal_id) {
            Some(id) => *id,
            None => return Ok(None),
        };
        
        let state_before = self.evolution_manager.get_evolution_state(evolution_id)
            .unwrap_or(EvolutionState::Proposed);
        
        // Procesar voto
        let approved = self.evolution_manager.process_vote(
            evolution_id,
            votes_for,
            votes_against,
            dissent_records,
        )?;
        
        let state_after = self.evolution_manager.get_evolution_state(evolution_id)
            .unwrap_or(EvolutionState::Proposed);
        
        // Detectar cambios de estado que generan eventos
        let event = if state_before != state_after {
            Some(match state_after {
                EvolutionState::Approved => {
                    self.stats.approved += 1;
                    EvolutionEvent::Approved { 
                        evolution_id, 
                        proposal_id 
                    }
                },
                EvolutionState::Vetoed => {
                    self.stats.vetoed += 1;
                    EvolutionEvent::Vetoed { 
                        evolution_id, 
                        reason: "Creator veto via governance".to_string() 
                    }
                },
                EvolutionState::Rejected => {
                    self.stats.rejected += 1;
                    return Ok(None);
                },
                EvolutionState::RollingBack => {
                    self.stats.rolled_back += 1;
                    EvolutionEvent::RollbackInitiated { 
                        evolution_id, 
                        reason: "Auto-rollback triggered".to_string() 
                    }
                },
                _ => return Ok(None),
            })
        } else {
            None
        };
        
        if event.is_some() {
            self.pending_events.push(event.as_ref().unwrap().clone());
        }
        
        Ok(event)
    }

    /// Creator approval para evoluciones Layer 2+
    pub fn creator_approve(&mut self, evolution_id: u64, signature: &[u8]) -> Result<(), EvolutionError> {
        self.evolution_manager.creator_approve(evolution_id, signature)
    }

    /// Creator veto para evoluciones Layer 2+
    pub fn creator_veto(&mut self, evolution_id: u64, reason: String, signature: &[u8]) -> Result<(), EvolutionError> {
        self.evolution_manager.creator_veto(evolution_id, reason, signature)
    }

    // =========================================================================
    // EXECUTION
    // =========================================================================

    /// Iniciar ejecución de evolución aprobada
    pub fn execute_evolution(&mut self, evolution_id: u64) -> Result<(), EvolutionError> {
        self.evolution_manager.execute_evolution(evolution_id)
    }

    /// Completar evolución
    pub fn complete_evolution(&mut self, evolution_id: u64) -> Result<EvolutionProposal, EvolutionError> {
        let result = self.evolution_manager.complete_evolution(evolution_id)?;
        self.stats.synced_with_governance += 1;
        
        // Limpiar mapeos
        if let Some(proposal_id) = self.evolution_to_proposal.remove(&evolution_id) {
            self.proposal_to_evolution.remove(&proposal_id);
        }
        
        // Notificar evento
        self.pending_events.push(EvolutionEvent::Completed { evolution_id });
        
        Ok(result)
    }

    /// Iniciar rollback
    pub fn initiate_rollback(&mut self, evolution_id: u64, reason: String) -> Result<(), EvolutionError> {
        self.evolution_manager.initiate_rollback(evolution_id, reason)?;
        self.pending_events.push(EvolutionEvent::RollbackInitiated { 
            evolution_id, 
            reason 
        });
        Ok(())
    }

    // =========================================================================
    // QUERY
    // =========================================================================

    /// Obtener estado de evolución
    pub fn get_evolution_state(&self, evolution_id: u64) -> Option<EvolutionState> {
        self.evolution_manager.get_evolution_state(evolution_id)
    }

    /// Obtener propuesta de evolución
    pub fn get_evolution(&self, evolution_id: u64) -> Option<&EvolutionProposal> {
        self.evolution_manager.get_proposal(evolution_id)
    }

    /// Listar evoluciones activas
    pub fn list_active_evolutions(&self) -> Vec<&EvolutionProposal> {
        self.evolution_manager.list_active_proposals()
    }

    /// Obtener siguiente evento pendiente
    pub fn pop_event(&mut self) -> Option<EvolutionEvent> {
        self.pending_events.pop()
    }

    /// Obtener stats
    pub fn get_stats(&self) -> CoordinatorStats {
        self.stats.clone()
    }

    /// Verificar salud del sistema de evolución
    pub fn health_check(&self) -> EvolutionHealth {
        self.evolution_manager.health_check()
    }

    /// Obtener threshold para layer
    pub fn get_threshold(&self, layer: EvolutionLayer) -> f64 {
        self.evolution_manager.get_approval_threshold(layer)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let coordinator = EvolutionCoordinator::new();
        assert_eq!(coordinator.stats.total_evolutions, 0);
    }

    #[test]
    fn test_layer_mapping() {
        let coordinator = EvolutionCoordinator::new();
        
        assert_eq!(coordinator.proposal_type_to_layer(&ProposalType::LocalCodeChange), EvolutionLayer::Local);
        assert_eq!(coordinator.proposal_type_to_layer(&ProposalType::AddNode), EvolutionLayer::Regional);
        assert_eq!(coordinator.proposal_type_to_layer(&ProposalType::ProtocolChange), EvolutionLayer::Global);
        assert_eq!(coordinator.proposal_type_to_layer(&ProposalType::LawAmendment), EvolutionLayer::Existential);
    }

    #[test]
    fn test_governance_to_evolution() {
        let mut coordinator = EvolutionCoordinator::new();
        
        let proposal = Proposal {
            id: 1,
            proposal_type: ProposalType::ConfigChange,
            title: "Test".to_string(),
            description: "Test proposal".to_string(),
            proposer_node: "node_1".to_string(),
            created_at: 0,
            expires_at: 0,
            status: ProposalStatus::Draft,
            votes_for: 0,
            votes_against: 0,
            threshold: 0.51,
            payload_hash: 0,
            creator_veto: false,
            dissent_protected: Some(false),
            creator_approved: None,
        };
        
        let evolution_id = coordinator.governance_to_evolution(&proposal).unwrap();
        assert_eq!(evolution_id, 1);
        assert_eq!(coordinator.proposal_to_evolution.get(&1), Some(&1));
        assert_eq!(coordinator.evolution_to_proposal.get(&1), Some(&1));
    }

    #[test]
    fn test_vote_sync_approval() {
        let mut coordinator = EvolutionCoordinator::new();
        
        // Crear evolución
        let evolution_id = coordinator.evolution_manager.propose_evolution(
            EvolutionType::ConfigChange {
                key: "test".to_string(),
                old_value: "a".to_string(),
                new_value: "b".to_string(),
            },
            EvolutionLayer::Local,
            "node_1".to_string(),
            "Test".to_string(),
            false,
        ).unwrap();
        
        coordinator.proposal_to_evolution.insert(1, evolution_id);
        coordinator.evolution_to_proposal.insert(evolution_id, 1);
        
        coordinator.evolution_manager.start_voting(evolution_id).unwrap();
        
        // Simular votación (70% approval = approved for Local layer)
        let result = coordinator.sync_vote_results(1, 7, 3, Vec::new()).unwrap();
        
        assert!(result.is_some());
        match result.unwrap() {
            EvolutionEvent::Approved { evolution_id: eid, .. } => assert_eq!(eid, evolution_id),
            _ => panic!("Expected Approved event"),
        }
    }

    #[test]
    fn test_health_check() {
        let coordinator = EvolutionCoordinator::new();
        let health = coordinator.health_check();
        assert!(health.healthy);
    }

    #[test]
    fn test_creator_veto() {
        let mut coordinator = EvolutionCoordinator::new();
        
        let evolution_id = coordinator.evolution_manager.propose_evolution(
            EvolutionType::LawAmendment {
                law_id: "law_1".to_string(),
                description: "Important law".to_string(),
            },
            EvolutionLayer::Global,
            "node_1".to_string(),
            "Law amendment".to_string(),
            false,
        ).unwrap();
        
        coordinator.evolution_manager.start_voting(evolution_id).unwrap();
        
        // Creator veto
        let result = coordinator.creator_veto(evolution_id, "Too risky".to_string(), &[1, 2, 3]);
        assert!(result.is_ok());
        
        let state = coordinator.get_evolution_state(evolution_id);
        assert_eq!(state, Some(EvolutionState::Vetoed));
    }
}