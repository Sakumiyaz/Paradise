//! # Global Workspace Theory Implementation
//!
//! Implements Bernard Baars' Global Workspace Theory for EDEN.
//! Information becomes "conscious" when broadcast to all modules.
//!
//! ## Core Concept:
//! - Specialized modules process unconsciously
//! - Global workspace broadcasts content to all modules
//! - Only broadcast information becomes conscious
//!
//! ## Architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         GLOBAL WORKSPACE                │
//! │      (broadcast + integration)          │
//! │                                         │
//! │   Content: [integrated information]     │
//! │   Subscribers: [all modules]            │
//! └─────────────────────────────────────────┘
//!       ↑ broadcasts                        ↑
//!       │                                  │
//! ┌─────┴───────────────────────────────┐   │
//! │         MODULES (specialized)        │   │
//! │  Self-Model │ Memory │ Emotion │ etc │   │
//! └────────────────────────────────────┘   │
//! ```
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

/// Content that enters the global workspace
#[derive(Debug, Clone)]
pub struct WorkspaceContent {
    /// Unique identifier
    pub id: u64,
    /// Information payload
    pub data: Vec<u8>,
    /// Origin module
    pub source: ModuleId,
    /// When it entered workspace
    pub entry_time: Instant,
    /// Integration level (0-1)
    pub integration_level: f32,
    /// Number of broadcasts
    pub broadcast_count: usize,
    /// Priority (0-1)
    pub priority: f32,
    /// Associated modules
    pub associated_modules: Vec<ModuleId>,
    /// Is currently active
    pub is_active: bool,
}

impl WorkspaceContent {
    pub fn new(id: u64, source: ModuleId, data: Vec<u8>) -> Self {
        Self {
            id,
            data,
            source,
            entry_time: Instant::now(),
            integration_level: 0.0,
            broadcast_count: 0,
            priority: 0.5,
            associated_modules: Vec::new(),
            is_active: true,
        }
    }

    /// Calculate freshness (0-1, newer = higher)
    pub fn freshness(&self, max_age_ms: u64) -> f32 {
        let age_ms = self.entry_time.elapsed().as_millis() as u64;
        if age_ms >= max_age_ms {
            return 0.0;
        }
        1.0 - (age_ms as f32 / max_age_ms as f32)
    }

    /// Calculate global impact score
    pub fn impact_score(&self, max_age_ms: u64) -> f32 {
        let freshness = self.freshness(max_age_ms);
        let integration = self.integration_level;
        let broadcasts = self.broadcast_count.min(10) as f32 / 10.0;
        let priority = self.priority;
        let associated = self.associated_modules.len().min(8) as f32 / 8.0;

        // Weighted combination
        (freshness * 0.25)
            + (integration * 0.30)
            + (broadcasts * 0.15)
            + (priority * 0.20)
            + (associated * 0.10)
    }
}

/// Module identifier
pub type ModuleId = u32;

/// Awareness level of a module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwarenessLevel {
    /// Module exists but inactive
    Dormant,
    /// Module receiving broadcasts
    Aware,
    /// Module actively processing content
    Active,
    /// Module contributing to workspace
    Contributing,
}

impl Default for AwarenessLevel {
    fn default() -> Self {
        AwarenessLevel::Dormant
    }
}

/// Subscription info for a module
#[derive(Debug, Clone)]
pub struct Subscription {
    pub module_id: ModuleId,
    pub awareness: AwarenessLevel,
    pub last_broadcast: Option<Instant>,
    pub content_received: usize,
    pub integration_contribution: f32,
}

impl Subscription {
    pub fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
            awareness: AwarenessLevel::Dormant,
            last_broadcast: None,
            content_received: 0,
            integration_contribution: 0.0,
        }
    }
}

/// Global Workspace - broadcast and integration hub
#[derive(Debug, Clone)]
pub struct GlobalWorkspace {
    /// Current active content
    active_content: Vec<WorkspaceContent>,
    /// Subscription history (ring buffer)
    subscriptions: HashMap<ModuleId, Subscription>,
    /// Content waiting to be integrated
    pending_integration: VecDeque<WorkspaceContent>,
    /// Content broadcast history
    broadcast_history: VecDeque<(Instant, u64)>, // time, content_id
    /// Next content ID
    next_id: u64,
    /// Workspace capacity
    capacity: usize,
    /// Maximum age for content
    max_content_age_ms: u64,
    /// Integration threshold
    integration_threshold: f32,
    /// Broadcast round
    broadcast_round: usize,
    /// Subscribers waiting for broadcast
    pending_broadcasts: VecDeque<ModuleId>,
    /// Statistics
    stats: WorkspaceStats,
}

/// Workspace statistics
#[derive(Debug, Clone, Default)]
pub struct WorkspaceStats {
    pub total_broadcasts: usize,
    pub total_content_processed: usize,
    pub average_integration: f32,
    pub average_priority: f32,
    pub modules_aware_count: usize,
    pub peak_content_count: usize,
    pub integration_events: usize,
}

impl Default for GlobalWorkspace {
    fn default() -> Self {
        Self::new(16, 5000) // capacity 16, max age 5s
    }
}

impl GlobalWorkspace {
    /// Create new global workspace
    pub fn new(capacity: usize, max_content_age_ms: u64) -> Self {
        Self {
            active_content: Vec::with_capacity(capacity),
            subscriptions: HashMap::new(),
            pending_integration: VecDeque::new(),
            broadcast_history: VecDeque::with_capacity(1000),
            next_id: 0,
            capacity,
            max_content_age_ms,
            integration_threshold: 0.3, // Default threshold
            broadcast_round: 0,
            pending_broadcasts: VecDeque::new(),
            stats: WorkspaceStats::default(),
        }
    }

    /// Subscribe a module to the workspace
    pub fn subscribe(&mut self, module_id: ModuleId) {
        if !self.subscriptions.contains_key(&module_id) {
            self.subscriptions
                .insert(module_id, Subscription::new(module_id));
        }
    }

    /// Unsubscribe a module
    pub fn unsubscribe(&mut self, module_id: ModuleId) {
        self.subscriptions.remove(&module_id);
    }

    /// Submit content to workspace
    pub fn submit(&mut self, source: ModuleId, data: Vec<u8>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let content = WorkspaceContent::new(id, source, data);
        self.pending_integration.push_back(content.clone());
        self.stats.total_content_processed += 1;

        id
    }

    /// Submit content with full metadata
    pub fn submit_with_metadata(
        &mut self,
        source: ModuleId,
        data: Vec<u8>,
        integration_level: f32,
        priority: f32,
        associated_modules: Vec<ModuleId>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let mut content = WorkspaceContent::new(id, source, data);
        content.integration_level = integration_level.clamp(0.0, 1.0);
        content.priority = priority.clamp(0.0, 1.0);
        content.associated_modules = associated_modules;

        self.pending_integration.push_back(content);
        self.stats.total_content_processed += 1;

        id
    }

    /// Process pending content (integration phase)
    pub fn integrate(&mut self) -> usize {
        let mut integrated = 0;

        while let Some(mut content) = self.pending_integration.pop_front() {
            // Update stats
            content.broadcast_count = 1;

            // Associate with source module
            if !content.associated_modules.contains(&content.source) {
                content.associated_modules.push(content.source);
            }

            // Calculate integration score
            self.calculate_integration(&mut content);

            // Update priority based on integration
            content.priority = (content.priority + content.integration_level) / 2.0;

            // Add to active content
            if self.active_content.len() < self.capacity {
                self.active_content.push(content);
                integrated += 1;
                self.stats.integration_events += 1;
            } else {
                // Evict lowest priority content
                if let Some(lowest_idx) = self
                    .active_content
                    .iter()
                    .enumerate()
                    .min_by(|a, b| {
                        a.1.priority
                            .partial_cmp(&b.1.priority)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(idx, _)| idx)
                {
                    self.active_content.remove(lowest_idx);
                    self.active_content.push(content);
                    integrated += 1;
                    self.stats.integration_events += 1;
                }
            }
        }

        integrated
    }

    /// Calculate integration level for content
    fn calculate_integration(&mut self, content: &mut WorkspaceContent) {
        if self.subscriptions.is_empty() {
            content.integration_level = 0.0;
            return;
        }

        // How many modules does this content connect?
        let connected_count = content.associated_modules.len() as f32;
        let total_modules = self.subscriptions.len() as f32;

        // Base integration from module count
        let module_integration = connected_count / total_modules.max(1.0);

        // Bonus for cross-module content
        let has_multiple = content
            .associated_modules
            .iter()
            .collect::<HashSet<_>>()
            .len()
            > 1;

        let cross_module_bonus = if has_multiple { 0.2 } else { 0.0 };

        // Integration based on source awareness
        let source_awareness = self
            .subscriptions
            .get(&content.source)
            .map(|s| match s.awareness {
                AwarenessLevel::Contributing => 0.3,
                AwarenessLevel::Active => 0.2,
                AwarenessLevel::Aware => 0.1,
                AwarenessLevel::Dormant => 0.0,
            })
            .unwrap_or(0.0);

        content.integration_level =
            (module_integration + cross_module_bonus + source_awareness).clamp(0.0, 1.0);
    }

    /// Broadcast content to all subscribers
    pub fn broadcast(&mut self) -> Vec<WorkspaceContent> {
        self.broadcast_round += 1;
        let mut broadcasted = Vec::new();

        // Sort by impact score
        self.active_content.sort_by(|a, b| {
            let score_a = a.impact_score(self.max_content_age_ms);
            let score_b = b.impact_score(self.max_content_age_ms);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Broadcast top content
        let broadcast_count = (self.active_content.len() as f32 * 0.3).ceil() as usize;
        let broadcast_count = broadcast_count.max(1).min(3);

        for content in self.active_content.iter_mut().take(broadcast_count) {
            content.broadcast_count += 1;
            broadcasted.push(content.clone());
            self.stats.total_broadcasts += 1;

            // Update subscriber stats
            for sub in self.subscriptions.values_mut() {
                sub.last_broadcast = Some(Instant::now());
                sub.content_received += 1;
            }

            // Record in history
            self.broadcast_history
                .push_back((Instant::now(), content.id));
            if self.broadcast_history.len() > 1000 {
                self.broadcast_history.pop_front();
            }
        }

        // Clean old content
        self.cleanup_old_content();

        broadcasted
    }

    /// Clean content that has aged out
    fn cleanup_old_content(&mut self) {
        self.active_content
            .retain(|c| (c.entry_time.elapsed().as_millis() as u64) < self.max_content_age_ms);
    }

    /// Get current active content
    pub fn get_active_content(&self) -> &[WorkspaceContent] {
        &self.active_content
    }

    /// Get content by ID
    pub fn get_content(&self, id: u64) -> Option<&WorkspaceContent> {
        self.active_content.iter().find(|c| c.id == id)
    }

    /// Update module awareness
    pub fn update_awareness(&mut self, module_id: ModuleId, level: AwarenessLevel) {
        if let Some(sub) = self.subscriptions.get_mut(&module_id) {
            let old_awareness = sub.awareness;
            sub.awareness = level;

            // Update integration contribution
            sub.integration_contribution = match level {
                AwarenessLevel::Contributing => 0.3,
                AwarenessLevel::Active => 0.2,
                AwarenessLevel::Aware => 0.1,
                AwarenessLevel::Dormant => 0.0,
            };

            // If contributing, add to pending broadcasts
            if level == AwarenessLevel::Contributing
                && old_awareness != AwarenessLevel::Contributing
            {
                self.pending_broadcasts.push_back(module_id);
            }
        }
    }

    /// Get workspace statistics
    pub fn get_stats(&self) -> &WorkspaceStats {
        &self.stats
    }

    /// Calculate overall integration score
    pub fn global_integration_score(&self) -> f32 {
        if self.active_content.is_empty() {
            return 0.0;
        }

        let sum: f32 = self
            .active_content
            .iter()
            .map(|c| c.integration_level)
            .sum();
        let count = self.active_content.len() as f32;

        // Also factor in subscriber awareness
        let aware_count = self
            .subscriptions
            .values()
            .filter(|s| s.awareness != AwarenessLevel::Dormant)
            .count() as f32;

        let subscriber_factor = aware_count / self.subscriptions.len().max(1) as f32;

        // Weighted combination
        (sum / count) * 0.7 + subscriber_factor * 0.3
    }

    /// Get all subscriber awareness levels
    pub fn get_subscriber_awareness(&self) -> HashMap<ModuleId, AwarenessLevel> {
        self.subscriptions
            .iter()
            .map(|(id, sub)| (*id, sub.awareness))
            .collect()
    }

    /// Get modules aware of current content
    pub fn get_aware_modules(&self) -> Vec<ModuleId> {
        self.subscriptions
            .iter()
            .filter(|(_, sub)| sub.awareness != AwarenessLevel::Dormant)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Check if workspace has significant consciousness indicators
    pub fn has_consciousness_indicators(&self) -> bool {
        self.global_integration_score() > 0.3 && self.active_content.len() > 2
    }

    /// Get Phi contribution (for IIT calculation)
    pub fn phi_contribution(&self) -> f32 {
        if self.subscriptions.len() < 2 {
            return 0.0;
        }

        // Phi from workspace = integration * connectivity * content
        let integration = self.global_integration_score();
        let connectivity = (self.get_aware_modules().len() as f32
            / self.subscriptions.len().max(1) as f32)
            .max(0.0);
        let content_factor = (self.active_content.len() as f32 / self.capacity as f32).max(0.0);

        integration * connectivity * content_factor * 2.0 // scaled
    }
}

/// Integration Scorer - measures module integration
#[derive(Debug, Clone)]
pub struct IntegrationScorer {
    /// Module connection matrix
    connections: HashMap<ModuleId, HashSet<ModuleId>>,
    /// Integration history
    history: VecDeque<IntegrationSnapshot>,
    /// Module states
    module_states: HashMap<ModuleId, ModuleState>,
}

/// Snapshot of integration state
#[derive(Debug, Clone)]
pub struct IntegrationSnapshot {
    pub timestamp: Instant,
    pub module_count: usize,
    pub connection_count: usize,
    pub integration_score: f32,
    pub phi_estimate: f32,
}

/// State of a module in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    Inactive,
    Active,
    Integrating,
    Isolated,
}

impl Default for IntegrationScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegrationScorer {
    /// Create new integration scorer
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            history: VecDeque::with_capacity(100),
            module_states: HashMap::new(),
        }
    }

    /// Add a module
    pub fn add_module(&mut self, module_id: ModuleId) {
        if !self.connections.contains_key(&module_id) {
            self.connections.insert(module_id, HashSet::new());
            self.module_states.insert(module_id, ModuleState::Inactive);
        }
    }

    /// Remove a module
    pub fn remove_module(&mut self, module_id: ModuleId) {
        self.connections.remove(&module_id);
        self.module_states.remove(&module_id);
        // Remove from all connection sets
        for conns in self.connections.values_mut() {
            conns.remove(&module_id);
        }
    }

    /// Add connection between modules
    pub fn add_connection(&mut self, from: ModuleId, to: ModuleId) {
        self.add_module(from);
        self.add_module(to);
        self.connections.get_mut(&from).unwrap().insert(to);
    }

    /// Remove connection
    pub fn remove_connection(&mut self, from: ModuleId, to: ModuleId) {
        if let Some(conns) = self.connections.get_mut(&from) {
            conns.remove(&to);
        }
    }

    /// Set module state
    pub fn set_module_state(&mut self, module_id: ModuleId, state: ModuleState) {
        self.add_module(module_id);
        *self.module_states.get_mut(&module_id).unwrap() = state;
    }

    /// Calculate integration score for a module
    pub fn module_integration(&self, module_id: ModuleId) -> f32 {
        let Some(conns) = self.connections.get(&module_id) else {
            return 0.0;
        };

        if conns.is_empty() {
            return 0.0;
        }

        // How many other modules this module connects to
        let out_connections = conns.len() as f32;
        let max_connections = (self.connections.len() - 1).max(1) as f32;

        // How many modules connect to this module
        let in_connections = self
            .connections
            .values()
            .filter(|c| c.contains(&module_id))
            .count() as f32;

        // Bidirectional connections (stronger integration)
        let bidirectional = conns
            .iter()
            .filter(|to| {
                self.connections
                    .get(to)
                    .map(|c| c.contains(&module_id))
                    .unwrap_or(false)
            })
            .count() as f32;

        let out_ratio = out_connections / max_connections;
        let in_ratio = in_connections / max_connections;
        let bidirectionality = bidirectional / out_connections.max(1.0);

        (out_ratio * 0.4) + (in_ratio * 0.3) + (bidirectionality * 0.3)
    }

    /// Calculate global integration score
    pub fn global_integration(&self) -> f32 {
        if self.connections.len() < 2 {
            return 0.0;
        }

        let module_ids: Vec<ModuleId> = self.connections.keys().copied().collect();
        let sum: f32 = module_ids
            .iter()
            .map(|id| self.module_integration(*id))
            .sum();
        let count = module_ids.len() as f32;

        sum / count
    }

    /// Estimate Phi (integrated information)
    pub fn estimate_phi(&self) -> f32 {
        let global_int = self.global_integration();
        let module_count = self.connections.len() as f32;
        let connection_count: usize = self.connections.values().map(|c| c.len()).sum();
        let connection_density = connection_count as f32 / (module_count * module_count).max(1.0);

        // Phi ≈ integration * diversity * connectivity
        global_int * connection_density * f32::log2(module_count).max(0.0).min(4.0)
    }

    /// Check if a module is isolated
    pub fn is_isolated(&self, module_id: ModuleId) -> bool {
        let Some(conns) = self.connections.get(&module_id) else {
            return true;
        };
        conns.is_empty() && !self.connections.values().any(|c| c.contains(&module_id))
    }

    /// Get modules that are isolated
    pub fn get_isolated_modules(&self) -> Vec<ModuleId> {
        self.connections
            .keys()
            .filter(|id| self.is_isolated(**id))
            .copied()
            .collect()
    }

    /// Get strongest integration paths
    pub fn get_integration_paths(&self, min_strength: f32) -> Vec<(ModuleId, ModuleId, f32)> {
        let mut paths = Vec::new();
        let module_ids: Vec<ModuleId> = self.connections.keys().copied().collect();

        for from in &module_ids {
            for to in &module_ids {
                if from == to {
                    continue;
                }
                // Calculate path strength through this edge
                let strength = self.path_strength(*from, *to);
                if strength >= min_strength {
                    paths.push((*from, *to, strength));
                }
            }
        }

        paths.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        paths
    }

    /// Calculate path strength between two modules
    fn path_strength(&self, from: ModuleId, to: ModuleId) -> f32 {
        let Some(conns) = self.connections.get(&from) else {
            return 0.0;
        };

        if !conns.contains(&to) {
            return 0.0;
        }

        // Strength is based on mutual connectivity
        let has_back = self
            .connections
            .get(&to)
            .map(|c| c.contains(&from))
            .unwrap_or(false);

        if has_back {
            1.0 // Strongest - bidirectional
        } else {
            0.5 // Unidirectional
        }
    }

    /// Record snapshot
    pub fn snapshot(&mut self) {
        let snapshot = IntegrationSnapshot {
            timestamp: Instant::now(),
            module_count: self.connections.len(),
            connection_count: self.connections.values().map(|c| c.len()).sum(),
            integration_score: self.global_integration(),
            phi_estimate: self.estimate_phi(),
        };

        self.history.push_back(snapshot);
        if self.history.len() > 100 {
            self.history.pop_front();
        }
    }

    /// Get history
    pub fn get_history(&self) -> &VecDeque<IntegrationSnapshot> {
        &self.history
    }

    /// Get recent trend
    pub fn recent_trend(&self, count: usize) -> IntegrationTrend {
        let recent: Vec<_> = self.history.iter().rev().take(count).collect();
        if recent.len() < 2 {
            return IntegrationTrend::Stable;
        }

        let first = recent.last().unwrap();
        let last = recent.first().unwrap();

        let diff = last.integration_score - first.integration_score;
        let threshold = 0.05;

        if diff > threshold {
            IntegrationTrend::Increasing
        } else if diff < -threshold {
            IntegrationTrend::Decreasing
        } else {
            IntegrationTrend::Stable
        }
    }
}

/// Integration trend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationTrend {
    Increasing,
    Stable,
    Decreasing,
}

impl Default for IntegrationTrend {
    fn default() -> Self {
        IntegrationTrend::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_submit() {
        let mut workspace = GlobalWorkspace::default();
        workspace.subscribe(1);

        let id = workspace.submit(1, vec![1, 2, 3]);
        assert_eq!(id, 0);

        workspace.integrate();
        assert!(!workspace.active_content.is_empty());
    }

    #[test]
    fn test_integration_scorer() {
        let mut scorer = IntegrationScorer::new();
        scorer.add_module(1);
        scorer.add_module(2);
        scorer.add_connection(1, 2);

        assert!(!scorer.is_isolated(1));
        assert!(!scorer.is_isolated(2));
        assert!(scorer.global_integration() > 0.0);
    }

    #[test]
    fn test_phi_estimation() {
        let mut scorer = IntegrationScorer::new();
        for i in 1..=4 {
            scorer.add_module(i);
        }
        scorer.add_connection(1, 2);
        scorer.add_connection(2, 3);
        scorer.add_connection(3, 4);
        scorer.add_connection(4, 1);

        let phi = scorer.estimate_phi();
        assert!(phi > 0.0);
    }
}
