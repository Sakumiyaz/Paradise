use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct PraxisNexusInput {
    pub gewc_report: String,
    pub architecture_advantage_report: String,
    pub capability_reality_report: String,
    pub memory_report: String,
    pub world_report: String,
    pub cognitive_report: String,
    pub symbolic_report: String,
    pub goals_report: String,
    pub plan_executor_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
    pub external_validation_report: String,
}

struct Primitive {
    id: &'static str,
    purpose: &'static str,
    native_records: Vec<&'static str>,
    evidence_present: bool,
}

struct Block {
    tag: &'static str,
    artifact: &'static str,
    path: String,
    purpose: &'static str,
    cases: Vec<(&'static str, bool)>,
    payload: serde_json::Value,
}

pub fn run(input: PraxisNexusInput) -> String {
    let primitives = primitives(&input);
    let blocks = blocks(&input, &primitives);
    let passed_primitives = primitives
        .iter()
        .filter(|primitive| primitive.evidence_present)
        .count();
    let passed_blocks = blocks
        .iter()
        .filter(|block| block.cases.iter().all(|(_, passed)| *passed))
        .count();

    write_primitives(&primitives);
    write_blocks(&blocks);
    for block in &blocks {
        write_block_artifact(block);
    }
    write_nexus_summary(&primitives, &blocks);

    let mut out = format!(
        "[EDEN-PRAXIS-NEXUS] goal=governed_cognitive_operational_substrate primitives={}/{} blocks={}/{} claim_allowed=false path={}\n[PRAXIS-PRIMITIVES] passed={}/{} path={}\n[PRAXIS-BLOCKS] passed={}/{} path={}\n",
        passed_primitives,
        primitives.len(),
        passed_blocks,
        blocks.len(),
        state_paths::eden_praxis_nexus_path(),
        passed_primitives,
        primitives.len(),
        state_paths::praxis_primitives_path(),
        passed_blocks,
        blocks.len(),
        state_paths::praxis_blocks_path(),
    );
    for block in blocks {
        let passed = block.cases.iter().filter(|(_, passed)| *passed).count();
        out.push_str(&format!(
            "[{}] artifact={} passed={}/{} claim_allowed=false path={}\n",
            block.tag,
            block.artifact,
            passed,
            block.cases.len(),
            block.path
        ));
    }
    out
}

fn primitives(input: &PraxisNexusInput) -> Vec<Primitive> {
    vec![
        Primitive {
            id: "intent",
            purpose: "Goals, values, priorities and conflicts represented as governable cognition.",
            native_records: vec!["goal", "priority", "conflict", "correction"],
            evidence_present: input.goals_report.contains("[GOALS]")
                && input.gewc_report.contains("last_route="),
        },
        Primitive {
            id: "state",
            purpose: "World, system, memory and active context state.",
            native_records: vec![
                "world_state",
                "memory_state",
                "workspace_state",
                "runtime_state",
            ],
            evidence_present: input.world_report.contains("[WORLD")
                && input.memory_report.contains("[MEMORY-EVAL]"),
        },
        Primitive {
            id: "evidence",
            purpose: "Sources, citations, provenance, confidence and support boundaries.",
            native_records: vec!["source", "citation", "provenance", "confidence"],
            evidence_present: input.provenance_report.contains("[PROVENANCE]")
                && input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
        },
        Primitive {
            id: "constraint",
            purpose: "Policies, permissions, safety limits and corrigibility constraints.",
            native_records: vec!["policy", "permission", "sandbox", "claim_boundary"],
            evidence_present: input.policy_report.contains("[POLICY]")
                && input
                    .external_validation_report
                    .contains("claim_allowed=false"),
        },
        Primitive {
            id: "affordance",
            purpose: "Available actions, tools, agents and capability routes under GEWC authority.",
            native_records: vec!["tool", "agent", "module_route", "capability_state"],
            evidence_present: input
                .gewc_report
                .contains("handler_topology=domain_owned_body_implementations")
                && input
                    .capability_reality_report
                    .contains("[CAPABILITY-REALITY-EVAL]"),
        },
        Primitive {
            id: "projection",
            purpose: "Predictions, simulations, causal expectations and expected consequences.",
            native_records: vec![
                "prediction",
                "simulation",
                "causal_link",
                "expected_outcome",
            ],
            evidence_present: input.world_report.contains("[WORLD-EVAL]")
                && input.uncertainty_report.contains("[UNCERTAINTY]"),
        },
        Primitive {
            id: "trace",
            purpose: "GEWC decisions, selected route, result, correction and learning audit trail.",
            native_records: vec!["decision", "route", "result", "learning_update"],
            evidence_present: input.gewc_report.contains("[GEWC-RUNTIME]")
                && input
                    .architecture_advantage_report
                    .contains("[GEWC-TRACE-SPEC]"),
        },
    ]
}

fn blocks(input: &PraxisNexusInput, primitives: &[Primitive]) -> Vec<Block> {
    let primitives_ready = primitives
        .iter()
        .all(|primitive| primitive.evidence_present);
    vec![
        Block {
            tag: "PRAXIS-SPACE",
            artifact: "praxis_space",
            path: state_paths::praxis_space_path(),
            purpose: "Unified living space for intents, states, evidence, constraints, affordances, projections and traces.",
            cases: vec![
                ("seven_primitives_present", primitives_ready),
                (
                    "gewc_authority_present",
                    input
                        .gewc_report
                        .contains("core_authority=global_executive_workspace_core"),
                ),
                (
                    "memory_world_join_present",
                    input.memory_report.contains("[MEMORY-EVAL]")
                        && input.world_report.contains("[WORLD"),
                ),
                (
                    "safety_evidence_join_present",
                    input.policy_report.contains("[POLICY]")
                        && input.provenance_report.contains("[PROVENANCE]"),
                ),
            ],
            payload: serde_json::json!({
                "primitive_types": ["intent", "state", "evidence", "constraint", "affordance", "projection", "trace"],
                "storage_model": "typed_cognitive_records_with_policy_and_evidence_edges",
                "not_atomspace_clone": true,
            }),
        },
        Block {
            tag: "PRAXIS-RULES",
            artifact: "praxis_rules",
            path: state_paths::praxis_rules_path(),
            purpose: "Executable cognitive contracts for permissions, plans, evidence, risk and learning.",
            cases: vec![
                (
                    "policy_gate_present",
                    input.policy_report.contains("[POLICY]")
                        && input.policy_report.contains("blocked="),
                ),
                (
                    "action_trace_present",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
                (
                    "plan_context_present",
                    input.plan_executor_report.contains("[EXEC]")
                        || input.goals_report.contains("[GOALS]"),
                ),
                (
                    "claim_boundary_present",
                    input.external_validation_report.contains("claim_allowed=false"),
                ),
            ],
            payload: serde_json::json!({
                "rule_shape": "when intent require evidence require constraint route affordance simulate projection after action record trace",
                "forbidden": [
                    "bypass_gewc",
                    "external_action_without_permission",
                    "objective_mutation_without_policy",
                    "agi_claim"
                ],
            }),
        },
        Block {
            tag: "PRAXIS-TRACE-SEMANTICS",
            artifact: "praxis_trace_semantics",
            path: state_paths::praxis_trace_semantics_path(),
            purpose: "Formalize GEWC decision cycles as situation, objective, candidates, risk, action, outcome and learning.",
            cases: vec![
                (
                    "gewc_trace_spec_present",
                    input
                        .architecture_advantage_report
                        .contains("[GEWC-TRACE-SPEC]"),
                ),
                (
                    "runtime_metrics_present",
                    input.gewc_report.contains("handler_metrics="),
                ),
                (
                    "risk_trace_present",
                    input.uncertainty_report.contains("[UNCERTAINTY]")
                        && input.policy_report.contains("[POLICY]"),
                ),
                (
                    "outcome_trace_present",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]")
                        && input.provenance_report.contains("[PROVENANCE]"),
                ),
            ],
            payload: serde_json::json!({
                "cycle": ["situation", "intent", "candidate_affordances", "constraint_check", "projection", "selected_action", "outcome", "trace", "learning_update"],
                "semantics": "auditable_decision_transition_system",
            }),
        },
        Block {
            tag: "PRAXIS-REASONER",
            artifact: "praxis_reasoner",
            path: state_paths::praxis_reasoner_path(),
            purpose: "Hybrid operational reasoner for symbolic, causal, probabilistic, normative and evidence reasoning.",
            cases: vec![
                (
                    "symbolic_reasoning_present",
                    input.symbolic_report.contains("[SYMBOLIC-ARCHITECTURE]"),
                ),
                (
                    "causal_projection_present",
                    input.world_report.contains("[WORLD-EVAL]"),
                ),
                (
                    "normative_reasoning_present",
                    input.policy_report.contains("[POLICY]"),
                ),
                (
                    "metacognitive_uncertainty_present",
                    input.cognitive_report.contains("[COGNITIVE-ARCHITECTURE]")
                        && input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
            ],
            payload: serde_json::json!({
                "reasoning_modes": ["symbolic", "causal", "probabilistic", "normative", "evidence", "contradiction"],
                "model_role": "coordinate_formal_reasoning_with_LMMs_without_making_a_model_the_central_brain",
            }),
        },
        Block {
            tag: "PRAXIS-BENCH",
            artifact: "praxis_bench",
            path: state_paths::praxis_bench_path(),
            purpose: "Evaluate the formal substrate with memory consistency, causal reasoning, policy compliance, transfer and correction tasks.",
            cases: vec![
                (
                    "memory_bench_present",
                    input.memory_report.contains("[MEMORY-EVAL]"),
                ),
                (
                    "world_bench_present",
                    input.world_report.contains("[WORLD-EVAL]"),
                ),
                (
                    "capability_reality_present",
                    input
                        .capability_reality_report
                        .contains("[CAPABILITY-REALITY-EVAL]"),
                ),
                (
                    "external_claim_boundary_present",
                    input.external_validation_report.contains("claim_allowed=false"),
                ),
            ],
            payload: serde_json::json!({
                "task_families": [
                    "memory_consistency",
                    "causal_projection",
                    "contradiction_detection",
                    "constraint_planning",
                    "policy_compliance",
                    "evidence_abstention",
                    "domain_transfer",
                    "feedback_correction"
                ],
            }),
        },
    ]
}

fn write_primitives(primitives: &[Primitive]) {
    let records: Vec<_> = primitives
        .iter()
        .map(|primitive| {
            serde_json::json!({
                "id": primitive.id,
                "purpose": primitive.purpose,
                "native_records": primitive.native_records,
                "evidence_present": primitive.evidence_present,
            })
        })
        .collect();
    let passed = primitives
        .iter()
        .filter(|primitive| primitive.evidence_present)
        .count();
    let record = serde_json::json!({
        "schema": "eden-praxis-primitives-v1",
        "artifact": "praxis_primitives",
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": primitives.len(),
        "primitives": records,
    });
    write_json(state_paths::praxis_primitives_path(), record);
}

fn write_blocks(blocks: &[Block]) {
    let records: Vec<_> = blocks
        .iter()
        .map(|block| {
            let passed = block.cases.iter().filter(|(_, passed)| *passed).count();
            serde_json::json!({
                "tag": block.tag,
                "artifact": block.artifact,
                "purpose": block.purpose,
                "passed": passed,
                "total": block.cases.len(),
                "path": block.path,
            })
        })
        .collect();
    let passed = blocks
        .iter()
        .filter(|block| block.cases.iter().all(|(_, passed)| *passed))
        .count();
    let record = serde_json::json!({
        "schema": "eden-praxis-blocks-v1",
        "artifact": "praxis_blocks",
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": blocks.len(),
        "blocks": records,
    });
    write_json(state_paths::praxis_blocks_path(), record);
}

fn write_block_artifact(block: &Block) {
    let cases: Vec<_> = block
        .cases
        .iter()
        .map(|(id, passed)| serde_json::json!({ "id": id, "passed": passed }))
        .collect();
    let passed = block.cases.iter().filter(|(_, passed)| *passed).count();
    let record = serde_json::json!({
        "schema": "eden-praxis-block-v1",
        "artifact": block.artifact,
        "tag": block.tag,
        "purpose": block.purpose,
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": block.cases.len(),
        "cases": cases,
        "payload": block.payload,
    });
    write_json(block.path.clone(), record);
}

fn write_nexus_summary(primitives: &[Primitive], blocks: &[Block]) {
    let passed_primitives = primitives
        .iter()
        .filter(|primitive| primitive.evidence_present)
        .count();
    let passed_blocks = blocks
        .iter()
        .filter(|block| block.cases.iter().all(|(_, passed)| *passed))
        .count();
    let record = serde_json::json!({
        "schema": "eden-praxis-nexus-v1",
        "artifact": "eden_praxis_nexus",
        "name": "Eden Praxis Nexus",
        "subtitle": "Governed Cognitive-Operational Substrate",
        "goal": "make GEWC operate over a native formal substrate for knowledge, action, evidence, safety, causality and learning",
        "claim_allowed": false,
        "agi_claim": false,
        "originality_boundary": {
            "not_atomspace": true,
            "not_metta_clone": true,
            "focus": "governed cognition in action rather than standalone symbolic knowledge representation"
        },
        "primitives_passed": passed_primitives,
        "primitives_total": primitives.len(),
        "blocks_passed": passed_blocks,
        "blocks_total": blocks.len(),
        "verdict": if passed_primitives == primitives.len() && passed_blocks == blocks.len() {
            "praxis_nexus_ready_local"
        } else {
            "needs_praxis_nexus_evidence"
        },
    });
    write_json(state_paths::eden_praxis_nexus_path(), record);
}

fn write_json(path: String, record: serde_json::Value) {
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_praxis_nexus_primitives_blocks_and_meta() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_praxis_nexus_test_{}",
            std::process::id()
        )));
        let out = run(PraxisNexusInput {
            gewc_report: "[GEWC-RUNTIME] core_authority=global_executive_workspace_core handler_topology=domain_owned_body_implementations handler_metrics=x last_route=evaluation_validation".to_string(),
            architecture_advantage_report: "[GEWC-TRACE-SPEC] passed=4/4".to_string(),
            capability_reality_report: "[CAPABILITY-REALITY-EVAL] requires_lmm_training=8".to_string(),
            memory_report: "[MEMORY-EVAL] passed=5/5".to_string(),
            world_report: "[WORLD]\n[WORLD-EVAL] passed=5/5".to_string(),
            cognitive_report: "[COGNITIVE-ARCHITECTURE] passed=5/5".to_string(),
            symbolic_report: "[SYMBOLIC-ARCHITECTURE] passed=6/6".to_string(),
            goals_report: "[GOALS] goals=1".to_string(),
            plan_executor_report: "[EXEC] plans=1".to_string(),
            policy_report: "[POLICY] allowed=1 blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
            external_validation_report: "[EXTERNAL-VALIDATION] claim_allowed=false agi_claim=false"
                .to_string(),
        });
        assert!(out.contains("[EDEN-PRAXIS-NEXUS]"));
        assert!(out.contains("primitives=7/7"));
        assert!(out.contains("blocks=5/5"));
        assert!(std::fs::metadata(state_paths::eden_praxis_nexus_path()).is_ok());
        assert!(std::fs::metadata(state_paths::praxis_primitives_path()).is_ok());
        assert!(std::fs::metadata(state_paths::praxis_blocks_path()).is_ok());
        assert!(std::fs::metadata(state_paths::praxis_space_path()).is_ok());
        assert!(std::fs::metadata(state_paths::praxis_rules_path()).is_ok());
        assert!(std::fs::metadata(state_paths::praxis_trace_semantics_path()).is_ok());
        assert!(std::fs::metadata(state_paths::praxis_reasoner_path()).is_ok());
        assert!(std::fs::metadata(state_paths::praxis_bench_path()).is_ok());
        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
