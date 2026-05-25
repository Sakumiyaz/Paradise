//! Módulo Genesis: Creación y persistencia de patrones únicos
//!
//! Sistema de generación de patrones binarios únicos sin dependencias externas.
#![allow(dead_code)]
#![allow(non_snake_case)]

mod pattern;
mod registry;

// Re-export todo desde pattern y registry
pub use pattern::*;
pub use registry::*;
