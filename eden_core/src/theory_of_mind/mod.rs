//! # Theory of Mind Module
//!
//! Sistema de modelado de mentes ajenas 100% original.
//! Sin dependencias de bibliotecas externas.
//!
//! ## Componentes
//!
//! - `models`: Representación de estados mentales (BDI)
//! - `inference`: Motor de inferencia de intenciones
//! - `emotion`: Detección y modelado de emociones
//! - `prediction`: Predicción de acciones
//! - `attribution`: Atribución de estados mentales
//! - `social`: Razonamiento social y relaciones
//!
//! ## Conceptos Fundamentales
//!
//! 1. **BDI (Belief-Desire-Intention)**: Modelo de agencia
//! 2. **Theory of Mind**: Capacidad de entender mentes ajenas
//! 3. **Intention Inference**: Inferir lo que otro agente intenta
//! 4. **Emotion Detection**: Detectar estados emocionales
//! 5. **Counterfactual Reasoning**: Razonar sobre "qué pasaría si"
//! 6. **Social Reasoning**: Entender dinámicas sociales
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod attribution;
pub mod emotion;
pub mod inference;
pub mod models;
pub mod prediction;
pub mod social;

// ============================================================================
// RE-EXPORTS
// ============================================================================

pub use attribution::{
    attribute_mental_state, attribution_pipeline, compute_attribution_confidence,
    AttributionConfidence, AttributionResult, MentalAttributor, MentalStateAttribution,
    ObservableSignal,
};
pub use emotion::{
    detect_emotion, emotion_from_signal, track_emotion_dynamics, AffectiveModel, EmotionDetector,
    EmotionDynamics, EmotionalState,
};
pub use inference::{
    desire_strength, infer_intention, BeliefRevision, InferenceContext, IntentionHypothesis,
    IntentionInferenceEngine, IntentionSignal,
};
pub use models::{
    AgentId, Belief, BeliefSource, Commitment, Confidence, Desire, EmotionType, ExecutionState,
    GroupIdentity, Intention, MentalAttitude, MentalModel, MentalSnapshot, Plan, PlanStep,
    RelationshipType, SocialRelationship, TimePoint, Valuation,
};
pub use prediction::{
    compute_action_distribution, predict_action, update_behavioral_model, ActionPrediction,
    ActionPredictor, ActionProbabilities, BehavioralPattern, MarkovModel, PredictionContext,
};
pub use social::{
    analyze_relationship, get_relationship_strength, reason_about_social, InteractionRecord,
    InterpersonalRelation, PowerStructure, SocialDynamics, SocialReasoner,
};
