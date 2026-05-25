//! # GPS - NMEA 0183 Parser and Navigation
//!
//! Parser completo de NMEA 0183 para GPS receivers.
//! Cálculo de posición, velocidad, timestamp UTC.
//! 100% original, sin dependencias externas.

#![allow(dead_code)]

pub mod nmea;
pub mod position;
pub mod utils;

pub use nmea::NmeaParser;
pub use position::{LatLon, Position, Velocity};
pub use utils::{bearing_to, calculate_distance};
