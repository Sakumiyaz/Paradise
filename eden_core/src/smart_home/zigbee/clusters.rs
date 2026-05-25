//! # ZIGBEE Clusters - Device type definitions
//!
//! Implementación de clusters específicos por tipo de dispositivo.
//! 100% original, basado en Zigbee ZCL specification.

#![allow(dead_code)]

use super::zcl::{AttributeValue, ClusterId, ZigbeeClusterLibrary};

/// Tipo de dispositivo Zigbee
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    Light,
    Dimmer,
    ColorLight,
    Thermostat,
    DoorLock,
    MotionSensor,
    ContactSensor,
    SmartPlug,
    Router,
    Unknown,
}

impl DeviceType {
    /// Detecta el tipo de dispositivo según clusters que soporta
    pub fn from_clusters(clusters: &[u16]) -> Self {
        let has_on_off = clusters.contains(&0x0006);
        let has_level = clusters.contains(&0x0008);
        let has_color = clusters.contains(&0x0300);
        let has_thermostat = clusters.contains(&0x0201);
        let has_door_lock = clusters.contains(&0x0101);
        let has_occupancy = clusters.contains(&0x0406);

        if has_door_lock {
            DeviceType::DoorLock
        } else if has_thermostat {
            DeviceType::Thermostat
        } else if has_color {
            DeviceType::ColorLight
        } else if has_level {
            DeviceType::Dimmer
        } else if has_on_off {
            DeviceType::Light
        } else if has_occupancy {
            DeviceType::MotionSensor
        } else {
            DeviceType::Unknown
        }
    }
}

/// Comando específico para luz
pub struct LightCommands;

impl LightCommands {
    /// Enciende la luz
    pub fn turn_on(zcl: &mut ZigbeeClusterLibrary) -> Vec<u8> {
        zcl.on_off_command(true)
    }

    /// Apaga la luz
    pub fn turn_off(zcl: &mut ZigbeeClusterLibrary) -> Vec<u8> {
        zcl.on_off_command(false)
    }

    /// Establece brillo (0-100%)
    pub fn set_brightness(zcl: &mut ZigbeeClusterLibrary, percent: u8) -> Vec<u8> {
        // LevelControl: 0-254, donde 254 = 100%
        let level = (percent as f32 * 2.54) as u8;
        zcl.level_command(level.max(1), Some(10))
    }

    /// Establece color (hue: 0-360, saturation: 0-100)
    pub fn set_color(zcl: &mut ZigbeeClusterLibrary, hue: u8, saturation: u8) -> Vec<u8> {
        // Convertir hue (0-360) a uint8 (0-254)
        let hue_zigbee = ((hue as f32 / 360.0) * 254.0) as u8;
        let sat_zigbee = ((saturation as f32 / 100.0) * 254.0) as u8;
        zcl.color_command(hue_zigbee, sat_zigbee)
    }
}

/// Comando específico para termostato
pub struct ThermostatCommands;

impl ThermostatCommands {
    /// Establece temperatura objetivo (en grados Celsius * 100)
    pub fn set_temperature(zcl: &mut ZigbeeClusterLibrary, temp_celsius: f32) -> Vec<u8> {
        // Occupancy cluster attribute for occupied setpoint
        let temp_raw = (temp_celsius * 100.0) as i16;
        zcl.write_attributes(
            ClusterId::Thermostat,
            &[(0x0012, AttributeValue::Int16(temp_raw))], // Local temperature
        )
    }

    /// Lee temperatura actual
    pub fn read_temperature(zcl: &mut ZigbeeClusterLibrary) -> Vec<u8> {
        zcl.read_attributes(ClusterId::Temperature, &[0x0000]) // Local temperature
    }

    /// Establece modo (0=off, 1=heat, 2=cool, 3=auto)
    pub fn set_mode(zcl: &mut ZigbeeClusterLibrary, mode: u8) -> Vec<u8> {
        zcl.write_attributes(
            ClusterId::Thermostat,
            &[(0x001C, AttributeValue::Enum8(mode))],
        )
    }
}

/// Comando específico para cerradura
pub struct DoorLockCommands;

impl DoorLockCommands {
    /// Bloquea la cerradura
    pub fn lock(zcl: &mut ZigbeeClusterLibrary) -> Vec<u8> {
        zcl.door_lock_command(true)
    }

    /// Desbloquea la cerradura
    pub fn unlock(zcl: &mut ZigbeeClusterLibrary) -> Vec<u8> {
        zcl.door_lock_command(false)
    }
}

/// Mensaje de alarma del IAS Zone (sensor de movimiento/contacto)
#[derive(Debug, Clone)]
pub struct IasZoneMessage {
    pub zone_status: u16,
    pub extended_status: u8,
}

impl IasZoneMessage {
    /// Parsea mensaje IAS Zone
    pub fn parse(payload: &[u8]) -> Option<Self> {
        if payload.len() < 3 {
            return None;
        }
        Some(IasZoneMessage {
            zone_status: u16::from_le_bytes([payload[0], payload[1]]),
            extended_status: payload[2],
        })
    }

    /// Indica si el sensor está en alarma (movimiento o contacto abierto)
    pub fn is_alarm(&self) -> bool {
        (self.zone_status & 0x0001) != 0
    }

    /// Indica si el sensor estáendido bajo
    pub fn is_low_battery(&self) -> bool {
        (self.zone_status & 0x0002) != 0
    }

    /// Indica si hay sabotaje
    pub fn is_tamper(&self) -> bool {
        (self.zone_status & 0x0004) != 0
    }
}

/// Información de dispositivo Zigbee
#[derive(Debug, Clone)]
pub struct ZigbeeDeviceInfo {
    pub ieee_addr: u64,
    pub network_addr: u16,
    pub device_type: DeviceType,
    pub clusters: Vec<u16>,
    pub endpoint: u8,
}

impl ZigbeeDeviceInfo {
    /// Crea desde datos crudos (IEEE address y otros)
    pub fn new(ieee: u64, nwk: u16, endpoint: u8) -> Self {
        Self {
            ieee_addr: ieee,
            network_addr: nwk,
            device_type: DeviceType::Unknown,
            clusters: vec![],
            endpoint,
        }
    }

    /// Detecta tipo de dispositivo basado en clusters
    pub fn detect_device_type(&mut self) {
        self.device_type = DeviceType::from_clusters(&self.clusters);
    }
}

/// Atributos específicos por tipo de dispositivo
pub struct DeviceAttributes;

impl DeviceAttributes {
    /// Atributos de un bombillo inteligente
    pub const ON_OFF: u16 = 0x0000;
    pub const LEVEL: u16 = 0x0000; // Current level
    pub const HUE: u16 = 0x0000; // Current hue
    pub const SATURATION: u16 = 0x0001; // Current saturation

    /// Atributos de termostato
    pub const LOCAL_TEMP: u16 = 0x0012;
    pub const OCCUPIED_SETPOINT: u16 = 0x0013;
    pub const SYSTEM_MODE: u16 = 0x001C;
    pub const PI_COOLING_DEMAND: u16 = 0x0025;
    pub const PI_HEATING_DEMAND: u16 = 0x0026;

    /// Atributos de cerradura
    pub const LOCK_STATE: u16 = 0x0000;
    pub const LOCK_TYPE: u16 = 0x0001;
    pub const DOOR_STATE: u16 = 0x0004;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_on_off() {
        let mut zcl = ZigbeeClusterLibrary::new();
        let on_frame = LightCommands::turn_on(&mut zcl);
        let off_frame = LightCommands::turn_off(&mut zcl);
        assert!(!on_frame.is_empty());
        assert!(!off_frame.is_empty());
    }

    #[test]
    fn test_brightness() {
        let mut zcl = ZigbeeClusterLibrary::new();
        let frame = LightCommands::set_brightness(&mut zcl, 75);
        assert!(!frame.is_empty());
    }

    #[test]
    fn test_door_lock() {
        let mut zcl = ZigbeeClusterLibrary::new();
        let lock_frame = DoorLockCommands::lock(&mut zcl);
        let unlock_frame = DoorLockCommands::unlock(&mut zcl);
        assert!(!lock_frame.is_empty());
        assert!(!unlock_frame.is_empty());
    }

    #[test]
    fn test_ias_zone_parse() {
        let payload = vec![0x01, 0x00, 0x00];
        let msg = IasZoneMessage::parse(&payload).unwrap();
        assert!(msg.is_alarm());
    }
}
