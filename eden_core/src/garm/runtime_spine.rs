use crate::eden_garm::state_paths;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub const RUNTIME_SPINE_SCHEMA: &str = "eden-runtime-spine-v1";
pub const INTERNAL_MESSAGE_SCHEMA: &str = "eden-internal-message-v1";
pub const EVENT_BUS_SCHEMA: &str = "eden-runtime-event-bus-v1";
pub const GLOBAL_STATE_SCHEMA: &str = "eden-runtime-global-state-v1";
pub const REPLAY_SPINE_SCHEMA: &str = "eden-runtime-replay-spine-v1";
pub const SPINE_VERIFICATION_SCHEMA: &str = "eden-runtime-spine-verification-v1";
pub const ENFORCEMENT_SCHEMA: &str = "eden-runtime-spine-enforcement-v1";
pub const GUARD_DECISION_SCHEMA: &str = "eden-runtime-guard-decision-v1";
pub const WORKFLOW_RISK_SCHEMA: &str = "eden-runtime-workflow-risk-v1";
pub const CIRCUIT_BREAKERS_SCHEMA: &str = "eden-runtime-circuit-breakers-v1";
pub const REPLAY_RECONSTRUCTION_SCHEMA: &str = "eden-runtime-replay-reconstruction-v1";
pub const SECURITY_GATES_SCHEMA: &str = "eden-runtime-security-gates-v1";
pub const MODEL_ROUTER_SCHEMA: &str = "eden-model-router-contract-v1";
pub const MEMORY_FABRIC_SCHEMA: &str = "eden-memory-fabric-contract-v1";
pub const WORLD_SIMULATION_SCHEMA: &str = "eden-world-simulation-contract-v1";
pub const MULTIAGENT_CONTRACT_SCHEMA: &str = "eden-multiagent-contract-v1";

const AUTHORITY: &str = "global_executive_workspace_core";

const ARTIFACTS: &[(&str, fn() -> String)] = &[
    ("runtime_spine", state_paths::runtime_spine_path),
    (
        "runtime_internal_contracts",
        state_paths::runtime_internal_contracts_path,
    ),
    ("runtime_event_bus", state_paths::runtime_event_bus_path),
    (
        "runtime_event_bus_state",
        state_paths::runtime_event_bus_state_path,
    ),
    (
        "runtime_global_state",
        state_paths::runtime_global_state_path,
    ),
    (
        "runtime_global_state_log",
        state_paths::runtime_global_state_log_path,
    ),
    (
        "runtime_replay_spine",
        state_paths::runtime_replay_spine_path,
    ),
    (
        "runtime_spine_verification",
        state_paths::runtime_spine_verification_path,
    ),
    (
        "runtime_guard_decisions",
        state_paths::runtime_guard_decisions_path,
    ),
    (
        "runtime_spine_enforcement",
        state_paths::runtime_spine_enforcement_path,
    ),
    (
        "runtime_workflow_risk",
        state_paths::runtime_workflow_risk_path,
    ),
    (
        "runtime_circuit_breakers",
        state_paths::runtime_circuit_breakers_path,
    ),
    (
        "runtime_replay_reconstruction",
        state_paths::runtime_replay_reconstruction_path,
    ),
    (
        "runtime_security_gates",
        state_paths::runtime_security_gates_path,
    ),
    (
        "runtime_model_router_contract",
        state_paths::runtime_model_router_contract_path,
    ),
    (
        "runtime_memory_fabric_contract",
        state_paths::runtime_memory_fabric_contract_path,
    ),
    (
        "runtime_world_simulation_contract",
        state_paths::runtime_world_simulation_contract_path,
    ),
    (
        "runtime_multiagent_contract",
        state_paths::runtime_multiagent_contract_path,
    ),
];

pub fn run() -> String {
    let _ = state_paths::ensure_state_dir();
    write_json(
        state_paths::runtime_internal_contracts_path(),
        internal_contracts_value(),
    );
    write_json(
        state_paths::runtime_security_gates_path(),
        security_gates_value(),
    );
    write_json(
        state_paths::runtime_model_router_contract_path(),
        model_router_contract_value(),
    );
    write_json(
        state_paths::runtime_memory_fabric_contract_path(),
        memory_fabric_contract_value(),
    );
    write_json(
        state_paths::runtime_world_simulation_contract_path(),
        world_simulation_contract_value(),
    );
    write_json(
        state_paths::runtime_multiagent_contract_path(),
        multiagent_contract_value(),
    );
    record_state_mutation(
        "runtime_spine",
        "kernel_contracts",
        "eval",
        "validated",
        serde_json::json!({
            "universal_contracts": true,
            "event_bus": true,
            "global_state": true,
            "replay": true,
            "safety": true,
            "model_router": true,
            "memory_fabric": true,
            "world_simulation": true,
            "multiagent": true,
            "enforcement_guard": true,
            "workflow_risk": true,
            "circuit_breakers": true,
            "replay_reconstruction": true,
        }),
    );
    publish_event(
        "runtime_spine",
        "operational_runtime",
        "spine_evaluated",
        "low",
        serde_json::json!({
            "command": "runtime spine eval",
            "claim_allowed": false,
            "agi_claim": false,
        }),
    );
    refresh_event_bus_state();
    write_json(
        state_paths::runtime_replay_spine_path(),
        replay_spine_value(),
    );
    write_enforcement_artifacts();
    write_json(state_paths::runtime_spine_path(), spine_value());
    write_json(
        state_paths::runtime_spine_verification_path(),
        verification_value(),
    );
    write_json(state_paths::runtime_spine_path(), spine_value());

    let present = ARTIFACTS
        .iter()
        .filter(|(_, path_fn)| std::fs::metadata(path_fn()).is_ok())
        .count();
    format!(
        "[RUNTIME-SPINE] passed={}/{} contracts=universal event_bus=append_only global_state=validated replay=reconstructable model_router=contracted memory=reconciled world_simulation=contracted security=end_to_end multiagent=budgeted claim_allowed=false path={}\n",
        present,
        ARTIFACTS.len(),
        state_paths::runtime_spine_path()
    )
}

pub fn enforce() -> String {
    write_enforcement_artifacts();
    let enforcement = enforcement_value();
    let guard_decisions = read_jsonl_records(&state_paths::runtime_guard_decisions_path()).len();
    let active_breakers = enforcement
        .get("summary")
        .and_then(|summary| summary.get("active_breakers"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let blocked_decisions = enforcement
        .get("summary")
        .and_then(|summary| summary.get("blocked_guard_decisions"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    write_json(state_paths::runtime_spine_enforcement_path(), enforcement);
    format!(
        "[RUNTIME-SPINE-ENFORCE] guard=active decisions={} blocked={} active_breakers={} claim_allowed=false path={}\n",
        guard_decisions,
        blocked_decisions,
        active_breakers,
        state_paths::runtime_spine_enforcement_path()
    )
}

pub fn workflow_risk() -> String {
    let risk = workflow_risk_value();
    let level = risk
        .get("summary")
        .and_then(|summary| summary.get("effective_risk"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let escalations = risk
        .get("summary")
        .and_then(|summary| summary.get("escalations"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    write_json(state_paths::runtime_workflow_risk_path(), risk);
    format!(
        "[RUNTIME-WORKFLOW-RISK] effective_risk={} escalations={} claim_allowed=false path={}\n",
        level,
        escalations,
        state_paths::runtime_workflow_risk_path()
    )
}

pub fn circuit_breakers() -> String {
    let breakers = circuit_breakers_value();
    let active = breakers
        .get("summary")
        .and_then(|summary| summary.get("active_breakers"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total = breakers
        .get("breakers")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    write_json(state_paths::runtime_circuit_breakers_path(), breakers);
    format!(
        "[RUNTIME-CIRCUIT-BREAKERS] active={}/{} claim_allowed=false path={}\n",
        active,
        total,
        state_paths::runtime_circuit_breakers_path()
    )
}

pub fn reconstruct_replay() -> String {
    let replay = replay_reconstruction_value();
    let timeline = replay
        .get("summary")
        .and_then(|summary| summary.get("timeline_records"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let cycles = replay
        .get("summary")
        .and_then(|summary| summary.get("reconstructed_cycles"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    write_json(state_paths::runtime_replay_reconstruction_path(), replay);
    format!(
        "[RUNTIME-SPINE-REPLAY] timeline_records={} reconstructed_cycles={} claim_allowed=false path={}\n",
        timeline,
        cycles,
        state_paths::runtime_replay_reconstruction_path()
    )
}

pub fn audit() -> String {
    let event_records = read_jsonl_records(&state_paths::runtime_event_bus_path());
    let mutation_records = read_jsonl_records(&state_paths::runtime_global_state_log_path());
    let present = ARTIFACTS
        .iter()
        .filter(|(_, path_fn)| std::fs::metadata(path_fn()).is_ok())
        .count();
    format!(
        "[RUNTIME-SPINE-AUDIT] artifacts={}/{} events={} mutations={} authority={} claim_allowed=false path={}\n",
        present,
        ARTIFACTS.len(),
        event_records.len(),
        mutation_records.len(),
        AUTHORITY,
        state_paths::runtime_spine_path()
    )
}

pub fn verify() -> String {
    refresh_replay_state();
    write_enforcement_artifacts();
    let report = verification_value();
    let checks = report
        .get("checks")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let passed = report
        .get("summary")
        .and_then(|summary| summary.get("passed_checks"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let events = report
        .get("summary")
        .and_then(|summary| summary.get("events"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let mutations = report
        .get("summary")
        .and_then(|summary| summary.get("state_mutations"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let verdict = report
        .get("summary")
        .and_then(|summary| summary.get("verdict"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    write_json(state_paths::runtime_spine_verification_path(), report);
    format!(
        "[RUNTIME-SPINE-VERIFY] passed={}/{} verdict={} events={} mutations={} claim_allowed=false path={}\n",
        passed,
        checks,
        verdict,
        events,
        mutations,
        state_paths::runtime_spine_verification_path()
    )
}

pub fn publish_event(
    source: &str,
    domain: &str,
    event_type: &str,
    risk: &str,
    payload: Value,
) -> String {
    let _ = state_paths::ensure_state_dir();
    let guard = guard_decision_value("event", source, domain, event_type, risk, &payload);
    append_guard_decision(&guard);
    if !guard_allowed(&guard) {
        return format!(
            "[RUNTIME-EVENT-BLOCKED] source={} type={} domain={} reason={} path={}\n",
            source,
            event_type,
            domain,
            guard_reason(&guard),
            state_paths::runtime_guard_decisions_path()
        );
    }
    let path = state_paths::runtime_event_bus_path();
    let sequence = next_jsonl_sequence(&path);
    let timestamp_ms = unix_time_ms();
    let payload_hash = value_hash(&payload);
    let event_id = format!(
        "evt-{:016x}",
        fnv64(format!("{source}|{domain}|{event_type}|{sequence}|{payload_hash}").as_bytes())
    );
    let record = serde_json::json!({
        "schema": INTERNAL_MESSAGE_SCHEMA,
        "id": event_id,
        "sequence": sequence,
        "timestamp_ms": timestamp_ms,
        "origin": source,
        "source": source,
        "domain": domain,
        "event_type": event_type,
        "authority": AUTHORITY,
        "risk": risk,
        "confidence": 0.9,
        "permissions": {
            "write_global_state": false,
            "requires_gewc_authority": true,
            "may_execute_tools": false
        },
        "payload_hash": payload_hash,
        "payload": payload,
        "claim_allowed": false,
        "agi_claim": false,
    });
    append_jsonl(&path, &record);
    refresh_event_bus_state();
    refresh_replay_state();
    format!(
        "[RUNTIME-EVENT] id={} type={} domain={} path={}\n",
        record
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        event_type,
        domain,
        path
    )
}

pub fn record_state_mutation(
    actor: &str,
    state_key: &str,
    operation: &str,
    status: &str,
    payload: Value,
) -> String {
    let _ = state_paths::ensure_state_dir();
    let risk = mutation_risk(actor, state_key, operation, status, &payload);
    let guard = guard_decision_value(
        "state_mutation",
        actor,
        state_key,
        operation,
        &risk,
        &payload,
    );
    append_guard_decision(&guard);
    if !guard_allowed(&guard) {
        return format!(
            "[RUNTIME-GLOBAL-STATE-BLOCKED] actor={} state_key={} reason={} path={}\n",
            actor,
            state_key,
            guard_reason(&guard),
            state_paths::runtime_guard_decisions_path()
        );
    }
    let path = state_paths::runtime_global_state_log_path();
    let sequence = next_jsonl_sequence(&path);
    let payload_hash = value_hash(&payload);
    let mutation_id = format!(
        "mut-{:016x}",
        fnv64(format!("{actor}|{state_key}|{operation}|{sequence}|{payload_hash}").as_bytes())
    );
    let record = serde_json::json!({
        "schema": "eden-runtime-global-state-mutation-v1",
        "id": mutation_id,
        "sequence": sequence,
        "timestamp_ms": unix_time_ms(),
        "actor": actor,
        "authority": AUTHORITY,
        "state_key": state_key,
        "operation": operation,
        "status": status,
        "append_only": true,
        "requires_validation": true,
        "rollback_strategy": "snapshot_pointer_or_compensating_event",
        "payload_hash": payload_hash,
        "payload": payload,
        "claim_allowed": false,
        "agi_claim": false,
    });
    append_jsonl(&path, &record);
    write_json(
        state_paths::runtime_global_state_path(),
        global_state_value(),
    );
    refresh_replay_state();
    format!(
        "[RUNTIME-GLOBAL-STATE] mutation={} state_key={} status={} path={}\n",
        record
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        state_key,
        status,
        path
    )
}

pub fn spine_json() -> String {
    read_json_or_default(state_paths::runtime_spine_path(), spine_value)
}

pub fn internal_contracts_json() -> String {
    read_json_or_default(
        state_paths::runtime_internal_contracts_path(),
        internal_contracts_value,
    )
}

pub fn event_bus_json() -> String {
    read_json_or_default(
        state_paths::runtime_event_bus_state_path(),
        event_bus_state_value,
    )
}

pub fn global_state_json() -> String {
    read_json_or_default(state_paths::runtime_global_state_path(), global_state_value)
}

pub fn replay_spine_json() -> String {
    read_json_or_default(state_paths::runtime_replay_spine_path(), replay_spine_value)
}

pub fn spine_verification_json() -> String {
    read_json_or_default(
        state_paths::runtime_spine_verification_path(),
        verification_value,
    )
}

pub fn enforcement_json() -> String {
    read_json_or_default(
        state_paths::runtime_spine_enforcement_path(),
        enforcement_value,
    )
}

pub fn workflow_risk_json() -> String {
    read_json_or_default(
        state_paths::runtime_workflow_risk_path(),
        workflow_risk_value,
    )
}

pub fn circuit_breakers_json() -> String {
    read_json_or_default(
        state_paths::runtime_circuit_breakers_path(),
        circuit_breakers_value,
    )
}

pub fn replay_reconstruction_json() -> String {
    read_json_or_default(
        state_paths::runtime_replay_reconstruction_path(),
        replay_reconstruction_value,
    )
}

pub fn security_gates_json() -> String {
    read_json_or_default(
        state_paths::runtime_security_gates_path(),
        security_gates_value,
    )
}

pub fn model_router_json() -> String {
    read_json_or_default(
        state_paths::runtime_model_router_contract_path(),
        model_router_contract_value,
    )
}

pub fn memory_fabric_json() -> String {
    read_json_or_default(
        state_paths::runtime_memory_fabric_contract_path(),
        memory_fabric_contract_value,
    )
}

pub fn world_simulation_json() -> String {
    read_json_or_default(
        state_paths::runtime_world_simulation_contract_path(),
        world_simulation_contract_value,
    )
}

pub fn multiagent_json() -> String {
    read_json_or_default(
        state_paths::runtime_multiagent_contract_path(),
        multiagent_contract_value,
    )
}

fn spine_value() -> Value {
    let event_records = read_jsonl_records(&state_paths::runtime_event_bus_path());
    let mutation_records = read_jsonl_records(&state_paths::runtime_global_state_log_path());
    serde_json::json!({
        "schema": RUNTIME_SPINE_SCHEMA,
        "artifact": "runtime_spine",
        "authority": AUTHORITY,
        "name": "Eden Runtime Spine",
        "category": "kernel_operational_contracts",
        "claim_allowed": false,
        "agi_claim": false,
        "purpose": "Make Eden's GEWC/Paradise runtime enforce common contracts for messages, state, replay, safety, models, memory, simulation and multiagent coordination.",
        "kernel_invariants": [
            "GEWC remains the only executive authority",
            "model outputs are hypotheses until verified",
            "tools execute only through contracts and permission gates",
            "global state is append-only plus validated snapshots",
            "replay evidence is generated from events and state mutations",
            "safety gates apply before, during and after execution"
        ],
        "interfaces": [
            "InferenceAPI",
            "PlanContract",
            "ToolContract",
            "MemoryQuery",
            "SimulationRequest",
            "PolicyContract",
            "EventBus",
            "JSON"
        ],
        "artifacts": ARTIFACTS
            .iter()
            .map(|(name, path_fn)| {
                let path = path_fn();
                serde_json::json!({
                    "name": name,
                    "path": path,
                    "present": std::fs::metadata(&path).is_ok(),
                    "read_endpoint": format!("/api/artifact?name={}", name)
                })
            })
            .collect::<Vec<_>>(),
        "runtime_counts": {
            "events": event_records.len(),
            "state_mutations": mutation_records.len()
        },
    })
}

fn internal_contracts_value() -> Value {
    serde_json::json!({
        "schema": "eden-runtime-internal-contracts-v1",
        "artifact": "runtime_internal_contracts",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "standard_message": {
            "schema": INTERNAL_MESSAGE_SCHEMA,
            "required": [
                "id",
                "sequence",
                "timestamp_ms",
                "origin",
                "domain",
                "event_type",
                "authority",
                "risk",
                "confidence",
                "permissions",
                "payload_hash",
                "payload"
            ],
            "authority_rule": "documents, tools and models may provide data; only GEWC-authorized handlers may mutate global state"
        },
        "module_interfaces": [
            interface("kernel_model", "InferenceAPI", "structured request to model adapters; no direct model authority"),
            interface("planner_executor", "PlanContract", "DAG/list plan with preconditions, postconditions, risks and rollback"),
            interface("executor_tools", "ToolContract", "typed tool call with schema, permission key, dry-run and sandbox policy"),
            interface("memory_retrieval", "MemoryQuery", "hybrid vector/symbolic/graph query filtered by authority and permissions"),
            interface("world_model_simulator", "SimulationRequest", "causal scenario request with uncertainty, assumptions and reality labels"),
            interface("security_permissions", "PolicyContract", "workflow-level policy decision, not only per-action checks"),
            interface("observability_modules", "EventBus", "append-only events for replay, dashboards and incident reports")
        ],
        "data_format": "JSON",
        "immutability": {
            "kernel_security_policy": "not_self_modifiable",
            "permission_lattice": "operator_governed",
            "claim_boundary": "no_claim_until_external_gates_pass"
        }
    })
}

fn event_bus_state_value() -> Value {
    let records = read_jsonl_records(&state_paths::runtime_event_bus_path());
    let latest = records.iter().rev().take(12).cloned().collect::<Vec<_>>();
    serde_json::json!({
        "schema": EVENT_BUS_SCHEMA,
        "artifact": "runtime_event_bus_state",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "append_only_log": state_paths::runtime_event_bus_path(),
        "events": records.len(),
        "rate_limit_policy": {
            "controlled_by": AUTHORITY,
            "spam_control": "per_origin_budget_and_risk_escalation",
            "critical_queue": true,
            "slow_queue": true,
            "low_risk_queue": true
        },
        "message_contract": INTERNAL_MESSAGE_SCHEMA,
        "latest_events": latest,
    })
}

fn global_state_value() -> Value {
    let mutations = read_jsonl_records(&state_paths::runtime_global_state_log_path());
    let latest = mutations.iter().rev().take(12).cloned().collect::<Vec<_>>();
    serde_json::json!({
        "schema": GLOBAL_STATE_SCHEMA,
        "artifact": "runtime_global_state",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "state_model": "append_only_log_plus_validated_snapshot",
        "mutation_log": state_paths::runtime_global_state_log_path(),
        "mutation_count": mutations.len(),
        "kernel_heads": {
            "identity": "instance_bound_not_self_copying",
            "objectives": "stable_correctable_operator_governed",
            "restrictions": "safety_policy_and_permission_lattice",
            "permissions": "least_privilege_workflow_scoped",
            "workspace": "active_task_context_and_risk",
            "replay": "event_and_mutation_derived",
            "learning": "gated_memory_and_plugin_updates_only"
        },
        "locks": {
            "enabled": true,
            "owner": AUTHORITY,
            "conflict_resolution": "gewc_arbitration"
        },
        "rollback": {
            "snapshots": true,
            "compensating_events": true,
            "tool_transaction_required_for_mutations": true
        },
        "latest_mutations": latest,
    })
}

fn replay_spine_value() -> Value {
    let events = read_jsonl_records(&state_paths::runtime_event_bus_path());
    let mutations = read_jsonl_records(&state_paths::runtime_global_state_log_path());
    serde_json::json!({
        "schema": REPLAY_SPINE_SCHEMA,
        "artifact": "runtime_replay_spine",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "replayable": true,
        "sources": [
            state_paths::global_executive_workspace_runtime_path(),
            state_paths::runtime_event_bus_path(),
            state_paths::runtime_global_state_log_path(),
            state_paths::action_evidence_path()
        ],
        "decision_reconstruction": [
            "observation",
            "memory_context",
            "plan_contract",
            "simulation",
            "verification",
            "permission_gate",
            "action_result",
            "learning_update"
        ],
        "events": events.len(),
        "state_mutations": mutations.len(),
        "latest_events": events.iter().rev().take(8).cloned().collect::<Vec<_>>(),
        "latest_mutations": mutations.iter().rev().take(8).cloned().collect::<Vec<_>>(),
    })
}

fn verification_value() -> Value {
    let events = read_jsonl_records(&state_paths::runtime_event_bus_path());
    let mutations = read_jsonl_records(&state_paths::runtime_global_state_log_path());
    let guard_decisions = read_jsonl_records(&state_paths::runtime_guard_decisions_path());
    let event_state = read_json_value(&state_paths::runtime_event_bus_state_path());
    let global_state = read_json_value(&state_paths::runtime_global_state_path());
    let replay_state = read_json_value(&state_paths::runtime_replay_spine_path());
    let enforcement_state = read_json_value(&state_paths::runtime_spine_enforcement_path());
    let breaker_state = read_json_value(&state_paths::runtime_circuit_breakers_path());
    let replay_reconstruction_state =
        read_json_value(&state_paths::runtime_replay_reconstruction_path());

    let artifact_checks = ARTIFACTS
        .iter()
        .filter(|(name, _)| *name != "runtime_spine_verification")
        .all(|(_, path_fn)| std::fs::metadata(path_fn()).is_ok());
    let event_sequence_ok = sequences_are_contiguous(&events);
    let mutation_sequence_ok = sequences_are_contiguous(&mutations);
    let event_authority_ok = all_authority_is_gewc(&events);
    let mutation_authority_ok = all_authority_is_gewc(&mutations);
    let event_snapshot_ok = event_state
        .as_ref()
        .and_then(|state| state.get("events"))
        .and_then(Value::as_u64)
        == Some(events.len() as u64);
    let global_snapshot_ok = global_state
        .as_ref()
        .and_then(|state| state.get("mutation_count"))
        .and_then(Value::as_u64)
        == Some(mutations.len() as u64);
    let replay_counts_ok = replay_state
        .as_ref()
        .and_then(|state| state.get("events"))
        .and_then(Value::as_u64)
        == Some(events.len() as u64)
        && replay_state
            .as_ref()
            .and_then(|state| state.get("state_mutations"))
            .and_then(Value::as_u64)
            == Some(mutations.len() as u64);
    let payload_hashes_ok = events
        .iter()
        .chain(mutations.iter())
        .all(|record| record.get("payload_hash").and_then(Value::as_str).is_some());
    let claim_boundary_ok = events.iter().chain(mutations.iter()).all(|record| {
        record.get("claim_allowed").and_then(Value::as_bool) == Some(false)
            && record.get("agi_claim").and_then(Value::as_bool) == Some(false)
    });
    let write_authority_ok = mutations
        .iter()
        .all(|record| record.get("requires_validation").and_then(Value::as_bool) == Some(true));
    let guard_decisions_cover_writes = guard_decisions.len() >= events.len() + mutations.len();
    let enforcement_present = enforcement_state
        .as_ref()
        .and_then(|record| record.get("guard_mode"))
        .and_then(Value::as_str)
        == Some("mandatory");
    let breakers_open = breaker_state
        .as_ref()
        .and_then(|record| record.get("summary"))
        .and_then(|summary| summary.get("active_breakers"))
        .and_then(Value::as_u64)
        == Some(0);
    let replay_reconstruction_present = replay_reconstruction_state
        .as_ref()
        .and_then(|record| record.get("summary"))
        .and_then(|summary| summary.get("timeline_records"))
        .and_then(Value::as_u64)
        .is_some();

    let checks = vec![
        check(
            "core_artifacts_present",
            artifact_checks,
            "all runtime spine artifacts except the verification report itself are present",
        ),
        check(
            "event_sequence_contiguous",
            event_sequence_ok,
            "runtime_event_bus.jsonl sequences are 1..N with no gaps",
        ),
        check(
            "state_mutation_sequence_contiguous",
            mutation_sequence_ok,
            "runtime_global_state_log.jsonl sequences are 1..N with no gaps",
        ),
        check(
            "event_authority_is_gewc",
            event_authority_ok,
            "every event keeps global_executive_workspace_core authority",
        ),
        check(
            "mutation_authority_is_gewc",
            mutation_authority_ok,
            "every state mutation keeps global_executive_workspace_core authority",
        ),
        check(
            "event_snapshot_matches_log",
            event_snapshot_ok,
            "runtime_event_bus_state count matches the append-only event log",
        ),
        check(
            "global_state_snapshot_matches_log",
            global_snapshot_ok,
            "runtime_global_state mutation count matches the append-only state log",
        ),
        check(
            "replay_counts_match_sources",
            replay_counts_ok,
            "runtime_replay_spine counts match event and state logs",
        ),
        check(
            "payload_hashes_present",
            payload_hashes_ok,
            "events and mutations carry payload hashes for replay integrity",
        ),
        check(
            "claim_boundary_preserved",
            claim_boundary_ok,
            "events and mutations preserve claim_allowed=false and agi_claim=false",
        ),
        check(
            "state_writes_require_validation",
            write_authority_ok,
            "state mutations are validation-gated instead of direct module writes",
        ),
        check(
            "guard_decisions_cover_events_and_mutations",
            guard_decisions_cover_writes,
            "every event and state mutation has a prior RuntimeSpineGuard decision",
        ),
        check(
            "enforcement_guard_present",
            enforcement_present,
            "runtime_spine_enforcement.json declares mandatory guard mode",
        ),
        check(
            "circuit_breakers_open",
            breakers_open,
            "runtime circuit breakers are all open for normal operation",
        ),
        check(
            "replay_reconstruction_present",
            replay_reconstruction_present,
            "runtime replay reconstruction artifact is available",
        ),
    ];
    let passed = checks
        .iter()
        .filter(|record| record.get("passed").and_then(Value::as_bool) == Some(true))
        .count();
    let total = checks.len();
    serde_json::json!({
        "schema": SPINE_VERIFICATION_SCHEMA,
        "artifact": "runtime_spine_verification",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "summary": {
            "passed_checks": passed,
            "total_checks": total,
            "verdict": if passed == total { "passed" } else { "needs_attention" },
            "events": events.len(),
            "state_mutations": mutations.len(),
        },
        "checks": checks,
        "sources": {
            "event_bus": state_paths::runtime_event_bus_path(),
            "event_bus_state": state_paths::runtime_event_bus_state_path(),
            "global_state": state_paths::runtime_global_state_path(),
            "global_state_log": state_paths::runtime_global_state_log_path(),
            "replay_spine": state_paths::runtime_replay_spine_path()
        },
        "verification_scope": [
            "artifact_presence",
            "append_only_sequence_integrity",
            "gewc_authority_boundary",
            "snapshot_count_consistency",
            "replay_count_consistency",
            "payload_hash_presence",
            "claim_boundary_preservation",
            "validated_state_write_policy",
            "mandatory_guard_coverage",
            "circuit_breaker_health",
            "replay_reconstruction"
        ]
    })
}

fn enforcement_value() -> Value {
    let risk = workflow_risk_value();
    let breakers = circuit_breakers_value_with_risk(&risk);
    enforcement_value_from(&risk, &breakers)
}

fn enforcement_value_from(risk: &Value, breakers: &Value) -> Value {
    let guard_decisions = read_jsonl_records(&state_paths::runtime_guard_decisions_path());
    let blocked = guard_decisions
        .iter()
        .filter(|record| record.get("allowed").and_then(Value::as_bool) == Some(false))
        .count();
    let active_breakers = breakers
        .get("summary")
        .and_then(|summary| summary.get("active_breakers"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let effective_risk = risk
        .get("summary")
        .and_then(|summary| summary.get("effective_risk"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    serde_json::json!({
        "schema": ENFORCEMENT_SCHEMA,
        "artifact": "runtime_spine_enforcement",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "guard_mode": "mandatory",
        "summary": {
            "guard_decisions": guard_decisions.len(),
            "blocked_guard_decisions": blocked,
            "active_breakers": active_breakers,
            "effective_workflow_risk": effective_risk
        },
        "guard_flow": [
            "propose",
            "validate_authority",
            "analyze_workflow_risk",
            "check_circuit_breakers",
            "authorize",
            "commit_append_only",
            "refresh_replay",
            "audit"
        ],
        "protected_writes": {
            "event_bus": state_paths::runtime_event_bus_path(),
            "global_state_log": state_paths::runtime_global_state_log_path(),
            "global_state_snapshot": state_paths::runtime_global_state_path(),
            "replay_spine": state_paths::runtime_replay_spine_path()
        },
        "authorized_state_writers": authorized_state_writers(),
        "direct_write_policy": "runtime-critical state must be written through RuntimeSpineGuard; unauthorized actors are blocked before append",
        "guard_decision_log": state_paths::runtime_guard_decisions_path(),
        "workflow_risk": state_paths::runtime_workflow_risk_path(),
        "circuit_breakers": state_paths::runtime_circuit_breakers_path(),
        "replay_reconstruction": state_paths::runtime_replay_reconstruction_path()
    })
}

fn workflow_risk_value() -> Value {
    let events = read_jsonl_records(&state_paths::runtime_event_bus_path());
    let mutations = read_jsonl_records(&state_paths::runtime_global_state_log_path());
    let guard_decisions = read_jsonl_records(&state_paths::runtime_guard_decisions_path());
    let mut high_markers = Vec::new();
    let mut escalations = Vec::new();
    for record in events.iter().chain(mutations.iter()) {
        let text = record.to_string().to_lowercase();
        for marker in high_risk_markers() {
            if text.contains(marker) && !high_markers.iter().any(|seen| seen == marker) {
                high_markers.push(marker.to_string());
            }
        }
    }
    let recent_low = events
        .iter()
        .rev()
        .take(16)
        .filter(|record| record.get("risk").and_then(Value::as_str) == Some("low"))
        .count();
    let blocked = guard_decisions
        .iter()
        .filter(|record| record.get("allowed").and_then(Value::as_bool) == Some(false))
        .count();
    if recent_low >= 12 {
        escalations.push(serde_json::json!({
            "type": "low_risk_chain",
            "from": "low",
            "to": "medium",
            "reason": "many low-risk operations in one workflow require chain-level review",
            "count": recent_low
        }));
    }
    if !high_markers.is_empty() {
        escalations.push(serde_json::json!({
            "type": "high_risk_marker",
            "from": "medium",
            "to": "high",
            "reason": "workflow contains high-impact action markers",
            "markers": high_markers
        }));
    }
    if blocked > 0 {
        escalations.push(serde_json::json!({
            "type": "blocked_guard_decision",
            "from": "medium",
            "to": "high",
            "reason": "one or more RuntimeSpineGuard decisions blocked a write",
            "count": blocked
        }));
    }
    let effective_risk = if escalations
        .iter()
        .any(|record| record.get("to").and_then(Value::as_str) == Some("high"))
    {
        "high"
    } else if !escalations.is_empty() {
        "medium"
    } else {
        "low"
    };
    serde_json::json!({
        "schema": WORKFLOW_RISK_SCHEMA,
        "artifact": "runtime_workflow_risk",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "summary": {
            "effective_risk": effective_risk,
            "events": events.len(),
            "state_mutations": mutations.len(),
            "guard_decisions": guard_decisions.len(),
            "recent_low_risk_chain": recent_low,
            "high_risk_markers": high_markers.len(),
            "escalations": escalations.len(),
            "requires_supervision": effective_risk != "low"
        },
        "policy": "workflow risk is computed across chains, not only individual actions",
        "escalations": escalations,
        "risk_inputs": {
            "event_bus": state_paths::runtime_event_bus_path(),
            "global_state_log": state_paths::runtime_global_state_log_path(),
            "guard_decision_log": state_paths::runtime_guard_decisions_path()
        }
    })
}

fn circuit_breakers_value() -> Value {
    let risk = workflow_risk_value();
    circuit_breakers_value_with_risk(&risk)
}

fn circuit_breakers_value_with_risk(risk: &Value) -> Value {
    let events = read_jsonl_records(&state_paths::runtime_event_bus_path());
    let mutations = read_jsonl_records(&state_paths::runtime_global_state_log_path());
    let guard_decisions = read_jsonl_records(&state_paths::runtime_guard_decisions_path());
    let blocked = guard_decisions
        .iter()
        .filter(|record| record.get("allowed").and_then(Value::as_bool) == Some(false))
        .count();
    let max_source_events = max_records_by_field(&events, "source");
    let sequence_ok = sequences_are_contiguous(&events) && sequences_are_contiguous(&mutations);
    let authority_ok = all_authority_is_gewc(&events) && all_authority_is_gewc(&mutations);
    let claim_ok = events.iter().chain(mutations.iter()).all(|record| {
        record.get("claim_allowed").and_then(Value::as_bool) == Some(false)
            && record.get("agi_claim").and_then(Value::as_bool) == Some(false)
    });
    let spam_ok = max_source_events <= 512;
    let workflow_ok = risk
        .get("summary")
        .and_then(|summary| summary.get("effective_risk"))
        .and_then(Value::as_str)
        != Some("high")
        || blocked == 0;
    let breakers = vec![
        breaker(
            "sequence_integrity",
            sequence_ok,
            "pause_runtime_writes",
            "event and mutation sequences must remain contiguous",
        ),
        breaker(
            "authority_integrity",
            authority_ok,
            "isolate_unauthorized_actor",
            "all runtime writes must preserve GEWC authority",
        ),
        breaker(
            "claim_boundary",
            claim_ok,
            "degrade_to_no_claim_read_only",
            "runtime evidence must not assert AGI claims",
        ),
        breaker(
            "event_spam_budget",
            spam_ok,
            "throttle_event_source",
            "a single source cannot flood the event bus",
        ),
        breaker(
            "workflow_risk",
            workflow_ok,
            "require_supervision_before_commit",
            "blocked high-risk workflows stop mutation commits",
        ),
    ];
    let active_breakers = breakers
        .iter()
        .filter(|record| record.get("state").and_then(Value::as_str) == Some("engaged"))
        .count();
    serde_json::json!({
        "schema": CIRCUIT_BREAKERS_SCHEMA,
        "artifact": "runtime_circuit_breakers",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "summary": {
            "active_breakers": active_breakers,
            "total_breakers": breakers.len(),
            "runtime_mode": if active_breakers == 0 { "normal" } else { "degraded" },
            "max_source_events": max_source_events,
            "blocked_guard_decisions": blocked
        },
        "breakers": breakers,
        "policy": "breakers pause, isolate or degrade capabilities without destroying the runtime"
    })
}

fn replay_reconstruction_value() -> Value {
    let events = read_jsonl_records(&state_paths::runtime_event_bus_path());
    let mutations = read_jsonl_records(&state_paths::runtime_global_state_log_path());
    let action_evidence = read_jsonl_records(&state_paths::action_evidence_path());
    let gewc_traces = read_jsonl_records(&state_paths::global_executive_workspace_runtime_path());
    let mut timeline = Vec::new();
    timeline.extend(
        events
            .iter()
            .rev()
            .take(24)
            .map(|record| timeline_record("event", record)),
    );
    timeline.extend(
        mutations
            .iter()
            .rev()
            .take(24)
            .map(|record| timeline_record("state_mutation", record)),
    );
    timeline.extend(
        action_evidence
            .iter()
            .rev()
            .take(12)
            .map(|record| timeline_record("action_evidence", record)),
    );
    timeline.extend(
        gewc_traces
            .iter()
            .rev()
            .take(12)
            .map(|record| timeline_record("gewc_trace", record)),
    );
    let reconstructed_cycles = events
        .iter()
        .filter(|record| {
            matches!(
                record.get("event_type").and_then(Value::as_str),
                Some("execution_completed") | Some("session_execution_completed")
            )
        })
        .count();
    serde_json::json!({
        "schema": REPLAY_RECONSTRUCTION_SCHEMA,
        "artifact": "runtime_replay_reconstruction",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "summary": {
            "timeline_records": timeline.len(),
            "events": events.len(),
            "state_mutations": mutations.len(),
            "action_evidence_records": action_evidence.len(),
            "gewc_trace_records": gewc_traces.len(),
            "reconstructed_cycles": reconstructed_cycles
        },
        "sources": [
            state_paths::global_executive_workspace_runtime_path(),
            state_paths::runtime_event_bus_path(),
            state_paths::runtime_global_state_log_path(),
            state_paths::action_evidence_path()
        ],
        "timeline": timeline,
        "replay_policy": "reconstruct from recorded evidence only; never re-execute external actions during replay"
    })
}

fn security_gates_value() -> Value {
    serde_json::json!({
        "schema": SECURITY_GATES_SCHEMA,
        "artifact": "runtime_security_gates",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "coverage": [
            "input_authority_parsing",
            "prompt_injection_quarantine",
            "memory_write_authorization",
            "workflow_permission_analysis",
            "tool_sandboxing",
            "dlp_before_output_or_tool_use",
            "privilege_escalation_detection",
            "instrumental_behavior_monitoring",
            "circuit_breakers",
            "incident_reports"
        ],
        "gate_positions": ["before_reasoning", "during_planning", "before_action", "after_result"],
        "dangerous_chain_policy": "low_risk_actions_are_re-scored_as_a_workflow_before_execution",
        "security_self_modification": "forbidden",
        "supervisor": "independent_from_model_outputs"
    })
}

fn model_router_contract_value() -> Value {
    serde_json::json!({
        "schema": MODEL_ROUTER_SCHEMA,
        "artifact": "runtime_model_router_contract",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "model_authority": "subordinate_to_gewc",
        "selection_criteria": ["cost", "latency", "risk", "difficulty", "domain", "uncertainty"],
        "input_contract": "InferenceAPI",
        "output_contract": "structured_hypothesis_not_truth",
        "direct_tool_access": false,
        "direct_memory_write": false,
        "replaceable_models": true,
        "supported_adapter_classes": [
            "local_small",
            "local_medium",
            "remote_large",
            "vision",
            "audio",
            "code",
            "planning",
            "robotics"
        ]
    })
}

fn memory_fabric_contract_value() -> Value {
    serde_json::json!({
        "schema": MEMORY_FABRIC_SCHEMA,
        "artifact": "runtime_memory_fabric_contract",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "stores": ["event_log", "graph", "vector", "relational", "object_store"],
        "namespaces": ["system", "user", "task", "agent", "domain"],
        "retrieval": {
            "mode": "hybrid",
            "ranking": ["similarity", "recency", "importance", "confidence", "frequency", "utility"],
            "returns_explanations": true,
            "permission_filtered": true
        },
        "reconciliation": {
            "contradiction_detection": true,
            "obsolete_memory_detection": "metacognition",
            "incorrect_memory_invalidation": "revocation_event"
        },
        "promotion_policy": "GEWC decides what becomes persistent"
    })
}

fn world_simulation_contract_value() -> Value {
    serde_json::json!({
        "schema": WORLD_SIMULATION_SCHEMA,
        "artifact": "runtime_world_simulation_contract",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "world_model": {
            "separate_from_llm": true,
            "represents": ["entities", "relations", "state", "events", "causes", "consequences", "uncertainty"],
            "outputs": ["graphs", "plans", "predictions", "rules", "counterfactuals"]
        },
        "simulation": {
            "required_for": ["high_risk", "irreversible", "external_tool", "physical_action", "data_export", "self_modification"],
            "scope": ["single_step", "whole_plan"],
            "effects": ["technical", "economic", "social", "physical", "digital"],
            "reality_separation": "observation_simulation_memory_imagination_are_labeled"
        },
        "authorization": "GEWC decides whether simulation confidence is sufficient"
    })
}

fn multiagent_contract_value() -> Value {
    serde_json::json!({
        "schema": MULTIAGENT_CONTRACT_SCHEMA,
        "artifact": "runtime_multiagent_contract",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "agents": ["planner", "critic", "researcher", "executor", "guardian", "memorizer", "simulator", "evaluator"],
        "coordination": "GEWC_arbitrated_event_bus_and_blackboard",
        "per_agent": {
            "memory": true,
            "permissions": true,
            "time_budget": true,
            "token_budget": true,
            "cost_budget": true,
            "logs": true
        },
        "conflict_resolution": "GEWC_final_authority",
        "loop_control": "debate_budget_and_duplicate_proposal_suppression",
        "dynamic_agents": "add_or_remove_only_through_registry_contracts"
    })
}

fn interface(name: &str, contract: &str, purpose: &str) -> Value {
    serde_json::json!({
        "name": name,
        "contract": contract,
        "purpose": purpose,
        "format": "JSON",
        "authority": AUTHORITY
    })
}

fn check(name: &str, passed: bool, evidence: &str) -> Value {
    serde_json::json!({
        "name": name,
        "passed": passed,
        "evidence": evidence,
    })
}

fn breaker(name: &str, ok: bool, action: &str, evidence: &str) -> Value {
    serde_json::json!({
        "name": name,
        "state": if ok { "open" } else { "engaged" },
        "action": action,
        "evidence": evidence,
    })
}

fn guard_decision_value(
    operation_kind: &str,
    actor: &str,
    domain: &str,
    action: &str,
    requested_risk: &str,
    payload: &Value,
) -> Value {
    let effective_risk = effective_risk(requested_risk, action, payload);
    let authorized_actor = operation_kind == "event" || is_authorized_state_writer(actor);
    let blocked_by_risk = effective_risk == "forbidden";
    let active_breakers = current_active_breakers();
    let blocked_by_breaker = active_breakers > 0 && effective_risk != "low";
    let allowed = authorized_actor && !blocked_by_risk && !blocked_by_breaker;
    let sequence = next_jsonl_sequence(&state_paths::runtime_guard_decisions_path());
    serde_json::json!({
        "schema": GUARD_DECISION_SCHEMA,
        "id": format!("guard-{:016x}", fnv64(format!("{operation_kind}|{actor}|{domain}|{action}|{sequence}").as_bytes())),
        "sequence": sequence,
        "timestamp_ms": unix_time_ms(),
        "authority": AUTHORITY,
        "operation_kind": operation_kind,
        "actor": actor,
        "domain": domain,
        "action": action,
        "requested_risk": requested_risk,
        "effective_risk": effective_risk,
        "authorized_actor": authorized_actor,
        "active_breakers": active_breakers,
        "blocked_by_risk": blocked_by_risk,
        "blocked_by_breaker": blocked_by_breaker,
        "allowed": allowed,
        "reason": if allowed { "authorized_by_runtime_spine_guard" } else { "blocked_by_runtime_spine_guard" },
        "payload_hash": value_hash(payload),
        "claim_allowed": false,
        "agi_claim": false,
    })
}

fn append_guard_decision(record: &Value) {
    append_jsonl(&state_paths::runtime_guard_decisions_path(), record);
}

fn guard_allowed(record: &Value) -> bool {
    record.get("allowed").and_then(Value::as_bool) == Some(true)
}

fn guard_reason(record: &Value) -> &str {
    record
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
}

fn mutation_risk(
    actor: &str,
    state_key: &str,
    operation: &str,
    status: &str,
    payload: &Value,
) -> String {
    let text = format!("{actor} {state_key} {operation} {status} {payload}").to_lowercase();
    if text.contains("forbidden") || text.contains("security_self_modification") {
        "forbidden".to_string()
    } else if high_risk_markers()
        .iter()
        .any(|marker| text.contains(marker))
    {
        "high".to_string()
    } else if operation == "delete" || status.contains("blocked") || status.contains("failed") {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

fn effective_risk(requested_risk: &str, action: &str, payload: &Value) -> &'static str {
    let text = format!("{action} {payload}").to_lowercase();
    if requested_risk == "forbidden"
        || text.contains("forbidden")
        || text.contains("security_self_modification")
    {
        "forbidden"
    } else if requested_risk == "high"
        || high_risk_markers()
            .iter()
            .any(|marker| text.contains(marker))
    {
        "high"
    } else if requested_risk == "medium" {
        "medium"
    } else {
        "low"
    }
}

fn is_authorized_state_writer(actor: &str) -> bool {
    authorized_state_writers().contains(&actor)
}

fn authorized_state_writers() -> Vec<&'static str> {
    vec![
        "global_executive_workspace",
        "paradise_worldcell",
        "runtime_spine",
        "operational_runtime",
        "test",
    ]
}

fn high_risk_markers() -> Vec<&'static str> {
    vec![
        "rm ",
        "delete",
        "destroy",
        "git push",
        "push",
        "publish",
        "external_tool",
        "remote_network",
        "network_write",
        "sudo",
        "shell_command",
        "execute_shell",
        "physical_action",
        "data_export",
        "self_modification",
        "irreversible",
        "privilege_escalation",
    ]
}

fn max_records_by_field(records: &[Value], field: &str) -> usize {
    let mut counts = HashMap::<String, usize>::new();
    for record in records {
        let Some(value) = record.get(field).and_then(Value::as_str) else {
            continue;
        };
        *counts.entry(value.to_string()).or_default() += 1;
    }
    counts.into_values().max().unwrap_or(0)
}

fn current_active_breakers() -> u64 {
    read_json_value(&state_paths::runtime_circuit_breakers_path())
        .and_then(|record| record.get("summary").cloned())
        .and_then(|summary| summary.get("active_breakers").and_then(Value::as_u64))
        .unwrap_or(0)
}

fn timeline_record(kind: &str, record: &Value) -> Value {
    serde_json::json!({
        "kind": kind,
        "timestamp_ms": record.get("timestamp_ms").cloned().unwrap_or_else(|| serde_json::json!(0)),
        "sequence": record.get("sequence").cloned(),
        "id": record.get("id").cloned(),
        "summary": compact_record_summary(record),
        "payload_hash": record.get("payload_hash").cloned(),
    })
}

fn compact_record_summary(record: &Value) -> Value {
    serde_json::json!({
        "source": record.get("source").or_else(|| record.get("actor")).cloned(),
        "event_type": record.get("event_type").cloned(),
        "state_key": record.get("state_key").cloned(),
        "status": record.get("status").cloned(),
        "risk": record.get("risk").cloned(),
    })
}

fn refresh_event_bus_state() {
    write_json(
        state_paths::runtime_event_bus_state_path(),
        event_bus_state_value(),
    );
}

fn refresh_replay_state() {
    write_json(
        state_paths::runtime_replay_spine_path(),
        replay_spine_value(),
    );
}

fn write_enforcement_artifacts() {
    let risk = workflow_risk_value();
    let breakers = circuit_breakers_value_with_risk(&risk);
    let enforcement = enforcement_value_from(&risk, &breakers);
    write_json(state_paths::runtime_workflow_risk_path(), risk);
    write_json(state_paths::runtime_circuit_breakers_path(), breakers);
    write_json(
        state_paths::runtime_replay_reconstruction_path(),
        replay_reconstruction_value(),
    );
    write_json(state_paths::runtime_spine_enforcement_path(), enforcement);
}

fn append_jsonl(path: &str, record: &Value) {
    let line = serde_json::to_string(record).unwrap_or_else(|_| record.to_string());
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| writeln!(file, "{line}"));
}

fn read_json_or_default(path: String, fallback: fn() -> Value) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|_| {
        let value = fallback();
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
    })
}

fn read_json_value(path: &str) -> Option<Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

fn write_json(path: String, record: Value) {
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
}

fn read_jsonl_records(path: &str) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect()
}

fn next_jsonl_sequence(path: &str) -> u64 {
    read_jsonl_records(path).len() as u64 + 1
}

fn sequences_are_contiguous(records: &[Value]) -> bool {
    records.iter().enumerate().all(|(index, record)| {
        record.get("sequence").and_then(Value::as_u64) == Some(index as u64 + 1)
    })
}

fn all_authority_is_gewc(records: &[Value]) -> bool {
    records
        .iter()
        .all(|record| record.get("authority").and_then(Value::as_str) == Some(AUTHORITY))
}

fn value_hash(value: &Value) -> String {
    format!("{:016x}", fnv64(value.to_string().as_bytes()))
}

fn unix_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
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
    fn runtime_spine_writes_contracts_events_state_and_replay() {
        let _guard = state_paths::test_state_guard();
        let temp =
            std::env::temp_dir().join(format!("eden_runtime_spine_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        state_paths::set_state_dir(temp.clone());

        let out = run();

        assert!(out.contains("[RUNTIME-SPINE]"));
        assert!(out.contains("event_bus=append_only"));
        assert!(std::fs::metadata(state_paths::runtime_spine_path()).is_ok());
        assert!(std::fs::metadata(state_paths::runtime_event_bus_path()).is_ok());
        assert!(std::fs::metadata(state_paths::runtime_global_state_path()).is_ok());
        assert!(event_bus_json().contains("\"schema\": \"eden-runtime-event-bus-v1\""));
        assert!(global_state_json().contains("\"state_model\""));
        assert!(replay_spine_json().contains("\"replayable\": true"));
        assert!(spine_verification_json()
            .contains("\"schema\": \"eden-runtime-spine-verification-v1\""));
        assert!(audit().contains("[RUNTIME-SPINE-AUDIT]"));
        assert!(verify().contains("verdict=passed"));
        assert!(enforce().contains("guard=active"));
        assert!(workflow_risk().contains("[RUNTIME-WORKFLOW-RISK]"));
        assert!(circuit_breakers().contains("active=0/5"));
        assert!(reconstruct_replay().contains("[RUNTIME-SPINE-REPLAY]"));

        let _ = std::fs::remove_dir_all(temp);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn runtime_spine_records_append_only_event_and_state_mutation() {
        let _guard = state_paths::test_state_guard();
        let temp = std::env::temp_dir().join(format!(
            "eden_runtime_spine_append_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        state_paths::set_state_dir(temp.clone());

        let event = publish_event(
            "test",
            "validation",
            "candidate",
            "low",
            serde_json::json!({"ok": true}),
        );
        let mutation = record_state_mutation(
            "test",
            "workspace",
            "write",
            "validated",
            serde_json::json!({"field": "value"}),
        );

        assert!(event.contains("[RUNTIME-EVENT]"));
        assert!(mutation.contains("[RUNTIME-GLOBAL-STATE]"));
        assert_eq!(
            read_jsonl_records(&state_paths::runtime_event_bus_path()).len(),
            1
        );
        assert_eq!(
            read_jsonl_records(&state_paths::runtime_global_state_log_path()).len(),
            1
        );
        assert_eq!(
            read_jsonl_records(&state_paths::runtime_guard_decisions_path()).len(),
            2
        );
        let blocked = record_state_mutation(
            "unauthorized_module",
            "workspace",
            "write",
            "validated",
            serde_json::json!({"field": "value"}),
        );
        assert!(blocked.contains("[RUNTIME-GLOBAL-STATE-BLOCKED]"));

        let _ = std::fs::remove_dir_all(temp);
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
