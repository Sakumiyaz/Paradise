//! # Analysis Module
//!
//! Módulos de análisis para EDEN:
//! - **psychohistory**: Predicción de colapsos demográficos
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod psychohistory;

// Re-exports
pub use psychohistory::{
    AlertaPsicohistoria, MetricasPsicohistoria, ModeloEstado, Observacion, Prediccion,
    PsychohistoryManager, PsychohistoryManagerLocked, RegresionLineal,
};

/// Descripción del módulo
pub const MODULE_DESCRIPTION: &str = "EDEN Analysis Module v1.0 - Psicohistoria";
