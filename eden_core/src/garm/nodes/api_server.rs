use crate::eden_garm::capabilities::GarmCapabilityState;
use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use crate::eden_garm::nodes::organ_registry;
use crate::eden_garm::{
    artifact_api, operational_api, operational_runtime, runtime_state_api, state_paths,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct QueuedCommand {
    pub id: Option<u64>,
    pub raw: String,
}

pub struct CommandResponseBus {
    next_id: AtomicU64,
    issued: Mutex<HashSet<u64>>,
    responses: Mutex<HashMap<u64, String>>,
    completed_order: Mutex<VecDeque<u64>>,
}

const MAX_STORED_COMMAND_RESULTS: usize = 128;
const OPERATOR_CONSOLE_HTML: &str = include_str!("../../../../docs/EDEN_OPERATOR_CONSOLE.html");
const API_HELP_TEXT: &str = concat!(
    "EDEN GARM API: /ready /state /status /metrics /report /export /artifacts ",
    "/api/artifact/catalog /api/artifact?name=... /api/artifact/runtime ",
    "/api/runtime/catalog /api/runtime/state?name=... /api/runtime/snapshot ",
    "/api/runtime/openapi /api/operational/runtime /api/operational/status ",
    "/api/operational/contract /api/operational/permissions /api/operational/replay ",
    "/api/operational/recovery /api/operational/demos /api/operational/schemas ",
    "/api/operational/schema?name=... /api/operational/runtime-phase ",
    "/api/operational/spine /api/operational/events /api/operational/global-state ",
    "/api/operational/replay-spine /api/operational/spine-verification ",
    "/api/operational/spine-enforcement /api/operational/workflow-risk ",
    "/api/operational/circuit-breakers /api/operational/spine-replay ",
    "/api/capabilities/catalog /api/capabilities/status /api/gewc/runtime ",
    "/api/gewc/handlers /api/validation/status /api/actions/contracts ",
    "/api/actions/dry-run?cmd=... /organs /organs/actions /organs/audit ",
    "/organs/recovery /command?cmd=... /api/command?cmd=... ",
    "/command_result?id=... /api/command_result?id=... /command_forget?id=... ",
    "/api/command_forget?id=... /command_sync?cmd=... /api/command_sync?cmd=...\n",
    "Optional local auth: set EDEN_API_TOKEN and send Authorization: Bearer <token> ",
    "or X-EDEN-API-Token: <token>.\n"
);

impl CommandResponseBus {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            issued: Mutex::new(HashSet::new()),
            responses: Mutex::new(HashMap::new()),
            completed_order: Mutex::new(VecDeque::new()),
        }
    }

    pub fn next_id(&self) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut issued) = self.issued.lock() {
            issued.insert(id);
        }
        id
    }

    pub fn respond(&self, id: u64, response: String) {
        if let Ok(mut responses) = self.responses.lock() {
            responses.insert(id, response);
        }
        if let Ok(mut issued) = self.issued.lock() {
            issued.remove(&id);
        }
        self.remember_completed(id);
    }

    pub fn wait_for(&self, id: u64, timeout: Duration) -> Option<String> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if let Ok(responses) = self.responses.lock() {
                if let Some(response) = responses.get(&id) {
                    return Some(response.clone());
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        None
    }

    pub fn take_result(&self, id: u64) -> CommandResultState {
        if let Ok(responses) = self.responses.lock() {
            if let Some(response) = responses.get(&id) {
                return CommandResultState::Done(response.clone());
            }
        }
        if let Ok(issued) = self.issued.lock() {
            if issued.contains(&id) {
                return CommandResultState::Pending;
            }
        }
        CommandResultState::Unknown
    }

    pub fn forget(&self, id: u64) -> bool {
        let mut existed = false;
        if let Ok(mut responses) = self.responses.lock() {
            existed |= responses.remove(&id).is_some();
        }
        if let Ok(mut issued) = self.issued.lock() {
            existed |= issued.remove(&id);
        }
        if let Ok(mut completed_order) = self.completed_order.lock() {
            completed_order.retain(|stored_id| *stored_id != id);
        }
        existed
    }

    fn remember_completed(&self, id: u64) {
        let mut evicted = Vec::new();
        if let Ok(mut completed_order) = self.completed_order.lock() {
            completed_order.retain(|stored_id| *stored_id != id);
            completed_order.push_back(id);
            while completed_order.len() > MAX_STORED_COMMAND_RESULTS {
                if let Some(old_id) = completed_order.pop_front() {
                    evicted.push(old_id);
                }
            }
        }
        if !evicted.is_empty() {
            if let Ok(mut responses) = self.responses.lock() {
                for old_id in evicted {
                    responses.remove(&old_id);
                }
            }
        }
    }
}

pub enum CommandResultState {
    Done(String),
    Pending,
    Unknown,
}

pub struct ApiServerNode {
    id: usize,
    port: u16,
    enabled: bool,
    requests_seen: Arc<AtomicU64>,
    readiness_checks: u64,
    internal_fe: f32,
}

pub struct ApiRuntimeMetrics {
    pub started_ms: u128,
    pub ready: AtomicBool,
    pub autonomous: AtomicBool,
    pub daemon_enabled: AtomicBool,
    pub alive_nodes: AtomicU64,
    pub edge_count: AtomicU64,
    pub memory_facts: AtomicU64,
    pub api_requests: AtomicU64,
    pub meltrace_grabados: AtomicU64,
    pub meltrace_muertes: AtomicU64,
    pub meltrace_autons_vivos: AtomicU64,
    pub awareness_micros: AtomicU64,
    pub integration_micros: AtomicU64,
    pub phi_micros: AtomicU64,
    pub complexity_micros: AtomicU64,
    pub max_complexity_micros: AtomicU64,
    pub self_modifications: AtomicU64,
    pub autonomous_thoughts: AtomicU64,
    pub children_alive: AtomicU64,
    pub cag_cache_entries: AtomicU64,
    pub cag_hits: AtomicU64,
    pub cag_misses: AtomicU64,
    pub cag_ttl_ticks: AtomicU64,
    pub cag_feedback_positive: AtomicU64,
    pub cag_feedback_negative: AtomicU64,
    pub cag_pending_actions: AtomicU64,
    pub cag_actions_executed: AtomicU64,
    pub cag_actions_blocked: AtomicU64,
    pub cag_autonomous_runs: AtomicU64,
}

impl ApiRuntimeMetrics {
    pub fn new(daemon_enabled: bool) -> Self {
        let started_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        Self {
            started_ms,
            ready: AtomicBool::new(false),
            autonomous: AtomicBool::new(true),
            daemon_enabled: AtomicBool::new(daemon_enabled),
            alive_nodes: AtomicU64::new(0),
            edge_count: AtomicU64::new(0),
            memory_facts: AtomicU64::new(0),
            api_requests: AtomicU64::new(0),
            meltrace_grabados: AtomicU64::new(0),
            meltrace_muertes: AtomicU64::new(0),
            meltrace_autons_vivos: AtomicU64::new(0),
            awareness_micros: AtomicU64::new(400_000),
            integration_micros: AtomicU64::new(100_000),
            phi_micros: AtomicU64::new(0),
            complexity_micros: AtomicU64::new(0),
            max_complexity_micros: AtomicU64::new(0),
            self_modifications: AtomicU64::new(0),
            autonomous_thoughts: AtomicU64::new(0),
            children_alive: AtomicU64::new(0),
            cag_cache_entries: AtomicU64::new(0),
            cag_hits: AtomicU64::new(0),
            cag_misses: AtomicU64::new(0),
            cag_ttl_ticks: AtomicU64::new(0),
            cag_feedback_positive: AtomicU64::new(0),
            cag_feedback_negative: AtomicU64::new(0),
            cag_pending_actions: AtomicU64::new(0),
            cag_actions_executed: AtomicU64::new(0),
            cag_actions_blocked: AtomicU64::new(0),
            cag_autonomous_runs: AtomicU64::new(0),
        }
    }

    pub fn uptime_sec(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(self.started_ms);
        now.saturating_sub(self.started_ms) as u64 / 1000
    }
}

impl ApiServerNode {
    pub fn new(
        id: usize,
        engine: Arc<Mutex<GarmCapabilityState>>,
        command_queue: Arc<Mutex<Vec<QueuedCommand>>>,
        response_bus: Arc<CommandResponseBus>,
        runtime_metrics: Arc<ApiRuntimeMetrics>,
        port: u16,
    ) -> Self {
        let requests_seen = Arc::new(AtomicU64::new(0));
        let enabled = if port == 0 {
            false
        } else {
            match Self::spawn_local_server(
                port,
                engine,
                command_queue,
                response_bus,
                runtime_metrics,
                Arc::clone(&requests_seen),
            ) {
                Ok(_) => {
                    println!("[API] Local GARM API listening on 127.0.0.1:{}", port);
                    true
                }
                Err(e) => {
                    println!("[API] disabled: failed to bind 127.0.0.1:{} | {}", port, e);
                    false
                }
            }
        };
        Self {
            id,
            port,
            enabled,
            requests_seen,
            readiness_checks: 0,
            internal_fe: 1.0,
        }
    }

    fn spawn_local_server(
        port: u16,
        engine: Arc<Mutex<GarmCapabilityState>>,
        command_queue: Arc<Mutex<Vec<QueuedCommand>>>,
        response_bus: Arc<CommandResponseBus>,
        runtime_metrics: Arc<ApiRuntimeMetrics>,
        requests_seen: Arc<AtomicU64>,
    ) -> Result<(), String> {
        let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    requests_seen.fetch_add(1, Ordering::Relaxed);
                    runtime_metrics.api_requests.fetch_add(1, Ordering::Relaxed);
                    Self::handle_stream(
                        &mut stream,
                        &engine,
                        &command_queue,
                        &response_bus,
                        &runtime_metrics,
                    );
                }
            }
        });
        Ok(())
    }

    fn handle_stream(
        stream: &mut TcpStream,
        engine: &Arc<Mutex<GarmCapabilityState>>,
        command_queue: &Arc<Mutex<Vec<QueuedCommand>>>,
        response_bus: &Arc<CommandResponseBus>,
        runtime_metrics: &Arc<ApiRuntimeMetrics>,
    ) {
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        let request = String::from_utf8_lossy(&buf[..n]);
        let response = Self::handle_request_text(
            &request,
            engine,
            command_queue,
            response_bus,
            runtime_metrics,
        );
        let _ = stream.write_all(response.as_bytes());
    }

    fn handle_request_text(
        request: &str,
        engine: &Arc<Mutex<GarmCapabilityState>>,
        command_queue: &Arc<Mutex<Vec<QueuedCommand>>>,
        response_bus: &Arc<CommandResponseBus>,
        runtime_metrics: &Arc<ApiRuntimeMetrics>,
    ) -> String {
        let first_line = request.lines().next().unwrap_or("");
        let method = first_line.split_whitespace().next().unwrap_or("");
        let path = first_line.split_whitespace().nth(1).unwrap_or("/");
        if let Some(response) = Self::auth_failure_response(request, path) {
            return response;
        }

        let (status, content_type, body) = if path == "/"
            || path == "/console"
            || path == "/api/console"
        {
            ("200 OK", "text/html", OPERATOR_CONSOLE_HTML.to_string())
        } else if path == "/api/help" || path == "/help" {
            ("200 OK", "text/plain", API_HELP_TEXT.to_string())
        } else if path == "/health" || path == "/api/health" {
            (
                "200 OK",
                "application/json",
                "{\"status\":\"ok\",\"service\":\"eden\"}".to_string(),
            )
        } else if path.starts_with("/ready") {
            if runtime_metrics.ready.load(Ordering::Relaxed) {
                ("200 OK", "text/plain", "ready\n".to_string())
            } else {
                (
                    "503 Service Unavailable",
                    "text/plain",
                    "starting\n".to_string(),
                )
            }
        } else if path.starts_with("/status") || path.starts_with("/api/status") {
            let guard = engine.lock().unwrap();
            let body = format!(
                "{{\"born\":{},\"autonomous\":{},\"cycles\":{},\"evolution_level\":{},\"complexity\":{:.4},\"garm_status\":\"{}\"}}",
                runtime_metrics.ready.load(Ordering::Relaxed),
                runtime_metrics.autonomous.load(Ordering::Relaxed),
                guard.state.tick_count,
                guard.state.tick_count / 200 + 1,
                runtime_metrics.alive_nodes.load(Ordering::Relaxed) as f32 + runtime_metrics.edge_count.load(Ordering::Relaxed) as f32 * 0.001,
                Self::json_escape(&guard.status_summary()),
            );
            ("200 OK", "application/json", body)
        } else if path.starts_with("/state") {
            ("200 OK", "text/plain", state_paths::state_report())
        } else if path.starts_with("/report/history") || path.starts_with("/api/report/history") {
            match std::fs::read_to_string(state_paths::garm_report_history_path()) {
                Ok(history) => ("200 OK", "application/x-ndjson", history),
                Err(_) => (
                    "404 Not Found",
                    "text/plain",
                    "garm report history not generated; run command_sync?cmd=garm+report\n"
                        .to_string(),
                ),
            }
        } else if path.starts_with("/report") || path.starts_with("/api/report") {
            match std::fs::read_to_string(state_paths::garm_report_path()) {
                Ok(report) => ("200 OK", "text/plain", report),
                Err(_) => (
                    "404 Not Found",
                    "text/plain",
                    "garm report not generated; run command_sync?cmd=garm+report\n".to_string(),
                ),
            }
        } else if path.starts_with("/export/verify") || path.starts_with("/api/export/verify") {
            match Self::verify_export_file() {
                Ok(body) => ("200 OK", "text/plain", body),
                Err(body) => ("404 Not Found", "text/plain", body),
            }
        } else if path.starts_with("/export") || path.starts_with("/api/export") {
            match std::fs::read_to_string(state_paths::garm_export_path()) {
                Ok(export) => ("200 OK", "application/json", export),
                Err(_) => (
                    "404 Not Found",
                    "text/plain",
                    "garm export not generated; run command_sync?cmd=garm+export\n".to_string(),
                ),
            }
        } else if path == "/artifact/catalog"
            || path == "/api/artifact/catalog"
            || path == "/api/artifact_catalog"
        {
            ("200 OK", "application/json", artifact_api::catalog_json())
        } else if path == "/artifact/runtime" || path == "/api/artifact/runtime" {
            ("200 OK", "application/json", artifact_api::runtime_json())
        } else if path.starts_with("/artifact?") || path.starts_with("/api/artifact?") {
            if let Some(name) = Self::extract_query_value(path, "name") {
                match artifact_api::read_artifact(&name) {
                    Some((body, content_type)) => ("200 OK", content_type, body),
                    None => (
                        "404 Not Found",
                        "text/plain",
                        format!("unknown or missing artifact: {}\n", name),
                    ),
                }
            } else {
                (
                    "400 Bad Request",
                    "text/plain",
                    "missing artifact name query parameter\n".to_string(),
                )
            }
        } else if path.starts_with("/artifacts") || path.starts_with("/api/artifacts") {
            ("200 OK", "text/plain", state_paths::artifacts_report())
        } else if path == "/runtime/catalog" || path == "/api/runtime/catalog" {
            (
                "200 OK",
                "application/json",
                runtime_state_api::catalog_json(),
            )
        } else if path == "/runtime/openapi" || path == "/api/runtime/openapi" {
            (
                "200 OK",
                "application/json",
                runtime_state_api::openapi_json(),
            )
        } else if path == "/runtime/runtime" || path == "/api/runtime/runtime" {
            (
                "200 OK",
                "application/json",
                runtime_state_api::runtime_json(),
            )
        } else if path == "/runtime/snapshot" || path == "/api/runtime/snapshot" {
            (
                "200 OK",
                "application/json",
                Self::runtime_snapshot_json(engine, runtime_metrics),
            )
        } else if path.starts_with("/runtime/state?") || path.starts_with("/api/runtime/state?") {
            if let Some(name) = Self::extract_query_value(path, "name") {
                match runtime_state_api::read_state(&name) {
                    Some((body, content_type)) => ("200 OK", content_type, body),
                    None => (
                        "404 Not Found",
                        "text/plain",
                        format!("unknown or missing runtime state: {}\n", name),
                    ),
                }
            } else {
                (
                    "400 Bad Request",
                    "text/plain",
                    "missing runtime state name query parameter\n".to_string(),
                )
            }
        } else if path == "/operational/catalog" || path == "/api/operational/catalog" {
            (
                "200 OK",
                "application/json",
                operational_api::catalog_json(),
            )
        } else if path == "/operational/openapi" || path == "/api/operational/openapi" {
            (
                "200 OK",
                "application/json",
                operational_api::openapi_json(),
            )
        } else if path == "/operational/runtime" || path == "/api/operational/runtime" {
            (
                "200 OK",
                "application/json",
                operational_api::runtime_json(),
            )
        } else if path == "/operational/status" || path == "/api/operational/status" {
            (
                "200 OK",
                "application/json",
                Self::operational_status_json(engine, runtime_metrics),
            )
        } else if path == "/paradise/sessions" || path == "/api/paradise/sessions" {
            (
                "200 OK",
                "application/json",
                operational_api::paradise_sessions_json(),
            )
        } else if path == "/paradise/worldcell" || path == "/api/paradise/worldcell" {
            (
                "200 OK",
                "application/json",
                operational_api::paradise_worldcell_json(),
            )
        } else if path == "/operational/contract" || path == "/api/operational/contract" {
            (
                "200 OK",
                "application/json",
                operational_api::contract_json(),
            )
        } else if path == "/operational/permissions" || path == "/api/operational/permissions" {
            (
                "200 OK",
                "application/json",
                operational_api::permissions_json(),
            )
        } else if path == "/operational/replay" || path == "/api/operational/replay" {
            (
                "200 OK",
                "application/json",
                operational_api::replay_index_json(),
            )
        } else if path.starts_with("/operational/replay?")
            || path.starts_with("/api/operational/replay?")
        {
            if let Some(decision_id) = Self::extract_query_value(path, "decision_id") {
                (
                    "200 OK",
                    "application/json",
                    operational_api::replay_decision_json(&decision_id),
                )
            } else {
                (
                    "400 Bad Request",
                    "text/plain",
                    "missing decision_id query parameter\n".to_string(),
                )
            }
        } else if path == "/operational/recovery" || path == "/api/operational/recovery" {
            (
                "200 OK",
                "application/json",
                operational_api::recovery_json(),
            )
        } else if path == "/operational/demos" || path == "/api/operational/demos" {
            ("200 OK", "application/json", operational_api::demos_json())
        } else if path == "/operational/schemas" || path == "/api/operational/schemas" {
            (
                "200 OK",
                "application/json",
                operational_api::schema_registry_json(),
            )
        } else if path.starts_with("/operational/schema?")
            || path.starts_with("/api/operational/schema?")
        {
            if let Some(name) = Self::extract_query_value(path, "name") {
                (
                    "200 OK",
                    "application/json",
                    operational_api::schema_record_json(&name),
                )
            } else {
                (
                    "400 Bad Request",
                    "text/plain",
                    "missing schema name query parameter\n".to_string(),
                )
            }
        } else if path == "/operational/runtime-phase" || path == "/api/operational/runtime-phase" {
            (
                "200 OK",
                "application/json",
                operational_runtime::runtime_phase_json(),
            )
        } else if path == "/operational/spine" || path == "/api/operational/spine" {
            (
                "200 OK",
                "application/json",
                operational_api::runtime_spine_json(),
            )
        } else if path == "/operational/events" || path == "/api/operational/events" {
            (
                "200 OK",
                "application/json",
                operational_api::runtime_events_json(),
            )
        } else if path == "/operational/global-state" || path == "/api/operational/global-state" {
            (
                "200 OK",
                "application/json",
                operational_api::runtime_global_state_json(),
            )
        } else if path == "/operational/replay-spine" || path == "/api/operational/replay-spine" {
            (
                "200 OK",
                "application/json",
                operational_api::runtime_replay_spine_json(),
            )
        } else if path == "/operational/spine-verification"
            || path == "/api/operational/spine-verification"
        {
            (
                "200 OK",
                "application/json",
                operational_api::runtime_spine_verification_json(),
            )
        } else if path == "/operational/spine-enforcement"
            || path == "/api/operational/spine-enforcement"
        {
            (
                "200 OK",
                "application/json",
                operational_api::runtime_spine_enforcement_json(),
            )
        } else if path == "/operational/workflow-risk" || path == "/api/operational/workflow-risk" {
            (
                "200 OK",
                "application/json",
                operational_api::runtime_workflow_risk_json(),
            )
        } else if path == "/operational/circuit-breakers"
            || path == "/api/operational/circuit-breakers"
        {
            (
                "200 OK",
                "application/json",
                operational_api::runtime_circuit_breakers_json(),
            )
        } else if path == "/operational/spine-replay" || path == "/api/operational/spine-replay" {
            (
                "200 OK",
                "application/json",
                operational_api::runtime_spine_replay_json(),
            )
        } else if path == "/capabilities/catalog" || path == "/api/capabilities/catalog" {
            let guard = engine.lock().unwrap();
            (
                "200 OK",
                "application/json",
                operational_api::capabilities_catalog_json(&guard),
            )
        } else if path == "/capabilities/status" || path == "/api/capabilities/status" {
            (
                "200 OK",
                "application/json",
                Self::capabilities_status_json(engine, runtime_metrics),
            )
        } else if path == "/gewc/runtime" || path == "/api/gewc/runtime" {
            (
                "200 OK",
                "application/json",
                operational_api::gewc_runtime_json(),
            )
        } else if path == "/gewc/handlers" || path == "/api/gewc/handlers" {
            (
                "200 OK",
                "application/json",
                operational_api::gewc_handlers_json(),
            )
        } else if path == "/validation/status" || path == "/api/validation/status" {
            (
                "200 OK",
                "application/json",
                operational_api::validation_status_json(),
            )
        } else if path == "/actions/contracts" || path == "/api/actions/contracts" {
            (
                "200 OK",
                "application/json",
                operational_api::action_contracts_json(),
            )
        } else if path.starts_with("/actions/dry-run?") || path.starts_with("/api/actions/dry-run?")
        {
            if let Some(cmd) = Self::extract_query_value(path, "cmd") {
                (
                    "200 OK",
                    "application/json",
                    operational_api::action_dry_run_json(&cmd),
                )
            } else {
                (
                    "400 Bad Request",
                    "text/plain",
                    "missing command query parameter\n".to_string(),
                )
            }
        } else if path.starts_with("/metrics") || path.starts_with("/api/metrics") {
            let guard = engine.lock().unwrap();
            let organ_metrics = organ_registry::metrics();
            let body = format!(
                "{{\"ticks\":{},\"parse_rate\":{:.6},\"reward_ema\":{:.6},\"energy\":{:.3},\"alive_nodes\":{},\"edges\":{},\"memory_facts\":{},\"uptime_sec\":{},\"daemon\":{},\"ready\":{},\"autonomous\":{},\"api_requests\":{},\"cycles\":{},\"premises\":{},\"evolution_level\":{},\"self_modifications\":{},\"awareness\":{:.4},\"integration\":{:.4},\"phi\":{:.4},\"complexity\":{:.4},\"max_complexity\":{:.4},\"autonomous_thoughts\":{},\"children_alive\":{},\"meltrace_grabados\":{},\"meltrace_muertes\":{},\"meltrace_autons_vivos\":{},\"cag_cache_entries\":{},\"cag_hits\":{},\"cag_misses\":{},\"cag_ttl_ticks\":{},\"cag_feedback_positive\":{},\"cag_feedback_negative\":{},\"cag_pending_actions\":{},\"cag_actions_executed\":{},\"cag_actions_blocked\":{},\"cag_autonomous_runs\":{},\"organ_pending_actions\":{},\"organ_actions_executed\":{},\"organ_actions_blocked\":{},\"organ_autonomous_runs\":{},\"organ_feedback_positive\":{},\"organ_feedback_negative\":{}}}",
                guard.state.tick_count,
                guard.gen_metrics.parse_rate(),
                guard.gen_metrics.reward_ema,
                guard.metabolism.energy,
                runtime_metrics.alive_nodes.load(Ordering::Relaxed),
                runtime_metrics.edge_count.load(Ordering::Relaxed),
                runtime_metrics.memory_facts.load(Ordering::Relaxed),
                runtime_metrics.uptime_sec(),
                runtime_metrics.daemon_enabled.load(Ordering::Relaxed),
                runtime_metrics.ready.load(Ordering::Relaxed),
                runtime_metrics.autonomous.load(Ordering::Relaxed),
                runtime_metrics.api_requests.load(Ordering::Relaxed),
                guard.state.tick_count,
                runtime_metrics.memory_facts.load(Ordering::Relaxed),
                guard.state.tick_count / 200 + 1,
                runtime_metrics.self_modifications.load(Ordering::Relaxed),
                runtime_metrics.awareness_micros.load(Ordering::Relaxed) as f32 / 1_000_000.0,
                runtime_metrics.integration_micros.load(Ordering::Relaxed) as f32 / 1_000_000.0,
                runtime_metrics.phi_micros.load(Ordering::Relaxed) as f32 / 1_000_000.0,
                runtime_metrics.complexity_micros.load(Ordering::Relaxed) as f32 / 1_000_000.0,
                runtime_metrics.max_complexity_micros.load(Ordering::Relaxed) as f32 / 1_000_000.0,
                runtime_metrics.autonomous_thoughts.load(Ordering::Relaxed),
                runtime_metrics.children_alive.load(Ordering::Relaxed),
                runtime_metrics.meltrace_grabados.load(Ordering::Relaxed),
                runtime_metrics.meltrace_muertes.load(Ordering::Relaxed),
                runtime_metrics.meltrace_autons_vivos.load(Ordering::Relaxed),
                runtime_metrics.cag_cache_entries.load(Ordering::Relaxed),
                runtime_metrics.cag_hits.load(Ordering::Relaxed),
                runtime_metrics.cag_misses.load(Ordering::Relaxed),
                runtime_metrics.cag_ttl_ticks.load(Ordering::Relaxed),
                runtime_metrics.cag_feedback_positive.load(Ordering::Relaxed),
                runtime_metrics.cag_feedback_negative.load(Ordering::Relaxed),
                runtime_metrics.cag_pending_actions.load(Ordering::Relaxed),
                runtime_metrics.cag_actions_executed.load(Ordering::Relaxed),
                runtime_metrics.cag_actions_blocked.load(Ordering::Relaxed),
                runtime_metrics.cag_autonomous_runs.load(Ordering::Relaxed),
                organ_metrics.pending_actions,
                organ_metrics.actions_executed,
                organ_metrics.actions_blocked,
                organ_metrics.autonomous_runs,
                organ_metrics.feedback_positive,
                organ_metrics.feedback_negative,
            );
            ("200 OK", "application/json", body)
        } else if path.starts_with("/organs/actions") || path.starts_with("/api/organs/actions") {
            (
                "200 OK",
                "text/plain",
                format!("{}\n", organ_registry::organ_actions_report()),
            )
        } else if path.starts_with("/organs/audit") || path.starts_with("/api/organs/audit") {
            (
                "200 OK",
                "text/plain",
                format!("{}\n", organ_registry::organ_autonomy_audit_report()),
            )
        } else if path.starts_with("/organs/recovery") || path.starts_with("/api/organs/recovery") {
            (
                "200 OK",
                "text/plain",
                format!("{}\n", organ_registry::organ_recovery_report()),
            )
        } else if path.starts_with("/organs") || path.starts_with("/api/organs") {
            let metrics = organ_registry::metrics();
            let body = format!(
                "{{\"organs\":{},\"pending\":{},\"executed\":{},\"blocked\":{},\"autonomous_runs\":{},\"feedback_positive\":{},\"feedback_negative\":{}}}",
                organ_registry::ORGAN_PROFILES.len(),
                metrics.pending_actions,
                metrics.actions_executed,
                metrics.actions_blocked,
                metrics.autonomous_runs,
                metrics.feedback_positive,
                metrics.feedback_negative,
            );
            ("200 OK", "application/json", body)
        } else if path.starts_with("/command_forget") || path.starts_with("/api/command_forget") {
            if let Some(id) =
                Self::extract_query_value(path, "id").and_then(|s| s.parse::<u64>().ok())
            {
                if response_bus.forget(id) {
                    ("200 OK", "text/plain", format!("forgot: {}\n", id))
                } else {
                    (
                        "404 Not Found",
                        "text/plain",
                        format!("unknown command id: {}\n", id),
                    )
                }
            } else {
                (
                    "400 Bad Request",
                    "text/plain",
                    "missing or invalid id query parameter\n".to_string(),
                )
            }
        } else if path.starts_with("/command_result") || path.starts_with("/api/command_result") {
            if let Some(id) =
                Self::extract_query_value(path, "id").and_then(|s| s.parse::<u64>().ok())
            {
                match response_bus.take_result(id) {
                    CommandResultState::Done(response) => ("200 OK", "text/plain", response),
                    CommandResultState::Pending => {
                        ("202 Accepted", "text/plain", format!("pending: {}\n", id))
                    }
                    CommandResultState::Unknown => (
                        "404 Not Found",
                        "text/plain",
                        format!("unknown command id: {}\n", id),
                    ),
                }
            } else {
                (
                    "400 Bad Request",
                    "text/plain",
                    "missing or invalid id query parameter\n".to_string(),
                )
            }
        } else if path.starts_with("/command_sync") || path.starts_with("/api/command_sync") {
            if !runtime_metrics.ready.load(Ordering::Relaxed) {
                return Self::format_response(
                    "503 Service Unavailable",
                    "text/plain",
                    "runtime not ready\n".to_string(),
                );
            }
            if let Some(cmd) = Self::extract_query_value(path, "cmd") {
                let id = response_bus.next_id();
                if let Ok(mut queue) = command_queue.lock() {
                    queue.push(QueuedCommand {
                        id: Some(id),
                        raw: cmd.clone(),
                    });
                }
                match response_bus.wait_for(id, Duration::from_secs(5)) {
                    Some(response) => ("200 OK", "text/plain", response),
                    None => (
                        "504 Gateway Timeout",
                        "text/plain",
                        format!("timeout waiting for command: {}\n", cmd),
                    ),
                }
            } else {
                (
                    "400 Bad Request",
                    "text/plain",
                    "missing cmd query parameter\n".to_string(),
                )
            }
        } else if path.starts_with("/command") || path.starts_with("/api/command") {
            if let Some(cmd) = Self::extract_query_value(path, "cmd") {
                let id = response_bus.next_id();
                if let Ok(mut queue) = command_queue.lock() {
                    queue.push(QueuedCommand {
                        id: Some(id),
                        raw: cmd.clone(),
                    });
                }
                (
                    "200 OK",
                    "text/plain",
                    format!("queued: {} cmd: {}\n", id, cmd),
                )
            } else if matches!(method, "POST" | "PUT") {
                let cmd = Self::extract_body(&request).trim().to_string();
                if cmd.is_empty() {
                    (
                        "400 Bad Request",
                        "application/json",
                        "{\"error\":\"invalid_command\"}".to_string(),
                    )
                } else {
                    let id = response_bus.next_id();
                    if let Ok(mut queue) = command_queue.lock() {
                        queue.push(QueuedCommand {
                            id: Some(id),
                            raw: cmd.clone(),
                        });
                    }
                    let legacy_message = if cmd.eq_ignore_ascii_case("evolve") {
                        "Evolution triggered".to_string()
                    } else if cmd.eq_ignore_ascii_case("save") {
                        "Session saved".to_string()
                    } else {
                        let response = response_bus
                            .wait_for(id, Duration::from_secs(5))
                            .unwrap_or_else(|| format!("queued: {} cmd: {}\n", id, cmd));
                        Self::json_escape(response.trim())
                    };
                    (
                        "200 OK",
                        "application/json",
                        format!("{{\"result\":\"ok\",\"message\":\"{}\"}}", legacy_message),
                    )
                }
            } else {
                (
                    "400 Bad Request",
                    "text/plain",
                    "missing cmd query parameter\n".to_string(),
                )
            }
        } else {
            ("404 Not Found", "text/plain", "not found\n".to_string())
        };

        Self::format_response(status, content_type, body)
    }

    fn runtime_snapshot_json(
        engine: &Arc<Mutex<GarmCapabilityState>>,
        runtime_metrics: &Arc<ApiRuntimeMetrics>,
    ) -> String {
        let guard = engine.lock().unwrap();
        let organ_metrics = organ_registry::metrics();
        runtime_state_api::live_snapshot_json(runtime_state_api::LiveRuntimeSnapshotInput {
            ready: runtime_metrics.ready.load(Ordering::Relaxed),
            autonomous: runtime_metrics.autonomous.load(Ordering::Relaxed),
            daemon_enabled: runtime_metrics.daemon_enabled.load(Ordering::Relaxed),
            uptime_sec: runtime_metrics.uptime_sec(),
            tick_count: guard.state.tick_count,
            evolution_level: guard.state.tick_count / 200 + 1,
            alive_nodes: runtime_metrics.alive_nodes.load(Ordering::Relaxed),
            edge_count: runtime_metrics.edge_count.load(Ordering::Relaxed),
            memory_facts: runtime_metrics.memory_facts.load(Ordering::Relaxed),
            api_requests: runtime_metrics.api_requests.load(Ordering::Relaxed),
            organ_count: organ_registry::ORGAN_PROFILES.len(),
            organ_pending_actions: organ_metrics.pending_actions,
            organ_actions_executed: organ_metrics.actions_executed,
            organ_actions_blocked: organ_metrics.actions_blocked,
            organ_autonomous_runs: organ_metrics.autonomous_runs,
            engine_status: &guard.status_summary(),
        })
    }

    fn capabilities_status_json(
        engine: &Arc<Mutex<GarmCapabilityState>>,
        runtime_metrics: &Arc<ApiRuntimeMetrics>,
    ) -> String {
        let guard = engine.lock().unwrap();
        let engine_status = guard.status_summary();
        operational_api::capabilities_status_json(operational_api::OperationalStatusInput {
            ready: runtime_metrics.ready.load(Ordering::Relaxed),
            autonomous: runtime_metrics.autonomous.load(Ordering::Relaxed),
            daemon_enabled: runtime_metrics.daemon_enabled.load(Ordering::Relaxed),
            uptime_sec: runtime_metrics.uptime_sec(),
            alive_nodes: runtime_metrics.alive_nodes.load(Ordering::Relaxed),
            edge_count: runtime_metrics.edge_count.load(Ordering::Relaxed),
            memory_facts: runtime_metrics.memory_facts.load(Ordering::Relaxed),
            api_requests: runtime_metrics.api_requests.load(Ordering::Relaxed),
            engine_status: &engine_status,
            capability_count: guard.state.capabilities.len(),
            tick_count: guard.state.tick_count,
            idle_ticks: guard.state.idle_ticks,
        })
    }

    fn operational_status_json(
        engine: &Arc<Mutex<GarmCapabilityState>>,
        runtime_metrics: &Arc<ApiRuntimeMetrics>,
    ) -> String {
        let guard = engine.lock().unwrap();
        let engine_status = guard.status_summary();
        operational_api::status_json(operational_api::OperationalStatusInput {
            ready: runtime_metrics.ready.load(Ordering::Relaxed),
            autonomous: runtime_metrics.autonomous.load(Ordering::Relaxed),
            daemon_enabled: runtime_metrics.daemon_enabled.load(Ordering::Relaxed),
            uptime_sec: runtime_metrics.uptime_sec(),
            alive_nodes: runtime_metrics.alive_nodes.load(Ordering::Relaxed),
            edge_count: runtime_metrics.edge_count.load(Ordering::Relaxed),
            memory_facts: runtime_metrics.memory_facts.load(Ordering::Relaxed),
            api_requests: runtime_metrics.api_requests.load(Ordering::Relaxed),
            engine_status: &engine_status,
            capability_count: guard.state.capabilities.len(),
            tick_count: guard.state.tick_count,
            idle_ticks: guard.state.idle_ticks,
        })
    }

    fn format_response(status: &str, content_type: &str, body: String) -> String {
        format!(
            "HTTP/1.1 {}\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status,
            content_type,
            body.as_bytes().len(),
            body,
        )
    }

    fn auth_failure_response(request: &str, path: &str) -> Option<String> {
        let token = Self::configured_api_token()?;
        if Self::is_public_without_token(path) || Self::request_has_token(request, &token) {
            return None;
        }
        Some(Self::format_response(
            "401 Unauthorized",
            "application/json",
            "{\"error\":\"unauthorized\",\"auth\":\"EDEN_API_TOKEN\"}\n".to_string(),
        ))
    }

    fn configured_api_token() -> Option<String> {
        std::env::var("EDEN_API_TOKEN")
            .ok()
            .map(|token| token.trim().to_string())
            .filter(|token| !token.is_empty())
    }

    fn is_public_without_token(path: &str) -> bool {
        let route = path.split('?').next().unwrap_or(path);
        matches!(
            route,
            "/" | "/console"
                | "/api/console"
                | "/help"
                | "/api/help"
                | "/health"
                | "/api/health"
                | "/ready"
        )
    }

    fn request_has_token(request: &str, expected: &str) -> bool {
        for line in request.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                break;
            }
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            let value = value.trim();
            if name.eq_ignore_ascii_case("authorization") {
                let Some((scheme, token)) = value.split_once(' ') else {
                    continue;
                };
                if scheme.eq_ignore_ascii_case("bearer") && token.trim() == expected {
                    return true;
                }
            }
            if name.eq_ignore_ascii_case("x-eden-api-token") && value == expected {
                return true;
            }
        }
        false
    }

    fn extract_query_value(path: &str, key: &str) -> Option<String> {
        let query = path.split_once('?')?.1;
        for pair in query.split('&') {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            if k == key {
                return Some(Self::url_decode(v));
            }
        }
        None
    }

    fn extract_body(request: &str) -> &str {
        request
            .split_once("\r\n\r\n")
            .map(|(_, body)| body)
            .or_else(|| request.split_once("\n\n").map(|(_, body)| body))
            .unwrap_or("")
    }

    fn json_escape(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    fn verify_export_file() -> Result<String, String> {
        let path = state_paths::garm_export_path();
        let data = std::fs::read_to_string(&path).map_err(|_| {
            format!(
                "[GARM-VERIFY-EXPORT] ok=false error=export_missing path={}\n",
                path
            )
        })?;
        let value: serde_json::Value = serde_json::from_str(&data).map_err(|_| {
            format!(
                "[GARM-VERIFY-EXPORT] ok=false error=invalid_json path={}\n",
                path
            )
        })?;
        let expected = value
            .pointer("/integrity/checksum_fnv64")
            .and_then(|v| v.as_str())
            .unwrap_or("missing");
        let actual = format!("{:016x}", Self::export_checksum_value(&value));
        let ok = expected == actual;
        Ok(format!(
            "[GARM-VERIFY-EXPORT] ok={} expected={} actual={} algorithm=fnv64 cryptographic=false path={}\n",
            ok, expected, actual, path
        ))
    }

    fn export_checksum_value(value: &serde_json::Value) -> u64 {
        let mut scoped = value.clone();
        if let Some(object) = scoped.as_object_mut() {
            object.remove("integrity");
        }
        let canonical = serde_json::to_string(&scoped).unwrap_or_default();
        Self::fnv64(canonical.as_bytes())
    }

    fn fnv64(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0xcbf29ce484222325u64, |mut hash, byte| {
            hash ^= *byte as u64;
            hash.wrapping_mul(0x100000001b3)
        })
    }

    fn url_decode(input: &str) -> String {
        let bytes = input.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'+' => {
                    out.push(b' ');
                    i += 1;
                }
                b'%' if i + 2 < bytes.len() => {
                    if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                        if let Ok(value) = u8::from_str_radix(hex, 16) {
                            out.push(value);
                            i += 3;
                            continue;
                        }
                    }
                    out.push(bytes[i]);
                    i += 1;
                }
                b => {
                    out.push(b);
                    i += 1;
                }
            }
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    pub fn status(&self) -> String {
        format!(
            "ApiServer | enabled={} | port={} | requests={}",
            self.enabled,
            self.port,
            self.requests_seen.load(Ordering::Relaxed),
        )
    }

    pub fn check_local_readiness(&mut self) -> String {
        self.readiness_checks += 1;
        format!(
            "[API-AUTO] local_ready={} port={} requests={} readiness_checks={}",
            self.enabled,
            self.port,
            self.requests_seen.load(Ordering::Relaxed),
            self.readiness_checks,
        )
    }

    pub fn readiness_snapshot(&self) -> String {
        format!(
            "api:enabled:{} port:{} requests:{} readiness_checks:{}",
            self.enabled,
            self.port,
            self.requests_seen.load(Ordering::Relaxed),
            self.readiness_checks
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "requests_seen": self.requests_seen.load(Ordering::Relaxed),
            "readiness_checks": self.readiness_checks,
            "internal_fe": self.internal_fe,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.requests_seen.store(
            snapshot
                .get("requests_seen")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            Ordering::Relaxed,
        );
        self.readiness_checks = snapshot
            .get("readiness_checks")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.internal_fe = snapshot
            .get("internal_fe")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        Ok(())
    }
}

impl GARMNode for ApiServerNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "api_server"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        if self.enabled {
            self.internal_fe + 0.5
        } else {
            self.internal_fe * 0.2
        }
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.enabled as u8 as f32,
            self.requests_seen.load(Ordering::Relaxed) as f32,
        ]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![
            self.enabled as u8 as f32,
            self.requests_seen.load(Ordering::Relaxed) as f32,
        ])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
        if self.enabled {
            0.5
        } else {
            0.05
        }
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        50.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Clone)]
    struct TestServerContext {
        engine: Arc<Mutex<GarmCapabilityState>>,
        command_queue: Arc<Mutex<Vec<QueuedCommand>>>,
        response_bus: Arc<CommandResponseBus>,
        runtime_metrics: Arc<ApiRuntimeMetrics>,
    }

    static NEXT_TEST_PORT: AtomicU64 = AtomicU64::new(30_000);
    static TEST_SERVERS: std::sync::OnceLock<Mutex<HashMap<u16, TestServerContext>>> =
        std::sync::OnceLock::new();

    fn test_servers() -> &'static Mutex<HashMap<u16, TestServerContext>> {
        TEST_SERVERS.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn free_port() -> u16 {
        NEXT_TEST_PORT.fetch_add(1, Ordering::Relaxed) as u16
    }

    fn http_get(port: u16, path: &str) -> String {
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
            path
        );
        test_request(port, &request)
    }

    fn http_get_with_header(port: u16, path: &str, header: &str) -> String {
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\n{}\r\nConnection: close\r\n\r\n",
            path, header
        );
        test_request(port, &request)
    }

    fn http_post(port: u16, path: &str, body: &str) -> String {
        let request = format!(
            "POST {} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            path,
            body.len(),
            body,
        );
        test_request(port, &request)
    }

    fn test_request(port: u16, request: &str) -> String {
        let context = test_servers()
            .lock()
            .unwrap()
            .get(&port)
            .cloned()
            .unwrap_or_else(|| panic!("test API context not found for port {}", port));
        context
            .runtime_metrics
            .api_requests
            .fetch_add(1, Ordering::Relaxed);
        ApiServerNode::handle_request_text(
            request,
            &context.engine,
            &context.command_queue,
            &context.response_bus,
            &context.runtime_metrics,
        )
    }

    fn http_get_tcp(port: u16, path: &str) -> String {
        let mut last_err = None;
        for _ in 0..20 {
            match std::net::TcpStream::connect(("127.0.0.1", port)) {
                Ok(mut stream) => {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let request = format!(
                        "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
                        path
                    );
                    stream.write_all(request.as_bytes()).unwrap();
                    let mut response = String::new();
                    stream.read_to_string(&mut response).unwrap();
                    return response;
                }
                Err(e) => {
                    last_err = Some(e);
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
        panic!("failed to connect to API server: {:?}", last_err);
    }

    fn real_free_port() -> u16 {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.local_addr().unwrap().port()
    }

    fn response_body(response: &str) -> &str {
        response.split("\r\n\r\n").nth(1).unwrap_or("")
    }

    fn test_server() -> (
        u16,
        Arc<Mutex<Vec<QueuedCommand>>>,
        Arc<CommandResponseBus>,
        Arc<ApiRuntimeMetrics>,
    ) {
        let port = free_port();
        let engine = Arc::new(Mutex::new(GarmCapabilityState::new_fast()));
        let queue = Arc::new(Mutex::new(Vec::new()));
        let response_bus = Arc::new(CommandResponseBus::new());
        let metrics = Arc::new(ApiRuntimeMetrics::new(false));
        test_servers().lock().unwrap().insert(
            port,
            TestServerContext {
                engine,
                command_queue: Arc::clone(&queue),
                response_bus: Arc::clone(&response_bus),
                runtime_metrics: Arc::clone(&metrics),
            },
        );
        (port, queue, response_bus, metrics)
    }

    #[test]
    fn reports_readiness_state_and_metrics() {
        let (port, _queue, _response_bus, metrics) = test_server();
        metrics.alive_nodes.store(108, Ordering::Relaxed);
        metrics.edge_count.store(467, Ordering::Relaxed);
        metrics.memory_facts.store(3, Ordering::Relaxed);

        let starting = http_get(port, "/ready");
        assert!(starting.starts_with("HTTP/1.1 503 Service Unavailable"));
        assert_eq!(response_body(&starting), "starting\n");

        metrics.ready.store(true, Ordering::Relaxed);
        let ready = http_get(port, "/ready");
        assert!(ready.starts_with("HTTP/1.1 200 OK"));
        assert_eq!(response_body(&ready), "ready\n");

        let state = http_get(port, "/state");
        assert!(state.contains("GARM state directory:"));
        assert!(state.contains("legacy_memory:"));

        let metrics_response = http_get(port, "/metrics");
        assert!(metrics_response.starts_with("HTTP/1.1 200 OK"));
        let body = response_body(&metrics_response);
        assert!(body.contains("\"alive_nodes\":108"));
        assert!(body.contains("\"edges\":467"));
        assert!(body.contains("\"memory_facts\":3"));
        assert!(body.contains("\"ready\":true"));

        let organs = http_get(port, "/api/organs");
        assert!(organs.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&organs).contains("\"organs\":32"));

        let recovery = http_get(port, "/organs/recovery");
        assert!(recovery.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&recovery).contains("[ORGANOS-RECOVERY]"));

        let console = http_get(port, "/");
        assert!(console.starts_with("HTTP/1.1 200 OK"));
        assert!(console.contains("Content-Type: text/html"));
        assert!(response_body(&console).contains("EDEN Operator Console"));

        let help = http_get(port, "/api/help");
        assert!(help.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&help).contains("/api/runtime/catalog"));
    }

    #[test]
    fn optional_api_token_protects_non_public_routes() {
        let previous_token = std::env::var("EDEN_API_TOKEN").ok();
        std::env::set_var("EDEN_API_TOKEN", "local-secret");

        let (port, _queue, _response_bus, metrics) = test_server();
        metrics.ready.store(true, Ordering::Relaxed);

        let public = http_get(port, "/ready");
        assert!(public.starts_with("HTTP/1.1 200 OK"));

        let blocked = http_get(port, "/api/operational/status");
        assert!(blocked.starts_with("HTTP/1.1 401 Unauthorized"));
        assert!(response_body(&blocked).contains("\"unauthorized\""));

        let authorized = http_get_with_header(
            port,
            "/api/operational/status",
            "Authorization: Bearer local-secret",
        );
        assert!(authorized.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&authorized).contains("eden-operational-status-v1"));

        let alternate = http_get_with_header(
            port,
            "/api/operational/status",
            "X-EDEN-API-Token: local-secret",
        );
        assert!(alternate.starts_with("HTTP/1.1 200 OK"));

        if let Some(token) = previous_token {
            std::env::set_var("EDEN_API_TOKEN", token);
        } else {
            std::env::remove_var("EDEN_API_TOKEN");
        }
    }

    #[test]
    fn socket_transport_serves_health_when_enabled() {
        if std::env::var("EDEN_API_SOCKET_TESTS").ok().as_deref() != Some("1") {
            eprintln!("skipping socket transport test; set EDEN_API_SOCKET_TESTS=1 to enable");
            return;
        }
        let port = real_free_port();
        let engine = Arc::new(Mutex::new(GarmCapabilityState::new_fast()));
        let queue = Arc::new(Mutex::new(Vec::new()));
        let response_bus = Arc::new(CommandResponseBus::new());
        let metrics = Arc::new(ApiRuntimeMetrics::new(false));
        let server = ApiServerNode::new(1, engine, queue, response_bus, Arc::clone(&metrics), port);
        assert!(server.enabled);

        let health = http_get_tcp(port, "/health");
        assert!(health.starts_with("HTTP/1.1 200 OK"));
        assert_eq!(
            response_body(&health),
            "{\"status\":\"ok\",\"service\":\"eden\"}"
        );
    }

    #[test]
    fn serves_persisted_garm_report() {
        let _state_guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_garm_api_report_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        state_paths::ensure_state_dir().unwrap();
        std::fs::write(
            state_paths::garm_report_path(),
            "[GARM-REPORT] verdict=ready\n",
        )
        .unwrap();
        std::fs::write(
            state_paths::garm_report_history_path(),
            "{\"tick\":1,\"verdict\":\"ready\"}\n",
        )
        .unwrap();
        let mut export_value = serde_json::json!({
            "schema": "garm-export-v1",
            "runtime": {"ready": true},
        });
        let checksum = ApiServerNode::export_checksum_value(&export_value);
        export_value["integrity"] = serde_json::json!({
            "checksum_fnv64": format!("{:016x}", checksum),
        });
        std::fs::write(state_paths::garm_export_path(), export_value.to_string()).unwrap();
        std::fs::create_dir_all(state_paths::backup_dir_path()).unwrap();
        std::fs::write(state_paths::memory_eval_path(), "{\"sample\":true}").unwrap();
        std::fs::write(state_paths::goal_scheduler_state_path(), "{\"goals\":1}").unwrap();
        let _ = runtime_state_api::run();
        let _ = operational_api::run();
        let _ = artifact_api::run();
        let (port, _queue, _response_bus, metrics) = test_server();
        metrics.ready.store(true, Ordering::Relaxed);
        metrics.alive_nodes.store(12, Ordering::Relaxed);

        let report = http_get(port, "/report");
        let api_report = http_get(port, "/api/report");
        let history = http_get(port, "/report/history");
        let api_history = http_get(port, "/api/report/history");
        let export = http_get(port, "/export");
        let api_export = http_get(port, "/api/export");
        let verify_export = http_get(port, "/export/verify");
        let api_verify_export = http_get(port, "/api/export/verify");
        let artifacts = http_get(port, "/artifacts");
        let api_artifacts = http_get(port, "/api/artifacts");
        let artifact_catalog = http_get(port, "/api/artifact/catalog");
        let artifact_catalog_alias = http_get(port, "/api/artifact_catalog");
        let artifact_runtime = http_get(port, "/api/artifact/runtime");
        let memory_artifact = http_get(port, "/api/artifact?name=memory_eval");
        let unsafe_artifact = http_get(port, "/api/artifact?name=../secret");
        let runtime_catalog = http_get(port, "/api/runtime/catalog");
        let runtime_openapi = http_get(port, "/api/runtime/openapi");
        let runtime_snapshot = http_get(port, "/api/runtime/snapshot");
        let goal_state = http_get(port, "/api/runtime/state?name=goal_scheduler");
        let unsafe_state = http_get(port, "/api/runtime/state?name=../secret");
        let capability_catalog = http_get(port, "/api/capabilities/catalog");
        let capability_status = http_get(port, "/api/capabilities/status");
        let gewc_handlers = http_get(port, "/api/gewc/handlers");
        let validation_status = http_get(port, "/api/validation/status");
        let operational_status = http_get(port, "/api/operational/status");
        let operational_contract = http_get(port, "/api/operational/contract");
        let operational_permissions = http_get(port, "/api/operational/permissions");
        let operational_replay = http_get(port, "/api/operational/replay");
        let operational_replay_decision =
            http_get(port, "/api/operational/replay?decision_id=missing");
        let operational_recovery = http_get(port, "/api/operational/recovery");
        let operational_demos = http_get(port, "/api/operational/demos");
        let operational_schemas = http_get(port, "/api/operational/schemas");
        let operational_schema_record =
            http_get(port, "/api/operational/schema?name=operational_status");
        let action_contracts = http_get(port, "/api/actions/contracts");
        let action_dry_run = http_get(port, "/api/actions/dry-run?cmd=evolve");

        assert!(report.starts_with("HTTP/1.1 200 OK"));
        assert_eq!(response_body(&report), "[GARM-REPORT] verdict=ready\n");
        assert_eq!(response_body(&api_report), "[GARM-REPORT] verdict=ready\n");
        assert!(history.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&history).contains("\"verdict\":\"ready\""));
        assert!(response_body(&api_history).contains("\"tick\":1"));
        assert!(export.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&export).contains("\"schema\":\"garm-export-v1\""));
        assert!(response_body(&api_export).contains("\"ready\":true"));
        assert!(verify_export.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&verify_export).contains("[GARM-VERIFY-EXPORT] ok=true"));
        assert!(response_body(&api_verify_export).contains("algorithm=fnv64"));
        assert!(artifacts.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&artifacts).contains("[GARM-ARTIFACTS]"));
        assert!(response_body(&api_artifacts).contains("name=garm_export"));
        assert!(artifact_catalog.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&artifact_catalog).contains("eden-artifact-api-catalog-v1"));
        assert!(response_body(&artifact_catalog_alias).contains("memory_eval"));
        assert!(response_body(&artifact_runtime).contains("eden-artifact-api-runtime-v1"));
        assert!(memory_artifact.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&memory_artifact).contains("\"sample\":true"));
        assert!(unsafe_artifact.starts_with("HTTP/1.1 404 Not Found"));
        assert!(runtime_catalog.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&runtime_catalog).contains("eden-runtime-state-api-catalog-v1"));
        assert!(response_body(&runtime_openapi).contains("readRuntimeStateByName"));
        assert!(response_body(&runtime_snapshot).contains("eden-runtime-state-snapshot-v1"));
        assert!(response_body(&runtime_snapshot).contains("\"alive_nodes\": 12"));
        assert!(goal_state.starts_with("HTTP/1.1 200 OK"));
        assert!(response_body(&goal_state).contains("\"goals\":1"));
        assert!(unsafe_state.starts_with("HTTP/1.1 404 Not Found"));
        assert!(response_body(&capability_catalog).contains("eden-capabilities-catalog-v1"));
        assert!(response_body(&capability_status).contains("eden-capabilities-status-v1"));
        assert!(response_body(&gewc_handlers).contains("gewc_validation_body_handler"));
        assert!(response_body(&validation_status).contains("eden-validation-status-api-v1"));
        assert!(response_body(&operational_status).contains("eden-operational-status-v1"));
        assert!(response_body(&operational_status).contains("\"permissions_endpoint\""));
        assert!(response_body(&operational_contract).contains("eden-operational-contract-v1"));
        assert!(response_body(&operational_contract).contains("\"readiness\""));
        assert!(response_body(&operational_permissions).contains("eden-operational-permissions-v1"));
        assert!(response_body(&operational_permissions).contains("remote_network"));
        assert!(response_body(&operational_replay).contains("eden-gewc-replay-index-v1"));
        assert!(
            response_body(&operational_replay_decision).contains("eden-gewc-decision-replay-v1")
        );
        assert!(response_body(&operational_recovery).contains("eden-operational-recovery-plan-v1"));
        assert!(response_body(&operational_demos).contains("eden-operational-demo-suite-v1"));
        assert!(response_body(&operational_schemas).contains("eden-schema-registry-v1"));
        assert!(response_body(&operational_schema_record).contains("\"found\": true"));
        assert!(response_body(&action_contracts).contains("/api/command_sync?cmd=<command>"));
        assert!(response_body(&action_dry_run).contains("\"would_execute\": false"));
        assert!(response_body(&action_dry_run).contains("\"persistent_permission\""));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn serves_legacy_http_shapes_and_body_command() {
        let (port, queue, response_bus, metrics) = test_server();
        metrics.ready.store(true, Ordering::Relaxed);

        let health = http_get(port, "/health");
        assert!(health.starts_with("HTTP/1.1 200 OK"));
        assert_eq!(
            response_body(&health),
            "{\"status\":\"ok\",\"service\":\"eden\"}"
        );

        let api_health = http_get(port, "/api/health");
        assert_eq!(
            response_body(&api_health),
            "{\"status\":\"ok\",\"service\":\"eden\"}"
        );

        let status = http_get(port, "/api/status");
        let status_body = response_body(&status);
        assert!(status_body.contains("\"born\":true"));
        assert!(status_body.contains("\"autonomous\":"));
        assert!(status_body.contains("\"cycles\":"));

        let metrics_response = http_get(port, "/api/metrics");
        let metrics_body = response_body(&metrics_response);
        assert!(metrics_body.contains("\"cycles\":"));
        assert!(metrics_body.contains("\"phi\":"));
        assert!(metrics_body.contains("\"meltrace_grabados\":"));

        let post = http_post(port, "/api/command", "save");
        let queued_command = queue.lock().unwrap().pop().unwrap();
        assert_eq!(queued_command.raw, "save");
        response_bus.respond(queued_command.id.unwrap(), "saved\n".to_string());
        assert!(response_body(&post).contains("Session saved"));
    }

    #[test]
    fn queues_async_commands_and_retains_results_until_forget() {
        let (port, queue, response_bus, _metrics) = test_server();

        let queued = http_get(port, "/command?cmd=remember+alpha%20beta");
        assert!(queued.starts_with("HTTP/1.1 200 OK"));
        let body = response_body(&queued);
        assert!(body.starts_with("queued: 1 cmd: remember alpha beta"));

        let queued_command = queue.lock().unwrap().pop().unwrap();
        assert_eq!(queued_command.id, Some(1));
        assert_eq!(queued_command.raw, "remember alpha beta");

        let pending = http_get(port, "/api/command_result?id=1");
        assert!(pending.starts_with("HTTP/1.1 202 Accepted"));
        assert_eq!(response_body(&pending), "pending: 1\n");

        response_bus.respond(1, "stored alpha beta\n".to_string());
        let done = http_get(port, "/api/command_result?id=1");
        assert!(done.starts_with("HTTP/1.1 200 OK"));
        assert_eq!(response_body(&done), "stored alpha beta\n");

        let forgot = http_get(port, "/api/command_forget?id=1");
        assert!(forgot.starts_with("HTTP/1.1 200 OK"));
        assert_eq!(response_body(&forgot), "forgot: 1\n");

        let unknown = http_get(port, "/command_result?id=1");
        assert!(unknown.starts_with("HTTP/1.1 404 Not Found"));
        assert_eq!(response_body(&unknown), "unknown command id: 1\n");
    }

    #[test]
    fn rejects_sync_commands_until_runtime_is_ready() {
        let (port, queue, _response_bus, metrics) = test_server();
        metrics.ready.store(false, Ordering::Relaxed);

        let response = http_get(port, "/command_sync?cmd=status");
        assert!(response.starts_with("HTTP/1.1 503 Service Unavailable"));
        assert_eq!(response_body(&response), "runtime not ready\n");
        assert!(queue.lock().unwrap().is_empty());

        let api_response = http_get(port, "/api/command_sync?cmd=status");
        assert!(api_response.starts_with("HTTP/1.1 503 Service Unavailable"));
        assert_eq!(response_body(&api_response), "runtime not ready\n");
        assert!(queue.lock().unwrap().is_empty());
    }

    #[test]
    fn reports_bad_requests_and_unknown_routes() {
        let (port, _queue, _response_bus, _metrics) = test_server();

        let missing_cmd = http_get(port, "/command");
        assert!(missing_cmd.starts_with("HTTP/1.1 400 Bad Request"));
        assert_eq!(response_body(&missing_cmd), "missing cmd query parameter\n");

        let bad_result_id = http_get(port, "/command_result?id=abc");
        assert!(bad_result_id.starts_with("HTTP/1.1 400 Bad Request"));
        assert_eq!(
            response_body(&bad_result_id),
            "missing or invalid id query parameter\n"
        );

        let unknown = http_get(port, "/missing");
        assert!(unknown.starts_with("HTTP/1.1 404 Not Found"));
        assert_eq!(response_body(&unknown), "not found\n");
    }

    #[test]
    fn bounds_retained_command_results() {
        let (_port, _queue, response_bus, _metrics) = test_server();

        for _ in 0..(MAX_STORED_COMMAND_RESULTS as u64 + 2) {
            let id = response_bus.next_id();
            response_bus.respond(id, format!("done {}\n", id));
        }

        assert!(matches!(
            response_bus.take_result(1),
            CommandResultState::Unknown
        ));
        assert!(matches!(
            response_bus.take_result(2),
            CommandResultState::Unknown
        ));
        match response_bus.take_result(3) {
            CommandResultState::Done(body) => assert_eq!(body, "done 3\n"),
            _ => panic!("expected retained result 3"),
        }
        match response_bus.take_result(MAX_STORED_COMMAND_RESULTS as u64 + 2) {
            CommandResultState::Done(body) => {
                assert_eq!(
                    body,
                    format!("done {}\n", MAX_STORED_COMMAND_RESULTS as u64 + 2)
                );
            }
            _ => panic!("expected latest retained result"),
        }
    }
}
