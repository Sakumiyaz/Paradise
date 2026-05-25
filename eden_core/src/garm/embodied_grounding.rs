use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct EmbodiedGroundingInput {
    pub body_before: String,
    pub body_after: String,
    pub world_before: String,
    pub world_after: String,
    pub world_model_report: String,
    pub action_evidence_report: String,
    pub grounding_facts: usize,
    pub physics_updates: u64,
}

pub fn run(input: EmbodiedGroundingInput) -> String {
    let body_steps = metric_after(&input.body_after, "steps=").unwrap_or(0);
    let world_objects = metric_after(&input.world_after, "objects=").unwrap_or(0);
    let world_steps = metric_after(&input.world_after, "steps=").unwrap_or(0);
    let scenarios = [
        (
            "sensorimotor_loop",
            input.body_before.contains("Body |")
                && input.body_after.contains("Body |")
                && body_steps > 0
                && input.body_after.contains("sensors="),
        ),
        (
            "simulated_world_consequence",
            input.world_before.contains("World3D |")
                && input.world_after.contains("World3D |")
                && world_objects > 0
                && world_steps > 0,
        ),
        (
            "world_model_feedback",
            input.world_model_report.contains("[WORLD]")
                && input.world_model_report.contains("predictions="),
        ),
        (
            "action_evidence_grounding",
            input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
        ),
        (
            "physical_grounding_bridge",
            input.grounding_facts > 0 || input.physics_updates > 0 || world_objects > 0,
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
        "schema": "garm-embodied-grounding-v1",
        "architecture": "embodied_agi_layer",
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": total,
        "verdict": if passed == total { "embodied_grounding_ready_local" } else { "needs_embodied_grounding_evidence" },
        "body_before": input.body_before,
        "body_after": input.body_after,
        "world_before": input.world_before,
        "world_after": input.world_after,
        "grounding_facts": input.grounding_facts,
        "physics_updates": input.physics_updates,
        "layers": [
            "sensorimotor_loop",
            "body_proprioception",
            "simulated_world_consequence",
            "world_model_feedback",
            "physical_grounding_bridge"
        ],
        "cases": cases,
    });
    let path = state_paths::embodied_grounding_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "embodied_grounding_written",
        Err(_) => "embodied_grounding_write_failed",
    };
    format!(
        "[EMBODIED-GROUNDING] passed={}/{} claim_allowed=false write_status={} path={}\n",
        passed, total, write_status, path
    )
}

fn metric_after(report: &str, key: &str) -> Option<u64> {
    report
        .split_whitespace()
        .find_map(|part| part.strip_prefix(key))
        .and_then(|value| {
            value
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .ok()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embodied_grounding_writes_no_claim_result() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_embodied_grounding_test_{}",
            std::process::id()
        )));
        let out = run(EmbodiedGroundingInput {
            body_before: "Body | pos=(0.0,0.0) theta=0.0 | sensors=[] | steps=0".to_string(),
            body_after: "Body | pos=(0.1,0.0) theta=0.0 | sensors=[\"0.1\"] | steps=3".to_string(),
            world_before: "World3D | objects=0 | steps=0 | collisions=0 | total_KE=0.0".to_string(),
            world_after: "World3D | objects=2 | steps=10 | collisions=0 | total_KE=1.0".to_string(),
            world_model_report: "[WORLD] predictions=1".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
            grounding_facts: 1,
            physics_updates: 0,
        });
        assert!(out.contains("[EMBODIED-GROUNDING]"));
        assert!(out.contains("passed=5/5"));
        assert!(out.contains("claim_allowed=false"));
        assert!(std::fs::metadata(state_paths::embodied_grounding_path()).is_ok());
    }
}
