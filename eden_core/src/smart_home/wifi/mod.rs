//! # WiFi Smart Devices - WiFi device protocol implementation
//!
//! Implementación de protocolos para dispositivos WiFi smart home.
//! Soporta: HTTP REST, MQTT, mDNS discovery.
//! 100% original, sin dependencias externas.

#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Tipo de protocolo usado
#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    Http,
    Mqtt,
    CoAP,
    Custom,
}

/// Estado del device
#[derive(Debug, Clone)]
pub enum WifiDeviceState {
    Online,
    Offline,
    Busy,
    Error(String),
}

impl WifiDeviceState {
    pub fn is_online(&self) -> bool {
        matches!(self, WifiDeviceState::Online)
    }
}

/// Representa un dispositivo WiFi en la red
pub struct WifiDevice {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub device_class: WifiDeviceClass,
    protocol: Protocol,
    state: WifiDeviceState,
    attributes: HashMap<String, String>,
    last_response: Option<String>,
    last_seen: u64,
}

impl WifiDevice {
    /// Crea nuevo device
    pub fn new(id: &str, ip: &str, port: u16, device_class: WifiDeviceClass) -> Self {
        Self {
            id: id.to_string(),
            ip: ip.to_string(),
            port,
            device_class,
            protocol: Protocol::Http, // Default
            state: WifiDeviceState::Offline,
            attributes: HashMap::new(),
            last_response: None,
            last_seen: current_timestamp_ms(),
        }
    }

    /// Establece protocolo
    pub fn set_protocol(&mut self, protocol: Protocol) {
        self.protocol = protocol;
    }

    /// Marca como seen
    pub fn mark_online(&mut self) {
        self.state = WifiDeviceState::Online;
        self.last_seen = current_timestamp_ms();
    }

    /// Marca como offline
    pub fn mark_offline(&mut self) {
        self.state = WifiDeviceState::Offline;
    }

    /// Actualiza atributo
    pub fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    /// Obtiene atributo
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Construye request HTTP para encender
    pub fn http_turn_on(&self) -> HttpRequest {
        let path = match self.device_class {
            WifiDeviceClass::Light | WifiDeviceClass::Dimmer | WifiDeviceClass::Color => "/power",
            WifiDeviceClass::Switch => "/relay/0/on",
            WifiDeviceClass::Lock => "/unlock",
            WifiDeviceClass::Thermostat => "/mode/heat",
            WifiDeviceClass::Camera => "/status",
            _ => "/state",
        };

        HttpRequest {
            method: HttpMethod::Post,
            host: self.ip.clone(),
            port: self.port,
            path: path.to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: Some(r#"{"state": "on"}"#.to_string()),
        }
    }

    /// Construye request HTTP para apagar
    pub fn http_turn_off(&self) -> HttpRequest {
        let path = match self.device_class {
            WifiDeviceClass::Light | WifiDeviceClass::Dimmer | WifiDeviceClass::Color => "/power",
            WifiDeviceClass::Switch => "/relay/0/off",
            WifiDeviceClass::Lock => "/lock",
            WifiDeviceClass::Thermostat => "/mode/off",
            _ => "/state",
        };

        HttpRequest {
            method: HttpMethod::Post,
            host: self.ip.clone(),
            port: self.port,
            path: path.to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: Some(r#"{"state": "off"}"#.to_string()),
        }
    }

    /// Construye request HTTP para obtener estado
    pub fn http_get_status(&self) -> HttpRequest {
        HttpRequest {
            method: HttpMethod::Get,
            host: self.ip.clone(),
            port: self.port,
            path: "/status".to_string(),
            headers: vec![],
            body: None,
        }
    }

    /// Parsea respuesta HTTP
    pub fn parse_response(&mut self, response: &str) -> Option<HashMap<String, String>> {
        self.last_response = Some(response.to_string());
        self.last_seen = current_timestamp_ms();

        // Simple JSON parse
        let mut result = HashMap::new();

        // Remove braces and whitespace
        let body = response.trim();
        if !body.contains('{') || !body.contains('}') {
            return None;
        }

        let content = body.split('{').nth(1)?.split('}').next()?;

        for pair in content.split(',') {
            let kv: Vec<&str> = pair.split(':').collect();
            if kv.len() == 2 {
                let key = kv[0].trim().trim_matches('"').to_string();
                let value = kv[1].trim().trim_matches('"').to_string();
                result.insert(key, value);
            }
        }

        Some(result)
    }

    /// Construye MQTT topic para este device
    pub fn mqtt_topic(&self, action: &str) -> String {
        format!("home/{}/{:?}/{}", self.id, self.device_class, action)
    }
}

/// Clase de dispositivo WiFi
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WifiDeviceClass {
    Light,
    Dimmer,
    Color,
    Switch,
    Plug, // Smart plug
    Lock, // Door lock
    Thermostat,
    Sensor, // Temperature, humidity
    Camera, // IP Camera
    Fan,
    Blinds,
    Unknown,
}

impl WifiDeviceClass {
    /// Nombre para display
    pub fn name(&self) -> &'static str {
        match self {
            WifiDeviceClass::Light => "Light",
            WifiDeviceClass::Dimmer => "Dimmer",
            WifiDeviceClass::Color => "Color Light",
            WifiDeviceClass::Switch => "Switch",
            WifiDeviceClass::Plug => "Smart Plug",
            WifiDeviceClass::Lock => "Door Lock",
            WifiDeviceClass::Thermostat => "Thermostat",
            WifiDeviceClass::Sensor => "Sensor",
            WifiDeviceClass::Camera => "Camera",
            WifiDeviceClass::Fan => "Fan",
            WifiDeviceClass::Blinds => "Blinds",
            WifiDeviceClass::Unknown => "Unknown",
        }
    }
}

/// Método HTTP
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// Request HTTP
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl HttpRequest {
    /// Serializa a bytes para enviar por TCP
    pub fn to_bytes(&self) -> Vec<u8> {
        let method_str = match self.method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
        };

        let mut request = format!(
            "{} {} HTTP/1.1\r\nHost: {}:{}\r\n",
            method_str, self.path, self.host, self.port
        );

        for (key, value) in &self.headers {
            request.push_str(&format!("{}: {}\r\n", key, value));
        }

        if let Some(body) = &self.body {
            request.push_str(&format!("Content-Length: {}\r\n", body.len()));
            request.push_str("\r\n");
            request.push_str(body);
        } else {
            request.push_str("\r\n");
        }

        request.into_bytes()
    }
}

/// Response HTTP
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl HttpResponse {
    /// Parsea bytes de respuesta HTTP
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let text = String::from_utf8_lossy(bytes);
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() {
            return None;
        }

        // Parse status line: HTTP/1.1 200 OK
        let status_line = lines[0].split_whitespace().collect::<Vec<_>>();
        if status_line.len() < 2 {
            return None;
        }
        let status_code: u16 = status_line[1].parse().ok()?;
        let status_text = if status_line.len() >= 3 {
            status_line[2..].join(" ")
        } else {
            String::new()
        };

        // Find headers and body
        let mut headers = Vec::new();
        let mut body_start = 0;

        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() {
                body_start = i + 1;
                break;
            }
            if let Some(colon) = line.find(':') {
                let key = line[..colon].trim().to_string();
                let value = line[colon + 1..].trim().to_string();
                headers.push((key, value));
            }
        }

        let body = if body_start > 0 && body_start < lines.len() {
            Some(lines[body_start..].join("\n"))
        } else {
            None
        };

        Some(HttpResponse {
            status_code,
            status_text,
            headers,
            body,
        })
    }

    /// Verifica si fue exitoso (2xx)
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
}

/// mDNS Service info
#[derive(Debug, Clone)]
pub struct MdnsService {
    pub name: String,
    pub service_type: String,
    pub domain: String,
    pub host: String,
    pub port: u16,
    pub txt_records: Vec<(String, String)>,
}

impl MdnsService {
    /// Crea desde record mDNS
    pub fn from_txt(_txt: &[u8]) -> Option<Self> {
        // Simple TXT record parsing
        let name = String::new();
        let service_type = String::new();
        let domain = String::new();
        let host = String::new();
        let port = 0u16;
        let txt_records = Vec::new();

        // This is simplified - real mDNS parsing would handle DNS name encoding
        Some(MdnsService {
            name,
            service_type,
            domain,
            host,
            port,
            txt_records,
        })
    }
}

/// WiFi Device Discovery
pub struct WifiDiscovery {
    devices: HashMap<String, WifiDevice>,
}

impl WifiDiscovery {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Añade device manualmente
    pub fn add_device(&mut self, device: WifiDevice) {
        self.devices.insert(device.id.clone(), device);
    }

    /// Obtiene device por ID
    pub fn get(&self, id: &str) -> Option<&WifiDevice> {
        self.devices.get(id)
    }

    /// Obtiene device mutable
    pub fn get_mut(&mut self, id: &str) -> Option<&mut WifiDevice> {
        self.devices.get_mut(id)
    }

    /// Lista todos los devices
    pub fn list_all(&self) -> Vec<&WifiDevice> {
        self.devices.values().collect()
    }

    /// Lista por clase
    pub fn list_by_class(&self, class: WifiDeviceClass) -> Vec<&WifiDevice> {
        self.devices
            .values()
            .filter(|d| d.device_class == class)
            .collect()
    }

    /// Lista luces
    pub fn list_lights(&self) -> Vec<&WifiDevice> {
        self.list_by_class(WifiDeviceClass::Light)
            .into_iter()
            .chain(self.list_by_class(WifiDeviceClass::Dimmer))
            .chain(self.list_by_class(WifiDeviceClass::Color))
            .collect()
    }

    /// Lista cerraduras
    pub fn list_locks(&self) -> Vec<&WifiDevice> {
        self.list_by_class(WifiDeviceClass::Lock)
    }

    /// Lista termostatos
    pub fn list_thermostats(&self) -> Vec<&WifiDevice> {
        self.list_by_class(WifiDeviceClass::Thermostat)
    }

    /// Remueve device
    pub fn remove(&mut self, id: &str) -> Option<WifiDevice> {
        self.devices.remove(id)
    }

    /// Obtiene count
    pub fn count(&self) -> usize {
        self.devices.len()
    }
}

impl Default for WifiDiscovery {
    fn default() -> Self {
        Self::new()
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
    fn test_device_creation() {
        let device = WifiDevice::new("light-1", "192.168.1.100", 80, WifiDeviceClass::Light);
        assert_eq!(device.id, "light-1");
        assert_eq!(device.ip, "192.168.1.100");
    }

    #[test]
    fn test_http_request() {
        let device = WifiDevice::new("switch-1", "192.168.1.101", 80, WifiDeviceClass::Switch);
        let request = device.http_turn_on();
        let bytes = request.to_bytes();
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"POST"));
    }

    #[test]
    fn test_http_response_parse() {
        let response_text =
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"state\": \"on\"}";
        let response = HttpResponse::parse(response_text.as_bytes()).unwrap();
        assert_eq!(response.status_code, 200);
        assert!(response.is_success());
    }
}
