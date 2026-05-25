//! # Attribution - Mental State Attribution
//!
//! Sistema para atribuir estados mentales a otros agentes.
//! 100% original, sin dependencias.
//!
//! ## Pipeline de Atribución
//!
//! 1. **Observar**: Recoger señales observables
//! 2. **Interpretar**: Interpretar señales en términos de estados mentales
//! 3. **Atribuir**: Atribuir creencias, deseos, intenciones
//! 4. **Evaluar**: Evaluar confianza en la atribución
//!
//! ## Conceptos
//!
//! - MentalStateAttribution: Resultado completo de atribución
//! - AttributionResult: Resultado de una atribución individual
//! - ObservableSignal: Señal directamente observable
//! - AttributionConfidence: Medida de confianza en atribución
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
use std::sync::Mutex;

use crate::theory_of_mind::{
    AgentId, Belief, Confidence, EmotionType, MentalModel, SocialRelationship, TimePoint,
};

// ============================================================================
// TIPOS BASE
// ============================================================================

/// Resultado de atribución de estado mental
#[derive(Clone, Debug)]
pub struct MentalStateAttribution {
    /// Agente al que se atribuye el estado
    pub target_agent: AgentId,
    /// Atribuciones realizadas
    pub attributions: Vec<AttributionResult>,
    /// Confianza general
    pub overall_confidence: f32,
    ///timestamp
    pub timestamp: TimePoint,
    /// Contexto de la atribución
    pub context: String,
}

impl MentalStateAttribution {
    pub fn new(target_agent: AgentId, context: &str) -> Self {
        Self {
            target_agent,
            attributions: Vec::new(),
            overall_confidence: 0.0,
            timestamp: 0,
            context: context.to_string(),
        }
    }

    pub fn with_time(mut self, time: TimePoint) -> Self {
        self.timestamp = time;
        self
    }

    pub fn add_attribution(&mut self, result: AttributionResult) {
        self.attributions.push(result);
        self.update_confidence();
    }

    fn update_confidence(&mut self) {
        if self.attributions.is_empty() {
            self.overall_confidence = 0.0;
            return;
        }

        let total: f32 = self.attributions.iter().map(|a| a.confidence.value()).sum();
        self.overall_confidence = total / self.attributions.len() as f32;
    }
}

/// Resultado de una atribución individual
#[derive(Clone, Debug)]
pub struct AttributionResult {
    /// Tipo de atribución
    pub attribution_type: AttributionType,
    /// Qué se atribuye
    pub content: String,
    /// Confianza
    pub confidence: Confidence,
    /// Evidencia usada
    pub evidence: Vec<ObservableSignal>,
    /// Método de atribución
    pub method: AttributionMethod,
}

/// Tipo de atribución
#[derive(Clone, Debug, PartialEq)]
pub enum AttributionType {
    /// Atribución de creencia
    Belief,
    /// Atribución de deseo
    Desire,
    /// Atribución de intención
    Intention,
    /// Atribución de emoción
    Emotion,
    /// Atribución de estado físico
    PhysicalState,
    /// Atribución de relación social
    SocialRelation,
}

/// Método de atribución usado
#[derive(Clone, Debug, PartialEq)]
pub enum AttributionMethod {
    /// Por eliminación de alternativas
    Elimination,
    /// Por analogía con uno mismo
    Analogy,
    /// Por reglas causales
    CausalRule,
    /// Por frecuencia统计
    FrequencyAnalysis,
    /// Por inferencia de objetivos
    GoalInference,
    /// Por simulación
    Simulation,
    /// Por comunicación directa
    DirectCommunication,
}

/// Señal observable que sirve como evidencia
#[derive(Clone, Debug, PartialEq)]
pub enum ObservableSignal {
    /// Acción observada
    Action { action: String, context: String },
    /// Expresión verbal
    Utterance { content: String, sincerity: f32 },
    /// Expresión emocional
    EmotionalExpression {
        emotion: EmotionType,
        intensity: f32,
    },
    /// Elección realizada
    Choice {
        alternatives: Vec<String>,
        chosen: String,
    },
    /// Uso de objeto
    ObjectUse { object: String, manner: String },
    /// Expresión facial
    FacialExpression { expression: String, intensity: f32 },
    /// Postura corporal
    BodyPosture { posture: String },
    /// Movimiento espacial
    SpatialMovement { from: String, to: String },
    /// Interacción social
    SocialInteraction {
        other: AgentId,
        type_interaction: String,
    },
    /// Búsqueda de información
    InformationSeeking { topic: String },
    /// Reaction a evento
    Reaction { event: String, response: String },
}

/// Confianza en una atribución
#[derive(Clone, Debug)]
pub struct AttributionConfidence {
    /// Confianza base [0.0 - 1.0]
    pub base: f32,
    /// Ajustes por evidencia
    pub evidence_adjustments: Vec<f32>,
    /// Ajustes por relación social
    pub relationship_adjustment: f32,
    /// Ajustes por conocimiento previo
    pub prior_adjustment: f32,
    /// Confianza final
    pub final_confidence: f32,
}

impl AttributionConfidence {
    pub fn new(base: f32) -> Self {
        Self {
            base,
            evidence_adjustments: Vec::new(),
            relationship_adjustment: 0.0,
            prior_adjustment: 0.0,
            final_confidence: base,
        }
    }

    pub fn with_evidence_adjustment(mut self, adjustment: f32) -> Self {
        self.evidence_adjustments.push(adjustment);
        self.recompute();
        self
    }

    pub fn with_relationship_adjustment(mut self, adjustment: f32) -> Self {
        self.relationship_adjustment = adjustment;
        self.recompute();
        self
    }

    pub fn with_prior_adjustment(mut self, adjustment: f32) -> Self {
        self.prior_adjustment = adjustment;
        self.recompute();
        self
    }

    fn recompute(&mut self) {
        let evidence_bonus: f32 = self.evidence_adjustments.iter().sum();
        self.final_confidence =
            (self.base + evidence_bonus + self.relationship_adjustment + self.prior_adjustment)
                .max(0.0)
                .min(1.0);
    }

    pub fn value(&self) -> f32 {
        self.final_confidence
    }
}

// ============================================================================
// ATTRIBUTION PIPELINE
// ============================================================================

/// Pipeline de atribución de estados mentales
pub struct MentalAttributor {
    /// Configuración
    config: AttributorConfig,
    /// Historial de atribuciones
    history: Vec<MentalStateAttribution>,
    /// Modelos mentales atribuidos por agente
    attributed_models: std::collections::HashMap<AgentId, MentalModel>,
}

/// Configuración del atribuidor
#[derive(Clone, Debug)]
pub struct AttributorConfig {
    /// Confianza mínima para aceptar atribución
    pub min_confidence: f32,
    /// Peso de relación social en atribución
    pub social_relationship_weight: f32,
    /// Considerar conocimiento previo
    pub use_prior_knowledge: bool,
    /// Decaimiento temporal de atribuciones
    pub attribution_decay: f32,
}

impl Default for AttributorConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.4,
            social_relationship_weight: 0.2,
            use_prior_knowledge: true,
            attribution_decay: 0.9,
        }
    }
}

impl MentalAttributor {
    pub fn new() -> Self {
        Self {
            config: AttributorConfig::default(),
            history: Vec::new(),
            attributed_models: std::collections::HashMap::new(),
        }
    }

    /// Atribuye estado mental completo
    pub fn attribute_mental_state(
        &mut self,
        target_id: AgentId,
        signals: &[ObservableSignal],
        observer_relationship: Option<&SocialRelationship>,
        context: &str,
        timestamp: TimePoint,
    ) -> MentalStateAttribution {
        let mut attribution = MentalStateAttribution::new(target_id, context).with_time(timestamp);

        // Attribute beliefs
        for signal in signals {
            if let Some(result) =
                self.attribute_from_signal(signal, target_id, observer_relationship)
            {
                if result.confidence.value() >= self.config.min_confidence {
                    attribution.add_attribution(result);
                }
            }
        }

        // Add to history
        self.history.push(attribution.clone());

        // Update attributed model
        let model = self.reconstruct_model(&attribution);
        self.attributed_models.insert(target_id, model);

        attribution
    }

    fn attribute_from_signal(
        &self,
        signal: &ObservableSignal,
        _target_id: AgentId,
        observer_relationship: Option<&SocialRelationship>,
    ) -> Option<AttributionResult> {
        match signal {
            ObservableSignal::Action { action, .. } => {
                // Attribute intention from action
                Some(AttributionResult {
                    attribution_type: AttributionType::Intention,
                    content: format!("Intends to: {}", action),
                    confidence: self.compute_action_confidence(signal, observer_relationship),
                    evidence: vec![signal.clone()],
                    method: AttributionMethod::GoalInference,
                })
            }
            ObservableSignal::Utterance { content, sincerity } => {
                // Attribute belief from utterance
                let conf = if *sincerity > 0.7 {
                    self.compute_action_confidence(signal, observer_relationship)
                } else {
                    Confidence::low()
                };
                Some(AttributionResult {
                    attribution_type: AttributionType::Belief,
                    content: format!("Believes: {} (sincerity: {:.2})", content, sincerity),
                    confidence: conf,
                    evidence: vec![signal.clone()],
                    method: AttributionMethod::DirectCommunication,
                })
            }
            ObservableSignal::EmotionalExpression { emotion, intensity } => {
                Some(AttributionResult {
                    attribution_type: AttributionType::Emotion,
                    content: format!("Feels {:?} with intensity {:.2}", emotion, intensity),
                    confidence: self.compute_emotion_confidence(intensity, observer_relationship),
                    evidence: vec![signal.clone()],
                    method: AttributionMethod::Simulation,
                })
            }
            ObservableSignal::Choice {
                alternatives,
                chosen,
            } => {
                // Attribute preference/desire from choice
                Some(AttributionResult {
                    attribution_type: AttributionType::Desire,
                    content: format!("Prefers: {} over {:?}", chosen, alternatives),
                    confidence: self.compute_choice_confidence(signal, observer_relationship),
                    evidence: vec![signal.clone()],
                    method: AttributionMethod::Elimination,
                })
            }
            ObservableSignal::SocialInteraction {
                other,
                type_interaction,
            } => Some(AttributionResult {
                attribution_type: AttributionType::SocialRelation,
                content: format!("Relation with {:?}: {}", other, type_interaction),
                confidence: self.compute_social_confidence(signal, observer_relationship),
                evidence: vec![signal.clone()],
                method: AttributionMethod::FrequencyAnalysis,
            }),
            _ => None,
        }
    }

    fn compute_action_confidence(
        &self,
        _signal: &ObservableSignal,
        observer_relationship: Option<&SocialRelationship>,
    ) -> Confidence {
        let base = Confidence::medium();

        let adjustment = if let Some(rel) = observer_relationship {
            rel.trust * self.config.social_relationship_weight
        } else {
            0.0
        };

        Confidence((base.0 + adjustment).min(1.0))
    }

    fn compute_emotion_confidence(
        &self,
        intensity: &f32,
        observer_relationship: Option<&SocialRelationship>,
    ) -> Confidence {
        // Higher intensity = more confident detection
        let base = if *intensity > 0.7 {
            Confidence::high()
        } else if *intensity > 0.4 {
            Confidence::medium()
        } else {
            Confidence::low()
        };

        let adjustment = observer_relationship
            .map(|rel| rel.trust * 0.1)
            .unwrap_or(0.0);

        Confidence((base.0 + adjustment).min(1.0))
    }

    fn compute_choice_confidence(
        &self,
        _signal: &ObservableSignal,
        _observer_relationship: Option<&SocialRelationship>,
    ) -> Confidence {
        Confidence::medium()
    }

    fn compute_social_confidence(
        &self,
        _signal: &ObservableSignal,
        _observer_relationship: Option<&SocialRelationship>,
    ) -> Confidence {
        Confidence::medium()
    }

    fn reconstruct_model(&self, attribution: &MentalStateAttribution) -> MentalModel {
        let mut model = MentalModel::new(attribution.target_agent);

        for result in &attribution.attributions {
            match result.attribution_type {
                AttributionType::Belief => {
                    let belief = Belief::new(
                        &result.content,
                        result.confidence.clone(),
                        crate::theory_of_mind::BeliefSource::Inference,
                    );
                    model.add_belief(belief);
                }
                AttributionType::Desire => {
                    let desire =
                        crate::theory_of_mind::Desire::new(&result.content, Valuation::medium());
                    model.add_desire(desire);
                }
                AttributionType::Intention => {
                    let plan = Plan {
                        steps: vec![PlanStep {
                            description: result.content.clone(),
                            preconditions: vec![],
                            effects: vec![],
                            estimated_duration: 0,
                            completed: false,
                        }],
                        goal: result.content.clone(),
                        alternative: None,
                    };
                    let intention =
                        crate::theory_of_mind::Intention::new(plan, Commitment::Moderate);
                    model.add_intention(intention);
                }
                AttributionType::Emotion => {
                    // Parse emotion from content
                    model.set_emotion(EmotionType::Joy, 0.5); // Default
                }
                _ => {}
            }
        }

        model
    }

    /// Obtiene el modelo atribuido de un agente
    pub fn get_attributed_model(&self, agent_id: AgentId) -> Option<&MentalModel> {
        self.attributed_models.get(&agent_id)
    }

    /// Obtiene historial de atribuciones
    pub fn get_history(&self) -> &[MentalStateAttribution] {
        &self.history
    }
}

// Needed imports
use crate::theory_of_mind::{Commitment, Plan, PlanStep, Valuation};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Pipeline completo de atribución
pub fn attribution_pipeline(
    attributor: &mut MentalAttributor,
    target_id: AgentId,
    signals: &[ObservableSignal],
    observer_relationship: Option<&SocialRelationship>,
    context: &str,
    timestamp: TimePoint,
) -> MentalStateAttribution {
    attributor.attribute_mental_state(
        target_id,
        signals,
        observer_relationship,
        context,
        timestamp,
    )
}

/// Atribuye estado mental
pub fn attribute_mental_state(
    attributor: &mut MentalAttributor,
    target_id: AgentId,
    signals: &[ObservableSignal],
    observer_relationship: Option<&SocialRelationship>,
    context: &str,
    timestamp: TimePoint,
) -> MentalStateAttribution {
    attributor.attribute_mental_state(
        target_id,
        signals,
        observer_relationship,
        context,
        timestamp,
    )
}

/// Computa confianza de atribución
pub fn compute_attribution_confidence(
    base: f32,
    evidence_count: usize,
    relationship: Option<&SocialRelationship>,
    prior_accuracy: Option<f32>,
) -> AttributionConfidence {
    let mut conf = AttributionConfidence::new(base);

    if evidence_count > 0 {
        conf = conf.with_evidence_adjustment(0.05 * evidence_count as f32);
    }

    if let Some(rel) = relationship {
        conf = conf.with_relationship_adjustment(rel.trust * 0.1);
    }

    if let Some(acc) = prior_accuracy {
        conf = conf.with_prior_adjustment((acc - 0.5) * 0.2);
    }

    conf
}
