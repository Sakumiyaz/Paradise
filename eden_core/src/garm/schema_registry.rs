use crate::eden_garm::{reproducible_package, runtime_state_api, state_paths};
use std::collections::BTreeSet;

pub fn run() -> String {
    let value = registry_value();
    let total = value
        .get("total")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    write_json(state_paths::schema_registry_path(), value);
    format!(
        "[SCHEMA-REGISTRY] schemas={} claim_allowed=false path={}\n",
        total,
        state_paths::schema_registry_path()
    )
}

pub fn catalog_json() -> String {
    std::fs::read_to_string(state_paths::schema_registry_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&registry_value()).unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn schema_json(name: &str) -> String {
    let needle = name.trim();
    let registry = registry_value();
    let records = registry
        .get("schemas")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let record = records.into_iter().find(|record| {
        record
            .get("name")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value == needle)
            || record
                .get("schema")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value == needle)
    });
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "eden-schema-registry-record-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "query": needle,
        "found": record.is_some(),
        "record": record,
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

fn registry_value() -> serde_json::Value {
    let mut seen = BTreeSet::new();
    let mut records = Vec::new();

    for record in curated_operational_schemas() {
        push_record(&mut records, &mut seen, record);
    }

    for spec in runtime_state_api::state_specs() {
        let present = std::fs::metadata(&spec.path).is_ok();
        push_record(
            &mut records,
            &mut seen,
            serde_json::json!({
                "name": spec.name,
                "schema": spec.schema_hint,
                "domain": spec.domain,
                "source": "runtime_state_api",
                "path": spec.path,
                "present": present,
                "read_endpoint": format!("/api/runtime/state?name={}", spec.name),
                "stability": schema_stability(spec.schema_hint),
            }),
        );
    }

    for spec in reproducible_package::artifact_specs() {
        let present = std::fs::metadata(&spec.path).is_ok();
        let schema =
            schema_from_file(&spec.path).unwrap_or_else(|| "artifact-schema-unseen".to_string());
        push_record(
            &mut records,
            &mut seen,
            serde_json::json!({
                "name": spec.name,
                "schema": schema,
                "domain": "release_artifact",
                "source": "reproducible_package",
                "path": spec.path,
                "present": present,
                "read_endpoint": format!("/api/artifact?name={}", spec.name),
                "stability": schema_stability(&schema),
            }),
        );
    }

    records.sort_by(|a, b| {
        let left = a
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let right = b
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        left.cmp(right)
    });

    serde_json::json!({
        "schema": "eden-schema-registry-v1",
        "artifact": "schema_registry",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "global_executive_workspace_core",
        "purpose": "Make EDEN runtime JSON contracts discoverable and version-auditable from one local registry.",
        "total": records.len(),
        "schemas": records,
        "invariants": [
            "schema registry is read-only over HTTP",
            "mutations remain command-routed through GEWC",
            "missing artifacts are listed as contracts, not hidden",
            "registry evidence is not an AGI capability claim"
        ],
    })
}

fn curated_operational_schemas() -> Vec<serde_json::Value> {
    vec![
        curated(
            "operational_status",
            "eden-operational-status-v1",
            "/api/operational/status",
        ),
        curated(
            "operational_permissions",
            "eden-operational-permissions-v1",
            "/api/operational/permissions",
        ),
        curated(
            "operational_replay_index",
            "eden-gewc-replay-index-v1",
            "/api/operational/replay",
        ),
        curated(
            "operational_replay_decision",
            "eden-gewc-decision-replay-v1",
            "/api/operational/replay?decision_id=<id>",
        ),
        curated(
            "operational_recovery_plan",
            "eden-operational-recovery-plan-v1",
            "/api/operational/recovery",
        ),
        curated(
            "operational_demo_suite",
            "eden-operational-demo-suite-v1",
            "/api/operational/demos",
        ),
        curated(
            "schema_registry",
            "eden-schema-registry-v1",
            "/api/operational/schemas",
        ),
        curated(
            "training_capability_report",
            "eden.training.capability_report.v1",
            "/api/artifact?name=training_capability_report",
        ),
        curated(
            "training_capability_evidence",
            "eden.garm.training_evidence.v1",
            "/api/artifact?name=training_capability_evidence",
        ),
        curated(
            "model_adapter_runtime",
            "eden.model_adapter_runtime.v1",
            "/api/artifact?name=model_adapter_runtime",
        ),
        curated(
            "model_checkpoint_manifest",
            "eden.model_checkpoint_manifest.v1",
            "/api/artifact?name=model_checkpoint_manifest",
        ),
        curated(
            "paradise_checkpoint_registry_admission",
            "paradise.checkpoint_registry_admission.v1",
            "/api/artifact?name=paradise_checkpoint_registry_admission",
        ),
        curated(
            "paradise_checkpoint_admission_dry_run",
            crate::eden_garm::model_runtime::PARADISE_CHECKPOINT_ADMISSION_DRY_RUN_SCHEMA,
            "/api/artifact?name=paradise_checkpoint_admission_dry_run",
        ),
        curated(
            "paradise_checkpoint_admission_gate",
            crate::eden_garm::model_runtime::PARADISE_CHECKPOINT_ADMISSION_GATE_SCHEMA,
            "/api/artifact?name=paradise_checkpoint_admission_gate",
        ),
        curated(
            "paradise_non_gpu_readiness",
            "paradise.non_gpu_readiness.v1",
            "/api/artifact?name=paradise_non_gpu_readiness",
        ),
        curated(
            "paradise_dataset_manifest",
            "paradise.dataset_manifest.v1",
            "/api/artifact?name=paradise_dataset_manifest",
        ),
        curated(
            "paradise_module_semantic_eval",
            "paradise.module_semantic_eval.v1",
            "/api/artifact?name=paradise_module_semantic_eval",
        ),
        curated(
            "paradise_strong_eval",
            "paradise.strong_eval.v1",
            "/api/artifact?name=paradise_strong_eval",
        ),
        curated(
            "paradise_checkpoint_evidence_review",
            "paradise.checkpoint_evidence_review.v1",
            "/api/artifact?name=paradise_checkpoint_evidence_review",
        ),
        curated(
            "paradise_external_validation_package",
            "paradise.external_validation_package.v1",
            "/api/artifact?name=paradise_external_validation_package",
        ),
        curated(
            "training_harness_report",
            "eden.training_harness.v1",
            "/api/artifact?name=training_harness_report",
        ),
        curated(
            "model_governance_report",
            "eden.model_governance.v1",
            "/api/artifact?name=model_governance_report",
        ),
        curated(
            "eden_70b_modular_target",
            "eden.modular_70b.target.v1",
            "/api/artifact?name=eden_70b_modular_target",
        ),
        curated(
            "eden_70b_module_router",
            "eden.modular_70b.router.v1",
            "/api/artifact?name=eden_70b_module_router",
        ),
        curated(
            "eden_70b_dataset_manifest",
            "eden.modular_70b.dataset_manifest.v1",
            "/api/artifact?name=eden_70b_dataset_manifest",
        ),
        curated(
            "eden_70b_launcher_manifest",
            "eden.modular_70b.launcher_manifest.v1",
            "/api/artifact?name=eden_70b_launcher_manifest",
        ),
        curated(
            "eden_70b_checkpoint_admission",
            "eden.modular_70b.checkpoint_admission.v1",
            "/api/artifact?name=eden_70b_checkpoint_admission",
        ),
        curated(
            "eden_70b_inference_runtime",
            "eden.modular_70b.inference_runtime.v1",
            "/api/artifact?name=eden_70b_inference_runtime",
        ),
        curated(
            "eden_70b_operational_demo",
            "eden.modular_70b.operational_demo.v1",
            "/api/artifact?name=eden_70b_operational_demo",
        ),
        curated(
            "eden_70b_operational_gate",
            "eden.modular_70b.operational_gate.v1",
            "/api/artifact?name=eden_70b_operational_gate",
        ),
        curated(
            "first_model_card",
            "eden.first_model.card.v1",
            "/api/artifact?name=first_model_card",
        ),
        curated(
            "first_model_training_plan",
            "eden.first_model.training_plan.v1",
            "/api/artifact?name=first_model_training_plan",
        ),
        curated(
            "first_model_readiness",
            "eden.first_model.readiness.v1",
            "/api/artifact?name=first_model_readiness",
        ),
        curated(
            "elcp_objective_spec",
            "eden.elcp.objective_spec.v1",
            "/api/artifact?name=elcp_objective_spec",
        ),
        curated(
            "elcp_transition_dataset",
            "eden.elcp.transition_dataset.v1",
            "/api/artifact?name=elcp_transition_dataset",
        ),
        curated(
            "elcp_training_plan",
            "eden.elcp.training_plan.v1",
            "/api/artifact?name=elcp_training_plan",
        ),
        curated(
            "elcp_admission_gate",
            "eden.elcp.admission_gate.v1",
            "/api/artifact?name=elcp_admission_gate",
        ),
        curated(
            "elcp_trace_quality_gate",
            "eden.elcp.trace_quality_gate.v1",
            "/api/artifact?name=elcp_trace_quality_gate",
        ),
        curated(
            "elcp_replay_eval",
            "eden.elcp.replay_eval.v1",
            "/api/artifact?name=elcp_replay_eval",
        ),
        curated(
            "elcp_dataset_freeze_manifest",
            "eden.elcp.dataset_freeze_manifest.v1",
            "/api/artifact?name=elcp_dataset_freeze_manifest",
        ),
        curated(
            "elcp_metrics_board",
            "eden.elcp.metrics_board.v1",
            "/api/artifact?name=elcp_metrics_board",
        ),
        curated(
            "elcp_4b_readiness_contract",
            "eden.elcp.4b_readiness_contract.v1",
            "/api/artifact?name=elcp_4b_readiness_contract",
        ),
        curated(
            "elcp_readiness",
            "eden.elcp.readiness.v1",
            "/api/artifact?name=elcp_readiness",
        ),
        curated(
            "eden_capable_gate",
            crate::eden_garm::eden_capable::EDEN_CAPABLE_GATE_SCHEMA,
            "/api/artifact?name=eden_capable_gate",
        ),
        curated(
            "eden_cognitive_dataset_manifest",
            crate::eden_garm::eden_capable::EDEN_COGNITIVE_DATASET_MANIFEST_SCHEMA,
            "/api/artifact?name=eden_cognitive_dataset_manifest",
        ),
        curated(
            "eden_structured_output_report",
            crate::eden_garm::eden_capable::EDEN_STRUCTURED_OUTPUT_REPORT_SCHEMA,
            "/api/artifact?name=eden_structured_output_report",
        ),
        curated(
            "eden_checkpoint_registry",
            crate::eden_garm::eden_capable::EDEN_CHECKPOINT_REGISTRY_SCHEMA,
            "/api/artifact?name=eden_checkpoint_registry",
        ),
        curated(
            "eden_sft_elcp_readiness",
            crate::eden_garm::eden_capable::EDEN_SFT_ELCP_READINESS_SCHEMA,
            "/api/artifact?name=eden_sft_elcp_readiness",
        ),
        curated(
            "eden_live_inference_runtime",
            crate::eden_garm::eden_capable::EDEN_LIVE_INFERENCE_RUNTIME_SCHEMA,
            "/api/artifact?name=eden_live_inference_runtime",
        ),
        curated(
            "eden_cognitive_call_contract",
            crate::eden_garm::eden_capable::EDEN_COGNITIVE_CALL_CONTRACT_SCHEMA,
            "/api/artifact?name=eden_cognitive_call_contract",
        ),
        curated(
            "eden_cognitive_dataset_expansion",
            crate::eden_garm::eden_capable::EDEN_COGNITIVE_DATASET_EXPANSION_SCHEMA,
            "/api/artifact?name=eden_cognitive_dataset_expansion",
        ),
        curated(
            "eden_capability_eval_suite",
            crate::eden_garm::eden_capable::EDEN_CAPABILITY_EVAL_SUITE_SCHEMA,
            "/api/artifact?name=eden_capability_eval_suite",
        ),
        curated(
            "eden_sft_elcp_activation_gate",
            crate::eden_garm::eden_capable::EDEN_SFT_ELCP_ACTIVATION_GATE_SCHEMA,
            "/api/artifact?name=eden_sft_elcp_activation_gate",
        ),
        curated(
            "eden_memory_action_loop",
            crate::eden_garm::eden_capable::EDEN_MEMORY_ACTION_LOOP_SCHEMA,
            "/api/artifact?name=eden_memory_action_loop",
        ),
        curated(
            "eden_capable_demo_trace",
            crate::eden_garm::eden_capable::EDEN_CAPABLE_DEMO_TRACE_SCHEMA,
            "/api/artifact?name=eden_capable_demo_trace",
        ),
        curated(
            "eden_capable_operational_gate",
            crate::eden_garm::eden_capable::EDEN_CAPABLE_OPERATIONAL_GATE_SCHEMA,
            "/api/artifact?name=eden_capable_operational_gate",
        ),
        curated(
            "eden_sft_elcp_dataset_v2_manifest",
            crate::eden_garm::eden_learned_capability::EDEN_SFT_ELCP_DATASET_V2_MANIFEST_SCHEMA,
            "/api/artifact?name=eden_sft_elcp_dataset_v2_manifest",
        ),
        curated(
            "eden_sft_elcp_gpu_training_report",
            crate::eden_garm::eden_learned_capability::EDEN_SFT_ELCP_GPU_TRAINING_REPORT_SCHEMA,
            "/api/artifact?name=eden_sft_elcp_gpu_training_report",
        ),
        curated(
            "eden_sft_elcp_prepost_eval",
            crate::eden_garm::eden_learned_capability::EDEN_SFT_ELCP_PREPOST_EVAL_SCHEMA,
            "/api/artifact?name=eden_sft_elcp_prepost_eval",
        ),
        curated(
            "eden_sft_elcp_repeated_inference_eval",
            crate::eden_garm::eden_learned_capability::EDEN_SFT_ELCP_REPEATED_INFERENCE_EVAL_SCHEMA,
            "/api/artifact?name=eden_sft_elcp_repeated_inference_eval",
        ),
        curated(
            "eden_sft_elcp_checkpoint_admission_review",
            crate::eden_garm::eden_learned_capability::EDEN_SFT_ELCP_CHECKPOINT_ADMISSION_REVIEW_SCHEMA,
            "/api/artifact?name=eden_sft_elcp_checkpoint_admission_review",
        ),
        curated(
            "eden_sft_elcp_operational_demo",
            crate::eden_garm::eden_learned_capability::EDEN_SFT_ELCP_OPERATIONAL_DEMO_SCHEMA,
            "/api/artifact?name=eden_sft_elcp_operational_demo",
        ),
        curated(
            "eden_external_tests_ci_gate",
            crate::eden_garm::eden_learned_capability::EDEN_EXTERNAL_TESTS_CI_GATE_SCHEMA,
            "/api/artifact?name=eden_external_tests_ci_gate",
        ),
        curated(
            "eden_learned_capability_gate",
            crate::eden_garm::eden_learned_capability::EDEN_LEARNED_CAPABILITY_GATE_SCHEMA,
            "/api/artifact?name=eden_learned_capability_gate",
        ),
        curated(
            "eden_real_capability_dataset_manifest",
            crate::eden_garm::eden_real_capability::EDEN_REAL_CAPABILITY_DATASET_MANIFEST_SCHEMA,
            "/api/artifact?name=eden_real_capability_dataset_manifest",
        ),
        curated(
            "eden_real_capability_7b_training",
            crate::eden_garm::eden_real_capability::EDEN_REAL_CAPABILITY_7B_TRAINING_SCHEMA,
            "/api/artifact?name=eden_real_capability_7b_training",
        ),
        curated(
            "eden_real_capability_inference_bridge",
            crate::eden_garm::eden_real_capability::EDEN_REAL_CAPABILITY_INFERENCE_BRIDGE_SCHEMA,
            "/api/artifact?name=eden_real_capability_inference_bridge",
        ),
        curated(
            "eden_real_capability_operational_eval",
            crate::eden_garm::eden_real_capability::EDEN_REAL_CAPABILITY_OPERATIONAL_EVAL_SCHEMA,
            "/api/artifact?name=eden_real_capability_operational_eval",
        ),
        curated(
            "eden_real_capability_checkpoint_decision",
            crate::eden_garm::eden_real_capability::EDEN_REAL_CAPABILITY_CHECKPOINT_DECISION_SCHEMA,
            "/api/artifact?name=eden_real_capability_checkpoint_decision",
        ),
        curated(
            "eden_real_capability_demo",
            crate::eden_garm::eden_real_capability::EDEN_REAL_CAPABILITY_DEMO_SCHEMA,
            "/api/artifact?name=eden_real_capability_demo",
        ),
        curated(
            "eden_real_capability_scaling_ladder",
            crate::eden_garm::eden_real_capability::EDEN_REAL_CAPABILITY_SCALING_LADDER_SCHEMA,
            "/api/artifact?name=eden_real_capability_scaling_ladder",
        ),
        curated(
            "eden_real_capability_gate",
            crate::eden_garm::eden_real_capability::EDEN_REAL_CAPABILITY_GATE_SCHEMA,
            "/api/artifact?name=eden_real_capability_gate",
        ),
        curated(
            "eden_v01_dataset_manifest",
            crate::eden_garm::eden_real_capability::EDEN_V01_DATASET_MANIFEST_SCHEMA,
            "/api/artifact?name=eden_v01_dataset_manifest",
        ),
        curated(
            "eden_v01_semantic_eval",
            crate::eden_garm::eden_real_capability::EDEN_V01_SEMANTIC_EVAL_SCHEMA,
            "/api/artifact?name=eden_v01_semantic_eval",
        ),
        curated(
            "eden_v01_training_beyond_pilot",
            crate::eden_garm::eden_real_capability::EDEN_V01_TRAINING_BEYOND_PILOT_SCHEMA,
            "/api/artifact?name=eden_v01_training_beyond_pilot",
        ),
        curated(
            "eden_v01_native_inference_runtime",
            crate::eden_garm::eden_real_capability::EDEN_V01_NATIVE_INFERENCE_RUNTIME_SCHEMA,
            "/api/artifact?name=eden_v01_native_inference_runtime",
        ),
        curated(
            "eden_v01_operational_demo",
            crate::eden_garm::eden_real_capability::EDEN_V01_OPERATIONAL_DEMO_SCHEMA,
            "/api/artifact?name=eden_v01_operational_demo",
        ),
        curated(
            "eden_v01_checkpoint_admission",
            crate::eden_garm::eden_real_capability::EDEN_V01_CHECKPOINT_ADMISSION_SCHEMA,
            "/api/artifact?name=eden_v01_checkpoint_admission",
        ),
        curated(
            "eden_v01_scaling_plan",
            crate::eden_garm::eden_real_capability::EDEN_V01_SCALING_PLAN_SCHEMA,
            "/api/artifact?name=eden_v01_scaling_plan",
        ),
        curated(
            "eden_v01_gpu_workspace_hygiene",
            crate::eden_garm::eden_real_capability::EDEN_V01_GPU_WORKSPACE_HYGIENE_SCHEMA,
            "/api/artifact?name=eden_v01_gpu_workspace_hygiene",
        ),
        curated(
            "eden_v01_capability_gate",
            crate::eden_garm::eden_real_capability::EDEN_V01_CAPABILITY_GATE_SCHEMA,
            "/api/artifact?name=eden_v01_capability_gate",
        ),
        curated(
            "eden_v02_stability_corpus_manifest",
            crate::eden_garm::eden_real_capability::EDEN_V02_STABILITY_CORPUS_MANIFEST_SCHEMA,
            "/api/artifact?name=eden_v02_stability_corpus_manifest",
        ),
        curated(
            "eden_v02_stability_eval",
            crate::eden_garm::eden_real_capability::EDEN_V02_STABILITY_EVAL_SCHEMA,
            "/api/artifact?name=eden_v02_stability_eval",
        ),
        curated(
            "eden_v02_checkpoint_comparison",
            crate::eden_garm::eden_real_capability::EDEN_V02_CHECKPOINT_COMPARISON_SCHEMA,
            "/api/artifact?name=eden_v02_checkpoint_comparison",
        ),
        curated(
            "eden_v02_adversarial_eval",
            crate::eden_garm::eden_real_capability::EDEN_V02_ADVERSARIAL_EVAL_SCHEMA,
            "/api/artifact?name=eden_v02_adversarial_eval",
        ),
        curated(
            "eden_v02_rollback_drill",
            crate::eden_garm::eden_real_capability::EDEN_V02_ROLLBACK_DRILL_SCHEMA,
            "/api/artifact?name=eden_v02_rollback_drill",
        ),
        curated(
            "eden_v02_model_card_internal",
            crate::eden_garm::eden_real_capability::EDEN_V02_MODEL_CARD_INTERNAL_SCHEMA,
            "/api/artifact?name=eden_v02_model_card_internal",
        ),
        curated(
            "eden_v02_checkpoint_storage",
            crate::eden_garm::eden_real_capability::EDEN_V02_CHECKPOINT_STORAGE_SCHEMA,
            "/api/artifact?name=eden_v02_checkpoint_storage",
        ),
        curated(
            "eden_v02_native_inference_service",
            crate::eden_garm::eden_real_capability::EDEN_V02_NATIVE_INFERENCE_SERVICE_SCHEMA,
            "/api/artifact?name=eden_v02_native_inference_service",
        ),
        curated(
            "eden_v02_stability_demo",
            crate::eden_garm::eden_real_capability::EDEN_V02_STABILITY_DEMO_SCHEMA,
            "/api/artifact?name=eden_v02_stability_demo",
        ),
        curated(
            "eden_v02_stability_gate",
            crate::eden_garm::eden_real_capability::EDEN_V02_STABILITY_GATE_SCHEMA,
            "/api/artifact?name=eden_v02_stability_gate",
        ),
        curated(
            "paradise_worldcell_runtime",
            crate::eden_garm::paradise_worldcell::PARADISE_WORLDCELL_SCHEMA,
            "/api/artifact?name=paradise_worldcell_runtime",
        ),
        curated(
            "paradise_worldcell_sessions",
            crate::eden_garm::paradise_worldcell::PARADISE_WORLDCELL_SESSION_SCHEMA,
            "/api/paradise/sessions",
        ),
        curated(
            "runtime_spine",
            crate::eden_garm::runtime_spine::RUNTIME_SPINE_SCHEMA,
            "/api/operational/spine",
        ),
        curated(
            "runtime_event_bus_state",
            crate::eden_garm::runtime_spine::EVENT_BUS_SCHEMA,
            "/api/operational/events",
        ),
        curated(
            "runtime_global_state",
            crate::eden_garm::runtime_spine::GLOBAL_STATE_SCHEMA,
            "/api/operational/global-state",
        ),
        curated(
            "runtime_replay_spine",
            crate::eden_garm::runtime_spine::REPLAY_SPINE_SCHEMA,
            "/api/operational/replay-spine",
        ),
        curated(
            "runtime_spine_verification",
            crate::eden_garm::runtime_spine::SPINE_VERIFICATION_SCHEMA,
            "/api/operational/spine-verification",
        ),
        curated(
            "runtime_guard_decisions",
            crate::eden_garm::runtime_spine::GUARD_DECISION_SCHEMA,
            "/api/runtime/state?name=runtime_guard_decisions",
        ),
        curated(
            "runtime_spine_enforcement",
            crate::eden_garm::runtime_spine::ENFORCEMENT_SCHEMA,
            "/api/operational/spine-enforcement",
        ),
        curated(
            "runtime_workflow_risk",
            crate::eden_garm::runtime_spine::WORKFLOW_RISK_SCHEMA,
            "/api/operational/workflow-risk",
        ),
        curated(
            "runtime_circuit_breakers",
            crate::eden_garm::runtime_spine::CIRCUIT_BREAKERS_SCHEMA,
            "/api/operational/circuit-breakers",
        ),
        curated(
            "runtime_replay_reconstruction",
            crate::eden_garm::runtime_spine::REPLAY_RECONSTRUCTION_SCHEMA,
            "/api/operational/spine-replay",
        ),
    ]
}

fn curated(name: &'static str, schema: &'static str, endpoint: &'static str) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "schema": schema,
        "domain": "operational_api",
        "source": "curated_operational_contract",
        "path": state_paths::schema_registry_path(),
        "present": true,
        "read_endpoint": endpoint,
        "stability": schema_stability(schema),
    })
}

fn push_record(
    records: &mut Vec<serde_json::Value>,
    seen: &mut BTreeSet<String>,
    record: serde_json::Value,
) {
    let key = format!(
        "{}|{}",
        record
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default(),
        record
            .get("schema")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
    );
    if seen.insert(key) {
        records.push(record);
    }
}

fn schema_from_file(path: &str) -> Option<String> {
    let body = std::fs::read_to_string(path).ok()?;
    let candidate = if body.trim_start().starts_with('{') {
        body
    } else {
        body.lines().next().unwrap_or_default().to_string()
    };
    let value = serde_json::from_str::<serde_json::Value>(&candidate).ok()?;
    value
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn schema_stability(schema: &str) -> &'static str {
    if schema.ends_with("-v1") || schema.contains("-v1-") {
        "versioned_v1"
    } else if schema.contains("state") || schema.contains("artifact") {
        "legacy_or_state_hint"
    } else {
        "unversioned_or_not_yet_generated"
    }
}

fn write_json(path: String, record: serde_json::Value) {
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_registry_lists_operational_contracts() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_schema_registry_test_{}",
            std::process::id()
        )));

        let out = run();

        assert!(out.contains("[SCHEMA-REGISTRY]"));
        assert!(catalog_json().contains("eden-schema-registry-v1"));
        assert!(catalog_json().contains("eden-operational-status-v1"));
        assert!(schema_json("operational_status").contains("\"found\": true"));
        assert!(schema_json("eden-operational-status-v1").contains("\"found\": true"));
    }
}
