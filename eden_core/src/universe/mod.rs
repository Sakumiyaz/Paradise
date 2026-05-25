//! # Universe Module
//!
//! Este módulo implementa la gestión de múltiples universos:
//!
//! - **multiverse**: Jardín de universos con fisión cósmica y poda
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod multiverse;

// Re-exports
pub use multiverse::{
    CosmicScheduler, EstadoUniverso, MensajeControl, MetricasConsolidadas, MetricasUniverso,
    MultiverseManager, UniversoHilo,
};
