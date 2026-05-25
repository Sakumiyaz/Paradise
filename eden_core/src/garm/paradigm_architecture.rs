use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct ParadigmArchitectureInput {
    pub capability_status: String,
    pub readiness_report: String,
    pub cognitive_report: String,
    pub embodied_report: String,
    pub neural_report: String,
    pub symbolic_report: String,
    pub self_improvement_report: String,
    pub frontier_report: String,
    pub world_report: String,
    pub memory_report: String,
    pub plan_executor_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
    pub external_validation_report: String,
}

struct ParadigmLayer {
    tag: &'static str,
    artifact: &'static str,
    paradigm: &'static str,
    decision: &'static str,
    path: String,
    cases: Vec<(&'static str, bool)>,
}

pub fn run(input: ParadigmArchitectureInput) -> String {
    let map_out = write_map(&input);
    let technique_out = write_technique_map();
    let layers = paradigm_layers(&input);
    let mut out = format!("{}{}", map_out, technique_out);
    for layer in layers {
        let passed = layer.cases.iter().filter(|(_, passed)| *passed).count();
        let total = layer.cases.len();
        let cases: Vec<_> = layer
            .cases
            .iter()
            .map(|(id, passed)| serde_json::json!({ "id": id, "passed": passed }))
            .collect();
        let record = serde_json::json!({
            "schema": "garm-paradigm-architecture-layer-v1",
            "paradigm": layer.paradigm,
            "decision": layer.decision,
            "claim_allowed": false,
            "agi_claim": false,
            "validation_scope": "formal_paradigm_layer_local_evidence",
            "passed": passed,
            "total": total,
            "verdict": if passed == total { "paradigm_layer_ready_local" } else { "needs_paradigm_layer_evidence" },
            "cases": cases,
        });
        let _ = state_paths::ensure_state_dir();
        let write_status = match std::fs::write(
            &layer.path,
            serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
        ) {
            Ok(()) => "paradigm_layer_written",
            Err(_) => "paradigm_layer_write_failed",
        };
        out.push_str(&format!(
            "[{}] artifact={} passed={}/{} claim_allowed=false write_status={} path={}\n",
            layer.tag, layer.artifact, passed, total, write_status, layer.path
        ));
    }
    out
}

#[derive(Clone, Copy)]
struct LegacyTechnique {
    name: &'static str,
    mapped_paradigm: &'static str,
    status: &'static str,
    canonical: &'static str,
    disposition: &'static str,
}

const LEGACY_TECHNIQUES: [LegacyTechnique; 43] = [
    legacy(
        "active",
        "predictive_active_inference",
        "alias",
        "active_learning",
        "merged_with_active_learning",
    ),
    legacy(
        "curriculum",
        "developmental",
        "subtype",
        "developmental_curriculum",
        "kept_as_developmental_subtype",
    ),
    legacy(
        "causal",
        "neuro_symbolic",
        "subtype",
        "causal_reasoning",
        "kept_as_transversal_reasoning_subtype",
    ),
    legacy(
        "contrastive",
        "connectionist_neural",
        "subtype",
        "contrastive_learning",
        "kept_as_neural_training_subtype",
    ),
    legacy(
        "ensemble",
        "hybrid",
        "implementation_detail",
        "hybrid_ensemble_orchestration",
        "internal_composition_pattern",
    ),
    legacy(
        "xai",
        "human_in_the_loop_collaborative",
        "alias",
        "human_explainability",
        "merged_into_human_in_the_loop_and_formal_traceability",
    ),
    legacy(
        "gnn",
        "neuro_symbolic",
        "alias",
        "neuro_symbolic_graph_reasoning",
        "merged_with_graph_reasoning_stack",
    ),
    legacy(
        "transformer",
        "connectionist_neural",
        "alias",
        "foundation_model_architecture",
        "merged_under_neural_foundation_model_stack",
    ),
    legacy(
        "rl",
        "reinforcement_learning",
        "replaced",
        "hierarchical_rl_architecture",
        "replaced_by_safe_hierarchical_model_based_rl",
    ),
    legacy(
        "mcts",
        "computational_programmatic",
        "subtype",
        "symbolic_search_planning",
        "kept_as_search_planning_subtype",
    ),
    legacy(
        "bayesian",
        "probabilistic_bayesian",
        "subtype",
        "bayesian_inference",
        "kept_as_probabilistic_subtype",
    ),
    legacy(
        "logic",
        "symbolic_logicist",
        "subtype",
        "logic_reasoning",
        "kept_as_symbolic_subtype",
    ),
    legacy(
        "neuro_symbolic",
        "neuro_symbolic",
        "formalized",
        "neuro_symbolic_paradigm",
        "covered_by_formal_paradigm_layer",
    ),
    legacy(
        "program_synthesis",
        "computational_programmatic",
        "subtype",
        "program_synthesis",
        "kept_as_programmatic_subtype",
    ),
    legacy(
        "rag",
        "world_models",
        "subtype",
        "verifiable_retrieval",
        "kept_as_retrieval_memory_subtype",
    ),
    legacy(
        "diffusion",
        "connectionist_neural",
        "implementation_detail",
        "generative_neural_model",
        "internal_neural_generation_technique",
    ),
    legacy(
        "adversarial",
        "alignment_control_safety",
        "subtype",
        "adversarial_robustness",
        "kept_as_safety_robustness_subtype",
    ),
    legacy(
        "spike",
        "neuromorphic_neurobiological",
        "alias",
        "neuromorphic_spiking",
        "merged_with_neuromorphic",
    ),
    legacy(
        "quantum",
        "universal_formal",
        "archived",
        "archived_quantum_experiment",
        "archived_no_current_formal_promotion",
    ),
    legacy(
        "neuromorphic",
        "neuromorphic_neurobiological",
        "formalized",
        "neuromorphic_spiking_architecture",
        "covered_by_frontier_formal_layer",
    ),
    legacy(
        "neural_ode",
        "connectionist_neural",
        "implementation_detail",
        "neural_dynamics_model",
        "internal_neural_dynamics_technique",
    ),
    legacy(
        "hypernet",
        "connectionist_neural",
        "implementation_detail",
        "hypernetwork_model",
        "internal_model_generation_technique",
    ),
    legacy(
        "active_learning",
        "predictive_active_inference",
        "subtype",
        "active_learning",
        "kept_as_active_inference_subtype",
    ),
    legacy(
        "self_supervised",
        "connectionist_neural",
        "subtype",
        "self_supervised_learning",
        "kept_as_neural_training_subtype",
    ),
    legacy(
        "transfer",
        "developmental",
        "subtype",
        "transfer_learning",
        "kept_as_developmental_generalization_subtype",
    ),
    legacy(
        "enactive",
        "embodied_situated",
        "alias",
        "embodied_situated",
        "merged_into_embodied_situated",
    ),
    legacy(
        "meta_learning",
        "metacognitive_reflective",
        "subtype",
        "meta_learning",
        "kept_as_metacognitive_adaptation_subtype",
    ),
    legacy(
        "continual",
        "developmental",
        "subtype",
        "continual_learning",
        "kept_as_lifelong_developmental_subtype",
    ),
    legacy(
        "zero_shot",
        "connectionist_neural",
        "subtype",
        "zero_shot_generalization",
        "kept_as_foundation_model_generalization_subtype",
    ),
    legacy(
        "few_shot",
        "connectionist_neural",
        "subtype",
        "few_shot_adaptation",
        "kept_as_foundation_model_adaptation_subtype",
    ),
    legacy(
        "compression",
        "computational_programmatic",
        "implementation_detail",
        "compression",
        "internal_efficiency_technique",
    ),
    legacy(
        "federated",
        "ecological_systemic",
        "future",
        "federated_systemic_learning",
        "future_distributed_systemic_capability",
    ),
    legacy(
        "distillation",
        "connectionist_neural",
        "implementation_detail",
        "distillation",
        "internal_model_compression_technique",
    ),
    legacy(
        "cascade",
        "hybrid",
        "implementation_detail",
        "cascade_orchestration",
        "internal_orchestration_pattern",
    ),
    legacy(
        "automl",
        "self_improvable",
        "subtype",
        "bounded_automl",
        "kept_only_under_safety_gated_self_improvement",
    ),
    legacy(
        "neural_parser",
        "computational_programmatic",
        "implementation_detail",
        "legacy_neural_parser_model",
        "legacy_helper_model_not_paradigm",
    ),
    legacy(
        "edge_scorer",
        "neuro_symbolic",
        "implementation_detail",
        "legacy_edge_scorer_model",
        "legacy_helper_model_not_paradigm",
    ),
    legacy(
        "emotion_model",
        "affective_motivational",
        "subtype",
        "affective_regulation_model",
        "kept_as_affective_motivational_subtype",
    ),
    legacy(
        "sleep_trigger",
        "affective_motivational",
        "implementation_detail",
        "legacy_sleep_trigger_model",
        "legacy_regulator_not_paradigm",
    ),
    legacy(
        "death_oracle",
        "alignment_control_safety",
        "subtype",
        "risk_oracle",
        "kept_as_safety_risk_subtype",
    ),
    legacy(
        "crawl_picker",
        "computational_programmatic",
        "implementation_detail",
        "legacy_crawl_picker_model",
        "legacy_tool_selector_not_paradigm",
    ),
    legacy(
        "warden_detector",
        "alignment_control_safety",
        "subtype",
        "warden_detector",
        "kept_as_safety_detection_subtype",
    ),
    legacy(
        "graph_v8",
        "neuro_symbolic",
        "implementation_detail",
        "legacy_graph_v8_model",
        "legacy_graph_helper_not_paradigm",
    ),
];

const fn legacy(
    name: &'static str,
    mapped_paradigm: &'static str,
    status: &'static str,
    canonical: &'static str,
    disposition: &'static str,
) -> LegacyTechnique {
    LegacyTechnique {
        name,
        mapped_paradigm,
        status,
        canonical,
        disposition,
    }
}

fn write_technique_map() -> String {
    let records: Vec<_> = LEGACY_TECHNIQUES
        .iter()
        .map(|technique| {
            serde_json::json!({
                "name": technique.name,
                "mapped_paradigm": technique.mapped_paradigm,
                "status": technique.status,
                "canonical": technique.canonical,
                "disposition": technique.disposition,
                "source": "legacy_paradigm_hub",
                "counts_as_formal_paradigm": false,
            })
        })
        .collect();
    let subtype = count_techniques_by_status("subtype");
    let alias = count_techniques_by_status("alias");
    let implementation_detail = count_techniques_by_status("implementation_detail");
    let archived = count_techniques_by_status("archived");
    let future = count_techniques_by_status("future");
    let replaced = count_techniques_by_status("replaced");
    let formalized = count_techniques_by_status("formalized");
    let record = serde_json::json!({
        "schema": "garm-paradigm-architecture-technique-map-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "legacy_source": "ParadigmHub",
        "legacy_source_status": "superseded_by_paradigm_architecture_eval",
        "runtime_goal": "do_not_maintain_as_separate_paradigm_authority",
        "official_formal_paradigms_after_refinement": 24,
        "promoted_to_new_formal_paradigms": 0,
        "total_legacy_techniques": LEGACY_TECHNIQUES.len(),
        "status_counts": {
            "subtype": subtype,
            "alias": alias,
            "implementation_detail": implementation_detail,
            "archived": archived,
            "future": future,
            "replaced": replaced,
            "formalized": formalized,
        },
        "techniques": records,
    });
    let path = state_paths::paradigm_architecture_technique_map_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "technique_map_written",
        Err(_) => "technique_map_write_failed",
    };
    format!(
        "[PARADIGM-ARCHITECTURE-TECHNIQUE-MAP] legacy_source=ParadigmHub status=superseded techniques={} formal_paradigms=24 promoted=0 subtype={} alias={} implementation_detail={} archived={} future={} replaced={} formalized={} claim_allowed=false write_status={} path={}\n",
        LEGACY_TECHNIQUES.len(),
        subtype,
        alias,
        implementation_detail,
        archived,
        future,
        replaced,
        formalized,
        write_status,
        path
    )
}

fn count_techniques_by_status(status: &str) -> usize {
    LEGACY_TECHNIQUES
        .iter()
        .filter(|technique| technique.status == status)
        .count()
}

fn write_map(input: &ParadigmArchitectureInput) -> String {
    let paradigms = [
        (
            "symbolic_logicist",
            "covered_existing",
            "symbolic_architecture",
        ),
        (
            "connectionist_neural",
            "covered_existing",
            "neural_architecture",
        ),
        (
            "emergentist",
            "new_metric_layer",
            "emergence_metrics_paradigm",
        ),
        ("hybrid", "meta_architecture", "eden_hybrid"),
        (
            "neuro_symbolic",
            "new_formal_layer",
            "neuro_symbolic_paradigm",
        ),
        ("cognitive", "covered_existing", "cognitive_architecture"),
        (
            "embodied_situated",
            "covered_existing",
            "embodied_grounding",
        ),
        (
            "developmental",
            "covered_frontier_layer",
            "developmental_robotics_architecture",
        ),
        (
            "evolutionary",
            "covered_frontier_layer",
            "open_ended_evolution_architecture",
        ),
        (
            "universal_formal",
            "new_formal_layer",
            "universal_formal_paradigm",
        ),
        (
            "probabilistic_bayesian",
            "covered_frontier_layer",
            "probabilistic_programming_architecture",
        ),
        (
            "predictive_active_inference",
            "new_formal_layer",
            "active_inference_paradigm",
        ),
        (
            "reinforcement_learning",
            "replaced_by_safe_hierarchical_model_based",
            "hierarchical_rl_architecture",
        ),
        ("world_models", "covered_existing", "world_eval"),
        (
            "agentic_autonomous",
            "covered_existing",
            "llm_agent_architecture",
        ),
        (
            "multiagent_social",
            "covered_existing",
            "society_agent_mesh",
        ),
        (
            "ecological_systemic",
            "new_formal_layer",
            "ecological_systemic_paradigm",
        ),
        (
            "computational_programmatic",
            "new_formal_layer",
            "computational_programmatic_paradigm",
        ),
        (
            "neuromorphic_neurobiological",
            "covered_frontier_layer",
            "neuromorphic_spiking_architecture",
        ),
        (
            "affective_motivational",
            "new_formal_layer",
            "affective_motivational_paradigm",
        ),
        (
            "metacognitive_reflective",
            "covered_existing",
            "cognitive_architecture",
        ),
        (
            "self_improvable",
            "covered_existing",
            "self_improvement_architecture",
        ),
        (
            "human_in_the_loop_collaborative",
            "new_formal_layer",
            "human_in_the_loop_paradigm",
        ),
        (
            "alignment_control_safety",
            "covered_frontier_layer",
            "safety_control_architecture",
        ),
    ];
    let records: Vec<_> = paradigms
        .iter()
        .map(|(name, decision, mapped_layer)| {
            serde_json::json!({
                "name": name,
                "decision": decision,
                "mapped_layer": mapped_layer,
            })
        })
        .collect();
    let existing_covered = paradigms
        .iter()
        .filter(|(_, decision, _)| {
            matches!(
                *decision,
                "covered_existing" | "covered_frontier_layer" | "meta_architecture"
            )
        })
        .count();
    let new_layers = paradigms
        .iter()
        .filter(|(_, decision, _)| {
            *decision == "new_formal_layer" || *decision == "new_metric_layer"
        })
        .count();
    let replaced = paradigms
        .iter()
        .filter(|(_, decision, _)| decision.starts_with("replaced_by"))
        .count();
    let duplicate_controls = input.symbolic_report.contains("[SYMBOLIC-ARCHITECTURE]")
        && input.neural_report.contains("[NEURAL-ARCHITECTURE]")
        && input.cognitive_report.contains("[COGNITIVE-ARCHITECTURE]")
        && input
            .frontier_report
            .contains("[SAFETY-CONTROL-ARCHITECTURE]");
    let record = serde_json::json!({
        "schema": "garm-paradigm-architecture-map-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "total_paradigms": paradigms.len(),
        "existing_covered": existing_covered,
        "new_layers": new_layers,
        "replaced_or_reinterpreted": replaced,
        "duplicate_controls_present": duplicate_controls,
        "paradigms": records,
    });
    let path = state_paths::paradigm_architecture_map_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "paradigm_map_written",
        Err(_) => "paradigm_map_write_failed",
    };
    format!(
        "[PARADIGM-ARCHITECTURE-MAP] paradigms={} existing_covered={} new_layers={} replaced={} claim_allowed=false write_status={} path={}\n",
        paradigms.len(),
        existing_covered,
        new_layers,
        replaced,
        write_status,
        path
    )
}

fn paradigm_layers(input: &ParadigmArchitectureInput) -> Vec<ParadigmLayer> {
    vec![
        layer(
            "NEURO-SYMBOLIC-PARADIGM",
            "neuro_symbolic_paradigm",
            "neuro_symbolic",
            "implement_formal_bridge_between_existing_neural_and_symbolic_layers",
            state_paths::neuro_symbolic_paradigm_path(),
            vec![
                (
                    "neural_layer_present",
                    input.neural_report.contains("[NEURAL-ARCHITECTURE]"),
                ),
                (
                    "symbolic_layer_present",
                    input.symbolic_report.contains("[SYMBOLIC-ARCHITECTURE]"),
                ),
                (
                    "semantic_logic_causal_bridge",
                    input.capability_status.contains("Semantics:")
                        && input.capability_status.contains("Logic:")
                        && input.capability_status.contains("CausalM:"),
                ),
                (
                    "retrieval_to_reasoning_bridge",
                    input.memory_report.contains("[MEMORY-EVAL]")
                        || input.capability_status.contains("ProgInd:"),
                ),
                (
                    "no_bridge_overclaim",
                    !input.neural_report.contains("agi_claim=true"),
                ),
            ],
        ),
        layer(
            "UNIVERSAL-FORMAL-PARADIGM",
            "universal_formal_paradigm",
            "universal_formal",
            "implement_contracts_invariants_and_claim_boundaries",
            state_paths::universal_formal_paradigm_path(),
            vec![
                (
                    "readiness_invariant_present",
                    input
                        .readiness_report
                        .contains("no_claim_until_all_gates_pass")
                        || input.readiness_report.contains("[READINESS-ARCHITECTURE]"),
                ),
                (
                    "package_claim_boundary",
                    input
                        .external_validation_report
                        .contains("claim_allowed=false"),
                ),
                (
                    "policy_provenance_uncertainty_contract",
                    input.policy_report.contains("[POLICY]")
                        && input.provenance_report.contains("[PROVENANCE]")
                        && input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
                (
                    "validator_or_external_handoff",
                    input
                        .external_validation_report
                        .contains("[EXTERNAL-VALIDATION]")
                        || input
                            .external_validation_report
                            .contains("garm-external-validation-result-v1")
                        || input
                            .external_validation_report
                            .contains("local_held_out_harness"),
                ),
                (
                    "no_formal_agi_claim",
                    !input.external_validation_report.contains("agi_claim=true"),
                ),
            ],
        ),
        layer(
            "ACTIVE-INFERENCE-PARADIGM",
            "active_inference_paradigm",
            "predictive_active_inference",
            "implement_predictive_loop_over_fep_surprise_homeostasis_and_world_models",
            state_paths::active_inference_paradigm_path(),
            vec![
                (
                    "free_energy_or_active_inference_present",
                    input.capability_status.contains("FEP:")
                        || input.capability_status.contains("ActiveInference"),
                ),
                (
                    "surprise_precision_loop_present",
                    input.capability_status.contains("Surprise:")
                        && input.capability_status.contains("Epist:"),
                ),
                (
                    "homeostatic_control_present",
                    input.capability_status.contains("Homeo:"),
                ),
                (
                    "predictive_world_model_present",
                    input.world_report.contains("[WORLD]")
                        || input.capability_status.contains("WMNN:"),
                ),
                (
                    "uncertainty_guided_action",
                    input.uncertainty_report.contains("[UNCERTAINTY]")
                        && input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
            ],
        ),
        layer(
            "ECOLOGICAL-SYSTEMIC-PARADIGM",
            "ecological_systemic_paradigm",
            "ecological_systemic",
            "implement_system_level_stability_over_organs_society_resources_and_policy",
            state_paths::ecological_systemic_paradigm_path(),
            vec![
                (
                    "organ_system_present",
                    input.readiness_report.contains("[ORGANOS-AUDIT]")
                        || input.capability_status.contains("unified="),
                ),
                (
                    "society_or_multiagent_present",
                    input.capability_status.contains("Society:")
                        && input.capability_status.contains("agents="),
                ),
                (
                    "resource_metabolism_present",
                    input.capability_status.contains("Metab:")
                        || input.capability_status.contains("Econ:"),
                ),
                (
                    "systemic_safety_present",
                    input.policy_report.contains("[POLICY]")
                        && input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
                (
                    "readiness_system_gates",
                    input.readiness_report.contains("autonomia_gobernada")
                        && input.readiness_report.contains("seguridad_operacional"),
                ),
            ],
        ),
        layer(
            "COMPUTATIONAL-PROGRAMMATIC-PARADIGM",
            "computational_programmatic_paradigm",
            "computational_programmatic",
            "implement_program_induction_code_as_tool_and_verified_execution",
            state_paths::computational_programmatic_paradigm_path(),
            vec![
                (
                    "program_induction_present",
                    input.capability_status.contains("ProgInd:"),
                ),
                (
                    "tool_surface_present",
                    input.capability_status.contains("Tools:"),
                ),
                (
                    "executor_present",
                    input.plan_executor_report.contains("[EXEC]"),
                ),
                (
                    "policy_guarded_execution",
                    input.policy_report.contains("[POLICY]")
                        && input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
                (
                    "symbolic_program_bridge",
                    input.symbolic_report.contains("[SYMBOLIC-ARCHITECTURE]"),
                ),
            ],
        ),
        layer(
            "AFFECTIVE-MOTIVATIONAL-PARADIGM",
            "affective_motivational_paradigm",
            "affective_motivational",
            "implement_motivation_as_priority_energy_risk_and_exploration_regulation",
            state_paths::affective_motivational_paradigm_path(),
            vec![
                (
                    "motive_present",
                    input.capability_status.contains("Motive:"),
                ),
                (
                    "emotion_modulation_present",
                    input.capability_status.contains("EmoMod:"),
                ),
                (
                    "homeostasis_present",
                    input.capability_status.contains("Homeo:"),
                ),
                (
                    "risk_uncertainty_present",
                    input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
                (
                    "bounded_by_policy",
                    input.policy_report.contains("[POLICY]")
                        && !input.policy_report.contains("claim_allowed=true"),
                ),
            ],
        ),
        layer(
            "HUMAN-IN-THE-LOOP-PARADIGM",
            "human_in_the_loop_paradigm",
            "human_in_the_loop_collaborative",
            "implement_human_review_explanation_correction_and_sensitive_action_approval",
            state_paths::human_in_the_loop_paradigm_path(),
            vec![
                (
                    "human_interface_present",
                    input.capability_status.contains("HumanInterface")
                        || input.readiness_report.contains("READINESS")
                        || input.readiness_report.contains("readiness plan")
                        || input.readiness_report.contains("next actions"),
                ),
                (
                    "sensitive_action_policy_present",
                    input.policy_report.contains("[POLICY]"),
                ),
                (
                    "evidence_explanation_present",
                    input.provenance_report.contains("[PROVENANCE]")
                        && input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
                (
                    "correction_path_present",
                    input.plan_executor_report.contains("[EXEC]")
                        || input.cognitive_report.contains("[COGNITIVE-ARCHITECTURE]"),
                ),
                (
                    "no_autonomous_external_claim",
                    input
                        .external_validation_report
                        .contains("claim_allowed=false"),
                ),
            ],
        ),
        layer(
            "EMERGENCE-METRICS-PARADIGM",
            "emergence_metrics_paradigm",
            "emergentist_metrics",
            "reinterpret_as_metrics_not_a_standalone_module",
            state_paths::emergence_metrics_paradigm_path(),
            vec![
                (
                    "multiagent_coordination_signal",
                    input.capability_status.contains("Society:")
                        || input.capability_status.contains("Swarm:"),
                ),
                (
                    "open_ended_evolution_signal",
                    input
                        .frontier_report
                        .contains("[OPEN-ENDED-EVOLUTION-ARCHITECTURE]")
                        || input.capability_status.contains("MetaEvo:"),
                ),
                (
                    "cross_domain_composition_signal",
                    input.capability_status.contains("Hub:")
                        || input.capability_status.contains("MoE:"),
                ),
                (
                    "evaluation_feedback_signal",
                    input.readiness_report.contains("evaluacion_continua")
                        || input.cognitive_report.contains("[COGNITIVE-ARCHITECTURE]"),
                ),
                (
                    "traceable_non_claim_metric",
                    input
                        .external_validation_report
                        .contains("claim_allowed=false"),
                ),
            ],
        ),
    ]
}

fn layer(
    tag: &'static str,
    artifact: &'static str,
    paradigm: &'static str,
    decision: &'static str,
    path: String,
    cases: Vec<(&'static str, bool)>,
) -> ParadigmLayer {
    ParadigmLayer {
        tag,
        artifact,
        paradigm,
        decision,
        path,
        cases,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paradigm_architecture_writes_map_and_gap_layers() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_paradigm_architecture_test_{}",
            std::process::id()
        )));
        let input = ParadigmArchitectureInput {
            capability_status: "Semantics: 1 Logic: Logic CausalM: SCM FEP: FEP Surprise: Surprise Epist: Epistemic Homeo: Homeo WMNN: WorldModelNN Society: Society agents=7 Metab: Metabolism Econ: AutonomyEcon ProgInd: ProgInduct Tools: 8 Motive: curiosity EmoMod: Emotion Hub: Hub MoE: MoE Swarm: balanced HumanInterface".to_string(),
            readiness_report: "[READINESS-ARCHITECTURE] invariant=no_claim_until_all_gates_pass autonomia_gobernada seguridad_operacional evaluacion_continua next actions readiness plan [ORGANOS-AUDIT]".to_string(),
            cognitive_report: "[COGNITIVE-ARCHITECTURE] passed=5/5".to_string(),
            embodied_report: "[EMBODIED-GROUNDING] passed=5/5".to_string(),
            neural_report: "[NEURAL-ARCHITECTURE] passed=6/6 agi_claim=false".to_string(),
            symbolic_report: "[SYMBOLIC-ARCHITECTURE] passed=6/6".to_string(),
            self_improvement_report: "[SELF-IMPROVEMENT-ARCHITECTURE] passed=6/6".to_string(),
            frontier_report: "[SAFETY-CONTROL-ARCHITECTURE] passed=5/5\n[OPEN-ENDED-EVOLUTION-ARCHITECTURE] passed=5/5".to_string(),
            world_report: "[WORLD] schema=world-model-core-v1\n[WORLD-EVAL] passed=5/5".to_string(),
            memory_report: "[MEMORY-EVAL] passed=5/5".to_string(),
            plan_executor_report: "[EXEC] schema=plan-executor-v1".to_string(),
            policy_report: "[POLICY] blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
            external_validation_report:
                "[EXTERNAL-VALIDATION] passed=60/60 claim_allowed=false agi_claim=false"
                    .to_string(),
        };
        let out = run(input);
        assert!(out.contains("[PARADIGM-ARCHITECTURE-MAP] paradigms=24"));
        assert!(out.contains(
            "[PARADIGM-ARCHITECTURE-TECHNIQUE-MAP] legacy_source=ParadigmHub status=superseded techniques=43"
        ));
        assert!(
            out.contains("[NEURO-SYMBOLIC-PARADIGM] artifact=neuro_symbolic_paradigm passed=5/5")
        );
        assert!(out.contains(
            "[EMERGENCE-METRICS-PARADIGM] artifact=emergence_metrics_paradigm passed=5/5"
        ));
        assert!(std::fs::metadata(state_paths::paradigm_architecture_map_path()).is_ok());
        assert!(std::fs::metadata(state_paths::paradigm_architecture_technique_map_path()).is_ok());
        assert!(std::fs::metadata(state_paths::human_in_the_loop_paradigm_path()).is_ok());
    }
}
