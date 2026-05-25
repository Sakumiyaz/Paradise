// EDEN GARM MCP Client — Absolute Ceiling Edition
// JSON-RPC 2.0 stdio client with timeouts, schema validation, notification handling, tool cache

use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub id: u64,
    pub method: String,
    pub sent_at: Instant,
}

#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub name: String,
    pub template: String,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct McpChunk {
    pub index: usize,
    pub total: Option<usize>,
    pub data: serde_json::Value,
}

pub struct McpClient {
    pub server_cmd: String,
    pub server_args: Vec<String>,
    pub next_id: u64,
    pub tools: Vec<McpTool>,
    pub child: Option<Child>,
    pub tool_cache: HashMap<String, McpTool>,
    pub request_timeout_ms: u64,
    pub connected: bool,
    pub last_error: Option<String>,
    // NEW: protocol version negotiation
    pub protocol_version: String,
    // NEW: request/response correlation
    pub pending_requests: HashMap<u64, PendingRequest>,
    // NEW: logging middleware
    pub traffic_log: Vec<String>,
    // NEW: reconnection state
    pub backoff_attempts: u32,
    // NEW: health checks
    pub health_last_ping: Option<Instant>,
    // NEW: resource subscriptions
    pub subscriptions: Vec<String>,
    // NEW: prompt template registry
    pub prompt_registry: HashMap<String, PromptTemplate>,
    // NEW: tool capability negotiation
    pub supported_tools: Vec<String>,
    // Internal: persistent stdout reader for parallel dispatch & streaming
    reader: Option<io::BufReader<std::process::ChildStdout>>,
}

impl McpClient {
    pub fn new(server_cmd: &str, args: &[&str]) -> Self {
        McpClient {
            server_cmd: server_cmd.into(),
            server_args: args.iter().map(|s| s.to_string()).collect(),
            next_id: 1,
            tools: Vec::new(),
            child: None,
            tool_cache: HashMap::new(),
            request_timeout_ms: 5000,
            connected: false,
            last_error: None,
            protocol_version: "2024-11-05".into(),
            pending_requests: HashMap::new(),
            traffic_log: Vec::new(),
            backoff_attempts: 0,
            health_last_ping: None,
            subscriptions: Vec::new(),
            prompt_registry: HashMap::new(),
            supported_tools: Vec::new(),
            reader: None,
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        let child = Command::new(&self.server_cmd)
            .args(&self.server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server: {}", e))?;
        self.child = Some(child);
        self.connected = true;
        // Send initialize
        let init = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(0),
            method: "initialize".into(),
            params: Some(serde_json::json!({ "protocolVersion": "2024-11-05" })),
        };
        let _ = self.send_request_raw(&init)?;
        Ok(())
    }

    pub fn list_tools(&mut self) -> Result<Vec<McpTool>, String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(self.next_id),
            method: "tools/list".into(),
            params: None,
        };
        self.next_id += 1;
        let resp = self.send_request_raw(&req)?;
        if let Some(result) = resp.result {
            let tools: Vec<McpTool> = serde_json::from_value(
                result
                    .get("tools")
                    .cloned()
                    .unwrap_or(serde_json::json!([])),
            )
            .map_err(|e| format!("Parse tools error: {}", e))?;
            self.tools = tools.clone();
            self.tool_cache.clear();
            for t in &tools {
                self.tool_cache.insert(t.name.clone(), t.clone());
            }
            Ok(tools)
        } else {
            Err("No result from tools/list".into())
        }
    }

    pub fn call_tool(
        &mut self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        // Schema validation if cached
        if let Some(tool) = self.tool_cache.get(name) {
            if let Some(schema) = tool
                .input_schema
                .get("properties")
                .and_then(|p| p.as_object())
            {
                if let Some(obj) = args.as_object() {
                    for (key, _) in obj {
                        if !schema.contains_key(key) {
                            return Err(format!("Unknown param '{}' for tool '{}'", key, name));
                        }
                    }
                }
            }
        }
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(self.next_id),
            method: "tools/call".into(),
            params: Some(serde_json::json!({ "name": name, "arguments": args })),
        };
        self.next_id += 1;
        let resp = self.send_request_raw(&req)?;
        if let Some(error) = resp.error {
            self.last_error = Some(format!("{}: {}", error.code, error.message));
            return Err(self.last_error.clone().unwrap());
        }
        if let Some(result) = resp.result {
            Ok(result)
        } else {
            Err("No result from tool call".into())
        }
    }

    fn send_request_raw(&mut self, req: &JsonRpcRequest) -> Result<JsonRpcResponse, String> {
        let req_str = serde_json::to_string(req).map_err(|e| format!("Serialize: {}", e))?;
        self.log_traffic(">>", &req_str);
        {
            let child = self.child.as_mut().ok_or("Not connected")?;
            let stdin = child.stdin.as_mut().ok_or("No stdin")?;
            writeln!(stdin, "{}", req_str).map_err(|e| format!("Write: {}", e))?;
            stdin.flush().map_err(|e| format!("Flush: {}", e))?;
        }
        let id = req.id;
        if let Some(id) = id {
            self.pending_requests.insert(
                id,
                PendingRequest {
                    id,
                    method: req.method.clone(),
                    sent_at: Instant::now(),
                },
            );
        }
        let start = Instant::now();
        let mut line = String::new();
        loop {
            let res = {
                let reader = self.ensure_reader()?;
                reader.read_line(&mut line)
            };
            match res {
                Ok(0) => {
                    if let Some(id) = id {
                        self.pending_requests.remove(&id);
                    }
                    return Err("EOF from MCP server".into());
                }
                Ok(_) => break,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if start.elapsed().as_millis() as u64 > self.request_timeout_ms {
                        if let Some(id) = id {
                            self.pending_requests.remove(&id);
                        }
                        return Err("MCP request timeout".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    if let Some(id) = id {
                        self.pending_requests.remove(&id);
                    }
                    return Err(format!("Read: {}", e));
                }
            }
        }
        self.log_traffic("<<", &line);
        let resp: JsonRpcResponse =
            serde_json::from_str(&line).map_err(|e| format!("Parse response: {}", e))?;
        if let Some(id) = resp.id {
            self.pending_requests.remove(&id);
        }
        Ok(resp)
    }

    pub fn status(&self) -> String {
        format!(
            "MCP client | tools: {} | cached: {} | connected: {} | timeout: {}ms",
            self.tools.len(),
            self.tool_cache.len(),
            self.connected,
            self.request_timeout_ms
        )
    }

    // =====================================================================
    // 1. MCP Server discovery — scan for Initialize messages, validate protocol version
    // =====================================================================
    pub fn scan_for_initialize(&mut self) -> Result<String, String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        let start = Instant::now();
        loop {
            let mut line = String::new();
            let read_res = {
                let reader = self.ensure_reader()?;
                reader.read_line(&mut line)
            };
            match read_res {
                Ok(0) => return Err("EOF while scanning for initialize".into()),
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if start.elapsed().as_millis() as u64 > self.request_timeout_ms {
                        return Err("Timeout scanning for initialize".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => return Err(format!("Read error: {}", e)),
            }
            if line.trim().is_empty() {
                continue;
            }
            self.log_traffic("<<", &line);
            let resp: JsonRpcResponse = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(_) => continue,
            };
            if let Some(result) = resp.result {
                if let Some(proto) = result.get("protocolVersion").and_then(|v| v.as_str()) {
                    if proto == "2024-11-05" || proto == "2025-03-26" {
                        self.protocol_version = proto.into();
                        return Ok(proto.into());
                    } else {
                        return Err(format!("Unsupported protocol version: {}", proto));
                    }
                }
            }
        }
    }

    // =====================================================================
    // 2. Tool capability negotiation — declare which tools client supports
    // =====================================================================
    pub fn negotiate_tool_capabilities(
        &mut self,
        capabilities: Vec<String>,
    ) -> Result<serde_json::Value, String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        self.supported_tools = capabilities.clone();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(self.next_id),
            method: "tools/negotiate".into(),
            params: Some(serde_json::json!({ "supportedTools": capabilities })),
        };
        self.next_id += 1;
        let resp = self.send_request_raw(&req)?;
        if let Some(error) = resp.error {
            return Err(format!(
                "Capability negotiation error {}: {}",
                error.code, error.message
            ));
        }
        Ok(resp.result.unwrap_or(serde_json::Value::Null))
    }

    // =====================================================================
    // 4. Reconnection logic with exponential backoff stub
    // =====================================================================
    pub fn reconnect_with_backoff(&mut self) -> Result<(), String> {
        self.connected = false;
        self.child = None;
        self.reader = None;
        for attempt in 0..=10 {
            match self.attempt_reconnect() {
                Ok(()) => {
                    self.backoff_attempts = 0;
                    return Ok(());
                }
                Err(e) => {
                    self.backoff_attempts = attempt + 1;
                    let delay = (100u64 * (1u64 << self.backoff_attempts.min(10))).min(10000);
                    std::thread::sleep(Duration::from_millis(delay));
                    if attempt == 10 {
                        return Err(format!("Reconnection failed after backoff: {}", e));
                    }
                }
            }
        }
        Err("Reconnection failed".into())
    }

    fn attempt_reconnect(&mut self) -> Result<(), String> {
        self.child = None;
        self.reader = None;
        self.connected = false;
        self.connect()
    }

    // =====================================================================
    // 5. Health checks (ping/heartbeat)
    // =====================================================================
    pub fn ping(&mut self) -> Result<Duration, String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(self.next_id),
            method: "ping".into(),
            params: None,
        };
        self.next_id += 1;
        let start = Instant::now();
        let _ = self.send_request_raw(&req)?;
        let elapsed = start.elapsed();
        self.health_last_ping = Some(Instant::now());
        Ok(elapsed)
    }

    // =====================================================================
    // 6. Parallel tool execution dispatcher
    // =====================================================================
    pub fn dispatch_tools_parallel(
        &mut self,
        calls: Vec<(&str, serde_json::Value)>,
    ) -> Result<Vec<serde_json::Value>, String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        let mut ids = Vec::with_capacity(calls.len());
        for (name, args) in calls {
            let id = self.next_id;
            self.next_id += 1;
            if let Some(tool) = self.tool_cache.get(name) {
                if let Some(schema) = tool
                    .input_schema
                    .get("properties")
                    .and_then(|p| p.as_object())
                {
                    if let Some(obj) = args.as_object() {
                        for (key, _) in obj {
                            if !schema.contains_key(key) {
                                return Err(format!("Unknown param '{}' for tool '{}'", key, name));
                            }
                        }
                    }
                }
            }
            let req = JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: Some(id),
                method: "tools/call".into(),
                params: Some(serde_json::json!({ "name": name, "arguments": args })),
            };
            self.pending_requests.insert(
                id,
                PendingRequest {
                    id,
                    method: req.method.clone(),
                    sent_at: Instant::now(),
                },
            );
            let req_str = serde_json::to_string(&req).map_err(|e| format!("Serialize: {}", e))?;
            self.log_traffic(">>", &req_str);
            {
                let child = self.child.as_mut().ok_or("Not connected")?;
                let stdin = child.stdin.as_mut().ok_or("No stdin")?;
                writeln!(stdin, "{}", req_str).map_err(|e| format!("Write: {}", e))?;
            }
            ids.push(id);
        }
        {
            let child = self.child.as_mut().ok_or("Not connected")?;
            let stdin = child.stdin.as_mut().ok_or("No stdin")?;
            stdin.flush().map_err(|e| format!("Flush: {}", e))?;
        }
        let start = Instant::now();
        let mut results = Vec::with_capacity(ids.len());
        let mut collected = 0usize;
        while collected < ids.len() {
            let mut line = String::new();
            let read_res = {
                let reader = self.ensure_reader()?;
                reader.read_line(&mut line)
            };
            match read_res {
                Ok(0) => return Err("EOF from MCP server during parallel dispatch".into()),
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    let timeout = self
                        .request_timeout_ms
                        .saturating_mul(ids.len() as u64)
                        .max(self.request_timeout_ms);
                    if start.elapsed().as_millis() as u64 > timeout {
                        return Err("Parallel dispatch timeout".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => return Err(format!("Read: {}", e)),
            }
            self.log_traffic("<<", &line);
            let resp: JsonRpcResponse =
                serde_json::from_str(&line).map_err(|e| format!("Parse response: {}", e))?;
            if let Some(id) = resp.id {
                self.pending_requests.remove(&id);
            }
            if let Some(error) = resp.error {
                return Err(format!("Tool call error {}: {}", error.code, error.message));
            }
            results.push(resp.result.unwrap_or(serde_json::Value::Null));
            collected += 1;
        }
        Ok(results)
    }

    // =====================================================================
    // 7. Tool result streaming (chunked response handler)
    // =====================================================================
    pub fn call_tool_streaming<F>(
        &mut self,
        name: &str,
        args: serde_json::Value,
        mut on_chunk: F,
    ) -> Result<serde_json::Value, String>
    where
        F: FnMut(&serde_json::Value),
    {
        if !self.connected {
            return Err("Not connected".into());
        }
        let id = self.next_id;
        self.next_id += 1;
        if let Some(tool) = self.tool_cache.get(name) {
            if let Some(schema) = tool
                .input_schema
                .get("properties")
                .and_then(|p| p.as_object())
            {
                if let Some(obj) = args.as_object() {
                    for (key, _) in obj {
                        if !schema.contains_key(key) {
                            return Err(format!("Unknown param '{}' for tool '{}'", key, name));
                        }
                    }
                }
            }
        }
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(id),
            method: "tools/call".into(),
            params: Some(serde_json::json!({ "name": name, "arguments": args })),
        };
        let req_str = serde_json::to_string(&req).map_err(|e| format!("Serialize: {}", e))?;
        self.log_traffic(">>", &req_str);
        {
            let child = self.child.as_mut().ok_or("Not connected")?;
            let stdin = child.stdin.as_mut().ok_or("No stdin")?;
            writeln!(stdin, "{}", req_str).map_err(|e| format!("Write: {}", e))?;
            stdin.flush().map_err(|e| format!("Flush: {}", e))?;
        }
        self.pending_requests.insert(
            id,
            PendingRequest {
                id,
                method: req.method.clone(),
                sent_at: Instant::now(),
            },
        );
        let start = Instant::now();
        let mut final_result = None;
        loop {
            let mut line = String::new();
            let read_res = {
                let reader = self.ensure_reader()?;
                reader.read_line(&mut line)
            };
            match read_res {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if start.elapsed().as_millis() as u64 > self.request_timeout_ms {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(_) => break,
            }
            if line.trim().is_empty() {
                continue;
            }
            self.log_traffic("<<", &line);
            let resp: JsonRpcResponse = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(_) => continue,
            };
            if resp.id != Some(id) {
                continue;
            }
            self.pending_requests.remove(&id);
            if let Some(result) = resp.result.clone() {
                if result
                    .get("done")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    final_result = Some(result.get("data").cloned().unwrap_or(result));
                    break;
                } else {
                    on_chunk(&result);
                }
            } else if let Some(error) = resp.error {
                return Err(format!("Streaming error {}: {}", error.code, error.message));
            }
        }
        final_result.ok_or("No final result from stream".into())
    }

    // =====================================================================
    // 8. Error recovery (retry on transient errors)
    // =====================================================================
    pub fn send_request_with_retry(
        &mut self,
        req: &JsonRpcRequest,
        max_retries: u32,
    ) -> Result<JsonRpcResponse, String> {
        let mut last_err = String::new();
        for attempt in 0..=max_retries {
            match self.send_request_raw(req) {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    last_err = e.clone();
                    if Self::is_transient_error(&e) {
                        let delay_ms = (100u64 * (1u64 << attempt.min(10))).min(1600);
                        std::thread::sleep(Duration::from_millis(delay_ms));
                        if let Err(reconn_err) = self.attempt_reconnect() {
                            return Err(format!("Reconnect failed during retry: {}", reconn_err));
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Err(format!(
            "Retry exhausted after {} attempts: {}",
            max_retries, last_err
        ))
    }

    fn is_transient_error(err: &str) -> bool {
        err.contains("timeout")
            || err.contains("EOF")
            || err.contains("Broken pipe")
            || err.contains("WouldBlock")
    }

    // =====================================================================
    // 9. Protocol version negotiation (2024-11-05 vs 2025-03-26)
    // =====================================================================
    pub fn negotiate_protocol(&mut self, preferred: &str) -> Result<String, String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(self.next_id),
            method: "initialize".into(),
            params: Some(serde_json::json!({ "protocolVersion": preferred })),
        };
        self.next_id += 1;
        let resp = self.send_request_raw(&req)?;
        let chosen = if let Some(result) = resp.result {
            let server_version = result
                .get("protocolVersion")
                .and_then(|v| v.as_str())
                .unwrap_or(preferred);
            if server_version != "2024-11-05" && server_version != "2025-03-26" {
                return Err(format!("Unsupported protocol version: {}", server_version));
            }
            server_version.to_string()
        } else if let Some(error) = resp.error {
            return Err(format!(
                "Protocol negotiation error {}: {}",
                error.code, error.message
            ));
        } else {
            preferred.to_string()
        };
        self.protocol_version = chosen.clone();
        Ok(chosen)
    }

    // =====================================================================
    // 10. Resource subscription / notification handler stub
    // =====================================================================
    pub fn subscribe_resource(&mut self, uri: &str) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(self.next_id),
            method: "resources/subscribe".into(),
            params: Some(serde_json::json!({ "uri": uri })),
        };
        self.next_id += 1;
        let _ = self.send_request_raw(&req)?;
        self.subscriptions.push(uri.to_string());
        Ok(())
    }

    pub fn unsubscribe_resource(&mut self, uri: &str) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(self.next_id),
            method: "resources/unsubscribe".into(),
            params: Some(serde_json::json!({ "uri": uri })),
        };
        self.next_id += 1;
        let _ = self.send_request_raw(&req)?;
        self.subscriptions.retain(|s| s != uri);
        Ok(())
    }

    pub fn handle_notification(&mut self, notification: &JsonRpcRequest) -> Result<(), String> {
        if notification.id.is_some() {
            return Err("Expected notification (no id)".into());
        }
        match notification.method.as_str() {
            "notifications/resources/updated" => {
                // stub: resource update notification
            }
            "notifications/tools/list_changed" => {
                // stub: tool list changed notification
            }
            _ => {}
        }
        Ok(())
    }

    // =====================================================================
    // 11. Prompt template registry
    // =====================================================================
    pub fn register_prompt(&mut self, name: &str, template: &str, variables: Vec<String>) {
        self.prompt_registry.insert(
            name.to_string(),
            PromptTemplate {
                name: name.to_string(),
                template: template.to_string(),
                variables,
            },
        );
    }

    pub fn render_prompt(
        &self,
        name: &str,
        values: &HashMap<String, String>,
    ) -> Result<String, String> {
        let tmpl = self
            .prompt_registry
            .get(name)
            .ok_or_else(|| format!("Prompt template '{}' not found", name))?;
        let mut rendered = tmpl.template.clone();
        for var in &tmpl.variables {
            let val = values
                .get(var)
                .ok_or_else(|| format!("Missing variable '{}' for prompt '{}'", var, name))?;
            rendered = rendered.replace(&format!("{{{}}}", var), val);
        }
        Ok(rendered)
    }

    // =====================================================================
    // 12. Logging middleware for all JSON-RPC traffic
    // =====================================================================
    pub fn log_traffic(&mut self, direction: &str, payload: &str) {
        let entry = format!("[{}] {}", direction, payload.trim());
        self.traffic_log.push(entry);
    }

    pub fn get_traffic_log(&self) -> &[String] {
        &self.traffic_log
    }

    // Internal helper: ensure persistent stdout reader exists
    fn ensure_reader(&mut self) -> Result<&mut io::BufReader<std::process::ChildStdout>, String> {
        if self.reader.is_none() {
            let child = self.child.as_mut().ok_or("Not connected")?;
            let stdout = child.stdout.take().ok_or("No stdout")?;
            self.reader = Some(io::BufReader::new(stdout));
        }
        self.reader.as_mut().ok_or("No reader".into())
    }
}
