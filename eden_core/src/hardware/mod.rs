//! # HARDWARE - Physical hardware control
//!
//! Control de hardware físico: GPIO, serial, I2C, SPI.
//! Sin dependencias externas - 100% Rust usando libc.

#![allow(dead_code)]

mod actuator;
mod gpio;
mod i2c;
mod serial;
mod spi;

pub use actuator::{Actuator, Robot, RobotCommand};
pub use gpio::{GpioMode, GpioPin, GpioState};
pub use i2c::I2CBus;
pub use serial::{SerialConfig, SerialPort};
pub use spi::SpiBus;

/// Tipo de máquina - detecta en runtime
#[derive(Debug, Clone, Copy)]
pub enum MachineType {
    Unknown,
    RaspberryPi,
    BeagleBone,
    Arduino,
    GenericLinux,
}

impl MachineType {
    /// Detecta el tipo de máquina
    pub fn detect() -> Self {
        // Check /proc/device-tree/model
        if let Ok(model) = std::fs::read_to_string("/proc/device-tree/model") {
            let model_lower = model.to_lowercase();
            if model_lower.contains("raspberry") {
                return MachineType::RaspberryPi;
            } else if model_lower.contains("beaglebone") || model_lower.contains("beagle") {
                return MachineType::BeagleBone;
            }
        }

        // Check /sys/class/gpio
        if std::path::Path::new("/sys/class/gpio").exists() {
            return MachineType::GenericLinux;
        }

        MachineType::Unknown
    }
}

impl Default for MachineType {
    fn default() -> Self {
        Self::detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_detection() {
        let mt = MachineType::detect();
        println!("Detected: {:?}", mt);
    }
}
