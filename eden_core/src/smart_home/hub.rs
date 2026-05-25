//! # SMART_HOME Hub - Central hub for all smart home devices
//!
//! Hub central que unifica Zigbee y WiFi devices.
//! Implementación 100% original.
#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::wifi::{WifiDevice, WifiDiscovery};
use super::zigbee::{ZigbeeDevice, ZigbeeDiscovery};

/// Hub principal de smart home
pub struct SmartHomeHub {
    name: String,
    zigbee: ZigbeeDiscovery,
    wifi: WifiDiscovery,
    scenes: HashMap<String, Scene>,
    schedules: Vec<Schedule>,
    rules: Vec<Rule>,
}

impl SmartHomeHub {
    /// Crea nuevo hub
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            zigbee: ZigbeeDiscovery::new(),
            wifi: WifiDiscovery::new(),
            scenes: HashMap::new(),
            schedules: Vec::new(),
            rules: Vec::new(),
        }
    }

    // === ZIGBEE OPERATIONS ===

    /// Inicia escaneo Zigbee
    pub fn zigbee_start_scan(&mut self) {
        self.zigbee.start_scan();
    }

    /// Finaliza escaneo Zigbee
    pub fn zigbee_finish_scan(&mut self) -> usize {
        let devices = self.zigbee.finish_scan();
        devices.len()
    }

    /// Lista dispositivos Zigbee
    pub fn zigbee_list(&self) -> Vec<&ZigbeeDevice> {
        self.zigbee.list_devices()
    }

    /// Lista luces Zigbee
    pub fn zigbee_list_lights(&mut self) -> Vec<&mut ZigbeeDevice> {
        self.zigbee.list_lights()
    }

    /// Lista cerraduras Zigbee
    pub fn zigbee_list_locks(&mut self) -> Vec<&mut ZigbeeDevice> {
        self.zigbee.list_locks()
    }

    // === WIFI OPERATIONS ===

    /// Añade dispositivo WiFi
    pub fn wifi_add(&mut self, device: WifiDevice) {
        self.wifi.add_device(device);
    }

    /// Lista dispositivos WiFi
    pub fn wifi_list(&self) -> Vec<&WifiDevice> {
        self.wifi.list_all()
    }

    /// Lista luces WiFi
    pub fn wifi_list_lights(&self) -> Vec<&WifiDevice> {
        self.wifi.list_lights()
    }

    // === UNIFIED OPERATIONS ===

    /// Apaga todas las luces (Zigbee + WiFi)
    pub fn all_lights_off(&mut self) -> (usize, usize) {
        let mut zigbee_count = 0;
        let mut wifi_count = 0;

        // Zigbee lights
        for device in self.zigbee.list_lights() {
            device.set_on_off(false);
            zigbee_count += 1;
        }

        // WiFi lights
        for device in self.wifi.list_lights() {
            // Create HTTP request (would be sent over network in real impl)
            let _request = device.http_turn_off();
            wifi_count += 1;
        }

        (zigbee_count, wifi_count)
    }

    /// Enciende todas las luces (Zigbee + WiFi)
    pub fn all_lights_on(&mut self) -> (usize, usize) {
        let mut zigbee_count = 0;
        let mut wifi_count = 0;

        for device in self.zigbee.list_lights() {
            device.set_on_off(true);
            zigbee_count += 1;
        }

        for device in self.wifi.list_lights() {
            let _request = device.http_turn_on();
            wifi_count += 1;
        }

        (zigbee_count, wifi_count)
    }

    /// Bloquea todas las cerraduras (Zigbee + WiFi)
    pub fn all_locks_lock(&mut self) -> (usize, usize) {
        let mut zigbee_count = 0;
        let mut wifi_count = 0;

        for device in self.zigbee.list_locks() {
            device.set_lock(true);
            zigbee_count += 1;
        }

        for device in self.wifi.list_locks() {
            let _request = device.http_turn_off(); // Lock = turn off for WiFi locks
            wifi_count += 1;
        }

        (zigbee_count, wifi_count)
    }

    // === SCENES ===

    /// Crea escena
    pub fn create_scene(&mut self, name: &str, actions: Vec<SceneAction>) {
        let scene = Scene {
            name: name.to_string(),
            actions,
            last_triggered: None,
        };
        self.scenes.insert(name.to_string(), scene);
    }

    /// Activa escena
    pub fn activate_scene(&mut self, name: &str) -> usize {
        let actions = if let Some(scene) = self.scenes.get_mut(name) {
            let _count = scene.actions.len();
            scene.last_triggered = Some(current_timestamp_ms());
            scene.actions.clone()
        } else {
            return 0;
        };

        let count = actions.len();
        for action in actions {
            self.execute_action(&action);
        }
        count
    }

    /// Ejecuta acción individual
    fn execute_action(&mut self, action: &SceneAction) {
        match action {
            SceneAction::ZigbeeLightOn(nwk) => {
                if let Some(device) = self.zigbee.get_device_mut(*nwk) {
                    device.set_on_off(true);
                }
            }
            SceneAction::ZigbeeLightOff(nwk) => {
                if let Some(device) = self.zigbee.get_device_mut(*nwk) {
                    device.set_on_off(false);
                }
            }
            SceneAction::ZigbeeLock(nwk) => {
                if let Some(device) = self.zigbee.get_device_mut(*nwk) {
                    device.set_lock(true);
                }
            }
            SceneAction::ZigbeeUnlock(nwk) => {
                if let Some(device) = self.zigbee.get_device_mut(*nwk) {
                    device.set_lock(false);
                }
            }
            SceneAction::WifiDeviceOn(id) => {
                if let Some(device) = self.wifi.get_mut(id) {
                    device.mark_online();
                }
            }
            SceneAction::WifiDeviceOff(id) => {
                if let Some(device) = self.wifi.get_mut(id) {
                    device.mark_offline();
                }
            }
            SceneAction::SetThermostat(nwk, temp) => {
                if let Some(device) = self.zigbee.get_device_mut(*nwk) {
                    let _ = device.set_thermostat(*temp);
                }
            }
        }
    }

    // === RULES ===

    /// Añade regla
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Evalúa reglas (llamar periódicamente)
    pub fn evaluate_rules(&mut self, context: &RuleContext) {
        let mut actions_to_execute = Vec::new();
        let mut triggered_rules = Vec::new();

        for (idx, rule) in self.rules.iter_mut().enumerate() {
            if rule.enabled && rule.condition.evaluate(context) {
                for action in &rule.actions {
                    actions_to_execute.push(action.clone());
                }
                triggered_rules.push(idx);
            }
        }

        for action in actions_to_execute {
            self.execute_action(&action);
        }

        for idx in triggered_rules {
            self.rules[idx].last_triggered = Some(current_timestamp_ms());
        }
    }

    // === STATS ===

    /// Obtiene estadísticas completas
    pub fn stats(&self) -> HubStats {
        HubStats {
            name: self.name.clone(),
            zigbee_devices: self.zigbee.list_devices().len(),
            wifi_devices: self.wifi.count(),
            scenes: self.scenes.len(),
            rules: self.rules.len(),
            active_rules: self.rules.iter().filter(|r| r.enabled).count(),
        }
    }
}

/// Estadísticas del hub
#[derive(Debug, Clone)]
pub struct HubStats {
    pub name: String,
    pub zigbee_devices: usize,
    pub wifi_devices: usize,
    pub scenes: usize,
    pub rules: usize,
    pub active_rules: usize,
}

/// Una escena es un conjunto de acciones
#[derive(Debug, Clone)]
pub struct Scene {
    pub name: String,
    pub actions: Vec<SceneAction>,
    pub last_triggered: Option<u64>,
}

/// Acción dentro de una escena
#[derive(Debug, Clone)]
pub enum SceneAction {
    ZigbeeLightOn(u16), // Por network address
    ZigbeeLightOff(u16),
    ZigbeeLock(u16),
    ZigbeeUnlock(u16),
    WifiDeviceOn(String), // Por ID
    WifiDeviceOff(String),
    SetThermostat(u16, f32),
}

/// Programa programable
#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: String,
    pub name: String,
    pub scene: String,
    pub time: ScheduleTime,
    pub days: Vec<u8>, // 0=Sun, 6=Sat
    pub enabled: bool,
}

/// Tiempo de schedule
#[derive(Debug, Clone)]
pub enum ScheduleTime {
    HourMinute(u8, u8),
    Sunrise,
    Sunset,
}

/// Regla de automatización
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub condition: RuleCondition,
    pub actions: Vec<SceneAction>,
    pub enabled: bool,
    pub last_triggered: Option<u64>,
}

/// Condición de regla
#[derive(Debug, Clone)]
pub enum RuleCondition {
    TimeAfter(HourMinute),
    TimeBefore(HourMinute),
    DeviceState(String, String), // device_id, expected_state
    ButtonPressed(String),       // button_id
    MotionDetected(String),      // sensor_id
    Always,
}

impl RuleCondition {
    pub fn evaluate(&self, context: &RuleContext) -> bool {
        match self {
            RuleCondition::TimeAfter(time) => context.current_time >= time.to_minutes(),
            RuleCondition::TimeBefore(time) => context.current_time < time.to_minutes(),
            RuleCondition::DeviceState(_, _) => true, // Would check actual state
            RuleCondition::ButtonPressed(_) => false,
            RuleCondition::MotionDetected(_) => false,
            RuleCondition::Always => true,
        }
    }
}

/// Contexto para evaluación de reglas
#[derive(Debug, Clone)]
pub struct RuleContext {
    pub current_time: u32, // Minutes from midnight
    pub motion_sensors: HashMap<String, bool>,
    pub button_states: HashMap<String, bool>,
}

impl Default for RuleContext {
    fn default() -> Self {
        Self {
            current_time: 0,
            motion_sensors: HashMap::new(),
            button_states: HashMap::new(),
        }
    }
}

/// Hora y minuto
#[derive(Debug, Clone)]
pub struct HourMinute(u8, u8);

impl HourMinute {
    pub fn new(hour: u8, minute: u8) -> Self {
        Self(hour.min(23), minute.min(59))
    }

    pub fn to_minutes(&self) -> u32 {
        (self.0 as u32) * 60 + (self.1 as u32)
    }
}

/// Timestamp helper
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
    fn test_hub_creation() {
        let hub = SmartHomeHub::new("EdenHome");
        let stats = hub.stats();
        assert_eq!(stats.name, "EdenHome");
    }

    #[test]
    fn test_all_lights() {
        let mut hub = SmartHomeHub::new("Test");
        // Initially should be 0, 0
        let (z, w) = hub.all_lights_off();
        assert_eq!(z, 0);
        assert_eq!(w, 0);
    }

    #[test]
    fn test_scene_creation() {
        let mut hub = SmartHomeHub::new("Test");
        hub.create_scene(
            "Movie Night",
            vec![SceneAction::WifiDeviceOff("living_room_light".to_string())],
        );
        let count = hub.activate_scene("Movie Night");
        assert_eq!(count, 1);
    }
}
