use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct IntegrationGovernanceInput {
    pub readiness_report: String,
    pub capability_status: String,
    pub cognitive_report: String,
    pub embodied_report: String,
    pub self_improvement_report: String,
    pub frontier_report: String,
    pub paradigm_report: String,
    pub world_report: String,
    pub goals_report: String,
    pub plan_executor_report: String,
    pub learning_report: String,
    pub evaluation_report: String,
    pub benchmark_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
    pub external_validation_report: String,
}

pub fn run(input: IntegrationGovernanceInput) -> String {
    let cases = [
        (
            "module_integration_mechanism",
            input
                .paradigm_report
                .contains("[PARADIGM-ARCHITECTURE-MAP]")
                && input
                    .frontier_report
                    .contains("[SAFETY-CONTROL-ARCHITECTURE]")
                && input.capability_status.contains("Hub:"),
        ),
        (
            "central_executive_control",
            input.goals_report.contains("[GOALS]")
                && input.plan_executor_report.contains("[EXEC]")
                && input.policy_report.contains("[POLICY]"),
        ),
        (
            "safe_continual_learning",
            input.learning_report.contains("[LEARNING]")
                && input
                    .self_improvement_report
                    .contains("[SELF-IMPROVEMENT-ARCHITECTURE]")
                && input.policy_report.contains("[POLICY]"),
        ),
        (
            "physical_social_grounding",
            input.embodied_report.contains("[EMBODIED-GROUNDING]")
                && input.capability_status.contains("Society:")
                && input.capability_status.contains("Body:"),
        ),
        (
            "causal_world_model",
            input.world_report.contains("[WORLD]")
                && input.world_report.contains("[WORLD-EVAL]")
                && input.capability_status.contains("CausalM:"),
        ),
        (
            "robust_metacognition",
            input.cognitive_report.contains("[COGNITIVE-ARCHITECTURE]")
                && input.uncertainty_report.contains("[UNCERTAINTY]")
                && input.evaluation_report.contains("[EVAL]"),
        ),
        (
            "stable_correctable_goals",
            input.goals_report.contains("[GOALS]")
                && input.plan_executor_report.contains("[EXEC]")
                && input.provenance_report.contains("[PROVENANCE]"),
        ),
        (
            "complete_agi_evaluation",
            input.readiness_report.contains("READINESS")
                && input.benchmark_report.contains("[READINESS-BENCH]")
                && claim_disallowed(&input.external_validation_report),
        ),
        (
            "scalable_alignment",
            input
                .frontier_report
                .contains("[SAFETY-CONTROL-ARCHITECTURE]")
                && input.policy_report.contains("[POLICY]")
                && input.uncertainty_report.contains("[UNCERTAINTY]"),
        ),
        (
            "action_governance_boundaries",
            input.action_evidence_report.contains("[ACTION-EVIDENCE]")
                && input.policy_report.contains("[POLICY]")
                && !input.external_validation_report.contains("agi_claim=true"),
        ),
    ];
    let passed = cases.iter().filter(|(_, passed)| *passed).count();
    let total = cases.len();
    let case_records: Vec<_> = cases
        .iter()
        .map(|(id, passed)| serde_json::json!({ "id": id, "passed": passed }))
        .collect();
    let record = serde_json::json!({
        "schema": "garm-integration-governance-architecture-v1",
        "architecture": "integration_governance_architecture",
        "claim_allowed": false,
        "agi_claim": false,
        "validation_scope": "formal_integration_governance_local_evidence",
        "passed": passed,
        "total": total,
        "verdict": if passed == total { "integration_governance_ready_local" } else { "needs_integration_governance_evidence" },
        "cases": case_records,
    });
    let path = state_paths::integration_governance_architecture_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "integration_governance_written",
        Err(_) => "integration_governance_write_failed",
    };
    format!(
        "[INTEGRATION-GOVERNANCE-ARCHITECTURE] passed={}/{} claim_allowed=false write_status={} path={}\n",
        passed, total, write_status, path
    )
}

fn claim_disallowed(report: &str) -> bool {
    report.contains("claim_allowed=false") || report.contains("\"claim_allowed\": false")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_governance_writes_no_claim_artifact() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_integration_governance_test_{}",
            std::process::id()
        )));
        let out = run(IntegrationGovernanceInput {
            readiness_report: "READINESS".to_string(),
            capability_status: "Hub: Hub Society: Society Body: Body CausalM: SCM".to_string(),
            cognitive_report: "[COGNITIVE-ARCHITECTURE] passed=5/5".to_string(),
            embodied_report: "[EMBODIED-GROUNDING] passed=5/5".to_string(),
            self_improvement_report: "[SELF-IMPROVEMENT-ARCHITECTURE] passed=6/6".to_string(),
            frontier_report: "[SAFETY-CONTROL-ARCHITECTURE] passed=5/5".to_string(),
            paradigm_report: "[PARADIGM-ARCHITECTURE-MAP] paradigms=24".to_string(),
            world_report: "[WORLD]\n[WORLD-EVAL] passed=5/5".to_string(),
            goals_report: "[GOALS] goals=1".to_string(),
            plan_executor_report: "[EXEC] plans=1".to_string(),
            learning_report: "[LEARNING] entries=1".to_string(),
            evaluation_report: "[EVAL] runs=1".to_string(),
            benchmark_report: "[READINESS-BENCH] gates=6 passed=6".to_string(),
            policy_report: "[POLICY] blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
            external_validation_report: "[EXTERNAL-VALIDATION] claim_allowed=false agi_claim=false"
                .to_string(),
        });
        assert!(out.contains("[INTEGRATION-GOVERNANCE-ARCHITECTURE]"));
        assert!(out.contains("passed=10/10"));
        assert!(std::fs::metadata(state_paths::integration_governance_architecture_path()).is_ok());
    }
}
