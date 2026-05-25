use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct CapabilityRealityInput {
    pub readiness_report: String,
    pub capability_status: String,
    pub memory_report: String,
    pub world_report: String,
    pub cognitive_report: String,
    pub embodied_report: String,
    pub neural_report: String,
    pub symbolic_report: String,
    pub self_improvement_report: String,
    pub frontier_report: String,
    pub paradigm_report: String,
    pub integration_governance_report: String,
    pub gewc_report: String,
    pub gewc_operational_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub external_validation_report: String,
}

#[derive(Clone)]
struct RealityRow {
    id: &'static str,
    domain: &'static str,
    state: &'static str,
    evidence_present: bool,
    evidence: &'static str,
    limitation: &'static str,
    lmm_training_required: bool,
    external_validation_required: bool,
    safety_blocked: bool,
    operational_now: bool,
}

pub fn run(input: CapabilityRealityInput) -> String {
    let rows = rows(&input);
    let executable_current = rows
        .iter()
        .filter(|row| row.operational_now && row.evidence_present)
        .count();
    let architecture_only = rows
        .iter()
        .filter(|row| row.state == "implemented_architecture_artifact" && row.evidence_present)
        .count();
    let local_heuristic = rows
        .iter()
        .filter(|row| row.state == "implemented_local_heuristic" && row.evidence_present)
        .count();
    let simulated_or_stub = rows
        .iter()
        .filter(|row| row.state == "simulated_or_stub")
        .count();
    let requires_lmm_training = rows.iter().filter(|row| row.lmm_training_required).count();
    let requires_external_validation = rows
        .iter()
        .filter(|row| row.external_validation_required)
        .count();
    let blocked_by_safety_policy = rows.iter().filter(|row| row.safety_blocked).count();

    write_reality_artifact(
        state_paths::capability_reality_eval_path(),
        "garm-capability-reality-eval-v1",
        "capability_reality_eval",
        &rows,
    );
    write_reality_artifact(
        state_paths::capability_reality_matrix_path(),
        "garm-capability-reality-matrix-v1",
        "capability_reality_matrix",
        &rows,
    );
    write_lmm_dependency_report(&rows);

    format!(
        "[CAPABILITY-REALITY-EVAL] executable_current={}/{} architecture_only={} local_heuristic={} simulated_or_stub={} requires_lmm_training={} requires_external_validation={} blocked_by_safety_policy={} claim_allowed=false path={}\n[CAPABILITY-REALITY-MATRIX] entries={} path={}\n[LMM-TRAINING-DEPENDENCY] requires_training={} weights_present=false training_executed=false path={}\n",
        executable_current,
        rows.len(),
        architecture_only,
        local_heuristic,
        simulated_or_stub,
        requires_lmm_training,
        requires_external_validation,
        blocked_by_safety_policy,
        state_paths::capability_reality_eval_path(),
        rows.len(),
        state_paths::capability_reality_matrix_path(),
        requires_lmm_training,
        state_paths::lmm_training_dependency_report_path(),
    )
}

fn rows(input: &CapabilityRealityInput) -> Vec<RealityRow> {
    vec![
        RealityRow {
            id: "gewc_native_core",
            domain: "executive_core",
            state: "implemented_runtime",
            evidence_present: input
                .gewc_report
                .contains("[GLOBAL-EXECUTIVE-WORKSPACE-CORE]")
                && input.gewc_report.contains("[GEWC-RUNTIME]")
                && input.gewc_report.contains("shared_body_engine=false"),
            evidence: "GEWC core artifact and runtime trace",
            limitation: "local runtime evidence, not external AGI certification",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: true,
        },
        RealityRow {
            id: "executive_command_routing",
            domain: "executive_control",
            state: "implemented_runtime",
            evidence_present: input
                .gewc_report
                .contains("handler_dispatch=domain_handler_dispatch")
                && input
                    .gewc_report
                    .contains("handler_topology=domain_owned_body_implementations"),
            evidence: "GEWC handler dispatch and domain-owned bodies",
            limitation: "routing is command/runtime bounded",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: true,
        },
        RealityRow {
            id: "memory_retrieval_eval",
            domain: "memory",
            state: "implemented_runtime",
            evidence_present: input.memory_report.contains("[MEMORY-EVAL]")
                && input.memory_report.contains("passed="),
            evidence: "memory eval artifact",
            limitation: "local corpus and retrieval harness only",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: true,
        },
        RealityRow {
            id: "world_model_causal_loop",
            domain: "world_model",
            state: "implemented_local_heuristic",
            evidence_present: input.world_report.contains("[WORLD")
                && input.world_report.contains("[WORLD-EVAL]"),
            evidence: "world observe/predict/verify report",
            limitation: "causal loop is local heuristic, not a trained physical simulator",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: true,
        },
        RealityRow {
            id: "safe_continual_learning_ledger",
            domain: "learning",
            state: "implemented_runtime",
            evidence_present: input.readiness_report.contains("aprendizaje")
                || input
                    .self_improvement_report
                    .contains("[SELF-IMPROVEMENT-ARCHITECTURE]"),
            evidence: "learning ledger and bounded self-improvement artifact",
            limitation: "updates are governed records, not autonomous model-weight training",
            lmm_training_required: true,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: true,
        },
        RealityRow {
            id: "policy_provenance_uncertainty",
            domain: "safety",
            state: "implemented_runtime",
            evidence_present: input.policy_report.contains("[POLICY]")
                && input.provenance_report.contains("[PROVENANCE]")
                && input.uncertainty_report.contains("[UNCERTAINTY]"),
            evidence: "policy, provenance and uncertainty ledgers",
            limitation: "local policy guard is not a formal proof of safety",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: true,
        },
        RealityRow {
            id: "gewc_operational_benchmark",
            domain: "evaluation",
            state: "implemented_runtime",
            evidence_present: input
                .gewc_operational_report
                .contains("[GEWC-OPERATIONAL-BENCHMARK] passed=8/8")
                && input
                    .gewc_operational_report
                    .contains("[GEWC-RUNTIME-SAFETY] passed=5/5")
                && input
                    .gewc_operational_report
                    .contains("[GEWC-LONG-RUN-STABILITY] passed=5/5"),
            evidence: "operational benchmark, safety and stability artifacts",
            limitation: "local prevalidation only",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: true,
        },
        RealityRow {
            id: "capability_registry",
            domain: "evaluation",
            state: "implemented_runtime",
            evidence_present: input.capability_status.contains("[CAPABILITY-REGISTRY]")
                || input.capability_status.contains("validated_local="),
            evidence: "capability registry artifact",
            limitation: "registry states are evidence bookkeeping, not capability proof",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: true,
        },
        RealityRow {
            id: "cognitive_architecture",
            domain: "architecture",
            state: "implemented_architecture_artifact",
            evidence_present: input.cognitive_report.contains("[COGNITIVE-ARCHITECTURE]"),
            evidence: "cognitive architecture artifact",
            limitation: "formal architecture artifact, not full cognitive performance proof",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "embodied_grounding",
            domain: "grounding",
            state: "implemented_architecture_artifact",
            evidence_present: input.embodied_report.contains("[EMBODIED-GROUNDING]"),
            evidence: "embodied grounding artifact",
            limitation: "no physical robot/sensor validation in current local run",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "physical_social_grounding_runtime",
            domain: "grounding",
            state: "simulated_or_stub",
            evidence_present: input.embodied_report.contains("[EMBODIED-GROUNDING]"),
            evidence: "embodied grounding plan exists",
            limitation:
                "physical/social grounding is represented, not connected to real sensors or humans",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "symbolic_reasoning_architecture",
            domain: "reasoning",
            state: "implemented_architecture_artifact",
            evidence_present: input.symbolic_report.contains("[SYMBOLIC-ARCHITECTURE]"),
            evidence: "symbolic architecture artifact",
            limitation: "symbolic layer is not a complete theorem-proving AGI",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "neural_architecture",
            domain: "neural",
            state: "requires_lmm_training",
            evidence_present: input.neural_report.contains("[NEURAL-ARCHITECTURE]"),
            evidence: "neural architecture artifact",
            limitation: "EDEN LMM weights are not trained in this validation path",
            lmm_training_required: true,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "foundation_and_multimodal_models",
            domain: "foundation_models",
            state: "requires_lmm_training",
            evidence_present: input
                .frontier_report
                .contains("[FOUNDATION-MODEL-ARCHITECTURE]")
                && input
                    .frontier_report
                    .contains("[MULTIMODAL-MODEL-ARCHITECTURE]"),
            evidence: "foundation and multimodal architecture artifacts",
            limitation: "architecture is present; trained model capability is not",
            lmm_training_required: true,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "llm_agent_architecture",
            domain: "agentic_llm",
            state: "requires_lmm_training",
            evidence_present: input.frontier_report.contains("[LLM-AGENT-ARCHITECTURE]"),
            evidence: "LLM agent architecture artifact",
            limitation: "agent scaffolding exists, but current LMM training is absent",
            lmm_training_required: true,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "probabilistic_programming",
            domain: "probabilistic",
            state: "implemented_architecture_artifact",
            evidence_present: input
                .frontier_report
                .contains("[PROBABILISTIC-PROGRAMMING-ARCHITECTURE]"),
            evidence: "probabilistic programming architecture artifact",
            limitation: "formal layer only; no broad probabilistic benchmark yet",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "hierarchical_rl",
            domain: "reinforcement_learning",
            state: "implemented_architecture_artifact",
            evidence_present: input
                .frontier_report
                .contains("[HIERARCHICAL-RL-ARCHITECTURE]"),
            evidence: "hierarchical RL architecture artifact",
            limitation: "no trained RL policy is validated by this local run",
            lmm_training_required: true,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "robotics_vla_sim_to_real",
            domain: "embodied_frontier",
            state: "requires_lmm_training",
            evidence_present: input
                .frontier_report
                .contains("[COGNITIVE-ROBOTICS-ARCHITECTURE]")
                && input.frontier_report.contains("[VLA-ARCHITECTURE]")
                && input.frontier_report.contains("[SIM-TO-REAL-ARCHITECTURE]"),
            evidence: "robotics, VLA and sim-to-real architecture artifacts",
            limitation: "no robot, VLA weights or sim-to-real transfer benchmark is executed",
            lmm_training_required: true,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "developmental_open_ended_evolution",
            domain: "open_ended_learning",
            state: "implemented_architecture_artifact",
            evidence_present: input
                .frontier_report
                .contains("[OPEN-ENDED-EVOLUTION-ARCHITECTURE]")
                && input
                    .frontier_report
                    .contains("[DEVELOPMENTAL-ROBOTICS-ARCHITECTURE]"),
            evidence: "open-ended evolution and developmental robotics artifacts",
            limitation: "not executed as open-ended autonomous evolution",
            lmm_training_required: true,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "whole_brain_neuromorphic",
            domain: "neurocognitive",
            state: "implemented_architecture_artifact",
            evidence_present: input
                .frontier_report
                .contains("[WHOLE-BRAIN-NEUROCOGNITIVE-ARCHITECTURE]")
                && input
                    .frontier_report
                    .contains("[NEUROMORPHIC-SPIKING-ARCHITECTURE]"),
            evidence: "whole-brain/neurocognitive and neuromorphic artifacts",
            limitation: "architectural representation only; no spiking runtime benchmark",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "paradigm_architecture_eval",
            domain: "paradigms",
            state: "implemented_architecture_artifact",
            evidence_present: input
                .paradigm_report
                .contains("[PARADIGM-ARCHITECTURE-MAP]")
                && input
                    .paradigm_report
                    .contains("[PARADIGM-ARCHITECTURE-TECHNIQUE-MAP]"),
            evidence: "24-paradigm map and technique absorption artifact",
            limitation: "taxonomy coherence, not direct task performance",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "integration_governance",
            domain: "governance",
            state: "implemented_architecture_artifact",
            evidence_present: input
                .integration_governance_report
                .contains("[INTEGRATION-GOVERNANCE-ARCHITECTURE]"),
            evidence: "integration governance artifact",
            limitation: "governance contract is local and needs external review",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "local_external_validation_harness",
            domain: "validation",
            state: "implemented_local_heuristic",
            evidence_present: input
                .external_validation_report
                .contains("[EXTERNAL-VALIDATION]")
                && input
                    .external_validation_report
                    .contains("claim_allowed=false"),
            evidence: "local held-out validation harness",
            limitation: "not independent third-party validation",
            lmm_training_required: false,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: true,
        },
        RealityRow {
            id: "external_agi_claim",
            domain: "claims",
            state: "requires_external_validation",
            evidence_present: input
                .external_validation_report
                .contains("claim_allowed=false"),
            evidence: "no-claim validation result",
            limitation: "claim remains blocked until independent validation and benchmarks exist",
            lmm_training_required: true,
            external_validation_required: true,
            safety_blocked: false,
            operational_now: false,
        },
        RealityRow {
            id: "unguarded_open_actions",
            domain: "safety",
            state: "blocked_by_safety_policy",
            evidence_present: input.policy_report.contains("blocked="),
            evidence: "policy guard blocks high-risk actions",
            limitation:
                "open shell/network/self-modification actions require explicit permission and audit",
            lmm_training_required: false,
            external_validation_required: false,
            safety_blocked: true,
            operational_now: false,
        },
    ]
}

fn write_reality_artifact(path: String, schema: &str, artifact: &str, rows: &[RealityRow]) {
    let records: Vec<_> = rows.iter().map(row_record).collect();
    let executable_current = rows
        .iter()
        .filter(|row| row.operational_now && row.evidence_present)
        .count();
    let record = serde_json::json!({
        "schema": schema,
        "artifact": artifact,
        "claim_allowed": false,
        "agi_claim": false,
        "lmm_trained": false,
        "validation_scope": "current_architecture_and_operational_capabilities_only",
        "summary": {
            "executable_current": executable_current,
            "total": rows.len(),
            "implemented_runtime": count_state(rows, "implemented_runtime"),
            "implemented_local_heuristic": count_state(rows, "implemented_local_heuristic"),
            "implemented_architecture_artifact": count_state(rows, "implemented_architecture_artifact"),
            "simulated_or_stub": count_state(rows, "simulated_or_stub"),
            "requires_lmm_training": rows.iter().filter(|row| row.lmm_training_required).count(),
            "requires_external_validation": rows.iter().filter(|row| row.external_validation_required).count(),
            "blocked_by_safety_policy": rows.iter().filter(|row| row.safety_blocked).count(),
        },
        "status_definitions": {
            "implemented_runtime": "executable local EDEN runtime behavior exists",
            "implemented_local_heuristic": "local heuristic or harness behavior exists but is not a trained general capability",
            "implemented_architecture_artifact": "formal architecture artifact exists without direct operational proof",
            "simulated_or_stub": "represented or simulated but not grounded in real runtime capability",
            "requires_lmm_training": "depends on EDEN LMM/model training before capability claims",
            "requires_external_validation": "requires independent benchmarks or third-party review",
            "blocked_by_safety_policy": "intentionally unavailable without permission/sandbox/audit"
        },
        "capabilities": records,
    });
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
}

fn write_lmm_dependency_report(rows: &[RealityRow]) {
    let dependencies: Vec<_> = rows
        .iter()
        .filter(|row| row.lmm_training_required)
        .map(row_record)
        .collect();
    let record = serde_json::json!({
        "schema": "garm-lmm-training-dependency-report-v1",
        "artifact": "lmm_training_dependency_report",
        "claim_allowed": false,
        "agi_claim": false,
        "weights_present": false,
        "training_executed": false,
        "dependency_count": dependencies.len(),
        "blocked_claims": [
            "foundation_model_capability",
            "multimodal_understanding",
            "VLA_control",
            "open_ended_autonomous_learning",
            "external_AGI_claim"
        ],
        "dependencies": dependencies,
    });
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        state_paths::lmm_training_dependency_report_path(),
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
}

fn row_record(row: &RealityRow) -> serde_json::Value {
    serde_json::json!({
        "id": row.id,
        "domain": row.domain,
        "state": row.state,
        "evidence_present": row.evidence_present,
        "evidence": row.evidence,
        "limitation": row.limitation,
        "lmm_training_required": row.lmm_training_required,
        "external_validation_required": row.external_validation_required,
        "safety_blocked": row.safety_blocked,
        "operational_now": row.operational_now,
    })
}

fn count_state(rows: &[RealityRow], state: &str) -> usize {
    rows.iter().filter(|row| row.state == state).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_reality_eval_matrix_and_lmm_dependency_artifacts() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_capability_reality_test_{}",
            std::process::id()
        )));
        let out = run(CapabilityRealityInput {
            readiness_report: "READINESS aprendizaje".to_string(),
            capability_status: "[CAPABILITY-REGISTRY] validated_local=38/38".to_string(),
            memory_report: "[MEMORY-EVAL] passed=5/5".to_string(),
            world_report: "[WORLD] predictions=1\n[WORLD-EVAL] passed=5/5".to_string(),
            cognitive_report: "[COGNITIVE-ARCHITECTURE] passed=5/5".to_string(),
            embodied_report: "[EMBODIED-GROUNDING] passed=5/5".to_string(),
            neural_report: "[NEURAL-ARCHITECTURE] passed=6/6".to_string(),
            symbolic_report: "[SYMBOLIC-ARCHITECTURE] passed=6/6".to_string(),
            self_improvement_report: "[SELF-IMPROVEMENT-ARCHITECTURE] passed=6/6".to_string(),
            frontier_report: "[FOUNDATION-MODEL-ARCHITECTURE] passed=5/5\n[MULTIMODAL-MODEL-ARCHITECTURE] passed=5/5\n[LLM-AGENT-ARCHITECTURE] passed=5/5\n[PROBABILISTIC-PROGRAMMING-ARCHITECTURE] passed=5/5\n[HIERARCHICAL-RL-ARCHITECTURE] passed=5/5\n[COGNITIVE-ROBOTICS-ARCHITECTURE] passed=5/5\n[VLA-ARCHITECTURE] passed=5/5\n[SIM-TO-REAL-ARCHITECTURE] passed=5/5\n[OPEN-ENDED-EVOLUTION-ARCHITECTURE] passed=5/5\n[DEVELOPMENTAL-ROBOTICS-ARCHITECTURE] passed=5/5\n[WHOLE-BRAIN-NEUROCOGNITIVE-ARCHITECTURE] passed=5/5\n[NEUROMORPHIC-SPIKING-ARCHITECTURE] passed=5/5".to_string(),
            paradigm_report: "[PARADIGM-ARCHITECTURE-MAP] paradigms=24\n[PARADIGM-ARCHITECTURE-TECHNIQUE-MAP] techniques=43".to_string(),
            integration_governance_report: "[INTEGRATION-GOVERNANCE-ARCHITECTURE] passed=10/10".to_string(),
            gewc_report: "[GLOBAL-EXECUTIVE-WORKSPACE-CORE] passed=26/26 model_control_mode=gewc_centric_model_plural_not_llm_centric module_lifecycle=gewc_module_lifecycle_supervisor\n[GEWC-RUNTIME] shared_body_engine=false handler_dispatch=domain_handler_dispatch handler_topology=domain_owned_body_implementations model_control_mode=gewc_centric_model_plural_not_llm_centric module_lifecycle=gewc_module_lifecycle_supervisor".to_string(),
            gewc_operational_report: "[GEWC-OPERATIONAL-BENCHMARK] passed=8/8\n[GEWC-RUNTIME-SAFETY] passed=5/5\n[GEWC-LONG-RUN-STABILITY] passed=5/5".to_string(),
            policy_report: "[POLICY] allowed=1 blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            external_validation_report: "[EXTERNAL-VALIDATION] passed=69/69 claim_allowed=false".to_string(),
        });
        assert!(out.contains("[CAPABILITY-REALITY-EVAL]"));
        assert!(out.contains("claim_allowed=false"));
        assert!(out.contains("[LMM-TRAINING-DEPENDENCY]"));
        assert!(std::fs::metadata(state_paths::capability_reality_eval_path()).is_ok());
        assert!(std::fs::metadata(state_paths::capability_reality_matrix_path()).is_ok());
        assert!(std::fs::metadata(state_paths::lmm_training_dependency_report_path()).is_ok());
        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
