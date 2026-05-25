//! XML Parsing Utilities for ONVIF
//!
//! 100% Rust puro - Sin dependencias externas
//!
//! Provides XML parsing and manipulation utilities
#![allow(unused_imports)]
#![allow(dead_code)]
use std::io::BufReader;

/// XML parser state machine
#[derive(Debug, Clone)]
pub enum XmlParserState {
    Start,
    InTag,
    InContent,
    InAttribute,
    InAttributeValue,
    Error,
}

/// Basic XML node representation
#[derive(Debug, Clone)]
pub struct XmlNode {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<XmlNode>,
    pub content: String,
}

impl XmlNode {
    /// Create new XML node
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            attributes: vec![],
            children: vec![],
            content: String::new(),
        }
    }

    /// Find child by name
    pub fn child(&self, name: &str) -> Option<&XmlNode> {
        self.children.iter().find(|c| c.name == name)
    }

    /// Get attribute value
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Get text content
    pub fn text(&self) -> &str {
        &self.content
    }
}

/// Simple XML tokenizer
pub struct XmlTokenizer {
    input: Vec<char>,
    pos: usize,
    in_open_tag: bool,
}

impl XmlTokenizer {
    /// Create new tokenizer
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            in_open_tag: false,
        }
    }

    /// Get next token
    pub fn next_token(&mut self) -> Option<XmlToken> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return None;
        }

        match self.input[self.pos] {
            '<' => {
                self.pos += 1;
                if self.pos < self.input.len() {
                    match self.input[self.pos] {
                        '/' => {
                            self.pos += 1;
                            let name = self.read_name();
                            Some(XmlToken::CloseTag(name))
                        }
                        '!' => {
                            self.pos += 1;
                            if self.pos < self.input.len() && self.input[self.pos] == '-' {
                                self.pos += 1;
                                if self.pos < self.input.len() && self.input[self.pos] == '-' {
                                    // Comment
                                    self.pos += 1;
                                    loop {
                                        if self.pos + 2 >= self.input.len() {
                                            break;
                                        }
                                        if self.input[self.pos] == '-'
                                            && self.input[self.pos + 1] == '-'
                                            && self.input[self.pos + 2] == '>'
                                        {
                                            self.pos += 3;
                                            break;
                                        }
                                        self.pos += 1;
                                    }
                                    return self.next_token();
                                }
                            }
                            Some(XmlToken::Comment)
                        }
                        '?' => {
                            self.pos += 1;
                            let _ = self.read_name();
                            Some(XmlToken::ProcessingInstruction)
                        }
                        _ => {
                            let name = self.read_name();
                            self.in_open_tag = true;
                            Some(XmlToken::OpenTag(name))
                        }
                    }
                } else {
                    None
                }
            }
            '>' => {
                self.pos += 1;
                self.in_open_tag = false;
                Some(XmlToken::AngleClose)
            }
            '/' => {
                self.pos += 1;
                if self.pos < self.input.len() && self.input[self.pos] == '>' {
                    self.pos += 1;
                    self.in_open_tag = false;
                    Some(XmlToken::SelfClose)
                } else {
                    Some(XmlToken::Slash)
                }
            }
            '=' => {
                self.pos += 1;
                Some(XmlToken::Equals)
            }
            '"' | '\'' => {
                let quote = self.input[self.pos];
                self.pos += 1;
                let value = self.read_until(quote);
                self.pos += 1; // Skip closing quote
                Some(XmlToken::AttributeValue(value))
            }
            _ => {
                if self.in_open_tag {
                    let _name = self.read_name();
                    self.skip_whitespace();
                    if self.pos < self.input.len() && self.input[self.pos] == '=' {
                        self.pos += 1;
                        self.skip_whitespace();
                        if self.pos < self.input.len()
                            && (self.input[self.pos] == '"' || self.input[self.pos] == '\'')
                        {
                            let quote = self.input[self.pos];
                            self.pos += 1;
                            let value = self.read_until(quote);
                            if self.pos < self.input.len() {
                                self.pos += 1;
                            }
                            return Some(XmlToken::AttributeValue(value));
                        }
                    }
                }
                let content = self.read_content();
                Some(XmlToken::Text(content))
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                ' ' | '\n' | '\r' | '\t' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn read_name(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | ':' | '.' => self.pos += 1,
                _ => break,
            }
        }
        self.input[start..self.pos].iter().collect()
    }

    fn read_content(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                '<' | '>' | '\n' => break,
                _ => self.pos += 1,
            }
        }
        self.input[start..self.pos].iter().collect()
    }

    fn read_until(&mut self, until: char) -> String {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos] != until {
            self.pos += 1;
        }
        self.input[start..self.pos].iter().collect()
    }
}

/// XML tokens
#[derive(Debug, Clone, PartialEq)]
pub enum XmlToken {
    OpenTag(String),
    CloseTag(String),
    SelfClose,
    AngleClose,
    Equals,
    Slash,
    Text(String),
    AttributeValue(String),
    Comment,
    ProcessingInstruction,
    Eof,
}

/// Parse XML to tree structure
pub fn parse_xml(input: &str) -> Result<XmlNode, String> {
    let input = input.trim();
    let start = input
        .find('<')
        .ok_or_else(|| "Failed to parse XML".to_string())?;
    parse_node_at(input, start).map(|(node, _)| node)
}

fn parse_node_at(xml: &str, start: usize) -> Result<(XmlNode, usize), String> {
    let tag_end = start
        .checked_add(
            xml[start..]
                .find('>')
                .ok_or_else(|| "Unclosed XML tag".to_string())?,
        )
        .ok_or_else(|| "XML index overflow".to_string())?;
    let raw_tag = xml[start + 1..tag_end].trim();

    if raw_tag.starts_with('/') || raw_tag.starts_with('!') || raw_tag.starts_with('?') {
        return Err("Expected XML opening tag".to_string());
    }

    let self_closing = raw_tag.ends_with('/');
    let raw_tag = raw_tag.trim_end_matches('/').trim();
    let mut parts = raw_tag.split_whitespace();
    let name = parts
        .next()
        .ok_or_else(|| "Expected XML tag name".to_string())?;
    let mut node = XmlNode::new(name);

    for attr in parts {
        if let Some((name, value)) = attr.split_once('=') {
            node.attributes.push((
                name.to_string(),
                value.trim_matches('"').trim_matches('\'').to_string(),
            ));
        }
    }

    if self_closing {
        return Ok((node, tag_end + 1));
    }

    let close_pattern = format!("</{}>", node.name);
    let mut cursor = tag_end + 1;
    let mut text = String::new();

    while cursor < xml.len() {
        if xml[cursor..].starts_with(&close_pattern) {
            node.content = text.trim().to_string();
            return Ok((node, cursor + close_pattern.len()));
        }

        if let Some(next_tag_rel) = xml[cursor..].find('<') {
            let next_tag = cursor + next_tag_rel;
            if !xml[cursor..next_tag].trim().is_empty() {
                text.push_str(xml[cursor..next_tag].trim());
            }
            if xml[next_tag..].starts_with("</") {
                cursor = next_tag;
                continue;
            }
            let (child, next_cursor) = parse_node_at(xml, next_tag)?;
            node.children.push(child);
            cursor = next_cursor;
        } else {
            break;
        }
    }

    Err("Failed to parse XML".to_string())
}

/// Serialize XmlNode back to XML string
pub fn to_xml_string(node: &XmlNode, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    let mut xml = format!("{}<{}", indent_str, node.name);

    for (name, value) in &node.attributes {
        xml.push_str(&format!(" {}=\"{}\"", name, value));
    }

    if node.children.is_empty() && node.content.is_empty() {
        xml.push_str("/>\n");
    } else if node.children.is_empty() {
        xml.push_str(&format!(">{}</{}>\n", node.content, node.name));
    } else {
        xml.push_str(&format!(">\n"));
        if !node.content.trim().is_empty() {
            xml.push_str(&format!("{}{}{}\n", indent_str, "  ", node.content));
        }
        for child in &node.children {
            xml.push_str(&to_xml_string(child, indent + 1));
        }
        xml.push_str(&format!("{}</{}>\n", indent_str, node.name));
    }

    xml
}

/// Escape XML special characters
pub fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Unescape XML special characters
pub fn unescape_xml(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let xml = "<tag attr=\"value\">content</tag>";
        let mut tokenizer = XmlTokenizer::new(xml);

        assert_eq!(
            tokenizer.next_token(),
            Some(XmlToken::OpenTag("tag".to_string()))
        );
        assert_eq!(
            tokenizer.next_token(),
            Some(XmlToken::AttributeValue("value".to_string()))
        );
        assert_eq!(tokenizer.next_token(), Some(XmlToken::AngleClose));
        assert_eq!(
            tokenizer.next_token(),
            Some(XmlToken::Text("content".to_string()))
        );
        assert_eq!(
            tokenizer.next_token(),
            Some(XmlToken::CloseTag("tag".to_string()))
        );
    }

    #[test]
    fn test_parse_simple() {
        let xml = "<root><child>text</child></root>";
        let node = parse_xml(xml).unwrap();

        assert_eq!(node.name, "root");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].name, "child");
    }

    #[test]
    fn test_escape() {
        assert_eq!(escape_xml("<>&\"\""), "&lt;&gt;&amp;&quot;&quot;");
        assert_eq!(unescape_xml("&lt;&gt;&amp;&quot;&quot;"), "<>&\"\"");
    }
}
