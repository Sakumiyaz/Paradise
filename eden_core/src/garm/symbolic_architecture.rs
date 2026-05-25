use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct SymbolicArchitectureInput {
    pub graph_report: String,
    pub capability_status: String,
    pub logic_report: String,
    pub goals_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub world_report: String,
}

pub fn run(input: SymbolicArchitectureInput) -> String {
    let scenarios = [
        (
            "hypergraph_symbol_table",
            input.graph_report.contains("nodes=")
                && input.graph_report.contains("edges=")
                && input.capability_status.contains("Semantics:"),
        ),
        (
            "formal_logic_reasoning",
            input.logic_report.contains("Logic |")
                && input.logic_report.contains("rules=")
                && input.logic_report.contains("inferences="),
        ),
        (
            "explicit_goal_policy_contracts",
            input.goals_report.contains("[GOALS]") && input.policy_report.contains("[POLICY]"),
        ),
        (
            "provenance_traceability",
            input.provenance_report.contains("[PROVENANCE]")
                && input.provenance_report.contains("records="),
        ),
        (
            "causal_world_symbols",
            input.capability_status.contains("Causal:")
                && input.capability_status.contains("CausalM:")
                && input.world_report.contains("[WORLD]"),
        ),
        (
            "neuro_symbolic_bridge",
            input.capability_status.contains("Transformer:")
                && input.capability_status.contains("Logic:"),
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
        "schema": "garm-symbolic-architecture-v1",
        "architecture": "symbolic_agi_layer",
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": total,
        "verdict": if passed == total { "symbolic_architecture_ready_local" } else { "needs_symbolic_evidence" },
        "layers": [
            "hypergraph_symbol_table",
            "formal_logic_reasoning",
            "explicit_goal_policy_contracts",
            "provenance_traceability",
            "causal_world_symbols",
            "neuro_symbolic_bridge"
        ],
        "cases": cases,
    });
    let path = state_paths::symbolic_architecture_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "symbolic_architecture_written",
        Err(_) => "symbolic_architecture_write_failed",
    };
    format!(
        "[SYMBOLIC-ARCHITECTURE] passed={}/{} claim_allowed=false write_status={} path={}\n",
        passed, total, write_status, path
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbolic_architecture_writes_no_claim_result() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_symbolic_architecture_test_{}",
            std::process::id()
        )));
        let out = run(SymbolicArchitectureInput {
            graph_report: "nodes=123 edges=510".to_string(),
            capability_status:
                "Semantics: vocab=10 | Causal: 2 | CausalM: SCM | Transformer: T | Logic: Logic |"
                    .to_string(),
            logic_report: "Logic | facts=2 | rules=1 | inferences=1".to_string(),
            goals_report: "[GOALS] actions=1".to_string(),
            policy_report: "[POLICY] decisions=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            world_report: "[WORLD] predictions=1".to_string(),
        });
        assert!(out.contains("[SYMBOLIC-ARCHITECTURE]"));
        assert!(out.contains("passed=6/6"));
        assert!(out.contains("claim_allowed=false"));
        assert!(std::fs::metadata(state_paths::symbolic_architecture_path()).is_ok());
    }
}
