use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct CapabilityRegistryInput {
    pub readiness_report: String,
    pub benchmark_report: String,
    pub memory_eval_report: String,
    pub world_eval_report: String,
    pub cognitive_architecture_report: String,
    pub embodied_grounding_report: String,
    pub neural_architecture_report: String,
    pub symbolic_architecture_report: String,
    pub self_improvement_architecture_report: String,
    pub frontier_architecture_report: String,
    pub paradigm_architecture_report: String,
    pub integration_governance_report: String,
    pub global_executive_workspace_report: String,
    pub gewc_operational_benchmark_report: String,
    pub capability_reality_report: String,
    pub architecture_advantage_report: String,
    pub praxis_nexus_report: String,
    pub external_ecosystem_report: String,
    pub sovereign_cognition_report: String,
    pub runtime_state_api_report: String,
    pub operational_api_report: String,
    pub artifact_api_report: String,
    pub action_evidence_report: String,
    pub external_validation_report: String,
}

pub fn audit(input: CapabilityRegistryInput) -> String {
    let capabilities = [
        (
            "verifiable_rag",
            input.readiness_report.contains("rag_verificable")
                && input.external_validation_report.contains("passed="),
        ),
        (
            "robust_memory",
            input.memory_eval_report.contains("passed=")
                && !input.memory_eval_report.contains("passed=0"),
        ),
        (
            "predictive_world_model",
            input.world_eval_report.contains("passed=")
                && !input.world_eval_report.contains("passed=0"),
        ),
        (
            "cognitive_architecture",
            input
                .cognitive_architecture_report
                .contains("[COGNITIVE-ARCHITECTURE]")
                && !input.cognitive_architecture_report.contains("passed=0"),
        ),
        (
            "embodied_grounding",
            input
                .embodied_grounding_report
                .contains("[EMBODIED-GROUNDING]")
                && !input.embodied_grounding_report.contains("passed=0"),
        ),
        (
            "neural_architecture",
            input
                .neural_architecture_report
                .contains("[NEURAL-ARCHITECTURE]")
                && !input.neural_architecture_report.contains("passed=0"),
        ),
        (
            "symbolic_architecture",
            input
                .symbolic_architecture_report
                .contains("[SYMBOLIC-ARCHITECTURE]")
                && !input.symbolic_architecture_report.contains("passed=0"),
        ),
        (
            "self_improvement_architecture",
            input
                .self_improvement_architecture_report
                .contains("[SELF-IMPROVEMENT-ARCHITECTURE]")
                && !input
                    .self_improvement_architecture_report
                    .contains("passed=0"),
        ),
        (
            "safety_control_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[SAFETY-CONTROL-ARCHITECTURE]",
            ),
        ),
        (
            "foundation_model_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[FOUNDATION-MODEL-ARCHITECTURE]",
            ),
        ),
        (
            "multimodal_model_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[MULTIMODAL-MODEL-ARCHITECTURE]",
            ),
        ),
        (
            "llm_agent_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[LLM-AGENT-ARCHITECTURE]",
            ),
        ),
        (
            "probabilistic_programming_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[PROBABILISTIC-PROGRAMMING-ARCHITECTURE]",
            ),
        ),
        (
            "hierarchical_rl_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[HIERARCHICAL-RL-ARCHITECTURE]",
            ),
        ),
        (
            "cognitive_robotics_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[COGNITIVE-ROBOTICS-ARCHITECTURE]",
            ),
        ),
        (
            "vla_architecture",
            frontier_validated(&input.frontier_architecture_report, "[VLA-ARCHITECTURE]"),
        ),
        (
            "sim_to_real_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[SIM-TO-REAL-ARCHITECTURE]",
            ),
        ),
        (
            "open_ended_evolution_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[OPEN-ENDED-EVOLUTION-ARCHITECTURE]",
            ),
        ),
        (
            "developmental_robotics_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[DEVELOPMENTAL-ROBOTICS-ARCHITECTURE]",
            ),
        ),
        (
            "whole_brain_neurocognitive_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[WHOLE-BRAIN-NEUROCOGNITIVE-ARCHITECTURE]",
            ),
        ),
        (
            "neuromorphic_spiking_architecture",
            frontier_validated(
                &input.frontier_architecture_report,
                "[NEUROMORPHIC-SPIKING-ARCHITECTURE]",
            ),
        ),
        (
            "governed_action",
            input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
        ),
        (
            "integration_governance_architecture",
            input
                .integration_governance_report
                .contains("[INTEGRATION-GOVERNANCE-ARCHITECTURE]")
                && !input.integration_governance_report.contains("passed=0"),
        ),
        (
            "global_executive_workspace_core",
            input
                .global_executive_workspace_report
                .contains("[GLOBAL-EXECUTIVE-WORKSPACE-CORE]")
                && !input.global_executive_workspace_report.contains("passed=0"),
        ),
        (
            "global_executive_workspace_runtime",
            input
                .global_executive_workspace_report
                .contains("[GEWC-RUNTIME]")
                && !input
                    .global_executive_workspace_report
                    .contains("decisions=0")
                && !input
                    .global_executive_workspace_report
                    .contains("completions=0")
                && input
                    .global_executive_workspace_report
                    .contains("body_executor=gewc_body_executor")
                && input
                    .global_executive_workspace_report
                    .contains("handler_dispatch=domain_handler_dispatch")
                && input
                    .global_executive_workspace_report
                    .contains("handler_topology=domain_owned_body_implementations")
                && input
                    .global_executive_workspace_report
                    .contains("shared_body_engine=false")
                && input
                    .global_executive_workspace_report
                    .contains("handler_metrics=")
                && !input
                    .global_executive_workspace_report
                    .contains("last_handler=none"),
        ),
        (
            "gewc_operational_benchmark",
            input
                .gewc_operational_benchmark_report
                .contains("[GEWC-OPERATIONAL-BENCHMARK] passed=8/8")
                && input
                    .gewc_operational_benchmark_report
                    .contains("[GEWC-RUNTIME-SAFETY] passed=5/5")
                && input
                    .gewc_operational_benchmark_report
                    .contains("[GEWC-LONG-RUN-STABILITY] passed=5/5")
                && input
                    .gewc_operational_benchmark_report
                    .contains("claim_allowed=false"),
        ),
        (
            "capability_reality_eval",
            input
                .capability_reality_report
                .contains("[CAPABILITY-REALITY-EVAL]")
                && input
                    .capability_reality_report
                    .contains("[LMM-TRAINING-DEPENDENCY]")
                && input
                    .capability_reality_report
                    .contains("claim_allowed=false"),
        ),
        (
            "architecture_advantage_eval",
            input
                .architecture_advantage_report
                .contains("[ARCHITECTURE-ADVANTAGE-EVAL] movements=6/6")
                && input
                    .architecture_advantage_report
                    .contains("[GEWC-TRACE-SPEC]")
                && input
                    .architecture_advantage_report
                    .contains("[EDEN-AGENT-SDK]")
                && input
                    .architecture_advantage_report
                    .contains("claim_allowed=false"),
        ),
        (
            "eden_praxis_nexus",
            input.praxis_nexus_report.contains("[EDEN-PRAXIS-NEXUS]")
                && input.praxis_nexus_report.contains("primitives=7/7")
                && input.praxis_nexus_report.contains("blocks=5/5")
                && input.praxis_nexus_report.contains("claim_allowed=false"),
        ),
        (
            "eden_external_ecosystem",
            input
                .external_ecosystem_report
                .contains("[EDEN-EXTERNAL-ECOSYSTEM] domains=6/6")
                && input
                    .external_ecosystem_report
                    .contains("[ECOSYSTEM-PARTICIPATION-CONTRACT]")
                && input
                    .external_ecosystem_report
                    .contains("[ECOSYSTEM-BENCHMARK-EXCHANGE]")
                && input
                    .external_ecosystem_report
                    .contains("claim_allowed=false"),
        ),
        (
            "eden_sovereign_cognition",
            input
                .sovereign_cognition_report
                .contains("[EDEN-SOVEREIGN-COGNITION]")
                && input.sovereign_cognition_report.contains("sectors=11/11")
                && input
                    .sovereign_cognition_report
                    .contains("claim_allowed=false"),
        ),
        (
            "artifact_api",
            input.artifact_api_report.contains("[ARTIFACT-API]")
                && input.artifact_api_report.contains("artifacts=")
                && input.artifact_api_report.contains("claim_allowed=false"),
        ),
        (
            "runtime_state_api",
            input
                .runtime_state_api_report
                .contains("[RUNTIME-STATE-API]")
                && input.runtime_state_api_report.contains("states=")
                && input
                    .runtime_state_api_report
                    .contains("claim_allowed=false"),
        ),
        (
            "operational_api",
            input.operational_api_report.contains("[OPERATIONAL-API]")
                && input.operational_api_report.contains("surfaces=")
                && input
                    .operational_api_report
                    .contains("action_mutation_allowed=false")
                && input.operational_api_report.contains("claim_allowed=false"),
        ),
        (
            "paradigm_architecture_map",
            paradigm_validated(
                &input.paradigm_architecture_report,
                "[PARADIGM-ARCHITECTURE-MAP]",
            ),
        ),
        (
            "paradigm_architecture_technique_map",
            paradigm_validated(
                &input.paradigm_architecture_report,
                "[PARADIGM-ARCHITECTURE-TECHNIQUE-MAP]",
            ),
        ),
        (
            "neuro_symbolic_paradigm",
            paradigm_validated(
                &input.paradigm_architecture_report,
                "[NEURO-SYMBOLIC-PARADIGM]",
            ),
        ),
        (
            "universal_formal_paradigm",
            paradigm_validated(
                &input.paradigm_architecture_report,
                "[UNIVERSAL-FORMAL-PARADIGM]",
            ),
        ),
        (
            "active_inference_paradigm",
            paradigm_validated(
                &input.paradigm_architecture_report,
                "[ACTIVE-INFERENCE-PARADIGM]",
            ),
        ),
        (
            "ecological_systemic_paradigm",
            paradigm_validated(
                &input.paradigm_architecture_report,
                "[ECOLOGICAL-SYSTEMIC-PARADIGM]",
            ),
        ),
        (
            "computational_programmatic_paradigm",
            paradigm_validated(
                &input.paradigm_architecture_report,
                "[COMPUTATIONAL-PROGRAMMATIC-PARADIGM]",
            ),
        ),
        (
            "affective_motivational_paradigm",
            paradigm_validated(
                &input.paradigm_architecture_report,
                "[AFFECTIVE-MOTIVATIONAL-PARADIGM]",
            ),
        ),
        (
            "human_in_the_loop_paradigm",
            paradigm_validated(
                &input.paradigm_architecture_report,
                "[HUMAN-IN-THE-LOOP-PARADIGM]",
            ),
        ),
        (
            "emergence_metrics_paradigm",
            paradigm_validated(
                &input.paradigm_architecture_report,
                "[EMERGENCE-METRICS-PARADIGM]",
            ),
        ),
        (
            "local_benchmarking",
            input.benchmark_report.contains("[READINESS-BENCH]")
                || input.benchmark_report.contains("[BENCH]"),
        ),
        (
            "external_handoff",
            input
                .external_validation_report
                .contains("claim_allowed=false"),
        ),
    ];
    let records: Vec<_> = capabilities
        .iter()
        .map(|(name, validated)| {
            serde_json::json!({
                "name": name,
                "state": if *validated { "validated_local" } else { "needs_evidence" },
                "needs_external": true,
            })
        })
        .collect();
    let validated = capabilities
        .iter()
        .filter(|(_, validated)| *validated)
        .count();
    let total = capabilities.len();
    let record = serde_json::json!({
        "schema": "garm-capability-registry-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "validated_local": validated,
        "total": total,
        "capabilities": records,
    });
    let path = state_paths::capability_registry_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "registry_written",
        Err(_) => "registry_write_failed",
    };
    format!(
        "[CAPABILITY-REGISTRY] validated_local={}/{} claim_allowed=false write_status={} path={}\n",
        validated, total, write_status, path
    )
}

fn frontier_validated(report: &str, tag: &str) -> bool {
    report
        .lines()
        .any(|line| line.contains(tag) && !line.contains("passed=0"))
}

fn paradigm_validated(report: &str, tag: &str) -> bool {
    report
        .lines()
        .any(|line| line.contains(tag) && !line.contains("passed=0"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_writes_capability_states() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_capability_registry_test_{}",
            std::process::id()
        )));
        let out = audit(CapabilityRegistryInput {
            readiness_report: "rag_verificable".to_string(),
            benchmark_report: "[READINESS-BENCH]".to_string(),
            memory_eval_report: "[MEMORY-EVAL] passed=5/5".to_string(),
            world_eval_report: "[WORLD-EVAL] passed=5/5".to_string(),
            cognitive_architecture_report: "[COGNITIVE-ARCHITECTURE] passed=5/5".to_string(),
            embodied_grounding_report: "[EMBODIED-GROUNDING] passed=5/5".to_string(),
            neural_architecture_report: "[NEURAL-ARCHITECTURE] passed=6/6".to_string(),
            symbolic_architecture_report: "[SYMBOLIC-ARCHITECTURE] passed=6/6".to_string(),
            self_improvement_architecture_report: "[SELF-IMPROVEMENT-ARCHITECTURE] passed=6/6"
                .to_string(),
            frontier_architecture_report:
                "[SAFETY-CONTROL-ARCHITECTURE] passed=5/5\n[FOUNDATION-MODEL-ARCHITECTURE] passed=5/5\n[MULTIMODAL-MODEL-ARCHITECTURE] passed=5/5\n[LLM-AGENT-ARCHITECTURE] passed=5/5\n[PROBABILISTIC-PROGRAMMING-ARCHITECTURE] passed=5/5\n[HIERARCHICAL-RL-ARCHITECTURE] passed=5/5\n[COGNITIVE-ROBOTICS-ARCHITECTURE] passed=5/5\n[VLA-ARCHITECTURE] passed=5/5\n[SIM-TO-REAL-ARCHITECTURE] passed=5/5\n[OPEN-ENDED-EVOLUTION-ARCHITECTURE] passed=5/5\n[DEVELOPMENTAL-ROBOTICS-ARCHITECTURE] passed=5/5\n[WHOLE-BRAIN-NEUROCOGNITIVE-ARCHITECTURE] passed=5/5\n[NEUROMORPHIC-SPIKING-ARCHITECTURE] passed=5/5".to_string(),
            paradigm_architecture_report: "[PARADIGM-ARCHITECTURE-MAP] paradigms=24\n[PARADIGM-ARCHITECTURE-TECHNIQUE-MAP] techniques=43\n[NEURO-SYMBOLIC-PARADIGM] passed=5/5\n[UNIVERSAL-FORMAL-PARADIGM] passed=5/5\n[ACTIVE-INFERENCE-PARADIGM] passed=5/5\n[ECOLOGICAL-SYSTEMIC-PARADIGM] passed=5/5\n[COMPUTATIONAL-PROGRAMMATIC-PARADIGM] passed=5/5\n[AFFECTIVE-MOTIVATIONAL-PARADIGM] passed=5/5\n[HUMAN-IN-THE-LOOP-PARADIGM] passed=5/5\n[EMERGENCE-METRICS-PARADIGM] passed=5/5".to_string(),
            integration_governance_report:
                "[INTEGRATION-GOVERNANCE-ARCHITECTURE] passed=10/10".to_string(),
            global_executive_workspace_report:
                "[GLOBAL-EXECUTIVE-WORKSPACE-CORE] passed=26/26 layers=3/3 model_control_mode=gewc_centric_model_plural_not_llm_centric module_lifecycle=gewc_module_lifecycle_supervisor\n[GEWC-RUNTIME] decisions=1 completions=1 body_executor=gewc_body_executor handler_dispatch=domain_handler_dispatch handler_topology=domain_owned_body_implementations shared_body_engine=false model_control_mode=gewc_centric_model_plural_not_llm_centric module_lifecycle=gewc_module_lifecycle_supervisor last_handler=gewc_validation_body_handler last_primary_model=evaluation_model handler_metrics=gewc_validation_body_handler:d1/c1/b0 model_metrics=evaluation_model:1 lifecycle_metrics=active:1"
                    .to_string(),
            gewc_operational_benchmark_report:
                "[GEWC-OPERATIONAL-BENCHMARK] passed=8/8 claim_allowed=false\n[GEWC-RUNTIME-SAFETY] passed=5/5 claim_allowed=false\n[GEWC-LONG-RUN-STABILITY] passed=5/5 claim_allowed=false"
                    .to_string(),
            capability_reality_report:
                "[CAPABILITY-REALITY-EVAL] executable_current=10/25 claim_allowed=false\n[LMM-TRAINING-DEPENDENCY] requires_training=7"
                    .to_string(),
            architecture_advantage_report:
                "[GEWC-TRACE-SPEC] passed=4/4 claim_allowed=false\n[EDEN-AGENT-SDK] passed=4/4 claim_allowed=false\n[ARCHITECTURE-ADVANTAGE-EVAL] movements=6/6 claim_allowed=false"
                    .to_string(),
            praxis_nexus_report:
                "[EDEN-PRAXIS-NEXUS] goal=governed_cognitive_operational_substrate primitives=7/7 blocks=5/5 claim_allowed=false"
                    .to_string(),
            external_ecosystem_report:
                "[ECOSYSTEM-PARTICIPATION-CONTRACT] passed=4/4 claim_allowed=false\n[ECOSYSTEM-BENCHMARK-EXCHANGE] passed=4/4 claim_allowed=false\n[EDEN-EXTERNAL-ECOSYSTEM] domains=6/6 claim_allowed=false"
                    .to_string(),
            sovereign_cognition_report:
                "[EDEN-SOVEREIGN-COGNITION] sectors=11/11 claim_allowed=false"
                    .to_string(),
            runtime_state_api_report:
                "[RUNTIME-STATE-API] states=42/42 endpoints=4 claim_allowed=false".to_string(),
            operational_api_report:
                "[OPERATIONAL-API] surfaces=10 endpoints=10 action_mutation_allowed=false claim_allowed=false"
                    .to_string(),
            artifact_api_report:
                "[ARTIFACT-API] artifacts=73/73 endpoints=3 claim_allowed=false".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
            external_validation_report: "[EXTERNAL-VALIDATION] passed=69/69 claim_allowed=false"
                .to_string(),
        });
        assert!(out.contains("[CAPABILITY-REGISTRY]"));
        assert!(std::fs::metadata(state_paths::capability_registry_path()).is_ok());
    }
}
