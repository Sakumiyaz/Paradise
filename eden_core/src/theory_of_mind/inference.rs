//! # Inference - Intention Inference Engine
//!
//! Motor para inferir las intenciones de otros agentes.
//! 100% original, sin dependencias.
//!
//! ## Algoritmos
//!
//! 1. **Pragmatic Inference**: Inferencia basada en contexto pragmático
//! 2. **Rational Agency**: Asumir que el agente actúa racionalmente
//! 3. **Plan Recognition**: Reconocer planes a partir de acciones
//! 4. **Goal Inference**: Inferir metas a partir de deseos observados
//!
//! ## Conceptos
//!
//! - IntentionHypothesis: Hipótesis sobre la intención de un agente
//! - InferenceContext: Contexto para la inferencia
//! - IntentionSignal: Señales observables que sugieren intención
//! - BeliefRevision: Revisión de creencias basada en evidencia
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use crate::theory_of_mind::{
    AgentId, Belief, Commitment, Confidence, Desire, EmotionType, Intention, MentalModel, Plan,
    TimePoint,
};
use std::collections::HashMap;

// ============================================================================
// TIPOS BASE
// ============================================================================

/// Señal observable que puede indicar intención
#[derive(Clone, Debug, PartialEq)]
pub enum IntentionSignal {
    /// El agente realizó una acción específica
    Action { action: String, context: String },
    /// El agente hizo una declaración
    Utterance { content: String, sincerity: f32 },
    /// Expresión emocional observada
    EmotionalExpression {
        emotion: EmotionType,
        intensity: f32,
    },
    /// El agente buscó información específica
    InformationSeeking { topic: String },
    /// El agente cambió su comportamiento
    BehavioralChange { previous: String, current: String },
    /// El agente停止了某种行为
    ActionInhibition { action: String },
    /// El agente cambió de objetivo
    GoalSwitch { from: String, to: String },
    /// El agente coordinó con otros
    Coordination {
        agents: Vec<AgentId>,
        action: String,
    },
    /// Observación directa de estado mental
    DirectObservation { observation: String },
}

/// Resultado de una hipótesis de intención
#[derive(Clone, Debug)]
pub struct IntentionHypothesis {
    /// Intención inferida
    pub inferred_intention: Intention,
    /// Confianza en la inferencia
    pub confidence: Confidence,
    /// Evidencia que apoya la inferencia
    pub supporting_evidence: Vec<IntentionSignal>,
    /// Alternativas que fueron descartadas
    pub alternatives: Vec<String>,
    /// Contexto de la inferencia
    pub context: String,
    /// timestamp
    pub timestamp: TimePoint,
}

impl IntentionHypothesis {
    pub fn new(intention: Intention, confidence: Confidence, context: &str) -> Self {
        Self {
            inferred_intention: intention,
            confidence,
            supporting_evidence: Vec::new(),
            alternatives: Vec::new(),
            context: context.to_string(),
            timestamp: 0,
        }
    }

    pub fn with_time(mut self, time: TimePoint) -> Self {
        self.timestamp = time;
        self
    }

    pub fn add_evidence(&mut self, signal: IntentionSignal) {
        self.supporting_evidence.push(signal);
        // Increase confidence as more evidence accumulates
        let evidence_factor = (self.supporting_evidence.len() as f32 * 0.05).min(0.3);
        let current = self.confidence.0;
        self.confidence = Confidence(current + evidence_factor);
    }

    pub fn add_alternative(&mut self, alternative: &str) {
        self.alternatives.push(alternative.to_string());
        // Decrease confidence as more alternatives appear
        let alt_factor = (self.alternatives.len() as f32 * 0.03).min(0.2);
        let current = self.confidence.0;
        self.confidence = Confidence((current - alt_factor).max(0.1));
    }
}

/// Contexto para la inferencia
#[derive(Clone, Debug)]
pub struct InferenceContext {
    /// Modelo mental actual del agente objetivo
    pub target_model: MentalModel,
    /// Historial de interacciones
    pub interaction_history: Vec<InteractionRecord>,
    /// Ambiente actual
    pub environment: EnvironmentState,
    /// Otros agentes presentes
    pub other_agents: Vec<AgentId>,
    /// Constraints temporales
    pub time_constraints: TimeConstraints,
    /// Conocimiento previo sobre el agente
    pub prior_knowledge: Vec<Belief>,
}

/// Registro de una interacción pasada
#[derive(Clone, Debug)]
pub struct InteractionRecord {
    pub timestamp: TimePoint,
    pub action: String,
    pub result: String,
    pub target_agent: Option<AgentId>,
}

/// Estado del ambiente
#[derive(Clone, Debug)]
pub struct EnvironmentState {
    pub location: String,
    pub nearby_objects: Vec<String>,
    pub social_context: String,
    pub time_of_day: TimeOfDay,
}

/// Tiempo del día
#[derive(Clone, Debug, PartialEq)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
    Unclear,
}

/// Constraints temporales
#[derive(Clone, Debug)]
pub struct TimeConstraints {
    pub deadline: Option<TimePoint>,
    pub urgency: f32,
    pub time_available: u64,
}

// ============================================================================
// BELIEF REVISION
// ============================================================================

/// Revisión de creencias basada en nueva evidencia
#[derive(Clone, Debug)]
pub struct BeliefRevision {
    pub belief: Belief,
    pub old_confidence: f32,
    pub new_confidence: f32,
    pub evidence: String,
    pub revision_type: RevisionType,
    pub timestamp: TimePoint,
}

/// Tipo de revisión
#[derive(Clone, Debug, PartialEq)]
pub enum RevisionType {
    /// Fortalecimiento de creencia existente
    Strengthening,
    /// Debilitación de creencia
    Weakening,
    /// Nueva creencia formada
    Formation,
    /// Eliminación de creencia
    Elimination,
    /// Cambio completo de creencia
    Replacement,
}

impl BeliefRevision {
    pub fn strengthen(belief: &Belief, new_evidence: &str) -> Self {
        Self {
            belief: belief.clone(),
            old_confidence: belief.confidence.0,
            new_confidence: (belief.confidence.0 + 0.1).min(1.0),
            evidence: new_evidence.to_string(),
            revision_type: RevisionType::Strengthening,
            timestamp: 0,
        }
    }

    pub fn weaken(belief: &Belief, counter_evidence: &str) -> Self {
        Self {
            belief: belief.clone(),
            old_confidence: belief.confidence.0,
            new_confidence: (belief.confidence.0 - 0.1).max(0.0),
            evidence: counter_evidence.to_string(),
            revision_type: RevisionType::Weakening,
            timestamp: 0,
        }
    }
}

/// Actualiza creencias basándose en señales observadas
pub fn revise_beliefs_from_signals(
    model: &mut MentalModel,
    signals: &[IntentionSignal],
    timestamp: TimePoint,
) -> Vec<BeliefRevision> {
    let mut revisions = Vec::new();

    for signal in signals {
        match signal {
            IntentionSignal::Action { action, context } => {
                // Inferir creencias sobre el mundo basadas en acciones
                if let Some(belief) = infer_belief_from_action(model, action, context) {
                    let revision = BeliefRevision::strengthen(&belief, action);
                    model.add_belief(belief);
                    revisions.push(revision);
                }
            }
            IntentionSignal::Utterance { content, sincerity } => {
                // Ajustar creencias basadas en lo dicho
                if *sincerity < 0.3 {
                    // Probablemente mentía, debilitar creencia contraria
                    if let Some(belief) = model
                        .beliefs
                        .values_mut()
                        .find(|b| b.proposition.contains(content))
                    {
                        let rev = BeliefRevision::weaken(belief, content);
                        revisions.push(rev);
                    }
                }
            }
            IntentionSignal::EmotionalExpression { emotion, intensity } => {
                model.set_emotion((*emotion).clone(), *intensity);
            }
            IntentionSignal::BehavioralChange { previous, current } => {
                // El comportamiento cambió, inferir nuevo estado mental
                if let Some(belief) = Belief::new(
                    &format!("Changed behavior from {} to {}", previous, current),
                    Confidence::medium(),
                    crate::theory_of_mind::BeliefSource::Inference,
                )
                .with_time(timestamp)
                .into()
                {
                    let revision = BeliefRevision::strengthen(
                        &belief,
                        &format!("Behavior change: {} to {}", previous, current),
                    );
                    revisions.push(revision);
                }
            }
            _ => {}
        }
    }

    revisions
}

fn infer_belief_from_action(_model: &MentalModel, action: &str, context: &str) -> Option<Belief> {
    // Simple inference: action implies belief about preconditions
    let belief_prop = format!("Believes {} is feasible in context {}", action, context);
    Some(Belief::new(
        &belief_prop,
        Confidence::medium(),
        crate::theory_of_mind::BeliefSource::Inference,
    ))
}

// ============================================================================
// INTENTION INFERENCE ENGINE
// ============================================================================

/// Motor de inferencia de intenciones
pub struct IntentionInferenceEngine {
    /// Modelos mentales de otros agentes
    agent_models: HashMap<AgentId, MentalModel>,
    /// Historial de inferencias
    inference_history: Vec<IntentionHypothesis>,
    /// Hipótesis activas por agente
    active_hypotheses: HashMap<AgentId, Vec<IntentionHypothesis>>,
    /// Contexto global
    global_context: InferenceContext,
    /// Configuración
    config: InferenceConfig,
}

/// Configuración del motor de inferencia
#[derive(Clone, Debug)]
pub struct InferenceConfig {
    /// Confianza mínima para aceptar inferencia
    pub min_confidence: f32,
    /// Número máximo de alternativas a considerar
    pub max_alternatives: usize,
    /// Decaimiento temporal de inferencias
    pub inference_decay: f32,
    /// Peso de señales directas vs indirectas
    pub direct_signal_weight: f32,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            max_alternatives: 5,
            inference_decay: 0.95,
            direct_signal_weight: 0.8,
        }
    }
}

impl IntentionInferenceEngine {
    pub fn new() -> Self {
        Self {
            agent_models: HashMap::new(),
            inference_history: Vec::new(),
            active_hypotheses: HashMap::new(),
            global_context: InferenceContext {
                target_model: MentalModel::new(0),
                interaction_history: Vec::new(),
                environment: EnvironmentState {
                    location: "Unknown".to_string(),
                    nearby_objects: Vec::new(),
                    social_context: "Unknown".to_string(),
                    time_of_day: TimeOfDay::Unclear,
                },
                other_agents: Vec::new(),
                time_constraints: TimeConstraints {
                    deadline: None,
                    urgency: 0.5,
                    time_available: u64::MAX,
                },
                prior_knowledge: Vec::new(),
            },
            config: InferenceConfig::default(),
        }
    }

    /// Registra un nuevo agente
    pub fn register_agent(&mut self, agent_id: AgentId) {
        let model = MentalModel::new(agent_id);
        self.agent_models.insert(agent_id, model);
        self.active_hypotheses.insert(agent_id, Vec::new());
    }

    /// Actualiza el modelo mental de un agente
    pub fn update_agent_model(&mut self, agent_id: AgentId, model: MentalModel) {
        self.agent_models.insert(agent_id, model);
    }

    /// Infiere intención basándose en señales observadas
    pub fn infer_intention(
        &mut self,
        agent_id: AgentId,
        signals: &[IntentionSignal],
        context: &str,
        timestamp: TimePoint,
    ) -> Vec<IntentionHypothesis> {
        let mut hypotheses = Vec::new();

        // Get or create agent model - use get_mut to allow modification, then clone
        let model = self
            .agent_models
            .entry(agent_id)
            .or_insert_with(|| MentalModel::new(agent_id));
        let model_clone = model.clone();

        // Generate hypotheses based on signals
        for signal in signals {
            if let Some(hyp) = self.generate_hypothesis(&model_clone, signal, context, timestamp) {
                hypotheses.push(hyp);
            }
        }

        // Combine hypotheses and rank
        let ranked = self.rank_hypotheses(hypotheses);

        // Prune low-confidence hypotheses
        let filtered: Vec<_> = ranked
            .into_iter()
            .filter(|h| h.confidence.0 >= self.config.min_confidence)
            .take(self.config.max_alternatives)
            .collect();

        // Update active hypotheses
        self.active_hypotheses.insert(agent_id, filtered.clone());

        // Add to history
        self.inference_history.extend(filtered.clone());

        filtered
    }

    fn generate_hypothesis(
        &self,
        model: &MentalModel,
        signal: &IntentionSignal,
        context: &str,
        timestamp: TimePoint,
    ) -> Option<IntentionHypothesis> {
        match signal {
            IntentionSignal::Action { action, context } => {
                // Inferir intención racional basada en acción
                let plan = self.construct_plan_from_action(action, model);
                let intention = Intention::new(plan, Commitment::Moderate);
                let mut hyp = IntentionHypothesis::new(intention, Confidence::high(), context)
                    .with_time(timestamp);
                hyp.add_evidence(signal.clone());
                Some(hyp)
            }
            IntentionSignal::Utterance { content, sincerity } => {
                // Inferir intención comunicativa
                if *sincerity > 0.7 {
                    let plan = Plan {
                        steps: vec![PlanStep {
                            description: format!("Said: {}", content),
                            preconditions: vec![],
                            effects: vec![format!("Believes: {}", content)],
                            estimated_duration: 0,
                            completed: true,
                        }],
                        goal: content.clone(),
                        alternative: None,
                    };
                    let intention = Intention::new(plan, Commitment::Weak);
                    let mut hyp =
                        IntentionHypothesis::new(intention, Confidence::medium(), context)
                            .with_time(timestamp);
                    hyp.add_evidence(signal.clone());
                    Some(hyp)
                } else {
                    None // Low sincerity, unreliable signal
                }
            }
            IntentionSignal::EmotionalExpression { emotion, intensity } => {
                // Inferir estado emocional que puede indicar intención
                if *intensity > 0.6 {
                    let goal = format!("Expressing {:?}", emotion);
                    let plan = Plan {
                        steps: vec![],
                        goal,
                        alternative: None,
                    };
                    let intention = Intention::new(plan, Commitment::Moderate);
                    let confidence = Confidence(*intensity);
                    let mut hyp = IntentionHypothesis::new(intention, confidence, context)
                        .with_time(timestamp);
                    hyp.add_evidence(signal.clone());
                    Some(hyp)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn construct_plan_from_action(&self, action: &str, model: &MentalModel) -> Plan {
        // Find desire that action would satisfy
        let goal = model
            .desires
            .iter()
            .find(|d| action.contains(&d.goal))
            .map(|d| d.goal.clone())
            .unwrap_or_else(|| action.to_string());

        Plan {
            steps: vec![PlanStep {
                description: action.to_string(),
                preconditions: vec![],
                effects: vec![goal.clone()],
                estimated_duration: 1000,
                completed: false,
            }],
            goal,
            alternative: None,
        }
    }

    fn rank_hypotheses(
        &self,
        mut hypotheses: Vec<IntentionHypothesis>,
    ) -> Vec<IntentionHypothesis> {
        // Sort by confidence (descending)
        hypotheses.sort_by(|a, b| b.confidence.0.partial_cmp(&a.confidence.0).unwrap());
        hypotheses
    }

    /// Obtiene la intención más probable para un agente
    pub fn primary_intention(&self, agent_id: AgentId) -> Option<&IntentionHypothesis> {
        self.active_hypotheses
            .get(&agent_id)
            .and_then(|hyps| hyps.first())
    }

    /// Obtiene el modelo mental de un agente
    pub fn get_agent_model(&self, agent_id: AgentId) -> Option<&MentalModel> {
        self.agent_models.get(&agent_id)
    }

    /// Obtiene historial de inferencias
    pub fn get_inference_history(&self) -> &[IntentionHypothesis] {
        &self.inference_history
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Calcula fuerza de un deseo basándose en observaciones
pub fn desire_strength(model: &MentalModel, desire: &Desire) -> f32 {
    // Base intensity from valuation
    let base = desire.intensity();

    // Boost based on recent related actions
    let action_boost = model
        .intentions
        .iter()
        .filter(|i| i.plan.goal.contains(&desire.goal))
        .count() as f32
        * 0.1;

    // Boost from emotion alignment
    let emotion_boost = model
        .emotions
        .iter()
        .filter(|(emotion, _)| emotion.valence() > 0.5)
        .count() as f32
        * 0.05;

    (base + action_boost + emotion_boost).min(1.0)
}

/// Infiere intención principal de un agente
pub fn infer_intention(
    engine: &mut IntentionInferenceEngine,
    agent_id: AgentId,
    signals: &[IntentionSignal],
    context: &str,
    timestamp: TimePoint,
) -> Option<IntentionHypothesis> {
    let hypotheses = engine.infer_intention(agent_id, signals, context, timestamp);
    hypotheses
        .into_iter()
        .max_by(|a, b| a.confidence.0.partial_cmp(&b.confidence.0).unwrap())
}

// Needed for Belief::with_time
use crate::theory_of_mind::PlanStep;
