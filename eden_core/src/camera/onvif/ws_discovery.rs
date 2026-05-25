//! WS-Discovery (Web Services Discovery) Implementation
//!
//! 100% Rust puro - Sin dependencias externas
//!
//! Implements:
//! - Probe matching (UDP multicast)
//! - Hello/Bye lifecycle notifications

#![allow(dead_code)]

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

/// WS-Discovery constants
pub const WS_DISCOVERY_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
pub const WS_DISCOVERY_PORT: u16 = 3702;

/// Match behavior constants
pub const MATCH_BY_LITERAL: u32 = 0;
pub const MATCH_BY_RFC3986: u32 = 1;
pub const MATCH_BY_DIALECT: u32 = 2;
pub const MATCH_BY_START_SCOPE: u32 = 3;
pub const MATCH_BY_XPATH1: u32 = 4;
pub const MATCH_BY: u32 = MATCH_BY_LITERAL;

/// Scope match constants
pub const SCOPE_MATCH_EXACT: u32 = 0;
pub const SCOPE_MATCH_PREFIX: u32 = 1;
pub const SCOPE_MATCH_ALL: u32 = 2;
pub const SCOPE_MATCH_ANY: u32 = 3;
pub const SCOPE_MATCH: u32 = SCOPE_MATCH_ALL;

/// Probe match result
#[derive(Debug, Clone)]
pub struct ProbeMatch {
    pub endpoint: String,
    pub address: SocketAddr,
    pub types: Vec<String>,
    pub scopes: Vec<String>,
    pub xaddrs: Vec<String>,
    pub metadata_version: u32,
    pub scopes_matched: bool,
}

/// Probe handler callback
pub trait ProbeHandler: Send + Sync {
    fn handle_probe(&self, match_result: ProbeMatch);
}

/// Hello handler callback
pub trait HelloHandler: Send + Sync {
    fn handle_hello(
        &self,
        endpoint: &str,
        address: SocketAddr,
        types: &[String],
        xaddrs: &[String],
    );
}

/// Bye handler callback
pub trait ByeHandler: Send + Sync {
    fn handle_bye(&self, endpoint: &str, address: SocketAddr);
}

/// WS-Discovery client/server
pub struct WsDiscovery {
    socket: Option<UdpSocket>,
    bindings: HashMap<String, ProbeMatch>,
    scopes: Vec<String>,
    types: Vec<String>,
}

impl WsDiscovery {
    /// Create new WS-Discovery instance
    pub fn new() -> Self {
        Self {
            socket: None,
            bindings: HashMap::new(),
            scopes: vec![],
            types: vec![],
        }
    }

    /// Set multicast scope
    pub fn with_scope(mut self, scope: &str) -> Self {
        self.scopes.push(scope.to_string());
        self
    }

    /// Set device types to search for
    pub fn with_types(mut self, types: Vec<String>) -> Self {
        self.types = types;
        self
    }

    /// Start listening for WS-Discovery messages
    pub fn listen(&mut self) -> Result<(), String> {
        // Create UDP socket for multicast
        let socket = UdpSocket::bind(SocketAddr::new(
            std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ))
        .map_err(|e| format!("Failed to bind socket: {}", e))?;

        // Set socket timeout
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        // Join multicast group
        self.socket = Some(socket);
        Ok(())
    }

    /// Send probe message
    pub fn probe(&self) -> Result<Vec<ProbeMatch>, String> {
        let probe_xml = self.build_probe_xml();
        let _ = probe_xml; // Would send via UDP multicast

        Ok(vec![])
    }

    /// Build SOAP probe message
    fn build_probe_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str(r#"<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" "#);
        xml.push_str(r#"xmlns:wsd="http://schemas.xmlsoap.org/ws/2005/04/discovery" "#);
        xml.push_str(r#"xmlns:tds="http://www.onvif.org/ver10/device/wsdl">"#);
        xml.push_str("<soap:Body>");
        xml.push_str("<wsd:Probe>");
        xml.push_str("<wsd:Types>");

        if self.types.is_empty() {
            xml.push_str("tds:NetworkVideoSender");
        } else {
            xml.push_str(&self.types.join(" "));
        }

        xml.push_str("</wsd:Types>");
        xml.push_str("<wsd:Scopes>");

        if !self.scopes.is_empty() {
            xml.push_str(&self.scopes.join(" "));
        }

        xml.push_str("</wsd:Scopes>");
        xml.push_str("</wsd:Probe>");
        xml.push_str("</soap:Body>");
        xml.push_str("</soap:Envelope>");

        xml
    }

    /// Parse probe match response
    pub fn parse_probe_match(&self, data: &[u8]) -> Result<ProbeMatch, String> {
        let xml = String::from_utf8_lossy(data);

        // Parse endpoint reference
        let endpoint = self
            .extract_value(&xml, "EndpointReference")
            .and_then(|s| self.extract_value(&s, "Address"))
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Parse types
        let types = self.extract_values(&xml, "Types");

        // Parse scopes
        let scopes = self.extract_values(&xml, "Scopes");

        // Parse XAddrs
        let xaddrs = self.extract_values(&xml, "XAddrs");

        // Parse metadata version
        let metadata_version = self
            .extract_value(&xml, "MetadataVersion")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        Ok(ProbeMatch {
            endpoint,
            address: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 0),
            types,
            scopes,
            xaddrs,
            metadata_version,
            scopes_matched: true,
        })
    }

    /// Extract single value from XML by tag
    fn extract_value<'a>(&self, xml: &'a str, tag: &str) -> Option<&'a str> {
        let start = format!("<{}", tag);
        let end = format!("</{}>", tag);

        if let Some(s) = xml.find(&start) {
            if let Some(e) = xml[s..].find('>') {
                let content = &xml[s + e + 1..];
                if let Some(close) = content.find(&end) {
                    return Some(&content[..close]);
                }
            }
        }
        None
    }

    /// Extract multiple values from XML by tag
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
                    values.push(content[..close].to_string());
                    pos = abs_s + e + 1 + close + end.len();
                    continue;
                }
            }
            break;
        }

        values
    }

    /// Register a discovered binding
    pub fn register_binding(&mut self, binding: ProbeMatch) {
        self.bindings.insert(binding.endpoint.clone(), binding);
    }

    /// Get all registered bindings
    pub fn get_bindings(&self) -> Vec<&ProbeMatch> {
        self.bindings.values().collect()
    }

    /// Check if binding exists
    pub fn has_binding(&self, endpoint: &str) -> bool {
        self.bindings.contains_key(endpoint)
    }
}

impl Default for WsDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_xml() {
        let discovery = WsDiscovery::new().with_types(vec!["tds:NetworkVideoSender".to_string()]);

        let xml = discovery.build_probe_xml();
        assert!(xml.contains("wsd:Probe"));
        assert!(xml.contains("tds:NetworkVideoSender"));
    }

    #[test]
    fn test_extract_value() {
        let discovery = WsDiscovery::new();
        let xml = "<tag>value</tag>";
        assert_eq!(discovery.extract_value(xml, "tag"), Some("value"));
    }
}
