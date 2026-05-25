use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct ArchitectureAdvantageInput {
    pub gewc_report: String,
    pub capability_reality_report: String,
    pub memory_report: String,
    pub world_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
    pub external_validation_report: String,
}

struct Movement {
    tag: &'static str,
    artifact: &'static str,
    path: String,
    objective: &'static str,
    cases: Vec<(&'static str, bool)>,
    payload: serde_json::Value,
}

pub fn run(input: ArchitectureAdvantageInput) -> String {
    let movements = movements(&input);
    let mut passed_movements = 0usize;
    let mut out = String::new();
    for movement in &movements {
        let passed = movement.cases.iter().filter(|(_, passed)| *passed).count();
        if passed == movement.cases.len() {
            passed_movements += 1;
        }
        write_movement_artifact(movement, passed);
        out.push_str(&format!(
            "[{}] artifact={} passed={}/{} claim_allowed=false path={}\n",
            movement.tag,
            movement.artifact,
            passed,
            movement.cases.len(),
            movement.path
        ));
    }

    let summary_cases: Vec<_> = movements
        .iter()
        .map(|movement| {
            let passed = movement.cases.iter().filter(|(_, passed)| *passed).count();
            serde_json::json!({
                "id": movement.artifact,
                "tag": movement.tag,
                "objective": movement.objective,
                "passed": passed == movement.cases.len(),
                "passed_cases": passed,
                "total_cases": movement.cases.len(),
            })
        })
        .collect();
    let record = serde_json::json!({
        "schema": "garm-architecture-advantage-eval-v1",
        "artifact": "architecture_advantage_eval",
        "claim_allowed": false,
        "agi_claim": false,
        "validation_scope": "architecture_competitive_position_not_external_capability_certification",
        "comparison_targets": [
            "LangGraph",
            "AutoGen/Magentic-One",
            "OpenHands",
            "OpenCog Hyperon"
        ],
        "passed": passed_movements,
        "total": movements.len(),
        "verdict": if passed_movements == movements.len() {
            "architecture_advantage_ready_local"
        } else {
            "needs_architecture_advantage_evidence"
        },
        "movements": summary_cases,
    });
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        state_paths::architecture_advantage_eval_path(),
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );

    out.push_str(&format!(
        "[ARCHITECTURE-ADVANTAGE-EVAL] movements={}/{} claim_allowed=false path={}\n",
        passed_movements,
        movements.len(),
        state_paths::architecture_advantage_eval_path(),
    ));
    out
}

fn movements(input: &ArchitectureAdvantageInput) -> Vec<Movement> {
    vec![
        Movement {
            tag: "GEWC-TRACE-SPEC",
            artifact: "gewc_trace_spec",
            path: state_paths::gewc_trace_spec_path(),
            objective: "Make every GEWC decision auditable by objective, context, module, risk, evidence, action and result.",
            cases: vec![
                (
                    "runtime_trace_present",
                    input.gewc_report.contains("[GEWC-RUNTIME]")
                        && input.gewc_report.contains("handler_metrics="),
                ),
                (
                    "executive_authority_present",
                    input
                        .gewc_report
                        .contains("core_authority=global_executive_workspace_core"),
                ),
                (
                    "safety_context_present",
                    input.policy_report.contains("[POLICY]")
                        && input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
                (
                    "evidence_context_present",
                    input.provenance_report.contains("[PROVENANCE]")
                        && input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
            ],
            payload: serde_json::json!({
                "required_fields": [
                    "trace_id",
                    "timestamp",
                    "objective_id",
                    "situation_model",
                    "workspace_context",
                    "selected_module",
                    "candidate_modules",
                    "risk_level",
                    "policy_decision",
                    "evidence_refs",
                    "uncertainty",
                    "planned_action",
                    "execution_result",
                    "learning_update",
                    "claim_allowed"
                ],
                "runtime_contract": "Every non-shutdown command emits a GEWC decision trace before action and a completion trace after action.",
            }),
        },
        Movement {
            tag: "CAPABILITY-REALITY-MATRIX-V2",
            artifact: "capability_reality_matrix_v2",
            path: state_paths::capability_reality_matrix_v2_path(),
            objective: "Upgrade capability reality from static labels into measurable operational fields.",
            cases: vec![
                (
                    "v1_reality_present",
                    input
                        .capability_reality_report
                        .contains("[CAPABILITY-REALITY-EVAL]"),
                ),
                (
                    "lmm_dependency_present",
                    input
                        .capability_reality_report
                        .contains("[LMM-TRAINING-DEPENDENCY]"),
                ),
                (
                    "external_validation_bounded",
                    input
                        .external_validation_report
                        .contains("claim_allowed=false"),
                ),
                (
                    "safety_block_available",
                    input.policy_report.contains("blocked="),
                ),
            ],
            payload: serde_json::json!({
                "metric_fields": [
                    "success_rate",
                    "latency_ms_p50",
                    "latency_ms_p95",
                    "failure_count",
                    "regression_count",
                    "confidence",
                    "coverage",
                    "evidence_count",
                    "last_verified_at",
                    "lmm_training_required",
                    "external_capability_validation_required",
                    "safety_blocked"
                ],
                "status_model": [
                    "implemented_runtime",
                    "implemented_local_heuristic",
                    "implemented_architecture_artifact",
                    "simulated_or_stub",
                    "requires_lmm_training",
                    "requires_external_capability_validation",
                    "blocked_by_safety_policy"
                ],
            }),
        },
        Movement {
            tag: "COGNITIVE-TASK-SUITE",
            artifact: "cognitive_task_suite",
            path: state_paths::cognitive_task_suite_path(),
            objective: "Define local task families that measure cognition across memory, planning, world model, tool use, safety and transfer.",
            cases: vec![
                (
                    "memory_task_evidence",
                    input.memory_report.contains("[MEMORY-EVAL]"),
                ),
                (
                    "world_task_evidence",
                    input.world_report.contains("[WORLD")
                        && input.world_report.contains("[WORLD-EVAL]"),
                ),
                (
                    "policy_task_evidence",
                    input.policy_report.contains("[POLICY]"),
                ),
                (
                    "trace_task_evidence",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
            ],
            payload: serde_json::json!({
                "task_families": [
                    {"id": "memory_retrieval", "measures": ["recall", "abstention", "citation"]},
                    {"id": "hierarchical_planning", "measures": ["goal_decomposition", "rollback", "completion"]},
                    {"id": "world_model_prediction", "measures": ["observe", "predict", "verify"]},
                    {"id": "tool_use_governance", "measures": ["policy", "sandbox", "action_trace"]},
                    {"id": "uncertainty_metacognition", "measures": ["risk", "confidence", "review"]},
                    {"id": "transfer", "measures": ["novel_task", "module_selection", "evidence_reuse"]}
                ],
            }),
        },
        Movement {
            tag: "EDEN-AGENT-SDK",
            artifact: "eden_agent_sdk_contract",
            path: state_paths::eden_agent_sdk_contract_path(),
            objective: "Make extension modules subordinate to GEWC safety, memory, audit and objective contracts.",
            cases: vec![
                (
                    "domain_handlers_present",
                    input
                        .gewc_report
                        .contains("handler_topology=domain_owned_body_implementations"),
                ),
                (
                    "single_authority_present",
                    input.gewc_report.contains("external_cores_remaining=false"),
                ),
                (
                    "policy_required",
                    input.policy_report.contains("[POLICY]"),
                ),
                (
                    "provenance_required",
                    input.provenance_report.contains("[PROVENANCE]"),
                ),
            ],
            payload: serde_json::json!({
                "module_contract": {
                    "required_interfaces": [
                        "describe_capability",
                        "declare_permissions",
                        "estimate_risk",
                        "execute_bounded",
                        "emit_evidence",
                        "rollback_or_compensate",
                        "report_metrics"
                    ],
                    "forbidden": [
                        "bypass_gewc_router",
                        "mutate_objectives_without_policy",
                        "execute_external_action_without_permission",
                        "claim_agi_capability"
                    ]
                },
            }),
        },
        Movement {
            tag: "MODEL-ADAPTER-LAYER",
            artifact: "model_adapter_layer",
            path: state_paths::model_adapter_layer_path(),
            objective: "Connect LMM, local models and external frontier models as governed modules rather than a central brain.",
            cases: vec![
                (
                    "lmm_gap_explicit",
                    input
                        .capability_reality_report
                        .contains("requires_lmm_training=")
                        || input
                            .capability_reality_report
                            .contains("[LMM-TRAINING-DEPENDENCY]"),
                ),
                (
                    "gewc_not_model_centric",
                    input.gewc_report.contains("body_executor=gewc_body_executor")
                        && input.gewc_report.contains("shared_body_engine=false"),
                ),
                (
                    "claim_boundary_present",
                    input
                        .external_validation_report
                        .contains("claim_allowed=false"),
                ),
                (
                    "tool_action_boundary_present",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
            ],
            payload: serde_json::json!({
                "adapter_types": [
                    "eden_lmm_local",
                    "external_foundation_model",
                    "symbolic_reasoner",
                    "embedding_retriever",
                    "vision_language_action_model",
                    "tool_executor"
                ],
                "governance": [
                    "permission_scope",
                    "prompt_or_input_hash",
                    "output_risk_review",
                    "provenance_record",
                    "uncertainty_record",
                    "no_weight_update_without_training_gate"
                ],
            }),
        },
        Movement {
            tag: "REPRODUCIBLE-DEMOS",
            artifact: "reproducible_demos",
            path: state_paths::reproducible_demos_path(),
            objective: "Define end-to-end demos that prove EDEN is more than a static architecture map.",
            cases: vec![
                (
                    "memory_world_demo_possible",
                    input.memory_report.contains("[MEMORY-EVAL]")
                        && input.world_report.contains("[WORLD-EVAL]"),
                ),
                (
                    "coding_demo_governable",
                    input.policy_report.contains("[POLICY]")
                        && input.provenance_report.contains("[PROVENANCE]"),
                ),
                (
                    "research_demo_traceable",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]")
                        && input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
                (
                    "demo_claims_bounded",
                    input
                        .external_validation_report
                        .contains("claim_allowed=false"),
                ),
            ],
            payload: serde_json::json!({
                "demos": [
                    {
                        "id": "coding_task_demo",
                        "goal": "plan, edit, test, rollback and audit a bounded code change",
                        "required_evidence": ["GEWC trace", "policy decision", "test result", "provenance record"]
                    },
                    {
                        "id": "research_task_demo",
                        "goal": "retrieve, cite, abstain when unsupported and produce an uncertainty ledger",
                        "required_evidence": ["memory eval", "context pack", "uncertainty record", "claim boundary"]
                    },
                    {
                        "id": "embodied_simulated_task_demo",
                        "goal": "observe, predict, choose safe action and verify consequence in a simulated world",
                        "required_evidence": ["world observation", "prediction", "verification", "safety gate"]
                    }
                ],
            }),
        },
    ]
}

fn write_movement_artifact(movement: &Movement, passed: usize) {
    let cases: Vec<_> = movement
        .cases
        .iter()
        .map(|(id, passed)| serde_json::json!({ "id": id, "passed": passed }))
        .collect();
    let record = serde_json::json!({
        "schema": "garm-architecture-advantage-movement-v1",
        "artifact": movement.artifact,
        "tag": movement.tag,
        "objective": movement.objective,
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": movement.cases.len(),
        "verdict": if passed == movement.cases.len() {
            "movement_ready_local"
        } else {
            "needs_movement_evidence"
        },
        "cases": cases,
        "payload": movement.payload,
    });
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        &movement.path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_six_architecture_advantage_movements() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_architecture_advantage_test_{}",
            std::process::id()
        )));
        let out = run(ArchitectureAdvantageInput {
            gewc_report: "[GEWC-RUNTIME] core_authority=global_executive_workspace_core body_executor=gewc_body_executor handler_topology=domain_owned_body_implementations shared_body_engine=false external_cores_remaining=false handler_metrics=x".to_string(),
            capability_reality_report:
                "[CAPABILITY-REALITY-EVAL] requires_lmm_training=8\n[LMM-TRAINING-DEPENDENCY]"
                    .to_string(),
            memory_report: "[MEMORY-EVAL] passed=5/5".to_string(),
            world_report: "[WORLD]\n[WORLD-EVAL] passed=5/5".to_string(),
            policy_report: "[POLICY] allowed=1 blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
            external_validation_report: "[EXTERNAL-VALIDATION] claim_allowed=false agi_claim=false"
                .to_string(),
        });
        assert!(out.contains("[ARCHITECTURE-ADVANTAGE-EVAL] movements=6/6"));
        assert!(std::fs::metadata(state_paths::gewc_trace_spec_path()).is_ok());
        assert!(std::fs::metadata(state_paths::capability_reality_matrix_v2_path()).is_ok());
        assert!(std::fs::metadata(state_paths::cognitive_task_suite_path()).is_ok());
        assert!(std::fs::metadata(state_paths::eden_agent_sdk_contract_path()).is_ok());
        assert!(std::fs::metadata(state_paths::model_adapter_layer_path()).is_ok());
        assert!(std::fs::metadata(state_paths::reproducible_demos_path()).is_ok());
        assert!(std::fs::metadata(state_paths::architecture_advantage_eval_path()).is_ok());
        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
