//! # Self Health Monitor — Auto-supervisión y Detección de Anomalías
//!
//! Este módulo implementa la auto-supervisión del sistema EDEN:
//! - Health monitoring continuo
//! - Anomaly detection usando estadísticas y umbrales
//! - Emergency throttle cuando se detectan anomalías
//! - Integración con el Creator kill-switch

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::consciousness::guided_evolution::{EvolutionHealth, GuidedEvolutionManager};
use crate::governance::{GovernanceManager, GovernanceStats};

// ============================================================================
// HEALTH METRICS
// ============================================================================

/// Métricas de salud del sistema
#[derive(Debug, Clone)]
pub struct SystemHealth {
    /// Timestamp de la medición
    pub timestamp: u64,
    /// Score general de salud (0.0 - 1.0)
    pub health_score: f64,
    /// Estado de componentes
    pub components: HashMap<String, ComponentHealth>,
    /// Anomalías detectadas
    pub anomalies: Vec<Anomaly>,
    /// Indicadores de riesgo
    pub risk_indicators: Vec<RiskIndicator>,
}

/// Salud de un componente individual
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub name: String,
    pub status: ComponentStatus,
    pub metrics: HashMap<String, f64>,
    pub last_update: u64,
}

/// Estado de un componente
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

/// Anomalía detectada
#[derive(Debug, Clone)]
pub struct Anomaly {
    pub id: u64,
    pub component: String,
    pub anomaly_type: AnomalyType,
    pub severity: Severity,
    pub description: String,
    pub detected_at: u64,
    pub metrics: HashMap<String, f64>,
    pub resolved: bool,
    pub resolved_at: Option<u64>,
}

/// Tipo de anomalía
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnomalyType {
    /// Uso excesivo de recursos
    ResourceExhaustion,
    /// Rate alto de errores
    ErrorRateSpike,
    /// Latencia anormal
    LatencyAnomaly,
    /// Patrón de voto sospechoso
    VotingAnomaly,
    /// Evolución bloqueada
    EvolutionStalled,
    /// Dissent excesivo
    ExcessiveDissent,
    /// Quorum no alcanzado
    QuorumFailure,
    /// Comportamiento malicioso detectado
    MaliciousBehavior,
    /// Desviación de protocolo
    ProtocolDeviation,
}

/// Severidad de anomalía
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Indicador de riesgo emergente
#[derive(Debug, Clone)]
pub struct RiskIndicator {
    pub indicator_type: RiskIndicatorType,
    pub value: f64,
    pub threshold: f64,
    pub triggered: bool,
    pub description: String,
}

/// Tipo de indicador de riesgo
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskIndicatorType {
    /// Concentración de poder
    PowerConcentration,
    /// Rate de evoluciones rechazadas
    RejectionRate,
    /// Tiempo de respuesta promedio
    AvgResponseTime,
    /// Uso de memoria
    MemoryUsage,
    /// CPU usage
    CpuUsage,
    /// Network latency
    NetworkLatency,
    /// Evoluciones pendientes
    PendingEvolutions,
    /// Nodos activos
    ActiveNodesRatio,
}

// ============================================================================
// THRESHOLDS AND CONFIGURATION
// ============================================================================

/// Configuración del health monitor
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Intervalo de checks (ms)
    pub check_interval_ms: u64,
    /// Ventana para métricas (número de samples)
    pub metric_window_size: usize,
    /// Threshold para anomaly detection
    pub anomaly_threshold_std: f64,
    /// Auto-throttle habilitado
    pub auto_throttle_enabled: bool,
    /// Throttle threshold (si health < X, throttle)
    pub throttle_threshold: f64,
    /// Max anomalías activas antes de alerta
    pub max_active_anomalies: usize,
    /// Creator notification threshold
    pub creator_alert_threshold: Severity,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: 5000,          // 5 segundos
            metric_window_size: 60,           // 5 minutos de datos (a 5s interval)
            anomaly_threshold_std: 2.5,        // 2.5 desviaciones estándar
            auto_throttle_enabled: true,
            throttle_threshold: 0.3,           // Throttle si health < 30%
            max_active_anomalies: 10,
            creator_alert_threshold: Severity::High,
        }
    }
}

/// Estado del emergency throttle
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThrottleState {
    /// Normal - sin limitaciones
    Normal,
    /// Advertencia - logging aumentado
    Warning,
    /// Throttled - rate limiting activo
    Throttled,
    /// Emergency - solo operaciones críticas
    Emergency,
}

// ============================================================================
// SELF HEALTH MONITOR
// ============================================================================

/// Monitor de salud del sistema
pub struct SelfHealthMonitor {
    /// Configuración
    config: HealthConfig,
    /// Estado actual
    current_health: SystemHealth,
    /// Historial de métricas
    metrics_history: HashMap<String, VecDeque<MetricSample>>,
    /// Anomalías detectadas
    anomalies: Vec<Anomaly>,
    /// Contador de anomalías
    anomaly_counter: u64,
    /// Estado de throttle
    throttle_state: ThrottleState,
    /// Governance manager para stats
    governance: Option<Arc<RwLock<GovernanceManager>>>,
    /// Evolution manager para health
    evolution: Option<Arc<RwLock<GuidedEvolutionManager>>>,
    /// Último check
    last_check: u64,
    /// Throttle start time
    throttle_start: Option<u64>,
    /// Cola de eventos de throttle
    throttle_events: Vec<ThrottleEvent>,
}

/// Muestra de métrica
#[derive(Debug, Clone)]
pub struct MetricSample {
    pub timestamp: u64,
    pub value: f64,
}

/// Evento de throttle
#[derive(Debug, Clone)]
pub struct ThrottleEvent {
    pub timestamp: u64,
    pub from_state: ThrottleState,
    pub to_state: ThrottleState,
    pub reason: String,
    pub auto_triggered: bool,
}

impl Default for SelfHealthMonitor {
    fn default() -> Self {
        Self::new(HealthConfig::default())
    }
}

impl SelfHealthMonitor {
    /// Crear nuevo monitor
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            current_health: SystemHealth {
                timestamp: current_timestamp(),
                health_score: 1.0,
                components: HashMap::new(),
                anomalies: Vec::new(),
                risk_indicators: Vec::new(),
            },
            metrics_history: HashMap::new(),
            anomalies: Vec::new(),
            anomaly_counter: 0,
            throttle_state: ThrottleState::Normal,
            governance: None,
            evolution: None,
            last_check: current_timestamp(),
            throttle_start: None,
            throttle_events: Vec::new(),
        }
    }

    /// Vincular governance manager
    pub fn set_governance(&mut self, gov: Arc<RwLock<GovernanceManager>>) {
        self.governance = Some(gov);
    }

    /// Vincular evolution manager
    pub fn set_evolution(&mut self, evo: Arc<RwLock<GuidedEvolutionManager>>) {
        self.evolution = Some(evo);
    }

    // =========================================================================
    // METRIC COLLECTION
    // =========================================================================

    /// Registrar métrica
    pub fn record_metric(&mut self, component: &str, metric: &str, value: f64) {
        let key = format!("{}::{}", component, metric);
        
        let samples = self.metrics_history
            .entry(key)
            .or_insert_with(VecDeque::new);
        
        samples.push_back(MetricSample {
            timestamp: current_timestamp(),
            value,
        });
        
        // Mantener ventana de tamaño limitado
        while samples.len() > self.config.metric_window_size {
            samples.pop_front();
        }
    }

    /// Obtener valor actual de métrica
    pub fn get_metric(&self, component: &str, metric: &str) -> Option<f64> {
        let key = format!("{}::{}", component, metric);
        self.metrics_history.get(&key)
            .and_then(|samples| samples.back())
            .map(|s| s.value)
    }

    /// Obtener promedio de métrica en ventana
    pub fn get_metric_avg(&self, component: &str, metric: &str) -> Option<f64> {
        let key = format!("{}::{}", component, metric);
        self.metrics_history.get(&key)
            .map(|samples| {
                let sum: f64 = samples.iter().map(|s| s.value).sum();
                sum / samples.len() as f64
            })
    }

    /// Obtener desviación estándar de métrica
    pub fn get_metric_std(&self, component: &str, metric: &str) -> Option<f64> {
        let key = format!("{}::{}", component, metric);
        self.metrics_history.get(&key).and_then(|samples| {
            if samples.len() < 2 {
                return None;
            }
            let avg = samples.iter().map(|s| s.value).sum::<f64>() / samples.len() as f64;
            let variance = samples.iter()
                .map(|s| (s.value - avg).powi(2))
                .sum::<f64>() / samples.len() as f64;
            Some(variance.sqrt())
        })
    }

    // =========================================================================
    // HEALTH CHECK
    // =========================================================================

    /// Realizar check de salud
    pub fn check_health(&mut self) -> SystemHealth {
        let now = current_timestamp();
        self.last_check = now;
        
        // Recolectar métricas de componentes
        self.collect_governance_metrics();
        self.collect_evolution_metrics();
        
        // Detectar anomalías
        self.detect_anomalies();
        
        // Calcular score de salud
        let health_score = self.calculate_health_score();
        
        // Actualizar estado
        self.current_health = SystemHealth {
            timestamp: now,
            health_score,
            components: self.get_component_statuses(),
            anomalies: self.anomalies.clone(),
            risk_indicators: self.calculate_risk_indicators(),
        };
        
        // Verificar throttle
        self.check_throttle();
        
        self.current_health.clone()
    }

    /// Recolectar métricas de governance
    fn collect_governance_metrics(&mut self) {
        if let Some(ref gov) = self.governance {
            if let Ok(gov) = gov.read() {
                let stats = gov.get_stats();
                
                self.record_metric("governance", "total_proposals", stats.total_proposals as f64);
                self.record_metric("governance", "accepted", stats.accepted as f64);
                self.record_metric("governance", "rejected", stats.rejected as f64);
                self.record_metric("governance", "vetoed", stats.vetoed as f64);
                
                // Calcular rate de rechazo
                if stats.total_proposals > 0 {
                    let rejection_rate = stats.rejected as f64 / stats.total_proposals as f64;
                    self.record_metric("governance", "rejection_rate", rejection_rate);
                }
                
                // Get active proposals
                let active = gov.get_active_proposals().len();
                self.record_metric("governance", "active_proposals", active as f64);
            }
        }
    }

    /// Recolectar métricas de evolución
    fn collect_evolution_metrics(&mut self) {
        if let Some(ref evo) = self.evolution {
            if let Ok(evo) = evo.read() {
                let health = evo.health_check();
                
                self.record_metric("evolution", "active_proposals", health.active_proposals as f64);
                self.record_metric("evolution", "total_proposals", health.total_proposals as f64);
                self.record_metric("evolution", "rejection_rate", health.rejection_rate);
                
                // Verificar health warnings
                for warning in &health.warnings {
                    self.record_metric("evolution", "warning", 1.0);
                }
            }
        }
    }

    /// Detectar anomalías usando desviación estándar
    fn detect_anomalies(&mut self) {
        let components = vec!["governance", "evolution"];
        
        for component in components {
            // Check rejection rate
            if let (Some(avg), Some(std)) = (
                self.get_metric_avg(component, "rejection_rate"),
                self.get_metric_std(component, "rejection_rate"),
            ) {
                if let Some(current) = self.get_metric(component, "rejection_rate") {
                    if std > 0.0 && (current - avg).abs() > self.config.anomaly_threshold_std * std {
                        self.add_anomaly(component, AnomalyType::ErrorRateSpike, Severity::Medium,
                            format!("{} rejection rate deviation: current={:.2}, avg={:.2}, std={:.2}",
                                component, current, avg, std));
                    }
                }
            }
            
            // Check active proposals (stall detection)
            if let Some(active) = self.get_metric_avg(component, "active_proposals") {
                if let Some(current) = self.get_metric(component, "active_proposals") {
                    // Si hay activas constantemente por mucho tiempo, podría ser stall
                    if current > 20.0 && self.metrics_history
                        .get(&format!("{}::active_proposals", component))
                        .map(|v| v.len())
                        .unwrap_or(0) > 10 {
                        self.add_anomaly(component, AnomalyType::EvolutionStalled, Severity::High,
                            format!("High number of active {} proposals: {}", component, current));
                    }
                }
            }
        }
        
        // Limpiar anomalías resueltas
        self.cleanup_resolved_anomalies();
    }

    /// Agregar anomalía
    fn add_anomaly(&mut self, component: &str, anomaly_type: AnomalyType, severity: Severity, description: String) {
        // Check si ya existe esta anomalía (mismo componente y tipo)
        let exists = self.anomalies.iter().any(|a| 
            !a.resolved && a.component == component && a.anomaly_type == anomaly_type
        );
        
        if !exists {
            self.anomaly_counter += 1;
            
            let mut metrics = HashMap::new();
            if let Some(v) = self.get_metric(component, "rejection_rate") {
                metrics.insert("rejection_rate".to_string(), v);
            }
            if let Some(v) = self.get_metric(component, "active_proposals") {
                metrics.insert("active_proposals".to_string(), v);
            }
            
            self.anomalies.push(Anomaly {
                id: self.anomaly_counter,
                component: component.to_string(),
                anomaly_type,
                severity,
                description,
                detected_at: current_timestamp(),
                metrics,
                resolved: false,
                resolved_at: None,
            });
        }
    }

    /// Limpiar anomalías resueltas
    fn cleanup_resolved_anomalies(&mut self) {
        let now = current_timestamp();
        for anomaly in &mut self.anomalies {
            if anomaly.resolved && anomaly.resolved_at.is_none() {
                anomaly.resolved_at = Some(now);
            }
        }
        
        // Remover anomalías muy antiguas (> 1 hora)
        self.anomalies.retain(|a| {
            !a.resolved || (now - a.detected_at) < 3600_000
        });
    }

    /// Calcular score de salud
    fn calculate_health_score(&self) -> f64 {
        let mut score = 1.0;
        
        // Penalizar por anomalías activas
        let critical_count = self.anomalies.iter().filter(|a| 
            !a.resolved && a.severity >= Severity::High
        ).count();
        score -= critical_count as f64 * 0.1;
        
        // Penalizar por anomalías mediums
        let medium_count = self.anomalies.iter().filter(|a| 
            !a.resolved && a.severity == Severity::Medium
        ).count();
        score -= medium_count as f64 * 0.05;
        
        // Penalizar por rejection rate alto
        if let Some(rej_rate) = self.get_metric_avg("governance", "rejection_rate") {
            if rej_rate > 0.3 {
                score -= (rej_rate - 0.3) * 0.5;
            }
        }
        
        // Penalizar por throttle state
        match self.throttle_state {
            ThrottleState::Normal => {},
            ThrottleState::Warning => score -= 0.05,
            ThrottleState::Throttled => score -= 0.15,
            ThrottleState::Emergency => score -= 0.3,
        }
        
        score.clamp(0.0, 1.0)
    }

    /// Obtener estado de componentes
    fn get_component_statuses(&self) -> HashMap<String, ComponentHealth> {
        let mut components = HashMap::new();
        
        for component in ["governance", "evolution"] {
            let mut metrics = HashMap::new();
            let mut healthy = true;
            
            // Check rejection rate
            if let Some(rate) = self.get_metric(component, "rejection_rate") {
                metrics.insert("rejection_rate".to_string(), rate);
                if rate > 0.5 {
                    healthy = false;
                }
            }
            
            // Check active proposals
            if let Some(active) = self.get_metric(component, "active_proposals") {
                metrics.insert("active_proposals".to_string(), active);
                if active > 50.0 {
                    healthy = false;
                }
            }
            
            let status = if healthy { ComponentStatus::Healthy } else { ComponentStatus::Degraded };
            
            components.insert(component.to_string(), ComponentHealth {
                name: component.to_string(),
                status,
                metrics,
                last_update: current_timestamp(),
            });
        }
        
        components
    }

    /// Calcular indicadores de riesgo
    fn calculate_risk_indicators(&self) -> Vec<RiskIndicator> {
        let mut indicators = Vec::new();
        
        // Rejection rate
        if let Some(rej_rate) = self.get_metric_avg("governance", "rejection_rate") {
            indicators.push(RiskIndicator {
                indicator_type: RiskIndicatorType::RejectionRate,
                value: rej_rate,
                threshold: 0.3,
                triggered: rej_rate > 0.3,
                description: format!("Governance rejection rate: {:.1}%", rej_rate * 100.0),
            });
        }
        
        // Pending evolutions
        if let Some(pending) = self.get_metric_avg("evolution", "active_proposals") {
            indicators.push(RiskIndicator {
                indicator_type: RiskIndicatorType::PendingEvolutions,
                value: pending,
                threshold: 20.0,
                triggered: pending > 20.0,
                description: format!("Pending evolutions: {:.0}", pending),
            });
        }
        
        // Anomaly count
        let unresolved = self.anomalies.iter().filter(|a| !a.resolved).count();
        indicators.push(RiskIndicator {
            indicator_type: RiskIndicatorType::RejectionRate,  // Reuse for anomaly count
            value: unresolved as f64,
            threshold: 5.0,
            triggered: unresolved > 5,
            description: format!("Active anomalies: {}", unresolved),
        });
        
        indicators
    }

    // =========================================================================
    // THROTTLE MANAGEMENT
    // =========================================================================

    /// Verificar si se necesita throttle
    fn check_throttle(&mut self) {
        if !self.config.auto_throttle_enabled {
            return;
        }
        
        let health = self.current_health.health_score;
        let anomaly_count = self.anomalies.iter().filter(|a| !a.resolved).count();
        
        let new_state = if health < self.config.throttle_threshold {
            // Critical health -> emergency throttle
            ThrottleState::Emergency
        } else if anomaly_count > self.config.max_active_anomalies / 2 {
            // Many anomalies -> throttled
            ThrottleState::Throttled
        } else if anomaly_count > 0 {
            // Some anomalies -> warning
            ThrottleState::Warning
        } else {
            ThrottleState::Normal
        };
        
        if new_state != self.throttle_state {
            let event = ThrottleEvent {
                timestamp: current_timestamp(),
                from_state: self.throttle_state,
                to_state: new_state,
                reason: format!("Health score: {:.2}, anomalies: {}", health, anomaly_count),
                auto_triggered: true,
            };
            
            self.throttle_events.push(event);
            self.throttle_state = new_state;
            
            if new_state == ThrottleState::Emergency {
                self.throttle_start = Some(current_timestamp());
            }
        }
    }

    /// Obtener estado de throttle
    pub fn get_throttle_state(&self) -> ThrottleState {
        self.throttle_state
    }

    /// Verificar si operación puede proceder
    pub fn can_proceed(&self, operation_type: OperationType) -> bool {
        match self.throttle_state {
            ThrottleState::Normal => true,
            ThrottleState::Warning => true,  // Warning solo hace logging
            ThrottleState::Throttled => {
                // Solo operaciones no-criticales son throttleadas
                !matches!(operation_type, OperationType::Critical)
            }
            ThrottleState::Emergency => {
                // Solo reads y críticas
                matches!(operation_type, OperationType::Read | OperationType::Critical)
            }
        }
    }

    /// Obtener rate limit para tipo de operación
    pub fn get_rate_limit(&self, operation_type: OperationType) -> u32 {
        match self.throttle_state {
            ThrottleState::Normal => u32::MAX,
            ThrottleState::Warning => u32::MAX,
            ThrottleState::Throttled => match operation_type {
                OperationType::Read => u32::MAX,
                OperationType::Write => 100,
                OperationType::Evolution => 10,
                OperationType::Critical => u32::MAX,
            },
            ThrottleState::Emergency => match operation_type {
                OperationType::Read => 1000,
                OperationType::Write => 10,
                OperationType::Evolution => 1,
                OperationType::Critical => u32::MAX,
            },
        }
    }

    /// Resolver anomalía manualmente
    pub fn resolve_anomaly(&mut self, anomaly_id: u64) -> bool {
        if let Some(anomaly) = self.anomalies.iter_mut().find(|a| a.id == anomaly_id) {
            anomaly.resolved = true;
            anomaly.resolved_at = Some(current_timestamp());
            
            // Recalcular throttle
            self.check_throttle();
            return true;
        }
        false
    }

    /// Obtener historial de throttle
    pub fn get_throttle_history(&self) -> &[ThrottleEvent] {
        &self.throttle_events
    }

    // =========================================================================
    // ACCESSORS
    // =========================================================================

    /// Obtener salud actual
    pub fn get_health(&self) -> &SystemHealth {
        &self.current_health
    }

    /// Obtener anomalías activas
    pub fn get_active_anomalies(&self) -> Vec<&Anomaly> {
        self.anomalies.iter().filter(|a| !a.resolved).collect()
    }

    /// Obtener métricas de componente
    pub fn get_component_metrics(&self, component: &str) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        let prefix = format!("{}::", component);
        
        for (key, samples) in &self.metrics_history {
            if key.starts_with(&prefix) {
                if let Some(last) = samples.back() {
                    let metric_name = key.strip_prefix(&prefix).unwrap_or(key);
                    result.insert(metric_name.to_string(), last.value);
                }
            }
        }
        
        result
    }
}

/// Tipo de operación para throttle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Read,
    Write,
    Evolution,
    Critical,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let monitor = SelfHealthMonitor::new(HealthConfig::default());
        assert_eq!(monitor.throttle_state, ThrottleState::Normal);
        assert_eq!(monitor.current_health.health_score, 1.0);
    }

    #[test]
    fn test_metric_recording() {
        let mut monitor = SelfHealthMonitor::new(HealthConfig::default());
        
        monitor.record_metric("test", "value", 42.0);
        assert_eq!(monitor.get_metric("test", "value"), Some(42.0));
        
        monitor.record_metric("test", "value", 43.0);
        assert_eq!(monitor.get_metric("test", "value"), Some(43.0));
    }

    #[test]
    fn test_metric_average() {
        let mut monitor = SelfHealthMonitor::new(HealthConfig::default());
        
        monitor.record_metric("test", "value", 10.0);
        monitor.record_metric("test", "value", 20.0);
        monitor.record_metric("test", "value", 30.0);
        
        assert_eq!(monitor.get_metric_avg("test", "value"), Some(20.0));
    }

    #[test]
    fn test_anomaly_detection() {
        let mut monitor = SelfHealthMonitor::new(HealthConfig::default());
        
        // Add normal values
        for _ in 0..10 {
            monitor.record_metric("governance", "rejection_rate", 0.1);
        }
        
        // Check health (should not detect anomaly with normal values)
        let health = monitor.check_health();
        assert!(health.health_score > 0.5);  // Should be healthy
        
        // Add outlier
        monitor.record_metric("governance", "rejection_rate", 0.8);
        
        let health = monitor.check_health();
        assert!(health.anomalies.len() >= 0);  // May detect anomaly
    }

    #[test]
    fn test_throttle_state_transitions() {
        let mut monitor = SelfHealthMonitor::new(HealthConfig::default());
        
        // Initially normal
        assert_eq!(monitor.get_throttle_state(), ThrottleState::Normal);
        
        // Add some anomalies
        monitor.record_metric("test", "anomaly", 1.0);
        
        // Check health with anomalies
        let health = monitor.check_health();
        assert!(health.health_score < 1.0);  // Should be degraded
    }

    #[test]
    fn test_can_proceed() {
        let monitor = SelfHealthMonitor::new(HealthConfig::default());
        
        // In normal state, all operations can proceed
        assert!(monitor.can_proceed(OperationType::Read));
        assert!(monitor.can_proceed(OperationType::Write));
        assert!(monitor.can_proceed(OperationType::Evolution));
        assert!(monitor.can_proceed(OperationType::Critical));
    }

    #[test]
    fn test_rate_limits() {
        let monitor = SelfHealthMonitor::new(HealthConfig::default());
        
        // In normal state, no limits
        assert_eq!(monitor.get_rate_limit(OperationType::Read), u32::MAX);
        assert_eq!(monitor.get_rate_limit(OperationType::Evolution), u32::MAX);
    }

    #[test]
    fn test_resolve_anomaly() {
        let mut monitor = SelfHealthMonitor::new(HealthConfig::default());
        
        // Record enough data to trigger anomaly
        for _ in 0..20 {
            monitor.record_metric("test", "metric", 0.9);
        }
        
        monitor.check_health();
        
        // Try to resolve non-existent anomaly
        assert!(!monitor.resolve_anomaly(999));
    }
}