//! # Actuator - High-level actuator control
//!
//! Control unificado de actuadores (motores, servos, relés).

#![allow(dead_code)]

use super::{GpioPin, GpioState, I2CBus};
use std::time::Duration;

/// Tipo de actuador
#[derive(Debug, Clone)]
pub enum ActuatorType {
    GpioPin(u32),
    Servo {
        i2c_bus: u8,
        addr: u8,
    },
    Stepper {
        serial_path: String,
        steps_per_rev: u32,
    },
    Relay {
        gpio_pin: u32,
    },
}

/// Estado de un actuador
#[derive(Debug, Clone, Copy)]
pub enum ActuatorState {
    Off,
    On,
    Position(f32), // Para servos y steppers (0.0 - 1.0)
}

/// Un actuador genérico
pub struct Actuator {
    kind: ActuatorType,
    state: ActuatorState,
}

impl Actuator {
    /// Crea actuador GPIO (LED, relé, etc)
    pub fn gpio(pin: u32) -> Result<Self, ActuatorError> {
        let _gpio = GpioPin::new(pin)?;
        Ok(Self {
            kind: ActuatorType::GpioPin(pin),
            state: ActuatorState::Off,
        })
    }

    /// Crea actuador relay
    pub fn relay(pin: u32) -> Result<Self, ActuatorError> {
        let gpio = GpioPin::new(pin)?;
        gpio.write(GpioState::High)?; // Relay OFF initially (active low)
        Ok(Self {
            kind: ActuatorType::Relay { gpio_pin: pin },
            state: ActuatorState::Off,
        })
    }

    /// Crea actuador servo via I2C (PCA9685, etc)
    pub fn servo(i2c_bus: u8, addr: u8) -> Self {
        Self {
            kind: ActuatorType::Servo { i2c_bus, addr },
            state: ActuatorState::Position(0.5),
        }
    }

    /// Crea actuador stepper via serial
    pub fn stepper(serial_path: &str, steps_per_rev: u32) -> Self {
        Self {
            kind: ActuatorType::Stepper {
                serial_path: serial_path.to_string(),
                steps_per_rev,
            },
            state: ActuatorState::Position(0.0),
        }
    }

    /// Enciende el actuador
    pub fn on(&mut self) -> Result<(), ActuatorError> {
        match &self.kind {
            ActuatorType::GpioPin(_) => {
                // GpioPin ya fue creado, buscar manera de actualizar
                // Por ahora solo cambiamos estado
                self.state = ActuatorState::On;
            }
            ActuatorType::Relay { .. } => {
                // TODO: implementar con GpioPin guardado
                self.state = ActuatorState::On;
            }
            _ => {}
        }
        Ok(())
    }

    /// Apaga el actuador
    pub fn off(&mut self) -> Result<(), ActuatorError> {
        self.state = ActuatorState::Off;
        Ok(())
    }

    /// Mueve a posición (0.0 - 1.0)
    pub fn set_position(&mut self, pos: f32) -> Result<(), ActuatorError> {
        let pos = pos.clamp(0.0, 1.0);

        match &self.kind {
            ActuatorType::Servo { i2c_bus, addr } => {
                // Enviar señal PWM al servo
                let mut bus = I2CBus::new(*i2c_bus)?;

                // Convertir posición a pulsos (500-2500 µs típicamente)
                let pulse_us = 500 + (pos * 2000.0) as u32;

                // El PCA9685 usa 4096 ticks por ciclo de 20ms
                // tick = pulse_us * 4096 / 20000
                let tick = (pulse_us as f32 * 4096.0 / 20000.0) as u16;

                bus.write_register(*addr, 0xFA, (tick & 0xFF) as u8)?;
                bus.write_register(*addr, 0xFB, ((tick >> 8) & 0xFF) as u8)?;
            }
            _ => {}
        }

        self.state = ActuatorState::Position(pos);
        Ok(())
    }

    /// Obtiene estado actual
    pub fn state(&self) -> ActuatorState {
        self.state
    }
}

/// Error de actuador
#[derive(Debug)]
pub enum ActuatorError {
    Gpio(super::gpio::GpioError),
    Serial(std::io::Error),
    I2C(std::io::Error),
    InvalidState(String),
}

impl std::fmt::Display for ActuatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActuatorError::Gpio(e) => write!(f, "GPIO error: {}", e),
            ActuatorError::Serial(e) => write!(f, "Serial error: {}", e),
            ActuatorError::I2C(e) => write!(f, "I2C error: {}", e),
            ActuatorError::InvalidState(s) => write!(f, "Invalid state: {}", s),
        }
    }
}

impl From<super::gpio::GpioError> for ActuatorError {
    fn from(e: super::gpio::GpioError) -> Self {
        ActuatorError::Gpio(e)
    }
}

impl From<std::io::Error> for ActuatorError {
    fn from(e: std::io::Error) -> Self {
        ActuatorError::I2C(e)
    }
}

/// Robot simple con múltiples actuadores
pub struct Robot {
    actuators: Vec<Actuator>,
}

impl Robot {
    pub fn new() -> Self {
        Self {
            actuators: Vec::new(),
        }
    }

    pub fn add_actuator(&mut self, actuator: Actuator) {
        self.actuators.push(actuator);
    }

    /// Ejecuta una secuencia de movimientos
    pub fn execute_sequence(&mut self, sequence: &[RobotCommand]) -> Result<(), ActuatorError> {
        for cmd in sequence {
            match cmd {
                RobotCommand::ActuatorOn(idx) => {
                    self.actuators
                        .get_mut(*idx)
                        .ok_or_else(|| ActuatorError::InvalidState("Invalid actuator".to_string()))?
                        .on()?;
                }
                RobotCommand::ActuatorOff(idx) => {
                    self.actuators
                        .get_mut(*idx)
                        .ok_or_else(|| ActuatorError::InvalidState("Invalid actuator".to_string()))?
                        .off()?;
                }
                RobotCommand::ActuatorPosition(idx, pos) => {
                    self.actuators
                        .get_mut(*idx)
                        .ok_or_else(|| ActuatorError::InvalidState("Invalid actuator".to_string()))?
                        .set_position(*pos)?;
                }
                RobotCommand::Delay(ms) => {
                    std::thread::sleep(Duration::from_millis(*ms));
                }
            }
        }
        Ok(())
    }
}

impl Default for Robot {
    fn default() -> Self {
        Self::new()
    }
}

/// Comando para el robot
#[derive(Debug, Clone)]
pub enum RobotCommand {
    ActuatorOn(usize),
    ActuatorOff(usize),
    ActuatorPosition(usize, f32),
    Delay(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robot_creation() {
        let robot = Robot::new();
        assert!(robot.actuators.is_empty());
    }
}
