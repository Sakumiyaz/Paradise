// eden_core/src/security/mod.rs
//! # Security Module
//!
//! Módulos de seguridad para EDEN:
//! - `seccomp_filter`: Restricción de syscalls usando BPF

pub mod energy_aware;
pub mod hot_patch_verifier;
pub mod seccomp_filter;

pub use energy_aware::MonitorRecursos;
pub use hot_patch_verifier::{verificar_codigo_seguro, verificar_parche, verificar_parche_base64, InstruccionProhibida, ResultadoVerificacion, SaltoInvalido};
pub use seccomp_filter::{install_seccomp_filter, install_default_filter, is_seccomp_enabled};
