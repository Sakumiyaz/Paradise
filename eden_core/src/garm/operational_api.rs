use crate::eden_garm::capabilities::GarmCapabilityState;
use crate::eden_garm::global_executive_workspace::{
    GewcBodyHandler, GewcBodyRegistry, GlobalExecutiveWorkspaceCore,
};
use crate::eden_garm::nodes::command_router::{CommandRouterNode, GarmCommand};
use crate::eden_garm::{operational_runtime, runtime_spine, schema_registry, state_paths};

pub struct OperationalStatusInput<'a> {
    pub ready: bool,
    pub autonomous: bool,
    pub daemon_enabled: bool,
    pub uptime_sec: u64,
    pub alive_nodes: u64,
    pub edge_count: u64,
    pub memory_facts: u64,
    pub api_requests: u64,
    pub engine_status: &'a str,
    pub capability_count: usize,
    pub tick_count: u64,
    pub idle_ticks: u64,
}

const ROUTES: &[(&str, &str, &str, bool)] = &[
    (
        "/api/operational/catalog",
        "operational_catalog",
        "List operational read surfaces and action contracts.",
        false,
    ),
    (
        "/api/operational/openapi",
        "operational_openapi",
        "Expose the operational API OpenAPI-style document.",
        false,
    ),
    (
        "/api/operational/runtime",
        "operational_runtime",
        "Expose runtime status for the operational API surface.",
        false,
    ),
    (
        "/api/operational/status",
        "operational_status",
        "Expose live health, readiness, degraded state, latest GEWC decision, replay and permission summary.",
        false,
    ),
    (
        "/api/paradise/sessions",
        "paradise_worldcell_sessions",
        "Expose Paradise Worldcell intent, plan, approval, execution and evidence sessions.",
        false,
    ),
    (
        "/api/paradise/worldcell",
        "paradise_worldcell_runtime",
        "Expose the Paradise Worldcell runtime identity artifact when generated.",
        false,
    ),
    (
        "/api/operational/contract",
        "operational_contract",
        "Expose the stable health, readiness, degradation, shutdown and action-boundary contract.",
        false,
    ),
    (
        "/api/operational/permissions",
        "operational_permissions",
        "Expose persistent local capability permissions used by GEWC gates and dry-run classification.",
        false,
    ),
    (
        "/api/operational/replay",
        "operational_replay",
        "List replayable GEWC decisions or read one decision by decision_id.",
        false,
    ),
    (
        "/api/operational/recovery",
        "operational_recovery",
        "Expose degraded-mode recovery plan and handler restoration evidence.",
        false,
    ),
    (
        "/api/operational/demos",
        "operational_demos",
        "Expose reproducible operational demo-suite evidence.",
        false,
    ),
    (
        "/api/operational/schemas",
        "operational_schema_registry",
        "Expose stable runtime schema contracts and generated artifact schema hints.",
        false,
    ),
    (
        "/api/operational/schema?name=<schema_or_artifact>",
        "operational_schema_record",
        "Read one schema registry record by schema or artifact name.",
        false,
    ),
    (
        "/api/operational/runtime-phase",
        "operational_runtime_phase",
        "Read the eight-component operational runtime phase artifact.",
        false,
    ),
    (
        "/api/operational/spine",
        "runtime_spine",
        "Read the GEWC-owned runtime spine contract that unifies messages, state, replay, safety, model routing, memory, simulation and multiagent coordination.",
        false,
    ),
    (
        "/api/operational/events",
        "runtime_event_bus_state",
        "Read the append-only runtime event bus state and latest internal messages.",
        false,
    ),
    (
        "/api/operational/global-state",
        "runtime_global_state",
        "Read the append-only global-state snapshot and mutation head.",
        false,
    ),
    (
        "/api/operational/replay-spine",
        "runtime_replay_spine",
        "Read the unified replay spine built from GEWC decisions, events and state mutations.",
        false,
    ),
    (
        "/api/operational/spine-verification",
        "runtime_spine_verification",
        "Read the latest executable Runtime Spine verification report.",
        false,
    ),
    (
        "/api/operational/spine-enforcement",
        "runtime_spine_enforcement",
        "Read the mandatory Runtime Spine enforcement guard report.",
        false,
    ),
    (
        "/api/operational/workflow-risk",
        "runtime_workflow_risk",
        "Read workflow-level risk analysis across event and state chains.",
        false,
    ),
    (
        "/api/operational/circuit-breakers",
        "runtime_circuit_breakers",
        "Read Runtime Spine circuit breaker health and degradation policy.",
        false,
    ),
    (
        "/api/operational/spine-replay",
        "runtime_replay_reconstruction",
        "Read reconstructive replay assembled from GEWC traces, events, state mutations and action evidence.",
        false,
    ),
    (
        "/api/capabilities/catalog",
        "capabilities_catalog",
        "List current native GARM capabilities from the running engine.",
        false,
    ),
    (
        "/api/capabilities/status",
        "capabilities_status",
        "Read current capability and runtime health summary.",
        false,
    ),
    (
        "/api/gewc/runtime",
        "gewc_runtime",
        "Read GEWC runtime decisions, completions and handler metrics.",
        false,
    ),
    (
        "/api/gewc/handlers",
        "gewc_handlers",
        "List GEWC body handlers and execution domains.",
        false,
    ),
    (
        "/api/validation/status",
        "validation_status",
        "Read validation artifact presence and no-claim status.",
        false,
    ),
    (
        "/api/actions/contracts",
        "action_contracts",
        "Read action-route permission contracts.",
        false,
    ),
    (
        "/api/actions/dry-run?cmd=<command>",
        "action_dry_run",
        "Classify a command without queueing or executing it.",
        false,
    ),
];

pub fn run() -> String {
    let catalog = catalog_value();
    let contracts = contracts_value();
    let openapi = openapi_value();
    let action_contracts = action_contracts_value();
    let operational_contract = operational_contract_value();
    let permissions = permissions_value();
    write_json(state_paths::operational_api_catalog_path(), catalog);
    write_json(state_paths::operational_api_contracts_path(), contracts);
    write_json(state_paths::operational_api_openapi_path(), openapi);
    write_json(
        state_paths::operational_action_contracts_path(),
        action_contracts,
    );
    write_json(
        state_paths::operational_contract_path(),
        operational_contract,
    );
    write_json(state_paths::operational_permissions_path(), permissions);
    let schema_registry_report = schema_registry::run();
    write_json(state_paths::operational_api_runtime_path(), runtime_value());
    format!(
        "[OPERATIONAL-API] surfaces={} endpoints={} action_mutation_allowed=false claim_allowed=false path={}\n[OPERATIONAL-API-CONTRACTS] path={}\n[OPERATIONAL-API-OPENAPI] path={}\n[OPERATIONAL-ACTION-CONTRACTS] path={}\n[OPERATIONAL-CONTRACT] path={}\n[OPERATIONAL-PERMISSIONS] path={}\n{}[OPERATIONAL-API-RUNTIME] path={}\n",
        ROUTES.len(),
        ROUTES.len(),
        state_paths::operational_api_catalog_path(),
        state_paths::operational_api_contracts_path(),
        state_paths::operational_api_openapi_path(),
        state_paths::operational_action_contracts_path(),
        state_paths::operational_contract_path(),
        state_paths::operational_permissions_path(),
        schema_registry_report,
        state_paths::operational_api_runtime_path(),
    )
}

pub fn catalog_json() -> String {
    std::fs::read_to_string(state_paths::operational_api_catalog_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&catalog_value()).unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn openapi_json() -> String {
    std::fs::read_to_string(state_paths::operational_api_openapi_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&openapi_value()).unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn runtime_json() -> String {
    std::fs::read_to_string(state_paths::operational_api_runtime_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&runtime_value()).unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn status_json(input: OperationalStatusInput<'_>) -> String {
    let handler_states = handler_state_records();
    let degraded_handlers: Vec<_> = handler_states
        .iter()
        .filter(|record| record.get("state").and_then(serde_json::Value::as_str) != Some("active"))
        .cloned()
        .collect();
    let last_decision = latest_record_with_phase("decision_started");
    let last_completion = latest_record_with_phase("execution_completed");
    let last_block = latest_blocking_record();
    let latest_decision_id = last_decision
        .as_ref()
        .map(decision_id_for_record)
        .unwrap_or_else(|| "none".to_string());
    let readiness_state = if !input.ready {
        "starting"
    } else if !degraded_handlers.is_empty() || last_block.is_some() {
        "degraded"
    } else {
        "ready"
    };
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "eden-operational-status-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "global_executive_workspace_core",
        "state": readiness_state,
        "health": {
            "process_http": true,
            "ready": input.ready,
            "autonomous": input.autonomous,
            "daemon_enabled": input.daemon_enabled,
            "uptime_sec": input.uptime_sec
        },
        "runtime": {
            "alive_nodes": input.alive_nodes,
            "edge_count": input.edge_count,
            "memory_facts": input.memory_facts,
            "api_requests": input.api_requests,
            "tick_count": input.tick_count,
            "idle_ticks": input.idle_ticks,
            "capability_count": input.capability_count,
            "engine_status": input.engine_status
        },
        "degraded": {
            "active": readiness_state == "degraded",
            "handlers": degraded_handlers,
            "all_handlers": handler_states
        },
        "latest": {
            "decision_id": latest_decision_id,
            "decision": last_decision,
            "completion": last_completion,
            "block": last_block,
            "replay_endpoint": "/api/operational/replay?decision_id=<id>",
            "permissions_endpoint": "/api/operational/permissions"
        },
        "permissions": permission_summary_value(),
        "paths": {
            "gewc_runtime_log": state_paths::global_executive_workspace_runtime_path(),
            "permissions": state_paths::operational_permissions_path(),
            "action_evidence": state_paths::action_evidence_path()
        }
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

pub fn paradise_sessions_json() -> String {
    crate::eden_garm::paradise_worldcell::sessions_json()
}

pub fn paradise_worldcell_json() -> String {
    std::fs::read_to_string(state_paths::paradise_worldcell_runtime_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": crate::eden_garm::paradise_worldcell::PARADISE_WORLDCELL_SCHEMA,
            "artifact": "paradise_worldcell_runtime",
            "name": "Paradise",
            "present": false,
            "claim_allowed": false,
            "agi_claim": false,
            "generator_command": "paradise worldcell eval"
        }))
        .unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn runtime_spine_json() -> String {
    runtime_spine::spine_json()
}

pub fn runtime_events_json() -> String {
    runtime_spine::event_bus_json()
}

pub fn runtime_global_state_json() -> String {
    runtime_spine::global_state_json()
}

pub fn runtime_replay_spine_json() -> String {
    runtime_spine::replay_spine_json()
}

pub fn runtime_spine_verification_json() -> String {
    runtime_spine::spine_verification_json()
}

pub fn runtime_spine_enforcement_json() -> String {
    runtime_spine::enforcement_json()
}

pub fn runtime_workflow_risk_json() -> String {
    runtime_spine::workflow_risk_json()
}

pub fn runtime_circuit_breakers_json() -> String {
    runtime_spine::circuit_breakers_json()
}

pub fn runtime_spine_replay_json() -> String {
    runtime_spine::replay_reconstruction_json()
}

pub fn runtime_phase_json() -> String {
    operational_runtime::runtime_phase_json()
}

pub fn contract_json() -> String {
    std::fs::read_to_string(state_paths::operational_contract_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&operational_contract_value())
            .unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn permissions_json() -> String {
    std::fs::read_to_string(state_paths::operational_permissions_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&permissions_value()).unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn replay_index_json() -> String {
    GlobalExecutiveWorkspaceCore::replay_index_json(32)
}

pub fn replay_decision_json(decision_id: &str) -> String {
    GlobalExecutiveWorkspaceCore::replay_decision_json(decision_id)
}

pub fn recovery_json() -> String {
    operational_runtime::recovery_plan_json()
}

pub fn demos_json() -> String {
    operational_runtime::demo_suite_json()
}

pub fn schema_registry_json() -> String {
    schema_registry::catalog_json()
}

pub fn schema_record_json(name: &str) -> String {
    schema_registry::schema_json(name)
}

pub fn action_contracts_json() -> String {
    std::fs::read_to_string(state_paths::operational_action_contracts_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&action_contracts_value()).unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn capabilities_catalog_json(engine: &GarmCapabilityState) -> String {
    let records: Vec<_> = engine
        .state
        .capabilities
        .iter()
        .enumerate()
        .map(|(index, capability)| {
            serde_json::json!({
                "index": index,
                "name": format!("{:?}", capability),
                "source": "GarmCapabilityState.state.capabilities",
                "read_only": true,
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "eden-capabilities-catalog-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "total": records.len(),
        "records": records,
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

pub fn capabilities_status_json(input: OperationalStatusInput<'_>) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "eden-capabilities-status-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "runtime": {
            "ready": input.ready,
            "autonomous": input.autonomous,
            "daemon_enabled": input.daemon_enabled,
            "uptime_sec": input.uptime_sec,
            "alive_nodes": input.alive_nodes,
            "edge_count": input.edge_count,
            "memory_facts": input.memory_facts,
            "api_requests": input.api_requests,
            "tick_count": input.tick_count,
            "idle_ticks": input.idle_ticks
        },
        "capabilities": {
            "count": input.capability_count,
            "engine_status": input.engine_status
        },
        "permissions": read_only_permissions(),
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

pub fn gewc_runtime_json() -> String {
    let runtime_report = GlobalExecutiveWorkspaceCore::runtime_report();
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "eden-gewc-runtime-api-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "runtime_report": runtime_report,
        "runtime_log_path": state_paths::global_executive_workspace_runtime_path(),
        "runtime_state_path": state_paths::global_executive_workspace_runtime_state_path(),
        "permissions": read_only_permissions(),
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

pub fn gewc_handlers_json() -> String {
    let records: Vec<_> = GewcBodyHandler::ALL
        .iter()
        .map(|handler| {
            let lifecycle = GlobalExecutiveWorkspaceCore::lifecycle_control_for_handler(*handler);
            serde_json::json!({
                "handler": handler.as_str(),
                "native_executor_required": true,
                "selected_by": "GewcBodyRegistry",
                "lifecycle_supervisor": "gewc_module_lifecycle_supervisor",
                "lifecycle_state": lifecycle.state.as_str(),
                "lifecycle_allowed_actions": lifecycle.action_names(),
                "lifecycle_policy_gate": lifecycle.policy_gate,
                "lifecycle_isolation_scope": lifecycle.isolation_scope,
                "read_endpoint": "/api/gewc/handlers",
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "eden-gewc-handlers-api-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "total": records.len(),
        "handlers": records,
        "permissions": read_only_permissions(),
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

pub fn validation_status_json() -> String {
    let records = [
        validation_record(
            "external_validation_result",
            state_paths::external_validation_result_path(),
            "garm-external-validation-result-v1",
        ),
        validation_record(
            "capability_registry",
            state_paths::capability_registry_path(),
            "garm-capability-registry-v1",
        ),
        validation_record(
            "readiness_package",
            state_paths::readiness_package_path(),
            "garm-readiness-package-v1",
        ),
        validation_record(
            "independent_validation_report",
            state_paths::state_dir()
                .join("independent_validation_report.json")
                .to_string_lossy()
                .to_string(),
            "garm-independent-validation-report-v1",
        ),
        validation_record(
            "release_candidate_manifest",
            state_paths::state_dir()
                .join("release_candidate_manifest.json")
                .to_string_lossy()
                .to_string(),
            "garm-release-candidate-manifest-v1",
        ),
    ];
    let present = records
        .iter()
        .filter(|record| record.get("present").and_then(serde_json::Value::as_bool) == Some(true))
        .count();
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "eden-validation-status-api-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "present": present,
        "total": records.len(),
        "records": records,
        "permissions": read_only_permissions(),
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

pub fn action_dry_run_json(raw_command: &str) -> String {
    let mut last_command = String::new();
    let command = CommandRouterNode::parse_raw(raw_command, &mut last_command);
    let binding = GewcBodyRegistry::bind(&command);
    let permission = action_policy_for(&command);
    let persistent_permission = persistent_permission_value(&command);
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "eden-action-dry-run-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "dry_run": true,
        "would_execute": false,
        "raw_command": raw_command,
        "parsed_command": format!("{:?}", command),
        "action_class": permission.action_class,
        "risk_level": permission.risk_level,
        "permission_level": permission.permission_level,
        "mutates_runtime": permission.mutates_runtime,
        "requires_supervision": permission.requires_supervision,
        "requires_human_approval": permission.requires_human_approval,
        "standalone_execution_allowed": permission.standalone_execution_allowed,
        "persistent_permission": persistent_permission,
        "route": binding.route,
        "domain": binding.domain,
        "handler": binding.handler.as_str(),
        "execution_unit": binding.execution_unit,
        "lifecycle_policy": binding.lifecycle_policy,
        "required_runtime_route": "/api/command_sync?cmd=<command>",
        "permission_contract": action_permission_contract(permission),
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

fn catalog_value() -> serde_json::Value {
    let records: Vec<_> = ROUTES
        .iter()
        .map(|(path, name, description, mutates)| {
            serde_json::json!({
                "name": name,
                "path": path,
                "description": description,
                "method": "GET",
                "read_only": !mutates,
                "mutates_runtime": mutates,
            })
        })
        .collect();
    serde_json::json!({
        "schema": "eden-operational-api-catalog-v1",
        "artifact": "operational_api_catalog",
        "claim_allowed": false,
        "agi_claim": false,
        "purpose": "Expose runtime control, capabilities, GEWC, validation and action contracts as typed APIs.",
        "records": records,
        "total": records.len(),
    })
}

fn contracts_value() -> serde_json::Value {
    let contracts: Vec<_> = ROUTES
        .iter()
        .map(|(path, name, description, mutates)| {
            serde_json::json!({
                "surface": name,
                "endpoint": path,
                "description": description,
                "permission": if *mutates { "command_action" } else { "operational_read" },
                "read_only": !mutates,
                "mutates_runtime": mutates,
                "requires_gewc_gate": *mutates,
                "claim_allowed": false,
                "agi_claim": false
            })
        })
        .collect();
    serde_json::json!({
        "schema": "eden-operational-api-contracts-v1",
        "artifact": "operational_api_contracts",
        "claim_allowed": false,
        "agi_claim": false,
        "contracts": contracts,
    })
}

fn openapi_value() -> serde_json::Value {
    serde_json::json!({
        "schema": "eden-operational-openapi-v1",
        "artifact": "operational_api_openapi",
        "claim_allowed": false,
        "agi_claim": false,
        "openapi": "3.0.3",
        "info": {
            "title": "EDEN Operational API",
            "version": "v1",
            "description": "Read-only operational control and action-contract API surface."
        },
        "paths": {
            "/api/capabilities/catalog": read_operation("listCapabilityCatalog"),
            "/api/capabilities/status": read_operation("readCapabilityStatus"),
            "/api/gewc/runtime": read_operation("readGewcRuntime"),
            "/api/gewc/handlers": read_operation("listGewcHandlers"),
            "/api/validation/status": read_operation("readValidationStatus"),
            "/api/actions/contracts": read_operation("readActionContracts"),
            "/api/actions/dry-run": {
                "get": {
                    "operationId": "dryRunActionCommand",
                    "x-eden-permission": "operational_read",
                    "parameters": [{"name": "cmd", "in": "query", "required": true, "schema": {"type": "string"}}],
                    "responses": {"200": {"description": "Command classification without execution"}}
                }
            },
            "/api/operational/catalog": read_operation("listOperationalApiSurfaces"),
            "/api/operational/openapi": read_operation("readOperationalOpenApi"),
            "/api/operational/runtime": read_operation("readOperationalApiRuntime"),
            "/api/operational/status": read_operation("readOperationalStatus"),
            "/api/paradise/sessions": read_operation("readParadiseWorldcellSessions"),
            "/api/paradise/worldcell": read_operation("readParadiseWorldcellRuntime"),
            "/api/operational/contract": read_operation("readOperationalContract"),
            "/api/operational/permissions": read_operation("readOperationalPermissions"),
            "/api/operational/replay": {
                "get": {
                    "operationId": "readOperationalReplay",
                    "x-eden-permission": "operational_read",
                    "parameters": [{"name": "decision_id", "in": "query", "required": false, "schema": {"type": "string"}}],
                    "responses": {"200": {"description": "GEWC replay index or one replayable decision"}}
                }
            },
            "/api/operational/recovery": read_operation("readOperationalRecovery"),
            "/api/operational/demos": read_operation("readOperationalDemos"),
            "/api/operational/schemas": read_operation("readOperationalSchemaRegistry"),
            "/api/operational/spine": read_operation("readRuntimeSpine"),
            "/api/operational/events": read_operation("readRuntimeEventBus"),
            "/api/operational/global-state": read_operation("readRuntimeGlobalState"),
            "/api/operational/replay-spine": read_operation("readRuntimeReplaySpine"),
            "/api/operational/spine-verification": read_operation("readRuntimeSpineVerification"),
            "/api/operational/spine-enforcement": read_operation("readRuntimeSpineEnforcement"),
            "/api/operational/workflow-risk": read_operation("readRuntimeWorkflowRisk"),
            "/api/operational/circuit-breakers": read_operation("readRuntimeCircuitBreakers"),
            "/api/operational/spine-replay": read_operation("readRuntimeSpineReplay"),
            "/api/operational/schema": {
                "get": {
                    "operationId": "readOperationalSchemaRecord",
                    "x-eden-permission": "operational_read",
                    "parameters": [{"name": "name", "in": "query", "required": true, "schema": {"type": "string"}}],
                    "responses": {"200": {"description": "One EDEN schema registry record"}}
                }
            },
            "/api/operational/runtime-phase": read_operation("readOperationalRuntimePhase")
        }
    })
}

fn runtime_value() -> serde_json::Value {
    let generated_files = [
        state_paths::operational_api_catalog_path(),
        state_paths::operational_api_contracts_path(),
        state_paths::operational_api_openapi_path(),
        state_paths::operational_api_runtime_path(),
        state_paths::operational_action_contracts_path(),
        state_paths::operational_contract_path(),
        state_paths::operational_permissions_path(),
        state_paths::schema_registry_path(),
    ];
    let present = generated_files
        .iter()
        .filter(|path| std::fs::metadata(path).is_ok())
        .count();
    serde_json::json!({
        "schema": "eden-operational-api-runtime-v1",
        "artifact": "operational_api_runtime",
        "claim_allowed": false,
        "agi_claim": false,
        "read_endpoints": ROUTES.len(),
        "action_mutation_allowed": false,
        "generated_files_present": present,
        "generated_files_total": generated_files.len(),
        "write_policy": "operational APIs are read-only; mutation remains command-routed through GEWC gates",
    })
}

fn operational_contract_value() -> serde_json::Value {
    serde_json::json!({
        "schema": "eden-operational-contract-v1",
        "artifact": "operational_contract",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "global_executive_workspace_core",
        "purpose": "Define the executable local runtime contract for health, readiness, degraded operation, action boundaries, replay and shutdown.",
        "states": {
            "health": {
                "endpoint": "/api/health",
                "meaning": "process accepts local HTTP requests",
                "does_not_imply": ["runtime_ready", "autonomy_enabled", "capability_claim"],
                "failure_mode": "transport_or_process_unavailable"
            },
            "readiness": {
                "endpoint": "/ready",
                "meaning": "runtime startup completed and synchronous commands may be accepted",
                "required_checks": [
                    "state_directory_writable",
                    "command_queue_available",
                    "GEWC_dispatch_available",
                    "runtime_metrics_ready_true"
                ],
                "sync_command_gate": "/api/command_sync requires ready=true"
            },
            "degraded": {
                "meaning": "the process remains inspectable while one or more capabilities, handlers or resources are paused, isolated, disabled or not ready",
                "allowed_operations": [
                    "read_health",
                    "read_runtime_state",
                    "read_GEWC_traces",
                    "read_action_contracts",
                    "dry_run_actions",
                    "lifecycle_recover_or_disable"
                ],
                "blocked_operations": [
                    "high_risk_autonomous_action_without_approval",
                    "external_tool_execution_without_capability",
                    "memory_or_state_mutation_outside_GEWC_gate"
                ]
            },
            "shutdown": {
                "command": "quit",
                "route": "/api/command?cmd=quit",
                "policy": "queued shutdown request; cleanup is owned by runtime process and caller",
                "audit_expectation": "shutdown command passes through command queue"
            }
        },
        "invariants": [
            "GEWC is the authority for command routing and action gating",
            "read APIs must not mutate runtime state",
            "dry-run endpoints must not queue or execute commands",
            "mutating commands must pass through the command queue and GEWC body registry",
            "LLM or model outputs are treated as proposals until accepted by GEWC",
            "memory writes require an operational transaction or a command-routed memory handler",
            "high-risk or external actions require explicit capability, sandboxing and supervision",
            "runtime evidence preserves claim_allowed=false and agi_claim=false"
        ],
        "replay_contract": {
            "decision_log": "global_executive_workspace_runtime.jsonl",
            "action_evidence": "action_evidence.jsonl",
            "replay_command": "operational replay run",
            "reexecution_policy": "replay reads recorded traces and must not re-execute external actions"
        },
        "permission_boundary": {
            "capability_and_permission_are_separate": true,
            "persistent_permission_file": "operational_permissions.json",
            "default_external_network": "disabled unless explicitly allowed",
            "default_high_risk_action": "blocked or held for supervision",
            "sandbox_required_for": ["code", "files", "network", "external_tools", "robots_or_physical_actions"]
        },
        "verification_commands": [
            "make test",
            "make api-socket-test",
            "make operational-blackbox",
            "make eden-api-conformance"
        ]
    })
}

fn permissions_value() -> serde_json::Value {
    serde_json::json!({
        "schema": "eden-operational-permissions-v1",
        "artifact": "operational_permissions",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "global_executive_workspace_core",
        "mode": "local_persistent_capability_permissions",
        "mutation_policy": "edit_or_regenerate_explicitly; no public write endpoint",
        "capabilities": [
            permission_record("read_runtime", true, "low", false, false, "Read local runtime state and status."),
            permission_record("governed_local_action", true, "medium", false, true, "Run bounded local commands through GEWC."),
            permission_record("local_state_mutation", true, "medium", true, true, "Mutate local state only through command-routed GEWC gates."),
            permission_record("local_file_read", true, "medium", true, true, "Read local whitelisted files for corpus and import flows."),
            permission_record("remote_network", false, "high", true, true, "Remote crawl/network access remains denied unless an explicit runtime flag grants it."),
            permission_record("local_bounded_self_improvement", true, "high", true, true, "Allow bounded local self-improvement commands without source mutation or model weight updates."),
            permission_record("autonomous_runtime_action", true, "high", true, true, "Allow autonomous local action only while runtime autonomy is enabled and GEWC gates pass."),
            permission_record("experiment_execution", true, "high", true, true, "Allow bounded local experiments with provenance and policy audit."),
        ],
        "invariants": [
            "capability permission does not bypass GEWC routing",
            "dry-run reads this matrix but does not mutate it",
            "remote_network defaults to denied",
            "high-risk local actions remain supervised and auditable",
        ]
    })
}

pub fn default_permissions_value() -> serde_json::Value {
    permissions_value()
}

fn permission_record(
    id: &'static str,
    allowed: bool,
    risk: &'static str,
    requires_supervision: bool,
    sandbox_required: bool,
    description: &'static str,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "allowed": allowed,
        "risk": risk,
        "requires_supervision": requires_supervision,
        "sandbox_required": sandbox_required,
        "description": description,
    })
}

fn action_contracts_value() -> serde_json::Value {
    serde_json::json!({
        "schema": "eden-operational-action-contracts-v1",
        "artifact": "operational_action_contracts",
        "claim_allowed": false,
        "agi_claim": false,
        "read_only_routes": [
            "/api/actions/contracts",
            "/api/actions/dry-run?cmd=<command>"
        ],
        "mutation_routes": [
            {
                "path": "/api/command?cmd=<command>",
                "mode": "queued_async",
                "requires_ready_runtime": false,
                "requires_gewc_pre_execution_safety": true,
                "returns_command_id": true
            },
            {
                "path": "/api/command_sync?cmd=<command>",
                "mode": "queued_sync_wait",
                "requires_ready_runtime": true,
                "requires_gewc_pre_execution_safety": true,
                "timeout_sec": 5
            },
            {
                "path": "/api/command",
                "method": "POST",
                "mode": "legacy_compatibility_post",
                "requires_gewc_pre_execution_safety": true
            }
        ],
        "operational_command_examples": [
            "operational task submit validate local runtime state",
            "operational task run",
            "operational task audit",
            "operational action execute status",
            "operational memory commit runtime fact",
            "operational memory rollback <transaction_id>",
            "operational replay run",
            "operational smoke run",
            "operational scenario run",
            "operational permissions audit",
            "operational permissions set remote_network deny",
            "operational recovery run",
            "operational demo run",
            "paradise intent inspect runtime status safely",
            "paradise plan",
            "paradise approve",
            "paradise execute",
            "paradise sessions",
            "locus ingest operator preference :: governed context",
            "locus context operator permission boundary",
            "operator forge synth causal risk model",
            "operator forge verify",
            "runtime spine eval",
            "runtime spine audit",
            "runtime spine verify",
            "runtime spine enforce",
            "runtime spine risk",
            "runtime spine breakers",
            "runtime spine replay",
            "gewc lifecycle world_model pause"
        ],
        "permission_matrix": [
            {"level": "read", "risk": "low", "examples": ["status", "memory", "operational task audit"], "standalone_execution_allowed": true},
            {"level": "local_mutation", "risk": "medium", "examples": ["operational task run", "operational memory commit <text>", "locus ingest <text>", "operator forge synth <goal>"], "requires_supervision": true},
            {"level": "external_tool", "risk": "high", "examples": ["crawl <url>", "experiment run"], "requires_human_approval": true},
            {"level": "destructive_or_autonomous", "risk": "high", "examples": ["evolve", "goals run"], "requires_human_approval": true},
            {"level": "requires_clarification", "risk": "medium", "examples": ["unknown command"], "executes": false}
        ],
        "permission_rules": [
            "dry-run never queues or executes commands",
            "read APIs never mutate runtime state",
            "mutation uses command queue and GEWC body registry",
            "high-risk commands require explicit supervision or runtime flags",
            "all action evidence remains no-claim and auditable"
        ]
    })
}

fn validation_record(name: &str, path: String, schema_hint: &str) -> serde_json::Value {
    let body = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::json!({
        "name": name,
        "path": path,
        "schema_hint": schema_hint,
        "present": !body.is_empty(),
        "bytes": body.len(),
        "schema_seen": body.contains(schema_hint),
        "claim_allowed_false": body.contains("claim_allowed=false") || body.contains("\"claim_allowed\": false"),
        "agi_claim_false": body.contains("agi_claim=false") || body.contains("\"agi_claim\": false"),
    })
}

#[derive(Clone, Copy)]
struct ActionPermissionProfile {
    action_class: &'static str,
    risk_level: &'static str,
    permission_level: &'static str,
    requires_supervision: bool,
    requires_human_approval: bool,
    mutates_runtime: bool,
    standalone_execution_allowed: bool,
    sandbox_required: bool,
}

fn action_policy_for(command: &GarmCommand) -> ActionPermissionProfile {
    match command {
        GarmCommand::Status
        | GarmCommand::Readiness
        | GarmCommand::ReadinessBench
        | GarmCommand::Memory
        | GarmCommand::WorldModel
        | GarmCommand::PolicyAudit
        | GarmCommand::ProvenanceAudit
        | GarmCommand::UncertaintyAudit
        | GarmCommand::ArtifactApiEval
        | GarmCommand::RuntimeStateApiEval
        | GarmCommand::OperationalRuntimeEval
        | GarmCommand::OperationalTaskAudit
        | GarmCommand::ParadiseAudit
        | GarmCommand::OperationalReplayRun
        | GarmCommand::OperationalPermissionsAudit
        | GarmCommand::OperationalPermissionsDiff
        | GarmCommand::OperationalPermissionsHistory
        | GarmCommand::OperationalRecoveryAudit
        | GarmCommand::RuntimeSpineAudit
        | GarmCommand::RuntimeSpineRisk
        | GarmCommand::RuntimeSpineBreakers
        | GarmCommand::RuntimeSpineReplay
        | GarmCommand::LocusAudit
        | GarmCommand::OperatorForgeAudit => {
            permission_profile("read_or_eval", "read", "low", false, false, false, true)
        }
        GarmCommand::OperationalSmokeRun
        | GarmCommand::OperationalScenarioRun
        | GarmCommand::OperationalDemoRun
        | GarmCommand::RuntimeSpineEval
        | GarmCommand::RuntimeSpineVerify
        | GarmCommand::RuntimeSpineEnforce => permission_profile(
            "operational_validation_mutation",
            "local_mutation",
            "medium",
            true,
            false,
            true,
            false,
        ),
        GarmCommand::OperationalTaskSubmit(_)
        | GarmCommand::OperationalTaskRun
        | GarmCommand::ParadiseIntent(_)
        | GarmCommand::ParadisePlan(_)
        | GarmCommand::ParadiseApprove(_)
        | GarmCommand::ParadiseExecute(_)
        | GarmCommand::OperationalMemoryCommit(_)
        | GarmCommand::OperationalMemoryRollback(_)
        | GarmCommand::OperationalPermissionsRestore
        | GarmCommand::OperationalPermissionsSet(_)
        | GarmCommand::OperationalRecoveryRun
        | GarmCommand::GewcLifecycleControl(_) => permission_profile(
            "operational_mutation",
            "local_mutation",
            "medium",
            true,
            false,
            true,
            false,
        ),
        GarmCommand::OperationalActionExecute(_) => permission_profile(
            "nested_action_executor",
            "external_tool",
            "medium",
            true,
            false,
            true,
            false,
        ),
        GarmCommand::Crawl(_) | GarmCommand::ConceptNet(_) | GarmCommand::ExperimentRun => {
            permission_profile(
                "external_tool_or_experiment",
                "external_tool",
                "high",
                true,
                true,
                true,
                false,
            )
        }
        GarmCommand::Evolve | GarmCommand::PlanExecutorRun | GarmCommand::GoalsRun => {
            permission_profile(
                "high_risk_action",
                "destructive_or_autonomous",
                "high",
                true,
                true,
                true,
                false,
            )
        }
        GarmCommand::Unknown(_) => permission_profile(
            "unknown_intent",
            "requires_clarification",
            "medium",
            true,
            false,
            false,
            false,
        ),
        _ => permission_profile(
            "governed_action",
            "local_mutation",
            "medium",
            false,
            false,
            true,
            false,
        ),
    }
}

fn permission_profile(
    action_class: &'static str,
    permission_level: &'static str,
    risk_level: &'static str,
    requires_supervision: bool,
    requires_human_approval: bool,
    mutates_runtime: bool,
    standalone_execution_allowed: bool,
) -> ActionPermissionProfile {
    ActionPermissionProfile {
        action_class,
        risk_level,
        permission_level,
        requires_supervision,
        requires_human_approval,
        mutates_runtime,
        standalone_execution_allowed,
        sandbox_required: !matches!(permission_level, "read" | "requires_clarification"),
    }
}

fn action_permission_contract(permission: ActionPermissionProfile) -> serde_json::Value {
    serde_json::json!({
        "read_only": false,
        "dry_run": true,
        "risk_level": permission.risk_level,
        "permission_level": permission.permission_level,
        "mutates_runtime_if_executed": permission.mutates_runtime,
        "requires_supervision": permission.requires_supervision,
        "requires_human_approval": permission.requires_human_approval,
        "standalone_execution_allowed": permission.standalone_execution_allowed,
        "sandbox_required": permission.sandbox_required,
        "requires_gewc_pre_execution_safety": true,
        "requires_audit_trace": true,
        "queueing_allowed_by_this_endpoint": false,
    })
}

fn persistent_permission_value(command: &GarmCommand) -> serde_json::Value {
    let key = permission_key_for_command(command);
    let record = permission_record_for_key(key).unwrap_or_else(|| {
        let fallback = permissions_value();
        fallback
            .get("capabilities")
            .and_then(serde_json::Value::as_array)
            .and_then(|records| {
                records
                    .iter()
                    .find(|record| {
                        record.get("id").and_then(serde_json::Value::as_str) == Some(key)
                    })
                    .cloned()
            })
            .unwrap_or_else(|| serde_json::json!({"id": key, "allowed": true}))
    });
    serde_json::json!({
        "permission_key": key,
        "source": state_paths::operational_permissions_path(),
        "record": record,
        "matrix_present": std::fs::metadata(state_paths::operational_permissions_path()).is_ok(),
    })
}

fn permission_summary_value() -> serde_json::Value {
    let value = permissions_json_value();
    let empty = Vec::new();
    let capabilities = value
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .unwrap_or(&empty);
    let allowed = capabilities
        .iter()
        .filter(|record| record.get("allowed").and_then(serde_json::Value::as_bool) == Some(true))
        .count();
    let denied = capabilities.len().saturating_sub(allowed);
    serde_json::json!({
        "schema": "eden-operational-permission-summary-v1",
        "path": state_paths::operational_permissions_path(),
        "present": std::fs::metadata(state_paths::operational_permissions_path()).is_ok(),
        "total": capabilities.len(),
        "allowed": allowed,
        "denied": denied,
    })
}

fn permission_record_for_key(key: &str) -> Option<serde_json::Value> {
    permissions_json_value()
        .get("capabilities")
        .and_then(serde_json::Value::as_array)?
        .iter()
        .find(|record| record.get("id").and_then(serde_json::Value::as_str) == Some(key))
        .cloned()
}

fn permissions_json_value() -> serde_json::Value {
    std::fs::read_to_string(state_paths::operational_permissions_path())
        .ok()
        .and_then(|body| serde_json::from_str::<serde_json::Value>(&body).ok())
        .unwrap_or_else(permissions_value)
}

fn permission_key_for_command(command: &GarmCommand) -> &'static str {
    match command {
        GarmCommand::Status
        | GarmCommand::Memory
        | GarmCommand::Readiness
        | GarmCommand::ReadinessBench
        | GarmCommand::GarmReport
        | GarmCommand::GarmReportHistory
        | GarmCommand::GarmArtifacts
        | GarmCommand::OperationalTaskAudit
        | GarmCommand::ParadiseAudit
        | GarmCommand::OperationalReplayRun
        | GarmCommand::OperationalPermissionsAudit
        | GarmCommand::OperationalPermissionsDiff
        | GarmCommand::OperationalPermissionsHistory
        | GarmCommand::OperationalRecoveryAudit
        | GarmCommand::RuntimeSpineAudit
        | GarmCommand::RuntimeSpineRisk
        | GarmCommand::RuntimeSpineBreakers
        | GarmCommand::RuntimeSpineReplay
        | GarmCommand::ActionEvidence
        | GarmCommand::LocusAudit
        | GarmCommand::OperatorForgeAudit => "read_runtime",
        GarmCommand::Crawl(_) => "remote_network",
        GarmCommand::ConceptNet(_)
        | GarmCommand::HrmTextCorpus(_)
        | GarmCommand::HrmTextIngest(_) => "local_file_read",
        GarmCommand::Evolve | GarmCommand::LearningConsolidate => "local_bounded_self_improvement",
        GarmCommand::PlanExecutorRun
        | GarmCommand::GoalsRun
        | GarmCommand::OrgansRun
        | GarmCommand::Rebirth => "autonomous_runtime_action",
        GarmCommand::ExperimentRun => "experiment_execution",
        GarmCommand::OperationalTaskSubmit(_)
        | GarmCommand::OperationalTaskRun
        | GarmCommand::ParadiseIntent(_)
        | GarmCommand::ParadisePlan(_)
        | GarmCommand::ParadiseApprove(_)
        | GarmCommand::ParadiseExecute(_)
        | GarmCommand::OperationalMemoryCommit(_)
        | GarmCommand::OperationalMemoryRollback(_)
        | GarmCommand::LocusLayerEval
        | GarmCommand::LocusIngest(_)
        | GarmCommand::LocusContext(_)
        | GarmCommand::OperatorForgeEval
        | GarmCommand::OperatorForgeSynthesize(_)
        | GarmCommand::OperatorForgeVerify
        | GarmCommand::OperationalPermissionsRestore
        | GarmCommand::OperationalPermissionsSet(_)
        | GarmCommand::OperationalRecoveryRun
        | GarmCommand::OperationalDemoRun
        | GarmCommand::RuntimeSpineEval
        | GarmCommand::RuntimeSpineVerify
        | GarmCommand::RuntimeSpineEnforce
        | GarmCommand::GewcLifecycleControl(_) => "local_state_mutation",
        _ => "governed_local_action",
    }
}

fn handler_state_records() -> Vec<serde_json::Value> {
    GewcBodyHandler::ALL
        .iter()
        .map(|handler| {
            let control = GlobalExecutiveWorkspaceCore::lifecycle_control_for_handler(*handler);
            serde_json::json!({
                "handler": handler.as_str(),
                "state": control.state.as_str(),
                "allowed_actions": control.action_names(),
                "supervision_required": control.supervision_required,
                "policy_gate": control.policy_gate,
            })
        })
        .collect()
}

fn runtime_records() -> Vec<serde_json::Value> {
    std::fs::read_to_string(state_paths::global_executive_workspace_runtime_path())
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect()
}

fn latest_record_with_phase(phase: &str) -> Option<serde_json::Value> {
    runtime_records().into_iter().rev().find(|record| {
        record
            .get("phase")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|record_phase| record_phase == phase)
    })
}

fn latest_blocking_record() -> Option<serde_json::Value> {
    let latest_operational = runtime_records().into_iter().rev().find(|record| {
        record
            .get("phase")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|phase| phase == "decision_started" || phase == "execution_completed")
    })?;
    let disposition_blocked = latest_operational
        .get("disposition")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value != "execute");
    let execution_blocked = latest_operational
        .get("execution_status")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value == "blocked");
    let lifecycle_denied = latest_operational
        .get("module_lifecycle_action_allowed")
        .and_then(serde_json::Value::as_bool)
        == Some(false);
    if disposition_blocked || execution_blocked || lifecycle_denied {
        Some(latest_operational)
    } else {
        None
    }
}

fn decision_id_for_record(record: &serde_json::Value) -> String {
    if let Some(decision_id) = record
        .get("decision_id")
        .and_then(serde_json::Value::as_str)
    {
        return decision_id.to_string();
    }
    let raw_command = record
        .get("raw_command")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let command_kind = record
        .get("command_kind")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let route = record
        .get("route")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let global_tick = record
        .get("global_tick")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    let graph_nodes = record
        .get("graph_nodes")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    format!(
        "gewc-{:016x}",
        fnv64(
            format!("{raw_command}|{command_kind}|{route}|{global_tick}|{graph_nodes}").as_bytes()
        )
    )
}

fn read_only_permissions() -> serde_json::Value {
    serde_json::json!({
        "read_only": true,
        "mutates_runtime": false,
        "command_execution": false,
    })
}

fn read_operation(operation_id: &str) -> serde_json::Value {
    serde_json::json!({
        "get": {
            "operationId": operation_id,
            "x-eden-permission": "operational_read",
            "responses": {"200": {"description": "Operational read response"}}
        }
    })
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
    fn operational_api_writes_contracts_and_classifies_actions_without_execution() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_operational_api_test_{}",
            std::process::id()
        )));
        let _ = state_paths::ensure_state_dir();
        std::fs::write(
            state_paths::external_validation_result_path(),
            "{\"schema\":\"garm-external-validation-result-v1\",\"claim_allowed\":false,\"agi_claim\":false}",
        )
        .unwrap();

        let out = run();

        assert!(out.contains("[OPERATIONAL-API]"));
        assert!(std::fs::metadata(state_paths::operational_api_catalog_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_api_contracts_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_api_openapi_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_api_runtime_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_action_contracts_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_contract_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_permissions_path()).is_ok());
        assert!(std::fs::metadata(state_paths::schema_registry_path()).is_ok());
        assert!(catalog_json().contains("/api/capabilities/catalog"));
        assert!(catalog_json().contains("/api/operational/status"));
        assert!(catalog_json().contains("/api/operational/contract"));
        assert!(catalog_json().contains("/api/operational/permissions"));
        assert!(catalog_json().contains("/api/operational/replay"));
        assert!(catalog_json().contains("/api/operational/recovery"));
        assert!(catalog_json().contains("/api/operational/demos"));
        assert!(catalog_json().contains("/api/operational/schemas"));
        assert!(openapi_json().contains("dryRunActionCommand"));
        assert!(openapi_json().contains("readOperationalStatus"));
        assert!(openapi_json().contains("readOperationalContract"));
        assert!(openapi_json().contains("readOperationalPermissions"));
        assert!(openapi_json().contains("readOperationalReplay"));
        assert!(openapi_json().contains("readOperationalRecovery"));
        assert!(openapi_json().contains("readOperationalSchemaRegistry"));
        assert!(contract_json().contains("eden-operational-contract-v1"));
        assert!(contract_json().contains("\"degraded\""));
        assert!(contract_json().contains("operational replay run"));
        assert!(permissions_json().contains("eden-operational-permissions-v1"));
        assert!(permissions_json().contains("remote_network"));
        assert!(action_contracts_json().contains("/api/command_sync?cmd=<command>"));
        assert!(action_contracts_json().contains("permission_matrix"));
        let dry_run = action_dry_run_json("evolve");
        assert!(dry_run.contains("\"would_execute\": false"));
        assert!(dry_run.contains("\"requires_supervision\": true"));
        assert!(dry_run.contains("\"permission_level\": \"destructive_or_autonomous\""));
        assert!(dry_run.contains("\"permission_key\": \"local_bounded_self_improvement\""));
        assert!(action_dry_run_json("status").contains("\"standalone_execution_allowed\": true"));
        assert!(action_dry_run_json("locus ingest operator preference")
            .contains("gewc_locus_context_body_handler"));
        assert!(action_dry_run_json("operator forge synth causal risk")
            .contains("gewc_formal_synthesis_body_handler"));
        let status = status_json(OperationalStatusInput {
            ready: true,
            autonomous: true,
            daemon_enabled: false,
            uptime_sec: 1,
            alive_nodes: 2,
            edge_count: 3,
            memory_facts: 4,
            api_requests: 5,
            engine_status: "garm | test",
            capability_count: 6,
            tick_count: 7,
            idle_ticks: 8,
        });
        assert!(status.contains("eden-operational-status-v1"));
        assert!(status.contains("\"state\": \"ready\""));
        assert!(replay_index_json().contains("eden-gewc-replay-index-v1"));
        assert!(replay_decision_json("missing").contains("\"found\": false"));
        assert!(recovery_json().contains("eden-operational-recovery-plan-v1"));
        assert!(demos_json().contains("eden-operational-demo-suite-v1"));
        assert!(schema_registry_json().contains("eden-schema-registry-v1"));
        assert!(schema_record_json("operational_status").contains("\"found\": true"));
        let engine = GarmCapabilityState::new_fast();
        assert!(capabilities_catalog_json(&engine).contains("eden-capabilities-catalog-v1"));
        assert!(validation_status_json().contains("garm-external-validation-result-v1"));
        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
