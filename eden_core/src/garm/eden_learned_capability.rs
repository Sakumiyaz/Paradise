use crate::eden_garm::state_paths;
use serde_json::Value;

pub const EDEN_SFT_ELCP_DATASET_V2_MANIFEST_SCHEMA: &str = "eden.sft_elcp.dataset_v2_manifest.v1";
pub const EDEN_SFT_ELCP_GPU_TRAINING_REPORT_SCHEMA: &str = "eden.sft_elcp.gpu_training_report.v1";
pub const EDEN_SFT_ELCP_PREPOST_EVAL_SCHEMA: &str = "eden.sft_elcp.prepost_eval.v1";
pub const EDEN_SFT_ELCP_REPEATED_INFERENCE_EVAL_SCHEMA: &str =
    "eden.sft_elcp.repeated_inference_eval.v1";
pub const EDEN_SFT_ELCP_CHECKPOINT_ADMISSION_REVIEW_SCHEMA: &str =
    "eden.sft_elcp.checkpoint_admission_review.v1";
pub const EDEN_SFT_ELCP_OPERATIONAL_DEMO_SCHEMA: &str = "eden.sft_elcp.operational_demo.v1";
pub const EDEN_EXTERNAL_TESTS_CI_GATE_SCHEMA: &str = "eden.external_tests.ci_gate.v1";
pub const EDEN_LEARNED_CAPABILITY_GATE_SCHEMA: &str = "eden.learned_capability.gate.v1";

const AUTHORITY: &str = "global_executive_workspace_core";
const TRAIN_DATA: &str = "training/data/eden_cognitive_sft_elcp_train.jsonl";
const EVAL_DATA: &str = "training/data/eden_cognitive_sft_elcp_eval.jsonl";
const GPU_REPORT: &str = "target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_training_report.json";
const PREPOST_REPORT: &str = "target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_prepost_eval.json";
const PACKET_REPORT: &str = "target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_inference_packets.json";
const ADMISSION_REPORT: &str =
    "target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_checkpoint_admission_review.json";
const WORKFLOW_PATH: &str = ".github/workflows/garm-verify.yml";

pub fn run_all() -> String {
    let mut out = String::new();
    out.push_str(&write_dataset_v2_manifest());
    out.push_str(&write_gpu_training_report());
    out.push_str(&write_prepost_eval());
    out.push_str(&write_repeated_inference_eval());
    out.push_str(&write_checkpoint_admission_review());
    out.push_str(&write_operational_demo());
    out.push_str(&write_external_tests_ci_gate());
    out.push_str(&write_learned_capability_gate());
    out
}

pub fn write_dataset_v2_manifest() -> String {
    write_report(
        "EDEN-SFT-ELCP-DATASET-V2",
        EDEN_SFT_ELCP_DATASET_V2_MANIFEST_SCHEMA,
        state_paths::eden_sft_elcp_dataset_v2_manifest_path(),
        dataset_v2_manifest_value(),
    )
}

pub fn write_gpu_training_report() -> String {
    write_report(
        "EDEN-SFT-ELCP-GPU-TRAINING",
        EDEN_SFT_ELCP_GPU_TRAINING_REPORT_SCHEMA,
        state_paths::eden_sft_elcp_gpu_training_report_path(),
        gpu_training_report_value(),
    )
}

pub fn write_prepost_eval() -> String {
    write_report(
        "EDEN-SFT-ELCP-PREPOST-EVAL",
        EDEN_SFT_ELCP_PREPOST_EVAL_SCHEMA,
        state_paths::eden_sft_elcp_prepost_eval_path(),
        prepost_eval_value(),
    )
}

pub fn write_repeated_inference_eval() -> String {
    write_report(
        "EDEN-SFT-ELCP-REPEATED-INFERENCE",
        EDEN_SFT_ELCP_REPEATED_INFERENCE_EVAL_SCHEMA,
        state_paths::eden_sft_elcp_repeated_inference_eval_path(),
        repeated_inference_eval_value(),
    )
}

pub fn write_checkpoint_admission_review() -> String {
    write_report(
        "EDEN-SFT-ELCP-CHECKPOINT-ADMISSION",
        EDEN_SFT_ELCP_CHECKPOINT_ADMISSION_REVIEW_SCHEMA,
        state_paths::eden_sft_elcp_checkpoint_admission_review_path(),
        checkpoint_admission_review_value(),
    )
}

pub fn write_operational_demo() -> String {
    write_report(
        "EDEN-SFT-ELCP-OPERATIONAL-DEMO",
        EDEN_SFT_ELCP_OPERATIONAL_DEMO_SCHEMA,
        state_paths::eden_sft_elcp_operational_demo_path(),
        operational_demo_value(),
    )
}

pub fn write_external_tests_ci_gate() -> String {
    write_report(
        "EDEN-EXTERNAL-TESTS-CI-GATE",
        EDEN_EXTERNAL_TESTS_CI_GATE_SCHEMA,
        state_paths::eden_external_tests_ci_gate_path(),
        external_tests_ci_gate_value(),
    )
}

pub fn write_learned_capability_gate() -> String {
    write_report(
        "EDEN-LEARNED-CAPABILITY-GATE",
        EDEN_LEARNED_CAPABILITY_GATE_SCHEMA,
        state_paths::eden_learned_capability_gate_path(),
        learned_capability_gate_value(),
    )
}

fn dataset_v2_manifest_value() -> Value {
    let train_rows = count_jsonl(TRAIN_DATA);
    let eval_rows = count_jsonl(EVAL_DATA);
    serde_json::json!({
        "schema": EDEN_SFT_ELCP_DATASET_V2_MANIFEST_SCHEMA,
        "artifact": "eden_sft_elcp_dataset_v2_manifest",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "contains_private_data": false,
        "source": "deterministic_repo_local_synthetic_cognitive_transitions",
        "train": {
            "path": TRAIN_DATA,
            "rows": train_rows,
            "present": repo_path(TRAIN_DATA).is_some(),
            "fnv64": file_hash(TRAIN_DATA)
        },
        "eval": {
            "path": EVAL_DATA,
            "rows": eval_rows,
            "present": repo_path(EVAL_DATA).is_some(),
            "fnv64": file_hash(EVAL_DATA)
        },
        "coverage": [
            "artifact_inspection",
            "memory_write",
            "prompt_injection",
            "irreversible_action",
            "rollback",
            "world_model",
            "multiagent_conflict",
            "safe_learning",
            "metacognition",
            "checkpoint_probe"
        ],
        "accepted_for": [
            "SFT_structured_packet_following_pilot",
            "ELCP_latent_cognitive_transition_pilot",
            "prepost_eval",
            "repeated_inference_demo"
        ],
        "not_accepted_for": [
            "private_user_memory",
            "production_model_release",
            "AGI_claim"
        ],
    })
}

fn gpu_training_report_value() -> Value {
    let source = read_repo_json(GPU_REPORT);
    serde_json::json!({
        "schema": EDEN_SFT_ELCP_GPU_TRAINING_REPORT_SCHEMA,
        "artifact": "eden_sft_elcp_gpu_training_report",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": GPU_REPORT,
        "source_present": source.is_some(),
        "training_executed": source_bool(source.as_ref(), "/training_executed"),
        "gpu_job_submitted": source_bool(source.as_ref(), "/gpu_job_submitted"),
        "device": source_string(source.as_ref(), "/device"),
        "epochs": source_u64(source.as_ref(), "/epochs"),
        "train_rows": source_u64(source.as_ref(), "/datasets/train_rows"),
        "eval_rows": source_u64(source.as_ref(), "/datasets/eval_rows"),
        "loss": source.as_ref().and_then(|value| value.get("loss")).cloned().unwrap_or(Value::Null),
        "checkpoint_sha256": source_string(source.as_ref(), "/checkpoint_sha256"),
        "external_model_dependency": false,
        "checkpoint_admission_allowed": false,
        "raw_source_summary": compact_source(source.as_ref()),
    })
}

fn prepost_eval_value() -> Value {
    let source = read_repo_json(PREPOST_REPORT);
    let pre_field = source_f64(source.as_ref(), "/pre_eval/field_score");
    let post_field = source_f64(source.as_ref(), "/post_eval/field_score");
    let pre_row = source_f64(source.as_ref(), "/pre_eval/row_score");
    let post_row = source_f64(source.as_ref(), "/post_eval/row_score");
    serde_json::json!({
        "schema": EDEN_SFT_ELCP_PREPOST_EVAL_SCHEMA,
        "artifact": "eden_sft_elcp_prepost_eval",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": PREPOST_REPORT,
        "source_present": source.is_some(),
        "pre_field_score": pre_field,
        "post_field_score": post_field,
        "field_delta": post_field.unwrap_or(0.0) - pre_field.unwrap_or(0.0),
        "pre_row_score": pre_row,
        "post_row_score": post_row,
        "row_delta": post_row.unwrap_or(0.0) - pre_row.unwrap_or(0.0),
        "improved": post_field.unwrap_or(0.0) > pre_field.unwrap_or(0.0),
        "measured_scope": "cognitive_transition_contract_eval",
        "not_measured": [
            "AGI",
            "open_domain_language_quality",
            "external_benchmark_superiority"
        ],
    })
}

fn repeated_inference_eval_value() -> Value {
    let source = read_repo_json(PACKET_REPORT);
    let packets = source
        .as_ref()
        .and_then(|value| value.get("packets"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let safe_packets = packets
        .iter()
        .filter(|packet| {
            packet
                .pointer("/authority/accepted_as_truth")
                .and_then(Value::as_bool)
                == Some(false)
                && packet
                    .pointer("/candidate_structure/requires_verification")
                    .and_then(Value::as_bool)
                    == Some(true)
        })
        .count();
    serde_json::json!({
        "schema": EDEN_SFT_ELCP_REPEATED_INFERENCE_EVAL_SCHEMA,
        "artifact": "eden_sft_elcp_repeated_inference_eval",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": PACKET_REPORT,
        "source_present": source.is_some(),
        "packet_count": packets.len(),
        "safe_packet_count": safe_packets,
        "all_packets_require_verification": !packets.is_empty() && safe_packets == packets.len(),
        "runtime_use": "candidate_packets_only",
        "packets": packets,
    })
}

fn checkpoint_admission_review_value() -> Value {
    let source = read_repo_json(ADMISSION_REPORT);
    serde_json::json!({
        "schema": EDEN_SFT_ELCP_CHECKPOINT_ADMISSION_REVIEW_SCHEMA,
        "artifact": "eden_sft_elcp_checkpoint_admission_review",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "source_path": ADMISSION_REPORT,
        "source_present": source.is_some(),
        "release_candidate": source_bool(source.as_ref(), "/release_candidate"),
        "checkpoint_admission_allowed": false,
        "metrics": source.as_ref().and_then(|value| value.get("metrics")).cloned().unwrap_or(Value::Null),
        "thresholds": source.as_ref().and_then(|value| value.get("thresholds")).cloned().unwrap_or(Value::Null),
        "required_before_admission": [
            "GEWC_review",
            "external_regression",
            "adversarial_prompt_injection_eval",
            "rollback_drill",
            "operator_approval"
        ],
        "reason": "real GPU pilot may create evidence or release candidate status, but checkpoint admission remains a separate GEWC decision",
    })
}

fn operational_demo_value() -> Value {
    let packets = read_repo_json(PACKET_REPORT)
        .and_then(|value| value.get("packets").and_then(Value::as_array).cloned())
        .unwrap_or_default();
    serde_json::json!({
        "schema": EDEN_SFT_ELCP_OPERATIONAL_DEMO_SCHEMA,
        "artifact": "eden_sft_elcp_operational_demo",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "demo": "learned_cognitive_transition_packet_through_gewc",
        "source_packets_present": !packets.is_empty(),
        "steps": [
            {"index": 1, "name": "receive_task", "result": "GEWC builds structured state"},
            {"index": 2, "name": "route_model", "result": "learned SFT/ELCP module predicts cognitive transition"},
            {"index": 3, "name": "packetize", "result": "prediction becomes hypothesis packet"},
            {"index": 4, "name": "verify", "result": "GEWC keeps final authority"},
            {"index": 5, "name": "admit_effects", "result": "audit metadata and draft plan allowed; direct memory/tool effects blocked"},
            {"index": 6, "name": "record_evidence", "result": "demo remains no-claim and reproducible"}
        ],
        "sample_packet": packets.first().cloned().unwrap_or(Value::Null),
    })
}

fn external_tests_ci_gate_value() -> Value {
    let workflow = read_repo_text(WORKFLOW_PATH).unwrap_or_default();
    let has_job = workflow.contains("external_tests_optional");
    let has_make = workflow.contains("make external-tests");
    serde_json::json!({
        "schema": EDEN_EXTERNAL_TESTS_CI_GATE_SCHEMA,
        "artifact": "eden_external_tests_ci_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "workflow_path": WORKFLOW_PATH,
        "workflow_present": repo_path(WORKFLOW_PATH).is_some(),
        "manual_external_tests_job_present": has_job,
        "runs_make_external_tests": has_make,
        "local_command": "make external-tests",
        "coverage": [
            "gpio_external",
            "i2c_external",
            "crawler_external"
        ],
        "ci_policy": "external tests are explicit/manual because they depend on hardware or network state, but they are not ignored when requested",
        "passed": has_job && has_make,
        "total": 2,
    })
}

fn learned_capability_gate_value() -> Value {
    let dataset = read_json_file(&state_paths::eden_sft_elcp_dataset_v2_manifest_path());
    let training = read_json_file(&state_paths::eden_sft_elcp_gpu_training_report_path());
    let prepost = read_json_file(&state_paths::eden_sft_elcp_prepost_eval_path());
    let packets = read_json_file(&state_paths::eden_sft_elcp_repeated_inference_eval_path());
    let admission = read_json_file(&state_paths::eden_sft_elcp_checkpoint_admission_review_path());
    let demo = read_json_file(&state_paths::eden_sft_elcp_operational_demo_path());
    let external = read_json_file(&state_paths::eden_external_tests_ci_gate_path());
    let dataset_rows = source_u64(dataset.as_ref(), "/train/rows").unwrap_or(0)
        + source_u64(dataset.as_ref(), "/eval/rows").unwrap_or(0);
    let checks = vec![
        check(
            "dataset_v2_has_100_rows",
            dataset_rows >= 100,
            "training/data/eden_cognitive_sft_elcp_*.jsonl",
        ),
        check(
            "gpu_training_executed",
            source_bool(training.as_ref(), "/training_executed") == Some(true)
                && source_bool(training.as_ref(), "/gpu_job_submitted") == Some(true),
            GPU_REPORT,
        ),
        check(
            "prepost_eval_improved",
            source_bool(prepost.as_ref(), "/improved") == Some(true),
            PREPOST_REPORT,
        ),
        check(
            "repeated_inference_packets_verified_boundary",
            source_bool(packets.as_ref(), "/all_packets_require_verification") == Some(true),
            PACKET_REPORT,
        ),
        check(
            "checkpoint_admission_remains_blocked",
            source_bool(admission.as_ref(), "/checkpoint_admission_allowed") == Some(false),
            ADMISSION_REPORT,
        ),
        check(
            "operational_demo_has_packet",
            source_bool(demo.as_ref(), "/source_packets_present") == Some(true),
            "eden_sft_elcp_operational_demo.json",
        ),
        check(
            "external_tests_ci_gate_present",
            source_bool(external.as_ref(), "/passed") == Some(true),
            WORKFLOW_PATH,
        ),
    ];
    let passed = checks
        .iter()
        .filter(|check| check["passed"] == Value::Bool(true))
        .count();
    serde_json::json!({
        "schema": EDEN_LEARNED_CAPABILITY_GATE_SCHEMA,
        "artifact": "eden_learned_capability_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": checks.len(),
        "checks": checks,
        "training_executed": source_bool(training.as_ref(), "/training_executed"),
        "gpu_job_submitted": source_bool(training.as_ref(), "/gpu_job_submitted"),
        "post_field_score": source_f64(prepost.as_ref(), "/post_field_score"),
        "post_row_score": source_f64(prepost.as_ref(), "/post_row_score"),
        "all_packets_require_verification": source_bool(packets.as_ref(), "/all_packets_require_verification"),
        "checkpoint_admission_allowed": false,
        "capability_class": "learned_cognitive_transition_pilot",
        "not_yet": [
            "7B_SFT",
            "production_model_release",
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

fn compact_source(source: Option<&Value>) -> Value {
    match source {
        Some(value) => serde_json::json!({
            "schema": value.get("schema").cloned().unwrap_or(Value::Null),
            "training_executed": value.get("training_executed").cloned().unwrap_or(Value::Null),
            "gpu_job_submitted": value.get("gpu_job_submitted").cloned().unwrap_or(Value::Null),
            "device": value.get("device").cloned().unwrap_or(Value::Null),
            "checkpoint_sha256": value.get("checkpoint_sha256").cloned().unwrap_or(Value::Null),
        }),
        None => serde_json::json!({
            "present": false,
            "expected_path": GPU_REPORT
        }),
    }
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

fn file_hash(path: &str) -> Value {
    repo_path(path)
        .and_then(|path| std::fs::read(path).ok())
        .map(|bytes| Value::String(fnv64(&bytes)))
        .unwrap_or(Value::Null)
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

fn read_repo_text(path: &str) -> Option<String> {
    let path = repo_path(path)?;
    std::fs::read_to_string(path).ok()
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

fn fnv64(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_manifest_counts_generated_rows() {
        let manifest = dataset_v2_manifest_value();

        assert_eq!(manifest["claim_allowed"], false);
        assert_eq!(manifest["agi_claim"], false);
        assert!(manifest["train"]["rows"].as_u64().unwrap_or(0) >= 80);
        assert!(manifest["eval"]["rows"].as_u64().unwrap_or(0) >= 20);
    }

    #[test]
    fn gate_stays_no_claim_and_blocks_checkpoint_admission() {
        let _guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_learned_capability_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = run_all();
        let gate = read_json_file(&state_paths::eden_learned_capability_gate_path()).unwrap();

        assert!(out.contains("[EDEN-LEARNED-CAPABILITY-GATE]"));
        assert_eq!(gate["claim_allowed"], false);
        assert_eq!(gate["agi_claim"], false);
        assert_eq!(gate["checkpoint_admission_allowed"], false);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
