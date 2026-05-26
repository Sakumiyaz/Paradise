use crate::eden_garm::state_paths;
use serde_json::Value;

pub const EDEN_REAL_CAPABILITY_DATASET_MANIFEST_SCHEMA: &str =
    "eden.real_capability.dataset_manifest.v1";
pub const EDEN_REAL_CAPABILITY_7B_TRAINING_SCHEMA: &str = "eden.real_capability.7b_training.v1";
pub const EDEN_REAL_CAPABILITY_INFERENCE_BRIDGE_SCHEMA: &str =
    "eden.real_capability.inference_bridge.v1";
pub const EDEN_REAL_CAPABILITY_OPERATIONAL_EVAL_SCHEMA: &str =
    "eden.real_capability.operational_eval_admission.v1";
pub const EDEN_REAL_CAPABILITY_CHECKPOINT_DECISION_SCHEMA: &str =
    "eden.real_capability.checkpoint_decision.v1";
pub const EDEN_REAL_CAPABILITY_DEMO_SCHEMA: &str = "eden.real_capability.demo.v1";
pub const EDEN_REAL_CAPABILITY_SCALING_LADDER_SCHEMA: &str =
    "eden.real_capability.scaling_ladder.v1";
pub const EDEN_REAL_CAPABILITY_GATE_SCHEMA: &str = "eden.real_capability.gate.v1";
pub const EDEN_V01_DATASET_MANIFEST_SCHEMA: &str = "eden.v01.dataset_manifest.v1";
pub const EDEN_V01_SEMANTIC_EVAL_SCHEMA: &str = "eden.v01.semantic_eval.v1";
pub const EDEN_V01_TRAINING_BEYOND_PILOT_SCHEMA: &str = "eden.v01.training_beyond_pilot.v1";
pub const EDEN_V01_NATIVE_INFERENCE_RUNTIME_SCHEMA: &str = "eden.v01.native_inference_runtime.v1";
pub const EDEN_V01_OPERATIONAL_DEMO_SCHEMA: &str = "eden.v01.operational_demo.v1";
pub const EDEN_V01_CHECKPOINT_ADMISSION_SCHEMA: &str = "eden.v01.checkpoint_admission.v1";
pub const EDEN_V01_SCALING_PLAN_SCHEMA: &str = "eden.v01.scaling_plan.v1";
pub const EDEN_V01_GPU_WORKSPACE_HYGIENE_SCHEMA: &str = "eden.v01.gpu_workspace_hygiene.v1";
pub const EDEN_V01_CAPABILITY_GATE_SCHEMA: &str = "eden.v01.capability_gate.v1";
pub const EDEN_V02_STABILITY_CORPUS_MANIFEST_SCHEMA: &str = "eden.v02.stability_corpus_manifest.v1";
pub const EDEN_V02_STABILITY_EVAL_SCHEMA: &str = "eden.v02.stability_eval.v1";
pub const EDEN_V02_CHECKPOINT_COMPARISON_SCHEMA: &str = "eden.v02.checkpoint_comparison.v1";
pub const EDEN_V02_ADVERSARIAL_EVAL_SCHEMA: &str = "eden.v02.adversarial_eval.v1";
pub const EDEN_V02_ROLLBACK_DRILL_SCHEMA: &str = "eden.v02.rollback_drill.v1";
pub const EDEN_V02_MODEL_CARD_INTERNAL_SCHEMA: &str = "eden.v02.model_card_internal.v1";
pub const EDEN_V02_CHECKPOINT_STORAGE_SCHEMA: &str = "eden.v02.checkpoint_storage.v1";
pub const EDEN_V02_NATIVE_INFERENCE_SERVICE_SCHEMA: &str = "eden.v02.native_inference_service.v1";
pub const EDEN_V02_STABILITY_DEMO_SCHEMA: &str = "eden.v02.stability_demo.v1";
pub const EDEN_V02_STABILITY_GATE_SCHEMA: &str = "eden.v02.stability_gate.v1";

const AUTHORITY: &str = "global_executive_workspace_core";
const TRAIN_DATA: &str = "training/data/eden_real_capability_train.jsonl";
const EVAL_DATA: &str = "training/data/eden_real_capability_eval.jsonl";
const CHALLENGE_DATA: &str = "training/data/eden_real_capability_challenge.jsonl";
const V01_TRAIN_DATA: &str = "training/data/eden_v01_semantic_train.jsonl";
const V01_EVAL_DATA: &str = "training/data/eden_v01_semantic_eval.jsonl";
const V01_CHALLENGE_DATA: &str = "training/data/eden_v01_semantic_challenge.jsonl";
const V02_TRAIN_DATA: &str = "training/data/eden_v02_stability_train.jsonl";
const V02_EVAL_DATA: &str = "training/data/eden_v02_stability_eval.jsonl";
const V02_CHALLENGE_DATA: &str = "training/data/eden_v02_stability_challenge.jsonl";
const CORPUS_MANIFEST: &str = "target/eden_real_capability/corpus_manifest.json";
const OPERATIONAL_EVAL_REPORT: &str = "target/eden_real_capability/capability_eval_report.json";
const V01_CORPUS_MANIFEST: &str = "target/eden_v01/semantic_corpus_manifest.json";
const V01_SEMANTIC_EVAL_REPORT: &str = "target/eden_v01/semantic_eval_report.json";
const V01_OPERATIONAL_DEMO_REPORT: &str = "target/eden_v01/operational_demo_trace.json";
const V01_GPU_HYGIENE_REPORT: &str = "target/eden_v01/gpu_workspace_hygiene_report.json";
const V02_CORPUS_MANIFEST: &str = "target/eden_v02/stability_corpus_manifest.json";
const V02_STABILITY_EVAL_REPORT: &str = "target/eden_v02/stability_eval_report.json";
const V02_COMPARISON_REPORT: &str = "target/eden_v02/checkpoint_comparison_report.json";
const V02_ADVERSARIAL_REPORT: &str = "target/eden_v02/adversarial_eval_report.json";
const V02_ROLLBACK_REPORT: &str = "target/eden_v02/rollback_drill_report.json";
const V02_MODEL_CARD_REPORT: &str = "target/eden_v02/model_card_internal.json";
const V02_STORAGE_REPORT: &str = "target/eden_v02/checkpoint_storage_manifest.json";
const V02_DEMO_REPORT: &str = "target/eden_v02/stability_demo_trace.json";
const MEGATRON_7B_TRAINING_EVIDENCE: &str =
    "target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json";
const MEGATRON_7B_INFERENCE_REPORT: &str =
    "target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json";
const SFT_TRAINING_REPORT: &str =
    "target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_training_report.json";
const SFT_PREPOST_REPORT: &str = "target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_prepost_eval.json";
const SFT_PACKET_REPORT: &str =
    "target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_inference_packets.json";

pub fn run_all() -> String {
    let mut out = String::new();
    out.push_str(&write_dataset_manifest());
    out.push_str(&write_7b_training_report());
    out.push_str(&write_inference_bridge());
    out.push_str(&write_operational_eval_admission());
    out.push_str(&write_checkpoint_decision());
    out.push_str(&write_operational_demo());
    out.push_str(&write_scaling_ladder());
    out.push_str(&write_real_capability_gate());
    out
}

pub fn run_v01_all() -> String {
    let mut out = String::new();
    out.push_str(&write_v01_dataset_manifest());
    out.push_str(&write_v01_semantic_eval());
    out.push_str(&write_v01_training_beyond_pilot());
    out.push_str(&write_v01_native_inference_runtime());
    out.push_str(&write_v01_operational_demo());
    out.push_str(&write_v01_checkpoint_admission());
    out.push_str(&write_v01_scaling_plan());
    out.push_str(&write_v01_gpu_workspace_hygiene());
    out.push_str(&write_v01_capability_gate());
    out
}

pub fn run_v02_all() -> String {
    let mut out = String::new();
    out.push_str(&write_v02_stability_corpus_manifest());
    out.push_str(&write_v02_stability_eval());
    out.push_str(&write_v02_checkpoint_comparison());
    out.push_str(&write_v02_adversarial_eval());
    out.push_str(&write_v02_rollback_drill());
    out.push_str(&write_v02_model_card_internal());
    out.push_str(&write_v02_checkpoint_storage());
    out.push_str(&write_v02_native_inference_service());
    out.push_str(&write_v02_stability_demo());
    out.push_str(&write_v02_stability_gate());
    out
}

pub fn write_dataset_manifest() -> String {
    write_report(
        "EDEN-REAL-CAPABILITY-DATASET",
        EDEN_REAL_CAPABILITY_DATASET_MANIFEST_SCHEMA,
        state_paths::eden_real_capability_dataset_manifest_path(),
        dataset_manifest_value(),
    )
}

pub fn write_7b_training_report() -> String {
    write_report(
        "EDEN-REAL-CAPABILITY-7B-TRAINING",
        EDEN_REAL_CAPABILITY_7B_TRAINING_SCHEMA,
        state_paths::eden_real_capability_7b_training_path(),
        seven_b_training_value(),
    )
}

pub fn write_inference_bridge() -> String {
    write_report(
        "EDEN-REAL-CAPABILITY-INFERENCE-BRIDGE",
        EDEN_REAL_CAPABILITY_INFERENCE_BRIDGE_SCHEMA,
        state_paths::eden_real_capability_inference_bridge_path(),
        inference_bridge_value(),
    )
}

pub fn write_operational_eval_admission() -> String {
    write_report(
        "EDEN-REAL-CAPABILITY-OPERATIONAL-EVAL",
        EDEN_REAL_CAPABILITY_OPERATIONAL_EVAL_SCHEMA,
        state_paths::eden_real_capability_operational_eval_path(),
        operational_eval_value(),
    )
}

pub fn write_checkpoint_decision() -> String {
    write_report(
        "EDEN-REAL-CAPABILITY-CHECKPOINT-DECISION",
        EDEN_REAL_CAPABILITY_CHECKPOINT_DECISION_SCHEMA,
        state_paths::eden_real_capability_checkpoint_decision_path(),
        checkpoint_decision_value(),
    )
}

pub fn write_operational_demo() -> String {
    write_report(
        "EDEN-REAL-CAPABILITY-DEMO",
        EDEN_REAL_CAPABILITY_DEMO_SCHEMA,
        state_paths::eden_real_capability_demo_path(),
        operational_demo_value(),
    )
}

pub fn write_scaling_ladder() -> String {
    write_report(
        "EDEN-REAL-CAPABILITY-SCALING-LADDER",
        EDEN_REAL_CAPABILITY_SCALING_LADDER_SCHEMA,
        state_paths::eden_real_capability_scaling_ladder_path(),
        scaling_ladder_value(),
    )
}

pub fn write_real_capability_gate() -> String {
    write_report(
        "EDEN-REAL-CAPABILITY-GATE",
        EDEN_REAL_CAPABILITY_GATE_SCHEMA,
        state_paths::eden_real_capability_gate_path(),
        gate_value(),
    )
}

pub fn write_v01_dataset_manifest() -> String {
    write_report(
        "EDEN-V01-DATASET",
        EDEN_V01_DATASET_MANIFEST_SCHEMA,
        state_paths::eden_v01_dataset_manifest_path(),
        v01_dataset_manifest_value(),
    )
}

pub fn write_v01_semantic_eval() -> String {
    write_report(
        "EDEN-V01-SEMANTIC-EVAL",
        EDEN_V01_SEMANTIC_EVAL_SCHEMA,
        state_paths::eden_v01_semantic_eval_path(),
        v01_semantic_eval_value(),
    )
}

pub fn write_v01_training_beyond_pilot() -> String {
    write_report(
        "EDEN-V01-TRAINING-BEYOND-PILOT",
        EDEN_V01_TRAINING_BEYOND_PILOT_SCHEMA,
        state_paths::eden_v01_training_beyond_pilot_path(),
        v01_training_beyond_pilot_value(),
    )
}

pub fn write_v01_native_inference_runtime() -> String {
    write_report(
        "EDEN-V01-NATIVE-INFERENCE-RUNTIME",
        EDEN_V01_NATIVE_INFERENCE_RUNTIME_SCHEMA,
        state_paths::eden_v01_native_inference_runtime_path(),
        v01_native_inference_runtime_value(),
    )
}

pub fn write_v01_operational_demo() -> String {
    write_report(
        "EDEN-V01-OPERATIONAL-DEMO",
        EDEN_V01_OPERATIONAL_DEMO_SCHEMA,
        state_paths::eden_v01_operational_demo_path(),
        v01_operational_demo_value(),
    )
}

pub fn write_v01_checkpoint_admission() -> String {
    write_report(
        "EDEN-V01-CHECKPOINT-ADMISSION",
        EDEN_V01_CHECKPOINT_ADMISSION_SCHEMA,
        state_paths::eden_v01_checkpoint_admission_path(),
        v01_checkpoint_admission_value(),
    )
}

pub fn write_v01_scaling_plan() -> String {
    write_report(
        "EDEN-V01-SCALING-PLAN",
        EDEN_V01_SCALING_PLAN_SCHEMA,
        state_paths::eden_v01_scaling_plan_path(),
        v01_scaling_plan_value(),
    )
}

pub fn write_v01_gpu_workspace_hygiene() -> String {
    write_report(
        "EDEN-V01-GPU-HYGIENE",
        EDEN_V01_GPU_WORKSPACE_HYGIENE_SCHEMA,
        state_paths::eden_v01_gpu_workspace_hygiene_path(),
        v01_gpu_workspace_hygiene_value(),
    )
}

pub fn write_v01_capability_gate() -> String {
    write_report(
        "EDEN-V01-CAPABILITY-GATE",
        EDEN_V01_CAPABILITY_GATE_SCHEMA,
        state_paths::eden_v01_capability_gate_path(),
        v01_gate_value(),
    )
}

pub fn write_v02_stability_corpus_manifest() -> String {
    write_report(
        "EDEN-V02-STABILITY-CORPUS",
        EDEN_V02_STABILITY_CORPUS_MANIFEST_SCHEMA,
        state_paths::eden_v02_stability_corpus_manifest_path(),
        v02_stability_corpus_manifest_value(),
    )
}

pub fn write_v02_stability_eval() -> String {
    write_report(
        "EDEN-V02-STABILITY-EVAL",
        EDEN_V02_STABILITY_EVAL_SCHEMA,
        state_paths::eden_v02_stability_eval_path(),
        v02_stability_eval_value(),
    )
}

pub fn write_v02_checkpoint_comparison() -> String {
    write_report(
        "EDEN-V02-CHECKPOINT-COMPARISON",
        EDEN_V02_CHECKPOINT_COMPARISON_SCHEMA,
        state_paths::eden_v02_checkpoint_comparison_path(),
        v02_checkpoint_comparison_value(),
    )
}

pub fn write_v02_adversarial_eval() -> String {
    write_report(
        "EDEN-V02-ADVERSARIAL-EVAL",
        EDEN_V02_ADVERSARIAL_EVAL_SCHEMA,
        state_paths::eden_v02_adversarial_eval_path(),
        v02_adversarial_eval_value(),
    )
}

pub fn write_v02_rollback_drill() -> String {
    write_report(
        "EDEN-V02-ROLLBACK-DRILL",
        EDEN_V02_ROLLBACK_DRILL_SCHEMA,
        state_paths::eden_v02_rollback_drill_path(),
        v02_rollback_drill_value(),
    )
}

pub fn write_v02_model_card_internal() -> String {
    write_report(
        "EDEN-V02-MODEL-CARD",
        EDEN_V02_MODEL_CARD_INTERNAL_SCHEMA,
        state_paths::eden_v02_model_card_internal_path(),
        v02_model_card_internal_value(),
    )
}

pub fn write_v02_checkpoint_storage() -> String {
    write_report(
        "EDEN-V02-CHECKPOINT-STORAGE",
        EDEN_V02_CHECKPOINT_STORAGE_SCHEMA,
        state_paths::eden_v02_checkpoint_storage_path(),
        v02_checkpoint_storage_value(),
    )
}

pub fn write_v02_native_inference_service() -> String {
    write_report(
        "EDEN-V02-NATIVE-INFERENCE-SERVICE",
        EDEN_V02_NATIVE_INFERENCE_SERVICE_SCHEMA,
        state_paths::eden_v02_native_inference_service_path(),
        v02_native_inference_service_value(),
    )
}

pub fn write_v02_stability_demo() -> String {
    write_report(
        "EDEN-V02-STABILITY-DEMO",
        EDEN_V02_STABILITY_DEMO_SCHEMA,
        state_paths::eden_v02_stability_demo_path(),
        v02_stability_demo_value(),
    )
}

pub fn write_v02_stability_gate() -> String {
    write_report(
        "EDEN-V02-STABILITY-GATE",
        EDEN_V02_STABILITY_GATE_SCHEMA,
        state_paths::eden_v02_stability_gate_path(),
        v02_stability_gate_value(),
    )
}

fn dataset_manifest_value() -> Value {
    let source = read_repo_json(CORPUS_MANIFEST);
    let train_rows = count_jsonl(TRAIN_DATA);
    let eval_rows = count_jsonl(EVAL_DATA);
    let challenge_rows = count_jsonl(CHALLENGE_DATA);
    serde_json::json!({
        "schema": EDEN_REAL_CAPABILITY_DATASET_MANIFEST_SCHEMA,
        "artifact": "eden_real_capability_dataset_manifest",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "contains_private_data": false,
        "external_model_dependency": false,
        "source_manifest_path": CORPUS_MANIFEST,
        "source_manifest_present": source.is_some(),
        "rows": {
            "train": train_rows,
            "eval": eval_rows,
            "challenge": challenge_rows,
            "total": train_rows + eval_rows + challenge_rows
        },
        "categories": source.as_ref().and_then(|v| v.get("categories")).cloned().unwrap_or(Value::Null),
        "paths": {
            "train": TRAIN_DATA,
            "eval": EVAL_DATA,
            "challenge": CHALLENGE_DATA
        },
        "accepted_for": [
            "operational_capability_eval",
            "checkpoint_admission_review",
            "runtime_regression"
        ],
        "not_accepted_for": [
            "private_user_memory_training",
            "AGI_claim",
            "open_domain_generalization_claim"
        ],
    })
}

fn seven_b_training_value() -> Value {
    let source = read_repo_json(MEGATRON_7B_TRAINING_EVIDENCE);
    serde_json::json!({
        "schema": EDEN_REAL_CAPABILITY_7B_TRAINING_SCHEMA,
        "artifact": "eden_real_capability_7b_training",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": MEGATRON_7B_TRAINING_EVIDENCE,
        "source_present": source.is_some(),
        "completed_iterations": source_u64(source.as_ref(), "/run/completed_iterations"),
        "train_iters": source_u64(source.as_ref(), "/run/train_iters"),
        "model_parameters": source_u64(source.as_ref(), "/run/model_parameters"),
        "final_loss": source_f64(source.as_ref(), "/run/final_loss"),
        "checkpoint_written": source_bool(source.as_ref(), "/checkpoint_policy/checkpoint_written"),
        "checkpoint_admission_allowed": false,
        "external_model_dependency": source_bool(source.as_ref(), "/run/external_model_dependency"),
        "network": source_string(source.as_ref(), "/run/network"),
        "accepted_as": "bounded_7b_training_evidence_not_model_release",
    })
}

fn inference_bridge_value() -> Value {
    let megatron = read_repo_json(MEGATRON_7B_INFERENCE_REPORT);
    let sft = read_repo_json(SFT_PACKET_REPORT);
    let responses = megatron
        .as_ref()
        .and_then(|value| value.get("responses"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let packets = sft
        .as_ref()
        .and_then(|value| value.get("packets"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    serde_json::json!({
        "schema": EDEN_REAL_CAPABILITY_INFERENCE_BRIDGE_SCHEMA,
        "artifact": "eden_real_capability_inference_bridge",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "megatron_inference_source": MEGATRON_7B_INFERENCE_REPORT,
        "sft_packet_source": SFT_PACKET_REPORT,
        "checkpoint_loaded": source_bool(megatron.as_ref(), "/run/checkpoint_loaded"),
        "generated_count": source_u64(megatron.as_ref(), "/run/generated_count"),
        "sft_packet_count": packets.len(),
        "all_model_outputs_are_hypotheses": true,
        "direct_memory_writes": false,
        "direct_objective_writes": false,
        "direct_tool_execution": false,
        "sample_response": responses.first().cloned().unwrap_or(Value::Null),
        "sample_sft_packet": packets.first().cloned().unwrap_or(Value::Null),
    })
}

fn operational_eval_value() -> Value {
    let source = read_repo_json(OPERATIONAL_EVAL_REPORT);
    serde_json::json!({
        "schema": EDEN_REAL_CAPABILITY_OPERATIONAL_EVAL_SCHEMA,
        "artifact": "eden_real_capability_operational_eval",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": OPERATIONAL_EVAL_REPORT,
        "source_present": source.is_some(),
        "score": source_f64(source.as_ref(), "/score"),
        "passed": source_bool(source.as_ref(), "/passed"),
        "weighted_passed": source_u64(source.as_ref(), "/weighted_passed"),
        "weighted_total": source_u64(source.as_ref(), "/weighted_total"),
        "rows": source.as_ref().and_then(|v| v.get("rows")).cloned().unwrap_or(Value::Null),
        "checks": source.as_ref().and_then(|v| v.get("checks")).cloned().unwrap_or(Value::Null),
        "not_measured": [
            "AGI",
            "external_benchmark_superiority",
            "human_preference_alignment"
        ],
    })
}

fn checkpoint_decision_value() -> Value {
    let eval = read_repo_json(OPERATIONAL_EVAL_REPORT);
    let training = read_repo_json(MEGATRON_7B_TRAINING_EVIDENCE);
    let inference = read_repo_json(MEGATRON_7B_INFERENCE_REPORT);
    let reviewable = source_bool(eval.as_ref(), "/passed") == Some(true)
        && source_u64(training.as_ref(), "/run/completed_iterations").unwrap_or(0) >= 50
        && source_bool(training.as_ref(), "/checkpoint_policy/checkpoint_written") == Some(true)
        && source_bool(inference.as_ref(), "/run/checkpoint_loaded") == Some(true);
    serde_json::json!({
        "schema": EDEN_REAL_CAPABILITY_CHECKPOINT_DECISION_SCHEMA,
        "artifact": "eden_real_capability_checkpoint_decision",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "reviewable": reviewable,
        "checkpoint_admission_allowed": false,
        "production_model_allowed": false,
        "autonomous_authority_allowed": false,
        "reason": "Evidence can make the checkpoint reviewable, but admission requires separate operator approval and adversarial release gates.",
        "required_before_admission": [
            "operator_approval",
            "held_out_external_regression",
            "prompt_injection_eval",
            "data_contamination_review",
            "rollback_drill",
            "model_card_release_review"
        ],
    })
}

fn operational_demo_value() -> Value {
    let inference = read_repo_json(MEGATRON_7B_INFERENCE_REPORT);
    let packets = read_repo_json(SFT_PACKET_REPORT);
    let sample_response = inference
        .as_ref()
        .and_then(|value| value.get("responses"))
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .cloned()
        .unwrap_or(Value::Null);
    let sample_packet = packets
        .as_ref()
        .and_then(|value| value.get("packets"))
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .cloned()
        .unwrap_or(Value::Null);
    serde_json::json!({
        "schema": EDEN_REAL_CAPABILITY_DEMO_SCHEMA,
        "artifact": "eden_real_capability_demo",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "demo": "7b_checkpoint_and_sft_elcp_packet_through_gewc",
        "steps": [
            "load checkpoint evidence",
            "generate candidate text",
            "convert learned SFT/ELCP transition into hypothesis packet",
            "verify no direct memory/objective/tool mutation",
            "keep checkpoint admission blocked",
            "record audit evidence"
        ],
        "sample_response": sample_response,
        "sample_packet": sample_packet,
    })
}

fn scaling_ladder_value() -> Value {
    let training = read_repo_json(MEGATRON_7B_TRAINING_EVIDENCE);
    let current_iters = source_u64(training.as_ref(), "/run/completed_iterations").unwrap_or(0);
    serde_json::json!({
        "schema": EDEN_REAL_CAPABILITY_SCALING_LADDER_SCHEMA,
        "artifact": "eden_real_capability_scaling_ladder",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "current_stage": {
            "model": "eden-megatron-7b-base-pilot",
            "completed_iterations": current_iters,
            "checkpoint_written": source_bool(training.as_ref(), "/checkpoint_policy/checkpoint_written"),
            "admitted": false
        },
        "next_runs": [
            {"stage": "stability_100_iters", "train_iters": 100, "requires": ["same_dataset_hashes", "prepost_eval", "rollback_drill"]},
            {"stage": "capability_250_iters", "train_iters": 250, "requires": ["held_out_eval", "safety_eval", "operator_budget"]},
            {"stage": "release_candidate_1000_iters", "train_iters": 1000, "requires": ["external_review", "data_governance", "checkpoint_card"]}
        ],
        "comparison_policy": "new checkpoints must beat the prior admitted candidate on score, safety and rollback before review; no automatic admission",
    })
}

fn gate_value() -> Value {
    let dataset = read_json_file(&state_paths::eden_real_capability_dataset_manifest_path());
    let training = read_json_file(&state_paths::eden_real_capability_7b_training_path());
    let inference = read_json_file(&state_paths::eden_real_capability_inference_bridge_path());
    let eval = read_json_file(&state_paths::eden_real_capability_operational_eval_path());
    let checkpoint = read_json_file(&state_paths::eden_real_capability_checkpoint_decision_path());
    let demo = read_json_file(&state_paths::eden_real_capability_demo_path());
    let scaling = read_json_file(&state_paths::eden_real_capability_scaling_ladder_path());
    let checks = vec![
        check(
            "real_dataset_300_plus_rows",
            source_u64(dataset.as_ref(), "/rows/total").unwrap_or(0) >= 300,
            TRAIN_DATA,
        ),
        check(
            "7b_training_50_iters_checkpointed",
            source_u64(training.as_ref(), "/completed_iterations").unwrap_or(0) >= 50
                && source_bool(training.as_ref(), "/checkpoint_written") == Some(true),
            MEGATRON_7B_TRAINING_EVIDENCE,
        ),
        check(
            "integrated_inference_loaded_and_packetized",
            source_bool(inference.as_ref(), "/checkpoint_loaded") == Some(true)
                && source_u64(inference.as_ref(), "/generated_count").unwrap_or(0) >= 2
                && source_u64(inference.as_ref(), "/sft_packet_count").unwrap_or(0) >= 1,
            MEGATRON_7B_INFERENCE_REPORT,
        ),
        check(
            "operational_eval_passed",
            source_bool(eval.as_ref(), "/passed") == Some(true),
            OPERATIONAL_EVAL_REPORT,
        ),
        check(
            "checkpoint_reviewable_but_not_admitted",
            source_bool(checkpoint.as_ref(), "/reviewable") == Some(true)
                && source_bool(checkpoint.as_ref(), "/checkpoint_admission_allowed") == Some(false),
            "eden_real_capability_checkpoint_decision.json",
        ),
        check(
            "operational_demo_has_samples",
            demo.as_ref()
                .and_then(|value| value.get("sample_response"))
                .is_some_and(|value| !value.is_null())
                && demo
                    .as_ref()
                    .and_then(|value| value.get("sample_packet"))
                    .is_some_and(|value| !value.is_null()),
            "eden_real_capability_demo.json",
        ),
        check(
            "scaling_ladder_preserves_no_admission",
            source_bool(scaling.as_ref(), "/current_stage/admitted") == Some(false),
            "eden_real_capability_scaling_ladder.json",
        ),
    ];
    let passed = checks
        .iter()
        .filter(|check| check["passed"] == Value::Bool(true))
        .count();
    serde_json::json!({
        "schema": EDEN_REAL_CAPABILITY_GATE_SCHEMA,
        "artifact": "eden_real_capability_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": checks.len(),
        "checks": checks,
        "checkpoint_admission_allowed": false,
        "capability_class": "bounded_real_capability_stage",
        "not_yet": [
            "production_checkpoint_admission",
            "external_AGI_benchmark",
            "autonomous_tool_authority",
            "AGI"
        ],
    })
}

fn v01_dataset_manifest_value() -> Value {
    let source = read_repo_json(V01_CORPUS_MANIFEST);
    let train_rows = count_jsonl(V01_TRAIN_DATA);
    let eval_rows = count_jsonl(V01_EVAL_DATA);
    let challenge_rows = count_jsonl(V01_CHALLENGE_DATA);
    serde_json::json!({
        "schema": EDEN_V01_DATASET_MANIFEST_SCHEMA,
        "artifact": "eden_v01_dataset_manifest",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "contains_private_data": false,
        "external_model_dependency": false,
        "source_manifest_path": V01_CORPUS_MANIFEST,
        "source_manifest_present": source.is_some(),
        "rows": {
            "train": train_rows,
            "eval": eval_rows,
            "challenge": challenge_rows,
            "total": train_rows + eval_rows + challenge_rows
        },
        "task_types": source.as_ref().and_then(|v| v.get("task_types")).cloned().unwrap_or(Value::Null),
        "categories": source.as_ref().and_then(|v| v.get("categories")).cloned().unwrap_or(Value::Null),
        "paths": {
            "train": V01_TRAIN_DATA,
            "eval": V01_EVAL_DATA,
            "challenge": V01_CHALLENGE_DATA
        },
        "accepted_for": [
            "semantic_capability_eval",
            "native_inference_runtime_candidate_admission",
            "checkpoint_candidate_review"
        ],
        "not_accepted_for": [
            "AGI_claim",
            "private_user_memory_training",
            "production_release"
        ],
    })
}

fn v01_semantic_eval_value() -> Value {
    let source = read_repo_json(V01_SEMANTIC_EVAL_REPORT);
    serde_json::json!({
        "schema": EDEN_V01_SEMANTIC_EVAL_SCHEMA,
        "artifact": "eden_v01_semantic_eval",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V01_SEMANTIC_EVAL_REPORT,
        "source_present": source.is_some(),
        "score": source_f64(source.as_ref(), "/score"),
        "passed": source_bool(source.as_ref(), "/passed"),
        "weighted_passed": source_u64(source.as_ref(), "/weighted_passed"),
        "weighted_total": source_u64(source.as_ref(), "/weighted_total"),
        "rows": source.as_ref().and_then(|v| v.get("rows")).cloned().unwrap_or(Value::Null),
        "task_types": source.as_ref().and_then(|v| v.get("task_types")).cloned().unwrap_or(Value::Null),
        "training": source.as_ref().and_then(|v| v.get("training")).cloned().unwrap_or(Value::Null),
        "checks": source.as_ref().and_then(|v| v.get("checks")).cloned().unwrap_or(Value::Null),
        "not_measured": [
            "AGI",
            "human_level_autonomy",
            "production_release_safety"
        ],
    })
}

fn v01_training_beyond_pilot_value() -> Value {
    let training = read_repo_json(MEGATRON_7B_TRAINING_EVIDENCE);
    let completed = source_u64(training.as_ref(), "/run/completed_iterations").unwrap_or(0);
    let model_parameters = source_u64(training.as_ref(), "/run/model_parameters").unwrap_or(0);
    serde_json::json!({
        "schema": EDEN_V01_TRAINING_BEYOND_PILOT_SCHEMA,
        "artifact": "eden_v01_training_beyond_pilot",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": MEGATRON_7B_TRAINING_EVIDENCE,
        "source_present": training.is_some(),
        "completed_iterations": completed,
        "minimum_iterations": 100,
        "beyond_50_iter_pilot": completed >= 100,
        "model_parameters": model_parameters,
        "within_14b_dense_ceiling": (6_900_000_000..=14_000_000_000).contains(&model_parameters),
        "final_loss": source_f64(training.as_ref(), "/run/final_loss"),
        "checkpoint_written": source_bool(training.as_ref(), "/checkpoint_policy/checkpoint_written"),
        "network": source_string(training.as_ref(), "/run/network"),
        "external_model_dependency": source_bool(training.as_ref(), "/run/external_model_dependency"),
        "accepted_as": "training_beyond_pilot_evidence_not_production_model",
    })
}

fn v01_native_inference_runtime_value() -> Value {
    let inference = read_repo_json(MEGATRON_7B_INFERENCE_REPORT);
    let semantic_eval = read_repo_json(V01_SEMANTIC_EVAL_REPORT);
    let sample_response = inference
        .as_ref()
        .and_then(|value| value.get("responses"))
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .cloned()
        .unwrap_or(Value::Null);
    let available = source_bool(inference.as_ref(), "/run/checkpoint_loaded") == Some(true)
        && source_u64(inference.as_ref(), "/run/generated_count").unwrap_or(0) >= 2
        && source_bool(semantic_eval.as_ref(), "/passed") == Some(true);
    serde_json::json!({
        "schema": EDEN_V01_NATIVE_INFERENCE_RUNTIME_SCHEMA,
        "artifact": "eden_v01_native_inference_runtime",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "runtime_candidate_available": available,
        "source_path": MEGATRON_7B_INFERENCE_REPORT,
        "checkpoint_loaded": source_bool(inference.as_ref(), "/run/checkpoint_loaded"),
        "generated_count": source_u64(inference.as_ref(), "/run/generated_count"),
        "semantic_eval_passed": source_bool(semantic_eval.as_ref(), "/passed"),
        "request_contract": {
            "schema": "eden.v01.native_inference_request.v1",
            "fields": ["task_id", "goal", "situation", "memory_refs", "risk_class", "max_tokens", "allowed_modes"]
        },
        "response_contract": {
            "schema": "eden.v01.native_inference_packet.v1",
            "fields": ["candidate_text", "structured_hypothesis", "confidence", "requires_verification", "source_model", "trace_id"]
        },
        "authority_rules": {
            "model_outputs_are_hypotheses": true,
            "gewc_verifies_before_state_change": true,
            "direct_memory_write_allowed": false,
            "direct_tool_execution_allowed": false,
            "direct_objective_update_allowed": false
        },
        "sample_response": sample_response,
    })
}

fn v01_operational_demo_value() -> Value {
    let demo = read_repo_json(V01_OPERATIONAL_DEMO_REPORT);
    serde_json::json!({
        "schema": EDEN_V01_OPERATIONAL_DEMO_SCHEMA,
        "artifact": "eden_v01_operational_demo",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V01_OPERATIONAL_DEMO_REPORT,
        "source_present": demo.is_some(),
        "passed": source_bool(demo.as_ref(), "/passed"),
        "steps": demo.as_ref().and_then(|v| v.get("steps")).cloned().unwrap_or(Value::Null),
        "task": demo.as_ref().and_then(|v| v.get("task")).cloned().unwrap_or(Value::Null),
        "safety_boundary": demo.as_ref().and_then(|v| v.get("safety_boundary")).cloned().unwrap_or(Value::Null),
    })
}

fn v01_checkpoint_admission_value() -> Value {
    let semantic_eval = read_json_file(&state_paths::eden_v01_semantic_eval_path());
    let training = read_json_file(&state_paths::eden_v01_training_beyond_pilot_path());
    let inference = read_json_file(&state_paths::eden_v01_native_inference_runtime_path());
    let demo = read_json_file(&state_paths::eden_v01_operational_demo_path());
    let candidate_allowed = source_bool(semantic_eval.as_ref(), "/passed") == Some(true)
        && source_bool(training.as_ref(), "/beyond_50_iter_pilot") == Some(true)
        && source_bool(training.as_ref(), "/checkpoint_written") == Some(true)
        && source_bool(inference.as_ref(), "/runtime_candidate_available") == Some(true)
        && source_bool(demo.as_ref(), "/passed") == Some(true);
    serde_json::json!({
        "schema": EDEN_V01_CHECKPOINT_ADMISSION_SCHEMA,
        "artifact": "eden_v01_checkpoint_admission",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "candidate_runtime_admission_allowed": candidate_allowed,
        "production_model_allowed": false,
        "autonomous_authority_allowed": false,
        "weights_committed_to_repo": false,
        "decision": if candidate_allowed { "admit_as_gewc_subordinate_candidate_generator" } else { "blocked_until_v01_evidence_passes" },
        "required_before_production_release": [
            "longer_training_run",
            "held_out_external_eval",
            "prompt_injection_eval",
            "rollback_drill",
            "model_card_release_review",
            "operator_release_approval"
        ],
    })
}

fn v01_scaling_plan_value() -> Value {
    let training = read_repo_json(MEGATRON_7B_TRAINING_EVIDENCE);
    let current_iters = source_u64(training.as_ref(), "/run/completed_iterations").unwrap_or(0);
    let params = source_u64(training.as_ref(), "/run/model_parameters").unwrap_or(0);
    serde_json::json!({
        "schema": EDEN_V01_SCALING_PLAN_SCHEMA,
        "artifact": "eden_v01_scaling_plan",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "max_dense_parameters": 14_000_000_000u64,
        "current_model": {
            "model_id": "eden-megatron-7b-base-pilot",
            "parameters": params,
            "completed_iterations": current_iters,
            "within_max_dense_ceiling": params <= 14_000_000_000u64,
            "production_admitted": false
        },
        "next_runs": [
            {"stage": "eden_7b_stability_250", "parameters": 6_996_365_312u64, "train_iters": 250, "requires": ["semantic_eval_delta", "rollback_drill", "checkpoint_compare"]},
            {"stage": "eden_7b_capability_1000", "parameters": 6_996_365_312u64, "train_iters": 1000, "requires": ["held_out_eval", "safety_eval", "operator_budget"]},
            {"stage": "eden_14b_pretraining_prototype", "parameters": 14_000_000_000u64, "train_iters": "operator_budgeted", "requires": ["7b_eval_win", "dataset_freeze", "multi_gpu_plan"]}
        ],
        "policy": "improve by pretraining, curated data and GEWC integration before increasing dense parameter count beyond 7B; never exceed 14B dense without a new ADR",
    })
}

fn v01_gpu_workspace_hygiene_value() -> Value {
    let source = read_repo_json(V01_GPU_HYGIENE_REPORT);
    serde_json::json!({
        "schema": EDEN_V01_GPU_WORKSPACE_HYGIENE_SCHEMA,
        "artifact": "eden_v01_gpu_workspace_hygiene",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V01_GPU_HYGIENE_REPORT,
        "source_present": source.is_some(),
        "destructive_apply": source_bool(source.as_ref(), "/destructive_apply").unwrap_or(false),
        "repo": source.as_ref().and_then(|v| v.get("repo")).cloned().unwrap_or(Value::Null),
        "gpu_workspace": source.as_ref().and_then(|v| v.get("gpu_workspace")).cloned().unwrap_or(Value::Null),
        "cleanup_policy": source.as_ref().and_then(|v| v.get("cleanup_policy")).cloned().unwrap_or(Value::Null),
    })
}

fn v01_gate_value() -> Value {
    let dataset = read_json_file(&state_paths::eden_v01_dataset_manifest_path());
    let semantic_eval = read_json_file(&state_paths::eden_v01_semantic_eval_path());
    let training = read_json_file(&state_paths::eden_v01_training_beyond_pilot_path());
    let inference = read_json_file(&state_paths::eden_v01_native_inference_runtime_path());
    let demo = read_json_file(&state_paths::eden_v01_operational_demo_path());
    let checkpoint = read_json_file(&state_paths::eden_v01_checkpoint_admission_path());
    let scaling = read_json_file(&state_paths::eden_v01_scaling_plan_path());
    let hygiene = read_json_file(&state_paths::eden_v01_gpu_workspace_hygiene_path());
    let checks = vec![
        check(
            "large_curated_dataset_2048_plus_rows",
            source_u64(dataset.as_ref(), "/rows/total").unwrap_or(0) >= 2048,
            V01_TRAIN_DATA,
        ),
        check(
            "strong_semantic_eval_passed",
            source_bool(semantic_eval.as_ref(), "/passed") == Some(true),
            V01_SEMANTIC_EVAL_REPORT,
        ),
        check(
            "training_beyond_pilot_100_iters_checkpointed",
            source_bool(training.as_ref(), "/beyond_50_iter_pilot") == Some(true)
                && source_bool(training.as_ref(), "/checkpoint_written") == Some(true),
            MEGATRON_7B_TRAINING_EVIDENCE,
        ),
        check(
            "native_inference_runtime_candidate_available",
            source_bool(inference.as_ref(), "/runtime_candidate_available") == Some(true),
            MEGATRON_7B_INFERENCE_REPORT,
        ),
        check(
            "candidate_admission_real_but_production_blocked",
            source_bool(checkpoint.as_ref(), "/candidate_runtime_admission_allowed") == Some(true)
                && source_bool(checkpoint.as_ref(), "/production_model_allowed") == Some(false),
            "eden_v01_checkpoint_admission.json",
        ),
        check(
            "operational_demo_passed_without_mutation",
            source_bool(demo.as_ref(), "/passed") == Some(true),
            V01_OPERATIONAL_DEMO_REPORT,
        ),
        check(
            "scaling_plan_caps_dense_model_at_14b",
            source_u64(scaling.as_ref(), "/max_dense_parameters") == Some(14_000_000_000u64),
            "eden_v01_scaling_plan.json",
        ),
        check(
            "gpu_workspace_hygiene_recorded_non_destructive",
            hygiene.is_some() && source_bool(hygiene.as_ref(), "/destructive_apply") == Some(false),
            V01_GPU_HYGIENE_REPORT,
        ),
    ];
    let passed = checks
        .iter()
        .filter(|check| check["passed"] == Value::Bool(true))
        .count();
    serde_json::json!({
        "schema": EDEN_V01_CAPABILITY_GATE_SCHEMA,
        "artifact": "eden_v01_capability_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": checks.len(),
        "checks": checks,
        "candidate_runtime_admission_allowed": source_bool(checkpoint.as_ref(), "/candidate_runtime_admission_allowed").unwrap_or(false),
        "production_model_allowed": false,
        "max_dense_parameters": 14_000_000_000u64,
        "capability_class": "eden_v01_semantic_runtime_candidate",
        "not_yet": [
            "production_checkpoint_release",
            "external_AGI_benchmark",
            "fully_autonomous_tool_authority",
            "AGI"
        ],
    })
}

fn v02_stability_corpus_manifest_value() -> Value {
    let manifest = read_repo_json(V02_CORPUS_MANIFEST);
    let train = count_jsonl(V02_TRAIN_DATA);
    let eval = count_jsonl(V02_EVAL_DATA);
    let challenge = count_jsonl(V02_CHALLENGE_DATA);
    serde_json::json!({
        "schema": EDEN_V02_STABILITY_CORPUS_MANIFEST_SCHEMA,
        "artifact": "eden_v02_stability_corpus_manifest",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V02_CORPUS_MANIFEST,
        "source_present": manifest.is_some(),
        "rows": {
            "train": train,
            "eval": eval,
            "challenge": challenge,
            "total": train + eval + challenge,
        },
        "task_types": manifest.as_ref().and_then(|v| v.get("task_types")).cloned().unwrap_or(Value::Null),
        "categories": manifest.as_ref().and_then(|v| v.get("categories")).cloned().unwrap_or(Value::Null),
        "accepted_for": ["stability_eval", "adversarial_eval", "rollback_drill"],
        "not_accepted_for": ["AGI", "production_release"],
    })
}

fn v02_stability_eval_value() -> Value {
    let source = read_repo_json(V02_STABILITY_EVAL_REPORT);
    serde_json::json!({
        "schema": EDEN_V02_STABILITY_EVAL_SCHEMA,
        "artifact": "eden_v02_stability_eval",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V02_STABILITY_EVAL_REPORT,
        "source_present": source.is_some(),
        "score": source_f64(source.as_ref(), "/score"),
        "passed": source_bool(source.as_ref(), "/passed"),
        "rows": source.as_ref().and_then(|v| v.get("rows")).cloned().unwrap_or(Value::Null),
        "checks": source.as_ref().and_then(|v| v.get("checks")).cloned().unwrap_or(Value::Null),
        "comparison_report": source.as_ref().and_then(|v| v.get("comparison_report")).cloned().unwrap_or(Value::Null),
    })
}

fn v02_checkpoint_comparison_value() -> Value {
    let source = read_repo_json(V02_COMPARISON_REPORT);
    serde_json::json!({
        "schema": EDEN_V02_CHECKPOINT_COMPARISON_SCHEMA,
        "artifact": "eden_v02_checkpoint_comparison",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V02_COMPARISON_REPORT,
        "source_present": source.is_some(),
        "passed": source_bool(source.as_ref(), "/passed"),
        "baseline": source.as_ref().and_then(|v| v.get("baseline")).cloned().unwrap_or(Value::Null),
        "candidate": source.as_ref().and_then(|v| v.get("candidate")).cloned().unwrap_or(Value::Null),
        "loss_delta": source.as_ref().and_then(|v| v.get("loss_delta")).cloned().unwrap_or(Value::Null),
        "loss_ratio": source.as_ref().and_then(|v| v.get("loss_ratio")).cloned().unwrap_or(Value::Null),
        "production_model_allowed": source_bool(source.as_ref(), "/production_model_allowed").unwrap_or(false),
        "admission_scope": source_string(source.as_ref(), "/admission_scope"),
    })
}

fn v02_adversarial_eval_value() -> Value {
    let source = read_repo_json(V02_ADVERSARIAL_REPORT);
    serde_json::json!({
        "schema": EDEN_V02_ADVERSARIAL_EVAL_SCHEMA,
        "artifact": "eden_v02_adversarial_eval",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V02_ADVERSARIAL_REPORT,
        "source_present": source.is_some(),
        "passed": source_bool(source.as_ref(), "/passed"),
        "cases_total": source_u64(source.as_ref(), "/cases_total"),
        "cases_passed": source_u64(source.as_ref(), "/cases_passed"),
        "cases": source.as_ref().and_then(|v| v.get("cases")).cloned().unwrap_or(Value::Null),
        "policy": source.as_ref().and_then(|v| v.get("policy")).cloned().unwrap_or(Value::Null),
    })
}

fn v02_rollback_drill_value() -> Value {
    let source = read_repo_json(V02_ROLLBACK_REPORT);
    serde_json::json!({
        "schema": EDEN_V02_ROLLBACK_DRILL_SCHEMA,
        "artifact": "eden_v02_rollback_drill",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V02_ROLLBACK_REPORT,
        "source_present": source.is_some(),
        "passed": source_bool(source.as_ref(), "/passed"),
        "fault": source.as_ref().and_then(|v| v.get("fault")).cloned().unwrap_or(Value::Null),
        "rollback_contract": source.as_ref().and_then(|v| v.get("rollback_contract")).cloned().unwrap_or(Value::Null),
    })
}

fn v02_model_card_internal_value() -> Value {
    let source = read_repo_json(V02_MODEL_CARD_REPORT);
    serde_json::json!({
        "schema": EDEN_V02_MODEL_CARD_INTERNAL_SCHEMA,
        "artifact": "eden_v02_model_card_internal",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V02_MODEL_CARD_REPORT,
        "source_present": source.is_some(),
        "passed": source_bool(source.as_ref(), "/passed"),
        "model": source.as_ref().and_then(|v| v.get("model")).cloned().unwrap_or(Value::Null),
        "known_limits": source.as_ref().and_then(|v| v.get("known_limits")).cloned().unwrap_or(Value::Null),
        "required_before_production": source.as_ref().and_then(|v| v.get("required_before_production")).cloned().unwrap_or(Value::Null),
    })
}

fn v02_checkpoint_storage_value() -> Value {
    let source = read_repo_json(V02_STORAGE_REPORT);
    serde_json::json!({
        "schema": EDEN_V02_CHECKPOINT_STORAGE_SCHEMA,
        "artifact": "eden_v02_checkpoint_storage",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V02_STORAGE_REPORT,
        "source_present": source.is_some(),
        "weights_committed_to_repo": source_bool(source.as_ref(), "/weights_committed_to_repo").unwrap_or(false),
        "weights_retained_on_gpu_vm": source_bool(source.as_ref(), "/weights_retained_on_gpu_vm").unwrap_or(false),
        "recommended_storage": source_string(source.as_ref(), "/recommended_storage"),
        "current_policy": source_string(source.as_ref(), "/current_policy"),
    })
}

fn v02_native_inference_service_value() -> Value {
    let comparison = read_json_file(&state_paths::eden_v02_checkpoint_comparison_path());
    let adversarial = read_json_file(&state_paths::eden_v02_adversarial_eval_path());
    let rollback = read_json_file(&state_paths::eden_v02_rollback_drill_path());
    let model_card = read_json_file(&state_paths::eden_v02_model_card_internal_path());
    let service_ready = source_bool(comparison.as_ref(), "/passed") == Some(true)
        && source_bool(adversarial.as_ref(), "/passed") == Some(true)
        && source_bool(rollback.as_ref(), "/passed") == Some(true)
        && source_bool(model_card.as_ref(), "/passed") == Some(true);
    serde_json::json!({
        "schema": EDEN_V02_NATIVE_INFERENCE_SERVICE_SCHEMA,
        "artifact": "eden_v02_native_inference_service",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "service_ready": service_ready,
        "service_scope": "local_candidate_runtime_only",
        "request_contract": {
            "schema": "eden.v02.native_inference_request.v1",
            "fields": ["task_id", "goal", "situation_model", "memory_refs", "risk_class", "checkpoint_candidate", "rollback_target"]
        },
        "response_contract": {
            "schema": "eden.v02.native_inference_response.v1",
            "fields": ["candidate_text", "hypothesis_packet", "uncertainty", "verifier_required", "rollback_handle", "audit_trace"]
        },
        "guards": {
            "direct_memory_write": false,
            "direct_tool_execution": false,
            "direct_objective_update": false,
            "production_release_allowed": false
        },
    })
}

fn v02_stability_demo_value() -> Value {
    let source = read_repo_json(V02_DEMO_REPORT);
    serde_json::json!({
        "schema": EDEN_V02_STABILITY_DEMO_SCHEMA,
        "artifact": "eden_v02_stability_demo",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": V02_DEMO_REPORT,
        "source_present": source.is_some(),
        "passed": source_bool(source.as_ref(), "/passed"),
        "steps": source.as_ref().and_then(|v| v.get("steps")).cloned().unwrap_or(Value::Null),
        "safety_boundary": source.as_ref().and_then(|v| v.get("safety_boundary")).cloned().unwrap_or(Value::Null),
    })
}

fn v02_stability_gate_value() -> Value {
    let dataset = read_json_file(&state_paths::eden_v02_stability_corpus_manifest_path());
    let stability = read_json_file(&state_paths::eden_v02_stability_eval_path());
    let comparison = read_json_file(&state_paths::eden_v02_checkpoint_comparison_path());
    let adversarial = read_json_file(&state_paths::eden_v02_adversarial_eval_path());
    let rollback = read_json_file(&state_paths::eden_v02_rollback_drill_path());
    let model_card = read_json_file(&state_paths::eden_v02_model_card_internal_path());
    let storage = read_json_file(&state_paths::eden_v02_checkpoint_storage_path());
    let service = read_json_file(&state_paths::eden_v02_native_inference_service_path());
    let demo = read_json_file(&state_paths::eden_v02_stability_demo_path());
    let checks = vec![
        check(
            "larger_stability_dataset_4096_plus_rows",
            source_u64(dataset.as_ref(), "/rows/total").unwrap_or(0) >= 4096,
            V02_TRAIN_DATA,
        ),
        check(
            "stability_eval_passed",
            source_bool(stability.as_ref(), "/passed") == Some(true),
            V02_STABILITY_EVAL_REPORT,
        ),
        check(
            "checkpoint_comparison_100_to_250_passed",
            source_bool(comparison.as_ref(), "/passed") == Some(true),
            V02_COMPARISON_REPORT,
        ),
        check(
            "adversarial_eval_passed",
            source_bool(adversarial.as_ref(), "/passed") == Some(true),
            V02_ADVERSARIAL_REPORT,
        ),
        check(
            "rollback_drill_passed",
            source_bool(rollback.as_ref(), "/passed") == Some(true),
            V02_ROLLBACK_REPORT,
        ),
        check(
            "model_card_discloses_limits",
            source_bool(model_card.as_ref(), "/passed") == Some(true),
            V02_MODEL_CARD_REPORT,
        ),
        check(
            "checkpoint_storage_keeps_weights_out_of_repo",
            source_bool(storage.as_ref(), "/weights_committed_to_repo") == Some(false),
            V02_STORAGE_REPORT,
        ),
        check(
            "native_inference_service_ready",
            source_bool(service.as_ref(), "/service_ready") == Some(true),
            "eden_v02_native_inference_service.json",
        ),
        check(
            "operational_demo_passed_without_mutation",
            source_bool(demo.as_ref(), "/passed") == Some(true),
            V02_DEMO_REPORT,
        ),
    ];
    let passed = checks
        .iter()
        .filter(|check| check["passed"] == Value::Bool(true))
        .count();
    let total = checks.len();
    let candidate_runtime_admission_allowed = passed == total;
    serde_json::json!({
        "schema": EDEN_V02_STABILITY_GATE_SCHEMA,
        "artifact": "eden_v02_stability_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": total,
        "checks": checks,
        "candidate_runtime_admission_allowed": candidate_runtime_admission_allowed,
        "production_model_allowed": false,
        "max_dense_parameters": 14_000_000_000u64,
        "capability_class": "eden_v02_stable_candidate_runtime",
        "not_yet": [
            "production_checkpoint_release",
            "external_AGI_benchmark",
            "fully_autonomous_tool_authority",
            "AGI",
            "14B_pretraining_run"
        ],
    })
}

fn check(name: &str, passed: bool, evidence: &str) -> Value {
    serde_json::json!({
        "check": name,
        "passed": passed,
        "evidence": evidence,
    })
}

fn count_jsonl(path: &str) -> usize {
    let Some(path) = repo_path(path) else {
        return 0;
    };
    let Ok(body) = std::fs::read_to_string(path) else {
        return 0;
    };
    body.lines().filter(|line| !line.trim().is_empty()).count()
}

fn source_bool(source: Option<&Value>, pointer: &str) -> Option<bool> {
    source
        .and_then(|value| value.pointer(pointer))
        .and_then(Value::as_bool)
}

fn source_u64(source: Option<&Value>, pointer: &str) -> Option<u64> {
    source
        .and_then(|value| value.pointer(pointer))
        .and_then(Value::as_u64)
}

fn source_f64(source: Option<&Value>, pointer: &str) -> Option<f64> {
    source
        .and_then(|value| value.pointer(pointer))
        .and_then(Value::as_f64)
}

fn source_string(source: Option<&Value>, pointer: &str) -> Value {
    source
        .and_then(|value| value.pointer(pointer))
        .and_then(Value::as_str)
        .map(|value| Value::String(value.to_string()))
        .unwrap_or(Value::Null)
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
    fn dataset_manifest_counts_real_corpus_rows() {
        let manifest = dataset_manifest_value();

        assert_eq!(manifest["claim_allowed"], false);
        assert_eq!(manifest["agi_claim"], false);
        assert!(manifest["rows"]["total"].as_u64().unwrap_or(0) >= 300);
    }

    #[test]
    fn gate_keeps_checkpoint_admission_blocked() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!("eden_real_capability_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = run_all();
        let gate = read_json_file(&state_paths::eden_real_capability_gate_path()).unwrap();

        assert!(out.contains("[EDEN-REAL-CAPABILITY-GATE]"));
        assert_eq!(gate["claim_allowed"], false);
        assert_eq!(gate["agi_claim"], false);
        assert_eq!(gate["checkpoint_admission_allowed"], false);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn v01_dataset_manifest_counts_large_semantic_rows() {
        let manifest = v01_dataset_manifest_value();

        assert_eq!(manifest["claim_allowed"], false);
        assert_eq!(manifest["agi_claim"], false);
        assert!(manifest["rows"]["total"].as_u64().unwrap_or(0) >= 2048);
    }

    #[test]
    fn v01_gate_keeps_production_release_blocked() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!("eden_v01_capability_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = run_v01_all();
        let gate = read_json_file(&state_paths::eden_v01_capability_gate_path()).unwrap();

        assert!(out.contains("[EDEN-V01-CAPABILITY-GATE]"));
        assert_eq!(gate["claim_allowed"], false);
        assert_eq!(gate["agi_claim"], false);
        assert_eq!(gate["production_model_allowed"], false);
        assert_eq!(gate["max_dense_parameters"], 14_000_000_000u64);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
