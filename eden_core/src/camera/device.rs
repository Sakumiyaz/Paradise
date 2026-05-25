//! Camera Device Module
//!
//! 100% Rust puro - Sin dependencias externas
//!
//! High-level camera control with ONVIF + RTSP

#![allow(dead_code)]

use std::time::{Duration, Instant};

pub use crate::camera::onvif::{
    MediaUri, OnvifCapabilities, OnvifDevice, OnvifProfile, PtzMove, PtzPosition, StreamProtocol,
    StreamSetup, Transport,
};
pub use crate::camera::rtsp::{
    AudioCodec, FrameData, RtspClient, RtspSession, RtspState, StreamInfo, VideoCodec,
};

/// Video resolution
#[derive(Debug, Clone, Copy)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

/// Frame rate configuration
#[derive(Debug, Clone, Copy)]
pub struct FrameRate {
    pub numerator: u32,
    pub denominator: u32,
}

impl FrameRate {
    /// Create frame rate from fps value
    pub fn from_fps(fps: f32) -> Self {
        if fps >= 1.0 {
            Self {
                numerator: fps as u32,
                denominator: 1,
            }
        } else {
            Self {
                numerator: 1,
                denominator: (1.0 / fps) as u32,
            }
        }
    }

    /// Get fps value
    pub fn fps(&self) -> f32 {
        if self.denominator > 0 {
            self.numerator as f32 / self.denominator as f32
        } else {
            0.0
        }
    }
}

/// Camera configuration
#[derive(Debug, Clone)]
pub struct CameraConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub stream_uri: Option<String>,
    pub snapshot_uri: Option<String>,
    pub rtsp_port: u16,
    pub onvif_port: u16,
    pub auto_connect: bool,
    pub reconnect_interval_ms: u64,
    pub stream_timeout_ms: u64,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            name: "Camera".to_string(),
            host: "192.168.1.100".to_string(),
            port: 80,
            username: None,
            password: None,
            stream_uri: None,
            snapshot_uri: None,
            rtsp_port: 554,
            onvif_port: 80,
            auto_connect: false,
            reconnect_interval_ms: 5000,
            stream_timeout_ms: 10000,
        }
    }
}

/// Camera capabilities
#[derive(Debug, Clone)]
pub struct CameraCapabilities {
    pub supports_onvif: bool,
    pub supports_rtsp: bool,
    pub supports_ptz: bool,
    pub supports_audio: bool,
    pub supports_recording: bool,
    pub supports_motion_detection: bool,
    pub video_codecs: Vec<VideoCodec>,
    pub audio_codecs: Vec<AudioCodec>,
    pub max_resolution: Resolution,
    pub has_static_ip: bool,
}

/// Camera status
#[derive(Debug, Clone, PartialEq)]
pub enum CameraStatus {
    Disconnected,
    Connecting,
    Connected,
    Streaming,
    Error(String),
    Reconnecting,
}

/// Motion detection configuration
#[derive(Debug, Clone)]
pub struct MotionConfig {
    pub enabled: bool,
    pub threshold: f32,
    pub min_area: u32,
    pub regions: Vec<MotionRegion>,
    pub send_notifications: bool,
}

/// Motion detection region
#[derive(Debug, Clone)]
pub struct MotionRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub sensitivity: f32,
}

/// Motion detection result
#[derive(Debug, Clone)]
pub struct MotionDetection {
    pub timestamp: Instant,
    pub regions_triggered: Vec<u32>,
    pub intensity: f32,
    pub is_motion: bool,
}

/// Recording configuration
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    pub enabled: bool,
    pub continuous: bool,
    pub on_motion_only: bool,
    pub max_clips: u32,
    pub clip_duration_sec: u32,
    pub pre_buffer_sec: u32,
}

/// Recording clip information
#[derive(Debug, Clone)]
pub struct ClipInfo {
    pub id: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub duration: Duration,
    pub size_bytes: u64,
    pub reason: String,
}

/// Snapshot configuration
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    pub format: String,
    pub quality: u8,
}

/// Snapshot data
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub timestamp: Instant,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

/// Camera device
pub struct Camera {
    config: CameraConfig,
    status: CameraStatus,
    onvif: Option<OnvifDevice>,
    rtsp: Option<RtspClient>,
    rtsp_session: Option<RtspSession>,
    profiles: Vec<OnvifProfile>,
    capabilities: Option<CameraCapabilities>,
    motion_config: MotionConfig,
    recording_config: RecordingConfig,
    clips: Vec<ClipInfo>,
    last_frame: Option<FrameData>,
    last_motion: Option<MotionDetection>,
    connected_at: Option<Instant>,
}

impl Camera {
    /// Create new camera instance
    pub fn new(config: CameraConfig) -> Self {
        Self {
            config,
            status: CameraStatus::Disconnected,
            onvif: None,
            rtsp: None,
            rtsp_session: None,
            profiles: vec![],
            capabilities: None,
            motion_config: MotionConfig {
                enabled: false,
                threshold: 0.5,
                min_area: 100,
                regions: vec![],
                send_notifications: false,
            },
            recording_config: RecordingConfig {
                enabled: false,
                continuous: false,
                on_motion_only: false,
                max_clips: 100,
                clip_duration_sec: 60,
                pre_buffer_sec: 5,
            },
            clips: vec![],
            last_frame: None,
            last_motion: None,
            connected_at: None,
        }
    }

    /// Connect to camera via ONVIF
    pub fn connect(&mut self) -> Result<(), String> {
        self.status = CameraStatus::Connecting;

        // Create ONVIF device client
        let mut onvif = OnvifDevice::new(&self.config.host, self.config.onvif_port);

        if let (Some(u), Some(p)) = (&self.config.username, &self.config.password) {
            onvif = onvif.with_auth(u, p);
        }

        self.onvif = Some(onvif);

        // Get capabilities
        if let Some(ref mut o) = self.onvif {
            // Get device info
            let _info = o.get_device_info();

            // Get profiles
            if let Ok(profiles) = o.get_profiles() {
                self.profiles = profiles;
            }

            // Get capabilities
            let caps = o.get_capabilities();
            self.capabilities = Some(CameraCapabilities {
                supports_onvif: true,
                supports_rtsp: true,
                supports_ptz: caps.ptz,
                supports_audio: true,
                supports_recording: true,
                supports_motion_detection: true,
                video_codecs: vec![VideoCodec::H264],
                audio_codecs: vec![],
                max_resolution: Resolution {
                    width: 1920,
                    height: 1080,
                },
                has_static_ip: false,
            });
        }

        self.status = CameraStatus::Connected;
        self.connected_at = Some(Instant::now());

        Ok(())
    }

    /// Connect to RTSP stream
    pub fn start_stream(&mut self) -> Result<(), String> {
        if self.status != CameraStatus::Connected && self.status != CameraStatus::Streaming {
            return Err("Camera not connected".to_string());
        }

        // Get stream URI from ONVIF
        let stream_uri = if let Some(ref mut onvif) = self.onvif {
            if let Some(ref profiles) = self.profiles.first() {
                let setup = StreamSetup {
                    protocol: StreamProtocol::Rtsp,
                    transport: Transport::Tcp,
                };
                onvif
                    .get_stream_uri(&profiles.token, &setup)
                    .map(|uri| uri.uri)
                    .ok()
            } else {
                None
            }
        } else {
            None
        };

        // Create RTSP client
        let mut rtsp = RtspClient::new(&self.config.host, self.config.rtsp_port);

        if let (Some(u), Some(p)) = (&self.config.username, &self.config.password) {
            rtsp = rtsp.with_auth(u, p);
        }

        // Connect via RTSP
        let url = stream_uri.unwrap_or_else(|| {
            format!(
                "rtsp://{}:{}/stream",
                self.config.host, self.config.rtsp_port
            )
        });

        rtsp.connect(&url)?;

        self.rtsp = Some(rtsp);
        self.status = CameraStatus::Streaming;

        Ok(())
    }

    /// Stop RTSP stream
    pub fn stop_stream(&mut self) -> Result<(), String> {
        if let Some(ref mut rtsp) = self.rtsp {
            rtsp.teardown()?;
        }

        self.rtsp = None;
        self.status = CameraStatus::Connected;

        Ok(())
    }

    /// Get next frame from RTSP stream
    pub fn get_frame(&mut self) -> Option<FrameData> {
        // In a real implementation, this would receive RTP packets
        // and reassemble frames. For now, return None as placeholder.
        self.last_frame.clone()
    }

    /// Get camera configuration
    pub fn get_config(&self) -> &CameraConfig {
        &self.config
    }

    /// Get camera status
    pub fn get_status(&self) -> &CameraStatus {
        &self.status
    }

    /// Get camera capabilities
    pub fn get_capabilities(&self) -> Option<&CameraCapabilities> {
        self.capabilities.as_ref()
    }

    /// Get available profiles
    pub fn get_profiles(&self) -> &[OnvifProfile] {
        &self.profiles
    }

    /// PTZ: Move camera
    pub fn ptz_move(&mut self, pan: f32, tilt: f32, zoom: f32) -> Result<(), String> {
        if let Some(ref mut onvif) = self.onvif {
            if let Some(ref profile) = self.profiles.first() {
                let velocity = PtzMove { pan, tilt, zoom };
                return onvif.ptz_continuous_move(&profile.token.0, &velocity);
            }
        }
        Err("PTZ not available".to_string())
    }

    /// PTZ: Stop movement
    pub fn ptz_stop(&mut self) -> Result<(), String> {
        if let Some(ref mut onvif) = self.onvif {
            if let Some(ref profile) = self.profiles.first() {
                return onvif.ptz_stop(&profile.token.0);
            }
        }
        Ok(())
    }

    /// PTZ: Go to preset
    pub fn ptz_goto_preset(&mut self, preset: &str) -> Result<(), String> {
        if let Some(ref mut onvif) = self.onvif {
            if let Some(ref profile) = self.profiles.first() {
                return onvif.ptz_goto_preset(&profile.token.0, preset);
            }
        }
        Err("PTZ not available".to_string())
    }

    /// Configure motion detection
    pub fn set_motion_config(&mut self, config: MotionConfig) {
        self.motion_config = config;
    }

    /// Get motion configuration
    pub fn get_motion_config(&self) -> &MotionConfig {
        &self.motion_config
    }

    /// Enable/disable motion detection
    pub fn enable_motion_detection(&mut self, enabled: bool) {
        self.motion_config.enabled = enabled;
    }

    /// Set recording configuration
    pub fn set_recording_config(&mut self, config: RecordingConfig) {
        self.recording_config = config;
    }

    /// Get recording configuration
    pub fn get_recording_config(&self) -> &RecordingConfig {
        &self.recording_config
    }

    /// Start recording
    pub fn start_recording(&mut self) -> Result<(), String> {
        self.recording_config.enabled = true;
        Ok(())
    }

    /// Stop recording
    pub fn stop_recording(&mut self) -> Result<(), String> {
        self.recording_config.enabled = false;
        Ok(())
    }

    /// Get recording clips
    pub fn get_clips(&self) -> &[ClipInfo] {
        &self.clips
    }

    /// Take snapshot
    pub fn snapshot(&mut self) -> Result<Snapshot, String> {
        // Use ONVIF snapshot URI
        if let Some(ref mut onvif) = self.onvif {
            if let Some(ref profile) = self.profiles.first() {
                if let Ok(_uri) = onvif.get_snapshot_uri(&profile.token) {
                    // In real implementation, fetch the image
                    return Ok(Snapshot {
                        timestamp: Instant::now(),
                        data: vec![],
                        width: 1920,
                        height: 1080,
                        format: "jpeg".to_string(),
                    });
                }
            }
        }

        Err("Snapshot not available".to_string())
    }

    /// Disconnect from camera
    pub fn disconnect(&mut self) {
        if let Some(ref mut rtsp) = self.rtsp {
            let _ = rtsp.teardown();
        }

        self.rtsp = None;
        self.onvif = None;
        self.rtsp_session = None;
        self.status = CameraStatus::Disconnected;
    }
}
