//! # Core Module
//!
//! Núcleo del sistema EDEN con capacidades fundamentales:
//!
//! - `autopoietic_kernel`: Sistema de auto-creación y auto-renovación
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod autopoietic_kernel;

// Re-exports
pub use autopoietic_kernel::{
    AutopoieticComponent, AutopoieticConfig, AutopoieticCycleResult, AutopoieticKernel,
    AutopoieticOp, AutopoieticOperator, AutopoieticState, BoundaryAwareness, ComponentModel,
    ComponentState, ComponentType, ErrorRecord, ErrorSeverity, ErrorType, IntegrityMonitor,
    KernelStats, OperationCriticality, SelfModel, SelfReferenceMetrics, StateChange,
    StateChangeCause, AUTOPOIESIS_COHERENCE_THRESHOLD, AUTOPOIESIS_OPS_PER_CYCLE,
    MAX_SELF_REFERENCE_DEPTH, SELF_RENEWAL_FACTOR,
};
