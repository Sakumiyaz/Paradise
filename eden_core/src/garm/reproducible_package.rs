use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct ReproducibleArtifactSpec {
    pub name: &'static str,
    pub path: String,
}

pub fn artifact_specs() -> Vec<ReproducibleArtifactSpec> {
    vec![
        artifact("garm_report", state_paths::garm_report_path()),
        artifact(
            "garm_report_history",
            state_paths::garm_report_history_path(),
        ),
        artifact("garm_export", state_paths::garm_export_path()),
        artifact(
            "external_validation_manifest",
            state_paths::external_validation_manifest_path(),
        ),
        artifact(
            "external_validation_result",
            state_paths::external_validation_result_path(),
        ),
        artifact(
            "external_validation_suite",
            state_paths::external_validation_suite_path(),
        ),
        artifact("action_evidence", state_paths::action_evidence_path()),
        artifact("goal_scheduler", state_paths::goal_scheduler_state_path()),
        artifact("evaluation_loop", state_paths::evaluation_loop_state_path()),
        artifact("learning_ledger", state_paths::learning_ledger_state_path()),
        artifact(
            "provenance_ledger",
            state_paths::provenance_ledger_state_path(),
        ),
        artifact(
            "uncertainty_ledger",
            state_paths::uncertainty_ledger_state_path(),
        ),
        artifact("policy_guard", state_paths::policy_guard_state_path()),
        artifact(
            "competence_benchmark",
            state_paths::competence_benchmark_state_path(),
        ),
        artifact("memory_eval", state_paths::memory_eval_path()),
        artifact("world_eval", state_paths::world_eval_path()),
        artifact(
            "capability_registry",
            state_paths::capability_registry_path(),
        ),
        artifact(
            "cognitive_architecture",
            state_paths::cognitive_architecture_path(),
        ),
        artifact("embodied_grounding", state_paths::embodied_grounding_path()),
        artifact(
            "neural_architecture",
            state_paths::neural_architecture_path(),
        ),
        artifact(
            "symbolic_architecture",
            state_paths::symbolic_architecture_path(),
        ),
        artifact(
            "self_improvement_architecture",
            state_paths::self_improvement_architecture_path(),
        ),
        artifact(
            "safety_control_architecture",
            state_paths::safety_control_architecture_path(),
        ),
        artifact(
            "foundation_model_architecture",
            state_paths::foundation_model_architecture_path(),
        ),
        artifact(
            "multimodal_model_architecture",
            state_paths::multimodal_model_architecture_path(),
        ),
        artifact(
            "llm_agent_architecture",
            state_paths::llm_agent_architecture_path(),
        ),
        artifact(
            "probabilistic_programming_architecture",
            state_paths::probabilistic_programming_architecture_path(),
        ),
        artifact(
            "hierarchical_rl_architecture",
            state_paths::hierarchical_rl_architecture_path(),
        ),
        artifact(
            "cognitive_robotics_architecture",
            state_paths::cognitive_robotics_architecture_path(),
        ),
        artifact("vla_architecture", state_paths::vla_architecture_path()),
        artifact(
            "sim_to_real_architecture",
            state_paths::sim_to_real_architecture_path(),
        ),
        artifact(
            "open_ended_evolution_architecture",
            state_paths::open_ended_evolution_architecture_path(),
        ),
        artifact(
            "developmental_robotics_architecture",
            state_paths::developmental_robotics_architecture_path(),
        ),
        artifact(
            "whole_brain_neurocognitive_architecture",
            state_paths::whole_brain_neurocognitive_architecture_path(),
        ),
        artifact(
            "neuromorphic_spiking_architecture",
            state_paths::neuromorphic_spiking_architecture_path(),
        ),
        artifact(
            "paradigm_architecture_map",
            state_paths::paradigm_architecture_map_path(),
        ),
        artifact(
            "paradigm_architecture_technique_map",
            state_paths::paradigm_architecture_technique_map_path(),
        ),
        artifact(
            "neuro_symbolic_paradigm",
            state_paths::neuro_symbolic_paradigm_path(),
        ),
        artifact(
            "universal_formal_paradigm",
            state_paths::universal_formal_paradigm_path(),
        ),
        artifact(
            "active_inference_paradigm",
            state_paths::active_inference_paradigm_path(),
        ),
        artifact(
            "ecological_systemic_paradigm",
            state_paths::ecological_systemic_paradigm_path(),
        ),
        artifact(
            "computational_programmatic_paradigm",
            state_paths::computational_programmatic_paradigm_path(),
        ),
        artifact(
            "affective_motivational_paradigm",
            state_paths::affective_motivational_paradigm_path(),
        ),
        artifact(
            "human_in_the_loop_paradigm",
            state_paths::human_in_the_loop_paradigm_path(),
        ),
        artifact(
            "emergence_metrics_paradigm",
            state_paths::emergence_metrics_paradigm_path(),
        ),
        artifact(
            "integration_governance_architecture",
            state_paths::integration_governance_architecture_path(),
        ),
        artifact(
            "global_executive_workspace_core",
            state_paths::global_executive_workspace_core_path(),
        ),
        artifact(
            "global_executive_workspace_runtime",
            state_paths::global_executive_workspace_runtime_path(),
        ),
        artifact(
            "global_executive_workspace_runtime_state",
            state_paths::global_executive_workspace_runtime_state_path(),
        ),
        artifact(
            "gewc_operational_benchmark",
            state_paths::gewc_operational_benchmark_path(),
        ),
        artifact(
            "gewc_runtime_safety_report",
            state_paths::gewc_runtime_safety_report_path(),
        ),
        artifact(
            "gewc_long_run_stability",
            state_paths::gewc_long_run_stability_path(),
        ),
        artifact(
            "capability_reality_eval",
            state_paths::capability_reality_eval_path(),
        ),
        artifact(
            "capability_reality_matrix",
            state_paths::capability_reality_matrix_path(),
        ),
        artifact(
            "lmm_training_dependency_report",
            state_paths::lmm_training_dependency_report_path(),
        ),
        artifact(
            "training_capability_report",
            state_paths::training_capability_report_path(),
        ),
        artifact(
            "training_capability_report_markdown",
            state_paths::training_capability_markdown_report_path(),
        ),
        artifact(
            "training_capability_evidence",
            state_paths::training_capability_evidence_path(),
        ),
        artifact(
            "megatron_7b_training_evidence",
            state_paths::megatron_7b_training_evidence_path(),
        ),
        artifact(
            "megatron_7b_model_adapter",
            state_paths::megatron_7b_model_adapter_path(),
        ),
        artifact(
            "megatron_7b_inference_report",
            state_paths::megatron_7b_inference_report_path(),
        ),
        artifact(
            "megatron_7b_capability_report",
            state_paths::megatron_7b_capability_report_path(),
        ),
        artifact(
            "megatron_7b_admission_gate",
            state_paths::megatron_7b_admission_gate_path(),
        ),
        artifact(
            "model_adapter_runtime",
            state_paths::model_adapter_runtime_path(),
        ),
        artifact(
            "model_checkpoint_manifest",
            state_paths::model_checkpoint_manifest_path(),
        ),
        artifact(
            "training_harness_report",
            state_paths::training_harness_report_path(),
        ),
        artifact(
            "model_governance_report",
            state_paths::model_governance_report_path(),
        ),
        artifact("first_model_card", state_paths::first_model_card_path()),
        artifact(
            "first_model_training_plan",
            state_paths::first_model_training_plan_path(),
        ),
        artifact(
            "first_model_readiness",
            state_paths::first_model_readiness_path(),
        ),
        artifact(
            "elcp_objective_spec",
            state_paths::elcp_objective_spec_path(),
        ),
        artifact(
            "elcp_transition_dataset",
            state_paths::elcp_transition_dataset_path(),
        ),
        artifact("elcp_training_plan", state_paths::elcp_training_plan_path()),
        artifact(
            "elcp_admission_gate",
            state_paths::elcp_admission_gate_path(),
        ),
        artifact(
            "elcp_trace_quality_gate",
            state_paths::elcp_trace_quality_gate_path(),
        ),
        artifact("elcp_replay_eval", state_paths::elcp_replay_eval_path()),
        artifact(
            "elcp_dataset_freeze_manifest",
            state_paths::elcp_dataset_freeze_manifest_path(),
        ),
        artifact("elcp_metrics_board", state_paths::elcp_metrics_board_path()),
        artifact(
            "elcp_4b_readiness_contract",
            state_paths::elcp_4b_readiness_contract_path(),
        ),
        artifact("elcp_readiness", state_paths::elcp_readiness_path()),
        artifact("gewc_trace_spec", state_paths::gewc_trace_spec_path()),
        artifact(
            "capability_reality_matrix_v2",
            state_paths::capability_reality_matrix_v2_path(),
        ),
        artifact(
            "cognitive_task_suite",
            state_paths::cognitive_task_suite_path(),
        ),
        artifact(
            "eden_agent_sdk_contract",
            state_paths::eden_agent_sdk_contract_path(),
        ),
        artifact(
            "model_adapter_layer",
            state_paths::model_adapter_layer_path(),
        ),
        artifact("reproducible_demos", state_paths::reproducible_demos_path()),
        artifact(
            "architecture_advantage_eval",
            state_paths::architecture_advantage_eval_path(),
        ),
        artifact(
            "paradise_worldcell_runtime",
            state_paths::paradise_worldcell_runtime_path(),
        ),
        artifact(
            "paradise_worldcell_sessions",
            state_paths::paradise_worldcell_sessions_path(),
        ),
        artifact("runtime_spine", state_paths::runtime_spine_path()),
        artifact(
            "runtime_internal_contracts",
            state_paths::runtime_internal_contracts_path(),
        ),
        artifact("runtime_event_bus", state_paths::runtime_event_bus_path()),
        artifact(
            "runtime_event_bus_state",
            state_paths::runtime_event_bus_state_path(),
        ),
        artifact(
            "runtime_global_state",
            state_paths::runtime_global_state_path(),
        ),
        artifact(
            "runtime_global_state_log",
            state_paths::runtime_global_state_log_path(),
        ),
        artifact(
            "runtime_replay_spine",
            state_paths::runtime_replay_spine_path(),
        ),
        artifact(
            "runtime_spine_verification",
            state_paths::runtime_spine_verification_path(),
        ),
        artifact(
            "runtime_guard_decisions",
            state_paths::runtime_guard_decisions_path(),
        ),
        artifact(
            "runtime_spine_enforcement",
            state_paths::runtime_spine_enforcement_path(),
        ),
        artifact(
            "runtime_workflow_risk",
            state_paths::runtime_workflow_risk_path(),
        ),
        artifact(
            "runtime_circuit_breakers",
            state_paths::runtime_circuit_breakers_path(),
        ),
        artifact(
            "runtime_replay_reconstruction",
            state_paths::runtime_replay_reconstruction_path(),
        ),
        artifact(
            "runtime_security_gates",
            state_paths::runtime_security_gates_path(),
        ),
        artifact(
            "runtime_model_router_contract",
            state_paths::runtime_model_router_contract_path(),
        ),
        artifact(
            "runtime_memory_fabric_contract",
            state_paths::runtime_memory_fabric_contract_path(),
        ),
        artifact(
            "runtime_world_simulation_contract",
            state_paths::runtime_world_simulation_contract_path(),
        ),
        artifact(
            "runtime_multiagent_contract",
            state_paths::runtime_multiagent_contract_path(),
        ),
        artifact("eden_praxis_nexus", state_paths::eden_praxis_nexus_path()),
        artifact("praxis_primitives", state_paths::praxis_primitives_path()),
        artifact("praxis_blocks", state_paths::praxis_blocks_path()),
        artifact("praxis_space", state_paths::praxis_space_path()),
        artifact("praxis_rules", state_paths::praxis_rules_path()),
        artifact(
            "praxis_trace_semantics",
            state_paths::praxis_trace_semantics_path(),
        ),
        artifact("praxis_reasoner", state_paths::praxis_reasoner_path()),
        artifact("praxis_bench", state_paths::praxis_bench_path()),
        artifact("eden_locus_layer", state_paths::eden_locus_layer_path()),
        artifact(
            "locus_authority_model",
            state_paths::locus_authority_model_path(),
        ),
        artifact(
            "locus_evidence_vault",
            state_paths::locus_evidence_vault_path(),
        ),
        artifact(
            "locus_permission_matrix",
            state_paths::locus_permission_matrix_path(),
        ),
        artifact(
            "locus_context_packet",
            state_paths::locus_context_packet_path(),
        ),
        artifact(
            "locus_operator_timeline",
            state_paths::locus_operator_timeline_path(),
        ),
        artifact(
            "eden_operator_forge",
            state_paths::eden_operator_forge_path(),
        ),
        artifact(
            "operator_primitive_basis",
            state_paths::operator_primitive_basis_path(),
        ),
        artifact(
            "operator_expression_graphs",
            state_paths::operator_expression_graphs_path(),
        ),
        artifact(
            "operator_verification_report",
            state_paths::operator_verification_report_path(),
        ),
        artifact(
            "operator_model_registry",
            state_paths::operator_model_registry_path(),
        ),
        artifact(
            "eden_sovereign_cognition",
            state_paths::eden_sovereign_cognition_path(),
        ),
        artifact(
            "sovereign_sector_wins",
            state_paths::sovereign_sector_wins_path(),
        ),
        artifact(
            "praxis_calculus_formalism",
            state_paths::praxis_calculus_formalism_path(),
        ),
        artifact(
            "cognitive_contract_language",
            state_paths::cognitive_contract_language_path(),
        ),
        artifact(
            "evidence_memory_fabric",
            state_paths::evidence_memory_fabric_path(),
        ),
        artifact(
            "federated_runtime_fabric",
            state_paths::federated_runtime_fabric_path(),
        ),
        artifact(
            "symbolic_reasoning_fabric",
            state_paths::symbolic_reasoning_fabric_path(),
        ),
        artifact(
            "eden_external_ecosystem",
            state_paths::eden_external_ecosystem_path(),
        ),
        artifact(
            "ecosystem_participation_contract",
            state_paths::ecosystem_participation_contract_path(),
        ),
        artifact(
            "ecosystem_interop_matrix",
            state_paths::ecosystem_interop_matrix_path(),
        ),
        artifact(
            "ecosystem_certification_ladder",
            state_paths::ecosystem_certification_ladder_path(),
        ),
        artifact(
            "ecosystem_onboarding_runbook",
            state_paths::ecosystem_onboarding_runbook_path(),
        ),
        artifact(
            "ecosystem_governance_model",
            state_paths::ecosystem_governance_model_path(),
        ),
        artifact(
            "ecosystem_benchmark_exchange",
            state_paths::ecosystem_benchmark_exchange_path(),
        ),
        artifact(
            "artifact_api_catalog",
            state_paths::artifact_api_catalog_path(),
        ),
        artifact(
            "artifact_api_contracts",
            state_paths::artifact_api_contracts_path(),
        ),
        artifact(
            "artifact_api_runtime",
            state_paths::artifact_api_runtime_path(),
        ),
        artifact(
            "runtime_state_api_catalog",
            state_paths::runtime_state_api_catalog_path(),
        ),
        artifact(
            "runtime_state_api_contracts",
            state_paths::runtime_state_api_contracts_path(),
        ),
        artifact(
            "runtime_state_api_openapi",
            state_paths::runtime_state_api_openapi_path(),
        ),
        artifact(
            "runtime_state_api_runtime",
            state_paths::runtime_state_api_runtime_path(),
        ),
        artifact(
            "operational_api_catalog",
            state_paths::operational_api_catalog_path(),
        ),
        artifact(
            "operational_api_contracts",
            state_paths::operational_api_contracts_path(),
        ),
        artifact(
            "operational_api_openapi",
            state_paths::operational_api_openapi_path(),
        ),
        artifact(
            "operational_api_runtime",
            state_paths::operational_api_runtime_path(),
        ),
        artifact(
            "operational_action_contracts",
            state_paths::operational_action_contracts_path(),
        ),
        artifact(
            "operational_contract",
            state_paths::operational_contract_path(),
        ),
        artifact(
            "operational_permissions",
            state_paths::operational_permissions_path(),
        ),
        artifact(
            "operational_permissions_audit",
            state_paths::operational_permissions_audit_path(),
        ),
        artifact(
            "operational_permissions_diff",
            state_paths::operational_permissions_diff_path(),
        ),
        artifact(
            "operational_permissions_history",
            state_paths::operational_permissions_history_path(),
        ),
        artifact(
            "operational_evidence_bundle",
            state_paths::operational_evidence_bundle_path(),
        ),
        artifact(
            "operational_recovery_plan",
            state_paths::operational_recovery_plan_path(),
        ),
        artifact(
            "operational_demo_suite",
            state_paths::operational_demo_suite_path(),
        ),
        artifact("schema_registry", state_paths::schema_registry_path()),
        artifact(
            "operational_runtime_phase",
            state_paths::operational_runtime_phase_path(),
        ),
        artifact(
            "operational_task_runtime",
            state_paths::operational_task_runtime_path(),
        ),
        artifact(
            "operational_action_executor",
            state_paths::operational_action_executor_path(),
        ),
        artifact(
            "operational_lifecycle_controls",
            state_paths::operational_lifecycle_controls_path(),
        ),
        artifact(
            "operational_memory_transactions",
            state_paths::operational_memory_transactions_path(),
        ),
        artifact(
            "cwm_operational_state",
            state_paths::cwm_operational_state_path(),
        ),
        artifact(
            "locus_operator_bridge",
            state_paths::locus_operator_bridge_path(),
        ),
        artifact(
            "governed_agent_runtime",
            state_paths::governed_agent_runtime_path(),
        ),
        artifact(
            "operational_replay_eval",
            state_paths::operational_replay_eval_path(),
        ),
    ]
}

fn artifact(name: &'static str, path: String) -> ReproducibleArtifactSpec {
    ReproducibleArtifactSpec { name, path }
}

pub fn write(readiness_report: &str, gate_report: &str) -> String {
    let artifacts = artifact_specs();
    let artifact_records: Vec<serde_json::Value> = artifacts
        .iter()
        .map(|artifact| {
            let bytes = std::fs::read(&artifact.path).unwrap_or_default();
            serde_json::json!({
                "name": artifact.name,
                "path": artifact.path.as_str(),
                "present": !bytes.is_empty(),
                "bytes": bytes.len(),
                "fnv64": format!("{:016x}", fnv64(&bytes)),
            })
        })
        .collect();
    let package = serde_json::json!({
        "schema": "garm-readiness-package-v1",
        "package_version": "v2",
        "claim_allowed": false,
        "agi_claim": false,
        "state_dir": state_paths::state_dir().to_string_lossy(),
        "expected_commands": [
            "readiness probe",
            "memory eval",
            "world eval",
            "cognitive eval",
            "embodied eval",
            "neural eval",
            "symbolic eval",
            "self improvement eval",
            "frontier architecture eval",
            "paradigm architecture eval",
            "integration governance eval",
            "global executive workspace eval",
            "gewc operational benchmark",
            "capability reality eval",
            "architecture advantage eval",
            "praxis nexus eval",
            "locus eval",
            "locus ingest operator preference :: keep EDEN local-first and no-claim gated",
            "locus context operator permission boundary",
            "operator forge eval",
            "operator forge synth causal risk model for governed action under uncertainty",
            "operator forge verify",
            "operational runtime eval",
            "external ecosystem eval",
            "sovereign cognition eval",
            "runtime state api eval",
            "operational api eval",
            "artifact api eval",
            "readiness external run",
            "capabilities audit",
            "readiness package"
        ],
        "make_targets": [
            "make eden-validate-local",
            "make eden-package",
            "make eden-independent-validate",
            "make eden-release-candidate",
            "make eden-release-check"
        ],
        "readiness_report_fnv64": format!("{:016x}", fnv64(readiness_report.as_bytes())),
        "gate_report_fnv64": format!("{:016x}", fnv64(gate_report.as_bytes())),
        "artifacts": artifact_records,
    });
    let body = serde_json::to_string_pretty(&package).unwrap_or_else(|_| package.to_string());
    let path = state_paths::readiness_package_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(&path, body) {
        Ok(()) => "package_written",
        Err(_) => "package_write_failed",
    };
    format!(
        "[READINESS-PACKAGE] schema=garm-readiness-package-v1 claim_allowed=false write_status={} path={}\n",
        write_status, path
    )
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
