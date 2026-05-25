//! # Smart Device - Unified device abstraction
//!
//! Abstracción unificada para todos los tipos de dispositivos smart home.
//! 100% original, sin dependencias externas.

#![allow(dead_code)]

use std::collections::HashMap;

/// Tipo genérico de dispositivo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    Light,
    Dimmer,
    ColorLight,
    Switch,
    Plug,
    Lock,
    Thermostat,
    Sensor,
    Camera,
    Fan,
    Blinds,
    Doorbell,
    MotionSensor,
    ContactSensor,
    Unknown,
}

impl DeviceType {
    /// Nombre legible
    pub fn name(&self) -> &'static str {
        match self {
            DeviceType::Light => "Light",
            DeviceType::Dimmer => "Dimmer",
            DeviceType::ColorLight => "Color Light",
            DeviceType::Switch => "Switch",
            DeviceType::Plug => "Smart Plug",
            DeviceType::Lock => "Lock",
            DeviceType::Thermostat => "Thermostat",
            DeviceType::Sensor => "Sensor",
            DeviceType::Camera => "Camera",
            DeviceType::Fan => "Fan",
            DeviceType::Blinds => "Blinds",
            DeviceType::Doorbell => "Doorbell",
            DeviceType::MotionSensor => "Motion Sensor",
            DeviceType::ContactSensor => "Contact Sensor",
            DeviceType::Unknown => "Unknown",
        }
    }

    /// Indica si puede ser controlado (on/off)
    pub fn is_controllable(&self) -> bool {
        matches!(
            self,
            DeviceType::Light
                | DeviceType::Dimmer
                | DeviceType::ColorLight
                | DeviceType::Switch
                | DeviceType::Plug
                | DeviceType::Lock
                | DeviceType::Thermostat
                | DeviceType::Fan
                | DeviceType::Blinds
        )
    }

    /// Indica si es un sensor (solo lectura)
    pub fn is_sensor(&self) -> bool {
        matches!(
            self,
            DeviceType::Sensor
                | DeviceType::MotionSensor
                | DeviceType::ContactSensor
                | DeviceType::Camera
        )
    }
}

/// Estado del dispositivo
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceState {
    On,
    Off,
    Locked,
    Unlocked,
    Open,
    Closed,
    Moving, // Para blinds, fans
    Idle,
    Error(String),
    Unknown,
}

impl DeviceState {
    /// Convierte a booleano (solo para On/Off)
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            DeviceState::On => Some(true),
            DeviceState::Off => Some(false),
            _ => None,
        }
    }

    /// Crea desde booleano
    pub fn from_bool(on: bool) -> Self {
        if on {
            DeviceState::On
        } else {
            DeviceState::Off
        }
    }
}

/// Dispositivo smart home genérico
#[derive(Debug, Clone)]
pub struct SmartDevice {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub state: DeviceState,
    pub room: Option<String>,
    pub protocol: String, // "zigbee", "wifi", "bluetooth"
    pub address: String,  // Depende del protocolo
    pub attributes: HashMap<String, String>,
    pub capabilities: Vec<Capability>,
    pub battery: Option<u8>, // Porcentaje
    pub last_seen: u64,
}

impl SmartDevice {
    /// Crea nuevo dispositivo
    pub fn new(id: &str, name: &str, device_type: DeviceType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            device_type,
            state: DeviceState::Unknown,
            room: None,
            protocol: "unknown".to_string(),
            address: String::new(),
            attributes: HashMap::new(),
            capabilities: Vec::new(),
            battery: None,
            last_seen: 0,
        }
    }

    /// Establece habitación
    pub fn set_room(&mut self, room: &str) {
        self.room = Some(room.to_string());
    }

    /// Establece protocolo y dirección
    pub fn set_connection(&mut self, protocol: &str, address: &str) {
        self.protocol = protocol.to_string();
        self.address = address.to_string();
    }

    /// Establece estado
    pub fn set_state(&mut self, state: DeviceState) {
        self.state = state;
        self.last_seen = current_time_ms();
    }

    /// Añade atributo
    pub fn set_attr(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    /// Obtiene atributo
    pub fn get_attr(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Añade capacidad
    pub fn add_capability(&mut self, cap: Capability) {
        self.capabilities.push(cap);
    }

    /// Indica si tiene cierta capacidad
    pub fn has_capability(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Actualiza nivel de batería
    pub fn set_battery(&mut self, level: u8) {
        self.battery = Some(level.min(100));
    }

    /// Verifica si está online
    pub fn is_online(&self) -> bool {
        current_time_ms() - self.last_seen < 60_000 // 60 segundos
    }

    /// Comando: encender
    pub fn turn_on(&mut self) -> bool {
        if self.device_type.is_controllable() {
            self.state = DeviceState::On;
            true
        } else {
            false
        }
    }

    /// Comando: apagar
    pub fn turn_off(&mut self) -> bool {
        if self.device_type.is_controllable() {
            self.state = DeviceState::Off;
            true
        } else {
            false
        }
    }

    /// Comando: bloquear
    pub fn lock(&mut self) -> bool {
        if matches!(self.device_type, DeviceType::Lock) {
            self.state = DeviceState::Locked;
            true
        } else {
            false
        }
    }

    /// Comando: desbloquear
    pub fn unlock(&mut self) -> bool {
        if matches!(self.device_type, DeviceType::Lock) {
            self.state = DeviceState::Unlocked;
            true
        } else {
            false
        }
    }

    /// Obtiene estado como string
    pub fn state_string(&self) -> String {
        format!("{:?}", self.state)
    }

    /// Obtiene descripción completa
    pub fn description(&self) -> String {
        let location = self.room.as_deref().unwrap_or("Unassigned");
        format!(
            "{} ({:?}) in {} - {:?}",
            self.name, self.device_type, location, self.state
        )
    }
}

/// Capacidad de un dispositivo
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    OnOff,
    Dimming,
    ColorRGB,
    ColorTemperature,
    Temperature,
    Humidity,
    Motion,
    Contact,
    Lock,
    Battery,
    SignalStrength,
    FirmwareUpdate,
}

impl Capability {
    /// Nombre legible
    pub fn name(&self) -> &'static str {
        match self {
            Capability::OnOff => "On/Off",
            Capability::Dimming => "Dimming",
            Capability::ColorRGB => "RGB Color",
            Capability::ColorTemperature => "Color Temperature",
            Capability::Temperature => "Temperature",
            Capability::Humidity => "Humidity",
            Capability::Motion => "Motion Detection",
            Capability::Contact => "Contact Detection",
            Capability::Lock => "Lock",
            Capability::Battery => "Battery Level",
            Capability::SignalStrength => "Signal Strength",
            Capability::FirmwareUpdate => "Firmware Update",
        }
    }
}

/// Timestamp helper
fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let mut device = SmartDevice::new("light-1", "Living Room Light", DeviceType::Light);
        device.set_room("Living Room");
        device.set_connection("zigbee", "0x1234");

        assert!(device.turn_on());
        assert_eq!(device.state, DeviceState::On);
    }

    #[test]
    fn test_device_type() {
        let sensor = SmartDevice::new("motion-1", "Motion", DeviceType::MotionSensor);
        assert!(sensor.device_type.is_sensor());
        assert!(!sensor.device_type.is_controllable());
    }

    #[test]
    fn test_capabilities() {
        let mut device = SmartDevice::new("bulb-1", "Bulb", DeviceType::ColorLight);
        device.add_capability(Capability::OnOff);
        device.add_capability(Capability::Dimming);
        device.add_capability(Capability::ColorRGB);

        assert!(device.has_capability(&Capability::OnOff));
        assert!(device.has_capability(&Capability::Dimming));
        assert!(!device.has_capability(&Capability::Temperature));
    }
}
