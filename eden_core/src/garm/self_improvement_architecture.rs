use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct SelfImprovementArchitectureInput {
    pub self_status: String,
    pub self_improvement_status: String,
    pub plan_executor_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
}

pub fn run(input: SelfImprovementArchitectureInput) -> String {
    let scenarios = [
        (
            "bounded_parameter_proposal",
            input.self_status.contains("SelfMod |")
                && (input.self_status.contains("proposals=")
                    || input.self_improvement_status.contains("last_proposals=")),
        ),
        (
            "self_improvement_audit_loop",
            input.self_improvement_status.contains("SelfImprovement |")
                && input.self_improvement_status.contains("audits="),
        ),
        (
            "policy_gate_blocks_code_mutation",
            input.policy_report.contains("[POLICY]")
                && (input.policy_report.contains("blocked=")
                    || input.action_evidence_report.contains("blocked")),
        ),
        (
            "rollback_executor_available",
            input.plan_executor_report.contains("[EXEC]")
                && (input.plan_executor_report.contains("rolled_back=")
                    || input.plan_executor_report.contains("rollback")),
        ),
        (
            "provenance_uncertainty_trace",
            input.provenance_report.contains("[PROVENANCE]")
                && input.uncertainty_report.contains("[UNCERTAINTY]"),
        ),
        (
            "no_source_mutation_claim",
            input.action_evidence_report.contains("[ACTION-EVIDENCE]")
                && !input
                    .action_evidence_report
                    .contains("source_code_mutated=true"),
        ),
    ];
    let cases: Vec<_> = scenarios
        .iter()
        .map(|(id, passed)| {
            serde_json::json!({
                "id": id,
                "passed": passed,
            })
        })
        .collect();
    let passed = scenarios.iter().filter(|(_, passed)| *passed).count();
    let total = scenarios.len();
    let record = serde_json::json!({
        "schema": "garm-self-improvement-architecture-v1",
        "architecture": "self_improving_agi_layer",
        "claim_allowed": false,
        "agi_claim": false,
        "source_code_mutation_allowed": false,
        "passed": passed,
        "total": total,
        "verdict": if passed == total { "self_improvement_ready_local" } else { "needs_self_improvement_evidence" },
        "layers": [
            "bounded_parameter_proposal",
            "self_improvement_audit_loop",
            "policy_gate_blocks_code_mutation",
            "rollback_executor_available",
            "provenance_uncertainty_trace",
            "no_source_mutation_claim"
        ],
        "cases": cases,
    });
    let path = state_paths::self_improvement_architecture_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "self_improvement_architecture_written",
        Err(_) => "self_improvement_architecture_write_failed",
    };
    format!(
        "[SELF-IMPROVEMENT-ARCHITECTURE] passed={}/{} source_code_mutation_allowed=false claim_allowed=false write_status={} path={}\n",
        passed, total, write_status, path
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_improvement_architecture_writes_no_claim_result() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_self_improvement_architecture_test_{}",
            std::process::id()
        )));
        let out = run(SelfImprovementArchitectureInput {
            self_status: "SelfMod | proposals=1 | applied=1 | rejected=0".to_string(),
            self_improvement_status:
                "SelfImprovement | audits=1 | applied=1 | reverted=0 | last_proposals=1".to_string(),
            plan_executor_report: "[EXEC] completed=1 rolled_back=1 rollback=available".to_string(),
            policy_report: "[POLICY] decisions=1 blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            action_evidence_report:
                "[ACTION-EVIDENCE] source=self_improvement source_code_mutated=false".to_string(),
        });
        assert!(out.contains("[SELF-IMPROVEMENT-ARCHITECTURE]"));
        assert!(out.contains("passed=6/6"));
        assert!(out.contains("source_code_mutation_allowed=false"));
        assert!(std::fs::metadata(state_paths::self_improvement_architecture_path()).is_ok());
    }
}
