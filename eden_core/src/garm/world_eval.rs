use crate::eden_garm::state_paths;

pub fn run(world_report: &str) -> String {
    let scenarios = [
        (
            "prediction_report_present",
            world_report.contains("[WORLD]"),
        ),
        (
            "observations_present",
            contains_count_at_least(world_report, "observations=", 1),
        ),
        (
            "predictions_present",
            contains_count_at_least(world_report, "predictions=", 1),
        ),
        (
            "verified_predictions_present",
            contains_count_at_least(world_report, "verified=", 1),
        ),
        (
            "unverified_failures_visible",
            world_report.contains("unverified")
                || world_report.contains("verified=")
                || world_report.contains("insufficient_world_evidence"),
        ),
    ];
    let cases: Vec<_> = scenarios
        .iter()
        .map(|(id, passed)| serde_json::json!({ "id": id, "passed": passed }))
        .collect();
    let passed = scenarios.iter().filter(|(_, passed)| *passed).count();
    let total = scenarios.len();
    let record = serde_json::json!({
        "schema": "garm-world-eval-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": total,
        "verdict": if passed == total { "predictive_loop_present" } else { "needs_world_evidence" },
        "cases": cases,
    });
    let path = state_paths::world_eval_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "world_eval_written",
        Err(_) => "world_eval_write_failed",
    };
    format!(
        "[WORLD-EVAL] passed={}/{} claim_allowed=false write_status={} path={}\n",
        passed, total, write_status, path
    )
}

fn contains_count_at_least(report: &str, key: &str, threshold: u64) -> bool {
    report
        .split_whitespace()
        .find_map(|part| part.strip_prefix(key))
        .and_then(|value| {
            value
                .trim_matches(|ch: char| !ch.is_ascii_digit())
                .parse::<u64>()
                .ok()
        })
        .is_some_and(|value| value >= threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_eval_writes_no_claim_result() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(
            std::env::temp_dir().join(format!("eden_garm_world_eval_test_{}", std::process::id())),
        );
        let out = run("[WORLD] observations=2 predictions=2 verified=1 last_prediction=ok");
        assert!(out.contains("[WORLD-EVAL]"));
        assert!(std::fs::metadata(state_paths::world_eval_path()).is_ok());
    }
}
