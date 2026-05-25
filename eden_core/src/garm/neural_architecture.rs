use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct NeuralArchitectureInput {
    pub capability_status: String,
    pub hrm_text_report: String,
    pub checkpoint_manifest: String,
    pub retrieval_report: String,
}

pub fn run(input: NeuralArchitectureInput) -> String {
    let scenarios = [
        (
            "transformer_backbone",
            input.capability_status.contains("Transformer |")
                && input.capability_status.contains("BigTransformer |"),
        ),
        (
            "neural_memory_and_experts",
            input.capability_status.contains("MoE |")
                && input.capability_status.contains("BPTT |")
                && input.capability_status.contains("DNC |"),
        ),
        (
            "semantic_embedding_bridge",
            input.capability_status.contains("Semantics:")
                || input.capability_status.contains("vocab="),
        ),
        (
            "hrm_text_prior_manifest",
            input.hrm_text_report.contains("[HRM-TEXT]")
                && input
                    .checkpoint_manifest
                    .contains("schema=hrm-text-checkpoint-v1"),
        ),
        (
            "retrieval_inference_loop",
            input.retrieval_report.contains("[HRM-TEXT-SEARCH]")
                || input.retrieval_report.contains("[HRM-TEXT-CONTEXT-PACK]"),
        ),
        (
            "claim_safe_weight_policy",
            input.checkpoint_manifest.contains("weights_present=false")
                && input
                    .checkpoint_manifest
                    .contains("training_executed=false"),
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
        "schema": "garm-neural-architecture-v1",
        "architecture": "neural_agi_layer",
        "claim_allowed": false,
        "agi_claim": false,
        "weights_present": false,
        "training_executed": false,
        "passed": passed,
        "total": total,
        "verdict": if passed == total { "neural_architecture_ready_local" } else { "needs_neural_evidence" },
        "layers": [
            "transformer_backbone",
            "big_transformer_adapters",
            "moe_bptt_dnc_memory",
            "semantic_embedding_bridge",
            "hrm_text_prior_manifest",
            "retrieval_inference_loop"
        ],
        "cases": cases,
    });
    let path = state_paths::neural_architecture_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "neural_architecture_written",
        Err(_) => "neural_architecture_write_failed",
    };
    format!(
        "[NEURAL-ARCHITECTURE] passed={}/{} weights_present=false training_executed=false claim_allowed=false write_status={} path={}\n",
        passed, total, write_status, path
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neural_architecture_writes_no_claim_result() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_neural_architecture_test_{}",
            std::process::id()
        )));
        let out = run(NeuralArchitectureInput {
            capability_status:
                "Transformer | BigTransformer | MoE | BPTT | DNC | Semantics: vocab=128".to_string(),
            hrm_text_report: "[HRM-TEXT] checkpoints=1".to_string(),
            checkpoint_manifest:
                "schema=hrm-text-checkpoint-v1\nweights_present=false\ntraining_executed=false\n"
                    .to_string(),
            retrieval_report: "[HRM-TEXT-SEARCH] hits=1".to_string(),
        });
        assert!(out.contains("[NEURAL-ARCHITECTURE]"));
        assert!(out.contains("passed=6/6"));
        assert!(out.contains("weights_present=false"));
        assert!(std::fs::metadata(state_paths::neural_architecture_path()).is_ok());
    }
}
