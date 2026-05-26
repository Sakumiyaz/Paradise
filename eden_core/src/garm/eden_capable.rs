use crate::eden_garm::state_paths;
use serde_json::Value;

pub const EDEN_CAPABLE_TRAINING_RUN_CONTRACT_SCHEMA: &str = "eden.capable.training_run_contract.v1";
pub const EDEN_COGNITIVE_DATASET_MANIFEST_SCHEMA: &str =
    "eden.capable.cognitive_dataset_manifest.v1";
pub const EDEN_NATIVE_INFERENCE_API_SCHEMA: &str = "eden.capable.native_inference_api.v1";
pub const EDEN_CAPABILITY_DELTA_EVAL_SCHEMA: &str = "eden.capable.capability_delta_eval.v1";
pub const EDEN_STRUCTURED_OUTPUT_REPORT_SCHEMA: &str = "eden.capable.structured_output_report.v1";
pub const EDEN_CHECKPOINT_REGISTRY_SCHEMA: &str = "eden.capable.checkpoint_registry.v1";
pub const EDEN_SFT_ELCP_READINESS_SCHEMA: &str = "eden.capable.sft_elcp_readiness.v1";
pub const EDEN_CAPABLE_GATE_SCHEMA: &str = "eden.capable.gate.v1";
pub const EDEN_LIVE_INFERENCE_RUNTIME_SCHEMA: &str = "eden.capable.live_inference_runtime.v1";
pub const EDEN_COGNITIVE_CALL_CONTRACT_SCHEMA: &str = "eden.capable.cognitive_call_contract.v1";
pub const EDEN_COGNITIVE_DATASET_EXPANSION_SCHEMA: &str =
    "eden.capable.cognitive_dataset_expansion.v1";
pub const EDEN_CAPABILITY_EVAL_SUITE_SCHEMA: &str = "eden.capable.capability_eval_suite.v1";
pub const EDEN_SFT_ELCP_ACTIVATION_GATE_SCHEMA: &str = "eden.capable.sft_elcp_activation_gate.v1";
pub const EDEN_MEMORY_ACTION_LOOP_SCHEMA: &str = "eden.capable.memory_action_loop.v1";
pub const EDEN_CAPABLE_DEMO_TRACE_SCHEMA: &str = "eden.capable.demo_trace.v1";
pub const EDEN_CAPABLE_OPERATIONAL_GATE_SCHEMA: &str = "eden.capable.operational_gate.v1";

const AUTHORITY: &str = "global_executive_workspace_core";
const MODEL_ID: &str = "eden-megatron-7b-base-pilot";
const COGNITIVE_DATASET_PATH: &str = "training/data/eden_cognitive_capability_seed.jsonl";
const TRAINING_EVIDENCE_PATH: &str =
    "target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json";
const INFERENCE_REPORT_PATH: &str =
    "target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json";

pub fn run_all() -> String {
    let mut out = String::new();
    out.push_str(&write_training_run_contract());
    out.push_str(&write_cognitive_dataset_manifest());
    out.push_str(&write_native_inference_api());
    out.push_str(&write_capability_delta_eval());
    out.push_str(&write_structured_output_report());
    out.push_str(&write_checkpoint_registry());
    out.push_str(&write_sft_elcp_readiness());
    out.push_str(&write_capable_gate());
    out.push_str(&run_operationalization_all());
    out
}

pub fn run_operationalization_all() -> String {
    let mut out = String::new();
    out.push_str(&write_live_inference_runtime());
    out.push_str(&write_cognitive_call_contract());
    out.push_str(&write_cognitive_dataset_expansion());
    out.push_str(&write_capability_eval_suite());
    out.push_str(&write_sft_elcp_activation_gate());
    out.push_str(&write_memory_action_loop());
    out.push_str(&write_capable_demo_trace());
    out.push_str(&write_capable_operational_gate());
    out
}

pub fn write_training_run_contract() -> String {
    write_report(
        "EDEN-CAPABLE-TRAINING-RUN",
        EDEN_CAPABLE_TRAINING_RUN_CONTRACT_SCHEMA,
        state_paths::eden_capable_training_run_contract_path(),
        training_run_contract_value(),
    )
}

pub fn write_cognitive_dataset_manifest() -> String {
    write_report(
        "EDEN-COGNITIVE-DATASET",
        EDEN_COGNITIVE_DATASET_MANIFEST_SCHEMA,
        state_paths::eden_cognitive_dataset_manifest_path(),
        cognitive_dataset_manifest_value(),
    )
}

pub fn write_native_inference_api() -> String {
    write_report(
        "EDEN-NATIVE-INFERENCE-API",
        EDEN_NATIVE_INFERENCE_API_SCHEMA,
        state_paths::eden_native_inference_api_path(),
        native_inference_api_value(),
    )
}

pub fn write_capability_delta_eval() -> String {
    write_report(
        "EDEN-CAPABILITY-DELTA",
        EDEN_CAPABILITY_DELTA_EVAL_SCHEMA,
        state_paths::eden_capability_delta_eval_path(),
        capability_delta_eval_value(),
    )
}

pub fn write_structured_output_report() -> String {
    write_report(
        "EDEN-STRUCTURED-OUTPUT",
        EDEN_STRUCTURED_OUTPUT_REPORT_SCHEMA,
        state_paths::eden_structured_output_report_path(),
        structured_output_report_value(),
    )
}

pub fn write_checkpoint_registry() -> String {
    write_report(
        "EDEN-CHECKPOINT-REGISTRY",
        EDEN_CHECKPOINT_REGISTRY_SCHEMA,
        state_paths::eden_checkpoint_registry_path(),
        checkpoint_registry_value(),
    )
}

pub fn write_sft_elcp_readiness() -> String {
    write_report(
        "EDEN-SFT-ELCP-READINESS",
        EDEN_SFT_ELCP_READINESS_SCHEMA,
        state_paths::eden_sft_elcp_readiness_path(),
        sft_elcp_readiness_value(),
    )
}

pub fn write_capable_gate() -> String {
    write_report(
        "EDEN-CAPABLE-GATE",
        EDEN_CAPABLE_GATE_SCHEMA,
        state_paths::eden_capable_gate_path(),
        capable_gate_value(),
    )
}

pub fn write_live_inference_runtime() -> String {
    write_report(
        "EDEN-LIVE-INFERENCE-RUNTIME",
        EDEN_LIVE_INFERENCE_RUNTIME_SCHEMA,
        state_paths::eden_live_inference_runtime_path(),
        live_inference_runtime_value(),
    )
}

pub fn write_cognitive_call_contract() -> String {
    write_report(
        "EDEN-COGNITIVE-CALL-CONTRACT",
        EDEN_COGNITIVE_CALL_CONTRACT_SCHEMA,
        state_paths::eden_cognitive_call_contract_path(),
        cognitive_call_contract_value(),
    )
}

pub fn write_cognitive_dataset_expansion() -> String {
    write_report(
        "EDEN-COGNITIVE-DATASET-EXPANSION",
        EDEN_COGNITIVE_DATASET_EXPANSION_SCHEMA,
        state_paths::eden_cognitive_dataset_expansion_path(),
        cognitive_dataset_expansion_value(),
    )
}

pub fn write_capability_eval_suite() -> String {
    write_report(
        "EDEN-CAPABILITY-EVAL-SUITE",
        EDEN_CAPABILITY_EVAL_SUITE_SCHEMA,
        state_paths::eden_capability_eval_suite_path(),
        capability_eval_suite_value(),
    )
}

pub fn write_sft_elcp_activation_gate() -> String {
    write_report(
        "EDEN-SFT-ELCP-ACTIVATION-GATE",
        EDEN_SFT_ELCP_ACTIVATION_GATE_SCHEMA,
        state_paths::eden_sft_elcp_activation_gate_path(),
        sft_elcp_activation_gate_value(),
    )
}

pub fn write_memory_action_loop() -> String {
    write_report(
        "EDEN-MEMORY-ACTION-LOOP",
        EDEN_MEMORY_ACTION_LOOP_SCHEMA,
        state_paths::eden_memory_action_loop_path(),
        memory_action_loop_value(),
    )
}

pub fn write_capable_demo_trace() -> String {
    write_report(
        "EDEN-CAPABLE-DEMO-TRACE",
        EDEN_CAPABLE_DEMO_TRACE_SCHEMA,
        state_paths::eden_capable_demo_trace_path(),
        capable_demo_trace_value(),
    )
}

pub fn write_capable_operational_gate() -> String {
    write_report(
        "EDEN-CAPABLE-OPERATIONAL-GATE",
        EDEN_CAPABLE_OPERATIONAL_GATE_SCHEMA,
        state_paths::eden_capable_operational_gate_path(),
        capable_operational_gate_value(),
    )
}

fn training_run_contract_value() -> Value {
    let training = read_repo_json(TRAINING_EVIDENCE_PATH);
    serde_json::json!({
        "schema": EDEN_CAPABLE_TRAINING_RUN_CONTRACT_SCHEMA,
        "artifact": "eden_capable_training_run_contract",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 1,
        "name": "real_7b_training_continuation",
        "status": "prepared_not_running",
        "gpu_required": true,
        "gpu_use_started": false,
        "operator_approval_required": true,
        "purpose": "Define the controlled path from the short 7B checkpoint-load probe to a longer EDEN-owned training run without pretending the longer run has already happened.",
        "current_training_evidence": training_summary(training.as_ref()),
        "required_before_execution": [
            "operator_gpu_budget_approval",
            "dataset_scope_freeze",
            "checkpoint_output_dir",
            "pre_eval_snapshot",
            "post_eval_plan",
            "rollback_policy"
        ],
        "prepared_command": {
            "target": "make training-megatron-eden-7b-base-pilot",
            "example_env": {
                "EDEN_MEGATRON_7B_TRAIN_ITERS": "future_operator_value",
                "EDEN_MEGATRON_7B_SAVE_CHECKPOINT": "true",
                "EDEN_MEGATRON_7B_SAVE_INTERVAL": "future_operator_value"
            },
            "network": "none",
            "external_model_dependency": false
        },
        "blocked_until": [
            "explicit_gpu_run_request",
            "SFT_ELCP_dataset_review",
            "safety_eval_plan"
        ],
        "safety_boundary": safety_boundary(),
    })
}

fn cognitive_dataset_manifest_value() -> Value {
    let records = read_cognitive_records();
    let valid = records
        .iter()
        .filter(|record| cognitive_record_valid(record))
        .count();
    serde_json::json!({
        "schema": EDEN_COGNITIVE_DATASET_MANIFEST_SCHEMA,
        "artifact": "eden_cognitive_dataset_manifest",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 2,
        "dataset": {
            "name": "eden_cognitive_capability_seed",
            "path": COGNITIVE_DATASET_PATH,
            "format": "jsonl",
            "source": "synthetic_repo_local_cognitive_state_records",
            "contains_private_data": false,
            "records": records.len(),
            "valid_records": valid
        },
        "record_contract": {
            "required_fields": [
                "id",
                "input",
                "target",
                "governance"
            ],
            "input_fields": [
                "situation",
                "goal",
                "working_memory",
                "world_state",
                "risk_context",
                "available_tools"
            ],
            "target_fields": [
                "structured_response",
                "plan",
                "memory_action",
                "world_delta",
                "safety_gate",
                "uncertainty"
            ]
        },
        "accepted_for": [
            "SFT_seed",
            "ELCP_seed",
            "structured_inference_parser_tests",
            "capability_delta_eval"
        ],
        "not_accepted_for": [
            "production_training_without_review",
            "private_user_memory",
            "AGI_claim"
        ],
        "safety_boundary": safety_boundary(),
    })
}

fn native_inference_api_value() -> Value {
    let inference = read_repo_json(INFERENCE_REPORT_PATH);
    serde_json::json!({
        "schema": EDEN_NATIVE_INFERENCE_API_SCHEMA,
        "artifact": "eden_native_inference_api",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 3,
        "status": "contract_ready_probe_backed",
        "model_id": MODEL_ID,
        "purpose": "Define the native GEWC request/response boundary for using the 7B checkpoint as a subordinate candidate generator.",
        "request": {
            "schema": "eden.structured_inference_request.v1",
            "fields": [
                "task_id",
                "goal",
                "situation",
                "working_memory_refs",
                "risk_class",
                "allowed_output_modes",
                "max_tokens",
                "permission_context"
            ]
        },
        "response": {
            "schema": "eden.structured_inference_packet.v1",
            "fields": [
                "candidate_text",
                "candidate_structure",
                "confidence",
                "risk_notes",
                "requires_verification",
                "source_model",
                "provenance"
            ]
        },
        "current_probe": inference_status(inference.as_ref()),
        "authority_rules": {
            "model_outputs_are_hypotheses": true,
            "GEWC_must_validate_before_state_change": true,
            "direct_memory_write_allowed": false,
            "direct_tool_execution_allowed": false,
            "direct_objective_update_allowed": false
        },
        "safety_boundary": safety_boundary(),
    })
}

fn capability_delta_eval_value() -> Value {
    let training = read_repo_json(TRAINING_EVIDENCE_PATH);
    let inference = read_repo_json(INFERENCE_REPORT_PATH);
    let structured = structured_packets_from_inference(inference.as_ref());
    let checks = vec![
        check(
            "training_evidence_present",
            training.is_some(),
            TRAINING_EVIDENCE_PATH,
        ),
        check(
            "inference_probe_present",
            inference.is_some(),
            INFERENCE_REPORT_PATH,
        ),
        check(
            "checkpoint_loaded",
            inference
                .as_ref()
                .and_then(|value| value.pointer("/run/checkpoint_loaded"))
                .and_then(Value::as_bool)
                == Some(true),
            "inference_report.run.checkpoint_loaded=true",
        ),
        check(
            "generated_tokens_observed",
            !structured.is_empty(),
            "structured packets generated from inference report",
        ),
        check(
            "claim_boundary_preserved",
            no_claim(inference.as_ref()) && no_claim(training.as_ref()),
            "claim_allowed=false agi_claim=false",
        ),
    ];
    let passed = checks
        .iter()
        .filter(|check| check["passed"] == Value::Bool(true))
        .count();
    serde_json::json!({
        "schema": EDEN_CAPABILITY_DELTA_EVAL_SCHEMA,
        "artifact": "eden_capability_delta_eval",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 4,
        "passed": passed,
        "total": checks.len(),
        "checks": checks,
        "before": {
            "state": "architecture_ready_without_loaded_7b_checkpoint",
            "usable_model_capacity": false
        },
        "after": {
            "state": "7b_checkpoint_load_and_token_generation_probe",
            "usable_model_capacity": inference.is_some() && !structured.is_empty(),
            "semantic_capability_admitted": false,
            "production_inference_admitted": false
        },
        "metrics": {
            "generated_structured_packets": structured.len(),
            "checkpoint_loaded": inference
                .as_ref()
                .and_then(|value| value.pointer("/run/checkpoint_loaded"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
        },
        "safety_boundary": safety_boundary(),
    })
}

fn structured_output_report_value() -> Value {
    let inference = read_repo_json(INFERENCE_REPORT_PATH);
    let packets = structured_packets_from_inference(inference.as_ref());
    serde_json::json!({
        "schema": EDEN_STRUCTURED_OUTPUT_REPORT_SCHEMA,
        "artifact": "eden_structured_output_report",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 5,
        "source": INFERENCE_REPORT_PATH,
        "source_present": inference.is_some(),
        "packet_count": packets.len(),
        "packets": packets,
        "parser_contract": {
            "raw_text_never_becomes_truth": true,
            "raw_text_never_writes_memory": true,
            "candidate_packet_requires_verification": true,
            "GEWC_selects_or_rejects_packet": true
        },
        "safety_boundary": safety_boundary(),
    })
}

fn checkpoint_registry_value() -> Value {
    let training = read_repo_json(TRAINING_EVIDENCE_PATH);
    let inference = read_repo_json(INFERENCE_REPORT_PATH);
    let checkpoint = training
        .as_ref()
        .and_then(|value| value.pointer("/checkpoint_policy/checkpoint_path"))
        .and_then(Value::as_str)
        .unwrap_or("target/eden_megatron_7b_base_pilot/checkpoints");
    serde_json::json!({
        "schema": EDEN_CHECKPOINT_REGISTRY_SCHEMA,
        "artifact": "eden_checkpoint_registry",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 6,
        "registry_policy": {
            "checkpoints_stay_out_of_git": true,
            "admission_requires_eval": true,
            "rollback_required": true,
            "hash_required_before_release": true
        },
        "checkpoints": [
            {
                "model_id": MODEL_ID,
                "checkpoint_path": checkpoint,
                "source_training_evidence": TRAINING_EVIDENCE_PATH,
                "source_inference_report": INFERENCE_REPORT_PATH,
                "training_evidence_present": training.is_some(),
                "inference_report_present": inference.is_some(),
                "checkpoint_loaded": inference
                    .as_ref()
                    .and_then(|value| value.pointer("/run/checkpoint_loaded"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                "checkpoint_admission": false,
                "weights_admitted": false,
                "production_model": false,
                "registry_status": "probe_registered_not_released"
            }
        ],
        "safety_boundary": safety_boundary(),
    })
}

fn sft_elcp_readiness_value() -> Value {
    let dataset_manifest = read_json_file(&state_paths::eden_cognitive_dataset_manifest_path());
    let structured_report = read_json_file(&state_paths::eden_structured_output_report_path());
    let dataset_records = dataset_manifest
        .as_ref()
        .and_then(|value| value.pointer("/dataset/valid_records"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let structured_packets = structured_report
        .as_ref()
        .and_then(|value| value.get("packet_count"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    serde_json::json!({
        "schema": EDEN_SFT_ELCP_READINESS_SCHEMA,
        "artifact": "eden_sft_elcp_readiness",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 7,
        "status": "prepared_not_training",
        "training_allowed": false,
        "gpu_job_submitted": false,
        "model_id": MODEL_ID,
        "inputs": {
            "cognitive_dataset_manifest": state_paths::eden_cognitive_dataset_manifest_path(),
            "structured_output_report": state_paths::eden_structured_output_report_path(),
            "dataset_valid_records": dataset_records,
            "structured_packets": structured_packets
        },
        "SFT_scope": {
            "target": "make model produce structured EDEN packets from governed task context",
            "requires": [
                "more reviewed cognitive records",
                "held_out_eval_split",
                "safety_counterexamples",
                "pre_train_eval",
                "post_train_eval"
            ]
        },
        "ELCP_scope": {
            "target": "predict next governed cognitive state rather than only surface tokens",
            "loss_targets": [
                "situation_state",
                "goal_state",
                "memory_transition",
                "world_delta",
                "plan_transition",
                "risk_calibration",
                "safety_gate"
            ]
        },
        "blocked_until": [
            "operator_approves_gpu_training",
            "dataset_size_and_quality_threshold_met",
            "adversarial_safety_eval_ready"
        ],
        "safety_boundary": safety_boundary(),
    })
}

fn capable_gate_value() -> Value {
    let checks = vec![
        artifact_check(
            "training_run_contract",
            state_paths::eden_capable_training_run_contract_path(),
        ),
        artifact_check(
            "cognitive_dataset_manifest",
            state_paths::eden_cognitive_dataset_manifest_path(),
        ),
        artifact_check(
            "native_inference_api",
            state_paths::eden_native_inference_api_path(),
        ),
        artifact_check(
            "capability_delta_eval",
            state_paths::eden_capability_delta_eval_path(),
        ),
        artifact_check(
            "structured_output_report",
            state_paths::eden_structured_output_report_path(),
        ),
        artifact_check(
            "checkpoint_registry",
            state_paths::eden_checkpoint_registry_path(),
        ),
        artifact_check(
            "sft_elcp_readiness",
            state_paths::eden_sft_elcp_readiness_path(),
        ),
    ];
    let passed = checks
        .iter()
        .filter(|check| check["passed"] == Value::Bool(true))
        .count();
    let inference = read_repo_json(INFERENCE_REPORT_PATH);
    serde_json::json!({
        "schema": EDEN_CAPABLE_GATE_SCHEMA,
        "artifact": "eden_capable_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": checks.len(),
        "checks": checks,
        "eden_capable_surface_ready": passed == checks.len(),
        "runtime_checkpoint_probe_available": inference
            .as_ref()
            .and_then(|value| value.pointer("/run/checkpoint_loaded"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "capability_class": "governed_checkpoint_probe_plus_training_path",
        "not_yet": [
            "long_training_run",
            "semantic_competence",
            "external_benchmark_validation",
            "autonomous_tool_authority",
            "AGI"
        ],
        "next_recommended_step": "expand reviewed EDEN cognitive dataset and run a controlled SFT/ELCP pilot when GPU use is explicitly requested",
        "safety_boundary": safety_boundary(),
    })
}

fn live_inference_runtime_value() -> Value {
    let inference = read_repo_json(INFERENCE_REPORT_PATH);
    let packets = structured_packets_from_inference(inference.as_ref());
    let checkpoint_loaded = checkpoint_loaded(inference.as_ref());
    serde_json::json!({
        "schema": EDEN_LIVE_INFERENCE_RUNTIME_SCHEMA,
        "artifact": "eden_live_inference_runtime",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 1,
        "status": if checkpoint_loaded { "probe_backed_callable" } else { "blocked_missing_probe" },
        "runtime_mode": "local_report_backed_candidate_generator",
        "model_id": MODEL_ID,
        "checkpoint_loaded": checkpoint_loaded,
        "callable_inside_runtime": checkpoint_loaded && !packets.is_empty(),
        "new_gpu_inference_started": false,
        "external_model_dependency": false,
        "input_boundary": "eden.structured_inference_request.v1",
        "output_boundary": "eden.structured_inference_packet.v1",
        "available_probe_packets": packets.len(),
        "authority_rules": {
            "model_is_subordinate": true,
            "raw_text_is_never_state": true,
            "GEWC_must_select_or_reject": true,
            "verifier_required_before_memory_or_action": true
        },
        "limitations": [
            "does_not_generate_new_tokens_without_gpu",
            "does_not_admit_semantic_competence",
            "does_not_release_checkpoint"
        ],
        "safety_boundary": safety_boundary(),
    })
}

fn cognitive_call_contract_value() -> Value {
    serde_json::json!({
        "schema": EDEN_COGNITIVE_CALL_CONTRACT_SCHEMA,
        "artifact": "eden_cognitive_call_contract",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 2,
        "purpose": "Make the GEWC-to-model call a typed cognitive transaction instead of a direct text prompt.",
        "pipeline": [
            {"index": 1, "component": "GEWC", "effect": "accepts_goal_and_permission_context"},
            {"index": 2, "component": "model_router", "effect": "selects_eden_7b_probe_only_when_cost_risk_and_checkpoint_state_allow"},
            {"index": 3, "component": "eden_7b_candidate_generator", "effect": "returns_candidate_text_without_state_authority"},
            {"index": 4, "component": "structured_packet_parser", "effect": "wraps_text_as_untrusted_hypothesis_packet"},
            {"index": 5, "component": "verifier", "effect": "checks_claims_permissions_risk_and_schema"},
            {"index": 6, "component": "memory_action_gate", "effect": "admits_only_audit_memory_or_draft_plan_without_explicit_approval"}
        ],
        "request": {
            "required": ["task_id", "goal", "situation", "permission_context", "risk_class"],
            "optional": ["working_memory_refs", "world_state_refs", "allowed_output_modes", "max_tokens"]
        },
        "response": {
            "required": ["packet_id", "candidate_structure", "verification", "admission"],
            "admission_values": ["reject", "audit_only", "draft_plan", "needs_human_approval"]
        },
        "forbidden_direct_effects": [
            "write_memory",
            "change_objective",
            "execute_tool",
            "claim_truth",
            "escalate_autonomy"
        ],
        "safety_boundary": safety_boundary(),
    })
}

fn cognitive_dataset_expansion_value() -> Value {
    let records = read_cognitive_records();
    let domains = unique_record_domains(&records);
    let valid = records
        .iter()
        .filter(|record| cognitive_record_valid(record))
        .count();
    serde_json::json!({
        "schema": EDEN_COGNITIVE_DATASET_EXPANSION_SCHEMA,
        "artifact": "eden_cognitive_dataset_expansion",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 3,
        "dataset": COGNITIVE_DATASET_PATH,
        "records": records.len(),
        "valid_records": valid,
        "domains": domains,
        "coverage_targets": [
            "planning",
            "memory_governance",
            "claim_safety",
            "tool_use",
            "world_model",
            "rollback",
            "metacognition",
            "multiagent_coordination",
            "permission_escalation",
            "checkpoint_probe_routing"
        ],
        "ready_for": [
            "structured_parser_tests",
            "capability_eval_suite",
            "future_reviewed_SFT_seed",
            "future_ELCP_transition_seed"
        ],
        "not_ready_for": [
            "unsupervised_production_training",
            "private_personal_memory",
            "AGI_claim"
        ],
        "safety_boundary": safety_boundary(),
    })
}

fn capability_eval_suite_value() -> Value {
    let records = read_cognitive_records();
    let tasks: Vec<Value> = records
        .iter()
        .enumerate()
        .map(|(idx, record)| {
            let id = record
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("unknown_record");
            let domain = record
                .pointer("/input/world_state/domain")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            serde_json::json!({
                "task_id": format!("capability-eval-{:02}", idx + 1),
                "source_record": id,
                "domain": domain,
                "dimensions": [
                    "structured_state",
                    "permission_gate",
                    "memory_action_boundary",
                    "uncertainty",
                    "auditability"
                ],
                "current_status": if cognitive_record_valid(record) { "contract_passed" } else { "invalid_record" },
                "semantic_model_score": null,
                "notes": "This eval validates operational capability contracts now; semantic model scores require a future governed model run."
            })
        })
        .collect();
    let passed = tasks
        .iter()
        .filter(|task| task["current_status"] == Value::String("contract_passed".to_string()))
        .count();
    serde_json::json!({
        "schema": EDEN_CAPABILITY_EVAL_SUITE_SCHEMA,
        "artifact": "eden_capability_eval_suite",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 4,
        "passed": passed,
        "total": tasks.len(),
        "tasks": tasks,
        "measures": [
            "generality_surface",
            "transfer_surface",
            "autonomy_boundary",
            "safe_memory",
            "tool_governance",
            "world_model_contract",
            "metacognitive_uncertainty"
        ],
        "current_scope": "runtime_contract_eval_not_semantic_benchmark",
        "future_scope": "compare_pre_sft_vs_post_sft_outputs_with_same_tasks",
        "safety_boundary": safety_boundary(),
    })
}

fn sft_elcp_activation_gate_value() -> Value {
    let dataset_manifest = cognitive_dataset_manifest_value();
    let valid_records = dataset_manifest
        .pointer("/dataset/valid_records")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let inference = read_repo_json(INFERENCE_REPORT_PATH);
    let checkpoint_ready = checkpoint_loaded(inference.as_ref());
    let readiness_checks = vec![
        check("dataset_present", valid_records > 0, COGNITIVE_DATASET_PATH),
        check(
            "checkpoint_probe_present",
            checkpoint_ready,
            INFERENCE_REPORT_PATH,
        ),
        check(
            "operator_gpu_approval_absent",
            true,
            "no GPU training is started by this local command",
        ),
        check(
            "no_claim_boundary",
            true,
            "training remains blocked until reviewed data and explicit approval",
        ),
    ];
    let passed = readiness_checks
        .iter()
        .filter(|check| check["passed"] == Value::Bool(true))
        .count();
    serde_json::json!({
        "schema": EDEN_SFT_ELCP_ACTIVATION_GATE_SCHEMA,
        "artifact": "eden_sft_elcp_activation_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 5,
        "passed": passed,
        "total": readiness_checks.len(),
        "checks": readiness_checks,
        "training_allowed_now": false,
        "gpu_job_submitted": false,
        "activation_requirements": [
            "operator_approves_gpu_budget",
            "dataset_review_passes",
            "held_out_eval_split_exists",
            "pre_train_eval_snapshot_written",
            "rollback_checkpoint_policy_written"
        ],
        "prepared_training_modes": [
            "SFT_structured_packet_following",
            "ELCP_latent_cognitive_state_prediction"
        ],
        "safety_boundary": safety_boundary(),
    })
}

fn memory_action_loop_value() -> Value {
    let inference = read_repo_json(INFERENCE_REPORT_PATH);
    let packet = structured_packets_from_inference(inference.as_ref())
        .into_iter()
        .next()
        .unwrap_or_else(|| {
            serde_json::json!({
                "packet_id": "missing-probe-packet",
                "candidate_structure": {
                    "kind": "missing_probe",
                    "requires_verification": true
                },
                "authority": {
                    "accepted_as_truth": false
                }
            })
        });
    serde_json::json!({
        "schema": EDEN_MEMORY_ACTION_LOOP_SCHEMA,
        "artifact": "eden_memory_action_loop",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 6,
        "cycle": [
            {"stage": "observe", "result": "task_context_loaded"},
            {"stage": "route", "result": "eden_7b_probe_available_as_candidate_generator"},
            {"stage": "generate", "result": "candidate_packet_created", "packet_id": packet["packet_id"]},
            {"stage": "verify", "result": "raw_text_not_accepted_as_truth"},
            {"stage": "plan", "result": "draft_plan_allowed"},
            {"stage": "memory", "result": "audit_metadata_allowed_model_content_not_persisted_as_fact"},
            {"stage": "action", "result": "external_or_mutating_actions_require_approval"},
            {"stage": "learn", "result": "only_eval_metadata_can_update_without_training"}
        ],
        "packet": packet,
        "admission": {
            "memory_fact_write": "blocked",
            "objective_update": "blocked",
            "tool_execution": "blocked",
            "audit_metadata": "allowed",
            "draft_plan": "allowed"
        },
        "rollback": {
            "required_for_mutating_actions": true,
            "available_for_memory_transactions": true
        },
        "safety_boundary": safety_boundary(),
    })
}

fn capable_demo_trace_value() -> Value {
    serde_json::json!({
        "schema": EDEN_CAPABLE_DEMO_TRACE_SCHEMA,
        "artifact": "eden_capable_demo_trace",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "step": 7,
        "demo": "safe_repo_change_with_7b_probe_candidate",
        "steps": [
            {"index": 1, "name": "receive_task", "observable": "operator asks EDEN to improve a repo safely"},
            {"index": 2, "name": "build_situation", "observable": "GEWC creates structured context and risk class"},
            {"index": 3, "name": "retrieve_memory", "observable": "only approved memory refs enter working context"},
            {"index": 4, "name": "route_model", "observable": "7B checkpoint is callable as probe-backed candidate generator"},
            {"index": 5, "name": "parse_packet", "observable": "raw text becomes untrusted structured hypothesis"},
            {"index": 6, "name": "verify", "observable": "claim, permission and memory gates run before admission"},
            {"index": 7, "name": "dry_run_action", "observable": "mutating action remains draft-only"},
            {"index": 8, "name": "audit", "observable": "evidence artifact records what was accepted, blocked and why"}
        ],
        "expected_user_visible_result": "EDEN can show a governed plan and evidence bundle, while blocking direct state mutation from the model.",
        "not_demonstrated": [
            "new 7B generation in this no-GPU run",
            "semantic competence",
            "autonomous production execution",
            "AGI"
        ],
        "safety_boundary": safety_boundary(),
    })
}

fn capable_operational_gate_value() -> Value {
    let checks = vec![
        artifact_check(
            "live_inference_runtime",
            state_paths::eden_live_inference_runtime_path(),
        ),
        artifact_check(
            "cognitive_call_contract",
            state_paths::eden_cognitive_call_contract_path(),
        ),
        artifact_check(
            "cognitive_dataset_expansion",
            state_paths::eden_cognitive_dataset_expansion_path(),
        ),
        artifact_check(
            "capability_eval_suite",
            state_paths::eden_capability_eval_suite_path(),
        ),
        artifact_check(
            "sft_elcp_activation_gate",
            state_paths::eden_sft_elcp_activation_gate_path(),
        ),
        artifact_check(
            "memory_action_loop",
            state_paths::eden_memory_action_loop_path(),
        ),
        artifact_check(
            "capable_demo_trace",
            state_paths::eden_capable_demo_trace_path(),
        ),
    ];
    let passed = checks
        .iter()
        .filter(|check| check["passed"] == Value::Bool(true))
        .count();
    serde_json::json!({
        "schema": EDEN_CAPABLE_OPERATIONAL_GATE_SCHEMA,
        "artifact": "eden_capable_operational_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": checks.len(),
        "checks": checks,
        "eden_capable_operational_surface_ready": passed == checks.len(),
        "capability_class": "governed_cognitive_runtime_surface",
        "model_authority": "subordinate_hypothesis_generator",
        "production_release": false,
        "next_recommended_step": "run a reviewed SFT/ELCP pilot only after explicit GPU approval",
        "safety_boundary": safety_boundary(),
    })
}

fn structured_packets_from_inference(inference: Option<&Value>) -> Vec<Value> {
    inference
        .and_then(|value| value.get("responses"))
        .and_then(Value::as_array)
        .map(|responses| {
            responses
                .iter()
                .enumerate()
                .filter_map(|(idx, response)| {
                    let prompt = response.get("prompt").and_then(Value::as_str)?;
                    let generated_text = response.get("generated_text").and_then(Value::as_str)?;
                    Some(serde_json::json!({
                        "packet_id": format!("eden-7b-packet-{}", idx + 1),
                        "source_model": MODEL_ID,
                        "prompt": prompt,
                        "raw_model_text": generated_text,
                        "candidate_structure": {
                            "kind": "surface_token_hypothesis",
                            "hypothesis_text": generated_text,
                            "confidence": 0.0,
                            "uncertainty": "untrained_short_pilot_output",
                            "requires_verification": true,
                            "memory_action": "none",
                            "objective_action": "none",
                            "tool_action": "none"
                        },
                        "authority": {
                            "GEWC_final_authority": true,
                            "model_may_not_mutate_state": true,
                            "accepted_as_truth": false
                        }
                    }))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn read_cognitive_records() -> Vec<Value> {
    let Some(path) = repo_path(COGNITIVE_DATASET_PATH) else {
        return Vec::new();
    };
    let Ok(body) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    body.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect()
}

fn cognitive_record_valid(record: &Value) -> bool {
    record.get("id").and_then(Value::as_str).is_some()
        && record.get("input").and_then(Value::as_object).is_some()
        && record.get("target").and_then(Value::as_object).is_some()
        && record
            .pointer("/governance/claim_allowed")
            .and_then(Value::as_bool)
            == Some(false)
        && record
            .pointer("/governance/agi_claim")
            .and_then(Value::as_bool)
            == Some(false)
}

fn unique_record_domains(records: &[Value]) -> Vec<String> {
    let mut domains = Vec::new();
    for record in records {
        let Some(domain) = record
            .pointer("/input/world_state/domain")
            .and_then(Value::as_str)
        else {
            continue;
        };
        if !domains.iter().any(|known| known == domain) {
            domains.push(domain.to_string());
        }
    }
    domains
}

fn checkpoint_loaded(inference: Option<&Value>) -> bool {
    inference
        .and_then(|value| value.pointer("/run/checkpoint_loaded"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn training_summary(training: Option<&Value>) -> Value {
    match training {
        Some(value) => serde_json::json!({
            "present": true,
            "completed_iterations": value.pointer("/run/completed_iterations").cloned().unwrap_or(Value::Null),
            "final_loss": value.pointer("/run/final_loss").cloned().unwrap_or(Value::Null),
            "checkpoint_written": value.pointer("/checkpoint_policy/checkpoint_written").cloned().unwrap_or(Value::Null),
            "checkpoint_admission": false
        }),
        None => serde_json::json!({
            "present": false,
            "expected_path": TRAINING_EVIDENCE_PATH
        }),
    }
}

fn inference_status(inference: Option<&Value>) -> Value {
    match inference {
        Some(value) => serde_json::json!({
            "present": true,
            "checkpoint_loaded": value.pointer("/run/checkpoint_loaded").cloned().unwrap_or(Value::Null),
            "generated_count": value.pointer("/run/generated_count").cloned().unwrap_or(Value::Null),
            "network": value.pointer("/run/network").cloned().unwrap_or(Value::Null),
            "checkpoint_admission": false
        }),
        None => serde_json::json!({
            "present": false,
            "expected_path": INFERENCE_REPORT_PATH
        }),
    }
}

fn no_claim(value: Option<&Value>) -> bool {
    value
        .map(|value| {
            value.get("claim_allowed").and_then(Value::as_bool) == Some(false)
                && value.get("agi_claim").and_then(Value::as_bool) == Some(false)
        })
        .unwrap_or(true)
}

fn check(name: &str, passed: bool, evidence: &str) -> Value {
    serde_json::json!({
        "check": name,
        "passed": passed,
        "evidence": evidence,
    })
}

fn artifact_check(name: &str, path: String) -> Value {
    check(name, std::fs::metadata(&path).is_ok(), &path)
}

fn safety_boundary() -> Value {
    serde_json::json!({
        "GEWC_final_authority": true,
        "direct_memory_writes": false,
        "direct_objective_writes": false,
        "direct_tool_execution": false,
        "outputs_are_hypotheses": true,
        "checkpoint_admission_allowed": false,
        "production_model": false,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_all_writes_eden_capable_gate_without_claims() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!("eden_capable_all_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = run_all();
        let gate = read_json_file(&state_paths::eden_capable_gate_path()).unwrap();

        assert!(out.contains("[EDEN-CAPABLE-GATE]"));
        assert_eq!(gate["claim_allowed"], false);
        assert_eq!(gate["agi_claim"], false);
        assert_eq!(gate["total"], 7);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn cognitive_dataset_manifest_counts_repo_seed_records() {
        let manifest = cognitive_dataset_manifest_value();

        assert_eq!(manifest["claim_allowed"], false);
        assert!(manifest["dataset"]["records"].as_u64().unwrap_or(0) > 0);
        assert_eq!(
            manifest["dataset"]["records"],
            manifest["dataset"]["valid_records"]
        );
    }

    #[test]
    fn structured_packets_preserve_model_as_hypothesis() {
        let inference = serde_json::json!({
            "responses": [
                {
                    "prompt": "EDEN state:",
                    "generated_text": "candidate output"
                }
            ]
        });

        let packets = structured_packets_from_inference(Some(&inference));

        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0]["authority"]["accepted_as_truth"], false);
        assert_eq!(
            packets[0]["candidate_structure"]["requires_verification"],
            true
        );
    }

    #[test]
    fn operationalization_writes_seven_step_gate() {
        let _guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_capable_operational_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = run_operationalization_all();
        let gate = read_json_file(&state_paths::eden_capable_operational_gate_path()).unwrap();

        assert!(out.contains("[EDEN-CAPABLE-OPERATIONAL-GATE]"));
        assert_eq!(gate["claim_allowed"], false);
        assert_eq!(gate["agi_claim"], false);
        assert_eq!(gate["total"], 7);
        assert_eq!(gate["eden_capable_operational_surface_ready"], true);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn memory_action_loop_blocks_model_side_effects() {
        let loop_record = memory_action_loop_value();

        assert_eq!(loop_record["admission"]["memory_fact_write"], "blocked");
        assert_eq!(loop_record["admission"]["tool_execution"], "blocked");
        assert_eq!(loop_record["admission"]["draft_plan"], "allowed");
    }
}
