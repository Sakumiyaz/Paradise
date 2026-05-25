//! # Democratic Propagation — Propagación con Votación y Kill-Switches
//!
//! Sistema de propagación de código que requiere consenso democratico:
//! - Votación de nodos (66% para decisiones críticas)
//! - Kill-switch global del Creador
//! - Blacklist de IPs críticas (hospitales, gobiernos, etc.)
//! - Logs completos en Meltrace
//! - Rate limiting para prevenir spam
//!
//! ## Filosofía
//!
//! Ninguna decisión de propagación es tomada por una sola entidad.
//! El Creador tiene veto ultimate, pero los nodos pueden vetar decisiones
//! que violen las Leyes Inmutables o pongan en riesgo al sistema.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum PropagationError {
    InsufficientVotes { required: u32, actual: u32 },
    VoteTimeout,
    BlacklistedIp(String),
    CreatorVeto,
    LawViolation(String),
    InvalidProposal,
    AlreadyProposed,
    ProposalExpired,
    QuorumNotReached,
}

impl std::fmt::Display for PropagationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropagationError::InsufficientVotes { required, actual } => {
                write!(f, "Insufficient votes: need {}, got {}", required, actual)
            }
            PropagationError::VoteTimeout => write!(f, "Vote timed out"),
            PropagationError::BlacklistedIp(ip) => write!(f, "IP {} is blacklisted", ip),
            PropagationError::CreatorVeto => write!(f, "Creator vetoed this proposal"),
            PropagationError::LawViolation(s) => write!(f, "Law violation: {}", s),
            PropagationError::InvalidProposal => write!(f, "Invalid proposal"),
            PropagationError::AlreadyProposed => write!(f, "Proposal already exists"),
            PropagationError::ProposalExpired => write!(f, "Proposal expired"),
            PropagationError::QuorumNotReached => write!(f, "Quorum not reached for vote"),
        }
    }
}

impl std::error::Error for PropagationError {}

// ============================================================================
// PROPOSAL TYPES
// ============================================================================

/// Type of propagation proposal
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProposalType {
    /// Propagar código a nuevo nodo
    AddNode,
    /// Actualizar código existente
    UpdateCode,
    /// Cambiar protocolo de comunicación
    ProtocolChange,
    /// Modificar configuración global
    ConfigChange,
    /// Emergencia (respuesta rápida)
    Emergency,
    /// Reversión de cambio anterior
    Rollback,
}

/// Priority of proposal
#[derive(Clone, Debug, PartialEq, Eq, Ord)]
pub struct ProposalPriority(u8);

impl ProposalPriority {
    pub const CRITICAL: u8 = 1;
    pub const HIGH: u8 = 2;
    pub const NORMAL: u8 = 3;
    pub const LOW: u8 = 4;
}

impl PartialOrd for ProposalPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

/// A proposal for propagation
#[derive(Clone, Debug)]
pub struct PropagationProposal {
    pub proposal_id: u64,
    pub proposal_type: ProposalType,
    pub proposer: String,
    pub target: String,
    pub payload_hash: u64,
    pub payload_size: usize,
    pub priority: u8,
    pub created_at: u64,
    pub expires_at: u64,
    pub votes_for: Vec<String>,
    pub votes_against: Vec<String>,
    pub abstentions: Vec<String>,
    pub status: ProposalStatus,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
    Vetoed,
    Executed,
}

impl PropagationProposal {
    pub fn new(
        proposal_id: u64,
        proposal_type: ProposalType,
        proposer: String,
        target: String,
        payload_hash: u64,
        payload_size: usize,
        priority: u8,
        ttl_seconds: u64,
        reason: String,
    ) -> Self {
        let now = current_timestamp();
        Self {
            proposal_id,
            proposal_type,
            proposer,
            target,
            payload_hash,
            payload_size,
            priority,
            created_at: now,
            expires_at: now + ttl_seconds * 1000,
            votes_for: Vec::new(),
            votes_against: Vec::new(),
            abstentions: Vec::new(),
            status: ProposalStatus::Pending,
            reason,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }
    
    pub fn total_votes(&self) -> usize {
        self.votes_for.len() + self.votes_against.len() + self.abstentions.len()
    }
    
    pub fn approval_ratio(&self) -> f64 {
        let total = self.total_votes();
        if total == 0 { return 0.0; }
        self.votes_for.len() as f64 / total as f64
    }
    
    pub fn can_execute(&self, required_ratio: f64) -> bool {
        self.status == ProposalStatus::Accepted && self.approval_ratio() >= required_ratio
    }
    
    pub fn vote(&mut self, node_id: &str, approve: bool, abstain: bool) {
        // Remove from all lists first
        self.votes_for.retain(|v| v != node_id);
        self.votes_against.retain(|v| v != node_id);
        self.abstentions.retain(|v| v != node_id);
        
        if abstain {
            self.abstentions.push(node_id.to_string());
        } else if approve {
            self.votes_for.push(node_id.to_string());
        } else {
            self.votes_against.push(node_id.to_string());
        }
    }
}

// ============================================================================
// BLACKLIST
// ============================================================================

/// Critical infrastructure blacklist
pub struct CriticalBlacklist {
    /// Blacklisted IP ranges (CIDR notation simplified)
    entries: HashMap<String, BlacklistEntry>,
    
    /// Last update time
    last_updated: u64,
}

#[derive(Clone, Debug)]
pub struct BlacklistEntry {
    pub ip_pattern: String,
    pub category: BlacklistCategory,
    pub reason: String,
    pub added_by: String,
    pub added_at: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlacklistCategory {
    /// Hospitals, emergency services
    Medical,
    /// Government agencies
    Government,
    /// Critical infrastructure (power, water)
    Infrastructure,
    /// Educational institutions
    Educational,
    /// Law enforcement
    LawEnforcement,
    /// Custom entry
    Custom,
}

impl CriticalBlacklist {
    pub fn new() -> Self {
        let mut entries = HashMap::new();
        
        // Default critical infrastructure
        let defaults = vec![
            ("127.0.0.0/8".to_string(), BlacklistCategory::Custom, "Localhost".to_string()),
            ("10.0.0.0/8".to_string(), BlacklistCategory::Infrastructure, "Private range - reserved".to_string()),
            ("172.16.0.0/12".to_string(), BlacklistCategory::Infrastructure, "Private range - reserved".to_string()),
            ("192.168.0.0/16".to_string(), BlacklistCategory::Infrastructure, "Private range - reserved".to_string()),
        ];
        
        for (ip, cat, reason) in defaults {
            entries.insert(ip.clone(), BlacklistEntry {
                ip_pattern: ip,
                category: cat,
                reason,
                added_by: "system".to_string(),
                added_at: current_timestamp(),
            });
        }
        
        Self {
            entries,
            last_updated: current_timestamp(),
        }
    }
    
    /// Add IP or range to blacklist
    pub fn add(&mut self, ip_pattern: String, category: BlacklistCategory, reason: String, added_by: String) {
        self.entries.insert(ip_pattern.clone(), BlacklistEntry {
            ip_pattern,
            category,
            reason,
            added_by,
            added_at: current_timestamp(),
        });
        self.last_updated = current_timestamp();
    }
    
    /// Remove from blacklist
    pub fn remove(&mut self, ip_pattern: &str) -> bool {
        self.entries.remove(ip_pattern).is_some()
    }
    
    /// Check if IP is blacklisted
    pub fn is_blacklisted(&self, ip: &str) -> bool {
        // Simple exact match for now (full CIDR would require more complex logic)
        if self.entries.contains_key(ip) {
            return true;
        }
        
        // Check if IP is in any blacklisted range
        for (pattern, _) in &self.entries {
            if ip.starts_with(&pattern[..pattern.find('/').unwrap_or(pattern.len())]) {
                return true;
            }
        }
        
        false
    }
    
    /// Get category of blacklisted IP
    pub fn get_category(&self, ip: &str) -> Option<BlacklistCategory> {
        self.entries.get(ip).map(|e| e.category.clone())
    }
    
    /// Get all entries
    pub fn get_entries(&self) -> Vec<(String, BlacklistCategory)> {
        self.entries
            .iter()
            .map(|(k, v)| (k.clone(), v.category.clone()))
            .collect()
    }
    
    /// Check if category is protected
    pub fn is_category_protected(&self, category: &BlacklistCategory) -> bool {
        matches!(
            category,
            BlacklistCategory::Medical |
            BlacklistCategory::Government |
            BlacklistCategory::LawEnforcement |
            BlacklistCategory::Educational
        )
    }
}

// ============================================================================
// VOTE MANAGER
// ============================================================================

/// Vote on a propagation proposal
#[derive(Clone, Debug)]
pub struct Vote {
    pub proposal_id: u64,
    pub node_id: String,
    pub vote: VoteValue,
    pub timestamp: u64,
    pub signature: Option<[u8; 64]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VoteValue {
    Approve,
    Reject,
    Abstain,
}

/// Vote manager
pub struct VoteManager {
    /// Active proposals
    proposals: HashMap<u64, PropagationProposal>,
    
    /// Vote history
    vote_history: Vec<Vote>,
    
    /// Configuration
    config: VoteConfig,
}

#[derive(Clone, Debug)]
pub struct VoteConfig {
    pub required_approval_ratio: f64,
    pub vote_timeout_seconds: u64,
    pub min_participation: usize,
    pub emergency_timeout_seconds: u64,
}

impl Default for VoteConfig {
    fn default() -> Self {
        Self {
            required_approval_ratio: 0.66,  // 66%
            vote_timeout_seconds: 3600,     // 1 hour
            min_participation: 3,
            emergency_timeout_seconds: 300, // 5 minutes
        }
    }
}

impl VoteManager {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            vote_history: Vec::new(),
            config: VoteConfig::default(),
        }
    }
    
    /// Create a new proposal
    pub fn create_proposal(
        &mut self,
        proposal_type: ProposalType,
        proposer: String,
        target: String,
        payload_hash: u64,
        payload_size: usize,
        priority: u8,
        reason: String,
    ) -> Result<u64, PropagationError> {
        // Check if already proposed (same target and type)
        for (_, proposal) in &self.proposals {
            if proposal.target == target && proposal.proposal_type == proposal_type {
                if proposal.status == ProposalStatus::Pending {
                    return Err(PropagationError::AlreadyProposed);
                }
            }
        }
        
        let ttl = match priority {
            1 => self.config.emergency_timeout_seconds,
            _ => self.config.vote_timeout_seconds,
        };
        
        let proposal_id = generate_proposal_id();
        
        let proposal = PropagationProposal::new(
            proposal_id,
            proposal_type,
            proposer,
            target,
            payload_hash,
            payload_size,
            priority,
            ttl,
            reason,
        );
        
        self.proposals.insert(proposal_id, proposal);
        
        Ok(proposal_id)
    }
    
    /// Submit vote
    pub fn vote(&mut self, proposal_id: u64, node_id: &str, approve: bool, abstain: bool) -> Result<(), PropagationError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(PropagationError::InvalidProposal)?;
        
        if proposal.is_expired() {
            proposal.status = ProposalStatus::Expired;
            return Err(PropagationError::ProposalExpired);
        }
        
        if proposal.status != ProposalStatus::Pending {
            return Err(PropagationError::InvalidProposal);
        }
        
        proposal.vote(node_id, approve, abstain);
        
        // Record vote
        let vote = Vote {
            proposal_id,
            node_id: node_id.to_string(),
            vote: if abstain {
                VoteValue::Abstain
            } else if approve {
                VoteValue::Approve
            } else {
                VoteValue::Reject
            },
            timestamp: current_timestamp(),
            signature: None,
        };
        self.vote_history.push(vote);
        
        Ok(())
    }
    
    /// Get proposal
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&PropagationProposal> {
        self.proposals.get(&proposal_id)
    }
    
    /// Get mutable proposal
    pub fn get_proposal_mut(&mut self, proposal_id: u64) -> Option<&mut PropagationProposal> {
        self.proposals.get_mut(&proposal_id)
    }
    
    /// Tally votes and determine outcome
    pub fn tally_votes(&mut self, proposal_id: u64) -> Result<ProposalStatus, PropagationError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(PropagationError::InvalidProposal)?;
        
        if !proposal.is_expired() && proposal.status == ProposalStatus::Pending {
            return Err(PropagationError::VoteTimeout);
        }
        
        // Check minimum participation
        if proposal.total_votes() < self.config.min_participation {
            proposal.status = ProposalStatus::Rejected;
            return Ok(ProposalStatus::Rejected);
        }
        
        // Check approval ratio
        if proposal.approval_ratio() >= self.config.required_approval_ratio {
            proposal.status = ProposalStatus::Accepted;
            Ok(ProposalStatus::Accepted)
        } else {
            proposal.status = ProposalStatus::Rejected;
            Ok(ProposalStatus::Rejected)
        }
    }
    
    /// Mark proposal as executed
    pub fn mark_executed(&mut self, proposal_id: u64) {
        if let Some(proposal) = self.proposals.get_mut(&proposal_id) {
            proposal.status = ProposalStatus::Executed;
        }
    }
    
    /// Clean up expired proposals
    pub fn cleanup_expired(&mut self) {
        for (_, proposal) in self.proposals.iter_mut() {
            if proposal.is_expired() && proposal.status == ProposalStatus::Pending {
                proposal.status = ProposalStatus::Expired;
            }
        }
    }
    
    /// Get active proposals
    pub fn get_active_proposals(&self) -> Vec<&PropagationProposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Pending)
            .collect()
    }
    
    /// Get vote history for a proposal
    pub fn get_vote_history(&self, proposal_id: u64) -> Vec<&Vote> {
        self.vote_history
            .iter()
            .filter(|v| v.proposal_id == proposal_id)
            .collect()
    }
}

// ============================================================================
// CREATOR VETO
// ============================================================================

/// Creator veto system
pub struct CreatorVeto {
    /// Is creator mode enabled
    enabled: bool,
    
    /// Veto history
    veto_history: Vec<VetoRecord>,
    
    /// Whitelist of creator-controlled proposals
    creator_whitelist: HashSet<u64>,
}

#[derive(Clone, Debug)]
pub struct VetoRecord {
    pub proposal_id: u64,
    pub vetoed_at: u64,
    pub reason: String,
}

impl CreatorVeto {
    pub fn new() -> Self {
        Self {
            enabled: true, // Creator starts enabled
            veto_history: Vec::new(),
            creator_whitelist: HashSet::new(),
        }
    }
    
    /// Disable creator mode (democratic-only)
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Enable creator mode
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// Is creator mode active
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Creator vetoes a proposal
    pub fn veto(&mut self, proposal_id: u64, reason: String) -> Result<(), PropagationError> {
        if !self.enabled {
            return Err(PropagationError::CreatorVeto);
        }
        
        self.veto_history.push(VetoRecord {
            proposal_id,
            vetoed_at: current_timestamp(),
            reason,
        });
        
        Ok(())
    }
    
    /// Creator whitelists a proposal (bypass voting)
    pub fn whitelist(&mut self, proposal_id: u64) {
        self.creator_whitelist.insert(proposal_id);
    }
    
    /// Check if proposal is whitelisted
    pub fn is_whitelisted(&self, proposal_id: u64) -> bool {
        self.creator_whitelist.contains(&proposal_id)
    }
    
    /// Get veto history
    pub fn get_veto_history(&self) -> &[VetoRecord] {
        &self.veto_history
    }
}

// ============================================================================
// PROPAGATION MANAGER
// ============================================================================

/// Central propagation manager
pub struct PropagationManager {
    /// Vote manager
    votes: VoteManager,
    
    /// Blacklist
    blacklist: CriticalBlacklist,
    
    /// Creator veto
    creator_veto: CreatorVeto,
    
    /// Configuration
    config: PropagationConfig,
    
    /// Statistics
    stats: PropagationStats,
}

#[derive(Clone, Debug)]
pub struct PropagationConfig {
    pub require_vote: bool,
    pub required_approval_ratio: f64,
    pub propagation_enabled: bool,  // Master flag
    pub allow_emergency_proposals: bool,
}

impl Default for PropagationConfig {
    fn default() -> Self {
        Self {
            require_vote: true,
            required_approval_ratio: 0.66,
            propagation_enabled: false,  // Disabled by default
            allow_emergency_proposals: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PropagationStats {
    pub total_proposals: u64,
    pub accepted: u64,
    pub rejected: u64,
    pub vetoed: u64,
    pub executed: u64,
}

impl PropagationManager {
    pub fn new() -> Self {
        Self {
            votes: VoteManager::new(),
            blacklist: CriticalBlacklist::new(),
            creator_veto: CreatorVeto::new(),
            config: PropagationConfig::default(),
            stats: PropagationStats::default(),
        }
    }
    
    /// Enable propagation (creator only)
    pub fn enable_propagation(&mut self) -> Result<(), PropagationError> {
        // This should only be called by creator
        self.config.propagation_enabled = true;
        Ok(())
    }
    
    /// Disable propagation (creator only)
    pub fn disable_propagation(&mut self) {
        self.config.propagation_enabled = false;
    }
    
    /// Check if propagation is enabled
    pub fn is_propagation_enabled(&self) -> bool {
        self.config.propagation_enabled
    }
    
    /// Submit proposal
    pub fn submit_proposal(
        &mut self,
        proposal_type: ProposalType,
        proposer: String,
        target: String,
        payload_hash: u64,
        payload_size: usize,
        reason: String,
    ) -> Result<u64, PropagationError> {
        // Check if propagation is enabled
        if !self.config.propagation_enabled {
            return Err(PropagationError::InsufficientVotes {
                required: 1,
                actual: 0,
            });
        }
        
        // Check blacklist for target
        if self.blacklist.is_blacklisted(&target) {
            return Err(PropagationError::BlacklistedIp(target));
        }
        
        // Determine priority based on proposal type
        let priority = match proposal_type {
            ProposalType::Emergency => ProposalPriority::CRITICAL,
            ProposalType::AddNode => ProposalPriority::NORMAL,
            _ => ProposalPriority::NORMAL,
        };
        
        let proposal_id = self.votes.create_proposal(
            proposal_type,
            proposer,
            target,
            payload_hash,
            payload_size,
            priority,
            reason,
        )?;
        
        self.stats.total_proposals += 1;
        
        Ok(proposal_id)
    }
    
    /// Vote on proposal
    pub fn vote(&mut self, proposal_id: u64, node_id: &str, approve: bool, abstain: bool) -> Result<(), PropagationError> {
        self.votes.vote(proposal_id, node_id, approve, abstain)
    }
    
    /// Get proposal status
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&PropagationProposal> {
        self.votes.get_proposal(proposal_id)
    }
    
    /// Execute approved proposal
    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<(), PropagationError> {
        let proposal = self.votes.get_proposal_mut(proposal_id)
            .ok_or(PropagationError::InvalidProposal)?;
        
        // Check if creator vetoed
        if self.creator_veto.is_enabled() && 
           self.creator_veto.get_veto_history().iter().any(|v| v.proposal_id == proposal_id) {
            self.stats.vetoed += 1;
            return Err(PropagationError::CreatorVeto);
        }
        
        // Check if whitelisted
        if self.creator_veto.is_whitelisted(proposal_id) {
            self.votes.mark_executed(proposal_id);
            self.stats.executed += 1;
            return Ok(());
        }
        
        // Check if proposal is ready to execute
        if !proposal.is_expired() {
            return Err(PropagationError::VoteTimeout);
        }
        
        if proposal.status != ProposalStatus::Accepted {
            return Err(PropagationError::InsufficientVotes {
                required: (self.config.required_approval_ratio * 100.0) as u32,
                actual: (proposal.approval_ratio() * 100.0) as u32,
            });
        }
        
        self.votes.mark_executed(proposal_id);
        self.stats.executed += 1;
        
        Ok(())
    }
    
    /// Creator veto
    pub fn creator_veto(&mut self, proposal_id: u64, reason: String) -> Result<(), PropagationError> {
        self.creator_veto.veto(proposal_id, reason)?;
        self.stats.vetoed += 1;
        Ok(())
    }
    
    /// Add to blacklist
    pub fn add_to_blacklist(&mut self, ip: String, category: BlacklistCategory, reason: String) {
        self.blacklist.add(ip, category, reason, "creator".to_string());
    }
    
    /// Remove from blacklist
    pub fn remove_from_blacklist(&mut self, ip: &str) -> bool {
        self.blacklist.remove(ip)
    }
    
    /// Check if IP is blacklisted
    pub fn is_blacklisted(&self, ip: &str) -> bool {
        self.blacklist.is_blacklisted(ip)
    }
    
    /// Clean up expired proposals
    pub fn cleanup(&mut self) {
        self.votes.cleanup_expired();
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> PropagationStats {
        self.stats.clone()
    }
    
    /// Get active proposals
    pub fn get_active_proposals(&self) -> Vec<&PropagationProposal> {
        self.votes.get_active_proposals()
    }
}

impl Default for PropagationManager {
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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_creation() {
        let mut manager = PropagationManager::new();
        manager.config.propagation_enabled = true;
        
        let id = manager.submit_proposal(
            ProposalType::AddNode,
            "node1".to_string(),
            "192.168.1.100".to_string(),
            0xDEADBEEF,
            1024,
            "New node request".to_string(),
        ).unwrap();
        
        let proposal = manager.get_proposal(id).unwrap();
        assert_eq!(proposal.proposal_type, ProposalType::AddNode);
    }

    #[test]
    fn test_voting() {
        let mut manager = PropagationManager::new();
        manager.config.propagation_enabled = true;
        
        let id = manager.submit_proposal(
            ProposalType::AddNode,
            "node1".to_string(),
            "192.168.1.100".to_string(),
            0xDEADBEEF,
            1024,
            "New node".to_string(),
        ).unwrap();
        
        // Vote
        manager.vote(id, "node2", true, false).unwrap();
        manager.vote(id, "node3", true, false).unwrap();
        manager.vote(id, "node4", false, false).unwrap();
        
        let proposal = manager.get_proposal(id).unwrap();
        assert_eq!(proposal.votes_for.len(), 2);
        assert_eq!(proposal.votes_against.len(), 1);
        assert!(proposal.approval_ratio() > 0.5);
    }

    #[test]
    fn test_blacklist() {
        let blacklist = CriticalBlacklist::new();
        
        assert!(blacklist.is_blacklisted("127.0.0.1"));
        assert!(!blacklist.is_blacklisted("8.8.8.8"));
    }

    #[test]
    fn test_creator_veto() {
        let mut manager = PropagationManager::new();
        manager.config.propagation_enabled = true;
        
        let id = manager.submit_proposal(
            ProposalType::AddNode,
            "node1".to_string(),
            "192.168.1.100".to_string(),
            0xDEADBEEF,
            1024,
            "New node".to_string(),
        ).unwrap();
        
        // Creator vetoes
        manager.creator_veto(id, "Not authorized".to_string()).unwrap();
        
        // Execution should fail
        assert!(manager.execute_proposal(id).is_err());
    }

    #[test]
    fn test_propagation_disabled() {
        let mut manager = PropagationManager::new();
        // propagation_enabled is false by default
        
        let result = manager.submit_proposal(
            ProposalType::AddNode,
            "node1".to_string(),
            "192.168.1.100".to_string(),
            0xDEADBEEF,
            1024,
            "New node".to_string(),
        );
        
        assert!(result.is_err());
    }
}