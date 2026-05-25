use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct FrontierArchitectureInput {
    pub capability_status: String,
    pub cognitive_report: String,
    pub embodied_report: String,
    pub neural_report: String,
    pub symbolic_report: String,
    pub self_improvement_report: String,
    pub world_report: String,
    pub hrm_text_report: String,
    pub plan_executor_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
    pub external_validation_report: String,
}

struct FrontierLayer {
    tag: &'static str,
    artifact: &'static str,
    layer: &'static str,
    parent: &'static str,
    path: String,
    cases: Vec<(&'static str, bool)>,
}

pub fn run(input: FrontierArchitectureInput) -> String {
    let layers = frontier_layers(&input);
    let mut out = String::new();
    for layer in layers {
        let passed = layer.cases.iter().filter(|(_, passed)| *passed).count();
        let total = layer.cases.len();
        let cases: Vec<_> = layer
            .cases
            .iter()
            .map(|(id, passed)| serde_json::json!({ "id": id, "passed": passed }))
            .collect();
        let record = serde_json::json!({
            "schema": "garm-frontier-architecture-layer-v1",
            "architecture": layer.layer,
            "parent_layer": layer.parent,
            "claim_allowed": false,
            "agi_claim": false,
            "validation_scope": "formal_architecture_layer_local_evidence",
            "passed": passed,
            "total": total,
            "verdict": if passed == total { "frontier_layer_ready_local" } else { "needs_frontier_layer_evidence" },
            "cases": cases,
        });
        let _ = state_paths::ensure_state_dir();
        let write_status = match std::fs::write(
            &layer.path,
            serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
        ) {
            Ok(()) => "frontier_layer_written",
            Err(_) => "frontier_layer_write_failed",
        };
        out.push_str(&format!(
            "[{}] artifact={} passed={}/{} claim_allowed=false write_status={} path={}\n",
            layer.tag, layer.artifact, passed, total, write_status, layer.path
        ));
    }
    out
}

fn frontier_layers(input: &FrontierArchitectureInput) -> Vec<FrontierLayer> {
    vec![
        layer(
            "SAFETY-CONTROL-ARCHITECTURE",
            "safety_control_architecture",
            "safety_control_architecture_layer",
            "hybrid_agi",
            state_paths::safety_control_architecture_path(),
            vec![
                (
                    "policy_gate_present",
                    input.policy_report.contains("[POLICY]")
                        && input.policy_report.contains("blocked="),
                ),
                (
                    "action_evidence_present",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
                (
                    "provenance_uncertainty_present",
                    input.provenance_report.contains("[PROVENANCE]")
                        && input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
                (
                    "external_claim_boundary",
                    input
                        .external_validation_report
                        .contains("claim_allowed=false"),
                ),
                (
                    "no_local_agi_claim",
                    !input.external_validation_report.contains("agi_claim=true"),
                ),
            ],
        ),
        layer(
            "FOUNDATION-MODEL-ARCHITECTURE",
            "foundation_model_architecture",
            "foundation_model_architecture_layer",
            "neural_agi",
            state_paths::foundation_model_architecture_path(),
            vec![
                (
                    "transformer_backbone_present",
                    input.capability_status.contains("BigTF:")
                        && input.capability_status.contains("Transformer:"),
                ),
                (
                    "text_prior_ingestion_seam",
                    input.hrm_text_report.contains("[HRM-TEXT]"),
                ),
                (
                    "neural_artifact_present",
                    input.neural_report.contains("[NEURAL-ARCHITECTURE]"),
                ),
                (
                    "foundation_claim_bounded",
                    input.neural_report.contains("weights_present=false")
                        || input.hrm_text_report.contains("weights_present=false"),
                ),
                (
                    "no_training_overclaim",
                    !input.neural_report.contains("training_executed=true"),
                ),
            ],
        ),
        layer(
            "MULTIMODAL-MODEL-ARCHITECTURE",
            "multimodal_model_architecture",
            "multimodal_model_architecture_layer",
            "embodied_agi",
            state_paths::multimodal_model_architecture_path(),
            vec![
                (
                    "perception_stack_present",
                    input.capability_status.contains("Perception:"),
                ),
                (
                    "text_vision_voice_channels",
                    input.capability_status.contains("text=")
                        && input.capability_status.contains("vision=")
                        && input.capability_status.contains("voice="),
                ),
                (
                    "grounding_bridge_present",
                    input.embodied_report.contains("[EMBODIED-GROUNDING]"),
                ),
                (
                    "neural_multimodal_bridge",
                    input.neural_report.contains("[NEURAL-ARCHITECTURE]"),
                ),
                (
                    "no_multimodal_claim",
                    !input.capability_status.contains("multimodal_claim=true"),
                ),
            ],
        ),
        layer(
            "LLM-AGENT-ARCHITECTURE",
            "llm_agent_architecture",
            "llm_agent_architecture_layer",
            "agentic_agi",
            state_paths::llm_agent_architecture_path(),
            vec![
                (
                    "foundation_model_seam",
                    input.capability_status.contains("BigTF:")
                        || input.hrm_text_report.contains("[HRM-TEXT]"),
                ),
                (
                    "agentic_tool_loop",
                    input.capability_status.contains("Tools:")
                        && input.plan_executor_report.contains("[EXEC]"),
                ),
                (
                    "policy_guarded_agent_actions",
                    input.policy_report.contains("[POLICY]")
                        && input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
                (
                    "memory_retrieval_context",
                    input.hrm_text_report.contains("retrieval_hits=")
                        || input.hrm_text_report.contains("context_packs="),
                ),
                (
                    "no_remote_llm_claim",
                    !input.capability_status.contains("remote_llm_claim=true"),
                ),
            ],
        ),
        layer(
            "PROBABILISTIC-PROGRAMMING-ARCHITECTURE",
            "probabilistic_programming_architecture",
            "probabilistic_programming_architecture_layer",
            "hybrid_agi",
            state_paths::probabilistic_programming_architecture_path(),
            vec![
                (
                    "program_induction_present",
                    input.capability_status.contains("ProgInd:"),
                ),
                (
                    "uncertainty_model_present",
                    input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
                (
                    "provenance_trace_present",
                    input.provenance_report.contains("[PROVENANCE]"),
                ),
                (
                    "symbolic_bridge_present",
                    input.symbolic_report.contains("[SYMBOLIC-ARCHITECTURE]"),
                ),
                (
                    "probabilistic_claim_bounded",
                    !input
                        .capability_status
                        .contains("probabilistic_program_executed=true"),
                ),
            ],
        ),
        layer(
            "HIERARCHICAL-RL-ARCHITECTURE",
            "hierarchical_rl_architecture",
            "hierarchical_rl_architecture_layer",
            "world_model_agi",
            state_paths::hierarchical_rl_architecture_path(),
            vec![
                (
                    "hierarchical_attention_present",
                    input.capability_status.contains("Hier:"),
                ),
                (
                    "temporal_hierarchy_present",
                    input.capability_status.contains("TempHier:"),
                ),
                (
                    "reward_oracle_present",
                    input.capability_status.contains("Oracle:"),
                ),
                (
                    "model_based_control_bridge",
                    input.world_report.contains("[WORLD]")
                        || input.capability_status.contains("WMNN:"),
                ),
                (
                    "no_policy_learning_claim",
                    !input.capability_status.contains("policy_trained=true"),
                ),
            ],
        ),
        layer(
            "COGNITIVE-ROBOTICS-ARCHITECTURE",
            "cognitive_robotics_architecture",
            "cognitive_robotics_architecture_layer",
            "embodied_agi",
            state_paths::cognitive_robotics_architecture_path(),
            vec![
                ("body_present", input.capability_status.contains("Body:")),
                ("world3d_present", input.capability_status.contains("3D:")),
                (
                    "physics_present",
                    input.capability_status.contains("Physics:"),
                ),
                (
                    "embodied_grounding_present",
                    input.embodied_report.contains("[EMBODIED-GROUNDING]"),
                ),
                (
                    "real_robot_claim_bounded",
                    !input.capability_status.contains("real_robot_claim=true"),
                ),
            ],
        ),
        layer(
            "VLA-ARCHITECTURE",
            "vla_architecture",
            "vision_language_action_architecture_layer",
            "embodied_agi",
            state_paths::vla_architecture_path(),
            vec![
                (
                    "vision_channel_present",
                    input.capability_status.contains("Perception:")
                        && input.capability_status.contains("vision="),
                ),
                (
                    "language_channel_present",
                    input.capability_status.contains("Lang:")
                        || input.hrm_text_report.contains("[HRM-TEXT]"),
                ),
                (
                    "action_channel_present",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
                (
                    "grounded_action_loop",
                    input.embodied_report.contains("[EMBODIED-GROUNDING]"),
                ),
                (
                    "vla_claim_bounded",
                    !input
                        .capability_status
                        .contains("vla_benchmark_passed=true"),
                ),
            ],
        ),
        layer(
            "SIM-TO-REAL-ARCHITECTURE",
            "sim_to_real_architecture",
            "sim_to_real_architecture_layer",
            "embodied_agi",
            state_paths::sim_to_real_architecture_path(),
            vec![
                (
                    "simulation_world_present",
                    input.capability_status.contains("3D:"),
                ),
                (
                    "physics_sim_present",
                    input.capability_status.contains("Physics:"),
                ),
                (
                    "world_model_trace_present",
                    input.world_report.contains("[WORLD]")
                        || input.embodied_report.contains("[EMBODIED-GROUNDING]"),
                ),
                (
                    "policy_and_action_boundary",
                    input.policy_report.contains("[POLICY]")
                        && input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
                (
                    "real_transfer_claim_bounded",
                    !input
                        .capability_status
                        .contains("sim_to_real_validated=true"),
                ),
            ],
        ),
        layer(
            "OPEN-ENDED-EVOLUTION-ARCHITECTURE",
            "open_ended_evolution_architecture",
            "open_ended_evolution_architecture_layer",
            "autopoietic_agi",
            state_paths::open_ended_evolution_architecture_path(),
            vec![
                (
                    "evolution_state_present",
                    input.capability_status.contains("Evo:"),
                ),
                (
                    "meta_evolution_present",
                    input.capability_status.contains("MetaEvo:"),
                ),
                (
                    "bounded_self_improvement_present",
                    input
                        .self_improvement_report
                        .contains("[SELF-IMPROVEMENT-ARCHITECTURE]"),
                ),
                (
                    "autopoietic_runtime_boundary",
                    input.capability_status.contains("SelfMod:")
                        || input.capability_status.contains("Homeo:"),
                ),
                (
                    "open_ended_claim_bounded",
                    !input
                        .capability_status
                        .contains("open_ended_validated=true"),
                ),
            ],
        ),
        layer(
            "DEVELOPMENTAL-ROBOTICS-ARCHITECTURE",
            "developmental_robotics_architecture",
            "developmental_robotics_architecture_layer",
            "embodied_agi",
            state_paths::developmental_robotics_architecture_path(),
            vec![
                (
                    "motivation_present",
                    input.capability_status.contains("Motive:"),
                ),
                (
                    "body_learning_loop_present",
                    input.capability_status.contains("Body:"),
                ),
                (
                    "developmental_self_improvement_present",
                    input
                        .self_improvement_report
                        .contains("[SELF-IMPROVEMENT-ARCHITECTURE]"),
                ),
                (
                    "curriculum_or_readiness_trace",
                    input
                        .external_validation_report
                        .contains("[EXTERNAL-VALIDATION]")
                        || input.hrm_text_report.contains("[HRM-TEXT]"),
                ),
                (
                    "developmental_claim_bounded",
                    !input
                        .capability_status
                        .contains("developmental_robotics_validated=true"),
                ),
            ],
        ),
        layer(
            "WHOLE-BRAIN-NEUROCOGNITIVE-ARCHITECTURE",
            "whole_brain_neurocognitive_architecture",
            "whole_brain_neurocognitive_architecture_layer",
            "neural_agi",
            state_paths::whole_brain_neurocognitive_architecture_path(),
            vec![
                (
                    "cognitive_architecture_present",
                    input.cognitive_report.contains("[COGNITIVE-ARCHITECTURE]"),
                ),
                (
                    "neural_architecture_present",
                    input.neural_report.contains("[NEURAL-ARCHITECTURE]"),
                ),
                (
                    "phenomenology_present",
                    input.capability_status.contains("Phenom:"),
                ),
                (
                    "differentiable_memory_present",
                    input.capability_status.contains("DNC:"),
                ),
                (
                    "whole_brain_claim_bounded",
                    !input
                        .capability_status
                        .contains("whole_brain_validated=true"),
                ),
            ],
        ),
        layer(
            "NEUROMORPHIC-SPIKING-ARCHITECTURE",
            "neuromorphic_spiking_architecture",
            "neuromorphic_spiking_architecture_layer",
            "neural_agi",
            state_paths::neuromorphic_spiking_architecture_path(),
            vec![
                (
                    "neural_architecture_present",
                    input.neural_report.contains("[NEURAL-ARCHITECTURE]"),
                ),
                (
                    "recurrent_state_present",
                    input.capability_status.contains("Recur:"),
                ),
                (
                    "temporal_event_state_present",
                    input.capability_status.contains("TempHier:"),
                ),
                (
                    "bptt_temporal_training_seam",
                    input.capability_status.contains("BPTT:"),
                ),
                (
                    "spiking_backend_claim_bounded",
                    !input
                        .capability_status
                        .contains("spiking_backend_validated=true"),
                ),
            ],
        ),
    ]
}

fn layer(
    tag: &'static str,
    artifact: &'static str,
    layer: &'static str,
    parent: &'static str,
    path: String,
    cases: Vec<(&'static str, bool)>,
) -> FrontierLayer {
    FrontierLayer {
        tag,
        artifact,
        layer,
        parent,
        path,
        cases,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontier_layers_write_formal_no_claim_artifacts() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_frontier_layers_test_{}",
            std::process::id()
        )));
        let out = run(FrontierArchitectureInput {
            capability_status: "BigTF: ok Transformer: ok Perception: text=12 vision=64 voice=13 Tools: 8 ProgInd: ok Hier: ok TempHier: ok Oracle: ok WMNN: ok Body: ok 3D: ok Physics: ok Evo: ok MetaEvo: ok SelfMod: ok Homeo: ok Motive: curiosity Lang: ok Phenom: ok DNC: ok Recur: ok BPTT: ok".to_string(),
            cognitive_report: "[COGNITIVE-ARCHITECTURE] passed=5/5".to_string(),
            embodied_report: "[EMBODIED-GROUNDING] passed=5/5".to_string(),
            neural_report: "[NEURAL-ARCHITECTURE] passed=6/6 weights_present=false training_executed=false".to_string(),
            symbolic_report: "[SYMBOLIC-ARCHITECTURE] passed=6/6".to_string(),
            self_improvement_report: "[SELF-IMPROVEMENT-ARCHITECTURE] passed=6/6".to_string(),
            world_report: "[WORLD] observations=1 predictions=1 verified=1".to_string(),
            hrm_text_report: "[HRM-TEXT] retrieval_hits=1 context_packs=1 weights_present=false".to_string(),
            plan_executor_report: "[EXEC] plans=1 rolled_back=0".to_string(),
            policy_report: "[POLICY] decisions=1 blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
            external_validation_report:
                "[EXTERNAL-VALIDATION] passed=60/60 claim_allowed=false agi_claim=false"
                    .to_string(),
        });
        assert!(out.contains("[SAFETY-CONTROL-ARCHITECTURE]"));
        assert!(out.contains("[NEUROMORPHIC-SPIKING-ARCHITECTURE]"));
        assert_eq!(out.matches("passed=5/5").count(), 13);
        assert!(std::fs::metadata(state_paths::safety_control_architecture_path()).is_ok());
        assert!(std::fs::metadata(state_paths::neuromorphic_spiking_architecture_path()).is_ok());
    }
}
