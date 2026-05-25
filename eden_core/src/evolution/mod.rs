//! # Evolution Module
//!
//! Este módulo implementa mecanismos de evolución y auto-modificación:
//!
//! - **hot_patch**: Sistema de parches en caliente para auto-modificación segura
//! - **open_endedness**: Motor de inagotabilidad natural
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod hot_patch;
pub mod open_endedness;

// Re-exports
pub use hot_patch::{
    CriteriosIluminacion, EstadoParche, HotPatchManager, Parche, PatchableAddr, PatchableFunc,
    VerificadorIluminacion, VerificadorInstrucciones,
};
