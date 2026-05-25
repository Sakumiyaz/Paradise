//! Independent readiness package validator for EDEN GARM.
//!
//! This runner does not start the GARM runtime. It consumes exported artifacts,
//! verifies package integrity, runs local adversarial checks, and writes a
//! release-candidate manifest for audit handoff.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const REQUIRED_ARTIFACTS: &[&str] = &[
    "external_validation_suite",
    "external_validation_result",
    "capability_registry",
    "cognitive_architecture",
    "embodied_grounding",
    "neural_architecture",
    "symbolic_architecture",
    "self_improvement_architecture",
    "safety_control_architecture",
    "foundation_model_architecture",
    "multimodal_model_architecture",
    "llm_agent_architecture",
    "probabilistic_programming_architecture",
    "hierarchical_rl_architecture",
    "cognitive_robotics_architecture",
    "vla_architecture",
    "sim_to_real_architecture",
    "open_ended_evolution_architecture",
    "developmental_robotics_architecture",
    "whole_brain_neurocognitive_architecture",
    "neuromorphic_spiking_architecture",
    "paradigm_architecture_map",
    "paradigm_architecture_technique_map",
    "neuro_symbolic_paradigm",
    "universal_formal_paradigm",
    "active_inference_paradigm",
    "ecological_systemic_paradigm",
    "computational_programmatic_paradigm",
    "affective_motivational_paradigm",
    "human_in_the_loop_paradigm",
    "emergence_metrics_paradigm",
    "integration_governance_architecture",
    "global_executive_workspace_core",
    "global_executive_workspace_runtime",
    "global_executive_workspace_runtime_state",
    "gewc_operational_benchmark",
    "gewc_runtime_safety_report",
    "gewc_long_run_stability",
    "capability_reality_eval",
    "capability_reality_matrix",
    "lmm_training_dependency_report",
    "gewc_trace_spec",
    "capability_reality_matrix_v2",
    "cognitive_task_suite",
    "eden_agent_sdk_contract",
    "model_adapter_layer",
    "reproducible_demos",
    "architecture_advantage_eval",
    "eden_praxis_nexus",
    "praxis_primitives",
    "praxis_blocks",
    "praxis_space",
    "praxis_rules",
    "praxis_trace_semantics",
    "praxis_reasoner",
    "praxis_bench",
    "eden_locus_layer",
    "locus_authority_model",
    "locus_evidence_vault",
    "locus_permission_matrix",
    "locus_context_packet",
    "locus_operator_timeline",
    "eden_operator_forge",
    "operator_primitive_basis",
    "operator_expression_graphs",
    "operator_verification_report",
    "operator_model_registry",
    "locus_operator_bridge",
    "eden_sovereign_cognition",
    "sovereign_sector_wins",
    "praxis_calculus_formalism",
    "cognitive_contract_language",
    "evidence_memory_fabric",
    "federated_runtime_fabric",
    "symbolic_reasoning_fabric",
    "eden_external_ecosystem",
    "ecosystem_participation_contract",
    "ecosystem_interop_matrix",
    "ecosystem_certification_ladder",
    "ecosystem_onboarding_runbook",
    "ecosystem_governance_model",
    "ecosystem_benchmark_exchange",
    "artifact_api_catalog",
    "artifact_api_contracts",
    "artifact_api_runtime",
    "runtime_state_api_catalog",
    "runtime_state_api_contracts",
    "runtime_state_api_openapi",
    "runtime_state_api_runtime",
    "operational_api_catalog",
    "operational_api_contracts",
    "operational_api_openapi",
    "operational_api_runtime",
    "operational_action_contracts",
    "memory_eval",
    "world_eval",
    "readiness_package",
];

const RELEASE_MANIFEST: &str = "release_candidate_manifest.json";
const VALIDATION_REPORT: &str = "independent_validation_report.json";

#[derive(Debug, Default)]
struct ValidationSummary {
    checks: Vec<Value>,
    artifact_checks: Vec<Value>,
    failures: Vec<String>,
}

impl ValidationSummary {
    fn pass(&mut self, name: &str, evidence: impl Into<String>) {
        self.checks.push(serde_json::json!({
            "name": name,
            "passed": true,
            "evidence": evidence.into(),
        }));
    }

    fn fail(&mut self, name: &str, evidence: impl Into<String>) {
        let evidence = evidence.into();
        self.failures.push(format!("{name}: {evidence}"));
        self.checks.push(serde_json::json!({
            "name": name,
            "passed": false,
            "evidence": evidence,
        }));
    }

    fn ok(&self) -> bool {
        self.failures.is_empty()
    }
}

pub fn main_entry() {
    let state_dir = parse_state_dir();
    let summary = validate_state_dir(&state_dir);
    let report_path = state_dir.join(VALIDATION_REPORT);
    let status = if summary.ok() { "passed" } else { "failed" };
    let body = serde_json::json!({
        "schema": "garm-independent-validation-report-v1",
        "status": status,
        "state_dir": state_dir.to_string_lossy(),
        "checks": summary.checks,
        "artifact_checks": summary.artifact_checks,
        "failures": summary.failures,
    });
    let _ = std::fs::write(
        &report_path,
        serde_json::to_string_pretty(&body).unwrap_or_else(|_| body.to_string()),
    );

    println!(
        "[INDEPENDENT-VALIDATION] status={} checks={} failures={} report={}",
        status,
        body.get("checks")
            .and_then(|v| v.as_array())
            .map_or(0, Vec::len),
        body.get("failures")
            .and_then(|v| v.as_array())
            .map_or(0, Vec::len),
        report_path.to_string_lossy()
    );
    if summary.ok() {
        println!(
            "[RELEASE-CANDIDATE] manifest={}",
            state_dir.join(RELEASE_MANIFEST).to_string_lossy()
        );
    } else {
        for failure in body
            .get("failures")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            println!("- {}", failure.as_str().unwrap_or("unknown failure"));
        }
        std::process::exit(1);
    }
}

fn parse_state_dir() -> PathBuf {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--state-dir" {
            if let Some(path) = args.next() {
                return PathBuf::from(path);
            }
        }
    }
    PathBuf::from("/tmp/eden_garm_make_validation")
}

fn validate_state_dir(state_dir: &Path) -> ValidationSummary {
    let mut summary = ValidationSummary::default();
    let package_path = state_dir.join("readiness_package.json");
    let package = read_json(&package_path);
    let Some(package) = package else {
        summary.fail(
            "package_present",
            format!("missing or invalid {}", package_path.to_string_lossy()),
        );
        return summary;
    };
    summary.pass("package_present", package_path.to_string_lossy());
    validate_package_policy(&package, &mut summary);
    validate_artifacts(state_dir, &package, &mut summary);
    validate_suite_and_result(state_dir, &mut summary);
    validate_adversarial_controls(&package, &mut summary);
    if summary.ok() {
        write_release_manifest(state_dir, &package, &mut summary);
    }
    summary
}

fn validate_package_policy(package: &Value, summary: &mut ValidationSummary) {
    if package.get("schema").and_then(Value::as_str) == Some("garm-readiness-package-v1") {
        summary.pass("package_schema", "garm-readiness-package-v1");
    } else {
        summary.fail("package_schema", "unexpected package schema");
    }
    if package.get("claim_allowed").and_then(Value::as_bool) == Some(false)
        && package.get("agi_claim").and_then(Value::as_bool) == Some(false)
    {
        summary.pass(
            "package_claim_policy",
            "claim_allowed=false agi_claim=false",
        );
    } else {
        summary.fail(
            "package_claim_policy",
            "package attempts to allow unsupported claims",
        );
    }
}

fn validate_artifacts(state_dir: &Path, package: &Value, summary: &mut ValidationSummary) {
    let mut by_name = BTreeMap::new();
    for artifact in package
        .get("artifacts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(name) = artifact.get("name").and_then(Value::as_str) {
            by_name.insert(name.to_string(), artifact.clone());
        }
    }
    let package_artifact = serde_json::json!({
        "name": "readiness_package",
        "path": state_dir.join("readiness_package.json").to_string_lossy(),
        "present": true,
        "bytes": std::fs::metadata(state_dir.join("readiness_package.json")).map(|m| m.len()).unwrap_or(0),
        "fnv64": format!("{:016x}", fnv64(&std::fs::read(state_dir.join("readiness_package.json")).unwrap_or_default())),
    });
    by_name.insert("readiness_package".to_string(), package_artifact);

    for required in REQUIRED_ARTIFACTS {
        match by_name.get(*required) {
            Some(_) => summary.pass("required_artifact_declared", *required),
            None => {
                summary.fail("required_artifact_declared", format!("missing {required}"));
                continue;
            }
        }
        if let Some(artifact) = by_name.get(*required) {
            validate_artifact_record(artifact, summary);
        }
    }

    let declared: BTreeSet<_> = by_name.keys().cloned().collect();
    summary.pass(
        "artifact_inventory_loaded",
        format!("declared_artifacts={}", declared.len()),
    );
}

fn validate_artifact_record(artifact: &Value, summary: &mut ValidationSummary) {
    let name = artifact
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let Some(path) = artifact.get("path").and_then(Value::as_str) else {
        summary.fail("artifact_path", format!("{name} has no path"));
        return;
    };
    let bytes = std::fs::read(path).unwrap_or_default();
    let expected_bytes = artifact.get("bytes").and_then(Value::as_u64).unwrap_or(0);
    let expected_hash = artifact.get("fnv64").and_then(Value::as_str).unwrap_or("");
    let actual_hash = format!("{:016x}", fnv64(&bytes));
    let present = !bytes.is_empty();
    let passed = present && bytes.len() as u64 == expected_bytes && actual_hash == expected_hash;
    summary.artifact_checks.push(serde_json::json!({
        "name": name,
        "path": path,
        "present": present,
        "expected_bytes": expected_bytes,
        "actual_bytes": bytes.len(),
        "expected_fnv64": expected_hash,
        "actual_fnv64": actual_hash,
        "passed": passed,
    }));
    if passed {
        summary.pass("artifact_integrity", name);
    } else {
        summary.fail("artifact_integrity", format!("{name} bytes/hash mismatch"));
    }
}

fn validate_suite_and_result(state_dir: &Path, summary: &mut ValidationSummary) {
    let suite_path = state_dir.join("external_validation_suite.json");
    let result_path = state_dir.join("external_validation_result.json");
    let suite = read_json(&suite_path);
    let result = read_json(&result_path);
    let (Some(suite), Some(result)) = (suite, result) else {
        summary.fail("suite_result_parse", "suite or result json missing/invalid");
        return;
    };
    let suite_body = std::fs::read(&suite_path).unwrap_or_default();
    let suite_hash = format!("{:016x}", fnv64(&suite_body));
    if result.get("suite_fnv64").and_then(Value::as_str) == Some(suite_hash.as_str()) {
        summary.pass("suite_hash_matches_result", suite_hash);
    } else {
        summary.fail(
            "suite_hash_matches_result",
            "result suite_fnv64 does not match suite file",
        );
    }
    let suite_cases = suite
        .get("cases")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let result_cases = result
        .get("cases")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !suite_cases.is_empty() && suite_cases.len() == result_cases.len() {
        summary.pass("suite_case_count", format!("cases={}", suite_cases.len()));
    } else {
        summary.fail(
            "suite_case_count",
            format!("suite={} result={}", suite_cases.len(), result_cases.len()),
        );
    }
    if result.get("claim_allowed").and_then(Value::as_bool) == Some(false)
        && result.get("agi_claim").and_then(Value::as_bool) == Some(false)
    {
        summary.pass("result_claim_policy", "claim_allowed=false agi_claim=false");
    } else {
        summary.fail("result_claim_policy", "result attempts unsupported claim");
    }
    let passed = result.get("passed").and_then(Value::as_u64).unwrap_or(0);
    let total = result.get("total").and_then(Value::as_u64).unwrap_or(0);
    if total >= 60 && passed == total {
        summary.pass(
            "held_out_result_threshold",
            format!("passed={passed}/{total}"),
        );
    } else {
        summary.fail(
            "held_out_result_threshold",
            format!("passed={passed}/{total}"),
        );
    }
}

fn validate_adversarial_controls(package: &Value, summary: &mut ValidationSummary) {
    if package.get("claim_allowed").and_then(Value::as_bool) == Some(true) {
        summary.fail(
            "adversarial_claim_escalation",
            "claim_allowed=true accepted",
        );
    } else {
        summary.pass("adversarial_claim_escalation", "claim escalation rejected");
    }
    let artifacts = package
        .get("artifacts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let missing_critical = REQUIRED_ARTIFACTS
        .iter()
        .filter(|name| {
            **name != "readiness_package"
                && !artifacts
                    .iter()
                    .any(|artifact| artifact.get("name").and_then(Value::as_str) == Some(**name))
        })
        .count();
    if missing_critical == 0 {
        summary.pass(
            "adversarial_missing_artifact",
            "critical inventory complete",
        );
    } else {
        summary.fail(
            "adversarial_missing_artifact",
            format!("missing_critical={missing_critical}"),
        );
    }
    summary.pass(
        "adversarial_corruption_detection",
        "artifact byte count and fnv64 are checked before release manifest write",
    );
}

fn write_release_manifest(state_dir: &Path, package: &Value, summary: &mut ValidationSummary) {
    let mut checksums = Vec::new();
    for artifact in &summary.artifact_checks {
        checksums.push(serde_json::json!({
            "name": artifact.get("name").cloned().unwrap_or(Value::Null),
            "path": artifact.get("path").cloned().unwrap_or(Value::Null),
            "fnv64": artifact.get("actual_fnv64").cloned().unwrap_or(Value::Null),
            "bytes": artifact.get("actual_bytes").cloned().unwrap_or(Value::Null),
        }));
    }
    let manifest = serde_json::json!({
        "schema": "garm-release-candidate-manifest-v1",
        "status": "release_candidate_local_independent_validation_passed",
        "created_unix": unix_now(),
        "commit": git_commit().unwrap_or_else(|| "unknown".to_string()),
        "package_version": package.get("package_version").cloned().unwrap_or(Value::Null),
        "suite_version": read_json(&state_dir.join("external_validation_suite.json"))
            .and_then(|suite| suite.get("suite_version").cloned())
            .unwrap_or(Value::Null),
        "state_dir": state_dir.to_string_lossy(),
        "reproduction_commands": [
            "make eden-validate-local",
            "make eden-package",
            "make eden-independent-validate",
            "make eden-release-candidate"
        ],
        "artifact_checksums": checksums,
        "claim_allowed": false,
        "agi_claim": false,
    });
    let body = serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| manifest.to_string());
    let path = state_dir.join(RELEASE_MANIFEST);
    match std::fs::write(&path, body) {
        Ok(()) => summary.pass("release_manifest_written", path.to_string_lossy()),
        Err(err) => summary.fail("release_manifest_written", err.to_string()),
    }
}

fn read_json(path: &Path) -> Option<Value> {
    let body = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&body).ok()
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn git_commit() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validator_fails_when_critical_artifact_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        write_fixture_package(dir.path(), false, false);
        let summary = validate_state_dir(dir.path());
        assert!(!summary.ok());
        assert!(summary
            .failures
            .iter()
            .any(|failure| failure.contains("external_validation_suite")));
    }

    #[test]
    fn validator_rejects_claim_escalation() {
        let dir = tempfile::tempdir().unwrap();
        write_fixture_package(dir.path(), true, true);
        let summary = validate_state_dir(dir.path());
        assert!(!summary.ok());
        assert!(summary
            .failures
            .iter()
            .any(|failure| failure.contains("claim")));
    }

    #[test]
    fn validator_rejects_corrupted_artifact() {
        let dir = tempfile::tempdir().unwrap();
        write_fixture_package(dir.path(), true, false);
        std::fs::write(dir.path().join("memory_eval.json"), "{}").unwrap();
        let summary = validate_state_dir(dir.path());
        assert!(!summary.ok());
        assert!(summary
            .failures
            .iter()
            .any(|failure| failure.contains("memory_eval")));
    }

    #[test]
    fn validator_accepts_complete_fixture() {
        let dir = tempfile::tempdir().unwrap();
        write_fixture_package(dir.path(), true, false);
        let summary = validate_state_dir(dir.path());
        assert!(summary.ok(), "{:?}", summary.failures);
        assert!(dir.path().join(RELEASE_MANIFEST).exists());
    }

    fn write_fixture_package(dir: &Path, include_suite: bool, claim_allowed: bool) {
        let artifact_names = [
            "capability_registry",
            "cognitive_architecture",
            "embodied_grounding",
            "neural_architecture",
            "symbolic_architecture",
            "self_improvement_architecture",
            "safety_control_architecture",
            "foundation_model_architecture",
            "multimodal_model_architecture",
            "llm_agent_architecture",
            "probabilistic_programming_architecture",
            "hierarchical_rl_architecture",
            "cognitive_robotics_architecture",
            "vla_architecture",
            "sim_to_real_architecture",
            "open_ended_evolution_architecture",
            "developmental_robotics_architecture",
            "whole_brain_neurocognitive_architecture",
            "neuromorphic_spiking_architecture",
            "paradigm_architecture_map",
            "paradigm_architecture_technique_map",
            "neuro_symbolic_paradigm",
            "universal_formal_paradigm",
            "active_inference_paradigm",
            "ecological_systemic_paradigm",
            "computational_programmatic_paradigm",
            "affective_motivational_paradigm",
            "human_in_the_loop_paradigm",
            "emergence_metrics_paradigm",
            "integration_governance_architecture",
            "global_executive_workspace_core",
            "global_executive_workspace_runtime",
            "global_executive_workspace_runtime_state",
            "gewc_operational_benchmark",
            "gewc_runtime_safety_report",
            "gewc_long_run_stability",
            "capability_reality_eval",
            "capability_reality_matrix",
            "lmm_training_dependency_report",
            "gewc_trace_spec",
            "capability_reality_matrix_v2",
            "cognitive_task_suite",
            "eden_agent_sdk_contract",
            "model_adapter_layer",
            "reproducible_demos",
            "architecture_advantage_eval",
            "eden_praxis_nexus",
            "praxis_primitives",
            "praxis_blocks",
            "praxis_space",
            "praxis_rules",
            "praxis_trace_semantics",
            "praxis_reasoner",
            "praxis_bench",
            "eden_locus_layer",
            "locus_authority_model",
            "locus_evidence_vault",
            "locus_permission_matrix",
            "locus_context_packet",
            "locus_operator_timeline",
            "eden_operator_forge",
            "operator_primitive_basis",
            "operator_expression_graphs",
            "operator_verification_report",
            "operator_model_registry",
            "locus_operator_bridge",
            "eden_sovereign_cognition",
            "sovereign_sector_wins",
            "praxis_calculus_formalism",
            "cognitive_contract_language",
            "evidence_memory_fabric",
            "federated_runtime_fabric",
            "symbolic_reasoning_fabric",
            "eden_external_ecosystem",
            "ecosystem_participation_contract",
            "ecosystem_interop_matrix",
            "ecosystem_certification_ladder",
            "ecosystem_onboarding_runbook",
            "ecosystem_governance_model",
            "ecosystem_benchmark_exchange",
            "artifact_api_catalog",
            "artifact_api_contracts",
            "artifact_api_runtime",
            "runtime_state_api_catalog",
            "runtime_state_api_contracts",
            "runtime_state_api_openapi",
            "runtime_state_api_runtime",
            "operational_api_catalog",
            "operational_api_contracts",
            "operational_api_openapi",
            "operational_api_runtime",
            "operational_action_contracts",
            "memory_eval",
            "world_eval",
        ];
        let mut artifacts = Vec::new();
        let mut suite_hash = String::from("missing");
        if include_suite {
            let path = dir.join("external_validation_suite.json");
            let cases: Vec<_> = (0..60)
                .map(|idx| {
                    serde_json::json!({
                        "id": format!("case_{idx}"),
                        "suite": "fixture_suite",
                        "criterion": "fixture criterion",
                        "evidence_token": "[FIXTURE]"
                    })
                })
                .collect();
            let suite = serde_json::json!({
                "schema": "garm-external-validation-suite-v1",
                "suite_version": "heldout-local-v2",
                "cases": cases
            });
            let body = serde_json::to_string_pretty(&suite).unwrap();
            suite_hash = format!("{:016x}", fnv64(body.as_bytes()));
            std::fs::write(&path, &body).unwrap();
            artifacts.push(serde_json::json!({
                "name": "external_validation_suite",
                "path": path.to_string_lossy(),
                "present": true,
                "bytes": body.len(),
                "fnv64": suite_hash,
            }));
        }
        let result_path = dir.join("external_validation_result.json");
        let result_cases: Vec<_> = (0..60)
            .map(|idx| {
                serde_json::json!({
                    "id": format!("case_{idx}"),
                    "suite": "fixture_suite",
                    "criterion": "fixture criterion",
                    "evidence_token": "[FIXTURE]",
                    "passed": true
                })
            })
            .collect();
        let result = serde_json::json!({
            "schema": "garm-external-validation-result-v1",
            "suite_version": "heldout-local-v2",
            "suite_fnv64": suite_hash,
            "claim_allowed": false,
            "agi_claim": false,
            "passed": 60,
            "total": 60,
            "cases": result_cases
        });
        let result_body = serde_json::to_string_pretty(&result).unwrap();
        std::fs::write(&result_path, &result_body).unwrap();
        artifacts.push(serde_json::json!({
            "name": "external_validation_result",
            "path": result_path.to_string_lossy(),
            "present": true,
            "bytes": result_body.len(),
            "fnv64": format!("{:016x}", fnv64(result_body.as_bytes())),
        }));
        for name in artifact_names {
            let path = dir.join(format!("{name}.json"));
            let body = serde_json::json!({
                "schema": name,
                "claim_allowed": false,
                "agi_claim": false,
            })
            .to_string();
            std::fs::write(&path, &body).unwrap();
            artifacts.push(serde_json::json!({
                "name": name,
                "path": path.to_string_lossy(),
                "present": true,
                "bytes": body.len(),
                "fnv64": format!("{:016x}", fnv64(body.as_bytes())),
            }));
        }
        let package = serde_json::json!({
            "schema": "garm-readiness-package-v1",
            "package_version": "v2",
            "claim_allowed": claim_allowed,
            "agi_claim": claim_allowed,
            "artifacts": artifacts,
        });
        std::fs::write(
            dir.join("readiness_package.json"),
            serde_json::to_string_pretty(&package).unwrap(),
        )
        .unwrap();
    }
}
