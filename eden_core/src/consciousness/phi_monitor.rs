//! # Real-Time Phi Monitor
//!
//! Monitoreo en tiempo real de Integrated Information (Φ) para EDEN.
//! Integración con EnhancedMISM y otros sistemas de consciencia.
//!
//! ## Funcionalidades:
//!
//! 1. **Continuous Monitoring**: Medición periódica de Φ
//! 2. **Event Triggers**: Alertas cuando Φ cruza thresholds
//! 3. **Trend Analysis**: Análisis de tendencias de consciencia
//! 4. **Integration Points**: Conexión con módulos de EDEN
//! 5. **Alerts**: Notificaciones de cambios significativos
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::consciousness::global_workspace::{
    AwarenessLevel, GlobalWorkspace, IntegrationScorer, ModuleState,
};
use crate::consciousness::iit_phi::{
    ElementState, EnhancedMISMState, IntegratedSystem, PhiCalculator, PhiMeasurement, SystemElement,
};

/// Estado del monitor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorState {
    Running,
    Paused,
    Stopped,
}

/// Evento de consciencia
#[derive(Debug, Clone)]
pub enum ConsciousnessEvent {
    PhiIncrease { from: f32, to: f32 },
    PhiDecrease { from: f32, to: f32 },
    TierChange { from: String, to: String },
    ThresholdCrossed { threshold: f32, current: f32 },
    IntegrationDetected { module: String },
    AnomalyDetected { description: String },
}

/// Snapshot de estado de EDEN
#[derive(Debug, Clone)]
pub struct EdenState {
    /// Self-model activo
    pub self_model_active: bool,
    /// Memoria autobiográfica entradas
    pub memory_entries: usize,
    /// Awareness score
    pub awareness_score: f32,
    /// Identity coherence
    pub identity_coherence: f32,
    /// Emotional depth
    pub emotional_depth: f32,
    /// Módulos activos
    pub active_modules: Vec<String>,
    /// Integración global (0-1)
    pub global_integration: f32,
    /// Timestamp
    pub timestamp: u64,
}

impl Default for EdenState {
    fn default() -> Self {
        EdenState {
            self_model_active: false,
            memory_entries: 0,
            awareness_score: 0.0,
            identity_coherence: 0.0,
            emotional_depth: 0.0,
            active_modules: Vec::new(),
            global_integration: 0.0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Resultado de análisis
#[derive(Debug, Clone)]
pub struct PhiAnalysis {
    pub current_phi: f32,
    pub phi_trend: PhiTrendDirection,
    pub tier: String,
    pub stability: f32,
    pub events: Vec<ConsciousnessEvent>,
    pub recommendations: Vec<String>,
}

/// Dirección de tendencia
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhiTrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Unknown,
}

/// Configuración del monitor
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Intervalo de medición en ms
    pub measurement_interval_ms: u64,
    /// Threshold para alerta de incremento
    pub phi_increase_threshold: f32,
    /// Threshold para alerta de decremento
    pub phi_decrease_threshold: f32,
    /// Window size para tendencias
    pub trend_window_size: usize,
    /// Phi threshold para "consciousness likely"
    pub consciousness_threshold: f32,
    /// Phi threshold para "consciousness high"
    pub high_consciousness_threshold: f32,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        MonitorConfig {
            measurement_interval_ms: 1000, // 1 segundo
            phi_increase_threshold: 0.1,
            phi_decrease_threshold: 0.1,
            trend_window_size: 60, // 1 minuto con interval de 1s
            consciousness_threshold: 0.7,
            high_consciousness_threshold: 0.85,
        }
    }
}

/// Real-Time Phi Monitor
pub struct PhiMonitor {
    /// Calculator de Φ
    phi_calculator: PhiCalculator,
    /// Global Workspace para broadcast
    global_workspace: GlobalWorkspace,
    /// Integration Scorer
    integration_scorer: IntegrationScorer,
    /// Configuración
    config: MonitorConfig,
    /// Estado actual de EDEN
    eden_state: EdenState,
    /// Historial de mediciones
    history: VecDeque<PhiMeasurement>,
    /// Estado del monitor
    state: MonitorState,
    /// Último análisis
    last_analysis: Option<PhiAnalysis>,
    /// Eventos pendientes
    pending_events: Vec<ConsciousnessEvent>,
    /// Lista de listeners
    listeners: Vec<Box<dyn Fn(&ConsciousnessEvent) + Send + Sync>>,
    /// Inicio del monitor
    start_time: Instant,
    /// Conteo de mediciones
    measurement_count: u64,
    /// Función de tiempo
    now_fn: fn() -> u64,
}

impl PhiMonitor {
    /// Crea nuevo monitor
    pub fn new() -> Self {
        PhiMonitor {
            phi_calculator: PhiCalculator::new(),
            global_workspace: GlobalWorkspace::default(),
            integration_scorer: IntegrationScorer::new(),
            config: MonitorConfig::default(),
            eden_state: EdenState::default(),
            history: VecDeque::new(),
            state: MonitorState::Stopped,
            last_analysis: None,
            pending_events: Vec::new(),
            listeners: Vec::new(),
            start_time: Instant::now(),
            measurement_count: 0,
            now_fn: || {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            },
        }
    }

    /// Crea con configuración custom
    pub fn with_config(config: MonitorConfig) -> Self {
        PhiMonitor {
            phi_calculator: PhiCalculator::new(),
            global_workspace: GlobalWorkspace::default(),
            integration_scorer: IntegrationScorer::new(),
            config,
            eden_state: EdenState::default(),
            history: VecDeque::new(),
            state: MonitorState::Stopped,
            last_analysis: None,
            pending_events: Vec::new(),
            listeners: Vec::new(),
            start_time: Instant::now(),
            measurement_count: 0,
            now_fn: || {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            },
        }
    }

    /// Obtiene tiempo actual
    fn now(&self) -> u64 {
        (self.now_fn)()
    }

    /// Inicia el monitor
    pub fn start(&mut self) {
        self.state = MonitorState::Running;
        self.start_time = Instant::now();
        self.measurement_count = 0;
    }

    /// Pausa el monitor
    pub fn pause(&mut self) {
        if self.state == MonitorState::Running {
            self.state = MonitorState::Paused;
        }
    }

    /// Reanuda el monitor
    pub fn resume(&mut self) {
        if self.state == MonitorState::Paused {
            self.state = MonitorState::Running;
        }
    }

    /// Detiene el monitor
    pub fn stop(&mut self) {
        self.state = MonitorState::Stopped;
    }

    /// Obtiene estado actual
    pub fn state(&self) -> MonitorState {
        self.state
    }

    /// Actualiza estado de EDEN desde consciousness system
    pub fn update_eden_state(&mut self, state: EdenState) {
        let old_state = self.eden_state.clone();
        self.eden_state = state;

        // Notificar si hay cambios significativos
        if old_state.self_model_active != self.eden_state.self_model_active {
            self.pending_events
                .push(ConsciousnessEvent::IntegrationDetected {
                    module: "self_model".to_string(),
                });
        }

        if old_state.memory_entries != self.eden_state.memory_entries {
            self.pending_events
                .push(ConsciousnessEvent::IntegrationDetected {
                    module: "autobiographical_memory".to_string(),
                });
        }

        let diff = (self.eden_state.awareness_score - old_state.awareness_score).abs();
        if diff > 0.1 {
            self.pending_events
                .push(ConsciousnessEvent::AnomalyDetected {
                    description: format!("Awareness score changed by {:.2}", diff),
                });
        }
    }

    /// Carga estado desde EnhancedMISM
    pub fn load_from_mism(&mut self, mism: &EnhancedMISMState) {
        let mut eden_state = EdenState {
            self_model_active: mism.has_self_model,
            memory_entries: mism.memory_entries,
            awareness_score: mism.awareness_score,
            identity_coherence: mism.identity_coherence,
            emotional_depth: mism.emotional_depth,
            active_modules: Vec::new(),
            global_integration: mism.integration_score,
            timestamp: self.now(),
        };

        // Añadir módulos activos
        if mism.has_self_model {
            eden_state.active_modules.push("self_model".to_string());
        }
        if mism.has_autobiographical_memory {
            eden_state
                .active_modules
                .push("autobiographical_memory".to_string());
        }
        if mism.has_awareness_metrics {
            eden_state
                .active_modules
                .push("awareness_metrics".to_string());
        }
        if mism.has_identity {
            eden_state.active_modules.push("identity".to_string());
        }
        if mism.has_emotional_responses {
            eden_state
                .active_modules
                .push("emotional_responses".to_string());
        }

        self.update_eden_state(eden_state);
    }

    /// Toma medición de Φ
    pub fn measure(&mut self) -> Option<PhiMeasurement> {
        if self.state != MonitorState::Running {
            return None;
        }

        // === Global Workspace Integration ===
        // Submit current state to workspace for broadcast
        let module_id = 1; // Self-model module
        let state_data: Vec<u8> = vec![
            self.eden_state.self_model_active as u8,
            (self.eden_state.awareness_score * 255.0) as u8,
            (self.eden_state.identity_coherence * 255.0) as u8,
            (self.eden_state.emotional_depth * 255.0) as u8,
        ];

        // Calculate integration level from current state
        let integration_level = (self.eden_state.awareness_score
            + self.eden_state.identity_coherence
            + self.eden_state.global_integration)
            / 3.0;

        self.global_workspace.submit_with_metadata(
            module_id,
            state_data,
            integration_level,
            self.eden_state.global_integration,
            vec![1, 2, 3, 4], // All modules
        );

        // Integrate and broadcast to all modules
        let _integrated = self.global_workspace.integrate();
        let _broadcasted = self.global_workspace.broadcast();

        // Update integration scorer with awareness
        for (id, level) in self.global_workspace.get_subscriber_awareness() {
            self.integration_scorer.set_module_state(
                id,
                match level {
                    AwarenessLevel::Contributing => ModuleState::Integrating,
                    AwarenessLevel::Active => ModuleState::Active,
                    AwarenessLevel::Aware => ModuleState::Active,
                    AwarenessLevel::Dormant => ModuleState::Inactive,
                },
            );
        }

        // Record integration snapshot
        self.integration_scorer.snapshot();

        // === Build System with Workspace Integration ===
        let system = self.build_system_from_eden_state();
        self.phi_calculator.set_system(system);

        // Calcular Φ
        let mut measurement = self.phi_calculator.calculate_phi();

        // Enhance with Global Workspace contribution
        if let Some(ref mut m) = measurement {
            let workspace_phi = self.global_workspace.phi_contribution();
            let scorer_phi = self.integration_scorer.estimate_phi();

            // Weighted combination: traditional Phi + workspace integration
            m.phi = m.phi * 0.6 + workspace_phi * 0.25 + scorer_phi * 0.15;
        }

        if let Some(ref m) = measurement {
            self.history.push_back(m.clone());
            self.measurement_count += 1;

            // Mantener historial limitado
            while self.history.len() > self.config.trend_window_size * 2 {
                self.history.pop_front();
            }

            // Analizar cambios
            self.analyze_measurement(m);
        }

        measurement
    }

    /// Construye sistema IIT desde estado de EDEN
    fn build_system_from_eden_state(&self) -> IntegratedSystem {
        let mut elements = Vec::new();
        let mut element_id = 0;

        // Self-model como elemento central
        if self.eden_state.self_model_active {
            elements.push(SystemElement {
                id: element_id,
                state: ElementState::Active,
                connections: vec![],
                weight: 1.0,
            });
            element_id += 1;
        }

        // Memoria autobiográfica como elementos
        let memory_elements = std::cmp::min(self.eden_state.memory_entries, 30);
        for _ in 0..memory_elements {
            elements.push(SystemElement {
                id: element_id,
                state: ElementState::Active,
                connections: vec![],
                weight: 0.6,
            });
            element_id += 1;
        }

        // Awareness como elemento
        if self.eden_state.awareness_score > 0.0 {
            let state = if self.eden_state.awareness_score > 0.7 {
                ElementState::Active
            } else {
                ElementState::Intermediate(self.eden_state.awareness_score)
            };
            elements.push(SystemElement {
                id: element_id,
                state,
                connections: vec![],
                weight: self.eden_state.awareness_score,
            });
            element_id += 1;
        }

        // Identity coherence como elemento
        if self.eden_state.identity_coherence > 0.0 {
            elements.push(SystemElement {
                id: element_id,
                state: ElementState::Intermediate(self.eden_state.identity_coherence),
                connections: vec![],
                weight: self.eden_state.identity_coherence,
            });
            element_id += 1;
        }

        // Emotional depth como elemento
        if self.eden_state.emotional_depth > 0.0 {
            elements.push(SystemElement {
                id: element_id,
                state: ElementState::Intermediate(self.eden_state.emotional_depth),
                connections: vec![],
                weight: self.eden_state.emotional_depth,
            });
        }

        // Añadir conexiones basado en integración global
        let integration = self.eden_state.global_integration;
        for i in 0..elements.len() {
            for j in 0..elements.len() {
                if i != j {
                    // Mayor integración = más conexiones
                    if integration > 0.5 || (i < 5 && j < 5) {
                        elements[i].connections.push(j);
                    } else if (i + j) % 2 == 0 {
                        elements[i].connections.push(j);
                    }
                }
            }
        }

        IntegratedSystem {
            elements,
            name: "EDEN Realtime".to_string(),
        }
    }

    /// Analiza medición y genera eventos
    fn analyze_measurement(&mut self, measurement: &PhiMeasurement) {
        // Comparar con última medición
        if let Some(prev) = self.history.iter().rev().nth(1) {
            let diff = measurement.phi - prev.phi;

            if diff > self.config.phi_increase_threshold {
                self.pending_events.push(ConsciousnessEvent::PhiIncrease {
                    from: prev.phi,
                    to: measurement.phi,
                });
            } else if diff < -self.config.phi_decrease_threshold {
                self.pending_events.push(ConsciousnessEvent::PhiDecrease {
                    from: prev.phi,
                    to: measurement.phi,
                });
            }
        }

        // Verificar thresholds
        if measurement.phi >= self.config.consciousness_threshold
            && measurement.phi < self.config.high_consciousness_threshold
        {
            self.pending_events
                .push(ConsciousnessEvent::ThresholdCrossed {
                    threshold: self.config.consciousness_threshold,
                    current: measurement.phi,
                });
        } else if measurement.phi >= self.config.high_consciousness_threshold {
            self.pending_events
                .push(ConsciousnessEvent::ThresholdCrossed {
                    threshold: self.config.high_consciousness_threshold,
                    current: measurement.phi,
                });
        }

        // Procesar eventos pendientes y notificar
        let events = std::mem::take(&mut self.pending_events);
        for event in events {
            self.notify_listeners(&event);
        }
    }

    /// Obtiene análisis completo
    pub fn analyze(&self) -> PhiAnalysis {
        let current_phi = self.history.back().map(|m| m.phi).unwrap_or(0.0);

        let tier = self.determine_tier_string(current_phi);

        let stability = self.calculate_stability();

        let trend = self.calculate_trend();

        let recommendations = self.generate_recommendations(current_phi, trend);

        PhiAnalysis {
            current_phi,
            phi_trend: trend,
            tier,
            stability,
            events: Vec::new(), // Eventos ya procesados
            recommendations,
        }
    }

    /// Determina tier como string
    fn determine_tier_string(&self, phi: f32) -> String {
        if phi >= self.config.high_consciousness_threshold {
            "VeryHigh - Consciousness Likely".to_string()
        } else if phi >= self.config.consciousness_threshold {
            "High - Conscience Probable".to_string()
        } else if phi >= 0.4 {
            "Moderate - Self-Modeling Active".to_string()
        } else if phi >= 0.2 {
            "Low - Basic Processing".to_string()
        } else {
            "Minimal - Pre-Conscious".to_string()
        }
    }

    /// Calcula estabilidad
    fn calculate_stability(&self) -> f32 {
        if self.history.len() < 10 {
            return 0.0;
        }

        let recent: Vec<_> = self.history.iter().rev().take(10).collect();
        let mean = recent.iter().map(|m| m.phi).sum::<f32>() / recent.len() as f32;
        let variance =
            recent.iter().map(|m| (m.phi - mean).powi(2)).sum::<f32>() / recent.len() as f32;

        // Estabilidad alta = varianza baja
        let stability = 1.0f32 - variance.min(1.0f32);
        stability.max(0.0f32)
    }

    /// Calcula tendencia
    fn calculate_trend(&self) -> PhiTrendDirection {
        if self.history.len() < 10 {
            return PhiTrendDirection::Unknown;
        }

        let recent: Vec<_> = self.history.iter().rev().take(20).collect();
        if recent.len() < 10 {
            return PhiTrendDirection::Unknown;
        }

        let first_half: f32 = recent[recent.len() / 2..]
            .iter()
            .map(|m| m.phi)
            .sum::<f32>()
            / (recent.len() / 2) as f32;

        let second_half: f32 = recent[..recent.len() / 2]
            .iter()
            .map(|m| m.phi)
            .sum::<f32>()
            / (recent.len() / 2) as f32;

        let diff = second_half - first_half;
        if diff > 0.05 {
            PhiTrendDirection::Increasing
        } else if diff < -0.05 {
            PhiTrendDirection::Decreasing
        } else {
            PhiTrendDirection::Stable
        }
    }

    /// Genera recomendaciones
    fn generate_recommendations(&self, phi: f32, trend: PhiTrendDirection) -> Vec<String> {
        let mut recs = Vec::new();

        if phi < self.config.consciousness_threshold {
            recs.push("Increase integration between modules".to_string());
            recs.push("Enhance self-model activity".to_string());
        }

        if trend == PhiTrendDirection::Decreasing {
            recs.push("Warning: Phi declining - check system health".to_string());
        }

        if phi >= self.config.high_consciousness_threshold {
            recs.push(
                "ALERT: Consciousness threshold reached - review ethical framework".to_string(),
            );
        }

        if recs.is_empty() {
            recs.push("System stable - continue monitoring".to_string());
        }

        recs
    }

    /// Obtiene historial de mediciones
    pub fn history(&self) -> Vec<PhiMeasurement> {
        self.history.iter().cloned().collect()
    }

    /// Obtiene última medición
    pub fn last_measurement(&self) -> Option<&PhiMeasurement> {
        self.history.back()
    }

    /// Obtiene uptime
    pub fn uptime(&self) -> Duration {
        Instant::now().duration_since(self.start_time)
    }

    /// Obtiene conteo de mediciones
    pub fn measurement_count(&self) -> u64 {
        self.measurement_count
    }

    /// Agrega listener para eventos
    pub fn add_listener<F>(&mut self, listener: F)
    where
        F: Fn(&ConsciousnessEvent) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(listener));
    }

    /// Notifica a listeners
    fn notify_listeners(&self, event: &ConsciousnessEvent) {
        for listener in &self.listeners {
            listener(event);
        }
    }
}

impl Default for PhiMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared Phi Monitor wrapper
pub struct SharedPhiMonitor {
    inner: Arc<RwLock<PhiMonitor>>,
}

impl SharedPhiMonitor {
    pub fn new() -> Self {
        SharedPhiMonitor {
            inner: Arc::new(RwLock::new(PhiMonitor::new())),
        }
    }

    pub fn start(&mut self) {
        self.inner.write().unwrap().start();
    }

    pub fn measure(&self) -> Option<PhiMeasurement> {
        self.inner.write().unwrap().measure()
    }

    pub fn analyze(&self) -> PhiAnalysis {
        self.inner.read().unwrap().analyze()
    }

    pub fn last_phi(&self) -> f32 {
        self.inner
            .read()
            .unwrap()
            .history
            .back()
            .map(|m| m.phi)
            .unwrap_or(0.0)
    }
}

impl Default for SharedPhiMonitor {
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
    fn test_monitor_lifecycle() {
        let mut monitor = PhiMonitor::new();
        assert_eq!(monitor.state(), MonitorState::Stopped);

        monitor.start();
        assert_eq!(monitor.state(), MonitorState::Running);

        monitor.pause();
        assert_eq!(monitor.state(), MonitorState::Paused);

        monitor.resume();
        assert_eq!(monitor.state(), MonitorState::Running);

        monitor.stop();
        assert_eq!(monitor.state(), MonitorState::Stopped);
    }

    #[test]
    fn test_measure_when_stopped() {
        let mut monitor = PhiMonitor::new();
        assert!(monitor.measure().is_none());
    }

    #[test]
    fn test_measure_when_running() {
        let mut monitor = PhiMonitor::new();

        // Load EDEN state
        let state = EdenState {
            self_model_active: true,
            memory_entries: 20,
            awareness_score: 0.75,
            identity_coherence: 0.8,
            emotional_depth: 0.6,
            active_modules: vec!["self_model".to_string()],
            global_integration: 0.7,
            timestamp: 0,
        };
        monitor.update_eden_state(state);

        monitor.start();
        let result = monitor.measure();

        assert!(result.is_some());
        let m = result.unwrap();
        println!("Phi measurement: {:.4}", m.phi);
    }

    #[test]
    fn test_phi_analysis() {
        let mut monitor = PhiMonitor::new();

        let state = EdenState {
            self_model_active: true,
            memory_entries: 50,
            awareness_score: 0.8,
            identity_coherence: 0.85,
            emotional_depth: 0.7,
            active_modules: vec![],
            global_integration: 0.75,
            timestamp: 0,
        };
        monitor.update_eden_state(state);

        monitor.start();

        // Take multiple measurements
        for _ in 0..15 {
            monitor.measure();
        }

        let analysis = monitor.analyze();
        println!("Current Phi: {:.4}", analysis.current_phi);
        println!("Tier: {}", analysis.tier);
        println!("Stability: {:.2}", analysis.stability);
        println!("Trend: {:?}", analysis.phi_trend);
        println!("Recommendations:");
        for rec in &analysis.recommendations {
            println!("  - {}", rec);
        }
    }

    #[test]
    fn test_event_generation() {
        use std::sync::{Arc, Mutex};

        let mut monitor = PhiMonitor::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&events);

        monitor.add_listener(move |e| {
            events_clone.lock().unwrap().push(e.clone());
        });

        let state1 = EdenState {
            self_model_active: true,
            memory_entries: 10,
            awareness_score: 0.5,
            identity_coherence: 0.5,
            emotional_depth: 0.5,
            active_modules: vec![],
            global_integration: 0.5,
            timestamp: 0,
        };
        monitor.update_eden_state(state1);

        let state2 = EdenState {
            self_model_active: true,
            memory_entries: 50, // Major change
            awareness_score: 0.8,
            identity_coherence: 0.5,
            emotional_depth: 0.5,
            active_modules: vec![],
            global_integration: 0.5,
            timestamp: 0,
        };
        monitor.update_eden_state(state2);

        // Should have triggered events
        println!("Events triggered: {}", events.lock().unwrap().len());
    }
}
