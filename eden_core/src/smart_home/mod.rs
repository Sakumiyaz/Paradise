//! # SMART_HOME - Smart Home Hub Integration
//!
//! Control de dispositivos smart home: luces, cerraduras, termostatos.
//! Implementación 100% original - protocolos Zigbee ZCL y WiFi desde cero.
//! Sin dependencias externas.

#![allow(dead_code)]

pub mod device;
pub mod hub;
pub mod wifi;
pub mod zigbee;

// Re-exports
pub use device::{DeviceState, DeviceType, SmartDevice};
pub use hub::{Rule, Scene, Schedule, SmartHomeHub};
pub use wifi::WifiDevice;
pub use zigbee::ZigbeeDevice;
