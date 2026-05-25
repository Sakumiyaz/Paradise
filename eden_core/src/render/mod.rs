//! # Render Module: SoftGPU and Terminal Rendering
//!
//! Este módulo proporciona renderizado para el estado del Mar Morfóseo y los Auton.
//!
//! ## Componentes
//!
//! - **soft_gpu**: Renderizado en framebuffer de Linux (/dev/fb0) o fallback a terminal
//! - **term_hex**: Visualizador hexagonal ASCII/Unicode para terminal

#![allow(dead_code)]

pub mod soft_gpu;
pub mod term_hex;

pub use soft_gpu::{ModoRender, RenderStats, SoftGPU};

pub use term_hex::{StatsSistema, TermHex, TermHexStats};
