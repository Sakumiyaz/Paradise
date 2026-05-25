use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct RuntimeStateSpec {
    pub name: &'static str,
    pub path: String,
    pub domain: &'static str,
    pub schema_hint: &'static str,
}

pub struct LiveRuntimeSnapshotInput<'a> {
    pub ready: bool,
    pub autonomous: bool,
    pub daemon_enabled: bool,
    pub uptime_sec: u64,
    pub tick_count: u64,
    pub evolution_level: u64,
    pub alive_nodes: u64,
    pub edge_count: u64,
    pub memory_facts: u64,
    pub api_requests: u64,
    pub organ_count: usize,
    pub organ_pending_actions: u64,
    pub organ_actions_executed: u64,
    pub organ_actions_blocked: u64,
    pub organ_autonomous_runs: u64,
    pub engine_status: &'a str,
}

pub fn state_specs() -> Vec<RuntimeStateSpec> {
    vec![
        state(
            "graph",
            state_paths::graph_state_path(),
            "core_runtime",
            "garm-graph-state",
        ),
        state(
            "capabilities",
            state_paths::capability_state_path(),
            "core_runtime",
            "garm-capability-state",
        ),
        state(
            "runtime",
            state_paths::runtime_state_path(),
            "core_runtime",
            "garm-runtime-state",
        ),
        state(
            "api_server",
            state_paths::api_server_state_path(),
            "api_runtime",
            "api-server-state",
        ),
        state(
            "daemon",
            state_paths::daemon_state_path(),
            "api_runtime",
            "daemon-state",
        ),
        state(
            "command_router",
            state_paths::command_router_state_path(),
            "api_runtime",
            "command-router-state",
        ),
        state(
            "telemetry",
            state_paths::telemetry_state_path(),
            "api_runtime",
            "telemetry-state",
        ),
        state(
            "persistence",
            state_paths::persistence_state_path(),
            "api_runtime",
            "persistence-state",
        ),
        state(
            "goal_scheduler",
            state_paths::goal_scheduler_state_path(),
            "planning_goal",
            "goal-scheduler-state",
        ),
        state(
            "evaluation_loop",
            state_paths::evaluation_loop_state_path(),
            "evaluation_validation",
            "evaluation-loop-state",
        ),
        state(
            "learning_ledger",
            state_paths::learning_ledger_state_path(),
            "safe_learning",
            "learning-ledger-state",
        ),
        state(
            "world_model_core",
            state_paths::world_model_core_state_path(),
            "world_model",
            "world-model-core-state",
        ),
        state(
            "competence_benchmark",
            state_paths::competence_benchmark_state_path(),
            "evaluation_validation",
            "competence-benchmark-state",
        ),
        state(
            "plan_executor",
            state_paths::plan_executor_state_path(),
            "planning_goal",
            "plan-executor-state",
        ),
        state(
            "working_memory",
            state_paths::working_memory_state_path(),
            "workspace_attention",
            "working-memory-state",
        ),
        state(
            "uncertainty_ledger",
            state_paths::uncertainty_ledger_state_path(),
            "metacognitive_safety",
            "uncertainty-ledger-state",
        ),
        state(
            "experiment_runner",
            state_paths::experiment_runner_state_path(),
            "experimentation",
            "experiment-runner-state",
        ),
        state(
            "provenance_ledger",
            state_paths::provenance_ledger_state_path(),
            "metacognitive_safety",
            "provenance-ledger-state",
        ),
        state(
            "policy_guard",
            state_paths::policy_guard_state_path(),
            "metacognitive_safety",
            "policy-guard-state",
        ),
        state(
            "capability_maturity",
            state_paths::capability_maturity_state_path(),
            "evaluation_validation",
            "capability-maturity-state",
        ),
        state(
            "organ_autonomy",
            state_paths::organ_autonomy_state_path(),
            "agentic",
            "organ-autonomy-state",
        ),
        state(
            "context_augmentation",
            state_paths::context_augmentation_state_path(),
            "tool_adapter",
            "context-augmentation-state",
        ),
        state(
            "conscious_graph_regulator",
            state_paths::conscious_graph_regulator_state_path(),
            "core_runtime",
            "conscious-graph-regulator-state",
        ),
        state(
            "coordinator",
            state_paths::coordinator_state_path(),
            "core_runtime",
            "coordinator-state",
        ),
        state(
            "meta_architect",
            state_paths::meta_architect_state_path(),
            "core_runtime",
            "meta-architect-state",
        ),
        state(
            "fast_reflexes",
            state_paths::fast_reflexes_state_path(),
            "core_runtime",
            "fast-reflexes-state",
        ),
        state(
            "human_interface",
            state_paths::human_interface_state_path(),
            "human_interface",
            "human-interface-state",
        ),
        state(
            "hrm_reasoner",
            state_paths::hrm_reasoner_state_path(),
            "specialized_model",
            "hrm-reasoner-state",
        ),
        state(
            "hrm_text_pretraining",
            state_paths::hrm_text_pretraining_state_path(),
            "specialized_model",
            "hrm-text-pretraining-state",
        ),
        state(
            "hrm_text_segments",
            state_paths::hrm_text_segments_path(),
            "specialized_model",
            "hrm-text-segments-jsonl",
        ),
        state(
            "hrm_text_context_pack",
            state_paths::hrm_text_context_pack_path(),
            "specialized_model",
            "hrm-text-context-pack",
        ),
        state(
            "hrm_text_checkpoint_manifest",
            state_paths::hrm_text_checkpoint_manifest_path(),
            "specialized_model",
            "hrm-text-checkpoint-manifest",
        ),
        state(
            "hrm_text_corpus_manifest",
            state_paths::hrm_text_corpus_manifest_path(),
            "specialized_model",
            "hrm-text-corpus-manifest",
        ),
        state(
            "voice_synthesizer",
            state_paths::voice_synthesizer_state_path(),
            "specialized_model",
            "voice-synthesizer-state",
        ),
        state(
            "hybrid_voice",
            state_paths::hybrid_voice_state_path(),
            "specialized_model",
            "hybrid-voice-state",
        ),
        state(
            "voice_last",
            state_paths::voice_last_artifact_path(),
            "specialized_model",
            "voice-last-text",
        ),
        state(
            "global_executive_workspace_runtime",
            state_paths::global_executive_workspace_runtime_path(),
            "global_executive_workspace",
            "gewc-runtime-jsonl",
        ),
        state(
            "global_executive_workspace_runtime_state",
            state_paths::global_executive_workspace_runtime_state_path(),
            "global_executive_workspace",
            "gewc-runtime-state",
        ),
        state(
            "garm_report",
            state_paths::garm_report_path(),
            "reporting",
            "garm-report-text",
        ),
        state(
            "garm_report_history",
            state_paths::garm_report_history_path(),
            "reporting",
            "garm-report-history-jsonl",
        ),
        state(
            "garm_export",
            state_paths::garm_export_path(),
            "reporting",
            "garm-export-json",
        ),
        state(
            "operational_runtime_phase",
            state_paths::operational_runtime_phase_path(),
            "operational_runtime",
            "eden-operational-runtime-phase-v1",
        ),
        state(
            "operational_task_runtime",
            state_paths::operational_task_runtime_path(),
            "operational_runtime",
            "eden-operational-task-runtime-v1",
        ),
        state(
            "operational_action_executor",
            state_paths::operational_action_executor_path(),
            "operational_runtime",
            "eden-operational-action-executor-v1",
        ),
        state(
            "operational_lifecycle_controls",
            state_paths::operational_lifecycle_controls_path(),
            "operational_runtime",
            "eden-operational-lifecycle-controls-v1",
        ),
        state(
            "operational_memory_transactions",
            state_paths::operational_memory_transactions_path(),
            "operational_runtime",
            "eden-memory-transaction-layer-v1",
        ),
        state(
            "cwm_operational_state",
            state_paths::cwm_operational_state_path(),
            "operational_runtime",
            "eden-cwm-operational-state-v1",
        ),
        state(
            "locus_operator_bridge",
            state_paths::locus_operator_bridge_path(),
            "operational_runtime",
            "eden-locus-operator-bridge-v1",
        ),
        state(
            "governed_agent_runtime",
            state_paths::governed_agent_runtime_path(),
            "operational_runtime",
            "eden-governed-agent-runtime-v1",
        ),
        state(
            "operational_replay_eval",
            state_paths::operational_replay_eval_path(),
            "operational_runtime",
            "eden-operational-replay-eval-v1",
        ),
        state(
            "paradise_worldcell_sessions",
            state_paths::paradise_worldcell_sessions_path(),
            "paradise_worldcell",
            "eden-paradise-worldcell-session-v1",
        ),
        state(
            "runtime_spine",
            state_paths::runtime_spine_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::RUNTIME_SPINE_SCHEMA,
        ),
        state(
            "runtime_internal_contracts",
            state_paths::runtime_internal_contracts_path(),
            "runtime_spine",
            "eden-runtime-internal-contracts-v1",
        ),
        state(
            "runtime_event_bus",
            state_paths::runtime_event_bus_path(),
            "runtime_spine",
            "runtime-event-bus-jsonl",
        ),
        state(
            "runtime_event_bus_state",
            state_paths::runtime_event_bus_state_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::EVENT_BUS_SCHEMA,
        ),
        state(
            "runtime_global_state",
            state_paths::runtime_global_state_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::GLOBAL_STATE_SCHEMA,
        ),
        state(
            "runtime_global_state_log",
            state_paths::runtime_global_state_log_path(),
            "runtime_spine",
            "runtime-global-state-log-jsonl",
        ),
        state(
            "runtime_replay_spine",
            state_paths::runtime_replay_spine_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::REPLAY_SPINE_SCHEMA,
        ),
        state(
            "runtime_spine_verification",
            state_paths::runtime_spine_verification_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::SPINE_VERIFICATION_SCHEMA,
        ),
        state(
            "runtime_guard_decisions",
            state_paths::runtime_guard_decisions_path(),
            "runtime_spine",
            "eden-runtime-guard-decision-jsonl",
        ),
        state(
            "runtime_spine_enforcement",
            state_paths::runtime_spine_enforcement_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::ENFORCEMENT_SCHEMA,
        ),
        state(
            "runtime_workflow_risk",
            state_paths::runtime_workflow_risk_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::WORKFLOW_RISK_SCHEMA,
        ),
        state(
            "runtime_circuit_breakers",
            state_paths::runtime_circuit_breakers_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::CIRCUIT_BREAKERS_SCHEMA,
        ),
        state(
            "runtime_replay_reconstruction",
            state_paths::runtime_replay_reconstruction_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::REPLAY_RECONSTRUCTION_SCHEMA,
        ),
        state(
            "runtime_security_gates",
            state_paths::runtime_security_gates_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::SECURITY_GATES_SCHEMA,
        ),
        state(
            "runtime_model_router_contract",
            state_paths::runtime_model_router_contract_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::MODEL_ROUTER_SCHEMA,
        ),
        state(
            "runtime_memory_fabric_contract",
            state_paths::runtime_memory_fabric_contract_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::MEMORY_FABRIC_SCHEMA,
        ),
        state(
            "runtime_world_simulation_contract",
            state_paths::runtime_world_simulation_contract_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::WORLD_SIMULATION_SCHEMA,
        ),
        state(
            "runtime_multiagent_contract",
            state_paths::runtime_multiagent_contract_path(),
            "runtime_spine",
            crate::eden_garm::runtime_spine::MULTIAGENT_CONTRACT_SCHEMA,
        ),
        state(
            "eden_locus_layer",
            state_paths::eden_locus_layer_path(),
            "locus_context_authority",
            "eden-locus-layer-v1",
        ),
        state(
            "locus_authority_model",
            state_paths::locus_authority_model_path(),
            "locus_context_authority",
            "eden-locus-authority-model-v1",
        ),
        state(
            "locus_evidence_vault",
            state_paths::locus_evidence_vault_path(),
            "locus_context_authority",
            "eden-locus-evidence-vault-v1",
        ),
        state(
            "locus_permission_matrix",
            state_paths::locus_permission_matrix_path(),
            "locus_context_authority",
            "eden-locus-permission-matrix-v1",
        ),
        state(
            "locus_context_packet",
            state_paths::locus_context_packet_path(),
            "locus_context_authority",
            "eden-locus-context-packet-v1",
        ),
        state(
            "locus_operator_timeline",
            state_paths::locus_operator_timeline_path(),
            "locus_context_authority",
            "eden-locus-operator-timeline-jsonl",
        ),
        state(
            "eden_operator_forge",
            state_paths::eden_operator_forge_path(),
            "formal_synthesis",
            "eden-operator-forge-v1",
        ),
        state(
            "operator_primitive_basis",
            state_paths::operator_primitive_basis_path(),
            "formal_synthesis",
            "eden-operator-primitive-basis-v1",
        ),
        state(
            "operator_expression_graphs",
            state_paths::operator_expression_graphs_path(),
            "formal_synthesis",
            "eden-operator-expression-graphs-jsonl",
        ),
        state(
            "operator_verification_report",
            state_paths::operator_verification_report_path(),
            "formal_synthesis",
            "eden-operator-verification-report-v1",
        ),
        state(
            "operator_model_registry",
            state_paths::operator_model_registry_path(),
            "formal_synthesis",
            "eden-operator-model-registry-v1",
        ),
        state(
            "operational_contract",
            state_paths::operational_contract_path(),
            "operational_runtime",
            "eden-operational-contract-v1",
        ),
        state(
            "operational_permissions",
            state_paths::operational_permissions_path(),
            "operational_runtime",
            "eden-operational-permissions-v1",
        ),
        state(
            "operational_permissions_audit",
            state_paths::operational_permissions_audit_path(),
            "operational_runtime",
            "eden-operational-permissions-audit-v1",
        ),
        state(
            "operational_permissions_diff",
            state_paths::operational_permissions_diff_path(),
            "operational_runtime",
            "eden-operational-permissions-diff-v1",
        ),
        state(
            "operational_permissions_history",
            state_paths::operational_permissions_history_path(),
            "operational_runtime",
            "eden-operational-permissions-history-v1",
        ),
        state(
            "operational_evidence_bundle",
            state_paths::operational_evidence_bundle_path(),
            "operational_runtime",
            "eden-operational-evidence-bundle-v1",
        ),
        state(
            "operational_recovery_plan",
            state_paths::operational_recovery_plan_path(),
            "operational_runtime",
            "eden-operational-recovery-plan-v1",
        ),
        state(
            "operational_demo_suite",
            state_paths::operational_demo_suite_path(),
            "operational_runtime",
            "eden-operational-demo-suite-v1",
        ),
        state(
            "schema_registry",
            state_paths::schema_registry_path(),
            "api_runtime",
            "eden-schema-registry-v1",
        ),
        state(
            "operational_smoke_test",
            state_paths::operational_smoke_test_path(),
            "operational_runtime",
            "eden-operational-smoke-test-v1",
        ),
        state(
            "operational_e2e_scenario",
            state_paths::operational_e2e_scenario_path(),
            "operational_runtime",
            "eden-operational-e2e-scenario-v1",
        ),
        state(
            "action_evidence",
            state_paths::action_evidence_path(),
            "metacognitive_safety",
            "action-evidence-jsonl",
        ),
    ]
}

pub fn run() -> String {
    let specs = state_specs();
    write_json(
        state_paths::runtime_state_api_contracts_path(),
        contracts_value(&specs),
    );
    write_json(
        state_paths::runtime_state_api_openapi_path(),
        openapi_value(&specs),
    );
    write_json(
        state_paths::runtime_state_api_catalog_path(),
        catalog_value(&specs),
    );
    write_json(
        state_paths::runtime_state_api_runtime_path(),
        runtime_value(&specs),
    );
    let catalog = catalog_value(&specs);
    let present = catalog
        .get("present")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let total = catalog
        .get("total")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    format!(
        "[RUNTIME-STATE-API] states={}/{} endpoints=4 claim_allowed=false path={}\n[RUNTIME-STATE-API-CONTRACTS] path={}\n[RUNTIME-STATE-API-OPENAPI] path={}\n[RUNTIME-STATE-API-RUNTIME] path={}\n",
        present,
        total,
        state_paths::runtime_state_api_catalog_path(),
        state_paths::runtime_state_api_contracts_path(),
        state_paths::runtime_state_api_openapi_path(),
        state_paths::runtime_state_api_runtime_path()
    )
}

pub fn catalog_json() -> String {
    std::fs::read_to_string(state_paths::runtime_state_api_catalog_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&catalog_value(&state_specs()))
            .unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn runtime_json() -> String {
    std::fs::read_to_string(state_paths::runtime_state_api_runtime_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&runtime_value(&state_specs()))
            .unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn openapi_json() -> String {
    std::fs::read_to_string(state_paths::runtime_state_api_openapi_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&openapi_value(&state_specs()))
            .unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn read_state(name: &str) -> Option<(String, &'static str)> {
    let spec = state_specs().into_iter().find(|state| state.name == name)?;
    let body = std::fs::read_to_string(&spec.path).ok()?;
    Some((body, content_type_for(&spec.path)))
}

pub fn live_snapshot_json(input: LiveRuntimeSnapshotInput<'_>) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "eden-runtime-state-snapshot-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "permissions": {
            "read_only": true,
            "mutates_runtime": false,
            "command_execution": false
        },
        "runtime": {
            "ready": input.ready,
            "autonomous": input.autonomous,
            "daemon_enabled": input.daemon_enabled,
            "uptime_sec": input.uptime_sec,
            "tick_count": input.tick_count,
            "evolution_level": input.evolution_level,
            "alive_nodes": input.alive_nodes,
            "edge_count": input.edge_count,
            "memory_facts": input.memory_facts,
            "api_requests": input.api_requests,
            "engine_status": input.engine_status
        },
        "organs": {
            "count": input.organ_count,
            "pending_actions": input.organ_pending_actions,
            "actions_executed": input.organ_actions_executed,
            "actions_blocked": input.organ_actions_blocked,
            "autonomous_runs": input.organ_autonomous_runs
        },
        "routes": [
            "/api/runtime/catalog",
            "/api/runtime/state?name=<state_name>",
            "/api/runtime/snapshot",
            "/api/runtime/openapi"
        ]
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

fn catalog_value(specs: &[RuntimeStateSpec]) -> serde_json::Value {
    let records: Vec<_> = specs.iter().map(record_for).collect();
    let present = records
        .iter()
        .filter(|record| record.get("present").and_then(serde_json::Value::as_bool) == Some(true))
        .count();
    serde_json::json!({
        "schema": "eden-runtime-state-api-catalog-v1",
        "artifact": "runtime_state_api_catalog",
        "claim_allowed": false,
        "agi_claim": false,
        "purpose": "Expose live runtime state-management surfaces as typed read-only APIs.",
        "base_routes": [
            "/api/runtime/catalog",
            "/api/runtime/state?name=<state_name>",
            "/api/runtime/snapshot",
            "/api/runtime/openapi"
        ],
        "present": present,
        "total": records.len(),
        "records": records,
    })
}

fn contracts_value(specs: &[RuntimeStateSpec]) -> serde_json::Value {
    let contracts: Vec<_> = specs
        .iter()
        .map(|spec| {
            serde_json::json!({
                "state": spec.name,
                "domain": spec.domain,
                "schema_hint": spec.schema_hint,
                "api": {
                    "read": format!("/api/runtime/state?name={}", spec.name),
                    "inspect": "/api/runtime/catalog",
                    "snapshot": "/api/runtime/snapshot",
                    "openapi": "/api/runtime/openapi"
                },
                "permission_contract": {
                    "read_only": true,
                    "mutates_runtime": false,
                    "command_execution": false,
                    "path_whitelist": "runtime_state_api_state_specs",
                    "claim_allowed": false,
                    "agi_claim": false
                }
            })
        })
        .collect();
    serde_json::json!({
        "schema": "eden-runtime-state-api-contracts-v1",
        "artifact": "runtime_state_api_contracts",
        "claim_allowed": false,
        "agi_claim": false,
        "contracts": contracts,
    })
}

fn openapi_value(specs: &[RuntimeStateSpec]) -> serde_json::Value {
    serde_json::json!({
        "schema": "eden-runtime-state-openapi-v1",
        "artifact": "runtime_state_api_openapi",
        "claim_allowed": false,
        "agi_claim": false,
        "openapi": "3.0.3",
        "info": {
            "title": "EDEN Runtime State API",
            "version": "v1",
            "description": "Read-only typed runtime state-management API surface."
        },
        "paths": {
            "/api/runtime/catalog": {
                "get": {
                    "operationId": "listRuntimeStateSurfaces",
                    "x-eden-permission": "runtime_state_read",
                    "responses": {"200": {"description": "Runtime state catalog"}}
                }
            },
            "/api/runtime/state": {
                "get": {
                    "operationId": "readRuntimeStateByName",
                    "x-eden-permission": "runtime_state_read",
                    "parameters": [{"name": "name", "in": "query", "required": true, "schema": {"type": "string", "enum": specs.iter().map(|spec| spec.name).collect::<Vec<_>>()}}],
                    "responses": {
                        "200": {"description": "Whitelisted runtime state body"},
                        "404": {"description": "Unknown or missing runtime state"}
                    }
                }
            },
            "/api/runtime/snapshot": {
                "get": {
                    "operationId": "readLiveRuntimeSnapshot",
                    "x-eden-permission": "runtime_state_read",
                    "responses": {"200": {"description": "Live runtime metrics snapshot"}}
                }
            },
            "/api/runtime/openapi": {
                "get": {
                    "operationId": "readRuntimeStateOpenApi",
                    "x-eden-permission": "runtime_state_read",
                    "responses": {"200": {"description": "Runtime State API OpenAPI document"}}
                }
            }
        }
    })
}

fn runtime_value(specs: &[RuntimeStateSpec]) -> serde_json::Value {
    let mut present = 0usize;
    let mut missing = 0usize;
    let mut total_bytes = 0usize;
    for spec in specs {
        let bytes = std::fs::read(&spec.path).unwrap_or_default();
        if bytes.is_empty() {
            missing += 1;
        } else {
            present += 1;
            total_bytes += bytes.len();
        }
    }
    serde_json::json!({
        "schema": "eden-runtime-state-api-runtime-v1",
        "artifact": "runtime_state_api_runtime",
        "claim_allowed": false,
        "agi_claim": false,
        "present": present,
        "missing": missing,
        "total": specs.len(),
        "total_bytes": total_bytes,
        "routes": [
            {"method": "GET", "path": "/api/runtime/catalog", "effect": "read_catalog"},
            {"method": "GET", "path": "/api/runtime/state?name=<state_name>", "effect": "read_whitelisted_runtime_state"},
            {"method": "GET", "path": "/api/runtime/snapshot", "effect": "read_live_snapshot"},
            {"method": "GET", "path": "/api/runtime/openapi", "effect": "read_openapi_contract"}
        ],
        "write_policy": "runtime state APIs are read-only; mutation remains command-routed through GEWC policy and safety checks",
    })
}

fn record_for(spec: &RuntimeStateSpec) -> serde_json::Value {
    let bytes = std::fs::read(&spec.path).unwrap_or_default();
    serde_json::json!({
        "name": spec.name,
        "path": spec.path.as_str(),
        "domain": spec.domain,
        "schema_hint": spec.schema_hint,
        "present": !bytes.is_empty(),
        "bytes": bytes.len(),
        "fnv64": format!("{:016x}", fnv64(&bytes)),
        "content_type": content_type_for(&spec.path),
        "read_endpoint": format!("/api/runtime/state?name={}", spec.name),
        "permission": "runtime_state_read",
        "read_only": true,
    })
}

fn state(
    name: &'static str,
    path: String,
    domain: &'static str,
    schema_hint: &'static str,
) -> RuntimeStateSpec {
    RuntimeStateSpec {
        name,
        path,
        domain,
        schema_hint,
    }
}

fn content_type_for(path: &str) -> &'static str {
    if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".jsonl") {
        "application/x-ndjson"
    } else {
        "text/plain"
    }
}

fn write_json(path: String, record: serde_json::Value) {
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_state_api_writes_catalog_contracts_openapi_and_runtime() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_runtime_state_api_test_{}",
            std::process::id()
        )));
        let _ = state_paths::ensure_state_dir();
        std::fs::write(state_paths::graph_state_path(), "{\"nodes\":1}").unwrap();
        std::fs::write(state_paths::goal_scheduler_state_path(), "{\"goals\":1}").unwrap();

        let out = run();

        assert!(out.contains("[RUNTIME-STATE-API]"));
        assert!(std::fs::metadata(state_paths::runtime_state_api_catalog_path()).is_ok());
        assert!(std::fs::metadata(state_paths::runtime_state_api_contracts_path()).is_ok());
        assert!(std::fs::metadata(state_paths::runtime_state_api_openapi_path()).is_ok());
        assert!(std::fs::metadata(state_paths::runtime_state_api_runtime_path()).is_ok());
        let catalog = catalog_json();
        assert!(catalog.contains("\"read_endpoint\""));
        assert!(catalog.contains("goal_scheduler"));
        assert!(catalog.contains("operational_contract"));
        assert!(catalog.contains("locus_operator_bridge"));
        assert!(openapi_json().contains("readRuntimeStateByName"));
        assert!(read_state("graph").is_some());
        assert!(read_state("../secret").is_none());
        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
