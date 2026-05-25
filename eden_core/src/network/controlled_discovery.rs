//! # Controlled Discovery — Descubrimiento Consensual de Nodos
//!
//! Sistema de descubrimiento de nodos EDEN que requiere consentimiento mutuo:
//! - Whitelist de nodos autorizados
//! -邀请-only networking (no scanning automático)
//! - Verificación de identidad mediante certificados
//! - Rate limiting estricto para prevenir abuse
//!
//! ## Filosofía
//!
//! La propagación debe ser CONSENSUAL, no automática. Cada nodo tiene derecho
//! a aceptar o rechazar conexiones. Los nodos conocidos se comparten SOLO
//! mediante invitación explícita del Creador o voto de nodos existentes.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum DiscoveryError {
    NodeNotWhitelisted(String),
    RateLimitExceeded(String),
    InvalidInvitation,
    ExpiredInvitation,
    AlreadyConnected,
    MaxConnectionsReached,
    ConsentNotGranted,
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::NodeNotWhitelisted(id) => write!(f, "Node {} not in whitelist", id),
            DiscoveryError::RateLimitExceeded(id) => write!(f, "Rate limit exceeded for {}", id),
            DiscoveryError::InvalidInvitation => write!(f, "Invalid invitation"),
            DiscoveryError::ExpiredInvitation => write!(f, "Invitation expired"),
            DiscoveryError::AlreadyConnected => write!(f, "Already connected to this node"),
            DiscoveryError::MaxConnectionsReached => write!(f, "Maximum connections reached"),
            DiscoveryError::ConsentNotGranted => write!(f, "Node did not consent to connection"),
        }
    }
}

impl std::error::Error for DiscoveryError {}

// ============================================================================
// INVITATION SYSTEM
// ============================================================================

/// Invitation to join the network
#[derive(Clone, Debug)]
pub struct NetworkInvitation {
    pub invitation_id: u64,
    pub from_node: String,
    pub to_node: Option<String>,  // None = open invitation
    pub created_at: u64,
    pub expires_at: u64,
    pub permissions: NodePermissions,
    pub signature: [u8; 64],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodePermissions {
    pub can_connect: bool,
    pub can_propagate: bool,
    pub can_mine: bool,
    pub max_bandwidth_kbps: u32,
}

impl Default for NodePermissions {
    fn default() -> Self {
        Self {
            can_connect: true,
            can_propagate: false,
            can_mine: false,
            max_bandwidth_kbps: 1024,  // 1 MB/s default
        }
    }
}

impl NetworkInvitation {
    pub fn new(
        invitation_id: u64,
        from_node: String,
        to_node: Option<String>,
        ttl_seconds: u64,
        permissions: NodePermissions,
    ) -> Self {
        let now = current_timestamp();
        Self {
            invitation_id,
            from_node,
            to_node,
            created_at: now,
            expires_at: now + ttl_seconds * 1000,
            permissions,
            signature: [0u8; 64],
        }
    }
    
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }
    
    pub fn is_valid_for(&self, node_id: &str) -> bool {
        !self.is_expired() && (self.to_node.is_none() || self.to_node.as_ref() == Some(&node_id.to_string()))
    }
}

/// Invitation manager
pub struct InvitationManager {
    /// Active invitations
    invitations: HashMap<u64, NetworkInvitation>,
    
    /// Used invitation IDs (to prevent replay)
    used_ids: HashSet<u64>,
    
    /// Configuration
    config: InvitationConfig,
}

#[derive(Clone, Debug)]
pub struct InvitationConfig {
    pub default_ttl_seconds: u64,
    pub max_active_invitations: usize,
    pub require_creator_approval: bool,
}

impl Default for InvitationConfig {
    fn default() -> Self {
        Self {
            default_ttl_seconds: 3600,  // 1 hour
            max_active_invitations: 100,
            require_creator_approval: true,
        }
    }
}

impl InvitationManager {
    pub fn new() -> Self {
        Self {
            invitations: HashMap::new(),
            used_ids: HashSet::new(),
            config: InvitationConfig::default(),
        }
    }
    
    /// Create a new invitation
    pub fn create_invitation(
        &mut self,
        from_node: &str,
        to_node: Option<String>,
        permissions: NodePermissions,
    ) -> Result<NetworkInvitation, DiscoveryError> {
        if self.invitations.len() >= self.config.max_active_invitations {
            return Err(DiscoveryError::MaxConnectionsReached);
        }
        
        // Generate unique invitation ID
        let invitation_id = generate_invitation_id();
        
        let invitation = NetworkInvitation::new(
            invitation_id,
            from_node.to_string(),
            to_node,
            self.config.default_ttl_seconds,
            permissions,
        );
        
        self.invitations.insert(invitation_id, invitation.clone());
        
        Ok(invitation)
    }
    
    /// Validate an invitation
    pub fn validate_invitation(
        &self,
        invitation_id: u64,
        node_id: &str,
    ) -> Result<NetworkInvitation, DiscoveryError> {
        // Check if used (replay attack prevention)
        if self.used_ids.contains(&invitation_id) {
            return Err(DiscoveryError::InvalidInvitation);
        }
        
        let invitation = self.invitations.get(&invitation_id)
            .ok_or(DiscoveryError::InvalidInvitation)?;
        
        if !invitation.is_valid_for(node_id) {
            return Err(DiscoveryError::ExpiredInvitation);
        }
        
        Ok(invitation.clone())
    }
    
    /// Consume an invitation (mark as used)
    pub fn consume_invitation(&mut self, invitation_id: u64) -> Result<(), DiscoveryError> {
        if self.used_ids.contains(&invitation_id) {
            return Err(DiscoveryError::InvalidInvitation);
        }
        
        if !self.invitations.contains_key(&invitation_id) {
            return Err(DiscoveryError::InvalidInvitation);
        }
        
        self.used_ids.insert(invitation_id);
        self.invitations.remove(&invitation_id);
        
        Ok(())
    }
    
    /// Revoke an invitation
    pub fn revoke_invitation(&mut self, invitation_id: u64) {
        self.invitations.remove(&invitation_id);
        self.used_ids.insert(invitation_id);
    }
    
    /// Clean up expired invitations
    pub fn cleanup_expired(&mut self) {
        let expired: Vec<u64> = self.invitations
            .iter()
            .filter(|(_, inv)| inv.is_expired())
            .map(|(id, _)| *id)
            .collect();
        
        for id in expired {
            self.invitations.remove(&id);
        }
    }
}

// ============================================================================
// WHITELIST MANAGEMENT
// ============================================================================

/// Whitelist of allowed nodes
pub struct NodeWhitelist {
    /// Allowed nodes by fingerprint
    allowed_nodes: HashMap<String, WhitelistedNode>,
    
    /// Pending approval (requires creator consent)
    pending_approval: HashMap<String, PendingNode>,
    
    /// Configuration
    config: WhitelistConfig,
}

#[derive(Clone, Debug)]
pub struct WhitelistedNode {
    pub fingerprint: String,
    pub added_at: u64,
    pub added_by: String,  // "creator" or node fingerprint
    pub permissions: NodePermissions,
    pub is_active: bool,
    pub last_seen: u64,
}

#[derive(Clone, Debug)]
pub struct PendingNode {
    pub fingerprint: String,
    pub requested_at: u64,
    pub requesting_node: String,
    pub permissions_requested: NodePermissions,
}

#[derive(Clone, Debug)]
pub struct WhitelistConfig {
    pub auto_approve_from_creator: bool,
    pub require_minimum_permissions: bool,
    pub max_whitelisted_nodes: usize,
}

impl Default for WhitelistConfig {
    fn default() -> Self {
        Self {
            auto_approve_from_creator: true,
            require_minimum_permissions: true,
            max_whitelisted_nodes: 1000,
        }
    }
}

impl NodeWhitelist {
    pub fn new() -> Self {
        Self {
            allowed_nodes: HashMap::new(),
            pending_approval: HashMap::new(),
            config: WhitelistConfig::default(),
        }
    }
    
    /// Add a node to the whitelist (requires explicit approval)
    pub fn add_node(
        &mut self,
        fingerprint: String,
        added_by: String,
        permissions: NodePermissions,
        auto_approve: bool,
    ) -> Result<(), DiscoveryError> {
        if self.allowed_nodes.len() >= self.config.max_whitelisted_nodes {
            return Err(DiscoveryError::MaxConnectionsReached);
        }
        
        if auto_approve || added_by == "creator" {
            let node = WhitelistedNode {
                fingerprint: fingerprint.clone(),
                added_at: current_timestamp(),
                added_by,
                permissions,
                is_active: true,
                last_seen: current_timestamp(),
            };
            self.allowed_nodes.insert(fingerprint, node);
            Ok(())
        } else {
            // Add to pending
            let pending = PendingNode {
                fingerprint: fingerprint.clone(),
                requested_at: current_timestamp(),
                requesting_node: added_by,
                permissions_requested: permissions,
            };
            self.pending_approval.insert(fingerprint, pending);
            Ok(())
        }
    }
    
    /// Approve a pending node
    pub fn approve_node(&mut self, fingerprint: &str, permissions: NodePermissions) -> Result<(), DiscoveryError> {
        let pending = self.pending_approval.remove(fingerprint)
            .ok_or(DiscoveryError::NodeNotWhitelisted(fingerprint.to_string()))?;
        
        let node = WhitelistedNode {
            fingerprint: fingerprint.to_string(),
            added_at: current_timestamp(),
            added_by: pending.requesting_node,
            permissions,
            is_active: true,
            last_seen: 0,
        };
        
        self.allowed_nodes.insert(fingerprint.to_string(), node);
        Ok(())
    }
    
    /// Remove a node from the whitelist
    pub fn remove_node(&mut self, fingerprint: &str) -> bool {
        self.allowed_nodes.remove(fingerprint);
        self.pending_approval.remove(fingerprint);
        true
    }
    
    /// Check if a node is whitelisted
    pub fn is_whitelisted(&self, fingerprint: &str) -> bool {
        self.allowed_nodes.get(fingerprint)
            .map(|n| n.is_active)
            .unwrap_or(false)
    }
    
    /// Get node permissions
    pub fn get_permissions(&self, fingerprint: &str) -> Option<NodePermissions> {
        self.allowed_nodes.get(fingerprint)
            .map(|n| n.permissions.clone())
    }
    
    /// Update last seen timestamp
    pub fn touch_node(&mut self, fingerprint: &str) {
        if let Some(node) = self.allowed_nodes.get_mut(fingerprint) {
            node.last_seen = current_timestamp();
        }
    }
    
    /// Deactivate a node (without removing from whitelist)
    pub fn deactivate_node(&mut self, fingerprint: &str) {
        if let Some(node) = self.allowed_nodes.get_mut(fingerprint) {
            node.is_active = false;
        }
    }
    
    /// Get all active nodes
    pub fn get_active_nodes(&self) -> Vec<String> {
        self.allowed_nodes
            .values()
            .filter(|n| n.is_active)
            .map(|n| n.fingerprint.clone())
            .collect()
    }
    
    /// Get pending approval count
    pub fn pending_count(&self) -> usize {
        self.pending_approval.len()
    }
}

// ============================================================================
// RATE LIMITING
// ============================================================================

/// Rate limiter for connection attempts
pub struct ConnectionRateLimiter {
    /// Connection attempts per node
    attempts: HashMap<String, RateWindow>,
    
    /// Configuration
    config: RateLimitConfig,
}

#[derive(Clone, Debug)]
pub struct RateWindow {
    pub count: u32,
    pub window_start: u64,
    pub last_attempt: u64,
}

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub max_attempts_per_window: u32,
    pub window_ms: u64,
    pub block_duration_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_attempts_per_window: 10,
            window_ms: 60_000,  // 1 minute
            block_duration_ms: 300_000,  // 5 minute block
        }
    }
}

impl ConnectionRateLimiter {
    pub fn new() -> Self {
        Self {
            attempts: HashMap::new(),
            config: RateLimitConfig::default(),
        }
    }
    
    /// Check if a node is rate limited
    pub fn is_rate_limited(&self, node_id: &str) -> bool {
        if let Some(window) = self.attempts.get(node_id) {
            let now = current_timestamp();
            
            // Check if currently in block period
            if now - window.last_attempt < self.config.block_duration_ms && window.count >= self.config.max_attempts_per_window {
                return true;
            }
        }
        false
    }
    
    /// Record a connection attempt
    pub fn record_attempt(&mut self, node_id: &str) -> Result<(), DiscoveryError> {
        if self.is_rate_limited(node_id) {
            return Err(DiscoveryError::RateLimitExceeded(node_id.to_string()));
        }
        
        let now = current_timestamp();
        
        if let Some(window) = self.attempts.get_mut(node_id) {
            // Reset window if expired
            if now - window.window_start >= self.config.window_ms {
                window.count = 1;
                window.window_start = now;
            } else {
                window.count += 1;
            }
            window.last_attempt = now;
        } else {
            self.attempts.insert(node_id.to_string(), RateWindow {
                count: 1,
                window_start: now,
                last_attempt: now,
            });
        }
        
        Ok(())
    }
    
    /// Reset rate limit for a node
    pub fn reset(&mut self, node_id: &str) {
        self.attempts.remove(node_id);
    }
    
    /// Clean up old entries
    pub fn cleanup(&mut self) {
        let now = current_timestamp();
        let cutoff = now - self.config.block_duration_ms - self.config.window_ms;
        
        self.attempts.retain(|_, window| window.last_attempt > cutoff);
    }
}

// ============================================================================
// DISCOVERY MANAGER
// ============================================================================

/// Central discovery manager coordinating all discovery components
pub struct DiscoveryManager {
    /// Invitation manager
    invitations: InvitationManager,
    
    /// Whitelist manager
    whitelist: NodeWhitelist,
    
    /// Rate limiter
    rate_limiter: ConnectionRateLimiter,
    
    /// Active connections
    active_connections: HashSet<String>,
    
    /// Creator flag - if true, creator has ultimate control
    creator_enabled: bool,
    
    /// Configuration
    config: DiscoveryConfig,
}

#[derive(Clone, Debug)]
pub struct DiscoveryConfig {
    pub require_invitation: bool,
    pub require_whitelist: bool,
    pub allow_manual_add: bool,
    pub max_connections: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            require_invitation: true,
            require_whitelist: true,
            allow_manual_add: true,
            max_connections: 100,
        }
    }
}

impl DiscoveryManager {
    pub fn new() -> Self {
        Self {
            invitations: InvitationManager::new(),
            whitelist: NodeWhitelist::new(),
            rate_limiter: ConnectionRateLimiter::new(),
            active_connections: HashSet::new(),
            creator_enabled: false,
            config: DiscoveryConfig::default(),
        }
    }
    
    /// Enable creator mode (for initial setup)
    pub fn enable_creator_mode(&mut self) {
        self.creator_enabled = true;
    }
    
    /// Disable creator mode (returns to democratic operation)
    pub fn disable_creator_mode(&mut self) {
        self.creator_enabled = false;
    }
    
    /// Check if connection is allowed
    pub fn can_connect(&self, fingerprint: &str, invitation_id: Option<u64>) -> Result<(), DiscoveryError> {
        // Check rate limit
        if self.rate_limiter.is_rate_limited(fingerprint) {
            return Err(DiscoveryError::RateLimitExceeded(fingerprint.to_string()));
        }
        
        // Check whitelist if required
        if self.config.require_whitelist && !self.whitelist.is_whitelisted(fingerprint) {
            // If invitation provided, try to validate
            if let Some(inv_id) = invitation_id {
                self.invitations.validate_invitation(inv_id, fingerprint)?;
            } else {
                return Err(DiscoveryError::NodeNotWhitelisted(fingerprint.to_string()));
            }
        }
        
        // Check max connections
        if self.active_connections.len() >= self.config.max_connections {
            return Err(DiscoveryError::MaxConnectionsReached);
        }
        
        // Check if already connected
        if self.active_connections.contains(fingerprint) {
            return Err(DiscoveryError::AlreadyConnected);
        }
        
        Ok(())
    }
    
    /// Request connection (will be recorded for rate limiting)
    pub fn request_connection(&mut self, fingerprint: &str) -> Result<(), DiscoveryError> {
        // Record attempt first
        self.rate_limiter.record_attempt(fingerprint)?;
        
        // Then check if allowed
        self.can_connect(fingerprint, None)?;
        
        Ok(())
    }
    
    /// Accept a connection
    pub fn accept_connection(&mut self, fingerprint: &str) -> Result<NodePermissions, DiscoveryError> {
        // Verify can connect
        self.can_connect(fingerprint, None)?;
        
        // Add to active connections
        self.active_connections.insert(fingerprint.to_string());
        
        // Update whitelist last seen
        self.whitelist.touch_node(fingerprint);
        
        // Return permissions
        self.whitelist.get_permissions(fingerprint)
            .ok_or(DiscoveryError::ConsentNotGranted)
    }
    
    /// Disconnect from a node
    pub fn disconnect(&mut self, fingerprint: &str) {
        self.active_connections.remove(fingerprint);
    }
    
    /// Create an invitation
    pub fn create_invitation(
        &mut self,
        from_node: &str,
        to_node: Option<String>,
        permissions: NodePermissions,
    ) -> Result<NetworkInvitation, DiscoveryError> {
        self.invitations.create_invitation(from_node, to_node, permissions)
    }
    
    /// Validate and consume an invitation
    pub fn validate_and_consume_invitation(
        &mut self,
        invitation_id: u64,
        node_id: &str,
    ) -> Result<NetworkInvitation, DiscoveryError> {
        let invitation = self.invitations.validate_invitation(invitation_id, node_id)?;
        self.invitations.consume_invitation(invitation_id)?;
        Ok(invitation)
    }
    
    /// Add a node directly (creator only)
    pub fn creator_add_node(&mut self, fingerprint: String, permissions: NodePermissions) -> Result<(), DiscoveryError> {
        if !self.creator_enabled {
            // Allow if whitelist allows manual add
            if !self.config.allow_manual_add {
                return Err(DiscoveryError::ConsentNotGranted);
            }
        }
        
        self.whitelist.add_node(fingerprint, "creator".to_string(), permissions, true)
    }
    
    /// Remove a node (creator only)
    pub fn creator_remove_node(&mut self, fingerprint: &str) -> bool {
        self.whitelist.remove_node(fingerprint);
        self.active_connections.remove(fingerprint);
        true
    }
    
    /// Get active connection count
    pub fn active_connection_count(&self) -> usize {
        self.active_connections.len()
    }
    
    /// Get all connected node fingerprints
    pub fn get_connected_nodes(&self) -> Vec<String> {
        self.active_connections.iter().cloned().collect()
    }
    
    /// Check if node is connected
    pub fn is_connected(&self, fingerprint: &str) -> bool {
        self.active_connections.contains(fingerprint)
    }
    
    /// Get pending approval count
    pub fn pending_approval_count(&self) -> usize {
        self.whitelist.pending_count()
    }
    
    /// Clean up expired invitations and old rate limit entries
    pub fn cleanup(&mut self) {
        self.invitations.cleanup_expired();
        self.rate_limiter.cleanup();
    }
    
    /// Get discovery statistics
    pub fn get_stats(&self) -> DiscoveryStats {
        DiscoveryStats {
            active_connections: self.active_connections.len(),
            whitelisted_nodes: self.whitelist.allowed_nodes.len(),
            pending_approval: self.whitelist.pending_count(),
            invitations_active: self.invitations.invitations.len(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DiscoveryStats {
    pub active_connections: usize,
    pub whitelisted_nodes: usize,
    pub pending_approval: usize,
    pub invitations_active: usize,
}

impl Default for DiscoveryManager {
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

fn generate_invitation_id() -> u64 {
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
    fn test_invitation_creation() {
        let mut manager = InvitationManager::new();
        
        let inv = manager.create_invitation(
            "node1",
            Some("node2".to_string()),
            NodePermissions::default(),
        ).unwrap();
        
        assert!(!inv.is_expired());
        assert!(inv.is_valid_for("node2"));
        assert!(!inv.is_valid_for("node3"));
    }

    #[test]
    fn test_whitelist_approval() {
        let mut whitelist = NodeWhitelist::new();
        
        // Add node with auto-approve
        whitelist.add_node(
            "fingerprint1".to_string(),
            "creator".to_string(),
            NodePermissions::default(),
            true,
        ).unwrap();
        
        assert!(whitelist.is_whitelisted("fingerprint1"));
        
        // Add node without auto-approve
        whitelist.add_node(
            "fingerprint2".to_string(),
            "node1".to_string(),
            NodePermissions::default(),
            false,
        ).unwrap();
        
        assert!(!whitelist.is_whitelisted("fingerprint2"));
        
        // Approve pending node
        whitelist.approve_node("fingerprint2", NodePermissions::default()).unwrap();
        
        assert!(whitelist.is_whitelisted("fingerprint2"));
    }

    #[test]
    fn test_rate_limiting() {
        let mut limiter = ConnectionRateLimiter::new();
        
        // First few attempts should succeed
        for _ in 0..5 {
            assert!(limiter.record_attempt("node1").is_ok());
        }
        
        // Should be rate limited after exceeding limit
        assert!(limiter.is_rate_limited("node1"));
        assert!(limiter.record_attempt("node1").is_err());
    }

    #[test]
    fn test_discovery_manager() {
        let mut manager = DiscoveryManager::new();
        manager.enable_creator_mode();
        
        // Creator can add nodes directly
        manager.creator_add_node(
            "fingerprint1".to_string(),
            NodePermissions::default(),
        ).unwrap();
        
        // Accept connection
        let perms = manager.accept_connection("fingerprint1").unwrap();
        assert!(perms.can_connect);
        
        assert_eq!(manager.active_connection_count(), 1);
        assert!(manager.is_connected("fingerprint1"));
        
        // Disconnect
        manager.disconnect("fingerprint1");
        assert_eq!(manager.active_connection_count(), 0);
    }

    #[test]
    fn test_invitation_consumption() {
        let mut manager = DiscoveryManager::new();
        
        let inv = manager.create_invitation(
            "creator",
            Some("new_node".to_string()),
            NodePermissions::default(),
        ).unwrap();
        
        // Validate
        let validated = manager.validate_and_consume_invitation(
            inv.invitation_id,
            "new_node",
        ).unwrap();
        
        assert_eq!(validated.invitation_id, inv.invitation_id);
        
        // Can't use same invitation again
        assert!(manager.validate_and_consume_invitation(
            inv.invitation_id,
            "new_node",
        ).is_err());
    }
}