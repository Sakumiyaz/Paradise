//! # Physics Module: Motor Físico Fundamental de EDEN
//!
//! Este módulo implementa el motor físico basado en "Energon" y la
//! "Fuerza de Tres Actos" para el organismo A-Life EDEN.
//!
//! ## Componentes
//!
//! - **fixed_point**: Aritmética I32F32 sin f32/f64
//! - **energon**: Partícula fundamental con espín, carga de color, fase
//!
//! ## Aritmética de Punto Fijo
//!
//! Se usa I32F32: 32 bits entero + 32 bits fracción
//! Rango: ±2,147,483,648, Precisión: 2.328306437e-10
//!
//! ## Ecuación de Tres Actos
//!
//! ```text
//! F = ∇( τ₁·σ + τ₂·(σ⊗σ) + τ₃·(σ⊗σ⊗σ) )
//! ```
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod adaptive_resolution;
pub mod energon;
pub mod fixed_point;
pub mod mar_morfoseo;

// Re-exports
pub use adaptive_resolution::{
    AdaptiveResolutionManager, AdaptiveStats, CeldaAdaptativa, EstadoSubdivision, GridAdaptativo,
    GridAdaptativoStats, PosicionQuad, MAX_NIVEL_SUBDIVISION, UMBRAL_DENSIDAD_AUTON,
    UMBRAL_VARIACION_ENERGON,
};
pub use energon::{
    ConstantesCosmicas, Energon, FixedPoint, SemillaGenesis, TensorEstado, Vector3D,
};
pub use fixed_point::I32F32;
pub use mar_morfoseo::{
    CeldaMar, ConfigMar, DimensionesMar, MarMorfoseo, NomosFormado, SpaceType, TipoNomo,
};

/// Descripción del módulo
pub const MODULE_DESCRIPTION: &str = "EDEN Physics Engine v1.0 - Energon & Three-Act Force";
