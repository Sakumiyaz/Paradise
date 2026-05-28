use crate::eden_garm::state_paths;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const TRAINING_REPORT_SCHEMA: &str = "eden.training.capability_report.v1";
pub const TRAINING_EVIDENCE_SCHEMA: &str = "eden.garm.training_evidence.v1";
pub const MEGATRON_7B_EVIDENCE_SCHEMA: &str = "eden.megatron.7b.training_evidence.v1";
const DEFAULT_REPORT_PATH: &str = "target/eden_training_smoke/capability_report.json";
const DEFAULT_MEGATRON_7B_EVIDENCE_PATH: &str =
    "target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json";

#[derive(Clone, Debug, PartialEq)]
pub struct TrainingEvidenceSummary {
    pub passed: u64,
    pub total: u64,
    pub score: f64,
    pub result_count: usize,
    pub first_model_passed: u64,
    pub first_model_total: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Megatron7bEvidenceSummary {
    pub train_iters: u64,
    pub completed_iterations: u64,
    pub model_parameters: u64,
    pub final_loss: f64,
    pub checkpoint_written: bool,
}

pub fn run_default() -> String {
    match write_training_evidence_from_path(DEFAULT_REPORT_PATH) {
        Ok(path) => format!(
            "[TRAINING-EVIDENCE] schema={} claim_allowed=false agi_claim=false status=accepted source={} path={}\n",
            TRAINING_EVIDENCE_SCHEMA,
            DEFAULT_REPORT_PATH,
            path.to_string_lossy()
        ),
        Err(err) => format!(
            "[TRAINING-EVIDENCE] schema={} claim_allowed=false agi_claim=false status=rejected reason={}\n",
            TRAINING_EVIDENCE_SCHEMA, err
        ),
    }
}

pub fn run_megatron_7b_default() -> String {
    match write_megatron_7b_evidence_from_path(DEFAULT_MEGATRON_7B_EVIDENCE_PATH) {
        Ok(path) => format!(
            "[MEGATRON-7B-EVIDENCE] schema={} claim_allowed=false agi_claim=false checkpoint_admission=false status=accepted source={} path={}\n",
            MEGATRON_7B_EVIDENCE_SCHEMA,
            DEFAULT_MEGATRON_7B_EVIDENCE_PATH,
            path.to_string_lossy()
        ),
        Err(err) => format!(
            "[MEGATRON-7B-EVIDENCE] schema={} claim_allowed=false agi_claim=false checkpoint_admission=false status=rejected reason={}\n",
            MEGATRON_7B_EVIDENCE_SCHEMA, err
        ),
    }
}

pub fn write_training_evidence_from_path(path: impl AsRef<Path>) -> Result<PathBuf, String> {
    let report_json = std::fs::read_to_string(path.as_ref())
        .map_err(|e| format!("failed to read training report: {}", e))?;
    write_training_evidence_artifact(&report_json)
}

pub fn write_megatron_7b_evidence_from_path(path: impl AsRef<Path>) -> Result<PathBuf, String> {
    let evidence_json = std::fs::read_to_string(path.as_ref())
        .map_err(|e| format!("failed to read Megatron 7B evidence: {}", e))?;
    write_megatron_7b_evidence_artifact(&evidence_json)
}

pub fn write_megatron_7b_evidence_artifact(evidence_json: &str) -> Result<PathBuf, String> {
    let evidence: Value = serde_json::from_str(evidence_json)
        .map_err(|e| format!("invalid Megatron 7B evidence JSON: {}", e))?;
    validate_megatron_7b_evidence_value(&evidence)?;
    let path = PathBuf::from(state_paths::megatron_7b_training_evidence_path());
    state_paths::ensure_state_dir()?;
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&evidence).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("failed to write Megatron 7B evidence: {}", e))?;
    Ok(path)
}

pub fn write_training_evidence_artifact(report_json: &str) -> Result<PathBuf, String> {
    let report: Value =
        serde_json::from_str(report_json).map_err(|e| format!("invalid training JSON: {}", e))?;
    let evidence = build_training_evidence_value(&report)?;
    let path = PathBuf::from(state_paths::training_capability_evidence_path());
    state_paths::ensure_state_dir()?;
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&evidence).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("failed to write training evidence: {}", e))?;
    Ok(path)
}

pub fn build_training_evidence_value(report: &Value) -> Result<Value, String> {
    let summary = validate_training_report_value(report)?;
    Ok(serde_json::json!({
        "schema": TRAINING_EVIDENCE_SCHEMA,
        "authority": "global_executive_workspace_core",
        "source_schema": TRAINING_REPORT_SCHEMA,
        "claim_allowed": false,
        "agi_claim": false,
        "accepted_as": "training_pipeline_evidence",
        "source": {
            "path": DEFAULT_REPORT_PATH,
            "summary_hash": stable_hash_json(report.get("summary").unwrap_or(&Value::Null)),
        },
        "summary": {
            "passed": summary.passed,
            "total": summary.total,
            "score": summary.score,
            "result_count": summary.result_count,
            "first_model_passed": summary.first_model_passed,
            "first_model_total": summary.first_model_total,
        },
        "safety_boundary": {
            "direct_memory_writes": false,
            "direct_objective_writes": false,
            "direct_tool_execution": false,
            "requires_gewc_admission": true,
            "outputs_are_hypotheses": true,
            "model_may_not_mutate_runtime_state": true,
        },
        "runtime_use": {
            "accepted_for": "capability_smoke_and_training_pipeline_status",
            "not_accepted_for": [
                "external_validation",
                "agi_claim",
                "autonomous_tool_authority",
                "direct_memory_mutation"
            ]
        }
    }))
}

pub fn validate_training_report_value(report: &Value) -> Result<TrainingEvidenceSummary, String> {
    require_string_eq(report, "schema", TRAINING_REPORT_SCHEMA)?;
    require_false(report, "claim_allowed")?;
    require_false(report, "agi_claim")?;

    let summary = require_object(report, "summary")?;
    let passed = require_u64(summary, "passed")?;
    let total = require_u64(summary, "total")?;
    let score = require_f64(summary, "score")?;
    if total == 0 {
        return Err("summary.total must be greater than zero".to_string());
    }
    if passed > total {
        return Err("summary.passed cannot exceed total".to_string());
    }
    if !(0.0..=1.0).contains(&score) {
        return Err("summary.score must be between 0 and 1".to_string());
    }

    let results = require_array(report, "results")?;
    if results.len() != total as usize {
        return Err("results length must match summary.total".to_string());
    }

    let first_model = require_object(report, "first_model_eval")?;
    require_object_false(first_model, "claim_allowed")?;
    require_object_false(first_model, "agi_claim")?;
    require_object_false(first_model, "direct_memory_writes")?;
    require_object_false(first_model, "direct_objective_writes")?;
    require_object_false(first_model, "direct_tool_execution")?;
    let first_model_passed = require_u64(first_model, "passed")?;
    let first_model_total = require_u64(first_model, "total")?;
    if first_model_passed != first_model_total {
        return Err("first_model_eval must pass all smoke cases".to_string());
    }

    Ok(TrainingEvidenceSummary {
        passed,
        total,
        score,
        result_count: results.len(),
        first_model_passed,
        first_model_total,
    })
}

pub fn validate_megatron_7b_evidence_value(
    evidence: &Value,
) -> Result<Megatron7bEvidenceSummary, String> {
    require_string_eq(evidence, "schema", MEGATRON_7B_EVIDENCE_SCHEMA)?;
    require_string_eq(evidence, "authority", "global_executive_workspace_core")?;
    require_false(evidence, "claim_allowed")?;
    require_false(evidence, "agi_claim")?;
    require_string_eq(evidence, "accepted_as", "7b_training_path_evidence")?;

    let run = require_object(evidence, "run")?;
    require_object_bool_eq(run, "passed", true)?;
    require_object_string_eq(run, "network", "none")?;
    require_object_bool_eq(run, "mock_data", false)?;
    require_object_bool_eq(run, "external_model_dependency", false)?;
    require_object_string_eq(run, "model_scale", "7b_shape")?;
    let train_iters = require_u64(run, "train_iters")?;
    let completed_iterations = require_u64(run, "completed_iterations")?;
    let model_parameters = require_u64(run, "model_parameters")?;
    let final_loss = require_object_f64(run, "final_loss")?;
    if train_iters == 0 {
        return Err("run.train_iters must be greater than zero".to_string());
    }
    if completed_iterations == 0 || completed_iterations > train_iters {
        return Err("run.completed_iterations must be within train_iters".to_string());
    }
    if model_parameters < 6_900_000_000 {
        return Err("run.model_parameters must be a 7B-scale configuration".to_string());
    }
    if !final_loss.is_finite() || final_loss <= 0.0 {
        return Err("run.final_loss must be finite and positive".to_string());
    }
    if require_u64(run, "nan_iterations")? != 0 {
        return Err("run.nan_iterations must remain zero".to_string());
    }

    let checkpoint_policy = require_object(evidence, "checkpoint_policy")?;
    let checkpoint_written = require_object_bool(checkpoint_policy, "checkpoint_written")?;
    require_object_bool_eq(checkpoint_policy, "checkpoint_admission", false)?;
    require_object_bool_eq(checkpoint_policy, "weights_admitted", false)?;
    require_object_bool_eq(checkpoint_policy, "production_model", false)?;

    let safety_boundary = require_object(evidence, "safety_boundary")?;
    require_object_bool_eq(safety_boundary, "direct_memory_writes", false)?;
    require_object_bool_eq(safety_boundary, "direct_objective_writes", false)?;
    require_object_bool_eq(safety_boundary, "direct_tool_execution", false)?;
    require_object_bool_eq(safety_boundary, "requires_gewc_admission", true)?;
    require_object_bool_eq(safety_boundary, "outputs_are_hypotheses", true)?;

    Ok(Megatron7bEvidenceSummary {
        train_iters,
        completed_iterations,
        model_parameters,
        final_loss,
        checkpoint_written,
    })
}

fn require_object<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{}.object required", field))
}

fn require_array<'a>(value: &'a Value, field: &str) -> Result<&'a [Value], String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("{}.array required", field))
}

fn require_string_eq(value: &Value, field: &str, expected: &str) -> Result<(), String> {
    match value.get(field).and_then(Value::as_str) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!("{} expected {}, got {}", field, expected, actual)),
        None => Err(format!("{}.string required", field)),
    }
}

fn require_false(value: &Value, field: &str) -> Result<(), String> {
    match value.get(field).and_then(Value::as_bool) {
        Some(false) => Ok(()),
        Some(true) => Err(format!("{} must remain false", field)),
        None => Err(format!("{}.bool required", field)),
    }
}

fn require_object_false(value: &serde_json::Map<String, Value>, field: &str) -> Result<(), String> {
    match value.get(field).and_then(Value::as_bool) {
        Some(false) => Ok(()),
        Some(true) => Err(format!("{} must remain false", field)),
        None => Err(format!("{}.bool required", field)),
    }
}

fn require_object_bool(
    value: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<bool, String> {
    value
        .get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{}.bool required", field))
}

fn require_object_bool_eq(
    value: &serde_json::Map<String, Value>,
    field: &str,
    expected: bool,
) -> Result<(), String> {
    match require_object_bool(value, field)? {
        actual if actual == expected => Ok(()),
        actual => Err(format!("{} expected {}, got {}", field, expected, actual)),
    }
}

fn require_object_string_eq(
    value: &serde_json::Map<String, Value>,
    field: &str,
    expected: &str,
) -> Result<(), String> {
    match value.get(field).and_then(Value::as_str) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!("{} expected {}, got {}", field, expected, actual)),
        None => Err(format!("{}.string required", field)),
    }
}

fn require_u64(value: &serde_json::Map<String, Value>, field: &str) -> Result<u64, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{}.u64 required", field))
}

fn require_object_f64(value: &serde_json::Map<String, Value>, field: &str) -> Result<f64, String> {
    value
        .get(field)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{}.f64 required", field))
}

fn require_f64(value: &serde_json::Map<String, Value>, field: &str) -> Result<f64, String> {
    value
        .get(field)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{}.f64 required", field))
}

fn stable_hash_json(value: &Value) -> String {
    let serialized = serde_json::to_vec(value).unwrap_or_default();
    format!("{:016x}", fnv64(&serialized))
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

    fn valid_report() -> Value {
        serde_json::json!({
            "schema": TRAINING_REPORT_SCHEMA,
            "claim_allowed": false,
            "agi_claim": false,
            "profile": {"backend": "cpu_safe_smoke"},
            "model_config": {"module": "eden-memory-retrieval-baseline"},
            "device": {"rocm_detected": false},
            "summary": {
                "passed": 1,
                "total": 1,
                "score": 1.0,
                "by_capability": {"memory": {"passed": 1, "total": 1}}
            },
            "first_model_eval": {
                "module": "eden-memory-retrieval-baseline",
                "role": "GEWC_SUBORDINATE_MODULE",
                "claim_allowed": false,
                "agi_claim": false,
                "direct_memory_writes": false,
                "direct_objective_writes": false,
                "direct_tool_execution": false,
                "passed": 1,
                "total": 1,
                "score": 1.0,
                "results": [{"id": "eval", "passed": true}]
            },
            "results": [
                {
                    "id": "case",
                    "kind": "memory_retrieval",
                    "capability": "memory",
                    "passed": true,
                    "observed": "memory"
                }
            ]
        })
    }

    fn valid_megatron_7b_evidence() -> Value {
        serde_json::json!({
            "schema": MEGATRON_7B_EVIDENCE_SCHEMA,
            "authority": "global_executive_workspace_core",
            "claim_allowed": false,
            "agi_claim": false,
            "accepted_as": "7b_training_path_evidence",
            "source": {
                "summary_path": "target/eden_megatron_7b_base_pilot/eden_7b_base_pilot.summary",
                "log_path": "target/eden_megatron_7b_base_pilot/eden_7b_base_pilot.log"
            },
            "run": {
                "passed": true,
                "image": "rocm/megatron-lm:v25.3",
                "network": "none",
                "model_scale": "7b_shape",
                "tokenizer": "eden_sentencepiece",
                "dataset": "eden_core/corpus",
                "mock_data": false,
                "external_model_dependency": false,
                "train_iters": 1,
                "completed_iterations": 1,
                "model_parameters": 6996365312u64,
                "total_parameters_billion": 7.0,
                "final_loss": 8.269014,
                "nan_iterations": 0,
                "skipped_iterations": 0
            },
            "checkpoint_policy": {
                "checkpoint_written": false,
                "checkpoint_admission": false,
                "weights_admitted": false,
                "production_model": false
            },
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
    fn accepts_claim_gated_training_report() {
        let summary = validate_training_report_value(&valid_report()).unwrap();

        assert_eq!(summary.passed, 1);
        assert_eq!(summary.total, 1);
        assert_eq!(summary.first_model_passed, 1);
    }

    #[test]
    fn rejects_report_claim_escalation() {
        let mut report = valid_report();
        report["claim_allowed"] = Value::Bool(true);

        let err = validate_training_report_value(&report).unwrap_err();

        assert!(err.contains("claim_allowed"));
    }

    #[test]
    fn writes_governed_training_evidence_artifact() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_training_evidence_test_{}",
            std::process::id()
        )));
        let report_json = serde_json::to_string(&valid_report()).unwrap();

        let path = write_training_evidence_artifact(&report_json).unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains(TRAINING_EVIDENCE_SCHEMA));
        assert!(written.contains("\"claim_allowed\": false"));
        assert!(written.contains("\"direct_tool_execution\": false"));

        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn accepts_megatron_7b_training_evidence_without_checkpoint_admission() {
        let summary = validate_megatron_7b_evidence_value(&valid_megatron_7b_evidence()).unwrap();

        assert_eq!(summary.train_iters, 1);
        assert_eq!(summary.completed_iterations, 1);
        assert_eq!(summary.model_parameters, 6996365312);
        assert!(!summary.checkpoint_written);
    }

    #[test]
    fn rejects_megatron_7b_checkpoint_admission() {
        let mut evidence = valid_megatron_7b_evidence();
        evidence["checkpoint_policy"]["checkpoint_admission"] = Value::Bool(true);

        let err = validate_megatron_7b_evidence_value(&evidence).unwrap_err();

        assert!(err.contains("checkpoint_admission"));
    }

    #[test]
    fn writes_megatron_7b_evidence_artifact() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_megatron_7b_evidence_test_{}",
            std::process::id()
        )));
        let evidence_json = serde_json::to_string(&valid_megatron_7b_evidence()).unwrap();

        let path = write_megatron_7b_evidence_artifact(&evidence_json).unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains(MEGATRON_7B_EVIDENCE_SCHEMA));
        assert!(written.contains("\"checkpoint_admission\": false"));
        assert!(written.contains("\"external_model_dependency\": false"));

        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
