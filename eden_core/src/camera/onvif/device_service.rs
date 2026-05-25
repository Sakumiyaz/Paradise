//! ONVIF Device Service Client
//!
//! 100% Rust puro - Sin dependencias externas
//!
//! Implements device information retrieval

#![allow(dead_code)]

use super::soap::{SoapBody, SoapEnvelope, SoapVersion, XmlElement};
use std::io::{Read, Write};
use std::net::TcpStream;

/// Device information
#[derive(Debug, Clone)]
pub struct DeviceInformation {
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: String,
    pub serial_number: String,
    pub hardware_id: String,
}

/// Network interface info
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub subnet_mask: Option<String>,
    pub gateway: Option<String>,
}

/// System date/time
#[derive(Debug, Clone)]
pub struct SystemDateTime {
    pub utc_date_time: String,
    pub local_date_time: String,
    pub timezone: String,
    pub daylight_saving: bool,
}

/// Device capabilities
#[derive(Debug, Clone)]
pub struct GetCapabilities {
    pub device: bool,
    pub media: bool,
    pub ptz: bool,
    pub events: bool,
    pub imaging: bool,
}

/// ONVIF Device Service client
pub struct DeviceServiceClient {
    host: String,
    port: u16,
    user: Option<String>,
    password: Option<String>,
}

impl DeviceServiceClient {
    /// Create new device service client
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            user: None,
            password: None,
        }
    }

    /// Set authentication credentials
    pub fn with_auth(mut self, user: &str, password: &str) -> Self {
        self.user = Some(user.to_string());
        self.password = Some(password.to_string());
        self
    }

    /// Build service URL
    fn service_url(&self) -> String {
        format!("http://{}:{}/onvif/device_service", self.host, self.port)
    }

    /// Send SOAP request
    fn send_request(&self, envelope: &SoapEnvelope) -> Result<String, String> {
        let mut stream = TcpStream::connect((&*self.host, self.port))
            .map_err(|e| format!("Connection failed: {}", e))?;

        let xml = envelope.to_xml();
        let request = format!(
            "POST /onvif/device_service HTTP/1.1\r\n\
             Host: {}:{}\r\n\
             Content-Type: text/xml; charset=utf-8\r\n\
             Content-Length: {}\r\n\
             \r\n\
             {}",
            self.host,
            self.port,
            xml.len(),
            xml
        );

        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("Send failed: {}", e))?;

        let mut response = vec![];
        stream
            .read_to_end(&mut response)
            .map_err(|e| format!("Receive failed: {}", e))?;

        // Parse HTTP response
        let response_str = String::from_utf8_lossy(&response);

        // Find body start (after headers)
        if let Some(body_start) = response_str.find("\r\n\r\n") {
            Ok(response_str[body_start + 4..].to_string())
        } else {
            Ok(response_str.to_string())
        }
    }

    /// Get device information
    pub fn get_device_info(&self) -> Result<DeviceInformation, String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetDeviceInformation".to_string(),
                namespace: Some("tds".to_string()),
                attributes: vec![],
                children: vec![],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        let response = self.send_request(&envelope)?;

        self.parse_device_info(&response)
    }

    /// Parse device information response
    fn parse_device_info(&self, xml: &str) -> Result<DeviceInformation, String> {
        let manufacturer = self
            .extract(&xml, "Manufacturer")
            .unwrap_or_else(|| "Unknown".to_string());
        let model = self
            .extract(&xml, "Model")
            .unwrap_or_else(|| "Unknown".to_string());
        let firmware_version = self
            .extract(&xml, "FirmwareVersion")
            .unwrap_or_else(|| "Unknown".to_string());
        let serial_number = self
            .extract(&xml, "SerialNumber")
            .unwrap_or_else(|| "Unknown".to_string());
        let hardware_id = self
            .extract(&xml, "HardwareId")
            .unwrap_or_else(|| "Unknown".to_string());

        Ok(DeviceInformation {
            manufacturer,
            model,
            firmware_version,
            serial_number,
            hardware_id,
        })
    }

    /// Get device capabilities
    pub fn get_capabilities(&self) -> Result<GetCapabilities, String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetCapabilities".to_string(),
                namespace: Some("tds".to_string()),
                attributes: vec![],
                children: vec![],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        let response = self.send_request(&envelope)?;

        self.parse_capabilities(&response)
    }

    /// Parse capabilities response
    fn parse_capabilities(&self, xml: &str) -> Result<GetCapabilities, String> {
        Ok(GetCapabilities {
            device: xml.contains("Device"),
            media: xml.contains("Media"),
            ptz: xml.contains("PTZ"),
            events: xml.contains("Events"),
            imaging: xml.contains("Imaging"),
        })
    }

    /// Get network interfaces
    pub fn get_network_interfaces(&self) -> Result<Vec<NetworkInterface>, String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetNetworkInterfaces".to_string(),
                namespace: Some("tds".to_string()),
                attributes: vec![],
                children: vec![],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        let response = self.send_request(&envelope)?;

        self.parse_network_interfaces(&response)
    }

    /// Parse network interfaces response
    fn parse_network_interfaces(&self, xml: &str) -> Result<Vec<NetworkInterface>, String> {
        let mut interfaces = vec![];
        let mut pos = 0;

        while let Some(token) = self.extract_token(xml, "NetworkInterface", &mut pos) {
            let name = self
                .extract_attr(&token, "token")
                .unwrap_or_else(|| "eth0".to_string());

            interfaces.push(NetworkInterface {
                name,
                mac_address: self
                    .extract(&token, "MACAddress")
                    .unwrap_or_else(|| "00:00:00:00:00:00".to_string()),
                ip_address: self.extract(&token, "IPv4Address"),
                subnet_mask: None,
                gateway: None,
            });
        }

        if interfaces.is_empty() {
            interfaces.push(NetworkInterface {
                name: "eth0".to_string(),
                mac_address: "00:00:00:00:00:00".to_string(),
                ip_address: None,
                subnet_mask: None,
                gateway: None,
            });
        }

        Ok(interfaces)
    }

    /// Get system date and time
    pub fn get_system_date_time(&self) -> Result<SystemDateTime, String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetSystemDateAndTime".to_string(),
                namespace: Some("tds".to_string()),
                attributes: vec![],
                children: vec![],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        let response = self.send_request(&envelope)?;

        self.parse_system_date_time(&response)
    }

    /// Parse system date/time response
    fn parse_system_date_time(&self, xml: &str) -> Result<SystemDateTime, String> {
        Ok(SystemDateTime {
            utc_date_time: self
                .extract(xml, "UTCDateTime")
                .unwrap_or_else(|| "2024-01-01T00:00:00Z".to_string()),
            local_date_time: self
                .extract(xml, "LocalDateTime")
                .unwrap_or_else(|| "2024-01-01T00:00:00".to_string()),
            timezone: self
                .extract(xml, "TimeZone")
                .unwrap_or_else(|| "UTC".to_string()),
            daylight_saving: xml.contains(" DST ") || xml.contains("DST"),
        })
    }

    /// Extract value by tag
    fn extract(&self, xml: &str, tag: &str) -> Option<String> {
        let start = format!("<{}", tag);
        let end = format!("</{}>", tag);

        if let Some(s) = xml.find(&start) {
            if let Some(e) = xml[s..].find('>') {
                let content = &xml[s + e + 1..];
                if let Some(close) = content.find(&end) {
                    return Some(content[..close].trim().to_string());
                }
            }
        }
        None
    }

    /// Extract attribute value
    fn extract_attr(&self, xml: &str, attr: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr);
        if let Some(pos) = xml.find(&pattern) {
            let start = pos + pattern.len();
            if let Some(end) = xml[start..].find('"') {
                return Some(xml[start..start + end].to_string());
            }
        }
        None
    }

    /// Extract token with position tracking
    fn extract_token<'a>(&self, xml: &'a str, tag: &str, pos: &mut usize) -> Option<&'a str> {
        let start = format!("<{}", tag);
        let end = format!("</{}>", tag);

        if let Some(s) = xml[*pos..].find(&start) {
            let abs_s = *pos + s;
            if let Some(e) = xml[abs_s..].find('>') {
                let content_start = abs_s + e + 1;
                if let Some(close) = xml[content_start..].find(&end) {
                    *pos = content_start + close + end.len();
                    return Some(&xml[abs_s..content_start + close + end.len()]);
                }
            }
        }
        None
    }
}
