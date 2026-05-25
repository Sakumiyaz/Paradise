//! ONVIF Protocol Implementation
//!
//! 100% Rust puro - SOAP + XML parsing desde cero
//!
//! Implements:
//! - WS-Discovery (UDP probe)
//! - ONVIF device service
//! - Media service (profiles, streams)
//! - PTZ service (pan/tilt/zoom)
//! - Event service
#![allow(unused_imports)]
#![allow(dead_code)]
use std::sync::RwLock;

pub mod device_service;
pub mod media_service;
pub mod ptz_service;
pub mod soap;
pub mod ws_discovery;
pub mod xml;

pub use device_service::{DeviceInformation, DeviceServiceClient, NetworkInterface};
pub use media_service::{
    MediaProfile, MediaServiceClient, MediaUri, ProfileToken, StreamProtocol, StreamSetup,
    Transport,
};
pub use ptz_service::{
    PtzConfiguration, PtzHome, PtzMove, PtzPosition, PtzServiceClient, PtzStatus,
};
pub use soap::{
    SoapAction, SoapBody, SoapEnvelope, SoapFault, SoapHeader, SoapMessage, SoapVersion,
    XmlAttribute, XmlElement, XmlNamespace,
};
pub use ws_discovery::{
    ByeHandler, HelloHandler, ProbeHandler, ProbeMatch, WsDiscovery, MATCH_BY, SCOPE_MATCH,
};

/// ONVIF device wrapper
pub struct OnvifDevice {
    host: String,
    port: u16,
    user: Option<String>,
    password: Option<String>,
    device_client: DeviceServiceClient,
    media_client: MediaServiceClient,
    ptz_client: PtzServiceClient,
}

impl OnvifDevice {
    /// Create new ONVIF device
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            user: None,
            password: None,
            device_client: DeviceServiceClient::new(host, port),
            media_client: MediaServiceClient::new(host, port),
            ptz_client: PtzServiceClient::new(host, port),
        }
    }

    /// Set authentication
    pub fn with_auth(mut self, user: &str, password: &str) -> Self {
        self.user = Some(user.to_string());
        self.password = Some(password.to_string());
        self.device_client = self.device_client.with_auth(user, password);
        self.media_client = self.media_client.with_auth(user, password);
        self.ptz_client = self.ptz_client.with_auth(user, password);
        self
    }

    /// Get device info
    pub fn get_device_info(&mut self) -> Result<DeviceInformation, String> {
        self.device_client.get_device_info()
    }

    /// Get device capabilities
    pub fn get_capabilities(&mut self) -> OnvifCapabilities {
        OnvifCapabilities {
            device: true,
            media: true,
            ptz: true,
            events: false,
            imaging: false,
        }
    }

    /// Get media profiles
    pub fn get_profiles(&mut self) -> Result<Vec<OnvifProfile>, String> {
        let media_profiles = self.media_client.get_profiles()?;
        Ok(media_profiles
            .into_iter()
            .map(|mp| OnvifProfile {
                token: mp.token,
                name: mp.name,
                video_source_token: mp.video_source.map(|vs| vs.token),
                video_encoder_token: mp.video_encoder.map(|ve| ve.token),
            })
            .collect())
    }

    /// Get stream URI
    pub fn get_stream_uri(
        &mut self,
        token: &ProfileToken,
        setup: &StreamSetup,
    ) -> Result<MediaUri, String> {
        self.media_client.get_stream_uri(token, setup)
    }

    /// Get snapshot URI
    pub fn get_snapshot_uri(&mut self, token: &ProfileToken) -> Result<MediaUri, String> {
        self.media_client.get_snapshot_uri(token)
    }

    /// PTZ continuous move
    pub fn ptz_continuous_move(&mut self, token: &str, velocity: &PtzMove) -> Result<(), String> {
        self.ptz_client.continuous_move(token, velocity)
    }

    /// PTZ stop
    pub fn ptz_stop(&mut self, token: &str) -> Result<(), String> {
        self.ptz_client.stop(token)
    }

    /// PTZ goto preset
    pub fn ptz_goto_preset(&mut self, token: &str, preset: &str) -> Result<(), String> {
        self.ptz_client.goto_preset(token, preset)
    }
}

/// ONVIF profile (simplified)
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OnvifProfile {
    pub token: ProfileToken,
    pub name: String,
    pub video_source_token: Option<String>,
    pub video_encoder_token: Option<String>,
}

/// ONVIF capabilities
#[derive(Debug, Clone)]
pub struct OnvifCapabilities {
    pub device: bool,
    pub media: bool,
    pub ptz: bool,
    pub events: bool,
    pub imaging: bool,
}
