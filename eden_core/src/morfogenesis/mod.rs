//! # Morfogénesis - Crecimiento Neural del Cerebro de Eden
//!
//! Sistema de crecimiento celular basado en tensión superficial
//! y conciencia térmica del módulo de consciousness.
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod sustrato;

pub use sustrato::{
    SustratoVital,
    EstadoCelular,
    TermicoReader,
    IterSustrato,
    TablaSinaptica,
};
