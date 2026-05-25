//! # Consciousness Module: Introspección y Autoconciencia
//!
//! Este módulo implementa el sistema de autoconciencia de EDEN,
//! incluyendo el Modelo Interno de Sí Mismo (MISM), ciclos reflexivos,
//! motor de sueños y narrativa generativa.
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod benchmarks;
pub mod dream_engine;
pub mod energy_aware;
pub mod ethical_review;
pub mod feature_flag;
pub mod global_workspace;
pub mod iit_phi;
pub mod integration_tests;
pub mod introspection;
pub mod memory_strategy;
pub mod phi_monitor;
pub mod philosophy;
pub mod storyteller;

// Re-exports
pub use introspection::{
    AnomalyType,
    AutobiographicalEntry,
    AutobiographicalMemory,
    CalibrationResult,
    ComponentState,
    // Nuevos exports para consciousness feature flag
    ConsciousnessConfig,
    ConsciousnessStats,
    EnhancedMISM,
    EstadoUniverso,
    Hipotesis,
    IdentityChange,
    IdentityCoherence,
    Intervencion,
    IntervencionTipo,
    IntrospectionManager,
    IntrospectionManagerLocked,
    IntrospectionStats,
    MISMStats,
    NodoCausal,
    RedBayesiana,
    SelfAnomaly,
    SelfAwarenessEngine,
    SelfAwarenessMetric,
    SelfModel,
    Severity,
    SnapshotMISM,
    MISM,
};

pub use dream_engine::{
    DreamEngine, DreamManagerLocked, DreamStats, EscenarioHipotetico, EstadoSueno, MetricasSueno,
    ResultadoSueno, TipoIntervencion, UniversoOnirico,
};

pub use storyteller::{
    EstadoNarrativo, EventoEspecial, Narrativa, Storyteller, StorytellerLocked, TonoEmocional,
};

pub use energy_aware::{
    AccionTermica, EnergyAware, EnergyAwareLocked, EnergyStats, EstadoBateria, EstadoEnergia,
    LecturaTermica, ModoPotencia, NivelTermico, PresupuestoEnergetico, TEMP_ALTO, TEMP_CRITICO,
    TEMP_NORMAL, TEMP_OPTIMO,
};

pub use benchmarks::{run_all_benchmarks, BenchmarkRunner, Measurement};
pub use ethical_review::{
    CertaintyLevel, ConsciousnessAssessment, ConsciousnessIndicator, ConsciousnessMetrics,
    EthicalAction, EthicalDecision, EthicalRecommendation, EthicalReviewEngine, ImpliedRight,
    RightsLevel, Stakeholder, StakeholderCategory,
};
pub use feature_flag::{
    FeatureFlag, FeatureFlagManager, FeatureScope, FeatureStats, FeatureValue, SharedFeatureFlags,
};
pub use global_workspace::{
    AwarenessLevel, GlobalWorkspace, IntegrationScorer, IntegrationSnapshot, IntegrationTrend,
    ModuleId, ModuleState, Subscription, WorkspaceContent, WorkspaceStats,
};
pub use iit_phi::{
    ConsciousnessTier, EnhancedMISMState, InformationMetrics, IntegratedSystem, PhiCalculator,
    PhiConfig, PhiMeasurement, PhiTrend, SharedPhiCalculator, SystemElement,
};
pub use integration_tests::run_integration_tests;
pub use memory_strategy::{
    EvictionPolicy, MemoryEntry, MemoryExporter, MemoryStats, MemoryStrategy, MemoryStrategyConfig,
    MemoryTier,
};
pub use phi_monitor::{
    ConsciousnessEvent, EdenState, MonitorConfig, MonitorState, PhiAnalysis, PhiMonitor,
    PhiTrendDirection, SharedPhiMonitor,
};

/// Descripción del módulo
pub const MODULE_DESCRIPTION: &str = "EDEN Consciousness Module v1.0";

/// Constante de versión de la filosofía
pub const CONSCIOUSNESS_PHILOSOPHY_VERSION: &str = philosophy::CONSCIOUSNESS_PHILOSOPHY_VERSION;
