//! RTSP Module - Real Time Streaming Protocol
//!
//! 100% Rust puro - Sin dependencias externas

#![allow(dead_code)]

pub mod client;

pub use client::{
    AudioCodec, AuthInfo, AuthType, FrameData, MediaInfo, RtpH264, RtpH265, RtpJpeg, RtpPacket,
    RtspClient, RtspMethod, RtspSession, RtspState, StreamInfo, VideoCodec,
};
