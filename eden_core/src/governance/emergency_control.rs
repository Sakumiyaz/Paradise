//! # Emergency Control System — Control de Emergencia y Contención
//!
//! Este módulo implementa:
//! - Emergency kill switch (parada de emergencia)
//! - Circuit breakers para prevenir cascadas
//! - Sandboxed emergence (contención de emergencia)
//! - Recovery procedures

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// EMERGENCY TYPES
// ============================================================================

/// Estado de emergencia
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmergencyState {
    /// Normal - sin emergencia
    Normal,
    /// Advertencia - algo no está bien
    Warning,
    /// Emergencia parcial - algunos sistemas afectados
    PartialEmergency,
    /// Emergencia total - todo parado
    TotalEmergency,
    /// Recuperación - volviendo a normal
    Recovery,
}

/// Tipo de emergencia
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmergencyType {
    /// Anomalía de comportamiento detectada
    BehaviorAnomaly,
    /// Seguridad comprometida
    SecurityBreach,
    /// Recurso exhausto
    ResourceExhaustion,
    /// Evolución fuera de control
    Evolution失控,
    /// Falla de quorum
    QuorumFailure,
    /// Creator kill-switch activado
    CreatorKillSwitch,
    /// Módulos no response
    ModuleTimeout,
    /// Error catastrófico
    CatastrophicError,
}

/// Acción de emergencia
#[derive(Debug, Clone)]
pub struct EmergencyAction {
    pub id: u64,
    pub action_type: EmergencyActionType,
    pub target: String,
    pub reason: String,
    pub executed_at: u64,
    pub succeeded: bool,
    pub rollback_available: bool,
}

/// Tipo de acción de emergencia
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmergencyActionType {
    /// Parar módulo específico
    StopModule,
    /// Pausar módulo
    PauseModule,
    /// Reiniciar módulo
    RestartModule,
    /// Cortar tráfico de red
    CutNetworkTraffic,
    /// Aislar nodo
    IsolateNode,
    /// Reset completo
    FullReset,
    /// Habilitar modo seguro
    EnableSafeMode,
    /// Deshabilitar evoluciones
    DisableEvolutions,
}

/// Circuito breaker para prevenir cascadas
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub name: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub last_failure: u64,
    pub threshold: u32,
    pub reset_timeout_ms: u64,
    pub half_open_after_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Failing, rejecting requests
    HalfOpen,   // Testing if recovered
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            name: String::new(),
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure: 0,
            threshold: 5,
            reset_timeout_ms: 30_000,
            half_open_after_ms: 60_000,
        }
    }
}

// ============================================================================
// EMERGENCY CONTROLLER
// ============================================================================

/// Controlador de emergencias
pub struct EmergencyController {
    /// Estado actual
    state: EmergencyState,
    /// Tipo de emergencia activa
    active_emergency: Option<EmergencyType>,
    /// Acciones ejecutadas
    action_history: Vec<EmergencyAction>,
    /// Circuit breakers
    circuit_breakers: HashMap<String, CircuitBreaker>,
    /// Módulos monitoreados
    monitored_modules: HashMap<String, ModuleStatus>,
    /// Cola de recovery
    recovery_queue: Vec<RecoveryTask>,
    /// Eventos de emergencia
    emergency_events: VecDeque<EmergencyEvent>,
    /// Stats
    stats: EmergencyStats,
}

/// Estado de módulo monitoreado
#[derive(Debug, Clone)]
pub struct ModuleStatus {
    pub name: String,
    pub enabled: bool,
    pub healthy: bool,
    pub last_heartbeat: u64,
    pub failures: u32,
}

impl Default for ModuleStatus {
    fn default() -> Self {
        Self {
            name: String::new(),
            enabled: true,
            healthy: true,
            last_heartbeat: 0,
            failures: 0,
        }
    }
}

/// Tarea de recuperación
#[derive(Debug, Clone)]
pub struct RecoveryTask {
    pub id: u64,
    pub target_module: String,
    pub task_type: RecoveryTaskType,
    pub status: RecoveryStatus,
    pub created_at: u64,
    pub completed_at: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryTaskType {
    Restart,
    ResetState,
    ClearCache,
    RebuildConnections,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Evento de emergencia
#[derive(Debug, Clone)]
pub struct EmergencyEvent {
    pub timestamp: u64,
    pub event_type: EmergencyEventType,
    pub severity: super::self_health_monitor::Severity,
    pub description: String,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmergencyEventType {
    EmergencyDeclared,
    EmergencyEscalated,
    EmergencyDeescalated,
    EmergencyEnded,
    CircuitBreakerOpened,
    CircuitBreakerClosed,
    ModuleStopped,
    ModuleRestarted,
    RecoveryStarted,
    RecoveryCompleted,
}

/// Estadísticas de emergencia
#[derive(Debug, Clone, Default)]
pub struct EmergencyStats {
    pub total_emergencies: u64,
    pub active_emergencies: u64,
    pub total_actions: u64,
    pub successful_actions: u64,
    pub circuit_trips: u64,
    pub recoveries_completed: u64,
}

impl Default for EmergencyController {
    fn default() -> Self {
        Self::new()
    }
}

impl EmergencyController {
    /// Crear nuevo controlador
    pub fn new() -> Self {
        Self {
            state: EmergencyState::Normal,
            active_emergency: None,
            action_history: Vec::new(),
            circuit_breakers: HashMap::new(),
            monitored_modules: HashMap::new(),
            recovery_queue: Vec::new(),
            emergency_events: VecDeque::new(),
            stats: EmergencyStats::default(),
        }
    }

    // =========================================================================
    // EMERGENCY MANAGEMENT
    // =========================================================================

    /// Declarar emergencia
    pub fn declare_emergency(&mut self, emergency_type: EmergencyType, reason: String) -> u64 {
        let now = current_timestamp();
        self.active_emergency = Some(emergency_type);
        
        // Update state based on emergency type
        let new_state = match emergency_type {
            EmergencyType::CreatorKillSwitch => EmergencyState::TotalEmergency,
            EmergencyType::CatastrophicError => EmergencyState::TotalEmergency,
            _ => EmergencyState::PartialEmergency,
        };
        
        let old_state = self.state;
        self.state = new_state;
        
        let event = EmergencyEvent {
            timestamp: now,
            event_type: EmergencyEventType::EmergencyDeclared,
            severity: super::self_health_monitor::Severity::Critical,
            description: format!("{:?}: {}", emergency_type, reason),
            data: HashMap::new(),
        };
        self.emergency_events.push_back(event);
        
        self.stats.total_emergencies += 1;
        self.stats.active_emergencies += 1;
        
        // Return emergency ID (using timestamp as simple ID)
        now
    }

    /// Terminar emergencia
    pub fn end_emergency(&mut self) {
        if let Some(_) = self.active_emergency.take() {
            self.state = EmergencyState::Recovery;
            self.stats.active_emergencies = self.stats.active_emergencies.saturating_sub(1);
            
            let event = EmergencyEvent {
                timestamp: current_timestamp(),
                event_type: EmergencyEventType::EmergencyEnded,
                severity: super::self_health_monitor::Severity::Low,
                description: "Emergency ended, entering recovery".to_string(),
                data: HashMap::new(),
            };
            self.emergency_events.push_back(event);
        }
    }

    /// Escalar emergencia
    pub fn escalate(&mut self) {
        match self.state {
            EmergencyState::Warning => {
                self.state = EmergencyState::PartialEmergency;
                self.push_event(EmergencyEventType::EmergencyEscalated, "Warning -> PartialEmergency".to_string());
            },
            EmergencyState::PartialEmergency => {
                self.state = EmergencyState::TotalEmergency;
                self.push_event(EmergencyEventType::EmergencyEscalated, "Partial -> TotalEmergency".to_string());
            },
            _ => {},
        }
    }

    /// De-escalar emergencia
    pub fn deescalate(&mut self) {
        match self.state {
            EmergencyState::TotalEmergency => {
                self.state = EmergencyState::PartialEmergency;
                self.push_event(EmergencyEventType::EmergencyDeescalated, "TotalEmergency -> PartialEmergency".to_string());
            },
            EmergencyState::PartialEmergency => {
                self.state = EmergencyState::Warning;
                self.push_event(EmergencyEventType::EmergencyDeescalated, "PartialEmergency -> Warning".to_string());
            },
            EmergencyState::Warning => {
                self.state = EmergencyState::Normal;
                self.active_emergency = None;
                self.push_event(EmergencyEventType::EmergencyEnded, "Warning -> Normal".to_string());
            },
            EmergencyState::Recovery => {
                self.state = EmergencyState::Normal;
                self.active_emergency = None;
                self.push_event(EmergencyEventType::EmergencyEnded, "Recovery -> Normal".to_string());
            },
            _ => {},
        }
    }

    fn push_event(&mut self, event_type: EmergencyEventType, description: String) {
        self.emergency_events.push_back(EmergencyEvent {
            timestamp: current_timestamp(),
            event_type,
            severity: super::self_health_monitor::Severity::Medium,
            description,
            data: HashMap::new(),
        });
    }

    /// Obtener estado actual
    pub fn get_state(&self) -> EmergencyState {
        self.state
    }

    /// Verificar si hay emergencia activa
    pub fn is_emergency(&self) -> bool {
        !matches!(self.state, EmergencyState::Normal | EmergencyState::Recovery)
    }

    // =========================================================================
    // EMERGENCY ACTIONS
    // =========================================================================

    /// Ejecutar acción de emergencia
    pub fn execute_action(&mut self, action: EmergencyActionType, target: String, reason: String) -> Result<EmergencyAction, EmergencyError> {
        let now = current_timestamp();
        let mut succeeded = true;
        let rollback_available = true;
        
        // Execute based on action type
        match action {
            EmergencyActionType::StopModule => {
                self.stop_module(&target)?;
            },
            EmergencyActionType::PauseModule => {
                self.pause_module(&target)?;
            },
            EmergencyActionType::RestartModule => {
                self.restart_module(&target)?;
            },
            EmergencyActionType::DisableEvolutions => {
                // Disable evolution across system
            },
            EmergencyActionType::EnableSafeMode => {
                // Enable safe mode
            },
            _ => {},
        }
        
        let emergency_action = EmergencyAction {
            id: self.stats.total_actions,
            action_type: action,
            target: target.clone(),
            reason,
            executed_at: now,
            succeeded,
            rollback_available,
        };
        
        self.action_history.push(emergency_action.clone());
        self.stats.total_actions += 1;
        if succeeded {
            self.stats.successful_actions += 1;
        }
        
        Ok(emergency_action)
    }

    fn stop_module(&mut self, name: &str) -> Result<(), EmergencyError> {
        if let Some(module) = self.monitored_modules.get_mut(name) {
            module.enabled = false;
            module.healthy = false;
            self.push_event(EmergencyEventType::ModuleStopped, format!("Module {} stopped", name));
        }
        Ok(())
    }

    fn pause_module(&mut self, name: &str) -> Result<(), EmergencyError> {
        if let Some(module) = self.monitored_modules.get_mut(name) {
            module.healthy = false;
            self.push_event(EmergencyEventType::ModuleStopped, format!("Module {} paused", name));
        }
        Ok(())
    }

    fn restart_module(&mut self, name: &str) -> Result<(), EmergencyError> {
        if let Some(module) = self.monitored_modules.get_mut(name) {
            module.healthy = true;
            module.last_heartbeat = current_timestamp();
            module.failures = 0;
            self.push_event(EmergencyEventType::ModuleRestarted, format!("Module {} restarted", name));
        }
        Ok(())
    }

    /// Creator kill switch - emergencia máxima
    pub fn creator_kill_switch(&mut self, reason: String) {
        self.declare_emergency(EmergencyType::CreatorKillSwitch, reason.clone());
        
        // Stop all non-essential modules
        for (name, module) in &mut self.monitored_modules {
            if !Self::is_essential_module(name) {
                module.enabled = false;
            }
        }
        
        // Execute emergency action
        let _ = self.execute_action(
            EmergencyActionType::FullReset,
            "system".to_string(),
            format!("Creator kill switch: {}", reason),
        );
    }

    fn is_essential_module(name: &str) -> bool {
        matches!(name, "self_health_monitor" | "emergency_controller" | "governance")
    }

    // =========================================================================
    // CIRCUIT BREAKERS
    // =========================================================================

    /// Registrar circuit breaker
    pub fn register_circuit_breaker(&mut self, name: String, threshold: u32) {
        let cb = CircuitBreaker {
            name: name.clone(),
            threshold,
            ..Default::default()
        };
        self.circuit_breakers.insert(name, cb);
    }

    /// Registrar failure en circuit breaker
    pub fn circuit_failure(&mut self, name: &str) {
        if let Some(cb) = self.circuit_breakers.get_mut(name) {
            cb.failure_count += 1;
            cb.last_failure = current_timestamp();
            
            if cb.failure_count >= cb.threshold && cb.state == CircuitState::Closed {
                cb.state = CircuitState::Open;
                self.stats.circuit_trips += 1;
                self.push_event(EmergencyEventType::CircuitBreakerOpened, format!("CB {} opened", name));
            }
        }
    }

    /// Verificar si circuit breaker permite request
    pub fn circuit_can_proceed(&self, name: &str) -> bool {
        if let Some(cb) = self.circuit_breakers.get(name) {
            match cb.state {
                CircuitState::Closed => true,
                CircuitState::Open => {
                    let elapsed = current_timestamp() - cb.last_failure;
                    elapsed > cb.half_open_after_ms
                },
                CircuitState::HalfOpen => true,
            }
        } else {
            true // No CB = allow
        }
    }

    /// Reset circuit breaker
    pub fn circuit_reset(&mut self, name: &str) {
        if let Some(cb) = self.circuit_breakers.get_mut(name) {
            cb.state = CircuitState::Closed;
            cb.failure_count = 0;
            self.push_event(EmergencyEventType::CircuitBreakerClosed, format!("CB {} closed", name));
        }
    }

    // =========================================================================
    // MODULE MANAGEMENT
    // =========================================================================

    /// Registrar módulo para monitoreo
    pub fn register_module(&mut self, name: String) {
        self.monitored_modules.insert(name.clone(), ModuleStatus {
            name,
            enabled: true,
            healthy: true,
            last_heartbeat: current_timestamp(),
            failures: 0,
        });
    }

    /// Heartbeat de módulo
    pub fn module_heartbeat(&mut self, name: &str) {
        if let Some(module) = self.monitored_modules.get_mut(name) {
            module.last_heartbeat = current_timestamp();
            if !module.healthy {
                module.healthy = true;
            }
        }
    }

    /// Reportar failure de módulo
    pub fn module_failure(&mut self, name: &str) {
        if let Some(module) = self.monitored_modules.get_mut(name) {
            module.failures += 1;
            if module.failures > 10 {
                module.healthy = false;
            }
        }
        
        // Also trigger circuit breaker
        self.circuit_failure(name);
    }

    /// Obtener estado de módulo
    pub fn get_module_status(&self, name: &str) -> Option<&ModuleStatus> {
        self.monitored_modules.get(name)
    }

    // =========================================================================
    // RECOVERY
    // =========================================================================

    /// Programar recovery
    pub fn schedule_recovery(&mut self, module: String, task_type: RecoveryTaskType) -> u64 {
        let id = self.stats.recoveries_completed + self.recovery_queue.len() as u64 + 1;
        
        self.recovery_queue.push(RecoveryTask {
            id,
            target_module: module,
            task_type,
            status: RecoveryStatus::Pending,
            created_at: current_timestamp(),
            completed_at: None,
        });
        
        self.push_event(EmergencyEventType::RecoveryStarted, format!("Recovery scheduled for {}", module));
        id
    }

    /// Procesar recovery queue
    pub fn process_recoveries(&mut self) {
        for task in &mut self.recovery_queue {
            if task.status == RecoveryStatus::Pending {
                task.status = RecoveryStatus::InProgress;
                // Simulate recovery
                task.status = RecoveryStatus::Completed;
                task.completed_at = Some(current_timestamp());
                self.stats.recoveries_completed += 1;
                self.push_event(EmergencyEventType::RecoveryCompleted, format!("Recovery {} done", task.id));
            }
        }
        
        // Clean completed tasks
        self.recovery_queue.retain(|t| t.status != RecoveryStatus::Completed);
    }

    // =========================================================================
    // ACCESSORS
    // =========================================================================

    /// Obtener historial de acciones
    pub fn get_action_history(&self) -> &[EmergencyAction] {
        &self.action_history
    }

    /// Obtener eventos de emergencia recientes
    pub fn get_recent_events(&self, count: usize) -> Vec<EmergencyEvent> {
        self.emergency_events.iter().rev().take(count).cloned().collect()
    }

    /// Obtener stats
    pub fn get_stats(&self) -> EmergencyStats {
        self.stats.clone()
    }

    /// Verificar si se puede operar dado el estado de emergencia
    pub fn can_operate(&self, operation: EmergencyOperation) -> bool {
        match self.state {
            EmergencyState::Normal => true,
            EmergencyState::Warning => true,
            EmergencyState::PartialEmergency => {
                matches!(operation, EmergencyOperation::Read | EmergencyOperation::Write)
            },
            EmergencyState::TotalEmergency => {
                matches!(operation, EmergencyOperation::Emergency | EmergencyOperation::Read)
            },
            EmergencyState::Recovery => true,
        }
    }
}

/// Operación categorizada para emergencia
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmergencyOperation {
    Read,
    Write,
    Evolution,
    Emergency,
}

#[derive(Debug, Clone)]
pub enum EmergencyError {
    ModuleNotFound(String),
    ActionFailed(String),
    InvalidState,
}

impl std::fmt::Display for EmergencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            Self::ActionFailed(msg) => write!(f, "Action failed: {}", msg),
            Self::InvalidState => write!(f, "Invalid state for operation"),
        }
    }
}

impl std::error::Error for EmergencyError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let controller = EmergencyController::new();
        assert_eq!(controller.get_state(), EmergencyState::Normal);
        assert!(!controller.is_emergency());
    }

    #[test]
    fn test_declare_emergency() {
        let mut controller = EmergencyController::new();
        let id = controller.declare_emergency(EmergencyType::ResourceExhaustion, "Out of memory".to_string());
        
        assert!(controller.is_emergency());
        assert_eq!(controller.get_state(), EmergencyState::PartialEmergency);
        assert!(controller.active_emergency.is_some());
    }

    #[test]
    fn test_end_emergency() {
        let mut controller = EmergencyController::new();
        controller.declare_emergency(EmergencyType::BehaviorAnomaly, "Test".to_string());
        controller.end_emergency();
        
        assert_eq!(controller.get_state(), EmergencyState::Recovery);
    }

    #[test]
    fn test_escalation() {
        let mut controller = EmergencyController::new();
        controller.declare_emergency(EmergencyType::BehaviorAnomaly, "Test".to_string());
        
        // Initial state should be PartialEmergency
        assert_eq!(controller.get_state(), EmergencyState::PartialEmergency);
        
        controller.escalate();
        assert_eq!(controller.get_state(), EmergencyState::TotalEmergency);
    }

    #[test]
    fn test_creator_kill_switch() {
        let mut controller = EmergencyController::new();
        
        controller.register_module("test_module".to_string());
        controller.register_module("self_health_monitor".to_string());
        
        controller.creator_kill_switch("Creator command".to_string());
        
        assert!(controller.is_emergency());
        assert_eq!(controller.get_state(), EmergencyState::TotalEmergency);
        
        // Essential modules should still be enabled
        let status = controller.get_module_status("self_health_monitor").unwrap();
        assert!(status.enabled);
        
        // Non-essential should be disabled
        let status = controller.get_module_status("test_module").unwrap();
        assert!(!status.enabled);
    }

    #[test]
    fn test_circuit_breaker() {
        let mut controller = EmergencyController::new();
        controller.register_circuit_breaker("test".to_string(), 3);
        
        // Failures should not trip until threshold
        controller.circuit_failure("test");
        controller.circuit_failure("test");
        assert!(controller.circuit_can_proceed("test"));
        
        // Third failure trips
        controller.circuit_failure("test");
        assert!(!controller.circuit_can_proceed("test"));
        
        // Reset
        controller.circuit_reset("test");
        assert!(controller.circuit_can_proceed("test"));
    }

    #[test]
    fn test_module_registration() {
        let mut controller = EmergencyController::new();
        controller.register_module("governance".to_string());
        controller.register_module("evolution".to_string());
        
        let status = controller.get_module_status("governance").unwrap();
        assert!(status.enabled);
        assert!(status.healthy);
    }

    #[test]
    fn test_module_heartbeat() {
        let mut controller = EmergencyController::new();
        controller.register_module("test".to_string());
        
        let before = controller.get_module_status("test").unwrap().last_heartbeat;
        controller.module_heartbeat("test");
        let after = controller.get_module_status("test").unwrap().last_heartbeat;
        
        assert!(after >= before);
    }

    #[test]
    fn test_recovery_queue() {
        let mut controller = EmergencyController::new();
        
        let task_id = controller.schedule_recovery("test_module".to_string(), RecoveryTaskType::Restart);
        assert!(task_id > 0);
        
        controller.process_recoveries();
        
        // Check recovery completed
        assert_eq!(controller.stats.recoveries_completed, 1);
    }

    #[test]
    fn test_can_operate() {
        let controller = EmergencyController::new();
        assert!(controller.can_operate(EmergencyOperation::Read));
        assert!(controller.can_operate(EmergencyOperation::Write));
        assert!(controller.can_operate(EmergencyOperation::Evolution));
        assert!(controller.can_operate(EmergencyOperation::Emergency));
        
        let mut emergency = EmergencyController::new();
        emergency.declare_emergency(EmergencyType::QuorumFailure, "Quorum lost".to_string());
        
        // In partial emergency, can't do evolution
        assert!(emergency.can_operate(EmergencyOperation::Read));
        assert!(emergency.can_operate(EmergencyOperation::Write));
        assert!(!emergency.can_operate(EmergencyOperation::Evolution));
    }

    #[test]
    fn test_deescalation() {
        let mut controller = EmergencyController::new();
        controller.declare_emergency(EmergencyType::SecurityBreach, "Breach detected".to_string());
        controller.escalate(); // Now TotalEmergency
        
        controller.deescalate();
        assert_eq!(controller.get_state(), EmergencyState::PartialEmergency);
        
        controller.deescalate();
        assert_eq!(controller.get_state(), EmergencyState::Warning);
        
        controller.deescalate();
        assert_eq!(controller.get_state(), EmergencyState::Normal);
    }
}