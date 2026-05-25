use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct CognitiveArchitectureInput {
    pub attention_report: String,
    pub memory_eval_report: String,
    pub goals_report: String,
    pub plan_executor_report: String,
    pub metacognition_report: String,
    pub policy_report: String,
    pub evaluation_report: String,
}

pub fn run(input: CognitiveArchitectureInput) -> String {
    let scenarios = [
        (
            "working_memory_attention",
            input.attention_report.contains("[ATTENTION]")
                && input.attention_report.contains("schema=working-memory-v1")
                && !input.attention_report.contains("top=none"),
        ),
        (
            "episodic_semantic_memory",
            input.memory_eval_report.contains("[MEMORY-EVAL]")
                && !input.memory_eval_report.contains("passed=0"),
        ),
        (
            "executive_goal_control",
            input.goals_report.contains("[GOALS]")
                && input.plan_executor_report.contains("[EXEC]")
                && input.policy_report.contains("[POLICY]"),
        ),
        (
            "metacognitive_monitoring",
            input.metacognition_report.contains("Meta |")
                && input.metacognition_report.contains("self_err="),
        ),
        (
            "evaluation_feedback_loop",
            input.evaluation_report.contains("[EVAL]")
                && input
                    .evaluation_report
                    .contains("schema=evaluation-loop-v1"),
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
        "schema": "garm-cognitive-architecture-v1",
        "architecture": "cognitive_architecture_agi_layer",
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": total,
        "verdict": if passed == total { "cognitive_architecture_ready_local" } else { "needs_cognitive_evidence" },
        "layers": [
            "attention_control",
            "working_memory",
            "episodic_semantic_memory",
            "executive_goal_control",
            "metacognitive_monitoring",
            "evaluation_feedback"
        ],
        "cases": cases,
    });
    let path = state_paths::cognitive_architecture_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "cognitive_architecture_written",
        Err(_) => "cognitive_architecture_write_failed",
    };
    format!(
        "[COGNITIVE-ARCHITECTURE] passed={}/{} claim_allowed=false write_status={} path={}\n",
        passed, total, write_status, path
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cognitive_architecture_writes_no_claim_result() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_cognitive_architecture_test_{}",
            std::process::id()
        )));
        let out = run(CognitiveArchitectureInput {
            attention_report:
                "[ATTENTION] schema=working-memory-v1 items=1 focus_shifts=1 clears=0 top=1:0.90:test"
                    .to_string(),
            memory_eval_report: "[MEMORY-EVAL] passed=5/5".to_string(),
            goals_report: "[GOALS] actions=1".to_string(),
            plan_executor_report: "[EXEC] plans=1".to_string(),
            metacognition_report: "Meta | swarm_err=1.000 | self_err=1.000".to_string(),
            policy_report: "[POLICY] decisions=1".to_string(),
            evaluation_report: "[EVAL] schema=evaluation-loop-v1 runs=1".to_string(),
        });
        assert!(out.contains("[COGNITIVE-ARCHITECTURE]"));
        assert!(out.contains("passed=5/5"));
        assert!(out.contains("claim_allowed=false"));
        assert!(std::fs::metadata(state_paths::cognitive_architecture_path()).is_ok());
    }
}
