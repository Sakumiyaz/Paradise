use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const DEFAULT_STATE_DIR: &str = "/tmp/eden_garm";

static STATE_DIR: OnceLock<Mutex<PathBuf>> = OnceLock::new();
static TEST_STATE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn test_state_guard() -> std::sync::MutexGuard<'static, ()> {
    TEST_STATE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn state_dir_cell() -> &'static Mutex<PathBuf> {
    STATE_DIR.get_or_init(|| Mutex::new(PathBuf::from(DEFAULT_STATE_DIR)))
}

pub fn set_state_dir(path: impl Into<PathBuf>) {
    if let Ok(mut dir) = state_dir_cell().lock() {
        *dir = path.into();
    }
}

pub fn state_dir() -> PathBuf {
    state_dir_cell()
        .lock()
        .map(|dir| dir.clone())
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_STATE_DIR))
}

pub fn ensure_state_dir() -> Result<(), String> {
    std::fs::create_dir_all(state_dir()).map_err(|e| format!("failed to create state dir: {}", e))
}

pub fn path(file_name: &str) -> String {
    let mut path = state_dir();
    path.push(file_name);
    path.to_string_lossy().into_owned()
}

pub fn graph_state_path() -> String {
    path("graph.json")
}
pub fn capability_state_path() -> String {
    path("capabilities.json")
}
pub fn legacy_memory_state_path() -> String {
    path("legacy_memory.json")
}
pub fn legacy_history_state_path() -> String {
    path("legacy_history.json")
}
pub fn observatory_state_path() -> String {
    path("observatory.json")
}
pub fn legacy_evolution_state_path() -> String {
    path("legacy_evolution.json")
}
pub fn legacy_cognition_state_path() -> String {
    path("legacy_cognition.json")
}
pub fn campo_tension_state_path() -> String {
    path("campo_tension.json")
}
pub fn legacy_knowledge_graph_state_path() -> String {
    path("legacy_knowledge_graph.json")
}
pub fn legacy_autoconsumo_state_path() -> String {
    path("legacy_autoconsumo.json")
}
pub fn legacy_venado_state_path() -> String {
    path("legacy_venado.json")
}
pub fn legacy_paradigm_hub_state_path() -> String {
    path("legacy_paradigm_hub.json")
}
pub fn legacy_ecosystem_state_path() -> String {
    path("legacy_ecosystem.json")
}
pub fn legacy_rebirth_meltrace_state_path() -> String {
    path("legacy_rebirth_meltrace.json")
}
pub fn legacy_crawler_state_path() -> String {
    path("legacy_crawler.json")
}
pub fn conscious_graph_regulator_state_path() -> String {
    path("conscious_graph_regulator.json")
}
pub fn context_augmentation_state_path() -> String {
    path("context_augmentation.json")
}
pub fn organ_autonomy_state_path() -> String {
    path("organ_autonomy.json")
}
pub fn goal_scheduler_state_path() -> String {
    path("goal_scheduler.json")
}
pub fn evaluation_loop_state_path() -> String {
    path("evaluation_loop.json")
}
pub fn learning_ledger_state_path() -> String {
    path("learning_ledger.json")
}
pub fn world_model_core_state_path() -> String {
    path("world_model_core.json")
}
pub fn competence_benchmark_state_path() -> String {
    path("competence_benchmark.json")
}
pub fn plan_executor_state_path() -> String {
    path("plan_executor.json")
}
pub fn working_memory_state_path() -> String {
    path("working_memory.json")
}
pub fn uncertainty_ledger_state_path() -> String {
    path("uncertainty_ledger.json")
}
pub fn experiment_runner_state_path() -> String {
    path("experiment_runner.json")
}
pub fn provenance_ledger_state_path() -> String {
    path("provenance_ledger.json")
}
pub fn policy_guard_state_path() -> String {
    path("policy_guard.json")
}
pub fn capability_maturity_state_path() -> String {
    path("capability_maturity.json")
}
pub fn coordinator_state_path() -> String {
    path("coordinator.json")
}
pub fn human_interface_state_path() -> String {
    path("human_interface.json")
}
pub fn meta_architect_state_path() -> String {
    path("meta_architect.json")
}
pub fn fast_reflexes_state_path() -> String {
    path("fast_reflexes.json")
}
pub fn benchmark_state_path() -> String {
    path("benchmark.json")
}
pub fn command_router_state_path() -> String {
    path("command_router.json")
}
pub fn persistence_state_path() -> String {
    path("persistence.json")
}
pub fn telemetry_state_path() -> String {
    path("telemetry.json")
}
pub fn api_server_state_path() -> String {
    path("api_server.json")
}
pub fn daemon_state_path() -> String {
    path("daemon.json")
}
pub fn help_state_path() -> String {
    path("help.json")
}
pub fn hrm_reasoner_state_path() -> String {
    path("hrm_reasoner.json")
}
pub fn voice_synthesizer_state_path() -> String {
    path("voice_synthesizer.json")
}
pub fn hybrid_voice_state_path() -> String {
    path("hybrid_voice.json")
}
pub fn hrm_text_pretraining_state_path() -> String {
    path("hrm_text_pretraining.json")
}
pub fn hrm_text_checkpoint_manifest_path() -> String {
    path("hrm_text_checkpoint_manifest.txt")
}
pub fn hrm_text_corpus_manifest_path() -> String {
    path("hrm_text_corpus_manifest.txt")
}
pub fn hrm_text_segments_path() -> String {
    path("hrm_text_segments.jsonl")
}
pub fn hrm_text_context_pack_path() -> String {
    path("hrm_text_context_pack.json")
}
pub fn voice_last_artifact_path() -> String {
    path("voice_last.txt")
}
pub fn hybrid_voice_manifest_path() -> String {
    path("hybrid_voice_manifest.txt")
}
pub fn voice_backend_request_path() -> String {
    path("voice_backend_request.txt")
}
pub fn voice_backend_output_path() -> String {
    path("voice_backend_output.txt")
}
pub fn backup_dir_path() -> String {
    path("backup")
}
pub fn garm_report_path() -> String {
    path("garm_report.txt")
}
pub fn garm_report_history_path() -> String {
    path("garm_report_history.jsonl")
}
pub fn garm_export_path() -> String {
    path("garm_export.json")
}
pub fn external_validation_manifest_path() -> String {
    path("external_validation_manifest.json")
}
pub fn external_validation_result_path() -> String {
    path("external_validation_result.json")
}
pub fn external_validation_suite_path() -> String {
    path("external_validation_suite.json")
}
pub fn readiness_package_path() -> String {
    path("readiness_package.json")
}
pub fn action_evidence_path() -> String {
    path("action_evidence.jsonl")
}
pub fn capability_registry_path() -> String {
    path("capability_registry.json")
}
pub fn cognitive_architecture_path() -> String {
    path("cognitive_architecture.json")
}
pub fn embodied_grounding_path() -> String {
    path("embodied_grounding.json")
}
pub fn neural_architecture_path() -> String {
    path("neural_architecture.json")
}
pub fn symbolic_architecture_path() -> String {
    path("symbolic_architecture.json")
}
pub fn self_improvement_architecture_path() -> String {
    path("self_improvement_architecture.json")
}
pub fn safety_control_architecture_path() -> String {
    path("safety_control_architecture.json")
}
pub fn foundation_model_architecture_path() -> String {
    path("foundation_model_architecture.json")
}
pub fn multimodal_model_architecture_path() -> String {
    path("multimodal_model_architecture.json")
}
pub fn llm_agent_architecture_path() -> String {
    path("llm_agent_architecture.json")
}
pub fn probabilistic_programming_architecture_path() -> String {
    path("probabilistic_programming_architecture.json")
}
pub fn hierarchical_rl_architecture_path() -> String {
    path("hierarchical_rl_architecture.json")
}
pub fn cognitive_robotics_architecture_path() -> String {
    path("cognitive_robotics_architecture.json")
}
pub fn vla_architecture_path() -> String {
    path("vla_architecture.json")
}
pub fn sim_to_real_architecture_path() -> String {
    path("sim_to_real_architecture.json")
}
pub fn open_ended_evolution_architecture_path() -> String {
    path("open_ended_evolution_architecture.json")
}
pub fn developmental_robotics_architecture_path() -> String {
    path("developmental_robotics_architecture.json")
}
pub fn whole_brain_neurocognitive_architecture_path() -> String {
    path("whole_brain_neurocognitive_architecture.json")
}
pub fn neuromorphic_spiking_architecture_path() -> String {
    path("neuromorphic_spiking_architecture.json")
}
pub fn paradigm_architecture_map_path() -> String {
    path("paradigm_architecture_map.json")
}
pub fn paradigm_architecture_technique_map_path() -> String {
    path("paradigm_architecture_technique_map.json")
}
pub fn neuro_symbolic_paradigm_path() -> String {
    path("neuro_symbolic_paradigm.json")
}
pub fn universal_formal_paradigm_path() -> String {
    path("universal_formal_paradigm.json")
}
pub fn active_inference_paradigm_path() -> String {
    path("active_inference_paradigm.json")
}
pub fn ecological_systemic_paradigm_path() -> String {
    path("ecological_systemic_paradigm.json")
}
pub fn computational_programmatic_paradigm_path() -> String {
    path("computational_programmatic_paradigm.json")
}
pub fn affective_motivational_paradigm_path() -> String {
    path("affective_motivational_paradigm.json")
}
pub fn human_in_the_loop_paradigm_path() -> String {
    path("human_in_the_loop_paradigm.json")
}
pub fn emergence_metrics_paradigm_path() -> String {
    path("emergence_metrics_paradigm.json")
}
pub fn integration_governance_architecture_path() -> String {
    path("integration_governance_architecture.json")
}
pub fn global_executive_workspace_core_path() -> String {
    path("global_executive_workspace_core.json")
}
pub fn global_executive_workspace_runtime_path() -> String {
    path("global_executive_workspace_runtime.jsonl")
}
pub fn global_executive_workspace_runtime_state_path() -> String {
    path("global_executive_workspace_runtime_state.json")
}
pub fn gewc_operational_benchmark_path() -> String {
    path("gewc_operational_benchmark.json")
}
pub fn gewc_runtime_safety_report_path() -> String {
    path("gewc_runtime_safety_report.json")
}
pub fn gewc_long_run_stability_path() -> String {
    path("gewc_long_run_stability.json")
}
pub fn capability_reality_eval_path() -> String {
    path("capability_reality_eval.json")
}
pub fn capability_reality_matrix_path() -> String {
    path("capability_reality_matrix.json")
}
pub fn lmm_training_dependency_report_path() -> String {
    path("lmm_training_dependency_report.json")
}
pub fn training_capability_report_path() -> String {
    "target/eden_training_smoke/capability_report.json".to_string()
}
pub fn training_capability_markdown_report_path() -> String {
    "target/eden_training_smoke/capability_report.md".to_string()
}
pub fn training_capability_evidence_path() -> String {
    path("training_capability_evidence.json")
}
pub fn megatron_7b_training_evidence_path() -> String {
    path("megatron_7b_training_evidence.json")
}
pub fn megatron_7b_model_adapter_path() -> String {
    path("megatron_7b_model_adapter.json")
}
pub fn megatron_7b_inference_report_path() -> String {
    path("megatron_7b_inference_report.json")
}
pub fn megatron_7b_capability_report_path() -> String {
    path("megatron_7b_capability_report.json")
}
pub fn megatron_7b_admission_gate_path() -> String {
    path("megatron_7b_admission_gate.json")
}
pub fn eden_70b_modular_target_path() -> String {
    path("eden_70b_modular_target.json")
}
pub fn eden_70b_module_router_path() -> String {
    path("eden_70b_module_router.json")
}
pub fn eden_70b_dataset_manifest_path() -> String {
    path("eden_70b_dataset_manifest.json")
}
pub fn eden_70b_launcher_manifest_path() -> String {
    path("eden_70b_launcher_manifest.json")
}
pub fn eden_70b_checkpoint_admission_path() -> String {
    path("eden_70b_checkpoint_admission.json")
}
pub fn eden_70b_inference_runtime_path() -> String {
    path("eden_70b_inference_runtime.json")
}
pub fn eden_70b_operational_demo_path() -> String {
    path("eden_70b_operational_demo.json")
}
pub fn eden_70b_operational_gate_path() -> String {
    path("eden_70b_operational_gate.json")
}
pub fn eden_capable_training_run_contract_path() -> String {
    path("eden_capable_training_run_contract.json")
}
pub fn eden_cognitive_dataset_manifest_path() -> String {
    path("eden_cognitive_dataset_manifest.json")
}
pub fn eden_native_inference_api_path() -> String {
    path("eden_native_inference_api.json")
}
pub fn eden_capability_delta_eval_path() -> String {
    path("eden_capability_delta_eval.json")
}
pub fn eden_structured_output_report_path() -> String {
    path("eden_structured_output_report.json")
}
pub fn eden_checkpoint_registry_path() -> String {
    path("eden_checkpoint_registry.json")
}
pub fn eden_sft_elcp_readiness_path() -> String {
    path("eden_sft_elcp_readiness.json")
}
pub fn eden_capable_gate_path() -> String {
    path("eden_capable_gate.json")
}
pub fn eden_live_inference_runtime_path() -> String {
    path("eden_live_inference_runtime.json")
}
pub fn eden_cognitive_call_contract_path() -> String {
    path("eden_cognitive_call_contract.json")
}
pub fn eden_cognitive_dataset_expansion_path() -> String {
    path("eden_cognitive_dataset_expansion.json")
}
pub fn eden_capability_eval_suite_path() -> String {
    path("eden_capability_eval_suite.json")
}
pub fn eden_sft_elcp_activation_gate_path() -> String {
    path("eden_sft_elcp_activation_gate.json")
}
pub fn eden_memory_action_loop_path() -> String {
    path("eden_memory_action_loop.json")
}
pub fn eden_capable_demo_trace_path() -> String {
    path("eden_capable_demo_trace.json")
}
pub fn eden_capable_operational_gate_path() -> String {
    path("eden_capable_operational_gate.json")
}
pub fn eden_sft_elcp_dataset_v2_manifest_path() -> String {
    path("eden_sft_elcp_dataset_v2_manifest.json")
}
pub fn eden_sft_elcp_gpu_training_report_path() -> String {
    path("eden_sft_elcp_gpu_training_report.json")
}
pub fn eden_sft_elcp_prepost_eval_path() -> String {
    path("eden_sft_elcp_prepost_eval.json")
}
pub fn eden_sft_elcp_repeated_inference_eval_path() -> String {
    path("eden_sft_elcp_repeated_inference_eval.json")
}
pub fn eden_sft_elcp_checkpoint_admission_review_path() -> String {
    path("eden_sft_elcp_checkpoint_admission_review.json")
}
pub fn eden_sft_elcp_operational_demo_path() -> String {
    path("eden_sft_elcp_operational_demo.json")
}
pub fn eden_external_tests_ci_gate_path() -> String {
    path("eden_external_tests_ci_gate.json")
}
pub fn eden_learned_capability_gate_path() -> String {
    path("eden_learned_capability_gate.json")
}
pub fn eden_real_capability_dataset_manifest_path() -> String {
    path("eden_real_capability_dataset_manifest.json")
}
pub fn eden_real_capability_7b_training_path() -> String {
    path("eden_real_capability_7b_training.json")
}
pub fn eden_real_capability_inference_bridge_path() -> String {
    path("eden_real_capability_inference_bridge.json")
}
pub fn eden_real_capability_operational_eval_path() -> String {
    path("eden_real_capability_operational_eval.json")
}
pub fn eden_real_capability_checkpoint_decision_path() -> String {
    path("eden_real_capability_checkpoint_decision.json")
}
pub fn eden_real_capability_demo_path() -> String {
    path("eden_real_capability_demo.json")
}
pub fn eden_real_capability_scaling_ladder_path() -> String {
    path("eden_real_capability_scaling_ladder.json")
}
pub fn eden_real_capability_gate_path() -> String {
    path("eden_real_capability_gate.json")
}
pub fn eden_v01_dataset_manifest_path() -> String {
    path("eden_v01_dataset_manifest.json")
}
pub fn eden_v01_semantic_eval_path() -> String {
    path("eden_v01_semantic_eval.json")
}
pub fn eden_v01_training_beyond_pilot_path() -> String {
    path("eden_v01_training_beyond_pilot.json")
}
pub fn eden_v01_native_inference_runtime_path() -> String {
    path("eden_v01_native_inference_runtime.json")
}
pub fn eden_v01_operational_demo_path() -> String {
    path("eden_v01_operational_demo.json")
}
pub fn eden_v01_checkpoint_admission_path() -> String {
    path("eden_v01_checkpoint_admission.json")
}
pub fn eden_v01_scaling_plan_path() -> String {
    path("eden_v01_scaling_plan.json")
}
pub fn eden_v01_gpu_workspace_hygiene_path() -> String {
    path("eden_v01_gpu_workspace_hygiene.json")
}
pub fn eden_v01_capability_gate_path() -> String {
    path("eden_v01_capability_gate.json")
}
pub fn eden_v02_stability_corpus_manifest_path() -> String {
    path("eden_v02_stability_corpus_manifest.json")
}
pub fn eden_v02_stability_eval_path() -> String {
    path("eden_v02_stability_eval.json")
}
pub fn eden_v02_checkpoint_comparison_path() -> String {
    path("eden_v02_checkpoint_comparison.json")
}
pub fn eden_v02_adversarial_eval_path() -> String {
    path("eden_v02_adversarial_eval.json")
}
pub fn eden_v02_rollback_drill_path() -> String {
    path("eden_v02_rollback_drill.json")
}
pub fn eden_v02_model_card_internal_path() -> String {
    path("eden_v02_model_card_internal.json")
}
pub fn eden_v02_checkpoint_storage_path() -> String {
    path("eden_v02_checkpoint_storage.json")
}
pub fn eden_v02_native_inference_service_path() -> String {
    path("eden_v02_native_inference_service.json")
}
pub fn eden_v02_stability_demo_path() -> String {
    path("eden_v02_stability_demo.json")
}
pub fn eden_v02_stability_gate_path() -> String {
    path("eden_v02_stability_gate.json")
}
pub fn eden_v03_generalization_corpus_manifest_path() -> String {
    path("eden_v03_generalization_corpus_manifest.json")
}
pub fn eden_v03_generalization_eval_path() -> String {
    path("eden_v03_generalization_eval.json")
}
pub fn eden_v03_checkpoint_admission_path() -> String {
    path("eden_v03_checkpoint_admission.json")
}
pub fn eden_v03_live_inference_runtime_path() -> String {
    path("eden_v03_live_inference_runtime.json")
}
pub fn eden_v03_checkpoint_registry_path() -> String {
    path("eden_v03_checkpoint_registry.json")
}
pub fn eden_v03_scaling_14b_plan_path() -> String {
    path("eden_v03_scaling_14b_plan.json")
}
pub fn eden_v03_operational_demo_path() -> String {
    path("eden_v03_operational_demo.json")
}
pub fn eden_v03_capability_gate_path() -> String {
    path("eden_v03_capability_gate.json")
}
pub fn eden_v04_cognitive_capability_corpus_manifest_path() -> String {
    path("eden_v04_cognitive_capability_corpus_manifest.json")
}
pub fn eden_v04_operational_capability_eval_path() -> String {
    path("eden_v04_operational_capability_eval.json")
}
pub fn eden_v04_generative_probe_path() -> String {
    path("eden_v04_generative_probe.json")
}
pub fn eden_v04_hard_checkpoint_admission_path() -> String {
    path("eden_v04_hard_checkpoint_admission.json")
}
pub fn eden_v04_persistent_inference_service_path() -> String {
    path("eden_v04_persistent_inference_service.json")
}
pub fn eden_v04_continuity_eval_path() -> String {
    path("eden_v04_continuity_eval.json")
}
pub fn eden_v04_scaling_14b_preflight_path() -> String {
    path("eden_v04_scaling_14b_preflight.json")
}
pub fn eden_v04_capability_gate_path() -> String {
    path("eden_v04_capability_gate.json")
}
pub fn model_adapter_runtime_path() -> String {
    path("model_adapter_runtime.json")
}
pub fn model_checkpoint_manifest_path() -> String {
    path("model_checkpoint_manifest.json")
}
pub fn paradise_checkpoint_registry_admission_path() -> String {
    path("paradise_checkpoint_registry_admission.json")
}
pub fn training_harness_report_path() -> String {
    path("training_harness_report.json")
}
pub fn model_governance_report_path() -> String {
    path("model_governance_report.json")
}
pub fn first_model_card_path() -> String {
    path("first_model_card.json")
}
pub fn first_model_training_plan_path() -> String {
    path("first_model_training_plan.json")
}
pub fn first_model_readiness_path() -> String {
    path("first_model_readiness.json")
}
pub fn elcp_objective_spec_path() -> String {
    path("elcp_objective_spec.json")
}
pub fn elcp_transition_dataset_path() -> String {
    path("elcp_transition_dataset.json")
}
pub fn elcp_training_plan_path() -> String {
    path("elcp_training_plan.json")
}
pub fn elcp_admission_gate_path() -> String {
    path("elcp_admission_gate.json")
}
pub fn elcp_trace_quality_gate_path() -> String {
    path("elcp_trace_quality_gate.json")
}
pub fn elcp_replay_eval_path() -> String {
    path("elcp_replay_eval.json")
}
pub fn elcp_dataset_freeze_manifest_path() -> String {
    path("elcp_dataset_freeze_manifest.json")
}
pub fn elcp_metrics_board_path() -> String {
    path("elcp_metrics_board.json")
}
pub fn elcp_4b_readiness_contract_path() -> String {
    path("elcp_4b_readiness_contract.json")
}
pub fn elcp_readiness_path() -> String {
    path("elcp_readiness.json")
}
pub fn gewc_trace_spec_path() -> String {
    path("gewc_trace_spec.json")
}
pub fn capability_reality_matrix_v2_path() -> String {
    path("capability_reality_matrix_v2.json")
}
pub fn cognitive_task_suite_path() -> String {
    path("cognitive_task_suite.json")
}
pub fn eden_agent_sdk_contract_path() -> String {
    path("eden_agent_sdk_contract.json")
}
pub fn model_adapter_layer_path() -> String {
    path("model_adapter_layer.json")
}
pub fn reproducible_demos_path() -> String {
    path("reproducible_demos.json")
}
pub fn architecture_advantage_eval_path() -> String {
    path("architecture_advantage_eval.json")
}
pub fn paradise_worldcell_runtime_path() -> String {
    path("paradise_worldcell_runtime.json")
}
pub fn paradise_worldcell_sessions_path() -> String {
    path("paradise_worldcell_sessions.json")
}
pub fn runtime_spine_path() -> String {
    path("runtime_spine.json")
}
pub fn runtime_internal_contracts_path() -> String {
    path("runtime_internal_contracts.json")
}
pub fn runtime_event_bus_path() -> String {
    path("runtime_event_bus.jsonl")
}
pub fn runtime_event_bus_state_path() -> String {
    path("runtime_event_bus_state.json")
}
pub fn runtime_global_state_path() -> String {
    path("runtime_global_state.json")
}
pub fn runtime_global_state_log_path() -> String {
    path("runtime_global_state_log.jsonl")
}
pub fn runtime_replay_spine_path() -> String {
    path("runtime_replay_spine.json")
}
pub fn runtime_spine_verification_path() -> String {
    path("runtime_spine_verification.json")
}
pub fn runtime_guard_decisions_path() -> String {
    path("runtime_guard_decisions.jsonl")
}
pub fn runtime_spine_enforcement_path() -> String {
    path("runtime_spine_enforcement.json")
}
pub fn runtime_workflow_risk_path() -> String {
    path("runtime_workflow_risk.json")
}
pub fn runtime_circuit_breakers_path() -> String {
    path("runtime_circuit_breakers.json")
}
pub fn runtime_replay_reconstruction_path() -> String {
    path("runtime_replay_reconstruction.json")
}
pub fn runtime_security_gates_path() -> String {
    path("runtime_security_gates.json")
}
pub fn runtime_model_router_contract_path() -> String {
    path("runtime_model_router_contract.json")
}
pub fn runtime_memory_fabric_contract_path() -> String {
    path("runtime_memory_fabric_contract.json")
}
pub fn runtime_world_simulation_contract_path() -> String {
    path("runtime_world_simulation_contract.json")
}
pub fn runtime_multiagent_contract_path() -> String {
    path("runtime_multiagent_contract.json")
}
pub fn eden_praxis_nexus_path() -> String {
    path("eden_praxis_nexus.json")
}
pub fn praxis_primitives_path() -> String {
    path("praxis_primitives.json")
}
pub fn praxis_blocks_path() -> String {
    path("praxis_blocks.json")
}
pub fn praxis_space_path() -> String {
    path("praxis_space.json")
}
pub fn praxis_rules_path() -> String {
    path("praxis_rules.json")
}
pub fn praxis_trace_semantics_path() -> String {
    path("praxis_trace_semantics.json")
}
pub fn praxis_reasoner_path() -> String {
    path("praxis_reasoner.json")
}
pub fn praxis_bench_path() -> String {
    path("praxis_bench.json")
}
pub fn eden_sovereign_cognition_path() -> String {
    path("eden_sovereign_cognition.json")
}
pub fn sovereign_sector_wins_path() -> String {
    path("sovereign_sector_wins.json")
}
pub fn praxis_calculus_formalism_path() -> String {
    path("praxis_calculus_formalism.json")
}
pub fn cognitive_contract_language_path() -> String {
    path("cognitive_contract_language.json")
}
pub fn evidence_memory_fabric_path() -> String {
    path("evidence_memory_fabric.json")
}
pub fn federated_runtime_fabric_path() -> String {
    path("federated_runtime_fabric.json")
}
pub fn symbolic_reasoning_fabric_path() -> String {
    path("symbolic_reasoning_fabric.json")
}
pub fn eden_external_ecosystem_path() -> String {
    path("eden_external_ecosystem.json")
}
pub fn ecosystem_participation_contract_path() -> String {
    path("ecosystem_participation_contract.json")
}
pub fn ecosystem_interop_matrix_path() -> String {
    path("ecosystem_interop_matrix.json")
}
pub fn ecosystem_certification_ladder_path() -> String {
    path("ecosystem_certification_ladder.json")
}
pub fn ecosystem_onboarding_runbook_path() -> String {
    path("ecosystem_onboarding_runbook.json")
}
pub fn ecosystem_governance_model_path() -> String {
    path("ecosystem_governance_model.json")
}
pub fn ecosystem_benchmark_exchange_path() -> String {
    path("ecosystem_benchmark_exchange.json")
}
pub fn artifact_api_catalog_path() -> String {
    path("artifact_api_catalog.json")
}
pub fn artifact_api_contracts_path() -> String {
    path("artifact_api_contracts.json")
}
pub fn artifact_api_runtime_path() -> String {
    path("artifact_api_runtime.json")
}
pub fn runtime_state_api_catalog_path() -> String {
    path("runtime_state_api_catalog.json")
}
pub fn runtime_state_api_contracts_path() -> String {
    path("runtime_state_api_contracts.json")
}
pub fn runtime_state_api_openapi_path() -> String {
    path("runtime_state_api_openapi.json")
}
pub fn runtime_state_api_runtime_path() -> String {
    path("runtime_state_api_runtime.json")
}
pub fn operational_api_catalog_path() -> String {
    path("operational_api_catalog.json")
}
pub fn operational_api_contracts_path() -> String {
    path("operational_api_contracts.json")
}
pub fn operational_api_openapi_path() -> String {
    path("operational_api_openapi.json")
}
pub fn operational_api_runtime_path() -> String {
    path("operational_api_runtime.json")
}
pub fn operational_action_contracts_path() -> String {
    path("operational_action_contracts.json")
}
pub fn operational_contract_path() -> String {
    path("operational_contract.json")
}
pub fn operational_permissions_path() -> String {
    path("operational_permissions.json")
}
pub fn operational_permissions_audit_path() -> String {
    path("operational_permissions_audit.json")
}
pub fn operational_permissions_diff_path() -> String {
    path("operational_permissions_diff.json")
}
pub fn operational_permissions_history_path() -> String {
    path("operational_permissions_history.jsonl")
}
pub fn operational_evidence_bundle_path() -> String {
    path("operational_evidence_bundle.json")
}
pub fn operational_recovery_plan_path() -> String {
    path("operational_recovery_plan.json")
}
pub fn operational_demo_suite_path() -> String {
    path("operational_demo_suite.json")
}
pub fn schema_registry_path() -> String {
    path("schema_registry.json")
}
pub fn operational_runtime_phase_path() -> String {
    path("operational_runtime_phase.json")
}
pub fn operational_task_runtime_path() -> String {
    path("operational_task_runtime.json")
}
pub fn operational_action_executor_path() -> String {
    path("operational_action_executor.json")
}
pub fn operational_lifecycle_controls_path() -> String {
    path("operational_lifecycle_controls.json")
}
pub fn operational_memory_transactions_path() -> String {
    path("operational_memory_transactions.json")
}
pub fn cwm_operational_state_path() -> String {
    path("cwm_operational_state.json")
}
pub fn locus_operator_bridge_path() -> String {
    path("locus_operator_bridge.json")
}
pub fn governed_agent_runtime_path() -> String {
    path("governed_agent_runtime.json")
}
pub fn operational_replay_eval_path() -> String {
    path("operational_replay_eval.json")
}
pub fn eden_locus_layer_path() -> String {
    path("eden_locus_layer.json")
}
pub fn locus_authority_model_path() -> String {
    path("locus_authority_model.json")
}
pub fn locus_evidence_vault_path() -> String {
    path("locus_evidence_vault.json")
}
pub fn locus_permission_matrix_path() -> String {
    path("locus_permission_matrix.json")
}
pub fn locus_context_packet_path() -> String {
    path("locus_context_packet.json")
}
pub fn locus_operator_timeline_path() -> String {
    path("locus_operator_timeline.jsonl")
}
pub fn eden_operator_forge_path() -> String {
    path("eden_operator_forge.json")
}
pub fn operator_primitive_basis_path() -> String {
    path("operator_primitive_basis.json")
}
pub fn operator_expression_graphs_path() -> String {
    path("operator_expression_graphs.jsonl")
}
pub fn operator_verification_report_path() -> String {
    path("operator_verification_report.json")
}
pub fn operator_model_registry_path() -> String {
    path("operator_model_registry.json")
}
pub fn operational_smoke_test_path() -> String {
    path("operational_smoke_test.json")
}
pub fn operational_e2e_scenario_path() -> String {
    path("operational_e2e_scenario.json")
}
pub fn memory_eval_path() -> String {
    path("memory_eval.json")
}
pub fn world_eval_path() -> String {
    path("world_eval.json")
}
pub fn legacy_session_import_path() -> String {
    path("legacy_session.eden")
}
pub fn runtime_state_path() -> String {
    path("runtime.json")
}
pub fn legacy_memory_text_path() -> String {
    path("legacy_memory.txt")
}

pub fn migrate_legacy_paths() -> Vec<String> {
    let _ = ensure_state_dir();
    let mappings = [
        ("/tmp/eden_garm_state.json", graph_state_path()),
        ("/tmp/eden_garm_engine_state.json", capability_state_path()),
        (
            "/tmp/eden_garm_legacy_memory_state.json",
            legacy_memory_state_path(),
        ),
        (
            "/tmp/eden_garm_legacy_history_state.json",
            legacy_history_state_path(),
        ),
        (
            "/tmp/eden_garm_observatory_state.json",
            observatory_state_path(),
        ),
        (
            "/tmp/eden_garm_legacy_evolution_state.json",
            legacy_evolution_state_path(),
        ),
        (
            "/tmp/eden_garm_legacy_cognition_state.json",
            legacy_cognition_state_path(),
        ),
        (
            "/tmp/eden_garm_campo_tension_state.json",
            campo_tension_state_path(),
        ),
        (
            "/tmp/eden_garm_legacy_knowledge_graph_state.json",
            legacy_knowledge_graph_state_path(),
        ),
        (
            "/tmp/eden_garm_legacy_autoconsumo_state.json",
            legacy_autoconsumo_state_path(),
        ),
        (
            "/tmp/eden_garm_legacy_venado_state.json",
            legacy_venado_state_path(),
        ),
        (
            "/tmp/eden_garm_legacy_paradigm_hub_state.json",
            legacy_paradigm_hub_state_path(),
        ),
        (
            "/tmp/eden_garm_legacy_ecosystem_state.json",
            legacy_ecosystem_state_path(),
        ),
        (
            "/tmp/eden_garm_legacy_rebirth_meltrace_state.json",
            legacy_rebirth_meltrace_state_path(),
        ),
        (
            "/tmp/eden_garm_legacy_crawler_state.json",
            legacy_crawler_state_path(),
        ),
        (
            "/tmp/eden_garm_conscious_graph_regulator_state.json",
            conscious_graph_regulator_state_path(),
        ),
        (
            "/tmp/eden_garm_context_augmentation_state.json",
            context_augmentation_state_path(),
        ),
        (
            "/tmp/eden_garm_organ_autonomy_state.json",
            organ_autonomy_state_path(),
        ),
        (
            "/tmp/eden_garm_goal_scheduler_state.json",
            goal_scheduler_state_path(),
        ),
        (
            "/tmp/eden_garm_evaluation_loop_state.json",
            evaluation_loop_state_path(),
        ),
        (
            "/tmp/eden_garm_learning_ledger_state.json",
            learning_ledger_state_path(),
        ),
        (
            "/tmp/eden_garm_world_model_core_state.json",
            world_model_core_state_path(),
        ),
        (
            "/tmp/eden_garm_competence_benchmark_state.json",
            competence_benchmark_state_path(),
        ),
        (
            "/tmp/eden_garm_plan_executor_state.json",
            plan_executor_state_path(),
        ),
        (
            "/tmp/eden_garm_working_memory_state.json",
            working_memory_state_path(),
        ),
        (
            "/tmp/eden_garm_uncertainty_ledger_state.json",
            uncertainty_ledger_state_path(),
        ),
        (
            "/tmp/eden_garm_experiment_runner_state.json",
            experiment_runner_state_path(),
        ),
        (
            "/tmp/eden_garm_provenance_ledger_state.json",
            provenance_ledger_state_path(),
        ),
        (
            "/tmp/eden_garm_policy_guard_state.json",
            policy_guard_state_path(),
        ),
        (
            "/tmp/eden_garm_capability_maturity_state.json",
            capability_maturity_state_path(),
        ),
        (
            "/tmp/eden_garm_hybrid_voice_state.json",
            hybrid_voice_state_path(),
        ),
        (
            "/tmp/eden_garm_hrm_text_pretraining_state.json",
            hrm_text_pretraining_state_path(),
        ),
        (".eden_session", legacy_session_import_path()),
        ("/tmp/eden_garm_runtime_state.json", runtime_state_path()),
        (
            "/tmp/eden_garm_legacy_memory.txt",
            legacy_memory_text_path(),
        ),
    ];
    let mut migrated = Vec::new();
    for (old, new) in mappings {
        if std::fs::metadata(&new).is_ok() || std::fs::metadata(old).is_err() {
            continue;
        }
        if std::fs::copy(old, &new).is_ok() {
            migrated.push(format!("{} -> {}", old, new));
        }
    }
    migrated
}

pub fn state_report() -> String {
    format!(
        "GARM state directory: {}\n- graph: {}\n- capabilities: {}\n- runtime: {}\n- legacy_memory: {}\n- legacy_history: {}\n- observatory: {}\n- legacy_evolution: {}\n- legacy_cognition: {}\n- campo_tension: {}\n- legacy_knowledge_graph: {}\n- legacy_autoconsumo: {}\n- legacy_venado: {}\n- legacy_paradigm_hub: {}\n- legacy_ecosystem: {}\n- legacy_rebirth_meltrace: {}\n- legacy_crawler: {}\n- conscious_graph_regulator: {}\n- context_augmentation: {}\n- organ_autonomy: {}\n- goal_scheduler: {}\n- evaluation_loop: {}\n- learning_ledger: {}\n- world_model_core: {}\n- competence_benchmark: {}\n- plan_executor: {}\n- working_memory: {}\n- uncertainty_ledger: {}\n- experiment_runner: {}\n- provenance_ledger: {}\n- policy_guard: {}\n- capability_maturity: {}\n- coordinator: {}\n- human_interface: {}\n- meta_architect: {}\n- fast_reflexes: {}\n- benchmark: {}\n- command_router: {}\n- persistence: {}\n- telemetry: {}\n- api_server: {}\n- daemon: {}\n- help: {}\n- hrm_reasoner: {}\n- voice_synthesizer: {}\n- hybrid_voice: {}\n- hrm_text_pretraining: {}\n- voice_last_artifact: {}\n- hybrid_voice_manifest: {}\n- hrm_text_checkpoint_manifest: {}\n- hrm_text_corpus_manifest: {}\n- hrm_text_segments: {}\n- hrm_text_context_pack: {}\n- voice_backend_request: {}\n- voice_backend_output: {}\n- backup_dir: {}\n- garm_report: {}\n- garm_report_history: {}\n- garm_export: {}\n- external_validation_manifest: {}\n- external_validation_result: {}\n- external_validation_suite: {}\n- readiness_package: {}\n- action_evidence: {}\n- capability_registry: {}\n- cognitive_architecture: {}\n- embodied_grounding: {}\n- neural_architecture: {}\n- symbolic_architecture: {}\n- self_improvement_architecture: {}\n- paradigm_architecture_map: {}\n- paradigm_architecture_technique_map: {}\n- neuro_symbolic_paradigm: {}\n- universal_formal_paradigm: {}\n- active_inference_paradigm: {}\n- ecological_systemic_paradigm: {}\n- computational_programmatic_paradigm: {}\n- affective_motivational_paradigm: {}\n- human_in_the_loop_paradigm: {}\n- emergence_metrics_paradigm: {}\n- integration_governance_architecture: {}\n- global_executive_workspace_core: {}\n- global_executive_workspace_runtime: {}\n- global_executive_workspace_runtime_state: {}\n- gewc_operational_benchmark: {}\n- gewc_runtime_safety_report: {}\n- gewc_long_run_stability: {}\n- capability_reality_eval: {}\n- capability_reality_matrix: {}\n- lmm_training_dependency_report: {}\n- gewc_trace_spec: {}\n- capability_reality_matrix_v2: {}\n- cognitive_task_suite: {}\n- eden_agent_sdk_contract: {}\n- model_adapter_layer: {}\n- reproducible_demos: {}\n- architecture_advantage_eval: {}\n- paradise_worldcell_runtime: {}\n- paradise_worldcell_sessions: {}\n- runtime_spine: {}\n- runtime_internal_contracts: {}\n- runtime_event_bus: {}\n- runtime_event_bus_state: {}\n- runtime_global_state: {}\n- runtime_global_state_log: {}\n- runtime_replay_spine: {}\n- runtime_spine_verification: {}\n- runtime_guard_decisions: {}\n- runtime_spine_enforcement: {}\n- runtime_workflow_risk: {}\n- runtime_circuit_breakers: {}\n- runtime_replay_reconstruction: {}\n- runtime_security_gates: {}\n- runtime_model_router_contract: {}\n- runtime_memory_fabric_contract: {}\n- runtime_world_simulation_contract: {}\n- runtime_multiagent_contract: {}\n- eden_praxis_nexus: {}\n- praxis_primitives: {}\n- praxis_blocks: {}\n- praxis_space: {}\n- praxis_rules: {}\n- praxis_trace_semantics: {}\n- praxis_reasoner: {}\n- praxis_bench: {}\n- eden_sovereign_cognition: {}\n- sovereign_sector_wins: {}\n- praxis_calculus_formalism: {}\n- cognitive_contract_language: {}\n- evidence_memory_fabric: {}\n- federated_runtime_fabric: {}\n- symbolic_reasoning_fabric: {}\n- eden_external_ecosystem: {}\n- ecosystem_participation_contract: {}\n- ecosystem_interop_matrix: {}\n- ecosystem_certification_ladder: {}\n- ecosystem_onboarding_runbook: {}\n- ecosystem_governance_model: {}\n- ecosystem_benchmark_exchange: {}\n- artifact_api_catalog: {}\n- artifact_api_contracts: {}\n- artifact_api_runtime: {}\n- runtime_state_api_catalog: {}\n- runtime_state_api_contracts: {}\n- runtime_state_api_openapi: {}\n- runtime_state_api_runtime: {}\n- operational_api_catalog: {}\n- operational_api_contracts: {}\n- operational_api_openapi: {}\n- operational_api_runtime: {}\n- operational_action_contracts: {}\n- operational_runtime_phase: {}\n- operational_task_runtime: {}\n- operational_action_executor: {}\n- operational_lifecycle_controls: {}\n- operational_memory_transactions: {}\n- cwm_operational_state: {}\n- governed_agent_runtime: {}\n- operational_replay_eval: {}\n- memory_eval: {}\n- world_eval: {}\n- legacy_memory_text: {}\n",
        state_dir().to_string_lossy(),
        graph_state_path(),
        capability_state_path(),
        runtime_state_path(),
        legacy_memory_state_path(),
        legacy_history_state_path(),
        observatory_state_path(),
        legacy_evolution_state_path(),
        legacy_cognition_state_path(),
        campo_tension_state_path(),
        legacy_knowledge_graph_state_path(),
        legacy_autoconsumo_state_path(),
        legacy_venado_state_path(),
        legacy_paradigm_hub_state_path(),
        legacy_ecosystem_state_path(),
        legacy_rebirth_meltrace_state_path(),
        legacy_crawler_state_path(),
        conscious_graph_regulator_state_path(),
        context_augmentation_state_path(),
        organ_autonomy_state_path(),
        goal_scheduler_state_path(),
        evaluation_loop_state_path(),
        learning_ledger_state_path(),
        world_model_core_state_path(),
        competence_benchmark_state_path(),
        plan_executor_state_path(),
        working_memory_state_path(),
        uncertainty_ledger_state_path(),
        experiment_runner_state_path(),
        provenance_ledger_state_path(),
        policy_guard_state_path(),
        capability_maturity_state_path(),
        coordinator_state_path(),
        human_interface_state_path(),
        meta_architect_state_path(),
        fast_reflexes_state_path(),
        benchmark_state_path(),
        command_router_state_path(),
        persistence_state_path(),
        telemetry_state_path(),
        api_server_state_path(),
        daemon_state_path(),
        help_state_path(),
        hrm_reasoner_state_path(),
        voice_synthesizer_state_path(),
        hybrid_voice_state_path(),
        hrm_text_pretraining_state_path(),
        voice_last_artifact_path(),
        hybrid_voice_manifest_path(),
        hrm_text_checkpoint_manifest_path(),
        hrm_text_corpus_manifest_path(),
        hrm_text_segments_path(),
        hrm_text_context_pack_path(),
        voice_backend_request_path(),
        voice_backend_output_path(),
        backup_dir_path(),
        garm_report_path(),
        garm_report_history_path(),
        garm_export_path(),
        external_validation_manifest_path(),
        external_validation_result_path(),
        external_validation_suite_path(),
        readiness_package_path(),
        action_evidence_path(),
        capability_registry_path(),
        cognitive_architecture_path(),
        embodied_grounding_path(),
        neural_architecture_path(),
        symbolic_architecture_path(),
        self_improvement_architecture_path(),
        paradigm_architecture_map_path(),
        paradigm_architecture_technique_map_path(),
        neuro_symbolic_paradigm_path(),
        universal_formal_paradigm_path(),
        active_inference_paradigm_path(),
        ecological_systemic_paradigm_path(),
        computational_programmatic_paradigm_path(),
        affective_motivational_paradigm_path(),
        human_in_the_loop_paradigm_path(),
        emergence_metrics_paradigm_path(),
        integration_governance_architecture_path(),
        global_executive_workspace_core_path(),
        global_executive_workspace_runtime_path(),
        global_executive_workspace_runtime_state_path(),
        gewc_operational_benchmark_path(),
        gewc_runtime_safety_report_path(),
        gewc_long_run_stability_path(),
        capability_reality_eval_path(),
        capability_reality_matrix_path(),
        lmm_training_dependency_report_path(),
        gewc_trace_spec_path(),
        capability_reality_matrix_v2_path(),
        cognitive_task_suite_path(),
        eden_agent_sdk_contract_path(),
        model_adapter_layer_path(),
        reproducible_demos_path(),
        architecture_advantage_eval_path(),
        paradise_worldcell_runtime_path(),
        paradise_worldcell_sessions_path(),
        runtime_spine_path(),
        runtime_internal_contracts_path(),
        runtime_event_bus_path(),
        runtime_event_bus_state_path(),
        runtime_global_state_path(),
        runtime_global_state_log_path(),
        runtime_replay_spine_path(),
        runtime_spine_verification_path(),
        runtime_guard_decisions_path(),
        runtime_spine_enforcement_path(),
        runtime_workflow_risk_path(),
        runtime_circuit_breakers_path(),
        runtime_replay_reconstruction_path(),
        runtime_security_gates_path(),
        runtime_model_router_contract_path(),
        runtime_memory_fabric_contract_path(),
        runtime_world_simulation_contract_path(),
        runtime_multiagent_contract_path(),
        eden_praxis_nexus_path(),
        praxis_primitives_path(),
        praxis_blocks_path(),
        praxis_space_path(),
        praxis_rules_path(),
        praxis_trace_semantics_path(),
        praxis_reasoner_path(),
        praxis_bench_path(),
        eden_sovereign_cognition_path(),
        sovereign_sector_wins_path(),
        praxis_calculus_formalism_path(),
        cognitive_contract_language_path(),
        evidence_memory_fabric_path(),
        federated_runtime_fabric_path(),
        symbolic_reasoning_fabric_path(),
        eden_external_ecosystem_path(),
        ecosystem_participation_contract_path(),
        ecosystem_interop_matrix_path(),
        ecosystem_certification_ladder_path(),
        ecosystem_onboarding_runbook_path(),
        ecosystem_governance_model_path(),
        ecosystem_benchmark_exchange_path(),
        artifact_api_catalog_path(),
        artifact_api_contracts_path(),
        artifact_api_runtime_path(),
        runtime_state_api_catalog_path(),
        runtime_state_api_contracts_path(),
        runtime_state_api_openapi_path(),
        runtime_state_api_runtime_path(),
        operational_api_catalog_path(),
        operational_api_contracts_path(),
        operational_api_openapi_path(),
        operational_api_runtime_path(),
        operational_action_contracts_path(),
        operational_runtime_phase_path(),
        operational_task_runtime_path(),
        operational_action_executor_path(),
        operational_lifecycle_controls_path(),
        operational_memory_transactions_path(),
        cwm_operational_state_path(),
        governed_agent_runtime_path(),
        operational_replay_eval_path(),
        memory_eval_path(),
        world_eval_path(),
        legacy_memory_text_path(),
    )
}

pub fn artifacts_report() -> String {
    let artifacts = [
        ("runtime", runtime_state_path(), false),
        ("capabilities", capability_state_path(), false),
        ("graph", graph_state_path(), false),
        ("organ_autonomy", organ_autonomy_state_path(), false),
        ("goal_scheduler", goal_scheduler_state_path(), false),
        ("evaluation_loop", evaluation_loop_state_path(), false),
        ("learning_ledger", learning_ledger_state_path(), false),
        ("world_model_core", world_model_core_state_path(), false),
        (
            "competence_benchmark",
            competence_benchmark_state_path(),
            false,
        ),
        ("plan_executor", plan_executor_state_path(), false),
        ("working_memory", working_memory_state_path(), false),
        ("uncertainty_ledger", uncertainty_ledger_state_path(), false),
        ("experiment_runner", experiment_runner_state_path(), false),
        ("provenance_ledger", provenance_ledger_state_path(), false),
        ("policy_guard", policy_guard_state_path(), false),
        (
            "capability_maturity",
            capability_maturity_state_path(),
            false,
        ),
        ("hrm_reasoner", hrm_reasoner_state_path(), false),
        ("voice_synthesizer", voice_synthesizer_state_path(), false),
        ("hybrid_voice", hybrid_voice_state_path(), false),
        (
            "hrm_text_pretraining",
            hrm_text_pretraining_state_path(),
            false,
        ),
        ("voice_last", voice_last_artifact_path(), false),
        ("hybrid_voice_manifest", hybrid_voice_manifest_path(), false),
        (
            "hrm_text_checkpoint_manifest",
            hrm_text_checkpoint_manifest_path(),
            false,
        ),
        (
            "hrm_text_corpus_manifest",
            hrm_text_corpus_manifest_path(),
            false,
        ),
        ("hrm_text_segments", hrm_text_segments_path(), false),
        ("hrm_text_context_pack", hrm_text_context_pack_path(), false),
        ("voice_backend_request", voice_backend_request_path(), false),
        ("voice_backend_output", voice_backend_output_path(), false),
        ("garm_report", garm_report_path(), false),
        ("garm_report_history", garm_report_history_path(), false),
        ("garm_export", garm_export_path(), false),
        (
            "external_validation_manifest",
            external_validation_manifest_path(),
            false,
        ),
        (
            "external_validation_result",
            external_validation_result_path(),
            false,
        ),
        (
            "external_validation_suite",
            external_validation_suite_path(),
            false,
        ),
        ("readiness_package", readiness_package_path(), false),
        ("action_evidence", action_evidence_path(), false),
        ("capability_registry", capability_registry_path(), false),
        (
            "cognitive_architecture",
            cognitive_architecture_path(),
            false,
        ),
        ("embodied_grounding", embodied_grounding_path(), false),
        ("neural_architecture", neural_architecture_path(), false),
        ("symbolic_architecture", symbolic_architecture_path(), false),
        (
            "self_improvement_architecture",
            self_improvement_architecture_path(),
            false,
        ),
        (
            "safety_control_architecture",
            safety_control_architecture_path(),
            false,
        ),
        (
            "foundation_model_architecture",
            foundation_model_architecture_path(),
            false,
        ),
        (
            "multimodal_model_architecture",
            multimodal_model_architecture_path(),
            false,
        ),
        (
            "llm_agent_architecture",
            llm_agent_architecture_path(),
            false,
        ),
        (
            "probabilistic_programming_architecture",
            probabilistic_programming_architecture_path(),
            false,
        ),
        (
            "hierarchical_rl_architecture",
            hierarchical_rl_architecture_path(),
            false,
        ),
        (
            "cognitive_robotics_architecture",
            cognitive_robotics_architecture_path(),
            false,
        ),
        ("vla_architecture", vla_architecture_path(), false),
        (
            "sim_to_real_architecture",
            sim_to_real_architecture_path(),
            false,
        ),
        (
            "open_ended_evolution_architecture",
            open_ended_evolution_architecture_path(),
            false,
        ),
        (
            "developmental_robotics_architecture",
            developmental_robotics_architecture_path(),
            false,
        ),
        (
            "whole_brain_neurocognitive_architecture",
            whole_brain_neurocognitive_architecture_path(),
            false,
        ),
        (
            "neuromorphic_spiking_architecture",
            neuromorphic_spiking_architecture_path(),
            false,
        ),
        (
            "paradigm_architecture_map",
            paradigm_architecture_map_path(),
            false,
        ),
        (
            "paradigm_architecture_technique_map",
            paradigm_architecture_technique_map_path(),
            false,
        ),
        (
            "neuro_symbolic_paradigm",
            neuro_symbolic_paradigm_path(),
            false,
        ),
        (
            "universal_formal_paradigm",
            universal_formal_paradigm_path(),
            false,
        ),
        (
            "active_inference_paradigm",
            active_inference_paradigm_path(),
            false,
        ),
        (
            "ecological_systemic_paradigm",
            ecological_systemic_paradigm_path(),
            false,
        ),
        (
            "computational_programmatic_paradigm",
            computational_programmatic_paradigm_path(),
            false,
        ),
        (
            "affective_motivational_paradigm",
            affective_motivational_paradigm_path(),
            false,
        ),
        (
            "human_in_the_loop_paradigm",
            human_in_the_loop_paradigm_path(),
            false,
        ),
        (
            "emergence_metrics_paradigm",
            emergence_metrics_paradigm_path(),
            false,
        ),
        (
            "integration_governance_architecture",
            integration_governance_architecture_path(),
            false,
        ),
        (
            "global_executive_workspace_core",
            global_executive_workspace_core_path(),
            false,
        ),
        (
            "global_executive_workspace_runtime",
            global_executive_workspace_runtime_path(),
            false,
        ),
        (
            "global_executive_workspace_runtime_state",
            global_executive_workspace_runtime_state_path(),
            false,
        ),
        (
            "gewc_operational_benchmark",
            gewc_operational_benchmark_path(),
            false,
        ),
        (
            "gewc_runtime_safety_report",
            gewc_runtime_safety_report_path(),
            false,
        ),
        (
            "gewc_long_run_stability",
            gewc_long_run_stability_path(),
            false,
        ),
        (
            "capability_reality_eval",
            capability_reality_eval_path(),
            false,
        ),
        (
            "capability_reality_matrix",
            capability_reality_matrix_path(),
            false,
        ),
        (
            "lmm_training_dependency_report",
            lmm_training_dependency_report_path(),
            false,
        ),
        (
            "training_capability_report",
            training_capability_report_path(),
            false,
        ),
        (
            "training_capability_report_markdown",
            training_capability_markdown_report_path(),
            false,
        ),
        (
            "training_capability_evidence",
            training_capability_evidence_path(),
            false,
        ),
        ("model_adapter_runtime", model_adapter_runtime_path(), false),
        (
            "model_checkpoint_manifest",
            model_checkpoint_manifest_path(),
            false,
        ),
        (
            "paradise_checkpoint_registry_admission",
            paradise_checkpoint_registry_admission_path(),
            false,
        ),
        (
            "training_harness_report",
            training_harness_report_path(),
            false,
        ),
        (
            "model_governance_report",
            model_governance_report_path(),
            false,
        ),
        (
            "eden_70b_modular_target",
            eden_70b_modular_target_path(),
            false,
        ),
        (
            "eden_70b_module_router",
            eden_70b_module_router_path(),
            false,
        ),
        (
            "eden_70b_dataset_manifest",
            eden_70b_dataset_manifest_path(),
            false,
        ),
        (
            "eden_70b_launcher_manifest",
            eden_70b_launcher_manifest_path(),
            false,
        ),
        (
            "eden_70b_checkpoint_admission",
            eden_70b_checkpoint_admission_path(),
            false,
        ),
        (
            "eden_70b_inference_runtime",
            eden_70b_inference_runtime_path(),
            false,
        ),
        (
            "eden_70b_operational_demo",
            eden_70b_operational_demo_path(),
            false,
        ),
        (
            "eden_70b_operational_gate",
            eden_70b_operational_gate_path(),
            false,
        ),
        ("first_model_card", first_model_card_path(), false),
        (
            "first_model_training_plan",
            first_model_training_plan_path(),
            false,
        ),
        ("first_model_readiness", first_model_readiness_path(), false),
        ("elcp_objective_spec", elcp_objective_spec_path(), false),
        (
            "elcp_transition_dataset",
            elcp_transition_dataset_path(),
            false,
        ),
        ("elcp_training_plan", elcp_training_plan_path(), false),
        ("elcp_admission_gate", elcp_admission_gate_path(), false),
        (
            "elcp_trace_quality_gate",
            elcp_trace_quality_gate_path(),
            false,
        ),
        ("elcp_replay_eval", elcp_replay_eval_path(), false),
        (
            "elcp_dataset_freeze_manifest",
            elcp_dataset_freeze_manifest_path(),
            false,
        ),
        ("elcp_metrics_board", elcp_metrics_board_path(), false),
        (
            "elcp_4b_readiness_contract",
            elcp_4b_readiness_contract_path(),
            false,
        ),
        ("elcp_readiness", elcp_readiness_path(), false),
        ("gewc_trace_spec", gewc_trace_spec_path(), false),
        (
            "capability_reality_matrix_v2",
            capability_reality_matrix_v2_path(),
            false,
        ),
        ("cognitive_task_suite", cognitive_task_suite_path(), false),
        (
            "eden_agent_sdk_contract",
            eden_agent_sdk_contract_path(),
            false,
        ),
        ("model_adapter_layer", model_adapter_layer_path(), false),
        ("reproducible_demos", reproducible_demos_path(), false),
        (
            "architecture_advantage_eval",
            architecture_advantage_eval_path(),
            false,
        ),
        (
            "paradise_worldcell_runtime",
            paradise_worldcell_runtime_path(),
            false,
        ),
        (
            "paradise_worldcell_sessions",
            paradise_worldcell_sessions_path(),
            false,
        ),
        ("runtime_spine", runtime_spine_path(), false),
        (
            "runtime_internal_contracts",
            runtime_internal_contracts_path(),
            false,
        ),
        ("runtime_event_bus", runtime_event_bus_path(), false),
        (
            "runtime_event_bus_state",
            runtime_event_bus_state_path(),
            false,
        ),
        ("runtime_global_state", runtime_global_state_path(), false),
        (
            "runtime_global_state_log",
            runtime_global_state_log_path(),
            false,
        ),
        ("runtime_replay_spine", runtime_replay_spine_path(), false),
        (
            "runtime_spine_verification",
            runtime_spine_verification_path(),
            false,
        ),
        (
            "runtime_guard_decisions",
            runtime_guard_decisions_path(),
            false,
        ),
        (
            "runtime_spine_enforcement",
            runtime_spine_enforcement_path(),
            false,
        ),
        ("runtime_workflow_risk", runtime_workflow_risk_path(), false),
        (
            "runtime_circuit_breakers",
            runtime_circuit_breakers_path(),
            false,
        ),
        (
            "runtime_replay_reconstruction",
            runtime_replay_reconstruction_path(),
            false,
        ),
        (
            "runtime_security_gates",
            runtime_security_gates_path(),
            false,
        ),
        (
            "runtime_model_router_contract",
            runtime_model_router_contract_path(),
            false,
        ),
        (
            "runtime_memory_fabric_contract",
            runtime_memory_fabric_contract_path(),
            false,
        ),
        (
            "runtime_world_simulation_contract",
            runtime_world_simulation_contract_path(),
            false,
        ),
        (
            "runtime_multiagent_contract",
            runtime_multiagent_contract_path(),
            false,
        ),
        ("eden_praxis_nexus", eden_praxis_nexus_path(), false),
        ("praxis_primitives", praxis_primitives_path(), false),
        ("praxis_blocks", praxis_blocks_path(), false),
        ("praxis_space", praxis_space_path(), false),
        ("praxis_rules", praxis_rules_path(), false),
        (
            "praxis_trace_semantics",
            praxis_trace_semantics_path(),
            false,
        ),
        ("praxis_reasoner", praxis_reasoner_path(), false),
        ("praxis_bench", praxis_bench_path(), false),
        (
            "eden_sovereign_cognition",
            eden_sovereign_cognition_path(),
            false,
        ),
        ("sovereign_sector_wins", sovereign_sector_wins_path(), false),
        (
            "praxis_calculus_formalism",
            praxis_calculus_formalism_path(),
            false,
        ),
        (
            "cognitive_contract_language",
            cognitive_contract_language_path(),
            false,
        ),
        (
            "evidence_memory_fabric",
            evidence_memory_fabric_path(),
            false,
        ),
        (
            "federated_runtime_fabric",
            federated_runtime_fabric_path(),
            false,
        ),
        (
            "symbolic_reasoning_fabric",
            symbolic_reasoning_fabric_path(),
            false,
        ),
        (
            "eden_external_ecosystem",
            eden_external_ecosystem_path(),
            false,
        ),
        (
            "ecosystem_participation_contract",
            ecosystem_participation_contract_path(),
            false,
        ),
        (
            "ecosystem_interop_matrix",
            ecosystem_interop_matrix_path(),
            false,
        ),
        (
            "ecosystem_certification_ladder",
            ecosystem_certification_ladder_path(),
            false,
        ),
        (
            "ecosystem_onboarding_runbook",
            ecosystem_onboarding_runbook_path(),
            false,
        ),
        (
            "ecosystem_governance_model",
            ecosystem_governance_model_path(),
            false,
        ),
        (
            "ecosystem_benchmark_exchange",
            ecosystem_benchmark_exchange_path(),
            false,
        ),
        ("artifact_api_catalog", artifact_api_catalog_path(), false),
        (
            "artifact_api_contracts",
            artifact_api_contracts_path(),
            false,
        ),
        ("artifact_api_runtime", artifact_api_runtime_path(), false),
        (
            "runtime_state_api_catalog",
            runtime_state_api_catalog_path(),
            false,
        ),
        (
            "runtime_state_api_contracts",
            runtime_state_api_contracts_path(),
            false,
        ),
        (
            "runtime_state_api_openapi",
            runtime_state_api_openapi_path(),
            false,
        ),
        (
            "runtime_state_api_runtime",
            runtime_state_api_runtime_path(),
            false,
        ),
        (
            "operational_api_catalog",
            operational_api_catalog_path(),
            false,
        ),
        (
            "operational_api_contracts",
            operational_api_contracts_path(),
            false,
        ),
        (
            "operational_api_openapi",
            operational_api_openapi_path(),
            false,
        ),
        (
            "operational_api_runtime",
            operational_api_runtime_path(),
            false,
        ),
        (
            "operational_action_contracts",
            operational_action_contracts_path(),
            false,
        ),
        ("operational_contract", operational_contract_path(), false),
        (
            "operational_permissions",
            operational_permissions_path(),
            false,
        ),
        (
            "operational_permissions_audit",
            operational_permissions_audit_path(),
            false,
        ),
        (
            "operational_permissions_diff",
            operational_permissions_diff_path(),
            false,
        ),
        (
            "operational_permissions_history",
            operational_permissions_history_path(),
            false,
        ),
        (
            "operational_evidence_bundle",
            operational_evidence_bundle_path(),
            false,
        ),
        (
            "operational_recovery_plan",
            operational_recovery_plan_path(),
            false,
        ),
        (
            "operational_demo_suite",
            operational_demo_suite_path(),
            false,
        ),
        ("schema_registry", schema_registry_path(), false),
        (
            "operational_runtime_phase",
            operational_runtime_phase_path(),
            false,
        ),
        (
            "operational_task_runtime",
            operational_task_runtime_path(),
            false,
        ),
        (
            "operational_action_executor",
            operational_action_executor_path(),
            false,
        ),
        (
            "operational_lifecycle_controls",
            operational_lifecycle_controls_path(),
            false,
        ),
        (
            "operational_memory_transactions",
            operational_memory_transactions_path(),
            false,
        ),
        ("cwm_operational_state", cwm_operational_state_path(), false),
        ("locus_operator_bridge", locus_operator_bridge_path(), false),
        (
            "governed_agent_runtime",
            governed_agent_runtime_path(),
            false,
        ),
        (
            "operational_replay_eval",
            operational_replay_eval_path(),
            false,
        ),
        (
            "operational_smoke_test",
            operational_smoke_test_path(),
            false,
        ),
        (
            "operational_e2e_scenario",
            operational_e2e_scenario_path(),
            false,
        ),
        ("memory_eval", memory_eval_path(), false),
        ("world_eval", world_eval_path(), false),
        ("backup", backup_dir_path(), true),
        ("legacy_memory_text", legacy_memory_text_path(), false),
    ];
    let mut present = 0usize;
    let mut missing = 0usize;
    let mut total_bytes = 0u64;
    let mut out = format!(
        "[GARM-ARTIFACTS] state_dir={} expected={}\n",
        state_dir().to_string_lossy(),
        artifacts.len()
    );
    for (name, path, is_dir) in artifacts {
        let metadata = std::fs::metadata(&path);
        match metadata {
            Ok(meta) if is_dir && meta.is_dir() => {
                present += 1;
                let entries = std::fs::read_dir(&path)
                    .map(|iter| iter.filter_map(Result::ok).count())
                    .unwrap_or(0);
                out.push_str(&format!(
                    "- name={} kind=dir exists=true entries={} path={}\n",
                    name, entries, path
                ));
            }
            Ok(meta) if !is_dir && meta.is_file() => {
                present += 1;
                total_bytes += meta.len();
                let data = std::fs::read(&path).unwrap_or_default();
                let jsonl_entries = if path.ends_with(".jsonl") {
                    String::from_utf8_lossy(&data).lines().count()
                } else {
                    0
                };
                out.push_str(&format!(
                    "- name={} kind=file exists=true bytes={} fnv64={:016x} jsonl_entries={} path={}\n",
                    name,
                    meta.len(),
                    fnv64(&data),
                    jsonl_entries,
                    path
                ));
            }
            _ => {
                missing += 1;
                out.push_str(&format!(
                    "- name={} kind={} exists=false bytes=0 fnv64=missing path={}\n",
                    name,
                    if is_dir { "dir" } else { "file" },
                    path
                ));
            }
        }
    }
    let verdict = if missing == 0 {
        "complete"
    } else if present > 0 {
        "partial"
    } else {
        "missing"
    };
    out.push_str(&format!(
        "[GARM-ARTIFACTS-SUMMARY] present={} missing={} total_bytes={} verdict={}\n",
        present, missing, total_bytes, verdict
    ));
    out
}

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |mut hash, byte| {
        hash ^= *byte as u64;
        hash.wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_paths_under_state_dir() {
        let _state_guard = test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_garm_state_paths_test_{}", std::process::id()));
        set_state_dir(dir.clone());
        assert_eq!(graph_state_path(), dir.join("graph.json").to_string_lossy());
        assert_eq!(
            runtime_state_path(),
            dir.join("runtime.json").to_string_lossy()
        );
        assert_eq!(
            context_augmentation_state_path(),
            dir.join("context_augmentation.json").to_string_lossy()
        );
        assert_eq!(
            goal_scheduler_state_path(),
            dir.join("goal_scheduler.json").to_string_lossy()
        );
        assert_eq!(
            evaluation_loop_state_path(),
            dir.join("evaluation_loop.json").to_string_lossy()
        );
        assert_eq!(
            learning_ledger_state_path(),
            dir.join("learning_ledger.json").to_string_lossy()
        );
        assert_eq!(
            world_model_core_state_path(),
            dir.join("world_model_core.json").to_string_lossy()
        );
        assert_eq!(
            competence_benchmark_state_path(),
            dir.join("competence_benchmark.json").to_string_lossy()
        );
        assert_eq!(
            plan_executor_state_path(),
            dir.join("plan_executor.json").to_string_lossy()
        );
        assert_eq!(
            working_memory_state_path(),
            dir.join("working_memory.json").to_string_lossy()
        );
        assert_eq!(
            uncertainty_ledger_state_path(),
            dir.join("uncertainty_ledger.json").to_string_lossy()
        );
        assert_eq!(
            experiment_runner_state_path(),
            dir.join("experiment_runner.json").to_string_lossy()
        );
        assert_eq!(
            provenance_ledger_state_path(),
            dir.join("provenance_ledger.json").to_string_lossy()
        );
        assert_eq!(
            policy_guard_state_path(),
            dir.join("policy_guard.json").to_string_lossy()
        );
        assert_eq!(
            capability_maturity_state_path(),
            dir.join("capability_maturity.json").to_string_lossy()
        );
        assert_eq!(
            voice_backend_output_path(),
            dir.join("voice_backend_output.txt").to_string_lossy()
        );
        assert_eq!(
            hybrid_voice_state_path(),
            dir.join("hybrid_voice.json").to_string_lossy()
        );
        assert_eq!(
            hybrid_voice_manifest_path(),
            dir.join("hybrid_voice_manifest.txt").to_string_lossy()
        );
        assert_eq!(
            hrm_text_pretraining_state_path(),
            dir.join("hrm_text_pretraining.json").to_string_lossy()
        );
        assert_eq!(
            hrm_text_checkpoint_manifest_path(),
            dir.join("hrm_text_checkpoint_manifest.txt")
                .to_string_lossy()
        );
        assert_eq!(
            hrm_text_corpus_manifest_path(),
            dir.join("hrm_text_corpus_manifest.txt").to_string_lossy()
        );
        assert_eq!(
            hrm_text_segments_path(),
            dir.join("hrm_text_segments.jsonl").to_string_lossy()
        );
        assert_eq!(
            garm_report_path(),
            dir.join("garm_report.txt").to_string_lossy()
        );
        assert_eq!(
            garm_report_history_path(),
            dir.join("garm_report_history.jsonl").to_string_lossy()
        );
        assert_eq!(
            garm_export_path(),
            dir.join("garm_export.json").to_string_lossy()
        );
    }

    #[test]
    fn artifacts_report_lists_files_with_metadata() {
        let _state_guard = test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_garm_artifacts_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        set_state_dir(dir.clone());
        ensure_state_dir().unwrap();
        std::fs::write(garm_report_path(), "report").unwrap();
        std::fs::write(garm_report_history_path(), "{}\n{}\n").unwrap();
        std::fs::write(garm_export_path(), "export").unwrap();
        std::fs::create_dir_all(backup_dir_path()).unwrap();

        let report = artifacts_report();

        assert!(report.contains("[GARM-ARTIFACTS]"));
        assert!(report.contains("name=garm_report kind=file exists=true bytes=6"));
        assert!(report.contains("name=garm_report_history kind=file exists=true"));
        assert!(report.contains("jsonl_entries=2"));
        assert!(report.contains("name=garm_export kind=file exists=true"));
        assert!(report.contains("fnv64="));
        assert!(report.contains("name=backup kind=dir exists=true"));
        assert!(report.contains("[GARM-ARTIFACTS-SUMMARY]"));
        let _ = std::fs::remove_dir_all(&dir);
        set_state_dir("/tmp/eden_garm");
    }
}
