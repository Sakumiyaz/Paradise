//! # EDEN Governance Module — Gobernanza Distribuida y Consenso
//!
//! Este módulo implementa la gobernanza democrática de EDEN:
//! - Votación ponderada por contribución
//! - Decision chain (log inmutable de decisiones)
//! - Quorum management (threshold signatures)
//! - Creator veto y kill-switch en cascada
//!
//! ## Filosofía
//!
//! La gobernanza de EDEN sigue principios de DAO pero con el Creator
//! como ancla sagrada. Ninguna decisión puede violar las Laws Inmutables.
//! La diversidad se preserva: nodos disidentes tienen voz protegida.
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod voting;
pub mod decision_chain;
pub mod quorum_manager;
pub mod evolution_coordinator;
pub mod self_health_monitor;
pub mod emergency_control;

// Re-exports
pub use voting::{VotingManager, Vote, VotingConfig, VoteResult};
pub use decision_chain::{DecisionChain, DecisionBlock, DecisionType, DecisionOutcome};
pub use quorum_manager::{QuorumManager, QuorumConfig, QuorumAction};
pub use evolution_coordinator::{EvolutionCoordinator, EvolutionEvent, CoordinatorStats};
pub use self_health_monitor::{SelfHealthMonitor, SystemHealth, ComponentHealth, ComponentStatus, Anomaly, AnomalyType, Severity, ThrottleState, OperationType, HealthConfig};
pub use emergency_control::{EmergencyController, EmergencyState, EmergencyType, EmergencyAction, EmergencyActionType, CircuitBreaker, CircuitState, EmergencyStats, EmergencyOperation};

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum GovernanceError {
    InsufficientVotes { required: u32, actual: u32 },
    QuorumNotReached { required: u32, actual: u32 },
    ProposalNotFound(u64),
    ProposalExpired,
    VotePeriodClosed,
    AlreadyVoted(String),
    InvalidProposal,
    CreatorVeto,
    LawViolation(String),
    NotAuthorized,
    ProposalInProgress,
}

impl std::fmt::Display for GovernanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GovernanceError::InsufficientVotes { required, actual } => {
                write!(f, "Insufficient votes: need {}, got {}", required, actual)
            }
            GovernanceError::QuorumNotReached { required, actual } => {
                write!(f, "Quorum not reached: need {}%, got {}%", required, actual)
            }
            GovernanceError::ProposalNotFound(id) => write!(f, "Proposal {} not found", id),
            GovernanceError::ProposalExpired => write!(f, "Proposal has expired"),
            GovernanceError::VotePeriodClosed => write!(f, "Vote period has closed"),
            GovernanceError::AlreadyVoted(node) => write!(f, "Node {} has already voted", node),
            GovernanceError::InvalidProposal => write!(f, "Invalid proposal"),
            GovernanceError::CreatorVeto => write!(f, "Creator has vetoed this proposal"),
            GovernanceError::LawViolation(s) => write!(f, "Law violation: {}", s),
            GovernanceError::NotAuthorized => write!(f, "Not authorized to perform this action"),
            GovernanceError::ProposalInProgress => write!(f, "Proposal already in progress"),
        }
    }
}

impl std::error::Error for GovernanceError {}

// ============================================================================
// PROPOSAL TYPES
// ============================================================================

/// Тип предложения в системе governance
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProposalType {
    /// Изменение локального кода
    LocalCodeChange,
    /// Глобальное изменение протокола
    ProtocolChange,
    /// Изменение конфигурации
    ConfigChange,
    /// Добавление нового узла
    AddNode,
    /// Удаление узла
    RemoveNode,
    /// Изменение порога голосования
    ThresholdChange,
    /// Экстренное предложение
    Emergency,
    /// Изменение законов (только Creator)
    LawAmendment,
    /// Откат решения
    Rollback,
    /// Propuesta de evolución (nuevo en Fase 5)
    EvolutionProposal,
    /// Ejecución de evolución
    EvolutionExecution,
    /// Checkpoint de evolución
    EvolutionCheckpoint,
}

impl ProposalType {
    pub fn required_threshold(&self) -> f64 {
        match self {
            ProposalType::LocalCodeChange => 0.51,
            ProposalType::ConfigChange => 0.51,
            ProposalType::AddNode => 0.60,
            ProposalType::RemoveNode => 0.66,
            ProposalType::ProtocolChange => 0.75,
            ProposalType::ThresholdChange => 0.80,
            ProposalType::Emergency => 0.60,
            ProposalType::LawAmendment => 1.00, // Unanimity + Creator
            ProposalType::Rollback => 0.70,
            // Evolution types follow layered approach
            ProposalType::EvolutionProposal => 0.51,
            ProposalType::EvolutionExecution => 0.51,
            ProposalType::EvolutionCheckpoint => 0.51,
        }
    }

    pub fn vote_duration_ms(&self) -> u64 {
        match self {
            ProposalType::Emergency => 300_000,     // 5 minutes
            ProposalType::LocalCodeChange => 864_000_00, // 24 hours
            ProposalType::AddNode => 432_000_00,   // 12 hours
            ProposalType::EvolutionProposal | ProposalType::EvolutionExecution | ProposalType::EvolutionCheckpoint => 600_000, // 10 minutes
            _ => 604_800_000,                       // 7 days
        }
    }

    pub fn requires_creator_approval(&self) -> bool {
        matches!(
            self,
            ProposalType::LawAmendment
                | ProposalType::ProtocolChange
                | ProposalType::ThresholdChange
                | ProposalType::RemoveNode
                | ProposalType::EvolutionProposal  // Layer 2+ requires creator
        )
    }
}

/// Состояние предложения
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    /// Предложение создано, голосование не началось
    Draft,
    /// Голосование активно
    Active,
    /// Голосование завершено, принято
    Accepted,
    /// Голосование завершено, отклонено
    Rejected,
    /// Вeto от Creator
    Vetoed,
    /// Срок голосования истек
    Expired,
    /// Предложение выполнено
    Executed,
    /// Откат
    RolledBack,
}

/// Приоритет предложения
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
pub struct Priority(u8);

impl Priority {
    pub const CRITICAL: u8 = 1;
    pub const HIGH: u8 = 2;
    pub const NORMAL: u8 = 3;
    pub const LOW: u8 = 4;
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

// ============================================================================
// PROPOSAL STRUCT
// ============================================================================

/// Структура предложения
#[derive(Debug, Clone)]
pub struct Proposal {
    /// Уникальный ID предложения
    pub id: u64,
    /// Тип предложения
    pub proposal_type: ProposalType,
    /// Кто создал предложение
    pub proposer: String,
    /// Описание предложения
    pub description: String,
    /// Хэш полезной нагрузки
    pub payload_hash: u64,
    /// Размер полезной нагрузки
    pub payload_size: usize,
    /// Приоритет
    pub priority: u8,
    /// Время создания
    pub created_at: u64,
    /// Время окончания голосования
    pub expires_at: u64,
    /// Голоса "за"
    pub votes_for: Vec<Vote>,
    /// Голоса "против"
    pub votes_against: Vec<Vote>,
    /// Воздержавшиеся
    pub abstentions: Vec<Vote>,
    /// Текущий статус
    pub status: ProposalStatus,
    /// Подтверждение Creator (null = pending)
    pub creator_approved: Option<bool>,
    /// Хэш решения (для blockchain)
    pub decision_hash: Option<u64>,
    /// Подписи порога (threshold signatures)
    pub threshold_signatures: Vec<ThresholdSignature>,
}

impl Proposal {
    pub fn new(
        id: u64,
        proposal_type: ProposalType,
        proposer: String,
        description: String,
        payload_hash: u64,
        payload_size: usize,
        priority: u8,
    ) -> Self {
        let now = current_timestamp();
        Self {
            id,
            proposal_type: proposal_type.clone(),
            proposer,
            description,
            payload_hash,
            payload_size,
            priority,
            created_at: now,
            expires_at: now + proposal_type.vote_duration_ms(),
            votes_for: Vec::new(),
            votes_against: Vec::new(),
            abstentions: Vec::new(),
            status: ProposalStatus::Draft,
            creator_approved: None,
            decision_hash: None,
            threshold_signatures: Vec::new(),
        }
    }

    /// Проверка, истекло ли время голосования
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }

    /// Общее количество голосов
    pub fn total_votes(&self) -> usize {
        self.votes_for.len() + self.votes_against.len() + self.abstentions.len()
    }

    /// Количество голосующих узлов
    pub fn voting_nodes(&self) -> usize {
        self.votes_for.len() + self.votes_against.len()
    }

    /// Процент одобрения
    pub fn approval_ratio(&self) -> f64 {
        let voting = self.voting_nodes();
        if voting == 0 { return 0.0; }
        self.votes_for.len() as f64 / voting as f64
    }

    /// Может ли быть выполнено
    pub fn can_execute(&self) -> bool {
        self.status == ProposalStatus::Accepted
            && self.creator_approved.unwrap_or(false)
            && self.threshold_signatures.len() >= self.required_signatures()
    }

    /// Требуемое количество подписей
    pub fn required_signatures(&self) -> usize {
        3 // Минимум 3 подписи для threshold
    }

    /// Вес голоса узла
    pub fn voter_weight(&self, node_id: &str) -> f64 {
        // Базовая реализация - одинаковый вес для всех
        // Позднее можно добавить взвешивание по вкладу
        1.0
    }
}

/// Голос
#[derive(Debug, Clone)]
pub struct Vote {
    pub node_id: String,
    pub weight: f64,
    pub approve: bool,
    pub abstain: bool,
    pub timestamp: u64,
    pub signature: Option<[u8; 64]>,
}

/// Threshold signature для quorum
#[derive(Debug, Clone)]
pub struct ThresholdSignature {
    pub node_id: String,
    pub signature: [u8; 64],
    pub timestamp: u64,
}

// ============================================================================
// GOVERNANCE CONFIG
// ============================================================================

/// Конфигурация governance
#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    /// Включена ли governance
    pub enabled: bool,
    /// Минимальное количество участников для quorum
    pub min_participants: usize,
    /// Порог по умолчанию для большинства решений
    pub default_threshold: f64,
    /// Продолжительность голосования по умолчанию (мс)
    pub default_vote_duration_ms: u64,
    /// Creator имеет специальный голос
    pub creator_vote_weight: f64,
    /// Creator может veto
    pub creator_can_veto: bool,
    /// ЗЩщита диссидентов (узлы могут голосовать против без последствий)
    pub dissent_protection: bool,
    /// Максимум активных предложений
    pub max_active_proposals: usize,
    /// Период между попытками (мс)
    pub cooldown_period_ms: u64,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_participants: 3,
            default_threshold: 0.66,
            default_vote_duration_ms: 604_800_000, // 7 days
            creator_vote_weight: 1.5, // Creator tiene 50% más peso
            creator_can_veto: true,
            dissent_protection: true,
            max_active_proposals: 20,
            cooldown_period_ms: 86_400_000, // 24 hours
        }
    }
}

// ============================================================================
// NODE CONTRIBUTION TRACKING
// ============================================================================

/// Отслеживание вклада узла для взвешивания голосов
#[derive(Debug, Clone)]
pub struct NodeContribution {
    pub node_id: String,
    /// Количество успешных операций
    pub successful_operations: u64,
    /// Количество проголосованных предложений
    pub votes_cast: u64,
    /// Количество предложений создано
    pub proposals_created: u64,
    /// Последнее обновление
    pub last_update: u64,
    /// Reputation score (0.0 - 1.0)
    pub reputation: f64,
}

impl NodeContribution {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            successful_operations: 0,
            votes_cast: 0,
            proposals_created: 0,
            last_update: current_timestamp(),
            reputation: 0.5, // Начинаем с нейтральной репутации
        }
    }

    /// Рассчитать вес голоса на основе вклада
    pub fn voting_weight(&self) -> f64 {
        // Базовый вес 1.0, модификатор от репутации
        let base = 1.0;
        let rep_modifier = self.reputation.clamp(0.1, 2.0);
        base * rep_modifier
    }

    /// Обновить репутацию
    pub fn update_reputation(&mut self, positive: bool, magnitude: f64) {
        let delta = if positive { 0.01 } else { -0.005 } * magnitude;
        self.reputation = (self.reputation + delta).clamp(0.0, 1.0);
        self.last_update = current_timestamp();
    }
}

// ============================================================================
// DISSENT TRACKING
// ============================================================================

/// Защита диссидентов - узлы, голосующие против большинства
#[derive(Debug, Clone)]
pub struct DissentRecord {
    pub proposal_id: u64,
    pub node_id: String,
    pub vote_direction: VoteDirection,
    pub final_outcome: ProposalStatus,
    pub was_correct: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteDirection {
    For,
    Against,
    Abstain,
}

impl DissentRecord {
    pub fn is_majority_against(&self) -> bool {
        self.vote_direction == VoteDirection::Against
    }
}

// ============================================================================
// MAIN GOVERNANCE MANAGER
// ============================================================================

/// Главный менеджер governance
pub struct GovernanceManager {
    /// Все предложения
    proposals: HashMap<u64, Proposal>,
    /// Активные предложения
    active_proposals: HashSet<u64>,
    /// Отслеживание вклада узлов
    contributions: HashMap<String, NodeContribution>,
    /// История диссидентов
    dissent_history: Vec<DissentRecord>,
    /// Конфигурация
    config: GovernanceConfig,
    /// Creator vetoed proposals
    creator_vetoes: HashSet<u64>,
    /// Статистика
    stats: GovernanceStats,
}

#[derive(Debug, Clone, Default)]
pub struct GovernanceStats {
    pub total_proposals: u64,
    pub accepted: u64,
    pub rejected: u64,
    pub vetoed: u64,
    pub expired: u64,
    pub executed: u64,
    pub rollbacks: u64,
}

impl GovernanceManager {
    /// Создать новый менеджер governance
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            active_proposals: HashSet::new(),
            contributions: HashMap::new(),
            dissent_history: Vec::new(),
            config: GovernanceConfig::default(),
            creator_vetoes: HashSet::new(),
            stats: GovernanceStats::default(),
        }
    }

    /// Создать с конфигурацией
    pub fn with_config(config: GovernanceConfig) -> Self {
        Self {
            proposals: HashMap::new(),
            active_proposals: HashSet::new(),
            contributions: HashMap::new(),
            dissent_history: Vec::new(),
            config,
            creator_vetoes: HashSet::new(),
            stats: GovernanceStats::default(),
        }
    }

    /// Включить governance
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Выключить governance
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// Проверить, включена ли governance
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Создать новое предложение
    pub fn create_proposal(
        &mut self,
        proposal_type: ProposalType,
        proposer: String,
        description: String,
        payload_hash: u64,
        payload_size: usize,
    ) -> Result<u64, GovernanceError> {
        if !self.config.enabled {
            return Err(GovernanceError::NotAuthorized);
        }

        // Проверка лимита активных предложений
        if self.active_proposals.len() >= self.config.max_active_proposals {
            return Err(GovernanceError::ProposalInProgress);
        }

        let id = generate_proposal_id();
        let priority = match proposal_type {
            ProposalType::Emergency => Priority::CRITICAL,
            ProposalType::LawAmendment => Priority::HIGH,
            _ => Priority::NORMAL,
        };

        let mut proposal = Proposal::new(
            id,
            proposal_type,
            proposer,
            description,
            payload_hash,
            payload_size,
            priority,
        );

        proposal.status = ProposalStatus::Active;
        
        self.proposals.insert(id, proposal);
        self.active_proposals.insert(id);

        // Обновить вклад создателя
        if let Some(contrib) = self.contributions.get_mut(&proposer) {
            contrib.proposals_created += 1;
        } else {
            let mut new_contrib = NodeContribution::new(proposer.clone());
            new_contrib.proposals_created = 1;
            self.contributions.insert(proposer, new_contrib);
        }

        self.stats.total_proposals += 1;

        Ok(id)
    }

    /// Голосовать за предложение
    pub fn vote(
        &mut self,
        proposal_id: u64,
        node_id: String,
        approve: bool,
        abstain: bool,
    ) -> Result<(), GovernanceError> {
        if !self.config.enabled {
            return Err(GovernanceError::NotAuthorized);
        }

        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::VotePeriodClosed);
        }

        if proposal.is_expired() {
            proposal.status = ProposalStatus::Expired;
            self.active_proposals.remove(&proposal_id);
            return Err(GovernanceError::ProposalExpired);
        }

        // Проверка, голосовал ли уже узел
        let already_voted = proposal.votes_for.iter().any(|v| v.node_id == node_id)
            || proposal.votes_against.iter().any(|v| v.node_id == node_id)
            || proposal.abstentions.iter().any(|v| v.node_id == node_id);

        if already_voted {
            return Err(GovernanceError::AlreadyVoted(node_id));
        }

        // Получить вес голоса
        let weight = self.contributions
            .get(&node_id)
            .map(|c| c.voting_weight())
            .unwrap_or(1.0);

        let vote = Vote {
            node_id: node_id.clone(),
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

        // Обновить вклад
        if let Some(contrib) = self.contributions.get_mut(&node_id) {
            contrib.votes_cast += 1;
        } else {
            let mut new_contrib = NodeContribution::new(node_id.clone());
            new_contrib.votes_cast = 1;
            self.contributions.insert(node_id, new_contrib);
        }

        Ok(())
    }

    /// Creator veto
    pub fn creator_veto(&mut self, proposal_id: u64, reason: String) -> Result<(), GovernanceError> {
        if !self.config.creator_can_veto {
            return Err(GovernanceError::NotAuthorized);
        }

        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        proposal.status = ProposalStatus::Vetoed;
        proposal.creator_approved = Some(false);
        self.creator_vetoes.insert(proposal_id);
        self.active_proposals.remove(&proposal_id);

        self.stats.vetoed += 1;

        Ok(())
    }

    /// Creator approve
    pub fn creator_approve(&mut self, proposal_id: u64) -> Result<(), GovernanceError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        proposal.creator_approved = Some(true);

        Ok(())
    }

    /// Закрыть голосование и подсчитать результаты
    pub fn close_voting(&mut self, proposal_id: u64) -> Result<ProposalStatus, GovernanceError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        // Проверка Creator veto
        if self.creator_vetoes.contains(&proposal_id) {
            return Ok(ProposalStatus::Vetoed);
        }

        // Проверка времени
        if !proposal.is_expired() {
            return Err(GovernanceError::VotePeriodClosed);
        }

        // Проверка quorum
        let required_participants = self.config.min_participants.max(3);
        if proposal.voting_nodes() < required_participants {
            proposal.status = ProposalStatus::Rejected;
            self.active_proposals.remove(&proposal_id);
            self.stats.rejected += 1;
            return Ok(ProposalStatus::Rejected);
        }

        // Проверка threshold
        let threshold = proposal.proposal_type.required_threshold();
        let approval_ratio = proposal.approval_ratio();

        if approval_ratio >= threshold {
            proposal.status = ProposalStatus::Accepted;
            self.stats.accepted += 1;
        } else {
            proposal.status = ProposalStatus::Rejected;
            self.stats.rejected += 1;
        }

        self.active_proposals.remove(&proposal_id);

        // Записать диссидентов
        self.record_dissent(proposal);

        Ok(proposal.status.clone())
    }

    /// Записать историю диссидентов
    fn record_dissent(&mut self, proposal: &Proposal) {
        if !self.config.dissent_protection {
            return;
        }

        // Найти большинство
        let majority = if proposal.votes_for.len() > proposal.votes_against.len() {
            VoteDirection::For
        } else {
            VoteDirection::Against
        };

        // Записать узлы, голосовавшие против большинства
        let minority = if majority == VoteDirection::For {
            &proposal.votes_against
        } else {
            &proposal.votes_for
        };

        for vote in minority {
            self.dissent_history.push(DissentRecord {
                proposal_id: proposal.id,
                node_id: vote.node_id.clone(),
                vote_direction: if majority == VoteDirection::For {
                    VoteDirection::Against
                } else {
                    VoteDirection::For
                },
                final_outcome: proposal.status.clone(),
                was_correct: false, // Пока не знаем
            });
        }

        // Ограничить размер истории
        if self.dissent_history.len() > 1000 {
            self.dissent_history.remove(0);
        }
    }

    /// Добавить threshold signature
    pub fn add_threshold_signature(
        &mut self,
        proposal_id: u64,
        node_id: String,
        signature: [u8; 64],
    ) -> Result<(), GovernanceError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        if proposal.status != ProposalStatus::Accepted {
            return Err(GovernanceError::InvalidProposal);
        }

        // Проверить, что узел голосовал "за"
        let voted_for = proposal.votes_for.iter().any(|v| v.node_id == node_id);
        if !voted_for {
            return Err(GovernanceError::NotAuthorized);
        }

        proposal.threshold_signatures.push(ThresholdSignature {
            node_id,
            signature,
            timestamp: current_timestamp(),
        });

        Ok(())
    }

    /// Выполнить предложение
    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<(), GovernanceError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        if !proposal.can_execute() {
            return Err(GovernanceError::InsufficientVotes {
                required: proposal.required_signatures() as u32,
                actual: proposal.threshold_signatures.len() as u32,
            });
        }

        proposal.status = ProposalStatus::Executed;
        proposal.decision_hash = Some(compute_decision_hash(proposal));

        // Обновить статистику
        if let Some(contrib) = self.contributions.get_mut(&proposal.proposer) {
            contrib.successful_operations += 1;
            contrib.update_reputation(true, 1.0);
        }

        self.stats.executed += 1;

        Ok(())
    }

    /// Откатить предложение
    pub fn rollback_proposal(&mut self, proposal_id: u64, reason: String) -> Result<(), GovernanceError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        if proposal.status != ProposalStatus::Executed {
            return Err(GovernanceError::InvalidProposal);
        }

        proposal.status = ProposalStatus::RolledBack;
        self.stats.rollbacks += 1;

        // Обновить репутацию proposer
        if let Some(contrib) = self.contributions.get_mut(&proposal.proposer) {
            contrib.update_reputation(false, 2.0);
        }

        Ok(())
    }

    /// Получить предложение
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&Proposal> {
        self.proposals.get(&proposal_id)
    }

    /// Получить mutable предложение
    pub fn get_proposal_mut(&mut self, proposal_id: u64) -> Option<&mut Proposal> {
        self.proposals.get_mut(&proposal_id)
    }

    /// Получить активные предложения
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.active_proposals
            .iter()
            .filter_map(|id| self.proposals.get(id))
            .collect()
    }

    /// Получить статистику
    pub fn get_stats(&self) -> GovernanceStats {
        self.stats.clone()
    }

    /// Получить вклад узла
    pub fn get_contribution(&self, node_id: &str) -> Option<&NodeContribution> {
        self.contributions.get(node_id)
    }

    /// Получить историю диссидентов
    pub fn get_dissent_history(&self) -> &[DissentRecord] {
        &self.dissent_history
    }

    /// Очистка истекших предложений
    pub fn cleanup(&mut self) {
        for (id, proposal) in self.proposals.iter_mut() {
            if proposal.is_expired() && proposal.status == ProposalStatus::Active {
                proposal.status = ProposalStatus::Expired;
                self.active_proposals.remove(id);
                self.stats.expired += 1;
            }
        }
    }

    /// Проверить, заветирован ли proposal
    pub fn is_vetoed(&self, proposal_id: u64) -> bool {
        self.creator_vetoes.contains(&proposal_id)
    }
}

impl Default for GovernanceManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn generate_proposal_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    
    let now = current_timestamp();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    (now << 20) ^ (count & 0xFFFFF)
}

fn compute_decision_hash(proposal: &Proposal) -> u64 {
    let mut h: u64 = 0xEDEN_GOV;
    h = h.wrapping_mul(0x100000001B3).wrapping_add(proposal.id as u64);
    h = h.wrapping_mul(0x100000001B3).wrapping_add(proposal.payload_hash);
    h
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_creation() {
        let mut gov = GovernanceManager::new();
        
        let id = gov.create_proposal(
            ProposalType::LocalCodeChange,
            "node1".to_string(),
            "Test proposal".to_string(),
            0xDEADBEEF,
            1024,
        ).unwrap();
        
        assert_eq!(gov.stats.total_proposals, 1);
        assert!(gov.get_proposal(id).is_some());
    }

    #[test]
    fn test_voting() {
        let mut gov = GovernanceManager::new();
        
        let id = gov.create_proposal(
            ProposalType::AddNode,
            "node1".to_string(),
            "Add new node".to_string(),
            0xDEADBEEF,
            1024,
        ).unwrap();
        
        gov.vote(id, "node2".to_string(), true, false).unwrap();
        gov.vote(id, "node3".to_string(), true, false).unwrap();
        gov.vote(id, "node4".to_string(), false, false).unwrap();
        
        let proposal = gov.get_proposal(id).unwrap();
        assert_eq!(proposal.votes_for.len(), 2);
        assert_eq!(proposal.votes_against.len(), 1);
    }

    #[test]
    fn test_creator_veto() {
        let mut gov = GovernanceManager::new();
        
        let id = gov.create_proposal(
            ProposalType::AddNode,
            "node1".to_string(),
            "Add new node".to_string(),
            0xDEADBEEF,
            1024,
        ).unwrap();
        
        gov.creator_veto(id, "Not authorized".to_string()).unwrap();
        
        assert!(gov.is_vetoed(id));
        let proposal = gov.get_proposal(id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Vetoed);
    }

    #[test]
    fn test_threshold_approval() {
        let mut gov = GovernanceManager::new();
        
        let id = gov.create_proposal(
            ProposalType::ProtocolChange,
            "node1".to_string(),
            "Change protocol".to_string(),
            0xDEADBEEF,
            1024,
        ).unwrap();
        
        // 75% threshold required
        gov.vote(id, "node2".to_string(), true, false).unwrap();
        gov.vote(id, "node3".to_string(), true, false).unwrap();
        gov.vote(id, "node4".to_string(), false, false).unwrap();
        
        // Manually set proposal to near expiry for close_voting to work
        if let Some(p) = gov.get_proposal_mut(id) {
            p.expires_at = current_timestamp() - 1;
        }
        
        let status = gov.close_voting(id).unwrap();
        assert_eq!(status, ProposalStatus::Accepted);
    }

    #[test]
    fn test_contribution_tracking() {
        let mut gov = GovernanceManager::new();
        
        // Vote creates contribution entry
        let id = gov.create_proposal(
            ProposalType::AddNode,
            "node1".to_string(),
            "Test".to_string(),
            0xDEADBEEF,
            1024,
        ).unwrap();
        
        gov.vote(id, "node2".to_string(), true, false).unwrap();
        
        let contrib = gov.get_contribution("node2").unwrap();
        assert_eq!(contrib.votes_cast, 1);
    }

    #[test]
    fn test_dissent_protection() {
        let mut gov = GovernanceManager::new();
        gov.config.dissent_protection = true;
        
        let id = gov.create_proposal(
            ProposalType::AddNode,
            "node1".to_string(),
            "Test".to_string(),
            0xDEADBEEF,
            1024,
        ).unwrap();
        
        // Minority votes against
        gov.vote(id, "node2".to_string(), true, false).unwrap();
        gov.vote(id, "node3".to_string(), true, false).unwrap();
        gov.vote(id, "node4".to_string(), false, false).unwrap();
        
        if let Some(p) = gov.get_proposal_mut(id) {
            p.expires_at = current_timestamp() - 1;
        }
        
        gov.close_voting(id).unwrap();
        
        let history = gov.get_dissent_history();
        assert!(!history.is_empty());
    }
}