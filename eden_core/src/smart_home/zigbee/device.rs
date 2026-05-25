//! # ZIGBEE Device - Device instance management
//!
//! Representa un dispositivo Zigbee individual en la red.

#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::clusters::{IasZoneMessage, ThermostatCommands, ZigbeeDeviceInfo};
use super::zcl::{AttributeValue, ClusterId, ZigbeeClusterLibrary};

/// Estado de conexión del dispositivo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionState {
    Online,
    Offline,
    Pending,
}

/// Comando pendiente
#[derive(Debug, Clone)]
pub struct PendingCommand {
    pub cluster: ClusterId,
    pub command: Vec<u8>,
    pub retries: u8,
    pub timestamp_ms: u64,
}

/// Dispositivo Zigbee individual
pub struct ZigbeeDevice {
    pub info: ZigbeeDeviceInfo,
    zcl: ZigbeeClusterLibrary,
    pub state: ConnectionState,
    attributes: HashMap<u16, AttributeValue>,
    pending_commands: Vec<PendingCommand>,
    last_seen: u64,
    battery_level: Option<u8>,
}

impl ZigbeeDevice {
    /// Crea nuevo dispositivo
    pub fn new(info: ZigbeeDeviceInfo) -> Self {
        Self {
            info,
            zcl: ZigbeeClusterLibrary::new(),
            state: ConnectionState::Pending,
            attributes: HashMap::new(),
            pending_commands: Vec::new(),
            last_seen: current_timestamp_ms(),
            battery_level: None,
        }
    }

    /// Actualiza timestamp de última vez visto
    pub fn mark_seen(&mut self) {
        self.last_seen = current_timestamp_ms();
        if self.state == ConnectionState::Pending {
            self.state = ConnectionState::Online;
        }
    }

    /// Obtiene estado de conexión
    pub fn connection_state(&self) -> ConnectionState {
        // Si no se ha visto en 60 segundos, está offline
        if current_timestamp_ms() - self.last_seen > 60_000 {
            ConnectionState::Offline
        } else {
            self.state
        }
    }

    /// Actualiza atributo
    pub fn update_attribute(&mut self, attr_id: u16, value: AttributeValue) {
        self.attributes.insert(attr_id, value);
    }

    /// Obtiene atributo
    pub fn get_attribute(&self, attr_id: u16) -> Option<&AttributeValue> {
        self.attributes.get(&attr_id)
    }

    /// Comando: encender/apagar (para luces)
    pub fn set_on_off(&mut self, on: bool) -> Vec<u8> {
        let frame = self.zcl.on_off_command(on);
        self.queue_command(ClusterId::OnOff, frame.clone());
        frame
    }

    /// Comando: brillo (para dimmers)
    pub fn set_level(&mut self, level: u8, transition: Option<u16>) -> Vec<u8> {
        let frame = self.zcl.level_command(level, transition);
        self.queue_command(ClusterId::LevelControl, frame.clone());
        frame
    }

    /// Comando: color (para RGB)
    pub fn set_color(&mut self, hue: u8, saturation: u8) -> Vec<u8> {
        let frame = self.zcl.color_command(hue, saturation);
        self.queue_command(ClusterId::ColorControl, frame.clone());
        frame
    }

    /// Comando: bloquear/desbloquear (para cerraduras)
    pub fn set_lock(&mut self, lock: bool) -> Vec<u8> {
        let frame = self.zcl.door_lock_command(lock);
        self.queue_command(ClusterId::DoorLock, frame.clone());
        frame
    }

    /// Comando: termostato (para termostatos)
    pub fn set_thermostat(&mut self, temp_celsius: f32) -> Vec<u8> {
        let frame = ThermostatCommands::set_temperature(&mut self.zcl, temp_celsius);
        self.queue_command(ClusterId::Thermostat, frame.clone());
        frame
    }

    /// Añade comando pendiente
    pub fn queue_command(&mut self, cluster: ClusterId, command: Vec<u8>) {
        self.pending_commands.push(PendingCommand {
            cluster,
            command,
            retries: 0,
            timestamp_ms: current_timestamp_ms(),
        });
    }

    /// Obtiene siguiente comando pendiente
    pub fn next_pending(&mut self) -> Option<(ClusterId, Vec<u8>)> {
        if let Some(cmd) = self.pending_commands.first() {
            Some((cmd.cluster, cmd.command.clone()))
        } else {
            None
        }
    }

    /// Confirma comando completado
    pub fn confirm_command(&mut self) {
        if let Some(_) = self.pending_commands.first() {
            self.pending_commands.remove(0);
        }
    }

    /// Retry comando fallido
    pub fn retry_pending(&mut self) -> Option<(ClusterId, Vec<u8>)> {
        if let Some(cmd) = self.pending_commands.first_mut() {
            cmd.retries += 1;
            if cmd.retries < 3 {
                return Some((cmd.cluster, cmd.command.clone()));
            }
        }
        None
    }

    /// Procesa mensaje IAS Zone (sensor)
    pub fn process_ias_zone(&mut self, payload: &[u8]) -> Option<IasZoneMessage> {
        let msg = IasZoneMessage::parse(payload)?;
        // Check battery bit
        if msg.is_low_battery() {
            self.battery_level = Some(10); // Low battery indicator
        }
        Some(msg)
    }

    /// Obtiene información de batería
    pub fn battery_level(&self) -> Option<u8> {
        self.battery_level
    }

    /// Devuelve todos los atributos como HashMap
    pub fn all_attributes(&self) -> &HashMap<u16, AttributeValue> {
        &self.attributes
    }
}

/// Timestamp actual en ms
fn current_timestamp_ms() -> u64 {
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
        let info = ZigbeeDeviceInfo::new(0x123456789ABCDEF0, 0x1234, 1);
        let device = ZigbeeDevice::new(info);
        assert_eq!(device.connection_state(), ConnectionState::Pending);
    }

    #[test]
    fn test_set_on_off() {
        let info = ZigbeeDeviceInfo::new(0x123456789ABCDEF0, 0x1234, 1);
        let mut device = ZigbeeDevice::new(info);
        let frame = device.set_on_off(true);
        assert!(!frame.is_empty());
    }
}
