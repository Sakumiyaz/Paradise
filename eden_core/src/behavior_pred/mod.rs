//! # Behavior Prediction Module
//!
//! Sistema de predicción comportamental 100% original.
//! Sin dependencias de bibliotecas externas.
//!
//! ## Componentes
//!
//! 1. **TemporalPatternAnalyzer**: Análisis de patrones temporales
//! 2. **AnomalyDetector**: Detección de anomalías comportamentales
//! 3. **MarkovPredictor**: Predicción basada en modelos de Markov (HMM)
//! 4. **ScenarioSimulator**: Simulación de escenarios futuros
//! 5. **CausalInference**: Inferencia causal para predicción
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet, VecDeque};
// ============================================================================
// TEMPORAL PATTERN ANALYZER
// ============================================================================

/// Acción observada con timestamp
#[derive(Clone, Debug)]
pub struct TimedAction {
    pub action: String,
    pub timestamp: u64,
    pub duration: u64,
    pub context: String,
}

/// Patrón temporal detectado
#[derive(Clone, Debug)]
pub struct TemporalPattern {
    pub pattern_id: u64,
    pub sequence: Vec<String>,
    pub period: Option<u64>,
    pub frequency: f32,
    pub confidence: f32,
    pub first_seen: u64,
    pub last_seen: u64,
}

/// Análisis de patrón temporal
pub struct TemporalPatternAnalyzer {
    patterns: HashMap<String, Vec<TimedAction>>,
    detected_patterns: Vec<TemporalPattern>,
    window_size: usize,
    min_frequency: f32,
}

impl TemporalPatternAnalyzer {
    pub fn new(window_size: usize, min_frequency: f32) -> Self {
        Self {
            patterns: HashMap::new(),
            detected_patterns: Vec::new(),
            window_size,
            min_frequency,
        }
    }

    /// Registra una acción
    pub fn observe(&mut self, action: String, timestamp: u64, context: &str) {
        let timed = TimedAction {
            action: action.clone(),
            timestamp,
            duration: 0,
            context: context.to_string(),
        };

        self.patterns
            .entry(action)
            .or_insert_with(Vec::new)
            .push(timed);

        // Trim old actions if window exceeded
        self.prune_old_actions(timestamp);
    }

    fn prune_old_actions(&mut self, current_time: u64) {
        let cutoff = current_time.saturating_sub(self.window_size as u64);

        for actions in self.patterns.values_mut() {
            actions.retain(|a| a.timestamp >= cutoff);
        }
    }

    /// Detecta patrones periódicos
    pub fn detect_periodic_patterns(&self) -> Vec<TemporalPattern> {
        let mut patterns = Vec::new();
        let mut pattern_id = 0;

        for (action, actions) in &self.patterns {
            if actions.len() < 3 {
                continue;
            }

            // Calculate intervals between consecutive actions
            let mut intervals: Vec<u64> = Vec::new();
            for i in 1..actions.len() {
                intervals.push(actions[i].timestamp - actions[i - 1].timestamp);
            }

            // Check for periodicity
            if let Some(period) = self.detect_period(&intervals) {
                patterns.push(TemporalPattern {
                    pattern_id,
                    sequence: vec![action.clone()],
                    period: Some(period),
                    frequency: actions.len() as f32,
                    confidence: self.compute_periodicity_confidence(&intervals, period),
                    first_seen: actions.first().map(|a| a.timestamp).unwrap_or(0),
                    last_seen: actions.last().map(|a| a.timestamp).unwrap_or(0),
                });
                pattern_id += 1;
            }
        }

        patterns
    }

    fn detect_period(&self, intervals: &[u64]) -> Option<u64> {
        if intervals.is_empty() {
            return None;
        }

        let avg: u64 = intervals.iter().sum::<u64>() / intervals.len() as u64;
        let tolerance = avg / 4;

        // Check if most intervals are close to average
        let close_count = intervals
            .iter()
            .filter(|&&i| {
                let diff = if i > avg { i - avg } else { avg - i };
                diff <= tolerance
            })
            .count();

        if close_count >= intervals.len() / 2 {
            Some(avg)
        } else {
            None
        }
    }

    fn compute_periodicity_confidence(&self, intervals: &[u64], period: u64) -> f32 {
        if intervals.is_empty() {
            return 0.0;
        }

        let tolerance = period / 4;
        let close_count = intervals
            .iter()
            .filter(|&&i| {
                let diff = if i > period { i - period } else { period - i };
                diff <= tolerance
            })
            .count();

        close_count as f32 / intervals.len() as f32
    }

    /// Obtiene patrones detectados
    pub fn get_detected_patterns(&self) -> &[TemporalPattern] {
        &self.detected_patterns
    }
}

// ============================================================================
// ANOMALY DETECTOR
// ============================================================================

/// Resultado de detección de anomalía
#[derive(Clone, Debug)]
pub struct AnomalyResult {
    pub is_anomaly: bool,
    pub score: f32,
    pub deviation: f32,
    pub anomaly_type: AnomalyType,
    pub description: String,
}

/// Tipo de anomalía
#[derive(Clone, Debug, PartialEq)]
pub enum AnomalyType {
    /// Comportamiento estadísticamente atípico
    Statistical,
    /// Patrón inesperado
    PatternDeviation,
    /// Cambio brusco
    SuddenChange,
    /// Ausencia de patrón esperado
    MissingPattern,
    /// Comportamiento sin precedentes
    Novel,
}

/// Detector de anomalías comportamentales
pub struct AnomalyDetector {
    /// Modelo de comportamiento normal
    normal_behavior: HashMap<String, BehavioralProfile>,
    /// Threshold de anomalía
    threshold: f32,
    /// Ventana de contexto
    context_window: usize,
}

#[derive(Clone, Debug)]
pub struct BehavioralProfile {
    pub action: String,
    pub mean_interval: f64,
    pub std_deviation: f64,
    pub frequency: f64,
    pub typical_contexts: HashSet<String>,
}

impl AnomalyDetector {
    pub fn new(threshold: f32, context_window: usize) -> Self {
        Self {
            normal_behavior: HashMap::new(),
            threshold,
            context_window,
        }
    }

    /// Aprende comportamiento normal
    pub fn learn_normal_behavior(&mut self, actions: &[TimedAction]) {
        // Group by action type
        let mut action_groups: HashMap<String, Vec<&TimedAction>> = HashMap::new();
        for action in actions {
            action_groups
                .entry(action.action.clone())
                .or_default()
                .push(action);
        }

        for (action, group) in action_groups {
            if group.len() < 2 {
                continue;
            }

            // Calculate statistics
            let intervals: Vec<u64> = group
                .windows(2)
                .map(|w| w[1].timestamp - w[0].timestamp)
                .collect();

            let sum: u64 = intervals.iter().sum();
            let mean = sum as f64 / intervals.len() as f64;

            let variance: f64 = intervals
                .iter()
                .map(|&i| {
                    let diff = i as f64 - mean;
                    diff * diff
                })
                .sum::<f64>()
                / intervals.len() as f64;
            let std_dev = variance.sqrt();

            let mut contexts = HashSet::new();
            for act in &group {
                contexts.insert(act.context.clone());
            }

            self.normal_behavior.insert(
                action,
                BehavioralProfile {
                    action: String::new(),
                    mean_interval: mean,
                    std_deviation: std_dev,
                    frequency: group.len() as f64,
                    typical_contexts: contexts,
                },
            );
        }
    }

    /// Detecta anomalía en una acción
    pub fn detect(&self, action: &str, timestamp: u64, context: &str) -> AnomalyResult {
        if let Some(profile) = self.normal_behavior.get(action) {
            // Check if context is unusual
            let context_unusual = !profile.typical_contexts.contains(context);

            // Calculate z-score for timing
            let last_time = timestamp as f64; // Simplified
            let z_score = ((last_time - profile.mean_interval) / profile.std_deviation).abs();

            let is_anomaly = z_score > 2.0 || context_unusual;
            let score = (z_score / 2.0).min(1.0);

            AnomalyResult {
                is_anomaly,
                score: score as f32,
                deviation: z_score as f32,
                anomaly_type: if context_unusual {
                    AnomalyType::PatternDeviation
                } else if z_score > 3.0 {
                    AnomalyType::SuddenChange
                } else {
                    AnomalyType::Statistical
                },
                description: if is_anomaly {
                    format!(
                        "Action '{}' deviates from normal behavior (z={:.2})",
                        action, z_score
                    )
                } else {
                    format!("Action '{}' within normal parameters", action)
                },
            }
        } else {
            // New action not seen before
            AnomalyResult {
                is_anomaly: true,
                score: 1.0,
                deviation: 0.0,
                anomaly_type: AnomalyType::Novel,
                description: format!("Novel action '{}' not seen in training data", action),
            }
        }
    }
}

// ============================================================================
// HIDDEN MARKOV MODEL PREDICTOR
// ============================================================================

/// Estado oculto en HMM
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HiddenState {
    Normal,
    Anxious,
    Aggressive,
    Calm,
    Excited,
    Tired,
}

/// Observación en HMM
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Observation {
    Action(String),
    Silence,
    Verbal,
}

/// Transición de estado en HMM
#[derive(Clone, Debug)]
pub struct StateTransition {
    pub from: HiddenState,
    pub to: HiddenState,
    pub probability: f32,
}

/// Predicción de comportamiento
#[derive(Clone, Debug)]
pub struct BehaviorPrediction {
    pub predicted_state: HiddenState,
    pub confidence: f32,
    pub next_actions: Vec<(String, f32)>,
    pub reasoning: String,
}

/// Predictor HMM de comportamiento
pub struct MarkovBehaviorPredictor {
    /// Probabilidades iniciales de estado
    initial_probabilities: HashMap<HiddenState, f32>,
    /// Matriz de transición
    transition_matrix: HashMap<(HiddenState, HiddenState), f32>,
    /// Probabilidades de observación
    emission_probabilities: HashMap<(HiddenState, Observation), f32>,
    /// Estado actual estimado
    current_state: HiddenState,
    /// Secuencia de observaciones recientes
    recent_observations: VecDeque<Observation>,
}

impl MarkovBehaviorPredictor {
    pub fn new() -> Self {
        let mut predictor = Self {
            initial_probabilities: HashMap::new(),
            transition_matrix: HashMap::new(),
            emission_probabilities: HashMap::new(),
            current_state: HiddenState::Normal,
            recent_observations: VecDeque::new(),
        };

        predictor.initialize_default_model();
        predictor
    }

    fn initialize_default_model(&mut self) {
        // Initial probabilities
        self.initial_probabilities.insert(HiddenState::Normal, 0.4);
        self.initial_probabilities.insert(HiddenState::Calm, 0.3);
        self.initial_probabilities.insert(HiddenState::Anxious, 0.1);
        self.initial_probabilities
            .insert(HiddenState::Aggressive, 0.05);
        self.initial_probabilities.insert(HiddenState::Excited, 0.1);
        self.initial_probabilities.insert(HiddenState::Tired, 0.05);

        // Transition probabilities (simplified)
        // Normal -> Normal: 0.7, Normal -> Anxious: 0.1, etc.
        self.add_transition(HiddenState::Normal, HiddenState::Normal, 0.7);
        self.add_transition(HiddenState::Normal, HiddenState::Anxious, 0.1);
        self.add_transition(HiddenState::Normal, HiddenState::Calm, 0.2);

        self.add_transition(HiddenState::Calm, HiddenState::Calm, 0.6);
        self.add_transition(HiddenState::Calm, HiddenState::Normal, 0.3);
        self.add_transition(HiddenState::Calm, HiddenState::Tired, 0.1);

        self.add_transition(HiddenState::Anxious, HiddenState::Anxious, 0.5);
        self.add_transition(HiddenState::Anxious, HiddenState::Normal, 0.3);
        self.add_transition(HiddenState::Anxious, HiddenState::Aggressive, 0.2);

        self.add_transition(HiddenState::Aggressive, HiddenState::Aggressive, 0.4);
        self.add_transition(HiddenState::Aggressive, HiddenState::Anxious, 0.3);
        self.add_transition(HiddenState::Aggressive, HiddenState::Normal, 0.3);

        self.add_transition(HiddenState::Excited, HiddenState::Excited, 0.5);
        self.add_transition(HiddenState::Excited, HiddenState::Normal, 0.4);
        self.add_transition(HiddenState::Excited, HiddenState::Tired, 0.1);

        self.add_transition(HiddenState::Tired, HiddenState::Tired, 0.5);
        self.add_transition(HiddenState::Tired, HiddenState::Calm, 0.3);
        self.add_transition(HiddenState::Tired, HiddenState::Normal, 0.2);

        // Emission probabilities
        self.add_emission(
            HiddenState::Normal,
            Observation::Action("work".to_string()),
            0.4,
        );
        self.add_emission(
            HiddenState::Normal,
            Observation::Action("communicate".to_string()),
            0.3,
        );
        self.add_emission(HiddenState::Normal, Observation::Silence, 0.2);
        self.add_emission(HiddenState::Normal, Observation::Verbal, 0.1);

        self.add_emission(
            HiddenState::Anxious,
            Observation::Action("pace".to_string()),
            0.3,
        );
        self.add_emission(
            HiddenState::Anxious,
            Observation::Action("communicate".to_string()),
            0.3,
        );
        self.add_emission(HiddenState::Anxious, Observation::Verbal, 0.3);
        self.add_emission(HiddenState::Anxious, Observation::Silence, 0.1);

        self.add_emission(
            HiddenState::Aggressive,
            Observation::Action("attack".to_string()),
            0.4,
        );
        self.add_emission(HiddenState::Aggressive, Observation::Verbal, 0.4);
        self.add_emission(
            HiddenState::Aggressive,
            Observation::Action("communicate".to_string()),
            0.2,
        );

        self.add_emission(
            HiddenState::Calm,
            Observation::Action("work".to_string()),
            0.3,
        );
        self.add_emission(HiddenState::Calm, Observation::Silence, 0.4);
        self.add_emission(
            HiddenState::Calm,
            Observation::Action("rest".to_string()),
            0.3,
        );

        self.add_emission(
            HiddenState::Excited,
            Observation::Action("celebrate".to_string()),
            0.3,
        );
        self.add_emission(HiddenState::Excited, Observation::Verbal, 0.4);
        self.add_emission(
            HiddenState::Excited,
            Observation::Action("communicate".to_string()),
            0.3,
        );

        self.add_emission(
            HiddenState::Tired,
            Observation::Action("rest".to_string()),
            0.5,
        );
        self.add_emission(HiddenState::Tired, Observation::Silence, 0.4);
        self.add_emission(
            HiddenState::Tired,
            Observation::Action("work".to_string()),
            0.1,
        );
    }

    fn add_transition(&mut self, from: HiddenState, to: HiddenState, prob: f32) {
        self.transition_matrix.insert((from, to), prob);
    }

    fn add_emission(&mut self, state: HiddenState, obs: Observation, prob: f32) {
        self.emission_probabilities.insert((state, obs), prob);
    }

    /// Observa una acción y actualiza el modelo
    pub fn observe(&mut self, action: &str) {
        let obs = if action.is_empty() {
            Observation::Silence
        } else if self.is_verbal(action) {
            Observation::Verbal
        } else {
            Observation::Action(action.to_string())
        };

        self.recent_observations.push_back(obs.clone());
        if self.recent_observations.len() > 10 {
            self.recent_observations.pop_front();
        }

        // Update current state using Viterbi approximation
        self.update_state_viterbi(&obs);
    }

    fn is_verbal(&self, action: &str) -> bool {
        let verbal_actions = ["say", "speak", "talk", "tell", "ask", "respond", "reply"];
        verbal_actions
            .iter()
            .any(|v| action.to_lowercase().contains(v))
    }

    fn update_state_viterbi(&mut self, obs: &Observation) {
        // Simplified Viterbi: just pick most likely state given observation
        let mut best_state = self.current_state.clone();
        let mut best_prob = 0.0f32;

        for state in &[
            HiddenState::Normal,
            HiddenState::Calm,
            HiddenState::Anxious,
            HiddenState::Aggressive,
            HiddenState::Excited,
            HiddenState::Tired,
        ] {
            let trans_prob = self
                .transition_matrix
                .get(&(self.current_state.clone(), state.clone()))
                .copied()
                .unwrap_or(0.1);
            let emit_prob = self
                .emission_probabilities
                .get(&(state.clone(), obs.clone()))
                .copied()
                .unwrap_or(0.1);

            let prob = trans_prob * emit_prob;
            if prob > best_prob {
                best_prob = prob;
                best_state = state.clone();
            }
        }

        self.current_state = best_state;
    }

    /// Predice próximo comportamiento
    pub fn predict(&self) -> BehaviorPrediction {
        let next_actions;

        // Get actions with highest probability for current state
        let mut action_probs: Vec<(String, f32)> = Vec::new();

        for ((state, obs), prob) in &self.emission_probabilities {
            if *state == self.current_state {
                if let Observation::Action(action) = obs {
                    action_probs.push((action.clone(), *prob));
                }
            }
        }

        action_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        next_actions = action_probs.into_iter().take(3).collect();

        let reasoning = match &self.current_state {
            HiddenState::Normal => "Agent appears to be in a normal, balanced state".to_string(),
            HiddenState::Anxious => "Agent shows signs of anxiety or nervousness".to_string(),
            HiddenState::Aggressive => "Agent may become aggressive - use caution".to_string(),
            HiddenState::Calm => "Agent is calm and relaxed".to_string(),
            HiddenState::Excited => "Agent is excited or enthusiastic".to_string(),
            HiddenState::Tired => "Agent appears tired or fatigued".to_string(),
        };

        BehaviorPrediction {
            predicted_state: self.current_state.clone(),
            confidence: 0.7,
            next_actions,
            reasoning,
        }
    }

    /// Obtiene estado actual
    pub fn get_current_state(&self) -> &HiddenState {
        &self.current_state
    }
}

impl Default for MarkovBehaviorPredictor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SCENARIO SIMULATOR
// ============================================================================

/// Escenario simulado
#[derive(Clone, Debug)]
pub struct SimulatedScenario {
    pub id: u64,
    pub actions: Vec<String>,
    pub probability: f32,
    pub outcome: ScenarioOutcome,
    pub reasoning: String,
}

/// Resultado de escenario
#[derive(Clone, Debug)]
pub enum ScenarioOutcome {
    Success,
    Failure,
    Uncertain,
    Neutral,
}

/// Simulador de escenarios futuros
pub struct ScenarioSimulator {
    predictor: MarkovBehaviorPredictor,
    max_depth: usize,
}

impl ScenarioSimulator {
    pub fn new(max_depth: usize) -> Self {
        Self {
            predictor: MarkovBehaviorPredictor::new(),
            max_depth,
        }
    }

    /// Simula posibles escenarios futuros
    pub fn simulate(&self, current_state: &HiddenState, action: &str) -> Vec<SimulatedScenario> {
        let mut scenarios = Vec::new();
        let mut scenario_id = 0;

        // Simulate action outcomes
        let outcomes = self.simulate_outcomes(action);

        for outcome in outcomes {
            let probability = match outcome {
                ScenarioOutcome::Success => 0.6,
                ScenarioOutcome::Failure => 0.2,
                ScenarioOutcome::Uncertain => 0.1,
                ScenarioOutcome::Neutral => 0.1,
            };

            scenarios.push(SimulatedScenario {
                id: scenario_id,
                actions: vec![action.to_string()],
                probability,
                outcome,
                reasoning: format!(
                    "Simulated outcome for action '{}' in state {:?}",
                    action, current_state
                ),
            });
            scenario_id += 1;
        }

        scenarios
    }

    fn simulate_outcomes(&self, action: &str) -> Vec<ScenarioOutcome> {
        // Simplified outcome simulation
        if action.contains("work") || action.contains("build") {
            vec![
                ScenarioOutcome::Success,
                ScenarioOutcome::Failure,
                ScenarioOutcome::Neutral,
            ]
        } else if action.contains("attack") || action.contains("harm") {
            vec![
                ScenarioOutcome::Failure,
                ScenarioOutcome::Success,
                ScenarioOutcome::Uncertain,
            ]
        } else if action.contains("communicate") || action.contains("talk") {
            vec![ScenarioOutcome::Neutral, ScenarioOutcome::Success]
        } else {
            vec![ScenarioOutcome::Uncertain, ScenarioOutcome::Neutral]
        }
    }
}

// ============================================================================
// CAUSAL INFERENCE
// ============================================================================

/// Relación causal
#[derive(Clone, Debug)]
pub struct CausalRelation {
    pub cause: String,
    pub effect: String,
    pub strength: f32,
    pub mechanism: String,
}

/// Inferencia causal
pub struct CausalInference {
    causal_graph: HashMap<String, Vec<CausalRelation>>,
}

impl CausalInference {
    pub fn new() -> Self {
        Self {
            causal_graph: HashMap::new(),
        }
    }

    /// Aprende relación causal
    pub fn learn_causal_relation(&mut self, cause: &str, effect: &str, strength: f32) {
        let relation = CausalRelation {
            cause: cause.to_string(),
            effect: effect.to_string(),
            strength,
            mechanism: format!("{} causes {} with strength {:.2}", cause, effect, strength),
        };

        self.causal_graph
            .entry(cause.to_string())
            .or_default()
            .push(relation);
    }

    /// Infiere efectos de una acción
    pub fn infer_effects(&self, cause: &str) -> Vec<&CausalRelation> {
        match self.causal_graph.get(cause) {
            Some(v) => v.iter().collect(),
            None => vec![],
        }
    }

    /// Predice basado en relaciones causales
    pub fn predict_causally(&self, observed_action: &str) -> Vec<String> {
        let mut predictions = Vec::new();

        // Find actions that typically precede this one
        for (cause, effects) in &self.causal_graph {
            for effect in effects {
                if effect.effect == observed_action {
                    predictions.push(format!(
                        "{} may occur because of {}",
                        observed_action, cause
                    ));
                }
            }
        }

        // Also predict what this action might cause
        if let Some(relations) = self.causal_graph.get(observed_action) {
            for relation in relations {
                predictions.push(format!(
                    "This action may cause {} (strength: {:.2})",
                    relation.effect, relation.strength
                ));
            }
        }

        predictions
    }
}

impl Default for CausalInference {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Analiza patrones temporales
pub fn analyze_temporal_patterns(analyzer: &TemporalPatternAnalyzer) -> Vec<TemporalPattern> {
    analyzer.detect_periodic_patterns()
}

/// Detecta anomalías
pub fn detect_anomalies(
    detector: &AnomalyDetector,
    action: &str,
    timestamp: u64,
    context: &str,
) -> AnomalyResult {
    detector.detect(action, timestamp, context)
}

/// Predice comportamiento
pub fn predict_behavior(predictor: &MarkovBehaviorPredictor) -> BehaviorPrediction {
    predictor.predict()
}

/// Simula escenarios
pub fn simulate_scenarios(
    simulator: &ScenarioSimulator,
    state: &HiddenState,
    action: &str,
) -> Vec<SimulatedScenario> {
    simulator.simulate(state, action)
}
