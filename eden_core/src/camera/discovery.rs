//! Camera Discovery Module
//!
//! 100% Rust puro - Sin dependencias externas
//!
//! Implements WS-Discovery and ONVIF probe matching

#![allow(dead_code)]

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

/// Discovery filter criteria
#[derive(Debug, Clone)]
pub struct ProbeFilter {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
    pub binding: Option<String>,
}

/// Probe match from discovery
#[derive(Debug, Clone)]
pub struct ProbeMatch {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
    pub xaddrs: Vec<String>,
    pub scopes_matched: bool,
}

/// Discovered camera info
#[derive(Debug, Clone)]
pub struct DiscoveredCamera {
    pub uuid: String,
    pub endpoint: String,
    pub address: SocketAddr,
    pub types: Vec<String>,
    pub xaddrs: Vec<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
}

/// Camera discovery result
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub cameras: Vec<DiscoveredCamera>,
    pub duration_ms: u64,
    pub probe_count: u32,
}

/// Camera discovery service
pub struct CameraDiscovery {
    socket: Option<UdpSocket>,
    filters: Vec<ProbeFilter>,
    discovered: HashMap<String, DiscoveredCamera>,
}

impl CameraDiscovery {
    /// Create new discovery service
    pub fn new() -> Self {
        Self {
            socket: None,
            filters: vec![],
            discovered: HashMap::new(),
        }
    }

    /// Add probe filter
    pub fn with_filter(mut self, filter: ProbeFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Start listening for WS-Discovery messages
    pub fn listen(&mut self) -> Result<(), String> {
        // Create UDP socket
        let socket = UdpSocket::bind(SocketAddr::new(
            std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ))
        .map_err(|e| format!("Failed to bind: {}", e))?;

        socket
            .set_broadcast(true)
            .map_err(|e| format!("Failed to set broadcast: {}", e))?;

        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        self.socket = Some(socket);
        Ok(())
    }

    /// Probe for cameras on the network
    pub fn probe(&mut self, timeout_ms: u64) -> Result<DiscoveryResult, String> {
        let start = std::time::Instant::now();
        let mut cameras = vec![];
        let mut probe_count = 0;

        // Send WS-Discovery probe via multicast
        let probe_msg = self.build_probe_xml();

        if let Some(ref socket) = self.socket {
            let multicast_addr = SocketAddr::new(Ipv4Addr::new(239, 255, 255, 250).into(), 3702);

            // Send probe
            socket
                .send_to(probe_msg.as_bytes(), multicast_addr)
                .map_err(|e| format!("Probe send failed: {}", e))?;
            probe_count += 1;

            // Listen for responses
            let mut buf = [0u8; 8192];
            let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);

            while std::time::Instant::now() < deadline {
                if let Ok((len, addr)) = socket.recv_from(&mut buf) {
                    if let Ok(camera) = self.parse_probe_match(&buf[..len], addr) {
                        if !self.discovered.contains_key(&camera.uuid) {
                            self.discovered.insert(camera.uuid.clone(), camera.clone());
                            cameras.push(camera);
                        }
                    }
                }
            }
        }

        Ok(DiscoveryResult {
            cameras,
            duration_ms: start.elapsed().as_millis() as u64,
            probe_count,
        })
    }

    /// Probe specific IP address
    pub fn probe_address(
        &mut self,
        address: SocketAddr,
    ) -> Result<Option<DiscoveredCamera>, String> {
        let probe_msg = self.build_probe_xml();

        if let Some(ref socket) = self.socket {
            socket
                .send_to(probe_msg.as_bytes(), address)
                .map_err(|e| format!("Probe send failed: {}", e))?;

            let mut buf = [0u8; 8192];
            socket
                .set_read_timeout(Some(Duration::from_millis(500)))
                .map_err(|e| format!("Set timeout failed: {}", e))?;

            if let Ok((len, addr)) = socket.recv_from(&mut buf) {
                if let Ok(camera) = self.parse_probe_match(&buf[..len], addr) {
                    return Ok(Some(camera));
                }
            }
        }

        Ok(None)
    }

    /// Build WS-Discovery probe message
    fn build_probe_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                xmlns:wsd="http://schemas.xmlsoap.org/ws/2005/04/discovery"
                xmlns:dn="http://www.onvif.org/ver10/network/wsdl">
<soap:Body>
<wsd:Probe>
<wsd:Types>dn:NetworkVideoTransmitter</wsd:Types>
<wsd:Scopes></wsd:Scopes>
</wsd:Probe>
</soap:Body>
</soap:Envelope>"#
        )
    }

    /// Parse WS-Discovery probe match response
    fn parse_probe_match(&self, data: &[u8], addr: SocketAddr) -> Result<DiscoveredCamera, String> {
        let xml = String::from_utf8_lossy(data);

        // Extract UUID from EndpointReference
        let uuid = self
            .extract_value(&xml, "Address")
            .and_then(|s| s.split(':').last().map(|u| u.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        // Extract types
        let types = self.extract_values(&xml, "Types");

        // Extract XAddrs
        let xaddrs = self.extract_values(&xml, "XAddrs");

        // Extract scopes
        let _scopes = self.extract_values(&xml, "Scopes");

        // Extract endpoint
        let endpoint = self
            .extract_value(&xml, "EndpointReference")
            .map(|s| format!("urn:uuid:{}", s))
            .unwrap_or_else(|| format!("urn:uuid:{}", uuid));

        Ok(DiscoveredCamera {
            uuid,
            endpoint,
            address: addr,
            types,
            xaddrs: xaddrs.clone(),
            manufacturer: None,
            model: None,
        })
    }

    /// Extract single value from XML
    fn extract_value<'a>(&self, xml: &'a str, tag: &str) -> Option<&'a str> {
        let start = format!("<{}", tag);
        if let Some(s) = xml.find(&start) {
            if let Some(e) = xml[s..].find('>') {
                let content = &xml[s + e + 1..];
                if let Some(end) = content.find(&format!("</{}>", tag)) {
                    return Some(&content[..end]);
                }
            }
        }
        None
    }

    /// Extract multiple values from XML
    fn extract_values(&self, xml: &str, tag: &str) -> Vec<String> {
        let mut values = vec![];
        let mut pos = 0;
        let start = format!("<{}", tag);
        let end = format!("</{}>", tag);

        while let Some(s) = xml[pos..].find(&start) {
            let abs_s = pos + s;
            if let Some(e) = xml[abs_s..].find('>') {
                let content = &xml[abs_s + e + 1..];
                if let Some(close) = content.find(&end) {
                    let value = content[..close].trim().to_string();
                    if !value.is_empty() {
                        values.push(value);
                    }
                    pos = abs_s + e + 1 + close + end.len();
                    continue;
                }
            }
            break;
        }

        values
    }

    /// Get all discovered cameras
    pub fn get_discovered(&self) -> Vec<&DiscoveredCamera> {
        self.discovered.values().collect()
    }

    /// Check if camera already discovered
    pub fn is_discovered(&self, uuid: &str) -> bool {
        self.discovered.contains_key(uuid)
    }

    /// Clear discovered cameras
    pub fn clear(&mut self) {
        self.discovered.clear();
    }
}

impl Default for CameraDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
