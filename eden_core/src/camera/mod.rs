//! Camera Module - ONVIF + RTSP surveillance system
//!
//! 100% Rust puro - Sin dependencias externas
//!
//! Capabilities:
//! - ONVIF protocol (SOAP + XML parsing)
//! - RTSP streaming client
//! - Camera discovery
//! - Motion detection
//! - Recording and snapshot capture

#![allow(dead_code)]

pub mod device;
pub mod discovery;
pub mod onvif;
pub mod rtsp;

pub use device::{
    AudioCodec, Camera, CameraCapabilities, CameraConfig, CameraStatus, FrameRate, MotionConfig,
    MotionDetection, MotionRegion, Resolution, VideoCodec,
};
pub use discovery::{CameraDiscovery, DiscoveredCamera, DiscoveryResult};
pub use onvif::{OnvifCapabilities, OnvifDevice, OnvifProfile};
pub use rtsp::{FrameData, RtspClient, RtspSession, RtspState};
