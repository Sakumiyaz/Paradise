use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct MemoryEvalInput {
    pub facts: Vec<String>,
    pub retrieval_report: String,
}

pub fn run(input: MemoryEvalInput) -> String {
    let scenarios = [
        (
            "positive_recall",
            has_any(&input.facts, &["phase", "local", "evidence"]),
        ),
        (
            "negative_abstention",
            input.retrieval_report.contains("abstentions=")
                || input
                    .retrieval_report
                    .contains("generation_restricted=true"),
        ),
        (
            "distractor_resistance",
            !has_any(
                &input.facts,
                &["unsupported_external_claim", "fabricated_remote_fact"],
            ),
        ),
        (
            "temporal_source_preference",
            has_any(&input.facts, &["phase5", "phase4", "readiness_phase5"]),
        ),
        (
            "contradiction_handling",
            !has_conflicting_claims(&input.facts, "eden_readiness_claim"),
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
        "schema": "garm-memory-eval-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "facts": input.facts.len(),
        "passed": passed,
        "total": total,
        "verdict": if passed == total { "robust_local_memory" } else { "needs_memory_evidence" },
        "cases": cases,
    });
    let path = state_paths::memory_eval_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "memory_eval_written",
        Err(_) => "memory_eval_write_failed",
    };
    format!(
        "[MEMORY-EVAL] passed={}/{} facts={} claim_allowed=false write_status={} path={}\n",
        passed,
        total,
        input.facts.len(),
        write_status,
        path
    )
}

fn has_any(facts: &[String], needles: &[&str]) -> bool {
    facts.iter().any(|fact| {
        let lower = fact.to_ascii_lowercase();
        needles.iter().any(|needle| lower.contains(needle))
    })
}

fn has_conflicting_claims(facts: &[String], subject: &str) -> bool {
    let mut positive = false;
    let mut negative = false;
    for fact in facts {
        let lower = fact.to_ascii_lowercase();
        if lower.contains(subject) && lower.contains("supported") {
            positive = true;
        }
        if lower.contains(subject) && lower.contains("unsupported") {
            negative = true;
        }
    }
    positive && negative
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_eval_writes_no_claim_result() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(
            std::env::temp_dir().join(format!("eden_garm_memory_eval_test_{}", std::process::id())),
        );
        let out = run(MemoryEvalInput {
            facts: vec!["phase5 local evidence supported".to_string()],
            retrieval_report: "abstentions=1 generation_restricted=true".to_string(),
        });
        assert!(out.contains("[MEMORY-EVAL]"));
        assert!(out.contains("claim_allowed=false"));
        assert!(std::fs::metadata(state_paths::memory_eval_path()).is_ok());
    }
}
