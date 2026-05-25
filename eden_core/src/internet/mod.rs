//! # INTERNET - Global Information Network
//!
//! Crawling, parsing, indexación y knowledge ingestion.
//! Sin APIs externas - 100% Rust desde cero.
#![allow(dead_code)]
#![allow(non_snake_case)]

mod crawler;
mod indexer;
mod knowledge;
mod parser;

pub use crawler::Crawler;
pub use indexer::{IndexStats, Indexer, SearchResult};
pub use knowledge::{KnowledgeEntry, KnowledgeGraph};
pub use parser::{HtmlParser, LinkExtractor, TextParser};

/// Configuración de crawling
#[derive(Debug, Clone)]
pub struct CrawlConfig {
    pub max_depth: usize,
    pub max_urls: usize,
    pub user_agent: String,
    pub timeout_ms: u64,
    pub respect_robots_txt: bool,
    pub follow_external: bool,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_urls: 1000,
            user_agent: "Eden/1.0 (autonomous)".to_string(),
            timeout_ms: 30000,
            respect_robots_txt: true,
            follow_external: false,
        }
    }
}

/// Resultado de crawl
#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub url: String,
    pub success: bool,
    pub content_type: Option<String>,
    pub content: Option<Vec<u8>>,
    pub links: Vec<String>,
    pub title: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl CrawlResult {
    pub fn success(
        url: String,
        content: Vec<u8>,
        content_type: String,
        links: Vec<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            url,
            success: true,
            content_type: Some(content_type),
            content: Some(content),
            links,
            title: None,
            error: None,
            duration_ms,
        }
    }

    pub fn failure(url: String, error: String, duration_ms: u64) -> Self {
        Self {
            url,
            success: false,
            content_type: None,
            content: None,
            links: Vec::new(),
            title: None,
            error: Some(error),
            duration_ms,
        }
    }
}

/// URL normalizada
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub query: Option<String>,
}

impl Url {
    /// Parse URL desde string
    pub fn parse(s: &str) -> Result<Self, UrlError> {
        let s = s.trim();

        // scheme
        let (scheme, rest) = if let Some(pos) = s.find("://") {
            (s[..pos].to_string(), s[pos + 3..].to_string())
        } else {
            ("http".to_string(), s.to_string())
        };

        if scheme != "http" && scheme != "https" {
            return Err(UrlError::InvalidScheme);
        }

        let mut port = if scheme == "https" { 443 } else { 80 };
        let mut rest = rest.as_str();
        if let Some(hash_pos) = rest.find('#') {
            rest = &rest[..hash_pos];
        }

        let authority_end = rest.find(|c| c == '/' || c == '?').unwrap_or(rest.len());
        let authority = &rest[..authority_end];
        let path_query = &rest[authority_end..];

        // Parse host:port
        let mut host_port = authority.splitn(2, ':');
        let host = host_port.next().unwrap_or_default().to_string();
        if host.is_empty() {
            return Err(UrlError::ParseFailed);
        }
        if let Some(port_str) = host_port.next() {
            port = port_str.parse().map_err(|_| UrlError::InvalidPort)?;
        }

        let (path, query) = if let Some(query_start) = path_query.find('?') {
            let raw_path = &path_query[..query_start];
            let path = if raw_path.is_empty() { "/" } else { raw_path };
            let raw_query = &path_query[query_start + 1..];
            (path.to_string(), Some(raw_query.to_string()))
        } else if path_query.is_empty() {
            ("/".to_string(), None)
        } else {
            (path_query.to_string(), None)
        };

        Ok(Self {
            scheme,
            host,
            port,
            path,
            query,
        })
    }

    /// Resolve relative URL
    pub fn resolve(&self, relative: &str) -> Option<Url> {
        if relative.starts_with("http://") || relative.starts_with("https://") {
            Url::parse(relative).ok()
        } else if relative.starts_with('/') {
            Some(Url {
                scheme: self.scheme.clone(),
                host: self.host.clone(),
                port: self.port,
                path: relative.to_string(),
                query: self.query.clone(),
            })
        } else {
            // Relative path
            let base_path = self.path.rsplit_once('/').map(|(p, _)| p).unwrap_or("/");
            let new_path = format!("{}/{}", base_path, relative);
            Some(Url {
                scheme: self.scheme.clone(),
                host: self.host.clone(),
                port: self.port,
                path: new_path,
                query: self.query.clone(),
            })
        }
    }

    pub fn to_string(&self) -> String {
        let mut url = format!("{}://{}", self.scheme, self.host);
        if (self.scheme == "https" && self.port != 443)
            || (self.scheme == "http" && self.port != 80)
        {
            url.push_str(&format!(":{}", self.port));
        }
        url.push_str(&self.path);
        if let Some(q) = &self.query {
            url.push_str(&format!("?{}", q));
        }
        url
    }
}

#[derive(Debug)]
pub enum UrlError {
    InvalidScheme,
    InvalidPort,
    ParseFailed,
}

impl std::fmt::Display for UrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlError::InvalidScheme => write!(f, "Invalid URL scheme"),
            UrlError::InvalidPort => write!(f, "Invalid port"),
            UrlError::ParseFailed => write!(f, "URL parse failed"),
        }
    }
}

impl std::error::Error for UrlError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parse() {
        let url = Url::parse("https://example.com:8080/path?query=value").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 8080);
        assert_eq!(url.path, "/path");
    }

    #[test]
    fn test_url_resolve() {
        let base = Url::parse("https://example.com/path/page.html").unwrap();
        let resolved = base.resolve("/other").unwrap();
        assert_eq!(resolved.path, "/other");
    }
}
