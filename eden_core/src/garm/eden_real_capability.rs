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

const AUTHORITY: &str = "global_executive_workspace_core";
const TRAIN_DATA: &str = "training/data/eden_real_capability_train.jsonl";
const EVAL_DATA: &str = "training/data/eden_real_capability_eval.jsonl";
const CHALLENGE_DATA: &str = "training/data/eden_real_capability_challenge.jsonl";
const CORPUS_MANIFEST: &str = "target/eden_real_capability/corpus_manifest.json";
const OPERATIONAL_EVAL_REPORT: &str = "target/eden_real_capability/capability_eval_report.json";
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
}
