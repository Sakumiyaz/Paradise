//! # Prediction - Action Prediction Engine
//!
//! Sistema de predicción de acciones de otros agentes.
//! 100% original, sin dependencias.
//!
//! ## Algoritmos
//!
//! 1. **Markov Model**: Predicción basada en cadenas de Markov
//! 2. **Behavioral Pattern**: Patrones comportamentales detectados
//! 3. **Action Distribution**: Distribución probabilística de acciones
//!
//! ## Conceptos
//!
//! - ActionPrediction: Predicción individual de acción
//! - ActionProbabilities: Distribución de probabilidades
//! - BehavioralPattern: Patrón comportamental detectado
//! - MarkovModel: Modelo de transiciones entre acciones
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::theory_of_mind::{AgentId, TimePoint};
use std::collections::{HashMap, HashSet};

// ============================================================================
// TIPOS BASE
// ============================================================================

/// Predicción de una acción específica
#[derive(Clone, Debug)]
pub struct ActionPrediction {
    /// Acción predicha
    pub action: String,
    /// Probabilidad de esta acción
    pub probability: f32,
    /// Confianza en la predicción
    pub confidence: f32,
    /// Tiempo estimado hasta la acción
    pub expected_time: Option<u64>,
    /// Contexto de la predicción
    pub context: String,
}

/// Distribución de probabilidades sobre acciones
#[derive(Clone, Debug)]
pub struct ActionProbabilities {
    /// Acciones posibles
    pub actions: Vec<(String, f32)>,
    /// Entropía de la distribución (incertidumbre)
    pub entropy: f32,
    ///timestamp
    pub timestamp: TimePoint,
}

impl ActionProbabilities {
    pub fn most_likely(&self) -> Option<&(String, f32)> {
        self.actions
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    pub fn entropy(&self) -> f32 {
        // Shannon entropy
        let mut entropy = 0.0f32;
        for (_, prob) in &self.actions {
            if *prob > 0.0 {
                entropy -= prob * prob.log2();
            }
        }
        entropy
    }
}

/// Patrón comportamental detectado
#[derive(Clone, Debug)]
pub struct BehavioralPattern {
    /// ID del patrón
    pub pattern_id: u64,
    /// Secuencia de acciones
    pub sequence: Vec<String>,
    /// Frecuencia de ocurrencia
    pub frequency: f32,
    /// Última vez que se observó
    pub last_observed: TimePoint,
    /// Contexto donde ocurre
    pub context: String,
    /// ¿Es un patrón nuevo?
    pub is_novel: bool,
}

/// Contexto para predicción
#[derive(Clone, Debug)]
pub struct PredictionContext {
    /// Ambiente actual
    pub environment: EnvironmentDescription,
    /// Objetivos del agente (si se conocen)
    pub goals: Vec<String>,
    /// Recursos disponibles
    pub resources: Vec<String>,
    /// Restricciones
    pub constraints: Vec<String>,
    /// Otros agentes presentes
    pub other_agents: Vec<AgentId>,
}

/// Descripción del ambiente
#[derive(Clone, Debug)]
pub struct EnvironmentDescription {
    pub location: String,
    pub objects: Vec<String>,
    pub social_setting: String,
}

// ============================================================================
// MARKOV MODEL
// ============================================================================

/// Modelo de Markov para predicción de acciones
pub struct MarkovModel {
    /// Transiciones: (estado_actual, acción) -> (siguiente_estado, count)
    transitions: HashMap<(String, String), (String, u32)>,
    /// Estados únicos
    states: HashSet<String>,
    /// Acciones únicas
    actions: HashSet<String>,
    /// Distribución inicial
    initial_distribution: HashMap<String, f32>,
    /// Contador total de transiciones
    total_transitions: u32,
}

impl MarkovModel {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            states: HashSet::new(),
            actions: HashSet::new(),
            initial_distribution: HashMap::new(),
            total_transitions: 0,
        }
    }

    /// Aprende de una secuencia de estados y acciones
    pub fn learn_sequence(&mut self, sequence: &[String], actions: &[String]) {
        if sequence.len() < 2 {
            return;
        }

        // Update initial distribution
        if let Some(first) = sequence.first() {
            *self
                .initial_distribution
                .entry(first.clone())
                .or_insert(0.0) += 1.0;
        }

        // Learn transitions
        for i in 0..sequence.len() - 1 {
            let current = &sequence[i];
            let next = &sequence[i + 1];
            let action = actions.get(i).map(|a| a.as_str()).unwrap_or("default");

            self.states.insert(current.clone());
            self.states.insert(next.clone());
            self.actions.insert(action.to_string());

            let key = (current.clone(), action.to_string());
            if let Some((existing_next, count)) = self.transitions.get_mut(&key) {
                if *existing_next == *next {
                    *count += 1;
                } else {
                    // Branching - keep the more frequent one
                    *existing_next = next.clone();
                }
            } else {
                self.transitions.insert(key, (next.clone(), 1));
            }

            self.total_transitions += 1;
        }
    }

    /// Predice siguiente estado dada una acción
    pub fn predict_next_state(&self, current_state: &str, action: &str) -> Option<(String, f32)> {
        let key = (current_state.to_string(), action.to_string());
        if let Some((next_state, count)) = self.transitions.get(&key) {
            let prob = *count as f32 / self.total_transitions as f32;
            Some((next_state.clone(), prob))
        } else {
            // Fallback: use initial distribution
            self.initial_distribution
                .get(current_state)
                .map(|&prob| (current_state.to_string(), prob))
        }
    }

    /// Obtiene distribución de próxima acción
    pub fn get_action_distribution(&self, state: &str) -> ActionProbabilities {
        let mut actions_probs: HashMap<String, f32> = HashMap::new();

        for action in &self.actions {
            if let Some((_, count)) = self.transitions.get(&(state.to_string(), action.clone())) {
                let prob = *count as f32 / self.total_transitions as f32;
                actions_probs.insert(action.clone(), prob);
            }
        }

        let actions: Vec<_> = actions_probs.into_iter().collect();
        let entropy = Self::compute_entropy(&actions);

        ActionProbabilities {
            entropy,
            timestamp: 0,
            actions,
        }
    }

    fn compute_entropy(actions: &[(String, f32)]) -> f32 {
        let mut entropy = 0.0f32;
        for (_, prob) in actions {
            if *prob > 0.0 {
                entropy -= prob * prob.log2();
            }
        }
        entropy
    }

    /// Serializa el modelo
    pub fn to_string(&self) -> String {
        let mut s = format!(
            "MarkovModel(states={}, actions={})\n",
            self.states.len(),
            self.actions.len()
        );
        for state in &self.states {
            s += &format!("  State: {}\n", state);
            for action in &self.actions {
                if let Some((next, count)) = self.transitions.get(&(state.clone(), action.clone()))
                {
                    s += &format!("    {} -> {} (count={})\n", action, next, count);
                }
            }
        }
        s
    }
}

// ============================================================================
// ACTION PREDICTOR
// ============================================================================

/// Motor de predicción de acciones
pub struct ActionPredictor {
    /// Modelos de Markov por agente
    markov_models: HashMap<AgentId, MarkovModel>,
    /// Patrones comportamentales por agente
    patterns: HashMap<AgentId, Vec<BehavioralPattern>>,
    /// Historial de predicciones
    prediction_history: Vec<PredictionRecord>,
    /// Configuración
    config: PredictorConfig,
}

/// Configuración del predictor
#[derive(Clone, Debug)]
pub struct PredictorConfig {
    /// Horizonte de predicción (en pasos)
    pub prediction_horizon: usize,
    /// Umbral de confianza mínimo
    pub min_confidence: f32,
    /// Peso del modelo de Markov vs patrones
    pub markov_weight: f32,
    /// Peso de patrones vs Markov
    pub pattern_weight: f32,
}

impl Default for PredictorConfig {
    fn default() -> Self {
        Self {
            prediction_horizon: 3,
            min_confidence: 0.3,
            markov_weight: 0.5,
            pattern_weight: 0.5,
        }
    }
}

/// Registro de predicción
#[derive(Clone, Debug)]
pub struct PredictionRecord {
    pub agent_id: AgentId,
    pub predicted_action: String,
    pub actual_action: String,
    pub was_correct: bool,
    pub confidence: f32,
    pub timestamp: TimePoint,
}

impl ActionPredictor {
    pub fn new() -> Self {
        Self {
            markov_models: HashMap::new(),
            patterns: HashMap::new(),
            prediction_history: Vec::new(),
            config: PredictorConfig::default(),
        }
    }

    /// Registra un agente
    pub fn register_agent(&mut self, agent_id: AgentId) {
        self.markov_models.insert(agent_id, MarkovModel::new());
        self.patterns.insert(agent_id, Vec::new());
    }

    /// Actualiza el modelo de un agente con nueva observación
    pub fn observe(&mut self, agent_id: AgentId, state: &str, action: &str, next_state: &str) {
        let model = self
            .markov_models
            .entry(agent_id)
            .or_insert_with(MarkovModel::new);
        model.learn_sequence(
            &[state.to_string(), next_state.to_string()],
            &[action.to_string()],
        );
    }

    /// Predice próxima acción de un agente
    pub fn predict_next_action(
        &mut self,
        agent_id: AgentId,
        current_state: &str,
        context: &PredictionContext,
        _timestamp: TimePoint,
    ) -> Vec<ActionPrediction> {
        let mut predictions = Vec::new();

        // Get or create model
        let model = self
            .markov_models
            .entry(agent_id)
            .or_insert_with(MarkovModel::new);

        // Get action distribution from Markov model
        let dist = model.get_action_distribution(current_state);

        // Generate predictions from distribution
        for (action, prob) in &dist.actions {
            if *prob >= self.config.min_confidence {
                predictions.push(ActionPrediction {
                    action: action.clone(),
                    probability: *prob,
                    confidence: *prob,
                    expected_time: None,
                    context: context.environment.location.clone(),
                });
            }
        }

        // Also check patterns
        if let Some(patterns) = self.patterns.get(&agent_id) {
            for pattern in patterns {
                if pattern.context == context.environment.location {
                    // Pattern matches current context
                    if let Some(predicted) = self.predict_from_pattern(pattern, current_state) {
                        predictions.push(predicted);
                    }
                }
            }
        }

        // Sort by probability
        predictions.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());

        predictions
    }

    fn predict_from_pattern(
        &self,
        pattern: &BehavioralPattern,
        current_state: &str,
    ) -> Option<ActionPrediction> {
        // Find if current state is part of this pattern
        for (i, state) in pattern.sequence.iter().enumerate() {
            if state == current_state {
                if i + 1 < pattern.sequence.len() {
                    return Some(ActionPrediction {
                        action: pattern.sequence[i + 1].clone(),
                        probability: pattern.frequency,
                        confidence: pattern.frequency * 0.8, // Slightly lower confidence for pattern-based
                        expected_time: None,
                        context: pattern.context.clone(),
                    });
                }
            }
        }
        None
    }

    /// Detecta nuevos patrones
    pub fn detect_patterns(
        &mut self,
        agent_id: AgentId,
        min_frequency: f32,
    ) -> Vec<BehavioralPattern> {
        let mut new_patterns = Vec::new();
        let mut pattern_id = 0;

        // Simple pattern detection: look for repeated subsequences
        let model = match self.markov_models.get(&agent_id) {
            Some(m) => m,
            None => return vec![],
        };

        // Extract sequences from transitions
        let mut sequences: Vec<Vec<String>> = Vec::new();
        for ((state, action), (next, _)) in &model.transitions {
            sequences.push(vec![state.clone(), action.clone(), next.clone()]);
        }

        // Find subsequences that appear more than once
        let mut subsequence_counts: HashMap<String, u32> = HashMap::new();
        for seq in &sequences {
            let key = seq.join("->");
            *subsequence_counts.entry(key).or_insert(0) += 1;
        }

        for (seq_str, count) in subsequence_counts {
            let freq = count as f32;
            if freq >= min_frequency {
                let parts: Vec<&str> = seq_str.split("->").collect();
                new_patterns.push(BehavioralPattern {
                    pattern_id,
                    sequence: parts.iter().map(|s| s.to_string()).collect(),
                    frequency: freq,
                    last_observed: 0,
                    context: "general".to_string(),
                    is_novel: true,
                });
                pattern_id += 1;
            }
        }

        // Update stored patterns
        if let Some(existing) = self.patterns.get_mut(&agent_id) {
            existing.extend(new_patterns.clone());
        }

        new_patterns
    }

    /// Evalúa una predicción contra la realidad
    pub fn evaluate_prediction(&mut self, record: PredictionRecord) {
        self.prediction_history.push(record.clone());

        // Update markov model if prediction was wrong
        if !record.was_correct {
            // Could adjust model based on prediction error
            let _ = record; // Would use for online learning
        }
    }

    /// Obtiene historial de predicciones para un agente
    pub fn get_prediction_history(&self, agent_id: AgentId) -> Vec<&PredictionRecord> {
        self.prediction_history
            .iter()
            .filter(|r| r.agent_id == agent_id)
            .collect()
    }

    /// Calcula precisión de predicciones recientes
    pub fn recent_accuracy(&self, agent_id: AgentId, n: usize) -> f32 {
        let recent: Vec<_> = self
            .prediction_history
            .iter()
            .filter(|r| r.agent_id == agent_id)
            .rev()
            .take(n)
            .collect();

        if recent.is_empty() {
            return 0.0;
        }

        let correct = recent.iter().filter(|r| r.was_correct).count();
        correct as f32 / recent.len() as f32
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Predice próxima acción
pub fn predict_action(
    predictor: &mut ActionPredictor,
    agent_id: AgentId,
    current_state: &str,
    context: &PredictionContext,
    timestamp: TimePoint,
) -> Option<ActionPrediction> {
    let predictions = predictor.predict_next_action(agent_id, current_state, context, timestamp);
    predictions
        .into_iter()
        .max_by(|a, b| a.probability.partial_cmp(&b.probability).unwrap())
}

/// Actualiza modelo comportamental
pub fn update_behavioral_model(
    predictor: &mut ActionPredictor,
    agent_id: AgentId,
    state: &str,
    action: &str,
    next_state: &str,
) {
    predictor.observe(agent_id, state, action, next_state);
}

/// Calcula distribución de acciones
pub fn compute_action_distribution(
    predictor: &ActionPredictor,
    agent_id: AgentId,
    state: &str,
) -> Option<ActionProbabilities> {
    predictor
        .markov_models
        .get(&agent_id)
        .map(|m| m.get_action_distribution(state))
}
