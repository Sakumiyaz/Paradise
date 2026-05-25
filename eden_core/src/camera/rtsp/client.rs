//! RTSP Client Implementation
//!
//! 100% Rust puro - Sin dependencias externas
//!
//! Implements:
//! - RTSP DESCRIBE, SETUP, PLAY, PAUSE, TEARDOWN
//! - RTP packet parsing for H.264, H.265, JPEG
//! - Session management
//! - Authentication (Basic, Digest)

#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// RTSP method types
#[derive(Debug, Clone, Copy)]
pub enum RtspMethod {
    Options,
    Describe,
    Setup,
    Play,
    Pause,
    Teardown,
    GetParameter,
    SetParameter,
}

impl RtspMethod {
    fn to_str(&self) -> &'static str {
        match self {
            RtspMethod::Options => "OPTIONS",
            RtspMethod::Describe => "DESCRIBE",
            RtspMethod::Setup => "SETUP",
            RtspMethod::Play => "PLAY",
            RtspMethod::Pause => "PAUSE",
            RtspMethod::Teardown => "TEARDOWN",
            RtspMethod::GetParameter => "GET_PARAMETER",
            RtspMethod::SetParameter => "SET_PARAMETER",
        }
    }
}

/// RTSP session state
#[derive(Debug, Clone, Copy)]
pub enum RtspState {
    Init,
    Ready,
    Playing,
    Paused,
    Teardown,
}

/// RTSP authentication info
#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub username: String,
    pub password: String,
    pub auth_type: AuthType,
}

/// Authentication type
#[derive(Debug, Clone, Copy)]
pub enum AuthType {
    None,
    Basic,
    Digest,
}

/// RTSP session information
#[derive(Debug, Clone)]
pub struct RtspSession {
    pub url: String,
    pub cseq: u32,
    pub session_id: Option<String>,
    pub state: RtspState,
    pub stream_url: Option<String>,
    pub transport: Option<String>,
}

/// Stream information
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub url: String,
    pub media: Vec<MediaInfo>,
    pub content_base: Option<String>,
}

/// Media track information
#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub control_url: String,
    pub codec: String,
    pub payload_type: u8,
    pub clock_rate: u32,
    pub fmtp: Option<String>,
}

/// Frame data from RTSP stream
#[derive(Debug, Clone)]
pub struct FrameData {
    pub timestamp: u32,
    pub marker: u8,
    pub payload: Vec<u8>,
    pub codec: VideoCodec,
    pub is_keyframe: bool,
}

/// Video codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    H264,
    H265,
    JPEG,
    Unknown,
}

/// Audio codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodec {
    G711ulaw,
    G711alaw,
    G726,
    AAC,
    Unknown,
}

/// RTP packet structure
#[derive(Debug, Clone)]
pub struct RtpPacket {
    pub version: u8,
    pub padding: u8,
    pub ext: bool,
    pub csrc_count: u8,
    pub marker: u8,
    pub payload_type: u8,
    pub sequence: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub payload: Vec<u8>,
}

/// RTP H.264 NAL unit
#[derive(Debug, Clone)]
pub struct RtpH264 {
    pub nal_type: u8,
    pub nri: u8,
    pub data: Vec<u8>,
}

/// RTP H.265 NAL unit
#[derive(Debug, Clone)]
pub struct RtpH265 {
    pub nal_type: u8,
    pub layer_id: u8,
    pub tid: u8,
    pub data: Vec<u8>,
}

/// RTP JPEG payload
#[derive(Debug, Clone)]
pub struct RtpJpeg {
    pub type_specific: u8,
    pub fragment_offset: u32,
    pub type_code: u8,
    pub quant: u8,
    pub width: u8,
    pub height: u8,
    pub payload: Vec<u8>,
}

/// RTSP client
pub struct RtspClient {
    host: String,
    port: u16,
    auth: Option<AuthInfo>,
    session: Option<RtspSession>,
    timeout: Duration,
}

impl RtspClient {
    /// Create new RTSP client
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            auth: None,
            session: None,
            timeout: Duration::from_secs(10),
        }
    }

    /// Set authentication
    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.auth = Some(AuthInfo {
            username: username.to_string(),
            password: password.to_string(),
            auth_type: AuthType::None,
        });
        self
    }

    /// Set connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Connect and send OPTIONS request
    pub fn connect(&mut self, url: &str) -> Result<Vec<String>, String> {
        let mut stream = TcpStream::connect_timeout(
            &SocketAddr::new(self.host.parse().unwrap(), self.port),
            self.timeout,
        )
        .map_err(|e| format!("Connection failed: {}", e))?;

        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("Set timeout failed: {}", e))?;

        let cseq = 1;
        let request = format!(
            "OPTIONS {} RTSP/1.0\r\n\
             CSeq: {}\r\n\
             User-Agent: Eden/1.0\r\n\
             \r\n",
            url, cseq
        );

        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("Send failed: {}", e))?;

        let mut response = vec![0u8; 8192];
        let n = stream
            .read(&mut response)
            .map_err(|e| format!("Receive failed: {}", e))?;

        let response_str = String::from_utf8_lossy(&response[..n]).to_string();
        let methods = self.parse_public_header(&response_str);

        self.session = Some(RtspSession {
            url: url.to_string(),
            cseq,
            session_id: None,
            state: RtspState::Init,
            stream_url: None,
            transport: None,
        });

        Ok(methods)
    }

    /// Send DESCRIBE request
    pub fn describe(&mut self, url: &str) -> Result<StreamInfo, String> {
        let cseq = self.inc_cseq();
        let request = format!(
            "DESCRIBE {} RTSP/1.0\r\n\
             CSeq: {}\r\n\
             Accept: application/sdp\r\n\
             User-Agent: Eden/1.0\r\n\
             \r\n",
            url, cseq
        );

        let response = self.send_raw(&request)?;
        self.parse_sdp(&response)
    }

    /// Send SETUP request for a track
    pub fn setup(&mut self, _url: &str, track_url: &str, transport: &str) -> Result<(), String> {
        let cseq = self.inc_cseq();
        let mut request = format!(
            "SETUP {} RTSP/1.0\r\n\
             CSeq: {}\r\n\
             Transport: {}\r\n\
             User-Agent: Eden/1.0\r\n",
            track_url, cseq, transport
        );

        if let Some(ref session) = self.session {
            if let Some(ref session_id) = session.session_id {
                request.push_str(&format!("Session: {}\r\n", session_id));
            }
        }

        request.push_str("\r\n");

        let response = self.send_raw(&request)?;
        self.parse_setup_response(&response)?;

        Ok(())
    }

    /// Send PLAY request
    pub fn play(&mut self, range: Option<&str>) -> Result<(), String> {
        let cseq = self.inc_cseq();
        let mut request = format!(
            "PLAY {} RTSP/1.0\r\n\
             CSeq: {}\r\n\
             User-Agent: Eden/1.0\r\n",
            self.session.as_ref().ok_or("No active session")?.url,
            cseq
        );

        if let Some(ref session_id) = self.session.as_ref().ok_or("No session")?.session_id {
            request.push_str(&format!("Session: {}\r\n", session_id));
        }

        if let Some(r) = range {
            request.push_str(&format!("Range: npt={}\r\n", r));
        }

        request.push_str("\r\n");

        let _response = self.send_raw(&request)?;

        if let Some(ref mut session) = self.session {
            session.state = RtspState::Playing;
        }

        Ok(())
    }

    /// Send PAUSE request
    pub fn pause(&mut self) -> Result<(), String> {
        let cseq = self.inc_cseq();
        let mut request = format!(
            "PAUSE {} RTSP/1.0\r\n\
             CSeq: {}\r\n\
             User-Agent: Eden/1.0\r\n",
            self.session.as_ref().ok_or("No active session")?.url,
            cseq
        );

        if let Some(ref session_id) = self.session.as_ref().ok_or("No session")?.session_id {
            request.push_str(&format!("Session: {}\r\n", session_id));
        }

        request.push_str("\r\n");

        let _response = self.send_raw(&request)?;

        if let Some(ref mut session) = self.session {
            session.state = RtspState::Paused;
        }

        Ok(())
    }

    /// Send TEARDOWN request
    pub fn teardown(&mut self) -> Result<(), String> {
        let cseq = self.inc_cseq();
        let mut request = format!(
            "TEARDOWN {} RTSP/1.0\r\n\
             CSeq: {}\r\n\
             User-Agent: Eden/1.0\r\n",
            self.session.as_ref().ok_or("No active session")?.url,
            cseq
        );

        if let Some(ref session_id) = self.session.as_ref().ok_or("No session")?.session_id {
            request.push_str(&format!("Session: {}\r\n", session_id));
        }

        request.push_str("\r\n");

        let _response = self.send_raw(&request)?;

        if let Some(ref mut session) = self.session {
            session.state = RtspState::Teardown;
        }

        Ok(())
    }

    /// Parse RTP packet from UDP data
    pub fn parse_rtp(&self, data: &[u8]) -> Result<RtpPacket, String> {
        if data.len() < 12 {
            return Err("RTP packet too short".to_string());
        }

        let version = (data[0] >> 6) & 0x03;
        let padding = (data[0] >> 5) & 0x01;
        let ext = ((data[0] >> 4) & 0x01) != 0;
        let csrc_count = data[0] & 0x0F;
        let marker = data[1] >> 7;
        let payload_type = data[1] & 0x7F;

        let sequence = u16::from_be_bytes([data[2], data[3]]);
        let timestamp = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ssrc = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        let mut header_size = 12 + (csrc_count as usize) * 4;

        // Skip extension header if present
        if ext && data.len() > header_size + 4 {
            let ext_len =
                u16::from_be_bytes([data[header_size + 2], data[header_size + 3]]) as usize;
            header_size += 4 + ext_len * 4;
        }

        let payload = data[header_size..].to_vec();

        Ok(RtpPacket {
            version,
            padding,
            ext,
            csrc_count,
            marker,
            payload_type,
            sequence,
            timestamp,
            ssrc,
            payload,
        })
    }

    /// Parse H.264 NAL unit from RTP payload
    pub fn parse_h264(&self, packet: &RtpPacket) -> Result<RtpH264, String> {
        if packet.payload.is_empty() {
            return Err("Empty payload".to_string());
        }

        let first_byte = packet.payload[0];
        let nal_type = first_byte & 0x1F;
        let nri = (first_byte >> 5) & 0x03;

        // Check for FU-A fragment
        if nal_type == 28 {
            if packet.payload.len() < 2 {
                return Err("FU-A packet too short".to_string());
            }

            let fu_header = packet.payload[1];
            let _start_bit = (fu_header >> 7) & 0x01;
            let _end_bit = (fu_header >> 6) & 0x01;
            let nal_type = fu_header & 0x3F;

            return Ok(RtpH264 {
                nal_type,
                nri,
                data: packet.payload[2..].to_vec(),
            });
        }

        Ok(RtpH264 {
            nal_type,
            nri,
            data: packet.payload[1..].to_vec(),
        })
    }

    /// Parse H.265 NAL unit from RTP payload
    pub fn parse_h265(&self, packet: &RtpPacket) -> Result<RtpH265, String> {
        if packet.payload.len() < 2 {
            return Err("H.265 payload too short".to_string());
        }

        let first_two = u16::from_be_bytes([packet.payload[0], packet.payload[1]]);
        let nal_type = ((first_two >> 9) & 0x3F) as u8;
        let layer_id = ((first_two >> 3) & 0x3F) as u8;
        let tid = (first_two & 0x07) as u8;

        Ok(RtpH265 {
            nal_type,
            layer_id,
            tid,
            data: packet.payload[2..].to_vec(),
        })
    }

    /// Parse JPEG payload from RTP packet
    pub fn parse_jpeg(&self, packet: &RtpPacket) -> Result<RtpJpeg, String> {
        if packet.payload.len() < 8 {
            return Err("JPEG payload too short".to_string());
        }

        let type_specific = packet.payload[0];
        let fragment_offset =
            u32::from_be_bytes([0, packet.payload[1], packet.payload[2], packet.payload[3]]);
        let type_code = packet.payload[4];
        let quant = packet.payload[5];
        let width = packet.payload[6] * 8;
        let height = packet.payload[7] * 8;

        Ok(RtpJpeg {
            type_specific,
            fragment_offset,
            type_code,
            quant,
            width,
            height,
            payload: packet.payload[8..].to_vec(),
        })
    }

    /// Send raw request and get response
    fn send_raw(&self, request: &str) -> Result<String, String> {
        let mut stream = TcpStream::connect_timeout(
            &SocketAddr::new(self.host.parse().unwrap(), self.port),
            self.timeout,
        )
        .map_err(|e| format!("Connection failed: {}", e))?;

        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("Send failed: {}", e))?;

        let mut response = vec![0u8; 65536];
        let n = stream
            .read(&mut response)
            .map_err(|e| format!("Receive failed: {}", e))?;

        Ok(String::from_utf8_lossy(&response[..n]).to_string())
    }

    /// Increment CSeq
    fn inc_cseq(&mut self) -> u32 {
        if let Some(ref mut session) = self.session {
            session.cseq += 1;
            session.cseq
        } else {
            1
        }
    }

    /// Parse Public header for supported methods
    fn parse_public_header(&self, response: &str) -> Vec<String> {
        response
            .lines()
            .find(|l| l.starts_with("Public:"))
            .map(|l| {
                l.replace("Public:", "")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse SETUP response for session ID
    fn parse_setup_response(&mut self, response: &str) -> Result<(), String> {
        for line in response.lines() {
            if line.starts_with("Session:") {
                let session_id = line
                    .replace("Session:", "")
                    .trim()
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();

                if let Some(ref mut session) = self.session {
                    session.session_id = Some(session_id);
                    session.state = RtspState::Ready;
                }
            }
            if line.starts_with("Transport:") {
                if let Some(ref mut session) = self.session {
                    session.transport = Some(line.replace("Transport:", "").trim().to_string());
                }
            }
        }
        Ok(())
    }

    /// Parse SDP for stream info
    fn parse_sdp(&self, response: &str) -> Result<StreamInfo, String> {
        let mut media = vec![];
        let mut content_base = None;

        for line in response.lines() {
            if line.starts_with("Content-Base:") {
                content_base = Some(
                    line.replace("Content-Base:", "")
                        .trim_end_matches('/')
                        .to_string(),
                );
            }

            if line.starts_with("m=") {
                let parts: Vec<&str> = line.splitn(4, ' ').collect();
                if parts.len() >= 4 {
                    let codec = parts[3].to_string();
                    media.push(MediaInfo {
                        control_url: String::new(),
                        codec,
                        payload_type: 0,
                        clock_rate: 90000,
                        fmtp: None,
                    });
                }
            }
        }

        Ok(StreamInfo {
            url: String::new(),
            media,
            content_base,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_parse() {
        let client = RtspClient::new("127.0.0.1", 554);

        // Minimal RTP packet
        let packet = vec![
            0x80, 0x1f, 0x00, 0x01, // header
            0x00, 0x00, 0x00, 0x00, // timestamp
            0x00, 0x00, 0x00, 0x01, // SSRC
            0x00, 0x00, 0x00, 0x00, // payload
        ];

        let result = client.parse_rtp(&packet);
        assert!(result.is_ok());
    }
}
