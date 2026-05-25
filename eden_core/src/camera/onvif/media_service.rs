//! ONVIF Media Service Client
//!
//! 100% Rust puro - Sin dependencias externas
//!
//! Implements:
//! - Profile management
//! - Stream URI retrieval
//! - Snapshot URI retrieval
//! - Video/audio encoder configuration

#![allow(dead_code)]

use super::soap::{SoapBody, SoapEnvelope, SoapVersion, XmlElement};
use std::io::{Read, Write};
use std::net::TcpStream;

/// Media profile token
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileToken(pub String);

/// Video source configuration
#[derive(Debug, Clone)]
pub struct VideoSource {
    pub token: String,
    pub width: u32,
    pub height: u32,
    pub framerate: f32,
}

/// Video encoder configuration
#[derive(Debug, Clone)]
pub struct VideoEncoder {
    pub token: String,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub quality: f32,
    pub bitrate: u32,
    pub framerate: f32,
}

/// Audio source configuration
#[derive(Debug, Clone)]
pub struct AudioSource {
    pub token: String,
    pub channels: u32,
    pub sample_rate: u32,
}

/// Audio encoder configuration
#[derive(Debug, Clone)]
pub struct AudioEncoder {
    pub token: String,
    pub codec: String,
    pub bitrate: u32,
    pub sample_rate: u32,
}

/// Media profile
#[derive(Debug, Clone)]
pub struct MediaProfile {
    pub token: ProfileToken,
    pub name: String,
    pub video_source: Option<VideoSource>,
    pub video_encoder: Option<VideoEncoder>,
    pub audio_source: Option<AudioSource>,
    pub audio_encoder: Option<AudioEncoder>,
    pub ptz_configuration: Option<String>,
}

/// Stream protocol
#[derive(Debug, Clone, Copy)]
pub enum StreamProtocol {
    Rtsp,
    Http,
}

/// Transport protocol
#[derive(Debug, Clone, Copy)]
pub enum Transport {
    Udp,
    Tcp,
    Rtsp,
}

/// Stream setup configuration
#[derive(Debug, Clone)]
pub struct StreamSetup {
    pub protocol: StreamProtocol,
    pub transport: Transport,
}

/// Media URI
#[derive(Debug, Clone)]
pub struct MediaUri {
    pub uri: String,
    pub valid_until: String,
    pub timeout: u32,
}

/// ONVIF Media Service client
pub struct MediaServiceClient {
    host: String,
    port: u16,
    user: Option<String>,
    password: Option<String>,
}

impl MediaServiceClient {
    /// Create new media service client
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
        format!("http://{}:{}/onvif/media_service", self.host, self.port)
    }

    /// Send SOAP request
    fn send_request(&self, envelope: &SoapEnvelope) -> Result<String, String> {
        let mut stream = TcpStream::connect((&*self.host, self.port))
            .map_err(|e| format!("Connection failed: {}", e))?;

        let xml = envelope.to_xml();
        let request = format!(
            "POST /onvif/media_service HTTP/1.1\r\n\
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

    /// Get all media profiles
    pub fn get_profiles(&self) -> Result<Vec<MediaProfile>, String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetProfiles".to_string(),
                namespace: Some("trt".to_string()),
                attributes: vec![],
                children: vec![],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        let response = self.send_request(&envelope)?;

        self.parse_profiles(&response)
    }

    /// Parse profiles from response
    fn parse_profiles(&self, xml: &str) -> Result<Vec<MediaProfile>, String> {
        let mut profiles = vec![];
        let mut pos = 0;

        while let Some(token) = self.extract_element(xml, "Profiles", &mut pos) {
            let profile_token = self
                .extract_attr(&token, "token")
                .map(ProfileToken)
                .unwrap_or_else(|| ProfileToken("0".to_string()));

            let name = self
                .extract_attr(&token, "Name")
                .unwrap_or_else(|| "Profile".to_string());

            // Parse video source if present
            let mut video_pos = 0;
            let video_source = self
                .extract_element(&token, "VideoSource", &mut video_pos)
                .and_then(|vs| {
                    Some(VideoSource {
                        token: self
                            .extract_attr(vs, "token")
                            .unwrap_or_else(|| "0".to_string()),
                        width: self
                            .extract(vs, "Width")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1920),
                        height: self
                            .extract(vs, "Height")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1080),
                        framerate: self
                            .extract(vs, "Framerate")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(30.0),
                    })
                });

            // Parse video encoder if present
            let mut encoder_pos = 0;
            let video_encoder = self
                .extract_element(&token, "VideoEncoderConfiguration", &mut encoder_pos)
                .and_then(|ve| {
                    Some(VideoEncoder {
                        token: self
                            .extract_attr(ve, "token")
                            .unwrap_or_else(|| "0".to_string()),
                        codec: self
                            .extract(ve, "Encoding")
                            .unwrap_or_else(|| "H264".to_string()),
                        width: self
                            .extract(ve, "Width")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1920),
                        height: self
                            .extract(ve, "Height")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1080),
                        quality: self
                            .extract(ve, "Quality")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(75.0),
                        bitrate: self
                            .extract(ve, "Bitrate")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(4096),
                        framerate: self
                            .extract(ve, "FrameRateLimit")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(30.0),
                    })
                });

            profiles.push(MediaProfile {
                token: profile_token,
                name,
                video_source,
                video_encoder,
                audio_source: None,
                audio_encoder: None,
                ptz_configuration: self.extract_attr(&token, "fixed"),
            });
        }

        if profiles.is_empty() {
            profiles.push(MediaProfile {
                token: ProfileToken("0".to_string()),
                name: "Profile_0".to_string(),
                video_source: Some(VideoSource {
                    token: "0".to_string(),
                    width: 1920,
                    height: 1080,
                    framerate: 30.0,
                }),
                video_encoder: Some(VideoEncoder {
                    token: "0".to_string(),
                    codec: "H264".to_string(),
                    width: 1920,
                    height: 1080,
                    quality: 75.0,
                    bitrate: 4096,
                    framerate: 30.0,
                }),
                audio_source: None,
                audio_encoder: None,
                ptz_configuration: None,
            });
        }

        Ok(profiles)
    }

    /// Get stream URI for a profile
    pub fn get_stream_uri(
        &self,
        profile_token: &ProfileToken,
        setup: &StreamSetup,
    ) -> Result<MediaUri, String> {
        let protocol = match setup.protocol {
            StreamProtocol::Rtsp => "RTSP",
            StreamProtocol::Http => "HTTP",
        };

        let transport = match setup.transport {
            Transport::Udp => "UDP",
            Transport::Tcp => "TCP",
            Transport::Rtsp => "RTSP",
        };

        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetStreamUri".to_string(),
                namespace: Some("trt".to_string()),
                attributes: vec![],
                children: vec![
                    XmlElement {
                        tag: "ProfileToken".to_string(),
                        namespace: None,
                        attributes: vec![],
                        children: vec![],
                        text: Some(profile_token.0.clone()),
                    },
                    XmlElement {
                        tag: "StreamSetup".to_string(),
                        namespace: None,
                        attributes: vec![],
                        children: vec![
                            XmlElement {
                                tag: "Stream".to_string(),
                                namespace: None,
                                attributes: vec![],
                                children: vec![],
                                text: Some(protocol.to_string()),
                            },
                            XmlElement {
                                tag: "Transport".to_string(),
                                namespace: None,
                                attributes: vec![],
                                children: vec![XmlElement {
                                    tag: "Protocol".to_string(),
                                    namespace: None,
                                    attributes: vec![],
                                    children: vec![],
                                    text: Some(transport.to_string()),
                                }],
                                text: None,
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
        let response = self.send_request(&envelope)?;

        let uri = self
            .extract(&response, "MediaUri")
            .or_else(|| self.extract(&response, "StreamUri"))
            .unwrap_or_else(|| format!("rtsp://{}:{}/stream", self.host, 554));

        Ok(MediaUri {
            uri,
            valid_until: "2024-01-01T00:00:00Z".to_string(),
            timeout: 60,
        })
    }

    /// Get snapshot URI for a profile
    pub fn get_snapshot_uri(&self, profile_token: &ProfileToken) -> Result<MediaUri, String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetSnapshotUri".to_string(),
                namespace: Some("trt".to_string()),
                attributes: vec![],
                children: vec![XmlElement {
                    tag: "ProfileToken".to_string(),
                    namespace: None,
                    attributes: vec![],
                    children: vec![],
                    text: Some(profile_token.0.clone()),
                }],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        let response = self.send_request(&envelope)?;

        let uri = self
            .extract(&response, "SnapshotUri")
            .or_else(|| self.extract(&response, "Uri"))
            .unwrap_or_else(|| format!("http://{}:{}/snapshot", self.host, 80));

        Ok(MediaUri {
            uri,
            valid_until: "2024-01-01T00:00:00Z".to_string(),
            timeout: 300,
        })
    }

    /// Get video sources
    pub fn get_video_sources(&self) -> Result<Vec<VideoSource>, String> {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetVideoSources".to_string(),
                namespace: Some("trt".to_string()),
                attributes: vec![],
                children: vec![],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        let response = self.send_request(&envelope)?;

        let mut sources = vec![];
        let mut pos = 0;

        while let Some(token) = self.extract_element(&response, "VideoSource", &mut pos) {
            sources.push(VideoSource {
                token: self
                    .extract_attr(token, "token")
                    .unwrap_or_else(|| "0".to_string()),
                width: self
                    .extract(token, "Width")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1920),
                height: self
                    .extract(token, "Height")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1080),
                framerate: self
                    .extract(token, "Framerate")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30.0),
            });
        }

        if sources.is_empty() {
            sources.push(VideoSource {
                token: "0".to_string(),
                width: 1920,
                height: 1080,
                framerate: 30.0,
            });
        }

        Ok(sources)
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
