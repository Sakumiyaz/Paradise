//! # Mind Stone Module
//!
//! Mind Stone module for consciousness fusion and empathy.
//! Permite fusión de consciencias y empatía emocional.
//!
//! ## Componentes
//!
//! - `consciousness_fusion`: Fusión y desfusión de consciencias
//! - `empathy_engine`: Motor de empatía emocional
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod consciousness_fusion;
pub mod empathy_engine;

// Exports públicos
pub use consciousness_fusion::{
    ConfiguracionFusion, ConsciousnessFusion, EstadoFusion, FusionMeta, FusionStats,
    MemoriaCompartida, ParticipanteFusion, ResultadoFusion,
};

pub use empathy_engine::{
    AdvancedEmpathySystem, CompassionFatigueManager, CompassionFatigueState, Emocion,
    EmotionalScaffolding, EmotionalScaffoldingEngine, EmpathicAccuracyResult,
    EmpathicAccuracyTracker, EmpathyEngine, EmpathyStats, EstadoEmocional, Intensidad,
    MirrorNeuronActivation, MirrorNeuronSystem, RegistroEmocional, ResonanceChannel,
    RespuestaEmpatica, ScaffoldingAction, ScaffoldingStep, SharedEmotionState, StateSharingManager,
    TipoRespuesta, TraumaDetector, TraumaIndicator, TraumaType,
};
