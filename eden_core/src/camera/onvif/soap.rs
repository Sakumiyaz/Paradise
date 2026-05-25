//! SOAP 1.1/1.2 Implementation
//!
//! 100% Rust puro - Sin dependencias externas
//!
//! Handles:
//! - SOAP envelope construction/parsing
//! - SOAP fault handling
//! - SOAP action routing
//! - namespaces (soap, wsdl, onvif, xsi, xsd)

#![allow(dead_code)]

use std::collections::HashMap;

/// SOAP version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoapVersion {
    Soap11,
    Soap12,
}

/// SOAP envelope structure
#[derive(Debug, Clone)]
pub struct SoapEnvelope {
    pub version: SoapVersion,
    pub header: Option<SoapHeader>,
    pub body: SoapBody,
    pub namespaces: HashMap<String, String>,
}

/// SOAP header (optional)
#[derive(Debug, Clone)]
pub struct SoapHeader {
    pub elements: Vec<XmlElement>,
}

/// SOAP body with content
#[derive(Debug, Clone)]
pub struct SoapBody {
    pub elements: Vec<XmlElement>,
    pub fault: Option<SoapFault>,
}

/// SOAP fault structure
#[derive(Debug, Clone)]
pub struct SoapFault {
    pub fault_code: String,
    pub fault_string: String,
    pub actor: Option<String>,
    pub detail: Option<String>,
}

/// SOAP message with action
#[derive(Debug, Clone)]
pub struct SoapMessage {
    pub envelope: SoapEnvelope,
    pub action: SoapAction,
    pub to: Option<String>,
    pub reply_to: Option<String>,
    pub message_id: Option<String>,
}

/// SOAP action types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoapAction {
    GetDeviceInfo,
    GetCapabilities,
    GetServices,
    GetNetworkInterfaces,
    GetSystemDateAndTime,
    GetProfiles,
    GetStreamUri,
    GetSnapshotUri,
    GetVideoSources,
    GetVideoSourceConfiguration,
    GetVideoEncoderConfiguration,
    GetAudioSources,
    GetAudioEncoderConfiguration,
    GetPTZConfigurations,
    GetStatus,
    ContinuousMove,
    Stop,
    Custom(String),
}

impl SoapAction {
    /// Get SOAP action URI
    pub fn to_uri(&self, service: &str) -> String {
        match self {
            SoapAction::GetDeviceInfo => format!("{}/GetDeviceInformation", service),
            SoapAction::GetCapabilities => format!("{}/GetCapabilities", service),
            SoapAction::GetServices => format!("{}/GetServices", service),
            SoapAction::GetNetworkInterfaces => format!("{}/GetNetworkInterfaces", service),
            SoapAction::GetSystemDateAndTime => format!("{}/GetSystemDateAndTime", service),
            SoapAction::GetProfiles => format!("{}/GetProfiles", service),
            SoapAction::GetStreamUri => format!("{}/GetStreamUri", service),
            SoapAction::GetSnapshotUri => format!("{}/GetSnapshotUri", service),
            SoapAction::GetVideoSources => format!("{}/GetVideoSources", service),
            SoapAction::GetVideoSourceConfiguration => {
                format!("{}/GetVideoSourceConfiguration", service)
            }
            SoapAction::GetVideoEncoderConfiguration => {
                format!("{}/GetVideoEncoderConfiguration", service)
            }
            SoapAction::GetAudioSources => format!("{}/GetAudioSources", service),
            SoapAction::GetAudioEncoderConfiguration => {
                format!("{}/GetAudioEncoderConfiguration", service)
            }
            SoapAction::GetPTZConfigurations => format!("{}/GetPTZConfigurations", service),
            SoapAction::GetStatus => format!("{}/GetStatus", service),
            SoapAction::ContinuousMove => format!("{}/ContinuousMove", service),
            SoapAction::Stop => format!("{}/Stop", service),
            SoapAction::Custom(action) => action.clone(),
        }
    }
}

/// XML element for SOAP body
#[derive(Debug, Clone)]
pub struct XmlElement {
    pub tag: String,
    pub namespace: Option<String>,
    pub attributes: Vec<XmlAttribute>,
    pub children: Vec<XmlElement>,
    pub text: Option<String>,
}

/// XML attribute
#[derive(Debug, Clone)]
pub struct XmlAttribute {
    pub name: String,
    pub namespace: Option<String>,
    pub value: String,
}

/// XML namespace definitions
#[derive(Debug, Clone)]
pub struct XmlNamespace {
    pub prefix: String,
    pub uri: String,
}

impl SoapEnvelope {
    /// Create new SOAP envelope
    pub fn new(version: SoapVersion, body: SoapBody) -> Self {
        let mut namespaces = HashMap::new();
        namespaces.insert(
            "soap".to_string(),
            if version == SoapVersion::Soap11 {
                "http://schemas.xmlsoap.org/soap/envelope/".to_string()
            } else {
                "http://www.w3.org/2003/05/soap-envelope".to_string()
            },
        );
        namespaces.insert(
            "wsdl".to_string(),
            "http://schemas.xmlsoap.org/wsdl/".to_string(),
        );
        namespaces.insert(
            "xsi".to_string(),
            "http://www.w3.org/2001/XMLSchema-instance".to_string(),
        );
        namespaces.insert(
            "xsd".to_string(),
            "http://www.w3.org/2001/XMLSchema".to_string(),
        );
        namespaces.insert(
            "onvif".to_string(),
            "http://www.onvif.org/ver10/device/wsdl".to_string(),
        );

        Self {
            version,
            header: None,
            body,
            namespaces,
        }
    }

    /// Create SOAP fault
    pub fn fault(version: SoapVersion, code: &str, string: &str, detail: Option<String>) -> Self {
        let fault = SoapFault {
            fault_code: code.to_string(),
            fault_string: string.to_string(),
            actor: None,
            detail,
        };
        let body = SoapBody {
            elements: vec![],
            fault: Some(fault),
        };
        Self::new(version, body)
    }

    /// Serialize to XML string
    pub fn to_xml(&self) -> String {
        let soap_ns = if self.version == SoapVersion::Soap11 {
            "http://schemas.xmlsoap.org/soap/envelope/"
        } else {
            "http://www.w3.org/2003/05/soap-envelope"
        };

        let mut xml = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!("<soap:Envelope xmlns:soap=\"{}\">\n", soap_ns));

        if let Some(header) = &self.header {
            xml.push_str("  <soap:Header>\n");
            for elem in &header.elements {
                xml.push_str(&Self::element_to_xml(elem, 2));
            }
            xml.push_str("  </soap:Header>\n");
        }

        xml.push_str("  <soap:Body>\n");
        if let Some(fault) = &self.body.fault {
            xml.push_str("    <soap:Fault>\n");
            xml.push_str(&format!(
                "      <faultcode>{}</faultcode>\n",
                fault.fault_code
            ));
            xml.push_str(&format!(
                "      <faultstring>{}</faultstring>\n",
                fault.fault_string
            ));
            if let Some(detail) = &fault.detail {
                xml.push_str(&format!("      <detail>{}</detail>\n", detail));
            }
            xml.push_str("    </soap:Fault>\n");
        } else {
            for elem in &self.body.elements {
                xml.push_str(&Self::element_to_xml(elem, 2));
            }
        }
        xml.push_str("  </soap:Body>\n");
        xml.push_str("</soap:Envelope>");

        xml
    }

    /// Serialize element to XML
    fn element_to_xml(elem: &XmlElement, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let ns_prefix = elem
            .namespace
            .as_ref()
            .map(|ns| format!("{}:", ns))
            .unwrap_or_default();

        let mut xml = format!("{}<{}{}", indent_str, ns_prefix, elem.tag);

        for attr in &elem.attributes {
            let ns_prefix = attr
                .namespace
                .as_ref()
                .map(|ns| format!("{}:", ns))
                .unwrap_or_default();
            xml.push_str(&format!(" {}{}=\"{}\"", ns_prefix, attr.name, attr.value));
        }

        if elem.children.is_empty() && elem.text.is_none() {
            xml.push_str("/>\n");
        } else {
            xml.push_str(">\n");
            if let Some(text) = &elem.text {
                xml.push_str(&format!("{}{}{}\n", indent_str, "  ".repeat(1), text));
            }
            for child in &elem.children {
                xml.push_str(&Self::element_to_xml(child, indent + 1));
            }
            xml.push_str(&format!("{}</{}{}>\n", indent_str, ns_prefix, elem.tag));
        }

        xml
    }

    /// Parse from XML string
    pub fn from_xml(xml: &str) -> Result<Self, String> {
        let version = if xml.contains("http://schemas.xmlsoap.org/soap/envelope/") {
            SoapVersion::Soap11
        } else {
            SoapVersion::Soap12
        };

        let mut header = None;
        let mut body = SoapBody {
            elements: vec![],
            fault: None,
        };

        // Parse header if present
        if let Some(header_start) = xml.find("<soap:Header>") {
            if let Some(header_end) = xml.find("</soap:Header>") {
                let header_xml = &xml[header_start + 14..header_end];
                let elements = Self::parse_elements(header_xml)?;
                header = Some(SoapHeader { elements });
            }
        }

        // Parse body
        if let Some(body_start) = xml.find("<soap:Body>") {
            if let Some(body_end) = xml.find("</soap:Body>") {
                let body_xml = &xml[body_start + 11..body_end];

                // Check for fault
                if body_xml.contains("<soap:Fault") || body_xml.contains("<Fault") {
                    body.fault = Some(Self::parse_fault(body_xml)?);
                } else {
                    body.elements = Self::parse_elements(body_xml)?;
                }
            }
        }

        let mut envelope = Self::new(version, body);
        envelope.header = header;
        Ok(envelope)
    }

    /// Parse fault from XML
    fn parse_fault(xml: &str) -> Result<SoapFault, String> {
        let fault_code = Self::extract_tag(xml, "faultcode")
            .or_else(|| Self::extract_tag(xml, "faultcode"))
            .unwrap_or_else(|| "Server".to_string());
        let fault_string = Self::extract_tag(xml, "faultstring")
            .or_else(|| Self::extract_tag(xml, "faultstring"))
            .unwrap_or_else(|| "Unknown error".to_string());
        let detail = Self::extract_tag(xml, "detail");

        Ok(SoapFault {
            fault_code,
            fault_string,
            actor: None,
            detail,
        })
    }

    /// Extract content of a tag
    fn extract_tag(xml: &str, tag: &str) -> Option<String> {
        let start_pattern = format!("<{}", tag);
        let end_pattern = format!("</{}>", tag);

        if let Some(start) = xml.find(&start_pattern) {
            if let Some(content_start) = xml[start..].find('>') {
                let content = &xml[start..start + content_start + 1];
                if content.contains("/>") {
                    return Some(String::new());
                }
                if let Some(end) = xml[start + content_start + 1..].find(&end_pattern) {
                    return Some(
                        xml[start + content_start + 1..start + content_start + 1 + end].to_string(),
                    );
                }
            }
        }
        None
    }

    /// Parse XML elements
    fn parse_elements(xml: &str) -> Result<Vec<XmlElement>, String> {
        let mut elements = vec![];
        let mut pos = 0;

        while let Some(tag_start_rel) = xml[pos..].find('<') {
            let tag_start = pos + tag_start_rel;
            let tag_end = tag_start
                + xml[tag_start..]
                    .find('>')
                    .ok_or_else(|| "Unclosed XML tag".to_string())?;
            let tag_content = xml[tag_start + 1..tag_end].trim();

            // Skip comments and processing instructions
            if tag_content.starts_with("!--") || tag_content.starts_with('?') {
                pos = tag_end + 1;
                continue;
            }

            if tag_content.starts_with('/') {
                break;
            }

            if tag_content.ends_with('/') {
                let tag_name = tag_content.trim_end_matches('/').trim();
                if let Some((tag, attributes)) = Self::parse_tag_name(tag_name) {
                    elements.push(XmlElement {
                        tag,
                        namespace: None,
                        attributes,
                        children: vec![],
                        text: None,
                    });
                }
                pos = tag_end + 1;
                continue;
            }

            if let Some((tag_name, attributes)) = Self::parse_tag_name(tag_content) {
                let end_tag = format!("</{}>", tag_name);
                let elem_end = Self::find_matching_end(xml, tag_end + 1, &tag_name)
                    .ok_or_else(|| format!("Missing closing tag for {}", tag_name))?;
                let elem_content = &xml[tag_end + 1..elem_end];
                let children = Self::parse_elements(elem_content)?;
                let text = if children.is_empty() {
                    Some(elem_content.trim().to_string()).filter(|s| !s.is_empty())
                } else {
                    None
                };

                elements.push(XmlElement {
                    tag: tag_name,
                    namespace: None,
                    attributes,
                    children,
                    text,
                });
                pos = elem_end + end_tag.len();
                continue;
            }

            pos = tag_end + 1;
        }

        Ok(elements)
    }

    fn find_matching_end(xml: &str, start: usize, tag_name: &str) -> Option<usize> {
        let open_pattern = format!("<{}", tag_name);
        let close_pattern = format!("</{}>", tag_name);
        let mut depth = 1usize;
        let mut cursor = start;

        while let Some(tag_rel) = xml[cursor..].find('<') {
            let tag_start = cursor + tag_rel;

            if xml[tag_start..].starts_with(&close_pattern) {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(tag_start);
                }
                cursor = tag_start + close_pattern.len();
                continue;
            }

            if xml[tag_start..].starts_with(&open_pattern) {
                let after_name = tag_start + open_pattern.len();
                let valid_boundary = xml[after_name..]
                    .chars()
                    .next()
                    .map(|c| c.is_whitespace() || c == '>' || c == '/')
                    .unwrap_or(false);
                if valid_boundary {
                    let tag_end = tag_start + xml[tag_start..].find('>')?;
                    if !xml[tag_start + 1..tag_end].trim_end().ends_with('/') {
                        depth += 1;
                    }
                    cursor = tag_end + 1;
                    continue;
                }
            }

            cursor = tag_start + 1;
        }

        None
    }

    /// Parse tag name and attributes
    fn parse_tag_name(tag_content: &str) -> Option<(String, Vec<XmlAttribute>)> {
        let parts: Vec<&str> = tag_content.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let tag = parts[0].to_string();
        let mut attributes = vec![];

        let attr_str = parts[1..].join(" ");
        let mut attr_pos = 0;

        while attr_pos < attr_str.len() {
            // Skip whitespace
            while attr_pos < attr_str.len() && attr_str[attr_pos..].starts_with(' ') {
                attr_pos += 1;
            }

            if attr_pos >= attr_str.len() {
                break;
            }

            // Find attribute name
            let name_start = attr_pos;
            while attr_pos < attr_str.len() && attr_str[attr_pos..].chars().next() != Some('=') {
                attr_pos += 1;
            }

            if attr_pos >= attr_str.len() {
                break;
            }

            let name = attr_str[name_start..attr_pos].trim().to_string();
            attr_pos += 1; // Skip =

            // Find attribute value (quoted)
            if attr_pos < attr_str.len() {
                let quote = attr_str[attr_pos..].chars().next().unwrap_or('"');
                attr_pos += 1;
                let value_start = attr_pos;
                while attr_pos < attr_str.len()
                    && attr_str[attr_pos..].chars().next() != Some(quote)
                {
                    attr_pos += 1;
                }
                let value = attr_str[value_start..attr_pos].to_string();
                attr_pos += 1; // Skip closing quote

                attributes.push(XmlAttribute {
                    name,
                    namespace: None,
                    value,
                });
            }
        }

        Some((tag, attributes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soap_envelope_creation() {
        let body = SoapBody {
            elements: vec![XmlElement {
                tag: "GetDeviceInformation".to_string(),
                namespace: Some("ns".to_string()),
                attributes: vec![],
                children: vec![],
                text: None,
            }],
            fault: None,
        };

        let envelope = SoapEnvelope::new(SoapVersion::Soap11, body);
        let xml = envelope.to_xml();

        assert!(xml.contains("soap:Envelope"));
        assert!(xml.contains("GetDeviceInformation"));
    }

    #[test]
    fn test_soap_parse() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
  <soap:Body>
    <GetDeviceInformationResponse>
      <Manufacturer>TestCam</Manufacturer>
    </GetDeviceInformationResponse>
  </soap:Body>
</soap:Envelope>"#;

        let envelope = SoapEnvelope::from_xml(xml).unwrap();
        assert_eq!(envelope.body.elements.len(), 1);
        assert_eq!(
            envelope.body.elements[0].tag,
            "GetDeviceInformationResponse"
        );
    }
}
