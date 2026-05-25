//! # Resource Monitor — Monitoreo Ético de Recursos
//!
//! Sistema de monitoreo de recursos para EDEN con límites éticos:
//! - CPU y memoria dentro de límites de las Leyes Inmutables
//! - Solo minar cuando hay recursos idle disponibles
//! - Consentimiento explícito del usuario para cualquier operación
//! - Auditoría completa de uso de recursos
//!
//! ## Filosofía
//!
//! El sistema NUNCA debe competir con otras aplicaciones por recursos.
//! Solo opera cuando hay capacidad libre verificada. El Creador tiene
//! control total y puede desactivar cualquier operación en cualquier momento.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum ResourceError {
    InsufficientResources,
    ConsentNotGranted,
    LawViolation(String),
    MonitoringFailed(String),
    ThresholdExceeded(String),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::InsufficientResources => write!(f, "Insufficient resources available"),
            ResourceError::ConsentNotGranted => write!(f, "User consent not granted"),
            ResourceError::LawViolation(s) => write!(f, "Law violation: {}", s),
            ResourceError::MonitoringFailed(s) => write!(f, "Monitoring failed: {}", s),
            ResourceError::ThresholdExceeded(s) => write!(f, "Threshold exceeded: {}", s),
        }
    }
}

impl std::error::Error for ResourceError {}

// ============================================================================
// RESOURCE METRICS
// ============================================================================

/// Current resource usage
#[derive(Clone, Debug)]
pub struct ResourceUsage {
    pub timestamp: u64,
    pub cpu_usage_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_recv: u64,
    pub process_count: u32,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            timestamp: current_timestamp(),
            cpu_usage_percent: 0.0,
            memory_used_bytes: 0,
            memory_total_bytes: 0,
            disk_used_bytes: 0,
            disk_total_bytes: 0,
            network_bytes_sent: 0,
            network_bytes_recv: 0,
            process_count: 0,
        }
    }
}

impl ResourceUsage {
    pub fn memory_usage_percent(&self) -> f32 {
        if self.memory_total_bytes == 0 { return 0.0; }
        (self.memory_used_bytes as f32 / self.memory_total_bytes as f32) * 100.0
    }
    
    pub fn disk_usage_percent(&self) -> f32 {
        if self.disk_total_bytes == 0 { return 0.0; }
        (self.disk_used_bytes as f32 / self.disk_total_bytes as f32) * 100.0
    }
    
    pub fn available_memory_bytes(&self) -> u64 {
        self.memory_total_bytes.saturating_sub(self.memory_used_bytes)
    }
    
    pub fn available_cpu_percent(&self) -> f32 {
        100.0 - self.cpu_usage_percent
    }
}

/// Thresholds for resource allocation
#[derive(Clone, Debug)]
pub struct ResourceThresholds {
    pub max_cpu_percent: f32,
    pub max_memory_percent: f32,
    pub max_disk_percent: f32,
    pub min_idle_cpu_percent: f32,    // Min CPU idle before mining
    pub min_idle_memory_mb: u64,       // Min free memory before mining
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        Self {
            max_cpu_percent: 90.0,      // Law: CPU ≤ 90%
            max_memory_percent: 85.0,
            max_disk_percent: 90.0,
            min_idle_cpu_percent: 30.0, // Only mine if 30%+ idle
            min_idle_memory_mb: 1024,   // Only mine if 1GB+ free
        }
    }
}

/// Resource budget for a task
#[derive(Clone, Debug)]
pub struct ResourceBudget {
    pub task_name: String,
    pub max_cpu_percent: f32,
    pub max_memory_bytes: u64,
    pub max_duration_ms: u64,
    pub priority: u8,
}

impl ResourceBudget {
    pub fn new(task_name: &str) -> Self {
        Self {
            task_name: task_name.to_string(),
            max_cpu_percent: 50.0,
            max_memory_bytes: 512 * 1024 * 1024, // 512 MB
            max_duration_ms: 60_000,
            priority: 5,
        }
    }
    
    pub fn with_cpu(mut self, percent: f32) -> Self {
        self.max_cpu_percent = percent;
        self
    }
    
    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = bytes;
        self
    }
    
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.max_duration_ms = ms;
        self
    }
    
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

// ============================================================================
// LAWS ENFORCEMENT
// ============================================================================

/// Law limits for resource usage
#[derive(Clone, Debug)]
pub struct LawLimits {
    pub cpu_max: f32,
    pub temp_max_celsius: f32,
    pub auton_max: u64,
}

impl Default for LawLimits {
    fn default() -> Self {
        Self {
            cpu_max: 90.0,
            temp_max_celsius: 75.0,
            auton_max: 50_000,
        }
    }
}

/// Law enforcement checker
pub struct LawEnforcer {
    limits: LawLimits,
    violations: Vec<LawViolation>,
}

#[derive(Clone, Debug)]
pub struct LawViolation {
    pub timestamp: u64,
    pub law_type: String,
    pub value: f32,
    pub limit: f32,
    pub severity: ViolationSeverity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViolationSeverity {
    Warning,
    Critical,
    Fatal,
}

impl LawEnforcer {
    pub fn new() -> Self {
        Self {
            limits: LawLimits::default(),
            violations: Vec::new(),
        }
    }
    
    pub fn with_limits(mut self, limits: LawLimits) -> Self {
        self.limits = limits;
        self
    }
    
    /// Check if operation would violate laws
    pub fn check(&self, usage: &ResourceUsage) -> Result<(), ResourceError> {
        // Check CPU
        if usage.cpu_usage_percent > self.limits.cpu_max {
            return Err(ResourceError::LawViolation(
                format!("CPU {}% exceeds limit {}%", usage.cpu_usage_percent, self.limits.cpu_max)
            ));
        }
        
        // Check memory
        let memory_percent = usage.memory_usage_percent();
        if memory_percent > self.limits.cpu_max { // Use same threshold for now
            return Err(ResourceError::LawViolation(
                format!("Memory {}% exceeds safe limit", memory_percent)
            ));
        }
        
        Ok(())
    }
    
    /// Record a violation
    pub fn record_violation(&mut self, violation: LawViolation) {
        self.violations.push(violation);
    }
    
    /// Get violations in time window
    pub fn recent_violations(&self, window_ms: u64) -> Vec<&LawViolation> {
        let cutoff = current_timestamp() - window_ms;
        self.violations.iter().filter(|v| v.timestamp > cutoff).collect()
    }
    
    /// Clear old violations
    pub fn cleanup(&mut self, max_age_ms: u64) {
        let cutoff = current_timestamp() - max_age_ms;
        self.violations.retain(|v| v.timestamp > cutoff);
    }
}

// ============================================================================
// CONSENT MANAGER
// ============================================================================

/// User consent for operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsentLevel {
    None,
    MonitoringOnly,
    LowImpact,
    FullAccess,
}

impl Default for ConsentLevel {
    fn default() -> Self {
        ConsentLevel::None
    }
}

/// Consent manager
pub struct ConsentManager {
    current_consent: ConsentLevel,
    consent_history: Vec<ConsentChange>,
    require_explicit_for_mining: bool,
}

#[derive(Clone, Debug)]
pub struct ConsentChange {
    pub timestamp: u64,
    pub from: ConsentLevel,
    pub to: ConsentLevel,
    pub reason: String,
}

impl ConsentManager {
    pub fn new() -> Self {
        Self {
            current_consent: ConsentLevel::None,
            consent_history: Vec::new(),
            require_explicit_for_mining: true,
        }
    }
    
    /// Set consent level (user action only)
    pub fn set_consent(&mut self, level: ConsentLevel, reason: String) {
        let from = self.current_consent.clone();
        self.current_consent = level;
        
        self.consent_history.push(ConsentChange {
            timestamp: current_timestamp(),
            from,
            to: level.clone(),
            reason,
        });
    }
    
    /// Get current consent
    pub fn get_consent(&self) -> ConsentLevel {
        self.current_consent.clone()
    }
    
    /// Check if can perform action
    pub fn can_perform(&self, required: ConsentLevel) -> Result<(), ResourceError> {
        let current = match &self.current_consent {
            ConsentLevel::None => 0,
            ConsentLevel::MonitoringOnly => 1,
            ConsentLevel::LowImpact => 2,
            ConsentLevel::FullAccess => 3,
        };
        
        let required_level = match required {
            ConsentLevel::None => 0,
            ConsentLevel::MonitoringOnly => 1,
            ConsentLevel::LowImpact => 2,
            ConsentLevel::FullAccess => 3,
        };
        
        if current >= required_level {
            Ok(())
        } else {
            Err(ResourceError::ConsentNotGranted)
        }
    }
    
    /// Check if mining is allowed
    pub fn can_mine(&self) -> Result<(), ResourceError> {
        if self.require_explicit_for_mining {
            self.can_perform(ConsentLevel::FullAccess)
        } else {
            self.can_perform(ConsentLevel::LowImpact)
        }
    }
    
    /// Get consent history
    pub fn history(&self) -> &[ConsentChange] {
        &self.consent_history
    }
}

// ============================================================================
// RESOURCE MONITOR
// ============================================================================

/// Resource monitor
pub struct ResourceMonitor {
    /// Current usage
    usage: ResourceUsage,
    
    /// Thresholds
    thresholds: ResourceThresholds,
    
    /// Law enforcer
    law_enforcer: LawEnforcer,
    
    /// Consent manager
    consent: ConsentManager,
    
    /// Active budgets
    active_budgets: HashMap<String, ResourceBudget>,
    
    /// History of usage
    usage_history: Vec<ResourceUsage>,
    
    /// Master enable flag
    enabled: bool,
    
    /// Creator can disable everything
    creator_disabled: bool,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            usage: ResourceUsage::default(),
            thresholds: ResourceThresholds::default(),
            law_enforcer: LawEnforcer::new(),
            consent: ConsentManager::new(),
            active_budgets: HashMap::new(),
            usage_history: Vec::new(),
            enabled: false,
            creator_disabled: false,
        }
    }
    
    /// Enable monitoring and operations
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// Disable monitoring and operations
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Creator can disable all operations
    pub fn creator_disable(&mut self) {
        self.creator_disabled = true;
        self.enabled = false;
    }
    
    /// Creator can re-enable
    pub fn creator_enable(&mut self) {
        self.creator_disabled = false;
    }
    
    /// Check if all operations are allowed
    pub fn is_operational(&self) -> bool {
        self.enabled && !self.creator_disabled
    }
    
    /// Set consent level
    pub fn set_consent(&mut self, level: ConsentLevel) {
        self.consent.set_consent(level, "User action".to_string());
    }
    
    /// Update resource usage (normally from system metrics)
    pub fn update_usage(&mut self, usage: ResourceUsage) {
        // Check laws
        if let Err(e) = self.law_enforcer.check(&usage) {
            self.law_enforcer.record_violation(LawViolation {
                timestamp: current_timestamp(),
                law_type: "ResourceLimit".to_string(),
                value: usage.cpu_usage_percent,
                limit: self.law_enforcer.limits.cpu_max,
                severity: ViolationSeverity::Warning,
            });
        }
        
        self.usage = usage;
        
        // Keep history
        self.usage_history.push(self.usage.clone());
        const MAX_HISTORY: usize = 1000;
        if self.usage_history.len() > MAX_HISTORY {
            self.usage_history.remove(0);
        }
    }
    
    /// Simulate resource check (for testing without real metrics)
    pub fn simulate_idle(&mut self, idle_cpu: f32, free_memory_mb: u64) {
        let mut usage = ResourceUsage::default();
        usage.cpu_usage_percent = 100.0 - idle_cpu;
        usage.memory_used_bytes = (8 * 1024 * 1024 * 1024) - (free_memory_mb * 1024 * 1024);
        usage.memory_total_bytes = 8 * 1024 * 1024 * 1024;
        self.update_usage(usage);
    }
    
    /// Check if resources are available for a task
    pub fn can_allocate(&self, budget: &ResourceBudget) -> Result<(), ResourceError> {
        if !self.is_operational() {
            return Err(ResourceError::MonitoringFailed("System disabled".to_string()));
        }
        
        // Check CPU threshold
        let available_cpu = self.usage.available_cpu_percent();
        if available_cpu < self.thresholds.min_idle_cpu_percent {
            return Err(ResourceError::InsufficientResources);
        }
        if budget.max_cpu_percent > available_cpu {
            return Err(ResourceError::InsufficientResources);
        }
        
        // Check memory threshold
        let available_memory_mb = self.usage.available_memory_bytes() / (1024 * 1024);
        if available_memory_mb < self.thresholds.min_idle_memory_mb {
            return Err(ResourceError::InsufficientResources);
        }
        if budget.max_memory_bytes > self.usage.available_memory_bytes() {
            return Err(ResourceError::InsufficientResources);
        }
        
        Ok(())
    }
    
    /// Check if mining is allowed
    pub fn can_mine(&self) -> Result<(), ResourceError> {
        // Check consent
        self.consent.can_mine()?;
        
        // Check if operational
        if !self.is_operational() {
            return Err(ResourceError::MonitoringFailed("System disabled".to_string()));
        }
        
        // Check idle resources
        if self.usage.available_cpu_percent() < self.thresholds.min_idle_cpu_percent {
            return Err(ResourceError::InsufficientResources);
        }
        
        let available_memory_mb = self.usage.available_memory_bytes() / (1024 * 1024);
        if available_memory_mb < self.thresholds.min_idle_memory_mb {
            return Err(ResourceError::InsufficientResources);
        }
        
        // Check laws
        self.law_enforcer.check(&self.usage)?;
        
        Ok(())
    }
    
    /// Allocate budget to task
    pub fn allocate(&mut self, budget: ResourceBudget) -> Result<(), ResourceError> {
        self.can_allocate(&budget)?;
        self.active_budgets.insert(budget.task_name.clone(), budget);
        Ok(())
    }
    
    /// Release budget
    pub fn release(&mut self, task_name: &str) {
        self.active_budgets.remove(task_name);
    }
    
    /// Get current usage
    pub fn get_usage(&self) -> &ResourceUsage {
        &self.usage
    }
    
    /// Get thresholds
    pub fn get_thresholds(&self) -> &ResourceThresholds {
        &self.thresholds
    }
    
    /// Set thresholds
    pub fn set_thresholds(&mut self, thresholds: ResourceThresholds) {
        self.thresholds = thresholds;
    }
    
    /// Get consent status
    pub fn get_consent_level(&self) -> ConsentLevel {
        self.consent.get_consent()
    }
    
    /// Get active budgets
    pub fn get_active_budgets(&self) -> Vec<&ResourceBudget> {
        self.active_budgets.values().collect()
    }
    
    /// Get usage history
    pub fn get_usage_history(&self) -> &[ResourceUsage] {
        &self.usage_history
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> ResourceStats {
        ResourceStats {
            enabled: self.enabled,
            creator_disabled: self.creator_disabled,
            consent_level: self.consent.get_consent(),
            current_usage: self.usage.clone(),
            active_budgets: self.active_budgets.len(),
            total_violations: self.law_enforcer.violations.len(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResourceStats {
    pub enabled: bool,
    pub creator_disabled: bool,
    pub consent_level: ConsentLevel,
    pub current_usage: ResourceUsage,
    pub active_budgets: usize,
    pub total_violations: usize,
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MINING ORCHESTRATOR (OPTIONAL FEATURE)
// ============================================================================

/// Mining orchestrator - coordinates mining operations
pub struct MiningOrchestrator {
    /// Resource monitor
    resource_monitor: ResourceMonitor,
    
    /// Is mining enabled
    mining_enabled: bool,
    
    /// Mining statistics
    stats: MiningStats,
}

#[derive(Clone, Debug, Default)]
pub struct MiningStats {
    pub total_shares: u64,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub total_earnings: u64,
    pub last_share_time: u64,
}

impl MiningOrchestrator {
    pub fn new() -> Self {
        Self {
            resource_monitor: ResourceMonitor::new(),
            mining_enabled: false,
            stats: MiningStats::default(),
        }
    }
    
    /// Enable mining (requires consent + idle resources)
    pub fn enable_mining(&mut self) -> Result<(), ResourceError> {
        // Check if system supports mining
        self.resource_monitor.can_mine()?;
        self.mining_enabled = true;
        Ok(())
    }
    
    /// Disable mining
    pub fn disable_mining(&mut self) {
        self.mining_enabled = false;
    }
    
    /// Check if can mine now
    pub fn can_mine(&self) -> bool {
        self.mining_enabled && self.resource_monitor.can_mine().is_ok()
    }
    
    /// Request mining job (would interface with pool in real implementation)
    pub fn request_job(&mut self) -> Result<(), ResourceError> {
        if !self.can_mine() {
            return Err(ResourceError::InsufficientResources);
        }
        
        // In real implementation, would request job from mining pool
        // For now, just record attempt
        self.stats.total_shares += 1;
        self.stats.last_share_time = current_timestamp();
        
        Ok(())
    }
    
    /// Record share result
    pub fn record_share(&mut self, accepted: bool) {
        if accepted {
            self.stats.accepted_shares += 1;
        } else {
            self.stats.rejected_shares += 1;
        }
    }
    
    /// Get mining stats
    pub fn get_stats(&self) -> MiningStats {
        self.stats.clone()
    }
    
    /// Get resource monitor
    pub fn resource_monitor(&self) -> &ResourceMonitor {
        &self.resource_monitor
    }
    
    /// Get mutable resource monitor
    pub fn resource_monitor_mut(&mut self) -> &mut ResourceMonitor {
        &mut self.resource_monitor
    }
}

impl Default for MiningOrchestrator {
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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consent_levels() {
        let mut monitor = ResourceMonitor::new();
        
        assert!(monitor.get_consent_level() == ConsentLevel::None);
        
        monitor.set_consent(ConsentLevel::LowImpact);
        assert!(monitor.get_consent_level() == ConsentLevel::LowImpact);
        
        monitor.set_consent(ConsentLevel::FullAccess);
        assert!(monitor.get_consent_level() == ConsentLevel::FullAccess);
    }

    #[test]
    fn test_resource_monitoring() {
        let mut monitor = ResourceMonitor::new();
        monitor.enable();
        
        // Simulate idle system
        monitor.simulate_idle(80.0, 6000);
        
        // Create budget
        let budget = ResourceBudget::new("test_task")
            .with_cpu(20.0)
            .with_memory(256 * 1024 * 1024);
        
        assert!(monitor.can_allocate(&budget).is_ok());
    }

    #[test]
    fn test_law_enforcement() {
        let mut monitor = ResourceMonitor::new();
        monitor.enable();
        
        // Simulate high CPU
        let mut usage = ResourceUsage::default();
        usage.cpu_usage_percent = 95.0;
        
        monitor.update_usage(usage);
        
        // Check if violation was recorded
        let stats = monitor.get_stats();
        assert!(stats.total_violations > 0);
    }

    #[test]
    fn test_mining_consent() {
        let mut orchestrator = MiningOrchestrator::new();
        
        // Without consent, mining should fail
        assert!(orchestrator.can_mine() == false);
        
        // Grant consent and idle resources
        orchestrator.resource_monitor.set_consent(ConsentLevel::FullAccess);
        orchestrator.resource_monitor.enable();
        orchestrator.resource_monitor.simulate_idle(80.0, 6000);
        
        // Should still be disabled
        assert!(orchestrator.can_mine() == false);
        
        // Enable mining
        orchestrator.enable_mining().unwrap();
        
        // Now should work
        assert!(orchestrator.can_mine() == true);
    }

    #[test]
    fn test_creator_disable() {
        let mut monitor = ResourceMonitor::new();
        monitor.enable();
        
        monitor.simulate_idle(80.0, 6000);
        monitor.set_consent(ConsentLevel::FullAccess);
        
        assert!(monitor.is_operational());
        
        // Creator disables
        monitor.creator_disable();
        
        assert!(!monitor.is_operational());
        
        // Creator re-enables
        monitor.creator_enable();
        
        assert!(monitor.is_operational());
    }

    #[test]
    fn test_idle_threshold() {
        let mut monitor = ResourceMonitor::new();
        monitor.enable();
        monitor.set_consent(ConsentLevel::FullAccess);
        
        // Not enough idle CPU
        monitor.simulate_idle(10.0, 6000);
        
        let budget = ResourceBudget::new("test");
        let result = monitor.can_allocate(&budget);
        
        assert!(result.is_err());
    }
}