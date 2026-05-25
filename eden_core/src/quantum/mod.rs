//! # Quantum Module
//!
//! Quantum processing module for EDEN.
//! Permite procesamiento paralelo y predicción de estados futuros.
//!
//! ## Componentes
//!
//! - `pre_cognition`: Predicción de estados futuros (Markov + Bayes)
//! - `quantum_processor`: Procesamiento masivo paralelo y superposición

#![allow(dead_code)]

pub mod pre_cognition;
pub mod quantum_processor;

// Exports públicos
pub use pre_cognition::{
    AnalisisTendencia, BifurcacionPredicha, BifurcationPoint, CadenaPrediccion,
    CausalInferenceEngine, CausalInferenceResult, CausalSample, DistributionType, EntanglementLink,
    EstadoPredicho, PreCogStats, PreCognition, PrediccionRiesgo, QuantumBranch,
    ResultadoSimulacion, SuperpositionState, TemporalBifurcation, TemporalBifurcationAnalyzer,
    TipoRiesgo, UncertaintyQuantification, UncertaintyQuantifier,
};

pub use quantum_processor::{
    Amplitud, EstadoEntrelazado, EstadoSuperposicion, QuantumProcessor, QuantumStats,
};
