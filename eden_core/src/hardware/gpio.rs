//! # GPIO - General Purpose Input/Output
//!
//! Control de pines GPIO via sysfs o character device.

#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

/// Modo de un pin GPIO
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpioMode {
    Input,
    Output,
    InputPullUp,
    InputPullDown,
}

impl GpioMode {
    fn to_sysfs_mode(&self) -> &str {
        match self {
            GpioMode::Input => "in",
            GpioMode::Output => "out",
            GpioMode::InputPullUp => "in",
            GpioMode::InputPullDown => "in",
        }
    }
}

/// Estado lógico de un pin
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpioState {
    High,
    Low,
}

impl GpioState {
    pub fn as_str(&self) -> &str {
        match self {
            GpioState::High => "1",
            GpioState::Low => "0",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "1" => Some(GpioState::High),
            "0" => Some(GpioState::Low),
            _ => None,
        }
    }
}

/// Un pin GPIO
pub struct GpioPin {
    pin: u32,
    base: PathBuf,
    path: PathBuf,
}

impl GpioPin {
    /// Exporta y configura un pin
    pub fn new(pin: u32) -> Result<Self, GpioError> {
        let base = PathBuf::from("/sys/class/gpio");
        Self::new_at(pin, base)
    }

    /// Exporta y configura un pin en una raíz sysfs específica.
    ///
    /// Esto permite probar la semántica GPIO con un árbol local sin tocar
    /// hardware real. En producción se usa `/sys/class/gpio`.
    pub fn new_at(pin: u32, base: PathBuf) -> Result<Self, GpioError> {
        // Exportar pin
        let export_path = base.join("export");
        fs::write(&export_path, pin.to_string())
            .map_err(|e| GpioError::ExportFailed(pin, e.to_string()))?;

        let path = base.join(format!("gpio{}", pin));

        // Esperar que aparezca
        std::thread::sleep(std::time::Duration::from_millis(100));

        if !path.exists() {
            return Err(GpioError::PinNotFound(pin));
        }

        let mut gpio = Self { pin, base, path };
        gpio.set_mode(GpioMode::Output)?;

        Ok(gpio)
    }

    /// Configura el modo del pin
    pub fn set_mode(&mut self, mode: GpioMode) -> Result<(), GpioError> {
        let direction_path = self.path.join("direction");
        fs::write(&direction_path, mode.to_sysfs_mode())
            .map_err(|e| GpioError::WriteFailed(e.to_string()))?;

        // Pull-ups downs via edge
        if mode == GpioMode::InputPullUp || mode == GpioMode::InputPullDown {
            let active_low_path = self.path.join("active_low");
            fs::write(
                &active_low_path,
                if mode == GpioMode::InputPullUp {
                    "1"
                } else {
                    "0"
                },
            )
            .ok();
        }

        Ok(())
    }

    /// Escribe un estado
    pub fn write(&self, state: GpioState) -> Result<(), GpioError> {
        let value_path = self.path.join("value");
        fs::write(&value_path, state.as_str()).map_err(|e| GpioError::WriteFailed(e.to_string()))
    }

    /// Lee el estado actual
    pub fn read(&self) -> Result<GpioState, GpioError> {
        let value_path = self.path.join("value");
        let data =
            fs::read_to_string(&value_path).map_err(|e| GpioError::ReadFailed(e.to_string()))?;

        GpioState::from_str(&data).ok_or_else(|| GpioError::InvalidState(data))
    }

    /// Libera el pin
    pub fn release(&mut self) -> Result<(), GpioError> {
        let unexport_path = self.base.join("unexport");
        fs::write(&unexport_path, self.pin.to_string())
            .map_err(|e| GpioError::WriteFailed(e.to_string()))
    }
}

impl Drop for GpioPin {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Errors del GPIO
#[derive(Debug)]
pub enum GpioError {
    ExportFailed(u32, String),
    PinNotFound(u32),
    WriteFailed(String),
    ReadFailed(String),
    InvalidState(String),
}

impl std::fmt::Display for GpioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpioError::ExportFailed(pin, msg) => {
                write!(f, "Failed to export GPIO {}: {}", pin, msg)
            }
            GpioError::PinNotFound(pin) => write!(f, "GPIO {} not found", pin),
            GpioError::WriteFailed(msg) => write!(f, "Write failed: {}", msg),
            GpioError::ReadFailed(msg) => write!(f, "Read failed: {}", msg),
            GpioError::InvalidState(s) => write!(f, "Invalid GPIO state: {}", s),
        }
    }
}

impl std::error::Error for GpioError {}

/// Manager de múltiples pines GPIO
pub struct GpioManager {
    pins: Vec<GpioPin>,
}

impl GpioManager {
    pub fn new() -> Self {
        Self { pins: Vec::new() }
    }

    pub fn export_pin(&mut self, pin: u32) -> Result<&mut GpioPin, GpioError> {
        let gpio = GpioPin::new(pin)?;
        self.pins.push(gpio);
        Ok(self.pins.last_mut().unwrap())
    }

    pub fn release_all(&mut self) {
        // GpioPin's Drop impl will handle release automatically
        self.pins.clear();
    }
}

impl Default for GpioManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpio_mock_sysfs_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().to_path_buf();
        fs::write(base.join("export"), "").unwrap();
        fs::write(base.join("unexport"), "").unwrap();

        let pin_dir = base.join("gpio4");
        fs::create_dir(&pin_dir).unwrap();
        fs::write(pin_dir.join("direction"), "").unwrap();
        fs::write(pin_dir.join("value"), "0").unwrap();
        fs::write(pin_dir.join("active_low"), "0").unwrap();

        let mut pin = GpioPin::new_at(4, base.clone()).unwrap();
        pin.set_mode(GpioMode::Output).unwrap();
        pin.write(GpioState::High).unwrap();

        assert_eq!(pin.read().unwrap(), GpioState::High);
        assert_eq!(fs::read_to_string(base.join("export")).unwrap(), "4");

        pin.release().unwrap();
        assert_eq!(fs::read_to_string(base.join("unexport")).unwrap(), "4");
    }

    #[test]
    #[cfg_attr(not(feature = "external-tests"), ignore)]
    fn test_gpio_export() {
        let pin = GpioPin::new(4);
        if pin.is_ok() {
            pin.unwrap().release().unwrap();
        }
    }
}
