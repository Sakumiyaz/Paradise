//! # Parser - HTML and text parsing
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;

/// Parser de HTML básico
pub struct HtmlParser {
    tags_stack: Vec<String>,
}

impl HtmlParser {
    pub fn new() -> Self {
        Self {
            tags_stack: Vec::new(),
        }
    }

    /// Parse HTML y extrae texto y metadatos
    pub fn parse(&mut self, html: &str) -> ParseResult {
        let title = None;
        let headings = Vec::new();
        let mut text_parts = Vec::new();
        let mut links = Vec::new();
        let mut in_title = false;
        let mut in_script = false;
        let mut in_style = false;
        let mut current_tag = String::new();
        let _current_attr = String::new();
        let mut in_tag = false;

        let mut i = 0;
        let bytes = html.as_bytes();

        while i < bytes.len() {
            let c = bytes[i] as char;

            if c == '<' && !in_tag {
                in_tag = true;
                current_tag.clear();
                continue;
            }

            if c == '>' && in_tag {
                in_tag = false;

                let tag_lower = current_tag.to_lowercase();

                if tag_lower.starts_with("script") {
                    in_script = true;
                } else if tag_lower.starts_with("/script") {
                    in_script = false;
                } else if tag_lower.starts_with("style") {
                    in_style = true;
                } else if tag_lower.starts_with("/style") {
                    in_style = false;
                } else if tag_lower == "title" && !in_script && !in_style {
                    in_title = true;
                } else if tag_lower.starts_with("/title") && !in_script && !in_style {
                    in_title = false;
                } else if tag_lower.starts_with("h1")
                    || tag_lower.starts_with("h2")
                    || tag_lower.starts_with("h3")
                {
                    // Capture heading
                } else if tag_lower.starts_with("a ") || tag_lower.starts_with("a>") {
                    // Extract href
                    if let Some(href) = self.extract_attr(&current_tag, "href") {
                        links.push(href);
                    }
                }

                current_tag.clear();
                continue;
            }

            if in_tag {
                current_tag.push(c);
            } else if !in_script && !in_style && !in_title {
                text_parts.push(c);
            }

            i += 1;
        }

        // Join text
        let text = self.clean_text(&text_parts.iter().collect::<String>());

        ParseResult {
            title,
            headings,
            text,
            links,
            metadata: HashMap::new(),
        }
    }

    fn extract_attr(&self, tag: &str, attr_name: &str) -> Option<String> {
        let tag_lower = tag.to_lowercase();
        if let Some(pos) = tag_lower.find(attr_name) {
            let after_attr = &tag[pos..];
            if let Some(eq_pos) = after_attr.find('=') {
                let value_start =
                    after_attr[eq_pos + 1..].find(|c| c == '"' || c == '\'')? + eq_pos + 2;
                let value_end =
                    after_attr[value_start..].find(|c| c == '"' || c == '\'')? + value_start;
                return Some(after_attr[value_start..value_end].to_string());
            }
        }
        None
    }

    fn clean_text(&self, raw: &str) -> String {
        let mut result = String::new();
        let mut last_was_space = true;

        for c in raw.chars() {
            if c.is_whitespace() {
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            } else if c.is_alphanumeric()
                || c == '.'
                || c == ','
                || c == '!'
                || c == '?'
                || c == '\''
            {
                result.push(c);
                last_was_space = false;
            }
        }

        result.trim().to_string()
    }

    /// Extrae solo texto (strip all HTML)
    pub fn extract_text(&self, html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;

        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(c),
                _ => {}
            }
        }

        self.clean_text(&result)
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Resultado del parsing
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub title: Option<String>,
    pub headings: Vec<String>,
    pub text: String,
    pub links: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Text parser para contenido sin HTML
pub struct TextParser;

impl TextParser {
    pub fn new() -> Self {
        Self
    }

    /// Tokeniza texto en palabras
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| s.len() > 2)
            .collect()
    }

    /// Calcula frecuencia de palabras
    pub fn word_frequency(&self, text: &str) -> HashMap<String, usize> {
        let mut freq = HashMap::new();
        for word in self.tokenize(text) {
            *freq.entry(word).or_insert(0) += 1;
        }
        freq
    }

    /// Extrae frases clave (n-grams)
    pub fn extract_phrases(&self, text: &str, n: usize) -> Vec<(String, usize)> {
        let tokens = self.tokenize(text);
        let mut phrases = HashMap::new();

        for window in tokens.windows(n) {
            let phrase = window.join(" ");
            *phrases.entry(phrase).or_insert(0) += 1;
        }

        let mut vec: Vec<_> = phrases.into_iter().collect();
        vec.sort_by(|a, b| b.1.cmp(&a.1));
        vec
    }
}

impl Default for TextParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Extractor de links
pub struct LinkExtractor;

impl LinkExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extrae todos los links de HTML
    pub fn extract(&self, html: &str) -> Vec<LinkInfo> {
        let mut links = Vec::new();
        let mut in_tag = false;
        let mut current_tag = String::new();

        for c in html.chars() {
            match c {
                '<' => {
                    in_tag = true;
                    current_tag.clear();
                }
                '>' => {
                    in_tag = false;
                    if current_tag.to_lowercase().starts_with("a ")
                        || current_tag.to_lowercase().starts_with("a>")
                    {
                        if let Some(href) = Self::extract_href(&current_tag) {
                            let text = Self::extract_text_between_tags(html, &current_tag);
                            links.push(LinkInfo {
                                url: href,
                                text,
                                nofollow: current_tag.to_lowercase().contains("nofollow"),
                            });
                        }
                    }
                }
                _ if in_tag => current_tag.push(c),
                _ => {}
            }
        }

        links
    }

    fn extract_href(tag: &str) -> Option<String> {
        let tag_lower = tag.to_lowercase();
        if let Some(pos) = tag_lower.find("href=") {
            let after_href = &tag[pos + 5..];
            let quote = after_href.chars().next()?;
            if quote == '"' || quote == '\'' {
                let rest = &after_href[1..];
                if let Some(end) = rest.find(quote) {
                    return Some(rest[..end].to_string());
                }
            }
        }
        None
    }

    fn extract_text_between_tags(html: &str, _tag: &str) -> String {
        // Simple extraction - find text after tag
        if let Some(pos) = html.find('>') {
            let after = &html[pos + 1..];
            let end = after.find('<').unwrap_or(after.len());
            after[..end].trim().to_string()
        } else {
            String::new()
        }
    }
}

impl Default for LinkExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Info de un link
#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub url: String,
    pub text: String,
    pub nofollow: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_parser() {
        let html = "<html><head><title>Test</title></head><body><p>Hello World</p></body></html>";
        let mut parser = HtmlParser::new();
        let result = parser.parse(html);
        assert!(result.text.contains("Hello"));
    }

    #[test]
    fn test_text_parser() {
        let tp = TextParser::new();
        let freq = tp.word_frequency("hello world hello");
        assert_eq!(*freq.get("hello").unwrap(), 2);
    }
}
