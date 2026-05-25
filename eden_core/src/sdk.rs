#![allow(dead_code)]

use serde_json::Value;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct EdenClient {
    base_url: String,
    host: String,
    port: u16,
    timeout: Duration,
    api_token: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EdenHttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub content_type: String,
    pub body: String,
}

#[derive(Debug)]
pub enum EdenClientError {
    InvalidBaseUrl(String),
    Io(String),
    InvalidResponse(String),
    Json(String),
    Http {
        status_code: u16,
        status_text: String,
        body: String,
    },
}

impl fmt::Display for EdenClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdenClientError::InvalidBaseUrl(message) => write!(f, "invalid base URL: {message}"),
            EdenClientError::Io(message) => write!(f, "I/O error: {message}"),
            EdenClientError::InvalidResponse(message) => {
                write!(f, "invalid HTTP response: {message}")
            }
            EdenClientError::Json(message) => write!(f, "JSON error: {message}"),
            EdenClientError::Http {
                status_code,
                status_text,
                body,
            } => write!(f, "HTTP {status_code} {status_text}: {body}"),
        }
    }
}

impl std::error::Error for EdenClientError {}

impl EdenClient {
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, EdenClientError> {
        let parsed = ParsedBaseUrl::parse(base_url.as_ref())?;
        Ok(Self {
            base_url: parsed.base_url,
            host: parsed.host,
            port: parsed.port,
            timeout: Duration::from_secs(5),
            api_token: std::env::var("EDEN_API_TOKEN")
                .ok()
                .filter(|token| !token.trim().is_empty()),
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_api_token(mut self, token: impl Into<String>) -> Self {
        let token = token.into();
        self.api_token = if token.trim().is_empty() {
            None
        } else {
            Some(token)
        };
        self
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn get(&self, path: &str) -> Result<EdenHttpResponse, EdenClientError> {
        let path = normalize_path(path);
        let mut stream = TcpStream::connect((self.host.as_str(), self.port))
            .map_err(|e| EdenClientError::Io(e.to_string()))?;
        let _ = stream.set_read_timeout(Some(self.timeout));
        let _ = stream.set_write_timeout(Some(self.timeout));
        let auth_header = self
            .api_token
            .as_ref()
            .map(|token| format!("Authorization: Bearer {token}\r\n"))
            .unwrap_or_default();
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}:{}\r\n{}Connection: close\r\n\r\n",
            path, self.host, self.port, auth_header
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|e| EdenClientError::Io(e.to_string()))?;

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .map_err(|e| EdenClientError::Io(e.to_string()))?;
        parse_http_response(&response)
    }

    pub fn get_text(&self, path: &str) -> Result<String, EdenClientError> {
        let response = self.get(path)?;
        if response.is_success() {
            Ok(response.body)
        } else {
            Err(response.into_http_error())
        }
    }

    pub fn get_json(&self, path: &str) -> Result<Value, EdenClientError> {
        let response = self.get(path)?;
        if !response.is_success() {
            return Err(response.into_http_error());
        }
        serde_json::from_str(&response.body).map_err(|e| EdenClientError::Json(e.to_string()))
    }

    pub fn health(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/health")
    }

    pub fn ready(&self) -> Result<String, EdenClientError> {
        self.get_text("/ready")
    }

    pub fn runtime_catalog(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/runtime/catalog")
    }

    pub fn runtime_openapi(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/runtime/openapi")
    }

    pub fn runtime_snapshot(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/runtime/snapshot")
    }

    pub fn runtime_state(&self, name: &str) -> Result<EdenHttpResponse, EdenClientError> {
        self.get(&format!("/api/runtime/state?name={}", encode_query(name)))
    }

    pub fn artifact_catalog(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/artifact/catalog")
    }

    pub fn artifact_runtime(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/artifact/runtime")
    }

    pub fn artifact(&self, name: &str) -> Result<EdenHttpResponse, EdenClientError> {
        self.get(&format!("/api/artifact?name={}", encode_query(name)))
    }

    pub fn operational_catalog(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/operational/catalog")
    }

    pub fn operational_openapi(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/operational/openapi")
    }

    pub fn operational_runtime(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/operational/runtime")
    }

    pub fn operational_status(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/operational/status")
    }

    pub fn operational_contract(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/operational/contract")
    }

    pub fn operational_permissions(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/operational/permissions")
    }

    pub fn operational_replay(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/operational/replay")
    }

    pub fn operational_replay_decision(&self, decision_id: &str) -> Result<Value, EdenClientError> {
        self.get_json(&format!(
            "/api/operational/replay?decision_id={}",
            encode_query(decision_id)
        ))
    }

    pub fn operational_recovery(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/operational/recovery")
    }

    pub fn operational_demos(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/operational/demos")
    }

    pub fn operational_schemas(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/operational/schemas")
    }

    pub fn operational_schema(&self, name: &str) -> Result<Value, EdenClientError> {
        self.get_json(&format!(
            "/api/operational/schema?name={}",
            encode_query(name)
        ))
    }

    pub fn capabilities_catalog(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/capabilities/catalog")
    }

    pub fn capabilities_status(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/capabilities/status")
    }

    pub fn gewc_runtime(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/gewc/runtime")
    }

    pub fn gewc_handlers(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/gewc/handlers")
    }

    pub fn validation_status(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/validation/status")
    }

    pub fn action_contracts(&self) -> Result<Value, EdenClientError> {
        self.get_json("/api/actions/contracts")
    }

    pub fn action_dry_run(&self, command: &str) -> Result<Value, EdenClientError> {
        self.get_json(&format!(
            "/api/actions/dry-run?cmd={}",
            encode_query(command)
        ))
    }

    pub fn locus_eval(&self) -> Result<String, EdenClientError> {
        self.run_command_sync("locus eval")
    }

    pub fn locus_ingest(&self, spec: &str) -> Result<String, EdenClientError> {
        self.run_command_sync(&format!("locus ingest {spec}"))
    }

    pub fn locus_context(&self, query: &str) -> Result<String, EdenClientError> {
        self.run_command_sync(&format!("locus context {query}"))
    }

    pub fn locus_audit(&self) -> Result<String, EdenClientError> {
        self.run_command_sync("locus audit")
    }

    pub fn locus_state(&self) -> Result<EdenHttpResponse, EdenClientError> {
        self.runtime_state("eden_locus_layer")
    }

    pub fn operator_forge_eval(&self) -> Result<String, EdenClientError> {
        self.run_command_sync("operator forge eval")
    }

    pub fn operator_forge_synth(&self, goal: &str) -> Result<String, EdenClientError> {
        self.run_command_sync(&format!("operator forge synth {goal}"))
    }

    pub fn operator_forge_verify(&self) -> Result<String, EdenClientError> {
        self.run_command_sync("operator forge verify")
    }

    pub fn operator_forge_audit(&self) -> Result<String, EdenClientError> {
        self.run_command_sync("operator forge audit")
    }

    pub fn operator_forge_state(&self) -> Result<EdenHttpResponse, EdenClientError> {
        self.runtime_state("eden_operator_forge")
    }

    pub fn queue_command(&self, command: &str) -> Result<String, EdenClientError> {
        self.get_text(&format!("/api/command?cmd={}", encode_query(command)))
    }

    pub fn run_command_sync(&self, command: &str) -> Result<String, EdenClientError> {
        self.get_text(&format!("/api/command_sync?cmd={}", encode_query(command)))
    }

    pub fn command_result(&self, id: u64) -> Result<EdenHttpResponse, EdenClientError> {
        self.get(&format!("/api/command_result?id={id}"))
    }

    pub fn forget_command(&self, id: u64) -> Result<EdenHttpResponse, EdenClientError> {
        self.get(&format!("/api/command_forget?id={id}"))
    }
}

impl EdenHttpResponse {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    fn into_http_error(self) -> EdenClientError {
        EdenClientError::Http {
            status_code: self.status_code,
            status_text: self.status_text,
            body: self.body,
        }
    }
}

struct ParsedBaseUrl {
    base_url: String,
    host: String,
    port: u16,
}

impl ParsedBaseUrl {
    fn parse(raw: &str) -> Result<Self, EdenClientError> {
        let raw = raw.trim().trim_end_matches('/');
        let without_scheme = raw.strip_prefix("http://").ok_or_else(|| {
            EdenClientError::InvalidBaseUrl("only http:// endpoints are supported".to_string())
        })?;
        if without_scheme.is_empty() || without_scheme.contains('/') {
            return Err(EdenClientError::InvalidBaseUrl(
                "expected http://host:port without a path".to_string(),
            ));
        }
        let (host, port) = without_scheme.rsplit_once(':').ok_or_else(|| {
            EdenClientError::InvalidBaseUrl("expected explicit host:port".to_string())
        })?;
        if host.is_empty() {
            return Err(EdenClientError::InvalidBaseUrl(
                "host cannot be empty".to_string(),
            ));
        }
        let port = port
            .parse::<u16>()
            .map_err(|_| EdenClientError::InvalidBaseUrl("invalid port".to_string()))?;
        Ok(Self {
            base_url: format!("http://{host}:{port}"),
            host: host.to_string(),
            port,
        })
    }
}

fn parse_http_response(bytes: &[u8]) -> Result<EdenHttpResponse, EdenClientError> {
    let response = String::from_utf8_lossy(bytes);
    let (header, body) = response
        .split_once("\r\n\r\n")
        .or_else(|| response.split_once("\n\n"))
        .ok_or_else(|| EdenClientError::InvalidResponse("missing header/body split".to_string()))?;
    let mut lines = header.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| EdenClientError::InvalidResponse("missing status line".to_string()))?;
    let mut status_parts = status_line.splitn(3, ' ');
    let protocol = status_parts.next().unwrap_or("");
    if !protocol.starts_with("HTTP/") {
        return Err(EdenClientError::InvalidResponse(format!(
            "unexpected protocol in {status_line}"
        )));
    }
    let status_code = status_parts
        .next()
        .ok_or_else(|| EdenClientError::InvalidResponse("missing status code".to_string()))?
        .parse::<u16>()
        .map_err(|_| EdenClientError::InvalidResponse("invalid status code".to_string()))?;
    let status_text = status_parts.next().unwrap_or("").to_string();
    let mut content_type = String::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-type") {
                content_type = value.trim().to_string();
            }
        }
    }
    Ok(EdenHttpResponse {
        status_code,
        status_text,
        content_type,
        body: body.to_string(),
    })
}

fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

pub fn encode_query(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_local_http_base_url() {
        let client = EdenClient::new("http://127.0.0.1:8110/").unwrap();

        assert_eq!(client.base_url(), "http://127.0.0.1:8110");
    }

    #[test]
    fn rejects_non_http_base_url() {
        let err = EdenClient::new("https://127.0.0.1:8110").unwrap_err();

        assert!(err.to_string().contains("only http://"));
    }

    #[test]
    fn encodes_command_queries_for_garm_router() {
        assert_eq!(
            encode_query("runtime state api eval"),
            "runtime+state+api+eval"
        );
        assert_eq!(encode_query("../secret"), "..%2Fsecret");
    }

    #[test]
    fn configures_optional_api_token() {
        let client = EdenClient::new("http://127.0.0.1:8110")
            .unwrap()
            .with_api_token("local-token");

        assert_eq!(client.api_token.as_deref(), Some("local-token"));
    }

    #[test]
    fn parses_minimal_http_response() {
        let response = parse_http_response(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\n\r\n{\"ok\":true}",
        )
        .unwrap();

        assert_eq!(response.status_code, 200);
        assert!(response.content_type.contains("application/json"));
        assert_eq!(response.body, "{\"ok\":true}");
    }
}
