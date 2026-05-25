//! ONVIF PTZ Service Client
//!
//! 100% Rust puro - Sin dependencias externas
//!
//! Implements:
//! - PTZ status retrieval
//! - Pan/tilt/zoom control
//! - Preset positions

#![allow(dead_code)]

use super::soap::{SoapBody, SoapEnvelope, SoapVersion, XmlElement};
use std::io::{Read, Write};
use std::net::TcpStream;

/// PTZ position (pan, tilt, zoom)
#[derive(Debug, Clone, Default)]
pub struct PtzPosition {
    pub pan: f32,
    pub tilt: f32,
    pub zoom: f32,
}

/// PTZ move request
#[derive(Debug, Clone)]
pub struct PtzMove {
    pub pan: f32,
    pub tilt: f32,
    pub zoom: f32,
}

/// PTZ home position
#[derive(Debug, Clone)]
pub struct PtzHome {
    pub token: String,
    pub name: String,
}

/// PTZ node (physical PTZ unit)
#[derive(Debug, Clone)]
pub struct PtzNode {
    pub token: String,
    pub name: String,
    pub supported_ptz_spaces: Vec<String>,
    pub home_position: PtzPosition,
}

/// PTZ configuration
#[derive(Debug, Clone)]
pub struct PtzConfiguration {
    pub token: String,
    pub name: String,
    pub node_token: String,
    pub default_ptz_timeout: u32,
    pub pan_tilt_limits: Option<(f32, f32)>,
    pub zoom_limits: Option<(f32, f32)>,
}

/// PTZ status
#[derive(Debug, Clone)]
pub struct PtzStatus {
    pub position: PtzPosition,
    pub move_status: String,
    pub error: Option<String>,
}

/// ONVIF PTZ Service client
pub struct PtzServiceClient {
    host: String,
    port: u16,
    user: Option<String>,
    password: Option<String>,
}

impl PtzServiceClient {
    /// Create new PTZ service client
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
        format!("http://{}:{}/onvif/ptz_service", self.host, self.port)
    }

    /// Send SOAP request
    fn send_request(&self, envelope: &SoapEnvelope) -> Result<String, String> {
        let mut stream = TcpStream::connect((&*self.host, self.port))
            .map_err(|e| format!("Connection failed: {}", e))?;

        let xml = envelope.to_xml();
        let request = format!(
            "POST /onvif/ptz_service HTTP/1.1\r\n\
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

    /// Get PTZ status
    pub fn get_status(&self, profile_token: &str) -> Result<PtzStatus, String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetStatus".to_string(),
                namespace: Some("ptz".to_string()),
                attributes: vec![],
                children: vec![XmlElement {
                    tag: "ProfileToken".to_string(),
                    namespace: None,
                    attributes: vec![],
                    children: vec![],
                    text: Some(profile_token.to_string()),
                }],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        let response = self.send_request(&envelope)?;

        let pan = self.extract_f32(&response, "Pan").unwrap_or(0.0);
        let tilt = self.extract_f32(&response, "Tilt").unwrap_or(0.0);
        let zoom = self.extract_f32(&response, "Zoom").unwrap_or(0.0);

        Ok(PtzStatus {
            position: PtzPosition { pan, tilt, zoom },
            move_status: self
                .extract(&response, "MoveStatus")
                .unwrap_or_else(|| "IDLE".to_string()),
            error: self.extract(&response, "Error"),
        })
    }

    /// Get PTZ configurations
    pub fn get_configurations(&self) -> Result<Vec<PtzConfiguration>, String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetConfigurations".to_string(),
                namespace: Some("ptz".to_string()),
                attributes: vec![],
                children: vec![],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        let response = self.send_request(&envelope)?;

        let mut configs = vec![];
        let mut pos = 0;

        while let Some(token) = self.extract_element(&response, "PTZConfiguration", &mut pos) {
            configs.push(PtzConfiguration {
                token: self
                    .extract_attr(token, "token")
                    .unwrap_or_else(|| "0".to_string()),
                name: self
                    .extract(token, "Name")
                    .unwrap_or_else(|| "PTZ".to_string()),
                node_token: self
                    .extract(token, "NodeToken")
                    .unwrap_or_else(|| "0".to_string()),
                default_ptz_timeout: self
                    .extract(token, "DefaultPTZTimeout")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5000),
                pan_tilt_limits: None,
                zoom_limits: None,
            });
        }

        if configs.is_empty() {
            configs.push(PtzConfiguration {
                token: "0".to_string(),
                name: "PTZ_Config".to_string(),
                node_token: "0".to_string(),
                default_ptz_timeout: 5000,
                pan_tilt_limits: Some((-1.0, 1.0)),
                zoom_limits: Some((0.0, 1.0)),
            });
        }

        Ok(configs)
    }

    /// Continuous pan/tilt/zoom move
    pub fn continuous_move(&self, profile_token: &str, velocity: &PtzMove) -> Result<(), String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "ContinuousMove".to_string(),
                namespace: Some("ptz".to_string()),
                attributes: vec![],
                children: vec![
                    XmlElement {
                        tag: "ProfileToken".to_string(),
                        namespace: None,
                        attributes: vec![],
                        children: vec![],
                        text: Some(profile_token.to_string()),
                    },
                    XmlElement {
                        tag: "Velocity".to_string(),
                        namespace: None,
                        attributes: vec![],
                        children: vec![
                            XmlElement {
                                tag: "PanTilt".to_string(),
                                namespace: None,
                                attributes: vec![],
                                children: vec![],
                                text: Some(format!("{} {}", velocity.pan, velocity.tilt)),
                            },
                            XmlElement {
                                tag: "Zoom".to_string(),
                                namespace: None,
                                attributes: vec![],
                                children: vec![],
                                text: Some(velocity.zoom.to_string()),
                            },
                        ],
                        text: None,
                    },
                ],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        self.send_request(&envelope)?;

        Ok(())
    }

    /// Stop PTZ movement
    pub fn stop(&self, profile_token: &str) -> Result<(), String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "Stop".to_string(),
                namespace: Some("ptz".to_string()),
                attributes: vec![],
                children: vec![
                    XmlElement {
                        tag: "ProfileToken".to_string(),
                        namespace: None,
                        attributes: vec![],
                        children: vec![],
                        text: Some(profile_token.to_string()),
                    },
                    XmlElement {
                        tag: "PanTilt".to_string(),
                        namespace: None,
                        attributes: vec![],
                        children: vec![],
                        text: Some("true".to_string()),
                    },
                    XmlElement {
                        tag: "Zoom".to_string(),
                        namespace: None,
                        attributes: vec![],
                        children: vec![],
                        text: Some("true".to_string()),
                    },
                ],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        self.send_request(&envelope)?;

        Ok(())
    }

    /// Go to preset position
    pub fn goto_preset(&self, profile_token: &str, preset_token: &str) -> Result<(), String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GotoPreset".to_string(),
                namespace: Some("ptz".to_string()),
                attributes: vec![],
                children: vec![
                    XmlElement {
                        tag: "ProfileToken".to_string(),
                        namespace: None,
                        attributes: vec![],
                        children: vec![],
                        text: Some(profile_token.to_string()),
                    },
                    XmlElement {
                        tag: "PresetToken".to_string(),
                        namespace: None,
                        attributes: vec![],
                        children: vec![],
                        text: Some(preset_token.to_string()),
                    },
                ],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        self.send_request(&envelope)?;

        Ok(())
    }

    /// Extract float value by tag
    fn extract_f32(&self, xml: &str, tag: &str) -> Option<f32> {
        self.extract(xml, tag)?.parse().ok()
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

    /// Extract element with position tracking
    fn extract_element<'a>(&self, xml: &'a str, tag: &str, pos: &mut usize) -> Option<&'a str> {
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
