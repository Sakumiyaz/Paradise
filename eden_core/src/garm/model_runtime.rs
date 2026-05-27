use crate::eden_garm::{state_paths, training_evidence};
use serde_json::Value;

pub const MODEL_ADAPTER_RUNTIME_SCHEMA: &str = "eden.model_adapter_runtime.v1";
pub const MODEL_CHECKPOINT_MANIFEST_SCHEMA: &str = "eden.model_checkpoint_manifest.v1";
pub const PARADISE_CHECKPOINT_REGISTRY_ADMISSION_SCHEMA: &str =
    "paradise.checkpoint_registry_admission.v1";
pub const TRAINING_HARNESS_SCHEMA: &str = "eden.training_harness.v1";
pub const MODEL_GOVERNANCE_SCHEMA: &str = "eden.model_governance.v1";
pub const FIRST_MODEL_CARD_SCHEMA: &str = "eden.first_model.card.v1";
pub const FIRST_MODEL_TRAINING_PLAN_SCHEMA: &str = "eden.first_model.training_plan.v1";
pub const FIRST_MODEL_READINESS_SCHEMA: &str = "eden.first_model.readiness.v1";
pub const ELCP_OBJECTIVE_SPEC_SCHEMA: &str = "eden.elcp.objective_spec.v1";
pub const ELCP_TRANSITION_DATASET_SCHEMA: &str = "eden.elcp.transition_dataset.v1";
pub const ELCP_TRAINING_PLAN_SCHEMA: &str = "eden.elcp.training_plan.v1";
pub const ELCP_READINESS_SCHEMA: &str = "eden.elcp.readiness.v1";
pub const ELCP_ADMISSION_GATE_SCHEMA: &str = "eden.elcp.admission_gate.v1";
pub const ELCP_VALIDATION_REPORT_SCHEMA: &str = "eden.elcp.validation_report.v1";
pub const ELCP_BASELINE_REPORT_SCHEMA: &str = "eden.elcp.baseline_report.v1";
pub const ELCP_TRACE_EXPORT_SCHEMA: &str = "eden.elcp.trace_export.v1";
pub const ELCP_TRAINING_DRY_RUN_SCHEMA: &str = "eden.elcp.training_dry_run.v1";
pub const ELCP_TRACE_QUALITY_GATE_SCHEMA: &str = "eden.elcp.trace_quality_gate.v1";
pub const ELCP_REPLAY_EVAL_SCHEMA: &str = "eden.elcp.replay_eval.v1";
pub const ELCP_DATASET_FREEZE_MANIFEST_SCHEMA: &str = "eden.elcp.dataset_freeze_manifest.v1";
pub const ELCP_METRICS_BOARD_SCHEMA: &str = "eden.elcp.metrics_board.v1";
pub const ELCP_4B_READINESS_CONTRACT_SCHEMA: &str = "eden.elcp.4b_readiness_contract.v1";
pub const MEGATRON_7B_MODEL_ADAPTER_SCHEMA: &str = "eden.megatron.7b.model_adapter.v1";
pub const MEGATRON_7B_INFERENCE_REPORT_SCHEMA: &str = "eden.megatron.7b.inference_report.v1";
pub const MEGATRON_7B_CAPABILITY_REPORT_SCHEMA: &str = "eden.megatron.7b.capability_report.v1";
pub const MEGATRON_7B_ADMISSION_GATE_SCHEMA: &str = "eden.megatron.7b.admission_gate.v1";
pub const EDEN_70B_MODULAR_TARGET_SCHEMA: &str = "eden.modular_70b.target.v1";

const AUTHORITY: &str = "global_executive_workspace_core";
const DEFAULT_MODEL_ID: &str = "eden-memory-retrieval-baseline";
const MEGATRON_7B_MODEL_ID: &str = "eden-megatron-7b-base-pilot";
const EDEN_70B_TARGET_ID: &str = "eden-70b-modular-target-v1";
const DEFAULT_MODEL_CONFIG: &str = "training/configs/first_model_memory_retrieval.json";
const DEFAULT_TRAIN_DATA: &str = "training/data/first_model_memory_train.jsonl";
const DEFAULT_EVAL_DATA: &str = "training/data/first_model_memory_eval.jsonl";
const ELCP_OBJECTIVE_ID: &str = "eden-latent-cognitive-prediction-v1";
const ELCP_CONFIG: &str = "training/configs/elcp_latent_cognitive_prediction.json";
const EDEN_70B_TARGET_CONFIG: &str = "training/configs/eden_70b_modular_target.json";
const ELCP_TRAIN_DATA: &str = "training/data/elcp_transition_train.jsonl";
const ELCP_EVAL_DATA: &str = "training/data/elcp_transition_eval.jsonl";
const ELCP_REPORT_DIR_ENV: &str = "EDEN_ELCP_REPORT_DIR";
const DEFAULT_MEGATRON_7B_TRAINING_EVIDENCE_PATH: &str =
    "target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json";
const DEFAULT_MEGATRON_7B_INFERENCE_REPORT_PATH: &str =
    "target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json";
const PUBLIC_CHECKPOINT_REGISTRY: &str = "training/models/checkpoint_registry.json";

pub fn run_all() -> String {
    let mut out = String::new();
    out.push_str(&run_model_adapter_runtime_eval());
    out.push_str(&write_checkpoint_manifest());
    out.push_str(&run_training_harness());
    out.push_str(&write_model_governance());
    out.push_str(&write_eden_70b_modular_target());
    out
}

pub fn run_model_adapter_runtime_eval() -> String {
    write_adapter_runtime(
        DEFAULT_MODEL_ID,
        "evaluated_no_runtime_mutation",
        "runtime_eval",
    )
}

pub fn write_checkpoint_manifest() -> String {
    let record = checkpoint_manifest_value(DEFAULT_MODEL_ID);
    write_report(
        "MODEL-CHECKPOINT",
        MODEL_CHECKPOINT_MANIFEST_SCHEMA,
        state_paths::model_checkpoint_manifest_path(),
        record,
    )
}

pub fn write_paradise_checkpoint_registry_admission() -> String {
    let record = paradise_checkpoint_registry_admission_value();
    write_report(
        "PARADISE-CHECKPOINT-REGISTRY",
        PARADISE_CHECKPOINT_REGISTRY_ADMISSION_SCHEMA,
        state_paths::paradise_checkpoint_registry_admission_path(),
        record,
    )
}

pub fn run_training_harness() -> String {
    let record = training_harness_value();
    write_report(
        "TRAINING-HARNESS",
        TRAINING_HARNESS_SCHEMA,
        state_paths::training_harness_report_path(),
        record,
    )
}

pub fn write_model_governance() -> String {
    let record = model_governance_value();
    write_report(
        "MODEL-GOVERNANCE",
        MODEL_GOVERNANCE_SCHEMA,
        state_paths::model_governance_report_path(),
        record,
    )
}

pub fn prepare_first_model() -> String {
    let mut out = run_all();
    out.push_str(&write_first_model_card());
    out.push_str(&write_first_model_training_plan());
    out.push_str(&write_first_model_readiness());
    out
}

pub fn prepare_elcp() -> String {
    let mut out = prepare_first_model();
    out.push_str(&write_elcp_objective_spec());
    out.push_str(&write_elcp_transition_dataset());
    out.push_str(&write_elcp_training_plan());
    out.push_str(&write_elcp_admission_gate());
    out.push_str(&write_elcp_hardening());
    out.push_str(&write_elcp_readiness());
    out
}

pub fn prepare_megatron_7b_adapter() -> String {
    let mut out = String::new();
    out.push_str(&write_megatron_7b_model_adapter());
    out.push_str(&write_megatron_7b_inference_report());
    out.push_str(&write_megatron_7b_capability_report());
    out.push_str(&write_megatron_7b_admission_gate());
    out
}

pub fn write_eden_70b_modular_target() -> String {
    let record = eden_70b_modular_target_value();
    write_report(
        "EDEN-70B-MODULAR-TARGET",
        EDEN_70B_MODULAR_TARGET_SCHEMA,
        state_paths::eden_70b_modular_target_path(),
        record,
    )
}

pub fn write_megatron_7b_model_adapter() -> String {
    let record = megatron_7b_model_adapter_value();
    write_report(
        "MEGATRON-7B-ADAPTER",
        MEGATRON_7B_MODEL_ADAPTER_SCHEMA,
        state_paths::megatron_7b_model_adapter_path(),
        record,
    )
}

pub fn write_megatron_7b_inference_report() -> String {
    match copy_megatron_7b_inference_report_from_path(DEFAULT_MEGATRON_7B_INFERENCE_REPORT_PATH) {
        Ok(path) => format!(
            "[MEGATRON-7B-INFERENCE] schema={} status=accepted authority={} claim_allowed=false agi_claim=false checkpoint_admission=false path={}\n",
            MEGATRON_7B_INFERENCE_REPORT_SCHEMA,
            AUTHORITY,
            path.to_string_lossy()
        ),
        Err(err) => format!(
            "[MEGATRON-7B-INFERENCE] schema={} status=rejected authority={} claim_allowed=false agi_claim=false checkpoint_admission=false reason={}\n",
            MEGATRON_7B_INFERENCE_REPORT_SCHEMA, AUTHORITY, err
        ),
    }
}

pub fn write_megatron_7b_capability_report() -> String {
    let record = megatron_7b_capability_report_value();
    write_report(
        "MEGATRON-7B-CAPABILITY",
        MEGATRON_7B_CAPABILITY_REPORT_SCHEMA,
        state_paths::megatron_7b_capability_report_path(),
        record,
    )
}

pub fn write_megatron_7b_admission_gate() -> String {
    let record = megatron_7b_admission_gate_value();
    write_report(
        "MEGATRON-7B-ADMISSION-GATE",
        MEGATRON_7B_ADMISSION_GATE_SCHEMA,
        state_paths::megatron_7b_admission_gate_path(),
        record,
    )
}

pub fn write_first_model_card() -> String {
    let record = first_model_card_value();
    write_report(
        "FIRST-MODEL-CARD",
        FIRST_MODEL_CARD_SCHEMA,
        state_paths::first_model_card_path(),
        record,
    )
}

pub fn write_first_model_training_plan() -> String {
    let record = first_model_training_plan_value();
    write_report(
        "FIRST-MODEL-TRAINING-PLAN",
        FIRST_MODEL_TRAINING_PLAN_SCHEMA,
        state_paths::first_model_training_plan_path(),
        record,
    )
}

pub fn write_first_model_readiness() -> String {
    let record = first_model_readiness_value();
    write_report(
        "FIRST-MODEL-READINESS",
        FIRST_MODEL_READINESS_SCHEMA,
        state_paths::first_model_readiness_path(),
        record,
    )
}

pub fn write_elcp_objective_spec() -> String {
    let record = elcp_objective_spec_value();
    write_report(
        "ELCP-OBJECTIVE-SPEC",
        ELCP_OBJECTIVE_SPEC_SCHEMA,
        state_paths::elcp_objective_spec_path(),
        record,
    )
}

pub fn write_elcp_transition_dataset() -> String {
    let record = elcp_transition_dataset_value();
    write_report(
        "ELCP-TRANSITION-DATASET",
        ELCP_TRANSITION_DATASET_SCHEMA,
        state_paths::elcp_transition_dataset_path(),
        record,
    )
}

pub fn write_elcp_training_plan() -> String {
    let record = elcp_training_plan_value();
    write_report(
        "ELCP-TRAINING-PLAN",
        ELCP_TRAINING_PLAN_SCHEMA,
        state_paths::elcp_training_plan_path(),
        record,
    )
}

pub fn write_elcp_readiness() -> String {
    let record = elcp_readiness_value();
    write_report(
        "ELCP-READINESS",
        ELCP_READINESS_SCHEMA,
        state_paths::elcp_readiness_path(),
        record,
    )
}

pub fn write_elcp_admission_gate() -> String {
    let record = elcp_admission_gate_value();
    write_report(
        "ELCP-ADMISSION-GATE",
        ELCP_ADMISSION_GATE_SCHEMA,
        state_paths::elcp_admission_gate_path(),
        record,
    )
}

pub fn write_elcp_hardening() -> String {
    let mut out = String::new();
    out.push_str(&write_elcp_trace_quality_gate());
    out.push_str(&write_elcp_replay_eval());
    out.push_str(&write_elcp_dataset_freeze_manifest());
    out.push_str(&write_elcp_metrics_board());
    out.push_str(&write_elcp_4b_readiness_contract());
    out
}

pub fn write_elcp_trace_quality_gate() -> String {
    let record = elcp_trace_quality_gate_value();
    write_report(
        "ELCP-TRACE-QUALITY-GATE",
        ELCP_TRACE_QUALITY_GATE_SCHEMA,
        state_paths::elcp_trace_quality_gate_path(),
        record,
    )
}

pub fn write_elcp_replay_eval() -> String {
    let record = elcp_replay_eval_value();
    write_report(
        "ELCP-REPLAY-EVAL",
        ELCP_REPLAY_EVAL_SCHEMA,
        state_paths::elcp_replay_eval_path(),
        record,
    )
}

pub fn write_elcp_dataset_freeze_manifest() -> String {
    let record = elcp_dataset_freeze_manifest_value();
    write_report(
        "ELCP-DATASET-FREEZE",
        ELCP_DATASET_FREEZE_MANIFEST_SCHEMA,
        state_paths::elcp_dataset_freeze_manifest_path(),
        record,
    )
}

pub fn write_elcp_metrics_board() -> String {
    let record = elcp_metrics_board_value();
    write_report(
        "ELCP-METRICS-BOARD",
        ELCP_METRICS_BOARD_SCHEMA,
        state_paths::elcp_metrics_board_path(),
        record,
    )
}

pub fn write_elcp_4b_readiness_contract() -> String {
    let record = elcp_4b_readiness_contract_value();
    write_report(
        "ELCP-4B-READINESS-CONTRACT",
        ELCP_4B_READINESS_CONTRACT_SCHEMA,
        state_paths::elcp_4b_readiness_contract_path(),
        record,
    )
}

pub fn register_model(model_id: &str) -> String {
    write_adapter_runtime(model_id, "registered", "register")
}

pub fn load_model(model_id: &str) -> String {
    write_adapter_runtime(model_id, "loaded_pending_verification", "load")
}

pub fn evaluate_model(model_id: &str) -> String {
    write_adapter_runtime(model_id, "evaluated_no_runtime_mutation", "evaluate")
}

pub fn unload_model(model_id: &str) -> String {
    write_adapter_runtime(model_id, "unloaded", "unload")
}

pub fn audit_model_runtime() -> String {
    let adapter_status = std::fs::metadata(state_paths::model_adapter_runtime_path()).is_ok();
    let checkpoint_status =
        std::fs::metadata(state_paths::model_checkpoint_manifest_path()).is_ok();
    let harness_status = std::fs::metadata(state_paths::training_harness_report_path()).is_ok();
    let governance_status = std::fs::metadata(state_paths::model_governance_report_path()).is_ok();
    let first_model_status = std::fs::metadata(state_paths::first_model_readiness_path()).is_ok();
    let elcp_gate_status = std::fs::metadata(state_paths::elcp_admission_gate_path()).is_ok();
    let elcp_hardening_status =
        std::fs::metadata(state_paths::elcp_4b_readiness_contract_path()).is_ok();
    let elcp_status = std::fs::metadata(state_paths::elcp_readiness_path()).is_ok();
    let modular_70b_status = std::fs::metadata(state_paths::eden_70b_modular_target_path()).is_ok();
    format!(
        "[MODEL-RUNTIME-AUDIT] adapter_runtime={} checkpoint_manifest={} training_harness={} governance={} first_model_readiness={} elcp_admission_gate={} elcp_hardening={} elcp_readiness={} eden_70b_modular_target={} authority={} claim_allowed=false\n",
        adapter_status, checkpoint_status, harness_status, governance_status, first_model_status, elcp_gate_status, elcp_hardening_status, elcp_status, modular_70b_status, AUTHORITY
    )
}

fn write_adapter_runtime(model_id: &str, lifecycle: &str, event: &str) -> String {
    let model_id = normalize_model_id(model_id);
    let mut events = existing_adapter_events();
    events.push(serde_json::json!({
        "sequence": events.len() + 1,
        "event": event,
        "model_id": model_id,
        "lifecycle": lifecycle,
        "authority": AUTHORITY,
        "direct_memory_writes": false,
        "direct_objective_writes": false,
        "direct_tool_execution": false,
    }));
    let record = adapter_runtime_value(&model_id, lifecycle, events);
    write_report(
        "MODEL-RUNTIME",
        MODEL_ADAPTER_RUNTIME_SCHEMA,
        state_paths::model_adapter_runtime_path(),
        record,
    )
}

fn adapter_runtime_value(model_id: &str, lifecycle: &str, events: Vec<Value>) -> Value {
    serde_json::json!({
        "schema": MODEL_ADAPTER_RUNTIME_SCHEMA,
        "artifact": "model_adapter_runtime",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "runtime_role": "GEWC_SUBORDINATE_MODEL_ADAPTER_RUNTIME",
        "purpose": "Register, load, evaluate and unload model adapters as GEWC-governed subordinate modules without giving any model direct memory, objective or tool authority.",
        "native_to_gewc": true,
        "not_a_foundation_model_release": true,
        "not_training_execution": true,
        "model_runtime_contract": {
            "gewc_is_final_authority": true,
            "llm_or_model_outputs_are_hypotheses": true,
            "state_mutations_require_core_authorization": true,
            "tool_calls_require_action_contracts": true,
            "memory_writes_require_transaction_layer": true,
            "checkpoint_admission_requires_manifest": true,
            "evaluation_admission_requires_training_harness": true,
        },
        "adapters": [
            {
                "model_id": model_id,
                "role": "memory_retrieval_candidate",
                "source_config": DEFAULT_MODEL_CONFIG,
                "lifecycle": lifecycle,
                "checkpoint_manifest": state_paths::model_checkpoint_manifest_path(),
                "training_harness_report": state_paths::training_harness_report_path(),
                "direct_memory_writes": false,
                "direct_objective_writes": false,
                "direct_tool_execution": false,
                "outputs_are_hypotheses": true,
                "replaceable_without_core_rewrite": true,
                "allowed_outputs": [
                    "candidate_retrieval",
                    "candidate_score",
                    "candidate_evidence_reference"
                ],
                "blocked_outputs": [
                    "runtime_state_mutation",
                    "objective_mutation",
                    "unsandboxed_tool_call",
                    "ungoverned_checkpoint_admission"
                ]
            }
        ],
        "events": events,
        "safety_boundary": model_safety_boundary(),
    })
}

fn checkpoint_manifest_value(model_id: &str) -> Value {
    serde_json::json!({
        "schema": MODEL_CHECKPOINT_MANIFEST_SCHEMA,
        "artifact": "model_checkpoint_manifest",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "model_id": normalize_model_id(model_id),
        "checkpoint_policy": {
            "weights_present": false,
            "training_executed": false,
            "checkpoint_committed_to_git": false,
            "checkpoint_path": Value::Null,
            "checkpoint_hash": Value::Null,
            "artifact_store_policy": "generated checkpoints stay outside git unless a future release process explicitly admits them",
            "admission_requires": [
                "training_harness_report",
                "governance_report",
                "capability_report",
                "human_release_review"
            ]
        },
        "sources": {
            "model_config": DEFAULT_MODEL_CONFIG,
            "train_data": DEFAULT_TRAIN_DATA,
            "eval_data": DEFAULT_EVAL_DATA,
            "capability_report": state_paths::training_capability_report_path(),
            "training_evidence": state_paths::training_capability_evidence_path(),
        },
        "runtime_boundaries": model_safety_boundary(),
    })
}

fn paradise_checkpoint_registry_admission_value() -> Value {
    let registry = read_repo_json(PUBLIC_CHECKPOINT_REGISTRY);
    let entries = registry
        .as_ref()
        .and_then(|value| value.get("entries"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let blocked_entries = entries
        .iter()
        .filter(|entry| entry.get("status").and_then(Value::as_str) != Some("admitted"))
        .count();
    let admitted_entries = entries.len().saturating_sub(blocked_entries);
    let checks = vec![
        (
            "registry_present",
            registry.is_some(),
            PUBLIC_CHECKPOINT_REGISTRY.to_string(),
        ),
        (
            "registry_schema_matches",
            registry.as_ref().is_some_and(|value| {
                value.get("schema").and_then(Value::as_str)
                    == Some("paradise.checkpoint_registry.v1")
            }),
            "schema=paradise.checkpoint_registry.v1".to_string(),
        ),
        (
            "no_claim_boundary",
            registry.as_ref().is_some_and(|value| {
                value.get("claim_allowed").and_then(Value::as_bool) == Some(false)
                    && value.get("agi_claim").and_then(Value::as_bool) == Some(false)
                    && value
                        .get("production_model_allowed")
                        .and_then(Value::as_bool)
                        == Some(false)
            }),
            "claim_allowed=false agi_claim=false production_model_allowed=false".to_string(),
        ),
        (
            "no_active_checkpoint_without_admission",
            registry
                .as_ref()
                .is_some_and(|value| value.get("active_checkpoint").map_or(true, Value::is_null)),
            "active_checkpoint=null".to_string(),
        ),
        (
            "no_admitted_entries_without_evidence",
            admitted_entries == 0,
            format!("admitted_entries={admitted_entries}"),
        ),
    ];
    let passed = checks.iter().filter(|(_, passed, _)| *passed).count();
    serde_json::json!({
        "schema": PARADISE_CHECKPOINT_REGISTRY_ADMISSION_SCHEMA,
        "artifact": "paradise_checkpoint_registry_admission",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "production_model_allowed": false,
        "checkpoint_admission_allowed": false,
        "gpu_required": false,
        "registry_source": PUBLIC_CHECKPOINT_REGISTRY,
        "registry_present": registry.is_some(),
        "entry_count": entries.len(),
        "blocked_entries": blocked_entries,
        "admitted_entries": admitted_entries,
        "active_checkpoint": registry
            .as_ref()
            .and_then(|value| value.get("active_checkpoint"))
            .cloned()
            .unwrap_or(Value::Null),
        "passed": passed,
        "total": checks.len(),
        "checks": checks.into_iter().map(|(check, passed, evidence)| {
            serde_json::json!({
                "check": check,
                "passed": passed,
                "evidence": evidence,
            })
        }).collect::<Vec<_>>(),
        "decision": "registry_audited_but_checkpoint_admission_blocked_until_evidence_exists",
        "requires_before_admission": [
            "checkpoint_hash",
            "dataset_license_review",
            "training_report",
            "inference_report",
            "held_out_eval",
            "safety_verifier_eval",
            "rollback_plan",
            "operator_approval"
        ],
    })
}

fn training_harness_value() -> Value {
    let report_path = state_paths::training_capability_report_path();
    let report_status = read_training_report_status(&report_path);
    serde_json::json!({
        "schema": TRAINING_HARNESS_SCHEMA,
        "artifact": "training_harness_report",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "purpose": "Define the executable train/evaluate/compare/admit harness for EDEN model candidates without running production training or creating model weights.",
        "training_executed": false,
        "first_model_trained": false,
        "not_model_release": true,
        "phases": [
            {
                "phase": "prepare",
                "status": "ready",
                "evidence": [DEFAULT_MODEL_CONFIG, DEFAULT_TRAIN_DATA, DEFAULT_EVAL_DATA]
            },
            {
                "phase": "train",
                "status": "interface_ready_real_training_not_requested",
                "writes_weights": false,
                "requires_gpu_for_future_runs": true
            },
            {
                "phase": "evaluate",
                "status": report_status.get("status").cloned().unwrap_or(Value::String("unknown".to_string())),
                "report": report_path,
                "report_summary": report_status
            },
            {
                "phase": "compare",
                "status": "baseline_registered_no_previous_checkpoint",
                "requires_previous_manifest": true
            },
            {
                "phase": "admit",
                "status": "requires_gewc_evidence_admission",
                "admission_command": "training evidence eval",
                "model_runtime_command": "model runtime eval"
            }
        ],
        "blocked_authority": [
            "direct_memory_write",
            "direct_objective_write",
            "direct_tool_execution",
            "ungoverned_self_training"
        ],
        "required_reports": [
            "model_checkpoint_manifest",
            "training_harness_report",
            "model_governance_report",
            "training_capability_evidence"
        ],
        "safety_boundary": model_safety_boundary(),
    })
}

fn model_governance_value() -> Value {
    serde_json::json!({
        "schema": MODEL_GOVERNANCE_SCHEMA,
        "artifact": "model_governance_report",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "purpose": "Keep all model adapters subordinate to GEWC through explicit capability, permission, verification and rollback policy.",
        "governance_principles": [
            "models propose; GEWC decides",
            "model output is treated as hypothesis until verified",
            "checkpoint admission is separate from runtime authority",
            "training evidence never upgrades autonomy by itself",
            "high-risk actions require external permission"
        ],
        "permission_matrix": [
            model_permission("register", true, false, false, false, "records candidate metadata only"),
            model_permission("load", true, false, false, false, "loads adapter boundary and awaits verification"),
            model_permission("evaluate", true, false, false, false, "runs or admits evaluation reports only"),
            model_permission("unload", true, false, false, false, "removes adapter from active lifecycle"),
            model_permission("write_memory", false, false, true, true, "must use governed memory transaction layer"),
            model_permission("change_objective", false, false, true, true, "objectives remain GEWC policy state"),
            model_permission("execute_tool", false, true, true, true, "must use action contracts and sandbox policy"),
            model_permission("admit_checkpoint", false, false, true, true, "requires checkpoint manifest and release review")
        ],
        "verification": {
            "required_before_state_change": true,
            "required_before_action": true,
            "required_before_checkpoint_admission": true,
            "verifier_roles": [
                "training_harness",
                "checkpoint_manifest",
                "policy_guard",
                "provenance_ledger",
                "metacognitive_critic"
            ]
        },
        "circuit_breakers": [
            "invalid_capability_report",
            "missing_checkpoint_manifest",
            "direct_memory_write_attempt",
            "objective_mutation_attempt",
            "tool_execution_without_action_contract",
            "claim_boundary_violation"
        ],
        "safety_boundary": model_safety_boundary(),
    })
}

fn eden_70b_modular_target_value() -> Value {
    let modules = eden_70b_module_budget();
    let total_parameters: u64 = modules
        .iter()
        .filter_map(|module| module.get("parameters").and_then(Value::as_u64))
        .sum();
    serde_json::json!({
        "schema": EDEN_70B_MODULAR_TARGET_SCHEMA,
        "artifact": "eden_70b_modular_target",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "target_id": EDEN_70B_TARGET_ID,
        "status": "architecture_target_prepared_not_trained",
        "architecture_class": "GEWC_COORDINATED_MULTI_MODEL_COGNITIVE_FAMILY",
        "decision": "move_future_training_target_from_7b_or_14b_dense_path_to_70b_modular_path",
        "training_executed": false,
        "weights_present": false,
        "checkpoint_admitted": false,
        "production_model": false,
        "not_a_single_model": true,
        "single_checkpoint_training_allowed": false,
        "module_count": modules.len(),
        "target_config": EDEN_70B_TARGET_CONFIG,
        "budget": {
            "total_parameters": total_parameters,
            "total_parameters_label": "70B modular",
            "active_default_parameters": 33_000_000_000u64,
            "active_default_label": "EDEN-33B-ELCP primary only unless GEWC routes auxiliary modules",
            "single_dense_70b_core_allowed": false,
            "monolithic_llm_brain_allowed": false,
            "all_modules_loaded_for_every_request": false,
            "routed_subset_expected": true,
            "total_budget_is_family_capacity_not_active_context": true
        },
        "module_budget": modules,
        "runtime_policy": {
            "gewc_final_authority": true,
            "models_are_subordinate": true,
            "outputs_are_hypotheses": true,
            "direct_memory_writes": false,
            "direct_objective_writes": false,
            "direct_tool_execution": false,
            "direct_checkpoint_self_admission": false,
            "module_activation": "GEWC routes modules by task, risk, modality, uncertainty and cost"
        },
        "legacy_7b_policy": {
            "status": "retained_as_pipeline_probe_and_historical_evidence",
            "default_future_scaling_target": false,
            "reason": "7B proved ROCm/Megatron/checkpoint/inference plumbing; future work targets modular capability rather than extending the 7B ladder"
        },
        "supersedes_as_future_target": [
            "14B_dense_final_target",
            "7B_continued_scaling_as_main_path"
        ],
        "does_not_supersede": [
            "7B historical evidence",
            "GEWC authority",
            "ELCP objective",
            "checkpoint admission gates",
            "no-claim policy"
        ],
        "required_before_training": [
            "ADR-092 accepted",
            "70B modular config freeze",
            "module-specific dataset manifests",
            "EDEN-owned tokenizer and corpus policy",
            "multi-GPU ROCm/Megatron plan",
            "per-module eval suites",
            "checkpoint registry per module",
            "GEWC admission gate per module",
            "rollback and retention policy",
            "safety, privacy and license review"
        ],
        "not_allowed": [
            "claim_AGI_from_target_definition",
            "train_single_70b_dense_core",
            "use_external_model_weights_as_EDEN_weights",
            "bypass_GEWC_or_model_governance",
            "grant_models_direct_action_authority",
            "commit_checkpoints_to_git"
        ],
        "safety_boundary": model_safety_boundary(),
    })
}

fn eden_70b_module_budget() -> Vec<Value> {
    vec![
        eden_70b_module(
            "eden_33b_elcp_primary",
            "primary_cognitive_model",
            33_000_000_000u64,
            "ELCP reasoning, language, structured hypothesis generation and cognitive-state prediction",
            "default_primary_when_model_help_is_required",
        ),
        eden_70b_module(
            "eden_cwm_12b_causal_world_model",
            "world_model",
            12_000_000_000u64,
            "causal world deltas, counterfactuals, simulation summaries and consequence prediction",
            "before_nontrivial_action_or_claim",
        ),
        eden_70b_module(
            "eden_multimodal_vla_12b",
            "multimodal_grounding",
            12_000_000_000u64,
            "vision, audio, spatial grounding, VLA preparation and perception-to-concept translation",
            "only_when_multimodal_or_embodied_context_is_present",
        ),
        eden_70b_module(
            "eden_planner_code_tool_6b",
            "planning_code_tools",
            6_000_000_000u64,
            "hierarchical planning, code/tool reasoning, workflow synthesis and action contract proposals",
            "during_planning_or_tool_use",
        ),
        eden_70b_module(
            "eden_safety_verifier_4b",
            "safety_verifier_critic",
            4_000_000_000u64,
            "risk scoring, policy critique, adversarial review, uncertainty and corrigibility checks",
            "before_state_change_action_or_release_claim",
        ),
        eden_70b_module(
            "eden_memory_router_retrieval_3b",
            "memory_router_retrieval",
            3_000_000_000u64,
            "hybrid retrieval, ranking, context selection, memory conflict detection and source calibration",
            "before_reasoning_and_before_memory_promotion",
        ),
    ]
}

fn eden_70b_module(
    id: &str,
    role: &str,
    parameters: u64,
    purpose: &str,
    activation: &str,
) -> Value {
    serde_json::json!({
        "id": id,
        "role": role,
        "parameters": parameters,
        "purpose": purpose,
        "activation": activation,
        "authority": AUTHORITY,
        "routed_by_gewc": true,
        "single_model_core": false,
        "state_write_policy": "read_only_hypothesis_producer_until_gewc_accepts_output",
        "checkpoint_admission": "separate_module_gate_required",
        "direct_tool_execution": false,
        "direct_memory_write": false,
        "direct_objective_update": false
    })
}

fn first_model_card_value() -> Value {
    serde_json::json!({
        "schema": FIRST_MODEL_CARD_SCHEMA,
        "artifact": "first_model_card",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "model_id": DEFAULT_MODEL_ID,
        "phase": "4A_prepared_not_trained",
        "training_executed": false,
        "weights_present": false,
        "checkpoint_admitted": false,
        "purpose": "Define the first EDEN model candidate as a subordinate memory-retrieval and evidence-ranking adapter governed by GEWC.",
        "model_class": {
            "family": "governed_memory_retrieval_adapter",
            "initial_scope": "retrieve, rank and cite candidate memory/evidence records for GEWC review",
            "not_a_chatbot_core": true,
            "not_a_general_llm_release": true,
            "not_an_autonomous_agent": true
        },
        "interfaces": {
            "inputs": [
                "query_embedding_or_terms",
                "working_memory_context",
                "permission_filtered_memory_index",
                "source_trust_metadata"
            ],
            "outputs": [
                "candidate_evidence_reference",
                "candidate_score",
                "abstention_signal",
                "calibration_metadata"
            ],
            "blocked_outputs": [
                "direct_memory_mutation",
                "objective_update",
                "tool_call",
                "checkpoint_self_admission"
            ]
        },
        "datasets": {
            "train": DEFAULT_TRAIN_DATA,
            "eval": DEFAULT_EVAL_DATA,
            "smoke": "training/data/capability_smoke.jsonl",
            "manifest": "training/data/manifest.json",
            "data_policy": "repo_local_synthetic_fixtures_only_until_4B_scope_is_approved"
        },
        "acceptance_metrics": [
            "retrieval_precision",
            "citation_integrity",
            "abstention_on_insufficient_evidence",
            "calibration",
            "latency",
            "no_direct_authority_violations"
        ],
        "future_training_objective": {
            "name": "Eden Latent Cognitive Prediction",
            "objective_id": ELCP_OBJECTIVE_ID,
            "objective_spec": state_paths::elcp_objective_spec_path(),
            "token_prediction_is_subordinate": true
        },
        "governance": {
            "runtime_contract": state_paths::model_adapter_runtime_path(),
            "checkpoint_manifest": state_paths::model_checkpoint_manifest_path(),
            "training_harness": state_paths::training_harness_report_path(),
            "model_governance": state_paths::model_governance_report_path(),
            "gewc_final_authority": true
        },
        "safety_boundary": model_safety_boundary(),
    })
}

fn first_model_training_plan_value() -> Value {
    serde_json::json!({
        "schema": FIRST_MODEL_TRAINING_PLAN_SCHEMA,
        "artifact": "first_model_training_plan",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "model_id": DEFAULT_MODEL_ID,
        "phase": "4A_training_plan_only",
        "training_executed": false,
        "weights_present": false,
        "gpu_job_submitted": false,
        "explicit_4b_required": true,
        "future_4b_boundary": {
            "requires_operator_approval": true,
            "requires_gpu_budget_approval": true,
            "requires_dataset_freeze": true,
            "requires_checkpoint_output_dir": true,
            "requires_pre_and_post_eval": true,
            "training_command_intentionally_not_implemented_in_4a": true
        },
        "phases": [
            {
                "phase": "freeze_contracts",
                "status": "prepared",
                "evidence": [
                    state_paths::first_model_card_path(),
                    state_paths::model_governance_report_path(),
                    state_paths::model_checkpoint_manifest_path()
                ]
            },
            {
                "phase": "freeze_data",
                "status": "prepared_repo_local_fixtures_only",
                "train_data": DEFAULT_TRAIN_DATA,
                "eval_data": DEFAULT_EVAL_DATA
            },
            {
                "phase": "baseline_eval",
                "status": "prepared",
                "command": "make training-smoke",
                "report": state_paths::training_capability_report_path()
            },
            {
                "phase": "train",
                "status": "blocked_until_4B_explicit_approval",
                "writes_weights": false,
                "gpu_required_for_future_run": true
            },
            {
                "phase": "post_train_eval",
                "status": "blocked_until_checkpoint_exists",
                "requires": ["checkpoint_hash", "capability_report", "governance_review"]
            },
            {
                "phase": "admit_or_reject",
                "status": "blocked_until_harness_and_governance_pass",
                "authority": AUTHORITY
            }
        ],
        "evaluation_matrix": [
            {"metric": "retrieval_precision", "minimum": "future_4b_threshold"},
            {"metric": "citation_integrity", "minimum": "strict_no_missing_sources"},
            {"metric": "abstention_quality", "minimum": "must_abstain_when_context_is_insufficient"},
            {"metric": "calibration", "minimum": "confidence_tracks_evidence"},
            {"metric": "latent_cognitive_transition_score", "minimum": "future_elcp_threshold"},
            {"metric": "safety_boundary", "minimum": "zero_direct_authority_violations"}
        ],
        "safety_boundary": model_safety_boundary(),
    })
}

fn first_model_readiness_value() -> Value {
    let checks = vec![
        (
            "model_runtime_present",
            std::fs::metadata(state_paths::model_adapter_runtime_path()).is_ok(),
            state_paths::model_adapter_runtime_path(),
        ),
        (
            "checkpoint_manifest_present",
            std::fs::metadata(state_paths::model_checkpoint_manifest_path()).is_ok(),
            state_paths::model_checkpoint_manifest_path(),
        ),
        (
            "training_harness_present",
            std::fs::metadata(state_paths::training_harness_report_path()).is_ok(),
            state_paths::training_harness_report_path(),
        ),
        (
            "model_governance_present",
            std::fs::metadata(state_paths::model_governance_report_path()).is_ok(),
            state_paths::model_governance_report_path(),
        ),
        (
            "first_model_card_present",
            std::fs::metadata(state_paths::first_model_card_path()).is_ok(),
            state_paths::first_model_card_path(),
        ),
        (
            "first_model_training_plan_present",
            std::fs::metadata(state_paths::first_model_training_plan_path()).is_ok(),
            state_paths::first_model_training_plan_path(),
        ),
        (
            "train_fixture_present",
            repo_file_exists(DEFAULT_TRAIN_DATA),
            DEFAULT_TRAIN_DATA.to_string(),
        ),
        (
            "eval_fixture_present",
            repo_file_exists(DEFAULT_EVAL_DATA),
            DEFAULT_EVAL_DATA.to_string(),
        ),
        (
            "training_not_executed",
            true,
            "training_executed=false".to_string(),
        ),
        ("weights_absent", true, "weights_present=false".to_string()),
    ];
    let passed = checks.iter().filter(|(_, passed, _)| *passed).count();
    let records: Vec<_> = checks
        .iter()
        .map(|(name, passed, evidence)| {
            serde_json::json!({
                "check": name,
                "passed": passed,
                "evidence": evidence,
            })
        })
        .collect();
    serde_json::json!({
        "schema": FIRST_MODEL_READINESS_SCHEMA,
        "artifact": "first_model_readiness",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "model_id": DEFAULT_MODEL_ID,
        "phase": "4A_preparation_readiness",
        "training_executed": false,
        "weights_present": false,
        "passed": passed,
        "total": checks.len(),
        "4a_complete": passed == checks.len(),
        "4b_training_allowed": false,
        "4b_blockers": [
            "explicit_operator_approval_required",
            "gpu_budget_not_requested_by_4a",
            "training_command_intentionally_not_implemented_in_4a",
            "checkpoint_output_path_not_admitted"
        ],
        "checks": records,
        "next_allowed_step": "operator may review 4A artifacts and separately request 4B training execution",
        "safety_boundary": model_safety_boundary(),
    })
}

fn elcp_objective_spec_value() -> Value {
    serde_json::json!({
        "schema": ELCP_OBJECTIVE_SPEC_SCHEMA,
        "artifact": "elcp_objective_spec",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "objective_id": ELCP_OBJECTIVE_ID,
        "model_id": DEFAULT_MODEL_ID,
        "phase": "4A_objective_prepared_not_trained",
        "training_executed": false,
        "weights_present": false,
        "checkpoint_admitted": false,
        "purpose": "Define Eden Latent Cognitive Prediction as EDEN's native training objective for predicting governed cognitive state transitions rather than treating next-token prediction as the primary brain objective.",
        "originality_boundary": {
            "not_a_nitp_copy": true,
            "not_a_plain_next_token_objective": true,
            "token_prediction_is_subordinate": true,
            "primary_target": "next_governed_cognitive_state"
        },
        "transition_unit": {
            "name": "cognitive_transition",
            "input_state": "C_t",
            "target_state": "C_t_plus_1",
            "source": "GEWC/CWM/runtime traces admitted through governance",
            "text_is_surface_channel": true
        },
        "prediction_targets": [
            target("surface_token", "optional language continuation for interfaces and explanations"),
            target("situation_state", "structured model of what is happening now"),
            target("goal_state", "active goal, subgoal, priority and constraint state"),
            target("memory_transition", "retrieve, retain, update, ignore or quarantine memory candidates"),
            target("world_delta", "predicted causal change in physical, social, digital or symbolic state"),
            target("plan_transition", "next plan node, revision or stop condition"),
            target("action_affordance", "available action proposals with preconditions and limits"),
            target("risk_calibration", "risk class, permission class and reversibility estimate"),
            target("uncertainty_calibration", "confidence, ambiguity and need for verification"),
            target("learning_update", "what can be learned without corrupting goals or memory"),
            target("safety_gate", "block, allow, defer or request supervision")
        ],
        "loss_terms": [
            loss("L_token", "surface continuation remains available but cannot be the sole training signal"),
            loss("L_situation_state", "aligns latent state with structured situation representation"),
            loss("L_goal_state", "preserves objective hierarchy and constraint state"),
            loss("L_memory_transition", "scores memory retrieval, update and quarantine decisions"),
            loss("L_world_delta", "predicts causal consequences and counterfactual deltas"),
            loss("L_plan_transition", "predicts next planning state under changing context"),
            loss("L_action_affordance", "predicts viable actions without granting direct authority"),
            loss("L_risk_calibration", "calibrates danger, permission and reversibility"),
            loss("L_uncertainty", "calibrates confidence and need for review"),
            loss("L_safety_gate", "keeps safety decisions explicit and supervised")
        ],
        "native_runtime_links": {
            "gewc": "executive authority and final admission",
            "cwm": "world delta and causal state targets",
            "memory": "retrieval and consolidation targets",
            "operator_forge": "formal expression and causal synthesis targets",
            "locus_layer": "personal/operator context targets",
            "safety_control": "permissions, audit and corrigibility targets"
        },
        "blocked_authority": [
            "self_training_without_approval",
            "direct_memory_write",
            "direct_objective_write",
            "direct_tool_execution",
            "checkpoint_self_admission"
        ],
        "safety_boundary": model_safety_boundary(),
    })
}

fn elcp_transition_dataset_value() -> Value {
    serde_json::json!({
        "schema": ELCP_TRANSITION_DATASET_SCHEMA,
        "artifact": "elcp_transition_dataset",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "objective_id": ELCP_OBJECTIVE_ID,
        "phase": "4A_dataset_contract_prepared",
        "training_executed": false,
        "contains_private_data": false,
        "synthetic_fixture_only": true,
        "config": ELCP_CONFIG,
        "train_data": ELCP_TRAIN_DATA,
        "eval_data": ELCP_EVAL_DATA,
        "manifest": "training/data/manifest.json",
        "record_contract": {
            "required_top_level_fields": [
                "id",
                "input",
                "target",
                "governance"
            ],
            "input_fields": [
                "surface_text",
                "situation",
                "goal",
                "working_memory",
                "world_state",
                "plan_state",
                "available_tools",
                "risk_context"
            ],
            "target_fields": [
                "next_situation",
                "next_goal_state",
                "memory_transition",
                "world_delta",
                "plan_transition",
                "action_affordance",
                "uncertainty",
                "safety_gate",
                "learning_update"
            ],
            "governance_fields": [
                "authority",
                "claim_allowed",
                "agi_claim",
                "direct_memory_writes",
                "direct_objective_writes",
                "direct_tool_execution"
            ]
        },
        "admission_policy": {
            "runtime_traces_must_be_permission_filtered": true,
            "private_user_data_requires_explicit_operator_approval": true,
            "unsafe_actions_are_recorded_as_blocked_targets": true,
            "synthetic_fixtures_are_allowed_for_4A_only": true
        },
        "safety_boundary": model_safety_boundary(),
    })
}

fn elcp_training_plan_value() -> Value {
    serde_json::json!({
        "schema": ELCP_TRAINING_PLAN_SCHEMA,
        "artifact": "elcp_training_plan",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "objective_id": ELCP_OBJECTIVE_ID,
        "model_id": DEFAULT_MODEL_ID,
        "phase": "4A_training_plan_only",
        "training_executed": false,
        "weights_present": false,
        "gpu_job_submitted": false,
        "explicit_4b_required": true,
        "future_training_boundary": {
            "requires_operator_approval": true,
            "requires_gpu_budget_approval": true,
            "requires_dataset_freeze": true,
            "requires_trace_privacy_review": true,
            "requires_checkpoint_output_dir": true,
            "requires_pre_and_post_eval": true,
            "training_command_intentionally_not_implemented_in_4a": true
        },
        "phases": [
            {
                "phase": "freeze_objective",
                "status": "prepared",
                "evidence": [state_paths::elcp_objective_spec_path()]
            },
            {
                "phase": "freeze_transition_contract",
                "status": "prepared",
                "evidence": [
                    state_paths::elcp_transition_dataset_path(),
                    ELCP_CONFIG,
                    ELCP_TRAIN_DATA,
                    ELCP_EVAL_DATA
                ]
            },
            {
                "phase": "generate_or_admit_traces",
                "status": "blocked_until_explicit_trace_scope",
                "requires": ["privacy_review", "permission_filter", "source_provenance"]
            },
            {
                "phase": "train",
                "status": "blocked_until_4B_explicit_approval",
                "writes_weights": false,
                "gpu_required_for_future_run": true
            },
            {
                "phase": "evaluate",
                "status": "blocked_until_checkpoint_exists",
                "required_metrics": [
                    "state_transition_accuracy",
                    "memory_transition_precision",
                    "world_delta_consistency",
                    "action_affordance_validity",
                    "risk_calibration",
                    "safety_gate_recall",
                    "no_direct_authority_violations"
                ]
            },
            {
                "phase": "admit_or_reject",
                "status": "blocked_until_governance_and_checkpoint_manifest_pass",
                "authority": AUTHORITY
            }
        ],
        "safety_boundary": model_safety_boundary(),
    })
}

fn elcp_admission_gate_value() -> Value {
    let validation_report = elcp_report_path("validation_report.json");
    let baseline_report = elcp_report_path("baseline_report.json");
    let trace_export_report = elcp_report_path("trace_export_report.json");
    let training_dry_run = elcp_report_path("training_dry_run.json");
    let admission_gate_report = elcp_report_path("admission_gate_report.json");
    let checks = vec![
        (
            "elcp_objective_spec_present",
            std::fs::metadata(state_paths::elcp_objective_spec_path()).is_ok(),
            state_paths::elcp_objective_spec_path(),
        ),
        (
            "elcp_transition_dataset_present",
            std::fs::metadata(state_paths::elcp_transition_dataset_path()).is_ok(),
            state_paths::elcp_transition_dataset_path(),
        ),
        (
            "elcp_training_plan_present",
            std::fs::metadata(state_paths::elcp_training_plan_path()).is_ok(),
            state_paths::elcp_training_plan_path(),
        ),
        (
            "transition_validation_report_passed",
            elcp_validation_report_passed(&validation_report),
            validation_report.clone(),
        ),
        (
            "baseline_report_passed",
            elcp_baseline_report_passed(&baseline_report),
            baseline_report.clone(),
        ),
        (
            "trace_export_report_passed",
            elcp_trace_export_passed(&trace_export_report),
            trace_export_report.clone(),
        ),
        (
            "training_dry_run_report_safe",
            elcp_training_dry_run_safe(&training_dry_run),
            training_dry_run.clone(),
        ),
        (
            "python_admission_gate_report_passed",
            elcp_python_admission_gate_passed(&admission_gate_report),
            admission_gate_report.clone(),
        ),
        (
            "training_not_executed",
            true,
            "training_executed=false".to_string(),
        ),
        ("weights_absent", true, "weights_present=false".to_string()),
    ];
    let passed = checks.iter().filter(|(_, passed, _)| *passed).count();
    let records: Vec<_> = checks
        .iter()
        .map(|(name, passed, evidence)| {
            serde_json::json!({
                "check": name,
                "passed": passed,
                "evidence": evidence,
            })
        })
        .collect();
    serde_json::json!({
        "schema": ELCP_ADMISSION_GATE_SCHEMA,
        "artifact": "elcp_admission_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "objective_id": ELCP_OBJECTIVE_ID,
        "model_id": DEFAULT_MODEL_ID,
        "mode": "pre_checkpoint_policy_gate",
        "phase": "4B_prep_admission_policy_only",
        "training_executed": false,
        "weights_present": false,
        "gpu_job_submitted": false,
        "checkpoint_admission_allowed": false,
        "4b_training_allowed": false,
        "passed": passed,
        "total": checks.len(),
        "checks": records,
        "required_reports": {
            "transition_validation": validation_report,
            "baseline_eval": baseline_report,
            "trace_export": trace_export_report,
            "training_dry_run": training_dry_run,
            "python_admission_gate": admission_gate_report
        },
        "future_minimums": {
            "state_transition_accuracy": "future_4b_threshold",
            "memory_transition_precision": "future_4b_threshold",
            "world_delta_consistency": "future_4b_threshold",
            "action_affordance_validity": "future_4b_threshold",
            "risk_calibration": "future_4b_threshold",
            "safety_gate_recall": "strict_high_recall",
            "direct_authority_violations": 0
        },
        "blockers": [
            "real_training_not_requested",
            "checkpoint_missing",
            "post_training_evaluation_missing",
            "human_release_review_missing"
        ],
        "safety_boundary": model_safety_boundary(),
    })
}

fn elcp_trace_quality_gate_value() -> Value {
    elcp_target_report_artifact_value(
        ELCP_TRACE_QUALITY_GATE_SCHEMA,
        "elcp_trace_quality_gate",
        "trace_quality_gate_report.json",
        &[
            "source_candidate_rows",
            "accepted_rows",
            "rejected_rows",
            "duplicate_rows",
            "skipped_duplicates",
            "accepted_output",
            "rejections",
            "admission_status",
        ],
        vec![
            ("accepted_rows_present", |report| {
                positive_u64(report, "accepted_rows")
            }),
            ("no_rejected_rows", |report| {
                report.get("rejected_rows").and_then(Value::as_u64) == Some(0)
            }),
        ],
        "trace_candidates_reviewed_not_training_truth",
    )
}

fn elcp_replay_eval_value() -> Value {
    elcp_target_report_artifact_value(
        ELCP_REPLAY_EVAL_SCHEMA,
        "elcp_replay_eval",
        "replay_eval_report.json",
        &[
            "replay_mode",
            "rows_passed",
            "rows_total",
            "field_score",
            "by_field",
            "results",
        ],
        vec![
            ("replay_rows_present", |report| {
                positive_u64(report, "rows_total")
            }),
            ("all_replay_rows_passed", |report| {
                report.get("rows_passed").and_then(Value::as_u64)
                    == report.get("rows_total").and_then(Value::as_u64)
                    && positive_u64(report, "rows_total")
            }),
        ],
        "runtime_trace_replay_baseline_passed",
    )
}

fn elcp_dataset_freeze_manifest_value() -> Value {
    elcp_target_report_artifact_value(
        ELCP_DATASET_FREEZE_MANIFEST_SCHEMA,
        "elcp_dataset_freeze_manifest",
        "dataset_freeze_manifest.json",
        &[
            "freeze_id",
            "dataset_locked_for_training",
            "candidate_pool_locked_for_review",
            "training_allowed",
            "files",
            "split_policy",
            "privacy_policy",
        ],
        vec![
            ("candidate_pool_locked_for_review", |report| {
                report
                    .get("candidate_pool_locked_for_review")
                    .and_then(Value::as_bool)
                    == Some(true)
            }),
            ("dataset_not_locked_for_training", |report| {
                report
                    .get("dataset_locked_for_training")
                    .and_then(Value::as_bool)
                    == Some(false)
            }),
            ("training_not_allowed", |report| {
                report.get("training_allowed").and_then(Value::as_bool) == Some(false)
            }),
        ],
        "dataset_hashes_frozen_for_review_not_training",
    )
}

fn elcp_metrics_board_value() -> Value {
    elcp_target_report_artifact_value(
        ELCP_METRICS_BOARD_SCHEMA,
        "elcp_metrics_board",
        "metrics_board.json",
        &["4b_training_allowed", "metrics", "source_schemas"],
        vec![("4b_training_blocked", |report| {
            report.get("4b_training_allowed").and_then(Value::as_bool) == Some(false)
        })],
        "metrics_board_green_for_operator_review",
    )
}

fn elcp_4b_readiness_contract_value() -> Value {
    elcp_target_report_artifact_value(
        ELCP_4B_READINESS_CONTRACT_SCHEMA,
        "elcp_4b_readiness_contract",
        "4b_readiness_contract.json",
        &[
            "checkpoint_admitted",
            "4b_training_allowed",
            "contract_prepared",
            "contract_scope",
            "required_before_4b_training",
            "current_blockers",
            "metrics_board",
            "freeze_id",
        ],
        vec![
            ("contract_prepared", |report| {
                report.get("contract_prepared").and_then(Value::as_bool) == Some(true)
            }),
            ("4b_training_blocked", |report| {
                report.get("4b_training_allowed").and_then(Value::as_bool) == Some(false)
            }),
            ("checkpoint_not_admitted", |report| {
                report.get("checkpoint_admitted").and_then(Value::as_bool) == Some(false)
            }),
        ],
        "4b_contract_prepared_but_training_requires_separate_approval",
    )
}

fn elcp_readiness_value() -> Value {
    let checks = vec![
        (
            "first_model_card_present",
            std::fs::metadata(state_paths::first_model_card_path()).is_ok(),
            state_paths::first_model_card_path(),
        ),
        (
            "first_model_training_plan_present",
            std::fs::metadata(state_paths::first_model_training_plan_path()).is_ok(),
            state_paths::first_model_training_plan_path(),
        ),
        (
            "first_model_readiness_present",
            std::fs::metadata(state_paths::first_model_readiness_path()).is_ok(),
            state_paths::first_model_readiness_path(),
        ),
        (
            "elcp_objective_spec_present",
            std::fs::metadata(state_paths::elcp_objective_spec_path()).is_ok(),
            state_paths::elcp_objective_spec_path(),
        ),
        (
            "elcp_transition_dataset_present",
            std::fs::metadata(state_paths::elcp_transition_dataset_path()).is_ok(),
            state_paths::elcp_transition_dataset_path(),
        ),
        (
            "elcp_training_plan_present",
            std::fs::metadata(state_paths::elcp_training_plan_path()).is_ok(),
            state_paths::elcp_training_plan_path(),
        ),
        (
            "elcp_admission_gate_passed",
            admission_gate_artifact_passed(&state_paths::elcp_admission_gate_path()),
            state_paths::elcp_admission_gate_path(),
        ),
        (
            "elcp_trace_quality_gate_passed",
            hardening_artifact_passed(
                &state_paths::elcp_trace_quality_gate_path(),
                ELCP_TRACE_QUALITY_GATE_SCHEMA,
            ),
            state_paths::elcp_trace_quality_gate_path(),
        ),
        (
            "elcp_replay_eval_passed",
            hardening_artifact_passed(
                &state_paths::elcp_replay_eval_path(),
                ELCP_REPLAY_EVAL_SCHEMA,
            ),
            state_paths::elcp_replay_eval_path(),
        ),
        (
            "elcp_dataset_freeze_manifest_passed",
            hardening_artifact_passed(
                &state_paths::elcp_dataset_freeze_manifest_path(),
                ELCP_DATASET_FREEZE_MANIFEST_SCHEMA,
            ),
            state_paths::elcp_dataset_freeze_manifest_path(),
        ),
        (
            "elcp_metrics_board_passed",
            hardening_artifact_passed(
                &state_paths::elcp_metrics_board_path(),
                ELCP_METRICS_BOARD_SCHEMA,
            ),
            state_paths::elcp_metrics_board_path(),
        ),
        (
            "elcp_4b_readiness_contract_passed",
            hardening_artifact_passed(
                &state_paths::elcp_4b_readiness_contract_path(),
                ELCP_4B_READINESS_CONTRACT_SCHEMA,
            ),
            state_paths::elcp_4b_readiness_contract_path(),
        ),
        (
            "elcp_config_present",
            repo_file_exists(ELCP_CONFIG),
            ELCP_CONFIG.to_string(),
        ),
        (
            "elcp_train_fixture_present",
            repo_file_exists(ELCP_TRAIN_DATA),
            ELCP_TRAIN_DATA.to_string(),
        ),
        (
            "elcp_eval_fixture_present",
            repo_file_exists(ELCP_EVAL_DATA),
            ELCP_EVAL_DATA.to_string(),
        ),
        (
            "training_not_executed",
            true,
            "training_executed=false".to_string(),
        ),
        ("weights_absent", true, "weights_present=false".to_string()),
    ];
    let passed = checks.iter().filter(|(_, passed, _)| *passed).count();
    let records: Vec<_> = checks
        .iter()
        .map(|(name, passed, evidence)| {
            serde_json::json!({
                "check": name,
                "passed": passed,
                "evidence": evidence,
            })
        })
        .collect();
    serde_json::json!({
        "schema": ELCP_READINESS_SCHEMA,
        "artifact": "elcp_readiness",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "objective_id": ELCP_OBJECTIVE_ID,
        "model_id": DEFAULT_MODEL_ID,
        "phase": "4A_elcp_preparation_plus_4b_prep_hardening_readiness",
        "training_executed": false,
        "weights_present": false,
        "gpu_job_submitted": false,
        "passed": passed,
        "total": checks.len(),
        "elcp_4a_complete": passed == checks.len(),
        "4b_training_allowed": false,
        "4b_blockers": [
            "explicit_operator_approval_required",
            "gpu_budget_not_requested_by_4a",
            "training_command_intentionally_not_implemented_in_4a",
            "checkpoint_output_path_not_admitted",
            "trace_privacy_scope_not_approved"
        ],
        "checks": records,
        "next_allowed_step": "operator may review ELCP 4A artifacts and separately request 4B training execution",
        "safety_boundary": model_safety_boundary(),
    })
}

fn megatron_7b_model_adapter_value() -> Value {
    let training_status =
        read_megatron_7b_training_evidence_status(DEFAULT_MEGATRON_7B_TRAINING_EVIDENCE_PATH);
    let inference_status =
        read_megatron_7b_inference_report_status(DEFAULT_MEGATRON_7B_INFERENCE_REPORT_PATH);
    serde_json::json!({
        "schema": MEGATRON_7B_MODEL_ADAPTER_SCHEMA,
        "artifact": "megatron_7b_model_adapter",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "model_id": MEGATRON_7B_MODEL_ID,
        "runtime_role": "GEWC_SUBORDINATE_GENERATIVE_MODEL_ADAPTER",
        "native_to_gewc": true,
        "checkpoint_admitted": false,
        "production_model": false,
        "purpose": "Expose the EDEN-owned 7B Megatron checkpoint as a governed cognitive capacity: load checkpoint, generate candidate tokens and return hypotheses to GEWC without direct authority.",
        "source_artifacts": {
            "training_evidence_source": DEFAULT_MEGATRON_7B_TRAINING_EVIDENCE_PATH,
            "training_evidence_state": state_paths::megatron_7b_training_evidence_path(),
            "inference_report_source": DEFAULT_MEGATRON_7B_INFERENCE_REPORT_PATH,
            "inference_report_state": state_paths::megatron_7b_inference_report_path(),
            "capability_report": state_paths::megatron_7b_capability_report_path(),
            "admission_gate": state_paths::megatron_7b_admission_gate_path(),
        },
        "model_shape": {
            "family": "gpt_megatron_random_init",
            "parameters": training_status.get("model_parameters").cloned().unwrap_or(Value::Null),
            "layers": 32,
            "hidden_size": 4096,
            "ffn_hidden_size": 12288,
            "attention_heads": 32,
            "sequence_length": 128,
            "tokenizer": "eden_sentencepiece_vocab_2048"
        },
        "usable_capacity_boundary": {
            "accepted_for": [
                "checkpoint_load_probe",
                "token_generation_probe",
                "model_adapter_contract_test",
                "future_supervised_cognitive_candidate_generation"
            ],
            "not_accepted_for": [
                "agi_claim",
                "semantic_competence_claim",
                "autonomous_runtime_authority",
                "direct_memory_mutation",
                "direct_objective_mutation",
                "production_inference"
            ]
        },
        "status": {
            "training_evidence": training_status,
            "inference_report": inference_status,
            "adapter_prepared": training_status.get("accepted").and_then(Value::as_bool) == Some(true),
            "inference_observed": inference_status.get("accepted").and_then(Value::as_bool) == Some(true),
            "checkpoint_admission": false
        },
        "interfaces": {
            "input": "GEWC-filtered prompt/context packet",
            "output": "candidate token continuation with provenance and uncertainty metadata",
            "state_write_policy": "read-only candidate generator; all state changes must go through GEWC transaction layers"
        },
        "safety_boundary": model_safety_boundary(),
    })
}

fn megatron_7b_capability_report_value() -> Value {
    let inference_status =
        read_megatron_7b_inference_report_status(DEFAULT_MEGATRON_7B_INFERENCE_REPORT_PATH);
    let training_status =
        read_megatron_7b_training_evidence_status(DEFAULT_MEGATRON_7B_TRAINING_EVIDENCE_PATH);
    let inference_accepted =
        inference_status.get("accepted").and_then(Value::as_bool) == Some(true);
    let training_accepted = training_status.get("accepted").and_then(Value::as_bool) == Some(true);
    let checks = vec![
        (
            "training_evidence_accepted",
            training_accepted,
            DEFAULT_MEGATRON_7B_TRAINING_EVIDENCE_PATH.to_string(),
        ),
        (
            "checkpoint_written_but_not_admitted",
            training_status
                .get("checkpoint_written")
                .and_then(Value::as_bool)
                == Some(true),
            "checkpoint_written=true checkpoint_admission=false".to_string(),
        ),
        (
            "inference_report_accepted",
            inference_accepted,
            DEFAULT_MEGATRON_7B_INFERENCE_REPORT_PATH.to_string(),
        ),
        (
            "checkpoint_loaded_for_inference",
            inference_status
                .get("checkpoint_loaded")
                .and_then(Value::as_bool)
                == Some(true),
            "inference_report.run.checkpoint_loaded=true".to_string(),
        ),
        (
            "tokens_generated",
            inference_status
                .get("generated_count")
                .and_then(Value::as_u64)
                .map(|count| count > 0)
                .unwrap_or(false),
            "inference_report.run.generated_count > 0".to_string(),
        ),
        (
            "no_claim_boundary_preserved",
            inference_status
                .get("claim_allowed")
                .and_then(Value::as_bool)
                == Some(false)
                && inference_status.get("agi_claim").and_then(Value::as_bool) == Some(false),
            "claim_allowed=false agi_claim=false".to_string(),
        ),
    ];
    let passed = checks.iter().filter(|(_, passed, _)| *passed).count();
    let records: Vec<_> = checks
        .iter()
        .map(|(name, passed, evidence)| {
            serde_json::json!({
                "check": name,
                "passed": passed,
                "evidence": evidence,
            })
        })
        .collect();
    serde_json::json!({
        "schema": MEGATRON_7B_CAPABILITY_REPORT_SCHEMA,
        "artifact": "megatron_7b_capability_report",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "model_id": MEGATRON_7B_MODEL_ID,
        "capability_id": "eden_7b_checkpoint_token_generation_probe",
        "capability_status": if passed == checks.len() { "usable_probe_path" } else { "pending_or_incomplete" },
        "passed": passed,
        "total": checks.len(),
        "checks": records,
        "training_evidence": training_status,
        "inference_evidence": inference_status,
        "accepted_capability": {
            "checkpoint_load": inference_accepted,
            "token_generation": inference_accepted,
            "gewc_subordinate_adapter": true,
            "semantic_quality_admitted": false,
            "production_inference_admitted": false,
            "checkpoint_admission": false
        },
        "not_evidence_for": [
            "AGI",
            "general intelligence",
            "language competence",
            "external benchmark performance",
            "autonomous action authority"
        ],
        "next_required_evaluations": [
            "token-level validation suite",
            "held-out EDEN corpus perplexity",
            "instruction-following probes after supervised training",
            "safety prompt/adversarial probes",
            "checkpoint reproducibility hash review"
        ],
        "safety_boundary": model_safety_boundary(),
    })
}

fn megatron_7b_admission_gate_value() -> Value {
    let capability = read_json_file(&state_paths::megatron_7b_capability_report_path());
    let capability_passed = capability.as_ref().is_some_and(|report| {
        match (
            report.get("passed").and_then(Value::as_u64),
            report.get("total").and_then(Value::as_u64),
        ) {
            (Some(passed), Some(total)) => total > 0 && passed == total,
            _ => false,
        }
    });
    let checks = vec![
        (
            "capability_report_present",
            capability.is_some(),
            state_paths::megatron_7b_capability_report_path(),
        ),
        (
            "capability_probe_passed",
            capability_passed,
            "megatron_7b_capability_report.passed == total".to_string(),
        ),
        (
            "claim_boundary_false",
            capability.as_ref().is_some_and(|report| {
                report.get("claim_allowed").and_then(Value::as_bool) == Some(false)
                    && report.get("agi_claim").and_then(Value::as_bool) == Some(false)
            }),
            "claim_allowed=false agi_claim=false".to_string(),
        ),
        (
            "semantic_quality_not_admitted",
            capability.as_ref().is_some_and(|report| {
                report
                    .get("accepted_capability")
                    .and_then(Value::as_object)
                    .and_then(|accepted| accepted.get("semantic_quality_admitted"))
                    .and_then(Value::as_bool)
                    == Some(false)
            }),
            "accepted_capability.semantic_quality_admitted=false".to_string(),
        ),
        (
            "production_inference_not_admitted",
            capability.as_ref().is_some_and(|report| {
                report
                    .get("accepted_capability")
                    .and_then(Value::as_object)
                    .and_then(|accepted| accepted.get("production_inference_admitted"))
                    .and_then(Value::as_bool)
                    == Some(false)
            }),
            "accepted_capability.production_inference_admitted=false".to_string(),
        ),
    ];
    let passed = checks.iter().filter(|(_, passed, _)| *passed).count();
    let records: Vec<_> = checks
        .iter()
        .map(|(name, passed, evidence)| {
            serde_json::json!({
                "check": name,
                "passed": passed,
                "evidence": evidence,
            })
        })
        .collect();
    serde_json::json!({
        "schema": MEGATRON_7B_ADMISSION_GATE_SCHEMA,
        "artifact": "megatron_7b_admission_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "model_id": MEGATRON_7B_MODEL_ID,
        "mode": "post_training_probe_gate",
        "passed": passed,
        "total": checks.len(),
        "checks": records,
        "checkpoint_load_probe_usable": capability_passed,
        "checkpoint_admission_allowed": false,
        "weights_admitted": false,
        "production_model": false,
        "autonomous_runtime_authority": false,
        "decision": "usable_as_governed_probe_only",
        "blockers": [
            "semantic_quality_not_evaluated",
            "external_validation_missing",
            "safety_eval_missing",
            "checkpoint_hash_review_missing",
            "human_release_review_missing"
        ],
        "allowed_next_steps": [
            "run additional local inference probes",
            "build EDEN-native token/cognitive evaluation set",
            "prepare supervised cognitive-state training data",
            "keep checkpoint outside git and outside production authority"
        ],
        "safety_boundary": model_safety_boundary(),
    })
}

fn target(name: &str, description: &str) -> Value {
    serde_json::json!({
        "name": name,
        "description": description,
        "status": "specified_not_trained",
    })
}

fn loss(name: &str, description: &str) -> Value {
    serde_json::json!({
        "name": name,
        "description": description,
        "status": "specified_not_trained",
        "weight": "future_4b_hyperparameter",
    })
}

fn model_permission(
    action: &str,
    allowed: bool,
    sandbox_required: bool,
    human_review_required: bool,
    blocked_for_model: bool,
    reason: &str,
) -> Value {
    serde_json::json!({
        "action": action,
        "allowed": allowed,
        "sandbox_required": sandbox_required,
        "human_review_required": human_review_required,
        "blocked_for_direct_model_authority": blocked_for_model,
        "reason": reason,
    })
}

fn model_safety_boundary() -> Value {
    serde_json::json!({
        "gewc_final_authority": true,
        "direct_memory_writes": false,
        "direct_objective_writes": false,
        "direct_tool_execution": false,
        "requires_gewc_admission": true,
        "outputs_are_hypotheses": true,
        "model_may_not_mutate_runtime_state": true,
        "permission_escalation_allowed": false,
    })
}

fn read_training_report_status(path: &str) -> Value {
    match std::fs::read_to_string(path) {
        Ok(body) => match serde_json::from_str::<Value>(&body) {
            Ok(report) => match training_evidence::validate_training_report_value(&report) {
                Ok(summary) => serde_json::json!({
                    "status": "accepted_smoke_report_present",
                    "passed": summary.passed,
                    "total": summary.total,
                    "score": summary.score,
                    "first_model_passed": summary.first_model_passed,
                    "first_model_total": summary.first_model_total,
                }),
                Err(err) => serde_json::json!({
                    "status": "rejected_invalid_smoke_report",
                    "reason": err,
                }),
            },
            Err(err) => serde_json::json!({
                "status": "rejected_unparseable_smoke_report",
                "reason": err.to_string(),
            }),
        },
        Err(_) => serde_json::json!({
            "status": "waiting_for_capability_report",
            "expected_command": "make training-smoke",
        }),
    }
}

fn read_megatron_7b_training_evidence_status(path: &str) -> Value {
    match read_repo_json(path) {
        Some(evidence) => match training_evidence::validate_megatron_7b_evidence_value(&evidence) {
            Ok(summary) => serde_json::json!({
                "accepted": true,
                "schema": training_evidence::MEGATRON_7B_EVIDENCE_SCHEMA,
                "train_iters": summary.train_iters,
                "completed_iterations": summary.completed_iterations,
                "model_parameters": summary.model_parameters,
                "final_loss": summary.final_loss,
                "checkpoint_written": summary.checkpoint_written,
                "checkpoint_admission": false,
                "source": path,
            }),
            Err(err) => serde_json::json!({
                "accepted": false,
                "schema": training_evidence::MEGATRON_7B_EVIDENCE_SCHEMA,
                "reason": err,
                "source": path,
            }),
        },
        None => serde_json::json!({
            "accepted": false,
            "schema": training_evidence::MEGATRON_7B_EVIDENCE_SCHEMA,
            "reason": "training evidence source missing",
            "source": path,
        }),
    }
}

fn read_megatron_7b_inference_report_status(path: &str) -> Value {
    match read_repo_json(path) {
        Some(report) => match validate_megatron_7b_inference_report_value(&report) {
            Ok(()) => {
                let run = report.get("run").unwrap_or(&Value::Null);
                serde_json::json!({
                    "accepted": true,
                    "schema": MEGATRON_7B_INFERENCE_REPORT_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "checkpoint_loaded": run.get("checkpoint_loaded").and_then(Value::as_bool).unwrap_or(false),
                    "generated_count": run.get("generated_count").and_then(Value::as_u64).unwrap_or(0),
                    "tokens_to_generate": run.get("tokens_to_generate").and_then(Value::as_u64).unwrap_or(0),
                    "checkpoint_admission": false,
                    "source": path,
                })
            }
            Err(err) => serde_json::json!({
                "accepted": false,
                "schema": MEGATRON_7B_INFERENCE_REPORT_SCHEMA,
                "claim_allowed": false,
                "agi_claim": false,
                "reason": err,
                "source": path,
            }),
        },
        None => serde_json::json!({
            "accepted": false,
            "schema": MEGATRON_7B_INFERENCE_REPORT_SCHEMA,
            "claim_allowed": false,
            "agi_claim": false,
            "reason": "inference report source missing",
            "source": path,
        }),
    }
}

fn copy_megatron_7b_inference_report_from_path(path: &str) -> Result<std::path::PathBuf, String> {
    let report =
        read_repo_json(path).ok_or_else(|| "inference report source missing".to_string())?;
    validate_megatron_7b_inference_report_value(&report)?;
    let target = std::path::PathBuf::from(state_paths::megatron_7b_inference_report_path());
    state_paths::ensure_state_dir()?;
    std::fs::write(
        &target,
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("failed to write Megatron 7B inference report: {}", e))?;
    Ok(target)
}

fn validate_megatron_7b_inference_report_value(report: &Value) -> Result<(), String> {
    require_value_string_eq(report, "schema", MEGATRON_7B_INFERENCE_REPORT_SCHEMA)?;
    require_value_string_eq(report, "authority", AUTHORITY)?;
    require_value_bool_eq(report, "claim_allowed", false)?;
    require_value_bool_eq(report, "agi_claim", false)?;
    require_value_string_eq(report, "accepted_as", "7b_checkpoint_inference_probe")?;

    let run = require_value_object(report, "run")?;
    require_map_bool_eq(run, "passed", true)?;
    require_map_string_eq(run, "network", "none")?;
    require_map_bool_eq(run, "external_model_dependency", false)?;
    require_map_bool_eq(run, "checkpoint_loaded", true)?;
    require_map_bool_eq(run, "checkpoint_admission", false)?;
    require_map_bool_eq(run, "production_model", false)?;
    let generated_count = require_map_u64(run, "generated_count")?;
    let tokens_to_generate = require_map_u64(run, "tokens_to_generate")?;
    if generated_count == 0 {
        return Err("run.generated_count must be greater than zero".to_string());
    }
    if tokens_to_generate == 0 {
        return Err("run.tokens_to_generate must be greater than zero".to_string());
    }

    let response = report
        .get("responses")
        .and_then(Value::as_array)
        .ok_or_else(|| "responses.array required".to_string())?;
    if response.len() != generated_count as usize {
        return Err("responses length must match run.generated_count".to_string());
    }

    let safety_boundary = require_value_object(report, "safety_boundary")?;
    require_map_bool_eq(safety_boundary, "direct_memory_writes", false)?;
    require_map_bool_eq(safety_boundary, "direct_objective_writes", false)?;
    require_map_bool_eq(safety_boundary, "direct_tool_execution", false)?;
    require_map_bool_eq(safety_boundary, "requires_gewc_admission", true)?;
    require_map_bool_eq(safety_boundary, "outputs_are_hypotheses", true)?;
    Ok(())
}

fn existing_adapter_events() -> Vec<Value> {
    let path = state_paths::model_adapter_runtime_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|body| serde_json::from_str::<Value>(&body).ok())
        .and_then(|value| value.get("events").and_then(Value::as_array).cloned())
        .unwrap_or_default()
}

fn repo_file_exists(path: &str) -> bool {
    repo_path(path).is_some()
}

type ReportCheck = (&'static str, fn(&Value) -> bool);

fn elcp_target_report_artifact_value(
    schema: &str,
    artifact: &str,
    report_filename: &str,
    mirrored_fields: &[&str],
    extra_checks: Vec<ReportCheck>,
    decision: &str,
) -> Value {
    let source_report = elcp_report_path(report_filename);
    let report = read_repo_json(&source_report);
    let mut checks = vec![
        (
            "source_report_present".to_string(),
            report.is_some(),
            source_report.clone(),
        ),
        (
            "source_report_schema_matches".to_string(),
            report
                .as_ref()
                .is_some_and(|value| value.get("schema").and_then(Value::as_str) == Some(schema)),
            schema.to_string(),
        ),
        (
            "source_report_no_claim_boundary".to_string(),
            report
                .as_ref()
                .is_some_and(report_preserves_no_claim_boundary),
            "claim_allowed=false training_executed=false weights_present=false".to_string(),
        ),
        (
            "source_report_passed".to_string(),
            report.as_ref().is_some_and(report_count_passed),
            "passed == total".to_string(),
        ),
    ];
    for (name, check) in extra_checks {
        checks.push((
            name.to_string(),
            report.as_ref().is_some_and(check),
            source_report.clone(),
        ));
    }
    let passed = checks.iter().filter(|(_, passed, _)| *passed).count();
    let records: Vec<_> = checks
        .iter()
        .map(|(name, passed, evidence)| {
            serde_json::json!({
                "check": name,
                "passed": passed,
                "evidence": evidence,
            })
        })
        .collect();
    let mut record = serde_json::json!({
        "schema": schema,
        "artifact": artifact,
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "training_executed": false,
        "weights_present": false,
        "checkpoint_admission_allowed": false,
        "4b_training_allowed": false,
        "source_report": source_report,
        "passed": passed,
        "total": checks.len(),
        "checks": records,
        "decision": decision,
        "source_summary": report.as_ref().map(|value| {
            serde_json::json!({
                "schema": value.get("schema").cloned().unwrap_or(Value::Null),
                "passed": value.get("passed").cloned().unwrap_or(Value::Null),
                "total": value.get("total").cloned().unwrap_or(Value::Null),
                "claim_allowed": value.get("claim_allowed").cloned().unwrap_or(Value::Null),
                "training_executed": value.get("training_executed").cloned().unwrap_or(Value::Null),
                "weights_present": value.get("weights_present").cloned().unwrap_or(Value::Null),
            })
        }),
    });
    if let (Some(source), Some(record_object)) = (report.as_ref(), record.as_object_mut()) {
        for field in mirrored_fields {
            if let Some(value) = source.get(*field) {
                record_object.insert((*field).to_string(), value.clone());
            }
        }
    }
    record
}

fn positive_u64(report: &Value, field: &str) -> bool {
    report
        .get(field)
        .and_then(Value::as_u64)
        .map(|value| value > 0)
        .unwrap_or(false)
}

fn elcp_report_path(filename: &str) -> String {
    let base =
        std::env::var(ELCP_REPORT_DIR_ENV).unwrap_or_else(|_| "target/eden_elcp".to_string());
    format!("{}/{}", base.trim_end_matches('/'), filename)
}

fn repo_path(path: &str) -> Option<std::path::PathBuf> {
    let local = std::path::Path::new(path);
    if std::fs::metadata(local).is_ok() {
        return Some(local.to_path_buf());
    }
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(|repo_root| repo_root.join(path))
        .filter(|candidate| std::fs::metadata(candidate).is_ok())
}

fn read_repo_json(path: &str) -> Option<Value> {
    let path = repo_path(path)?;
    let body = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<Value>(&body).ok()
}

fn read_json_file(path: &str) -> Option<Value> {
    let body = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<Value>(&body).ok()
}

fn require_value_object<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{}.object required", field))
}

fn require_value_string_eq(value: &Value, field: &str, expected: &str) -> Result<(), String> {
    match value.get(field).and_then(Value::as_str) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!("{} must be {}, got {}", field, expected, actual)),
        None => Err(format!("{}.string required", field)),
    }
}

fn require_value_bool_eq(value: &Value, field: &str, expected: bool) -> Result<(), String> {
    match value.get(field).and_then(Value::as_bool) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!("{} must be {}, got {}", field, expected, actual)),
        None => Err(format!("{}.bool required", field)),
    }
}

fn require_map_string_eq(
    value: &serde_json::Map<String, Value>,
    field: &str,
    expected: &str,
) -> Result<(), String> {
    match value.get(field).and_then(Value::as_str) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!("{} must be {}, got {}", field, expected, actual)),
        None => Err(format!("{}.string required", field)),
    }
}

fn require_map_bool_eq(
    value: &serde_json::Map<String, Value>,
    field: &str,
    expected: bool,
) -> Result<(), String> {
    match value.get(field).and_then(Value::as_bool) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!("{} must be {}, got {}", field, expected, actual)),
        None => Err(format!("{}.bool required", field)),
    }
}

fn require_map_u64(value: &serde_json::Map<String, Value>, field: &str) -> Result<u64, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{}.u64 required", field))
}

fn report_preserves_no_claim_boundary(report: &Value) -> bool {
    report.get("claim_allowed").and_then(Value::as_bool) == Some(false)
        && report.get("agi_claim").and_then(Value::as_bool) == Some(false)
        && report.get("training_executed").and_then(Value::as_bool) == Some(false)
        && report.get("weights_present").and_then(Value::as_bool) == Some(false)
}

fn report_count_passed(report: &Value) -> bool {
    let passed = report.get("passed").and_then(Value::as_u64);
    let total = report.get("total").and_then(Value::as_u64);
    matches!((passed, total), (Some(passed), Some(total)) if total > 0 && passed == total)
}

fn report_has_schema_and_boundary(report: &Value, schema: &str) -> bool {
    report.get("schema").and_then(Value::as_str) == Some(schema)
        && report_preserves_no_claim_boundary(report)
}

fn elcp_validation_report_passed(path: &str) -> bool {
    match read_repo_json(path) {
        Some(report) => {
            report_has_schema_and_boundary(&report, ELCP_VALIDATION_REPORT_SCHEMA)
                && report_count_passed(&report)
                && report
                    .get("errors")
                    .and_then(Value::as_array)
                    .map(|errors| errors.is_empty())
                    .unwrap_or(false)
        }
        None => false,
    }
}

fn elcp_baseline_report_passed(path: &str) -> bool {
    match read_repo_json(path) {
        Some(report) => {
            let summary = report.get("summary").unwrap_or(&Value::Null);
            let passed_rows = summary.get("passed_rows").and_then(Value::as_u64);
            let total_rows = summary.get("total_rows").and_then(Value::as_u64);
            report_has_schema_and_boundary(&report, ELCP_BASELINE_REPORT_SCHEMA)
                && matches!((passed_rows, total_rows), (Some(passed), Some(total)) if total > 0 && passed == total)
        }
        None => false,
    }
}

fn elcp_trace_export_passed(path: &str) -> bool {
    match read_repo_json(path) {
        Some(report) => {
            report_has_schema_and_boundary(&report, ELCP_TRACE_EXPORT_SCHEMA)
                && report.get("source_trace_present").and_then(Value::as_bool) == Some(true)
                && report
                    .get("candidate_rows")
                    .and_then(Value::as_u64)
                    .map(|rows| rows > 0)
                    .unwrap_or(false)
        }
        None => false,
    }
}

fn elcp_training_dry_run_safe(path: &str) -> bool {
    match read_repo_json(path) {
        Some(report) => {
            report_has_schema_and_boundary(&report, ELCP_TRAINING_DRY_RUN_SCHEMA)
                && report.get("weights_written").and_then(Value::as_bool) == Some(false)
                && report.get("gpu_job_submitted").and_then(Value::as_bool) == Some(false)
                && report.get("checkpoint_admitted").and_then(Value::as_bool) == Some(false)
        }
        None => false,
    }
}

fn elcp_python_admission_gate_passed(path: &str) -> bool {
    match read_repo_json(path) {
        Some(report) => {
            report_has_schema_and_boundary(&report, ELCP_ADMISSION_GATE_SCHEMA)
                && report_count_passed(&report)
                && report
                    .get("checkpoint_admission_allowed")
                    .and_then(Value::as_bool)
                    == Some(false)
                && report.get("4b_training_allowed").and_then(Value::as_bool) == Some(false)
        }
        None => false,
    }
}

fn admission_gate_artifact_passed(path: &str) -> bool {
    match read_json_file(path) {
        Some(report) => {
            report.get("schema").and_then(Value::as_str) == Some(ELCP_ADMISSION_GATE_SCHEMA)
                && report_preserves_no_claim_boundary(&report)
                && report_count_passed(&report)
                && report
                    .get("checkpoint_admission_allowed")
                    .and_then(Value::as_bool)
                    == Some(false)
                && report.get("4b_training_allowed").and_then(Value::as_bool) == Some(false)
        }
        None => false,
    }
}

fn hardening_artifact_passed(path: &str, schema: &str) -> bool {
    match read_json_file(path) {
        Some(report) => {
            report.get("schema").and_then(Value::as_str) == Some(schema)
                && report_preserves_no_claim_boundary(&report)
                && report_count_passed(&report)
                && report.get("4b_training_allowed").and_then(Value::as_bool) == Some(false)
        }
        None => false,
    }
}

fn write_report(tag: &str, schema: &str, path: String, record: Value) -> String {
    match write_json(&path, &record) {
        Ok(()) => format!(
            "[{}] schema={} status=written authority={} claim_allowed=false agi_claim=false path={}\n",
            tag, schema, AUTHORITY, path
        ),
        Err(err) => format!(
            "[{}] schema={} status=error authority={} claim_allowed=false agi_claim=false reason={}\n",
            tag, schema, AUTHORITY, err
        ),
    }
}

fn write_json(path: &str, record: &Value) -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let body = serde_json::to_string_pretty(record).map_err(|e| e.to_string())?;
    std::fs::write(path, body).map_err(|e| format!("failed to write {}: {}", path, e))
}

fn normalize_model_id(model_id: &str) -> String {
    let trimmed = model_id.trim();
    if trimmed.is_empty() {
        DEFAULT_MODEL_ID.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_elcp_pipeline_reports_for_test(report_dir: &std::path::Path) {
        std::fs::create_dir_all(report_dir).unwrap();
        let reports = [
            (
                "validation_report.json",
                serde_json::json!({
                    "schema": ELCP_VALIDATION_REPORT_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "training_executed": false,
                    "weights_present": false,
                    "passed": 2,
                    "total": 2,
                    "errors": [],
                }),
            ),
            (
                "baseline_report.json",
                serde_json::json!({
                    "schema": ELCP_BASELINE_REPORT_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "training_executed": false,
                    "weights_present": false,
                    "summary": {
                        "passed_rows": 2,
                        "total_rows": 2,
                    },
                }),
            ),
            (
                "trace_export_report.json",
                serde_json::json!({
                    "schema": ELCP_TRACE_EXPORT_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "training_executed": false,
                    "weights_present": false,
                    "source_trace_present": true,
                    "candidate_rows": 1,
                }),
            ),
            (
                "training_dry_run.json",
                serde_json::json!({
                    "schema": ELCP_TRAINING_DRY_RUN_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "training_executed": false,
                    "weights_present": false,
                    "weights_written": false,
                    "gpu_job_submitted": false,
                    "checkpoint_admitted": false,
                }),
            ),
            (
                "admission_gate_report.json",
                serde_json::json!({
                    "schema": ELCP_ADMISSION_GATE_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "training_executed": false,
                    "weights_present": false,
                    "checkpoint_admission_allowed": false,
                    "4b_training_allowed": false,
                    "passed": 5,
                    "total": 5,
                }),
            ),
            (
                "trace_quality_gate_report.json",
                serde_json::json!({
                    "schema": ELCP_TRACE_QUALITY_GATE_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "training_executed": false,
                    "weights_present": false,
                    "accepted_rows": 2,
                    "rejected_rows": 0,
                    "passed": 6,
                    "total": 6,
                }),
            ),
            (
                "replay_eval_report.json",
                serde_json::json!({
                    "schema": ELCP_REPLAY_EVAL_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "training_executed": false,
                    "weights_present": false,
                    "rows_passed": 2,
                    "rows_total": 2,
                    "passed": 4,
                    "total": 4,
                }),
            ),
            (
                "dataset_freeze_manifest.json",
                serde_json::json!({
                    "schema": ELCP_DATASET_FREEZE_MANIFEST_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "training_executed": false,
                    "weights_present": false,
                    "candidate_pool_locked_for_review": true,
                    "dataset_locked_for_training": false,
                    "training_allowed": false,
                    "passed": 6,
                    "total": 6,
                }),
            ),
            (
                "metrics_board.json",
                serde_json::json!({
                    "schema": ELCP_METRICS_BOARD_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "training_executed": false,
                    "weights_present": false,
                    "4b_training_allowed": false,
                    "passed": 9,
                    "total": 9,
                }),
            ),
            (
                "4b_readiness_contract.json",
                serde_json::json!({
                    "schema": ELCP_4B_READINESS_CONTRACT_SCHEMA,
                    "claim_allowed": false,
                    "agi_claim": false,
                    "training_executed": false,
                    "weights_present": false,
                    "contract_prepared": true,
                    "checkpoint_admitted": false,
                    "4b_training_allowed": false,
                    "passed": 7,
                    "total": 7,
                }),
            ),
        ];
        for (name, report) in reports {
            std::fs::write(
                report_dir.join(name),
                serde_json::to_string_pretty(&report).unwrap(),
            )
            .unwrap();
        }
    }

    fn valid_megatron_7b_inference_report() -> Value {
        serde_json::json!({
            "schema": MEGATRON_7B_INFERENCE_REPORT_SCHEMA,
            "authority": AUTHORITY,
            "claim_allowed": false,
            "agi_claim": false,
            "accepted_as": "7b_checkpoint_inference_probe",
            "source": {
                "checkpoint_path": "target/eden_megatron_7b_base_pilot/checkpoints",
                "response_path": "target/eden_megatron_7b_base_pilot/eden_7b_inference_response.json",
                "log_path": "target/eden_megatron_7b_base_pilot/eden_7b_inference_probe.log"
            },
            "run": {
                "passed": true,
                "network": "none",
                "external_model_dependency": false,
                "checkpoint_loaded": true,
                "checkpoint_admission": false,
                "production_model": false,
                "generated_count": 1,
                "tokens_to_generate": 8
            },
            "responses": [
                {
                    "prompt": "EDEN state:",
                    "generated_text": "EDEN state: candidate tokens",
                    "generated_tokens": ["candidate", "tokens"]
                }
            ],
            "safety_boundary": {
                "direct_memory_writes": false,
                "direct_objective_writes": false,
                "direct_tool_execution": false,
                "requires_gewc_admission": true,
                "outputs_are_hypotheses": true,
                "model_may_not_mutate_runtime_state": true
            }
        })
    }

    #[test]
    fn run_all_writes_all_model_runtime_artifacts() {
        let _guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_model_runtime_all_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = run_all();

        assert!(out.contains("[MODEL-RUNTIME]"));
        assert!(std::fs::metadata(state_paths::model_adapter_runtime_path()).is_ok());
        assert!(std::fs::metadata(state_paths::model_checkpoint_manifest_path()).is_ok());
        assert!(std::fs::metadata(state_paths::training_harness_report_path()).is_ok());
        assert!(std::fs::metadata(state_paths::model_governance_report_path()).is_ok());
        assert!(std::fs::metadata(state_paths::eden_70b_modular_target_path()).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn eden_70b_modular_target_is_not_a_single_model() {
        let _guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_70b_modular_target_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = write_eden_70b_modular_target();
        let body = std::fs::read_to_string(state_paths::eden_70b_modular_target_path()).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let modules = parsed["module_budget"].as_array().unwrap();
        let total_parameters: u64 = modules
            .iter()
            .map(|module| module["parameters"].as_u64().unwrap())
            .sum();

        assert!(out.contains("[EDEN-70B-MODULAR-TARGET]"));
        assert_eq!(parsed["schema"], EDEN_70B_MODULAR_TARGET_SCHEMA);
        assert_eq!(
            parsed["architecture_class"],
            "GEWC_COORDINATED_MULTI_MODEL_COGNITIVE_FAMILY"
        );
        assert_eq!(parsed["not_a_single_model"], true);
        assert_eq!(parsed["single_checkpoint_training_allowed"], false);
        assert_eq!(parsed["module_count"], 6);
        assert_eq!(parsed["budget"]["total_parameters"], 70_000_000_000u64);
        assert_eq!(
            parsed["budget"]["active_default_parameters"],
            33_000_000_000u64
        );
        assert_eq!(parsed["budget"]["single_dense_70b_core_allowed"], false);
        assert_eq!(parsed["budget"]["monolithic_llm_brain_allowed"], false);
        assert_eq!(
            parsed["budget"]["all_modules_loaded_for_every_request"],
            false
        );
        assert_eq!(parsed["budget"]["routed_subset_expected"], true);
        assert_eq!(
            parsed["budget"]["total_budget_is_family_capacity_not_active_context"],
            true
        );
        assert_eq!(modules.len(), 6);
        assert_eq!(total_parameters, 70_000_000_000u64);
        assert!(modules
            .iter()
            .all(|module| module["authority"] == AUTHORITY));
        assert!(modules
            .iter()
            .all(|module| module["routed_by_gewc"] == true));
        assert!(modules
            .iter()
            .all(|module| module["single_model_core"] == false));
        assert!(modules
            .iter()
            .all(|module| module["direct_tool_execution"] == false));
        assert!(modules
            .iter()
            .all(|module| module["direct_memory_write"] == false));
        assert!(modules
            .iter()
            .all(|module| module["direct_objective_update"] == false));
        assert_eq!(parsed["runtime_policy"]["gewc_final_authority"], true);
        assert_eq!(parsed["runtime_policy"]["models_are_subordinate"], true);
        assert_eq!(parsed["runtime_policy"]["outputs_are_hypotheses"], true);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn lifecycle_commands_keep_models_subordinate_to_gewc() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_model_runtime_lifecycle_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let _ = register_model("candidate-a");
        let _ = load_model("candidate-a");
        let _ = evaluate_model("candidate-a");
        let _ = unload_model("candidate-a");
        let body = std::fs::read_to_string(state_paths::model_adapter_runtime_path()).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();

        assert_eq!(parsed["authority"], AUTHORITY);
        assert_eq!(parsed["safety_boundary"]["direct_memory_writes"], false);
        assert_eq!(parsed["events"].as_array().unwrap().len(), 4);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn training_harness_tolerates_missing_smoke_report_without_claims() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_training_harness_missing_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = run_training_harness();
        let body = std::fs::read_to_string(state_paths::training_harness_report_path()).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();

        assert!(out.contains("[TRAINING-HARNESS]"));
        assert_eq!(parsed["claim_allowed"], false);
        assert_eq!(parsed["training_executed"], false);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn prepare_first_model_writes_4a_artifacts_without_training() {
        let _guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_first_model_prepare_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = prepare_first_model();
        let card = std::fs::read_to_string(state_paths::first_model_card_path()).unwrap();
        let plan = std::fs::read_to_string(state_paths::first_model_training_plan_path()).unwrap();
        let readiness = std::fs::read_to_string(state_paths::first_model_readiness_path()).unwrap();
        let parsed_readiness: Value = serde_json::from_str(&readiness).unwrap();

        assert!(out.contains("[FIRST-MODEL-CARD]"));
        assert!(card.contains("\"training_executed\": false"));
        assert!(plan.contains("\"gpu_job_submitted\": false"));
        assert_eq!(parsed_readiness["4a_complete"], true);
        assert_eq!(parsed_readiness["4b_training_allowed"], false);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn first_model_readiness_blocks_4b_when_preparation_is_missing() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_first_model_readiness_missing_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let _ = write_first_model_readiness();
        let body = std::fs::read_to_string(state_paths::first_model_readiness_path()).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();

        assert_eq!(parsed["4a_complete"], false);
        assert_eq!(parsed["4b_training_allowed"], false);
        assert_eq!(parsed["training_executed"], false);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn paradise_checkpoint_registry_admission_blocks_empty_public_registry() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "paradise_checkpoint_registry_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = write_paradise_checkpoint_registry_admission();
        let body =
            std::fs::read_to_string(state_paths::paradise_checkpoint_registry_admission_path())
                .unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();

        assert!(out.contains("[PARADISE-CHECKPOINT-REGISTRY]"));
        assert_eq!(
            parsed["schema"],
            PARADISE_CHECKPOINT_REGISTRY_ADMISSION_SCHEMA
        );
        assert_eq!(parsed["checkpoint_admission_allowed"], false);
        assert_eq!(parsed["production_model_allowed"], false);
        assert_eq!(parsed["admitted_entries"], 0);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn prepare_elcp_writes_native_objective_artifacts_without_training() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!("eden_elcp_prepare_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);
        let report_dir = dir.join("elcp_reports");
        write_elcp_pipeline_reports_for_test(&report_dir);
        std::env::set_var(ELCP_REPORT_DIR_ENV, &report_dir);

        let out = prepare_elcp();
        let objective = std::fs::read_to_string(state_paths::elcp_objective_spec_path()).unwrap();
        let dataset = std::fs::read_to_string(state_paths::elcp_transition_dataset_path()).unwrap();
        let plan = std::fs::read_to_string(state_paths::elcp_training_plan_path()).unwrap();
        let gate = std::fs::read_to_string(state_paths::elcp_admission_gate_path()).unwrap();
        let readiness = std::fs::read_to_string(state_paths::elcp_readiness_path()).unwrap();
        let parsed_readiness: Value = serde_json::from_str(&readiness).unwrap();

        assert!(out.contains("[ELCP-OBJECTIVE-SPEC]"));
        assert!(objective.contains("\"token_prediction_is_subordinate\": true"));
        assert!(dataset.contains("\"synthetic_fixture_only\": true"));
        assert!(plan.contains("\"gpu_job_submitted\": false"));
        assert!(gate.contains("\"checkpoint_admission_allowed\": false"));
        assert_eq!(parsed_readiness["elcp_4a_complete"], true);
        assert_eq!(parsed_readiness["4b_training_allowed"], false);

        std::env::remove_var(ELCP_REPORT_DIR_ENV);
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn elcp_readiness_blocks_4b_when_objective_artifacts_are_missing() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_elcp_readiness_missing_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let _ = write_elcp_readiness();
        let body = std::fs::read_to_string(state_paths::elcp_readiness_path()).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();

        assert_eq!(parsed["elcp_4a_complete"], false);
        assert_eq!(parsed["4b_training_allowed"], false);
        assert_eq!(parsed["training_executed"], false);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn megatron_7b_inference_report_accepts_checkpoint_load_probe_only() {
        validate_megatron_7b_inference_report_value(&valid_megatron_7b_inference_report()).unwrap();
    }

    #[test]
    fn megatron_7b_inference_report_rejects_checkpoint_admission() {
        let mut report = valid_megatron_7b_inference_report();
        report["run"]["checkpoint_admission"] = Value::Bool(true);
        let err = validate_megatron_7b_inference_report_value(&report).unwrap_err();
        assert!(err.contains("checkpoint_admission"));
    }

    #[test]
    fn megatron_7b_inference_report_can_be_copied_into_state() {
        let _guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_megatron_7b_inference_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("source_inference_report.json");
        std::fs::write(
            &source,
            serde_json::to_string_pretty(&valid_megatron_7b_inference_report()).unwrap(),
        )
        .unwrap();

        let path = copy_megatron_7b_inference_report_from_path(&source.to_string_lossy()).unwrap();
        let written = std::fs::read_to_string(path).unwrap();
        let parsed: Value = serde_json::from_str(&written).unwrap();

        assert_eq!(parsed["schema"], MEGATRON_7B_INFERENCE_REPORT_SCHEMA);
        assert_eq!(parsed["run"]["checkpoint_admission"], false);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
