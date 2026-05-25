//! # EdenFS: Sistema de Archivos para Auton
//!
//! EdenFS proporciona persistencia para Auton usando el sistema de archivos del host,
//! soportando "bifurcaciones temporales" (time forking).
//!
//! ## Estructura de Directorios
//!
//! ```text
//! ~/.eden/
//!   universes/
//!     <semilla_universo>/
//!       autons/
//!         <uuid_auton>.bin    # Estado del Auton
//!         <uuid_auton>.meta   # Metadatos
//!       universe.meta         # Metadatos del universo
//!   meltrace/
//!     <uuid_auton>.<timestamp>.dead  # Auton muertos
//! ```
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod edenfs;

pub use edenfs::{AutonArchivo, CausaMuerte, EdenFS, EdenFsStats, MetadatosAuton};
