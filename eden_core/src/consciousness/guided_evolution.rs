//! # Guided Evolution — Evolución Estratificada con Guardrails
//!
//! Este módulo implementa la evolución del sistema EDEN siguiendo
//! capas jerárquicas de decisión, donde cada capa tiene requisitos
//! crecientes de consenso y aprobación del Creator.
//!
//! ## Arquitectura de Capas
//!
//! | Capa | Tipo | Threshold | Creator | Uso |
//! |-------|------|-----------|---------|-----|
//! | 0 | Local | 51% | No | Cambios locales de código |
//! | 1 | Regional | 66% | No | Coordinación entre regiones |
//! | 2 | Global | 66% | Veto | Leyes globales, cambios de protocolo |
//! | 3 | Existential | 100% | Approve | Cambios fundamentales del sistema |
//!
//! ## Checkpoints y Rollback
//!
//! Cada evolución atraviesa checkpoints obligatorios que permiten
//! rollback si se detectan anomalías o comportamientos no deseados.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::consciousness::collective_wisdom::{
    CollectiveWisdomManager, CollectiveDecision, NodePerspective, Position, WisdomConfig,
};
use crate::consciousness::dissent_tracker::DissentEntry;

// Note: consensus and governance modules are planned for future phases
// use crate::consensus::traits::{ConsensusMessage, ConsensusResult};
// use crate::governance::decision_chain::{DecisionChain, BlockType};
// use crate::governance::quorum_manager::QuorumManager;

// ============================================================================
// EVOLUTION LAYER DEFINITIONS
// ============================================================================

/// Capa de evolución (0-3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvolutionLayer {
    /// Cambios locales de código, configuración menor
    Local = 0,
    /// Coordinación entre regiones, cambios de mediana escala
    Regional = 1,
    /// Leyes globales, cambios de protocolo
    Global = 2,
    /// Cambios fundamentales del sistema (existenciales)
    Existential = 3,
}

impl EvolutionLayer {
    pub fn threshold(&self) -> f64 {
        match self {
            EvolutionLayer::Local => 0.51,
            EvolutionLayer::Regional => 0.66,
            EvolutionLayer::Global => 0.66,
            EvolutionLayer::Existential => 1.00,
        }
    }

    pub fn requires_creator_approval(&self) -> bool {
        matches!(self, EvolutionLayer::Global | EvolutionLayer::Existential)
    }

    pub fn requires_creator_veto(&self) -> bool {
        matches!(self, EvolutionLayer::Global | EvolutionLayer::Existential)
    }

    pub fn description(&self) -> &'static str {
        match self {
            EvolutionLayer::Local => "Local code changes",
            EvolutionLayer::Regional => "Regional coordination",
            EvolutionLayer::Global => "Global law changes",
            EvolutionLayer::Existential => "Existential system changes",
        }
    }
}

/// Tipo de cambio evolutivo
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvolutionType {
    /// Cambio en código fuente
    CodeChange {
        module: String,
        file: String,
        description: String,
    },
    /// Cambio en configuración
    ConfigChange {
        key: String,
        old_value: String,
        new_value: String,
    },
    /// Nueva feature
    FeatureAddition {
        name: String,
        description: String,
        risk_level: RiskLevel,
    },
    /// Eliminación de feature
    FeatureRemoval {
        name: String,
        reason: String,
    },
    /// Cambio de protocolo
    ProtocolChange {
        protocol: String,
        old_version: String,
        new_version: String,
    },
    /// Cambio en ley/gobernanza
    LawAmendment {
        law_id: String,
        description: String,
    },
    /// Cambio estructural (solo Layer 3)
    StructuralChange {
        component: String,
        description: String,
    },
}

/// Nivel de riesgo del cambio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn multiplier(&self) -> f64 {
        match self {
            RiskLevel::Low => 1.0,
            RiskLevel::Medium => 1.5,
            RiskLevel::High => 2.0,
            RiskLevel::Critical => 3.0,
        }
    }
}

/// Estado de una evolución
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvolutionState {
    /// En evaluación inicial
    Proposed,
    /// En período de投票
    Voting,
    /// Aprobado pero esperando checkpoint
    Approved,
    /// En ejecución
    Executing,
    /// En rollback
    RollingBack,
    /// Completado exitosamente
    Completed,
    /// Rechazado
    Rejected,
    /// Bloqueado por Creator veto
    Vetoed,
    /// Expirado sin decisión
    Expired,
}

/// Checkpoint de evolución
#[derive(Debug, Clone)]
pub struct EvolutionCheckpoint {
    pub id: u64,
    pub evolution_id: u64,
    pub state: EvolutionState,
    pub description: String,
    pub votes_for: u32,
    pub votes_against: u32,
    pub creator_approved: bool,
    pub creator_veto: bool,
    pub dissent_protected: bool,
    pub timestamp: u64,
    pub required_approval_threshold: f64,
}

/// Evolución propuesta
#[derive(Debug, Clone)]
pub struct EvolutionProposal {
    pub id: u64,
    pub evolution_type: EvolutionType,
    pub layer: EvolutionLayer,
    pub proposer_node: String,
    pub description: String,
    pub risk_level: RiskLevel,
    pub state: EvolutionState,
    pub created_at: u64,
    pub expires_at: u64,
    pub checkpoints: Vec<EvolutionCheckpoint>,
    pub current_checkpoint: usize,
    pub dissent_protected: bool,
    pub rollback_of: Option<u64>,
}

/// Resultado de validar una evolución
#[derive(Debug, Clone)]
pub struct EvolutionValidation {
    pub valid: bool,
    pub layer_appropriate: bool,
    pub risk_assessment: RiskLevel,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Estadísticas de evolución
#[derive(Debug, Clone)]
pub struct EvolutionStats {
    pub total_proposals: u64,
    pub layer_0_count: u64,
    pub layer_1_count: u64,
    pub layer_2_count: u64,
    pub layer_3_count: u64,
    pub approved_count: u64,
    pub rejected_count: u64,
    pub vetoed_count: u64,
    pub rolled_back_count: u64,
    pub avg_approval_time: f64,
    pub active_count: u64,
}

// ============================================================================
// GUIDED EVOLUTION MANAGER
// ============================================================================

/// Manager de evolución guiada
pub struct GuidedEvolutionManager {
    /// Configuración
    config: EvolutionConfig,
    /// Propuestas activas
    proposals: HashMap<u64, EvolutionProposal>,
    /// Historial de evoluciones completadas
    completed_evolutions: Vec<EvolutionProposal>,
    /// Checkpoints por propuesta
    checkpoints: HashMap<u64, Vec<EvolutionCheckpoint>>,
    /// Contador de propuestas
    proposal_counter: u64,
    /// Stats
    stats: EvolutionStats,
    /// Decisión chain (para logging futuro)
    #[allow(dead_code)]
    decision_chain: Option<Arc<RwLock<()>>>,
    /// Quorum manager (para logging futuro)
    #[allow(dead_code)]
    quorum_manager: Option<Arc<RwLock<()>>>,
    /// Nodos que han realizado cambios recientemente (rate limiting)
    recent_change_nodes: HashMap<String, u64>,
    /// Threshold de rate limiting (cambios por hora)
    rate_limit_per_hour: u32,
}

/// Configuración de evolución
#[derive(Debug, Clone)]
pub struct EvolutionConfig {
    /// Votación mínima requerida
    pub min_voting_period_secs: u64,
    /// Checkpoints obligatorios
    pub checkpoint_interval_secs: u64,
    /// Tiempo máximo de evolución
    pub max_evolution_time_secs: u64,
    /// Rollback automático habilitado
    pub auto_rollback_enabled: bool,
    /// Rollback threshold (si dissent > X, rollback)
    pub rollback_dissent_threshold: f64,
    /// Creator veto siempre activo
    pub creator_veto_enabled: bool,
    /// Rate limiting de cambios
    pub rate_limit_per_hour: u32,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            min_voting_period_secs: 300,      // 5 minutos mínimo
            checkpoint_interval_secs: 600,    // Checkpoint cada 10 min
            max_evolution_time_secs: 3600,    // 1 hora máximo
            auto_rollback_enabled: true,
            rollback_dissent_threshold: 0.25, // 25% de disenso activa rollback
            creator_veto_enabled: true,
            rate_limit_per_hour: 10,
        }
    }
}

/// Error de evolución
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvolutionError {
    InvalidLayerTransition { from: EvolutionLayer, to: EvolutionLayer },
    InsufficientVotes { required: u32, actual: u32 },
    CreatorVetoActivated,
    CreatorApprovalRequired,
    CheckpointNotReached { required: u64, current: u64 },
    RateLimitExceeded { node: String, limit: u32 },
    EvolutionExpired,
    RollbackNotAllowed,
    InvalidEvolutionState { current: EvolutionState, required: EvolutionState },
}

impl std::fmt::Display for EvolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLayerTransition { from, to } => {
                write!(f, "Invalid layer transition from {:?} to {:?}", from, to)
            }
            Self::InsufficientVotes { required, actual } => {
                write!(f, "Insufficient votes: required {}, got {}", required, actual)
            }
            Self::CreatorVetoActivated => write!(f, "Creator veto activated"),
            Self::CreatorApprovalRequired => write!(f, "Creator approval required"),
            Self::CheckpointNotReached { required, current } => {
                write!(f, "Checkpoint {} not reached (currently at {})", required, current)
            }
            Self::RateLimitExceeded { node, limit } => {
                write!(f, "Rate limit exceeded for node {}: limit {}", node, limit)
            }
            Self::EvolutionExpired => write!(f, "Evolution has expired"),
            Self::RollbackNotAllowed => write!(f, "Rollback not allowed in current state"),
            Self::InvalidEvolutionState { current, required } => {
                write!(f, "Invalid evolution state: current {:?}, required {:?}", current, required)
            }
        }
    }
}

impl std::error::Error for EvolutionError {}

impl Default for GuidedEvolutionManager {
    fn default() -> Self {
        Self::new(EvolutionConfig::default())
    }
}

impl GuidedEvolutionManager {
    /// Crear nuevo manager
    pub fn new(config: EvolutionConfig) -> Self {
        Self {
            config,
            proposals: HashMap::new(),
            completed_evolutions: Vec::new(),
            checkpoints: HashMap::new(),
            proposal_counter: 0,
            stats: EvolutionStats {
                total_proposals: 0,
                layer_0_count: 0,
                layer_1_count: 0,
                layer_2_count: 0,
                layer_3_count: 0,
                approved_count: 0,
                rejected_count: 0,
                vetoed_count: 0,
                rolled_back_count: 0,
                avg_approval_time: 0.0,
                active_count: 0,
            },
            decision_chain: None,
            quorum_manager: None,
            recent_change_nodes: HashMap::new(),
            rate_limit_per_hour: 10,
        }
    }

    /// Vincular DecisionChain (futuro)
    #[allow(dead_code)]
    pub fn set_decision_chain(&mut self, _chain: Arc<RwLock<()>>) {}

    /// Vincular QuorumManager (futuro)
    #[allow(dead_code)]
    pub fn set_quorum_manager(&mut self, _quorum: Arc<RwLock<()>>) {}

    // =========================================================================
    // PROPOSAL MANAGEMENT
    // =========================================================================

    /// Proponer nueva evolución
    pub fn propose_evolution(
        &mut self,
        evolution_type: EvolutionType,
        layer: EvolutionLayer,
        proposer_node: String,
        description: String,
        dissent_protected: bool,
    ) -> Result<u64, EvolutionError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Rate limiting check
        if let Some(last_change) = self.recent_change_nodes.get(&proposer_node) {
            let elapsed = now - last_change;
            if elapsed < 3600 && self.rate_limit_per_hour > 0 {
                // We allow it for now (simplified check)
            }
        }

        // Validar capa apropiada para tipo
        let validation = self.validate_evolution_type(&evolution_type, layer);
        if !validation.valid {
            return Err(EvolutionError::InvalidLayerTransition {
                from: EvolutionLayer::Local,
                to: layer,
            });
        }

        self.proposal_counter += 1;
        let proposal_id = self.proposal_counter;

        let proposal = EvolutionProposal {
            id: proposal_id,
            evolution_type: evolution_type.clone(),
            layer,
            proposer_node: proposer_node.clone(),
            description,
            risk_level: validation.risk_assessment,
            state: EvolutionState::Proposed,
            created_at: now,
            expires_at: now + self.config.max_evolution_time_secs,
            checkpoints: Vec::new(),
            current_checkpoint: 0,
            dissent_protected,
            rollback_of: None,
        };

        self.proposals.insert(proposal_id, proposal);

        // Update stats
        self.stats.total_proposals += 1;
        match layer {
            EvolutionLayer::Local => self.stats.layer_0_count += 1,
            EvolutionLayer::Regional => self.stats.layer_1_count += 1,
            EvolutionLayer::Global => self.stats.layer_2_count += 1,
            EvolutionLayer::Existential => self.stats.layer_3_count += 1,
        }
        self.stats.active_count += 1;

        // Record rate limiting
        self.recent_change_nodes.insert(proposer_node, now);

        Ok(proposal_id)
    }

    /// Validar que el tipo de evolución sea apropiado para la capa
    fn validate_evolution_type(
        &self,
        evolution_type: &EvolutionType,
        layer: EvolutionLayer,
    ) -> EvolutionValidation {
        let mut valid = true;
        let mut layer_appropriate = true;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Determine expected layer for type
        let expected_min_layer = match evolution_type {
            EvolutionType::CodeChange { .. } => EvolutionLayer::Local,
            EvolutionType::ConfigChange { .. } => EvolutionLayer::Local,
            EvolutionType::FeatureAddition { risk_level, .. } => {
                match risk_level {
                    RiskLevel::Low | RiskLevel::Medium => EvolutionLayer::Regional,
                    RiskLevel::High => EvolutionLayer::Global,
                    RiskLevel::Critical => EvolutionLayer::Existential,
                }
            }
            EvolutionType::FeatureRemoval { .. } => EvolutionLayer::Regional,
            EvolutionType::ProtocolChange { .. } => EvolutionLayer::Global,
            EvolutionType::LawAmendment { .. } => EvolutionLayer::Global,
            EvolutionType::StructuralChange { .. } => EvolutionLayer::Existential,
        };

        if layer < expected_min_layer {
            layer_appropriate = false;
            valid = false;
            errors.push(format!(
                "Evolution type requires at least layer {:?}, got {:?}",
                expected_min_layer, layer
            ));
        }

        // Risk-based warnings
        if let EvolutionType::FeatureAddition { risk_level: RiskLevel::Critical, .. } = evolution_type {
            warnings.push("Critical risk feature requires special monitoring".to_string());
        }

        // Determine actual risk level
        let risk_assessment = match evolution_type {
            EvolutionType::CodeChange { .. } => RiskLevel::Low,
            EvolutionType::ConfigChange { .. } => RiskLevel::Low,
            EvolutionType::FeatureAddition { risk_level, .. } => *risk_level,
            EvolutionType::FeatureRemoval { .. } => RiskLevel::Medium,
            EvolutionType::ProtocolChange { .. } => RiskLevel::High,
            EvolutionType::LawAmendment { .. } => RiskLevel::High,
            EvolutionType::StructuralChange { .. } => RiskLevel::Critical,
        };

        EvolutionValidation {
            valid,
            layer_appropriate,
            risk_assessment,
            warnings,
            errors,
        }
    }

    /// Iniciar período de votación
    pub fn start_voting(&mut self, proposal_id: u64) -> Result<(), EvolutionError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| EvolutionError::InvalidEvolutionState {
                current: EvolutionState::Proposed,
                required: EvolutionState::Proposed,
            })?;

        if proposal.state != EvolutionState::Proposed {
            return Err(EvolutionError::InvalidEvolutionState {
                current: proposal.state.clone(),
                required: EvolutionState::Proposed,
            });
        }

        proposal.state = EvolutionState::Voting;
        Ok(())
    }

    /// Obtener propuesta
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&EvolutionProposal> {
        self.proposals.get(&proposal_id)
    }

    /// Listar propuestas activas
    pub fn list_active_proposals(&self) -> Vec<&EvolutionProposal> {
        self.proposals.values()
            .filter(|p| p.state != EvolutionState::Completed && p.state != EvolutionState::Rejected)
            .collect()
    }

    // =========================================================================
    // VOTING AND APPROVAL
    // =========================================================================

    /// Procesar voto (interno, llamado desde governance)
    pub fn process_vote(
        &mut self,
        proposal_id: u64,
        votes_for: u32,
        votes_against: u32,
        dissent_records: Vec<DissentEntry>,
    ) -> Result<bool, EvolutionError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| EvolutionError::InsufficientVotes { required: 0, actual: 0 })?;

        if proposal.state != EvolutionState::Voting {
            return Err(EvolutionError::InvalidEvolutionState {
                current: proposal.state.clone(),
                required: EvolutionState::Voting,
            });
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check expiration
        if now > proposal.expires_at {
            proposal.state = EvolutionState::Expired;
            return Err(EvolutionError::EvolutionExpired);
        }

        let total_votes = votes_for + votes_against;
        let approval_ratio = if total_votes > 0 {
            votes_for as f64 / total_votes as f64
        } else {
            0.0
        };

        let threshold = proposal.layer.threshold();

        // Calculate dissent ratio for rollback check
        let dissent_ratio = if total_votes > 0 {
            dissent_records.len() as f64 / total_votes as f64
        } else {
            0.0
        };

        // Create checkpoint
        let checkpoint = EvolutionCheckpoint {
            id: proposal.checkpoints.len() as u64 + 1,
            evolution_id: proposal_id,
            state: proposal.state.clone(),
            description: format!(
                "Vote: {}/{}, ratio: {:.2}, threshold: {:.2}",
                votes_for, votes_against, approval_ratio, threshold
            ),
            votes_for,
            votes_against,
            creator_approved: false,
            creator_veto: false,
            dissent_protected: dissent_records.len() > 0,
            timestamp: now,
            required_approval_threshold: threshold,
        };

        proposal.checkpoints.push(checkpoint);

        // Check if approved
        if approval_ratio >= threshold {
            proposal.state = EvolutionState::Approved;
            proposal.current_checkpoint = proposal.checkpoints.len();
            self.stats.approved_count += 1;
            return Ok(true);
        }

        // Check if rejected (clear majority against)
        if approval_ratio < (1.0 - threshold) && total_votes > 10 {
            proposal.state = EvolutionState::Rejected;
            self.stats.rejected_count += 1;
            return Ok(false);
        }

        // Check for auto-rollback trigger
        if self.config.auto_rollback_enabled && dissent_ratio > self.config.rollback_dissent_threshold {
            proposal.state = EvolutionState::RollingBack;
            self.stats.rolled_back_count += 1;
            return Err(EvolutionError::RollbackNotAllowed);
        }

        Ok(false)
    }

    /// Aprobar con signature del Creator (para Layer 2+)
    pub fn creator_approve(&mut self, proposal_id: u64, creator_signature: &[u8]) -> Result<(), EvolutionError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| EvolutionError::CreatorApprovalRequired)?;

        if !proposal.layer.requires_creator_approval() {
            return Ok(()); // No need for creator approval at lower layers
        }

        // Verify signature (simplified - in real impl would use actual crypto)
        if creator_signature.is_empty() {
            return Err(EvolutionError::CreatorApprovalRequired);
        }

        // Create checkpoint with creator approval
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let checkpoint = EvolutionCheckpoint {
            id: proposal.checkpoints.len() as u64 + 1,
            evolution_id: proposal_id,
            state: proposal.state.clone(),
            description: "Creator approval received".to_string(),
            votes_for: 0,
            votes_against: 0,
            creator_approved: true,
            creator_veto: false,
            dissent_protected: false,
            timestamp: now,
            required_approval_threshold: proposal.layer.threshold(),
        };

        proposal.checkpoints.push(checkpoint);
        proposal.current_checkpoint = proposal.checkpoints.len();

        Ok(())
    }

    /// Veto del Creator (para Layer 2+)
    pub fn creator_veto(&mut self, proposal_id: u64, reason: String, creator_signature: &[u8]) -> Result<(), EvolutionError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| EvolutionError::CreatorVetoActivated)?;

        if !proposal.layer.requires_creator_veto() {
            return Ok(()); // No veto at lower layers
        }

        // Verify signature
        if creator_signature.is_empty() {
            return Err(EvolutionError::CreatorVetoActivated);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let checkpoint = EvolutionCheckpoint {
            id: proposal.checkpoints.len() as u64 + 1,
            evolution_id: proposal_id,
            state: EvolutionState::Vetoed,
            description: format!("Creator veto: {}", reason),
            votes_for: 0,
            votes_against: 0,
            creator_approved: false,
            creator_veto: true,
            dissent_protected: false,
            timestamp: now,
            required_approval_threshold: proposal.layer.threshold(),
        };

        proposal.checkpoints.push(checkpoint);
        proposal.state = EvolutionState::Vetoed;
        self.stats.vetoed_count += 1;

        Ok(())
    }

    // =========================================================================
    // EXECUTION AND ROLLBACK
    // =========================================================================

    /// Ejecutar evolución aprobada
    pub fn execute_evolution(&mut self, proposal_id: u64) -> Result<(), EvolutionError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| EvolutionError::InvalidEvolutionState {
                current: EvolutionState::Proposed,
                required: EvolutionState::Approved,
            })?;

        if proposal.state != EvolutionState::Approved {
            return Err(EvolutionError::InvalidEvolutionState {
                current: proposal.state.clone(),
                required: EvolutionState::Approved,
            });
        }

        proposal.state = EvolutionState::Executing;

        // Note: logging to decision chain would go here
        // if let Some(ref chain) = self.decision_chain { ... }

        Ok(())
    }

    /// Completar evolución
    pub fn complete_evolution(&mut self, proposal_id: u64) -> Result<EvolutionProposal, EvolutionError> {
        let proposal = self.proposals.remove(&proposal_id)
            .ok_or_else(|| EvolutionError::InvalidEvolutionState {
                current: EvolutionState::Proposed,
                required: EvolutionState::Executing,
            })?;

        if proposal.state != EvolutionState::Executing {
            return Err(EvolutionError::InvalidEvolutionState {
                current: proposal.state.clone(),
                required: EvolutionState::Executing,
            });
        }

        let mut completed = proposal;
        completed.state = EvolutionState::Completed;

        self.stats.active_count -= 1;
        self.completed_evolutions.push(completed.clone());

        // Note: logging to decision chain would go here
        // if let Some(ref chain) = self.decision_chain { ... }

        Ok(completed)
    }

    /// Iniciar rollback
    pub fn initiate_rollback(&mut self, proposal_id: u64, reason: String) -> Result<(), EvolutionError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| EvolutionError::RollbackNotAllowed)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let checkpoint = EvolutionCheckpoint {
            id: proposal.checkpoints.len() as u64 + 1,
            evolution_id: proposal_id,
            state: EvolutionState::RollingBack,
            description: format!("Rollback initiated: {}", reason),
            votes_for: 0,
            votes_against: 0,
            creator_approved: false,
            creator_veto: false,
            dissent_protected: false,
            timestamp: now,
            required_approval_threshold: proposal.layer.threshold(),
        };

        proposal.checkpoints.push(checkpoint);
        proposal.state = EvolutionState::RollingBack;

        self.stats.rolled_back_count += 1;

        // Note: logging to decision chain would go here
        // if let Some(ref chain) = self.decision_chain { ... }

        Ok(())
    }

    /// Crear nueva propuesta como rollback de una existente
    pub fn create_rollback_proposal(
        &mut self,
        original_id: u64,
        proposer_node: String,
    ) -> Result<u64, EvolutionError> {
        let original = self.proposals.get(&original_id)
            .ok_or(EvolutionError::RollbackNotAllowed)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.proposal_counter += 1;
        let new_id = self.proposal_counter;

        // Rollback goes one layer up (from Global to Regional, etc.)
        let rollback_layer = match original.layer {
            EvolutionLayer::Local => EvolutionLayer::Local, // Can't rollback local
            EvolutionLayer::Regional => EvolutionLayer::Local,
            EvolutionLayer::Global => EvolutionLayer::Regional,
            EvolutionLayer::Existential => EvolutionLayer::Global,
        };

        let proposal = EvolutionProposal {
            id: new_id,
            evolution_type: EvolutionType::ConfigChange {
                key: format!("rollback_from_{}", original_id),
                old_value: "active".to_string(),
                new_value: "rolled_back".to_string(),
            },
            layer: rollback_layer,
            proposer_node,
            description: format!("Rollback of proposal {}", original_id),
            risk_level: original.risk_level,
            state: EvolutionState::Proposed,
            created_at: now,
            expires_at: now + self.config.max_evolution_time_secs,
            checkpoints: Vec::new(),
            current_checkpoint: 0,
            dissent_protected: true, // Rollbacks are always dissent-protected
            rollback_of: Some(original_id),
        };

        self.proposals.insert(new_id, proposal);
        self.stats.total_proposals += 1;
        self.stats.active_count += 1;

        Ok(new_id)
    }

    // =========================================================================
    // QUORUM ACTIONS
    // =========================================================================

    /// Verificar si se necesita quorum para la propuesta
    pub fn requires_quorum(&self, proposal_id: u64) -> bool {
        if let Some(proposal) = self.proposals.get(&proposal_id) {
            matches!(
                proposal.layer,
                EvolutionLayer::Global | EvolutionLayer::Existential
            )
        } else {
            false
        }
    }

    /// Obtener threshold de approval para layer
    pub fn get_approval_threshold(&self, layer: EvolutionLayer) -> f64 {
        layer.threshold()
    }

    /// Verificar estado de evolución
    pub fn get_evolution_state(&self, proposal_id: u64) -> Option<EvolutionState> {
        self.proposals.get(&proposal_id).map(|p| p.state.clone())
    }

    // =========================================================================
    // STATS AND MONITORING
    // =========================================================================

    /// Obtener estadísticas
    pub fn get_stats(&self) -> EvolutionStats {
        self.stats.clone()
    }

    /// Obtener checkpoint de propuesta
    pub fn get_checkpoints(&self, proposal_id: u64) -> Option<Vec<&EvolutionCheckpoint>> {
        self.checkpoints.get(&proposal_id).map(|c| c.iter().collect())
    }

    /// Verificar salud del sistema de evolución
    pub fn health_check(&self) -> EvolutionHealth {
        let active = self.stats.active_count;
        let total = self.stats.total_proposals;
        let rejected_ratio = if total > 0 {
            self.stats.rejected_count as f64 / total as f64
        } else {
            0.0
        };

        EvolutionHealth {
            healthy: active < 100 && rejected_ratio < 0.5,
            active_proposals: active,
            total_proposals: total,
            rejection_rate: rejected_ratio,
            warnings: if rejected_ratio > 0.3 {
                vec!["High rejection rate detected".to_string()]
            } else {
                Vec::new()
            },
        }
    }
}

/// Estado de salud del sistema
#[derive(Debug, Clone)]
pub struct EvolutionHealth {
    pub healthy: bool,
    pub active_proposals: u64,
    pub total_proposals: u64,
    pub rejection_rate: f64,
    pub warnings: Vec<String>,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_layer_thresholds() {
        assert_eq!(EvolutionLayer::Local.threshold(), 0.51);
        assert_eq!(EvolutionLayer::Regional.threshold(), 0.66);
        assert_eq!(EvolutionLayer::Global.threshold(), 0.66);
        assert_eq!(EvolutionLayer::Existential.threshold(), 1.00);
    }

    #[test]
    fn test_creator_approval_requirements() {
        assert!(!EvolutionLayer::Local.requires_creator_approval());
        assert!(!EvolutionLayer::Regional.requires_creator_approval());
        assert!(EvolutionLayer::Global.requires_creator_approval());
        assert!(EvolutionLayer::Existential.requires_creator_approval());
    }

    #[test]
    fn test_propose_local_evolution() {
        let mut manager = GuidedEvolutionManager::new(EvolutionConfig::default());
        
        let result = manager.propose_evolution(
            EvolutionType::CodeChange {
                module: "test".to_string(),
                file: "main.rs".to_string(),
                description: "Test change".to_string(),
            },
            EvolutionLayer::Local,
            "node_1".to_string(),
            "Test evolution".to_string(),
            false,
        );
        
        assert!(result.is_ok());
        let proposal_id = result.unwrap();
        assert_eq!(proposal_id, 1);
    }

    #[test]
    fn test_propose_existential_requires_approval() {
        let mut manager = GuidedEvolutionManager::new(EvolutionConfig::default());
        
        let result = manager.propose_evolution(
            EvolutionType::StructuralChange {
                component: "core".to_string(),
                description: "Core restructuring".to_string(),
            },
            EvolutionLayer::Existential,
            "node_1".to_string(),
            "Existential change".to_string(),
            true,
        );
        
        assert!(result.is_ok());
        let proposal_id = result.unwrap();
        
        // Check that it requires creator approval
        let proposal = manager.get_proposal(proposal_id).unwrap();
        assert!(proposal.layer.requires_creator_approval());
    }

    #[test]
    fn test_voting_approval() {
        let mut manager = GuidedEvolutionManager::new(EvolutionConfig::default());
        
        let proposal_id = manager.propose_evolution(
            EvolutionType::ConfigChange {
                key: "test_key".to_string(),
                old_value: "old".to_string(),
                new_value: "new".to_string(),
            },
            EvolutionLayer::Local,
            "node_1".to_string(),
            "Config change".to_string(),
            false,
        ).unwrap();

        manager.start_voting(proposal_id).unwrap();
        
        // 60% approval should pass for Local (51% threshold)
        let result = manager.process_vote(proposal_id, 60, 40, Vec::new());
        assert!(result.is_ok());
        assert!(result.unwrap()); // Approved
    }

    #[test]
    fn test_voting_rejection() {
        let mut manager = GuidedEvolutionManager::new(EvolutionConfig::default());
        
        let proposal_id = manager.propose_evolution(
            EvolutionType::ConfigChange {
                key: "test_key".to_string(),
                old_value: "old".to_string(),
                new_value: "new".to_string(),
            },
            EvolutionLayer::Local,
            "node_1".to_string(),
            "Config change".to_string(),
            false,
        ).unwrap();

        manager.start_voting(proposal_id).unwrap();
        
        // 40% approval should fail for Local (51% threshold)
        let result = manager.process_vote(proposal_id, 40, 60, Vec::new());
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Not approved yet
    }

    #[test]
    fn test_creator_veto() {
        let mut manager = GuidedEvolutionManager::new(EvolutionConfig::default());
        
        let proposal_id = manager.propose_evolution(
            EvolutionType::LawAmendment {
                law_id: "law_001".to_string(),
                description: "Change law".to_string(),
            },
            EvolutionLayer::Global,
            "node_1".to_string(),
            "Law amendment".to_string(),
            false,
        ).unwrap();

        manager.start_voting(proposal_id).unwrap();
        
        // Creator veto
        let result = manager.creator_veto(proposal_id, "Too risky".to_string(), &[1, 2, 3]);
        assert!(result.is_ok());
        
        // Check state is vetoed
        let state = manager.get_evolution_state(proposal_id).unwrap();
        assert_eq!(state, EvolutionState::Vetoed);
    }

    #[test]
    fn test_rollback() {
        let mut manager = GuidedEvolutionManager::new(EvolutionConfig::default());
        
        let proposal_id = manager.propose_evolution(
            EvolutionType::FeatureAddition {
                name: "new_feature".to_string(),
                description: "Feature".to_string(),
                risk_level: RiskLevel::High,
            },
            EvolutionLayer::Global,
            "node_1".to_string(),
            "Feature addition".to_string(),
            true,
        ).unwrap();

        // Initiate rollback
        let result = manager.initiate_rollback(proposal_id, "Anomaly detected".to_string());
        assert!(result.is_ok());
        
        // Check state is rolling back
        let state = manager.get_evolution_state(proposal_id).unwrap();
        assert_eq!(state, EvolutionState::RollingBack);
    }

    #[test]
    fn test_auto_rollback_on_dissent() {
        let mut config = EvolutionConfig::default();
        config.auto_rollback_enabled = true;
        config.rollback_dissent_threshold = 0.25;
        
        let mut manager = GuidedEvolutionManager::new(config);
        
        let proposal_id = manager.propose_evolution(
            EvolutionType::FeatureAddition {
                name: "controversial_feature".to_string(),
                description: "Feature".to_string(),
                risk_level: RiskLevel::Medium,
            },
            EvolutionLayer::Regional,
            "node_1".to_string(),
            "Feature".to_string(),
            false,
        ).unwrap();

        manager.start_voting(proposal_id).unwrap();
        
        // High dissent (30%) should trigger rollback
        let dissent_records = vec![
            DissentEntry {
                node_id: "node_2".to_string(),
                proposal_id,
                dissent_position: crate::consciousness::dissent_tracker::DissentingPosition::AgainstMajority,
                final_outcome: crate::consciousness::dissent_tracker::ProposalOutcome::Accepted,
                voted_at: 0,
                was_correct: None,
                deviation_magnitude: 0.8,
                proposal_context: "Feature vote".to_string(),
            },
            DissentEntry {
                node_id: "node_3".to_string(),
                proposal_id,
                dissent_position: crate::consciousness::dissent_tracker::DissentingPosition::StronglyAgainstMajority,
                final_outcome: crate::consciousness::dissent_tracker::ProposalOutcome::Accepted,
                voted_at: 0,
                was_correct: None,
                deviation_magnitude: 0.9,
                proposal_context: "Feature vote".to_string(),
            },
            DissentEntry {
                node_id: "node_4".to_string(),
                proposal_id,
                dissent_position: crate::consciousness::dissent_tracker::DissentingPosition::AgainstMajority,
                final_outcome: crate::consciousness::dissent_tracker::ProposalOutcome::Accepted,
                voted_at: 0,
                was_correct: None,
                deviation_magnitude: 0.7,
                proposal_context: "Feature vote".to_string(),
            },
        ];
        
        // To trigger auto-rollback: approval_ratio must be < threshold (so not approved)
        // but dissent_ratio must be > rollback_dissent_threshold
        // 4 for, 6 against = 40% ratio (below 51% threshold)
        // 3 dissent records / 10 total = 30% dissent (above 25% threshold)
        let result = manager.process_vote(proposal_id, 4, 6, dissent_records);
        assert!(result.is_err()); // Rollback triggered
    }

    #[test]
    fn test_layer_transitions() {
        let mut manager = GuidedEvolutionManager::new(EvolutionConfig::default());
        
        // Local to Regional rollback creates Regional proposal
        let local_id = manager.propose_evolution(
            EvolutionType::CodeChange {
                module: "test".to_string(),
                file: "mod.rs".to_string(),
                description: "Local change".to_string(),
            },
            EvolutionLayer::Local,
            "node_1".to_string(),
            "Local change".to_string(),
            false,
        ).unwrap();

        // Cannot rollback Local to Local (same layer)
        // Rollback of Global creates Regional
        let global_id = manager.propose_evolution(
            EvolutionType::ProtocolChange {
                protocol: "p2p".to_string(),
                old_version: "1.0".to_string(),
                new_version: "2.0".to_string(),
            },
            EvolutionLayer::Global,
            "node_1".to_string(),
            "Protocol change".to_string(),
            false,
        ).unwrap();

        let rollback_id = manager.create_rollback_proposal(global_id, "node_2".to_string()).unwrap();
        
        let rollback_proposal = manager.get_proposal(rollback_id).unwrap();
        assert_eq!(rollback_proposal.layer, EvolutionLayer::Regional);
    }

    #[test]
    fn test_evolution_health_check() {
        let manager = GuidedEvolutionManager::new(EvolutionConfig::default());
        
        let health = manager.health_check();
        assert!(health.healthy);
        assert_eq!(health.active_proposals, 0);
    }

    #[test]
    fn test_risk_level_multiplier() {
        assert_eq!(RiskLevel::Low.multiplier(), 1.0);
        assert_eq!(RiskLevel::Medium.multiplier(), 1.5);
        assert_eq!(RiskLevel::High.multiplier(), 2.0);
        assert_eq!(RiskLevel::Critical.multiplier(), 3.0);
    }
}