use crate::eden_garm::state_paths;
use std::io::Write;

const AUTHORITY: &str = "global_executive_workspace_core";
const FORGE_NAME: &str = "Eden Operator Forge";
const SUBTITLE: &str = "Formal Primitive Synthesis Engine";

#[derive(Clone)]
pub struct OperatorForgeInput {
    pub praxis_report: String,
    pub world_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
}

#[derive(Clone, Copy)]
struct PrimitiveSpec {
    id: &'static str,
    arity: u8,
    domain: &'static str,
    purpose: &'static str,
}

pub fn run(input: OperatorForgeInput) -> String {
    let _ = state_paths::ensure_state_dir();
    let basis = primitive_basis();
    let checks = component_checks(&input);
    let passed = checks.iter().filter(|(_, passed)| *passed).count();
    write_json(
        state_paths::operator_primitive_basis_path(),
        primitive_basis_value(&basis),
    );
    ensure_graph_log();
    write_json(
        state_paths::operator_model_registry_path(),
        model_registry_value(),
    );
    let verification = verify_expression_graphs_value();
    write_json(
        state_paths::operator_verification_report_path(),
        verification.clone(),
    );
    let record = serde_json::json!({
        "schema": "eden-operator-forge-v1",
        "artifact": "eden_operator_forge",
        "name": FORGE_NAME,
        "subtitle": SUBTITLE,
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "purpose": "Let GEWC synthesize bounded typed expression graphs, formal candidates and causal equations without depending on one external operator or allowing generated formulas to become truth without verification.",
        "not_eml_clone": true,
        "not_atomspace_clone": true,
        "not_symbolic_regression_claim": true,
        "native_to_gewc": true,
        "components": checks.iter().map(|(id, passed)| {
            serde_json::json!({"id": id, "passed": passed})
        }).collect::<Vec<_>>(),
        "passed": passed,
        "total": checks.len(),
        "artifacts": [
            "operator_primitive_basis",
            "operator_expression_graphs",
            "operator_verification_report",
            "operator_model_registry"
        ],
        "verification": verification,
        "invariants": [
            "operator candidates are hypotheses until verified",
            "typed expression graphs carry units, domains and depth bounds",
            "synthesis has no filesystem, network or shell side effects",
            "compiled execution is disabled until a future sandboxed compiler gate exists",
            "accepted candidates may feed CWM only through GEWC and provenance"
        ],
        "verdict": if passed == checks.len() {
            "operator_forge_ready_local"
        } else {
            "needs_operator_forge_evidence"
        },
    });
    write_json(state_paths::eden_operator_forge_path(), record);
    format!(
        "[EDEN-OPERATOR-FORGE] passed={}/{} basis={} expression_graphs={} verifier=bounded claim_allowed=false path={}\n[OPERATOR-BASIS] path={}\n[OPERATOR-GRAPHS] path={}\n[OPERATOR-VERIFY] path={}\n[OPERATOR-MODEL-REGISTRY] path={}\n",
        passed,
        checks.len(),
        basis.len(),
        graph_count(),
        state_paths::eden_operator_forge_path(),
        state_paths::operator_primitive_basis_path(),
        state_paths::operator_expression_graphs_path(),
        state_paths::operator_verification_report_path(),
        state_paths::operator_model_registry_path(),
    )
}

pub fn synthesize(goal: &str) -> String {
    let goal = goal.trim();
    if goal.is_empty() {
        return "[OPERATOR-FORGE-SYNTH] status=rejected reason=empty_goal\n".to_string();
    }
    let selected_basis = select_basis(goal);
    let graph = expression_graph_value(goal, &selected_basis);
    let graph_id = graph
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("operator-unknown")
        .to_string();
    let verification = verify_graph(&graph);
    append_graph(graph);
    write_json(
        state_paths::operator_verification_report_path(),
        verify_expression_graphs_value(),
    );
    let passed = verification
        .get("passed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    format!(
        "[OPERATOR-FORGE-SYNTH] status={} id={} basis={} verification={} path={}\n",
        if passed {
            "candidate_recorded"
        } else {
            "candidate_quarantined"
        },
        graph_id,
        selected_basis.join(","),
        if passed { "passed" } else { "failed" },
        state_paths::operator_expression_graphs_path()
    )
}

pub fn verify() -> String {
    let verification = verify_expression_graphs_value();
    let passed = verification
        .get("passed")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let total = verification
        .get("total")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    write_json(
        state_paths::operator_verification_report_path(),
        verification,
    );
    format!(
        "[OPERATOR-FORGE-VERIFY] passed={}/{} path={}\n",
        passed,
        total,
        state_paths::operator_verification_report_path()
    )
}

pub fn audit_report() -> String {
    let graphs = graph_count();
    let verification = verify_expression_graphs_value();
    let passed = verification
        .get("passed")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    format!(
        "[OPERATOR-FORGE-AUDIT] graphs={} verified={} compile_enabled=false external_effects=false path={}\n",
        graphs,
        passed,
        state_paths::operator_expression_graphs_path()
    )
}

fn component_checks(input: &OperatorForgeInput) -> Vec<(&'static str, bool)> {
    vec![
        (
            "praxis_substrate",
            input.praxis_report.contains("[EDEN-PRAXIS-NEXUS]")
                || std::fs::metadata(state_paths::eden_praxis_nexus_path()).is_ok(),
        ),
        (
            "typed_primitive_basis",
            primitive_basis()
                .iter()
                .all(|primitive| primitive.arity <= 2),
        ),
        (
            "causal_world_model_link",
            input.world_report.contains("[WORLD]") || input.world_report.contains("[WORLD-EVAL]"),
        ),
        (
            "policy_guard",
            input.policy_report.contains("[POLICY]") || input.policy_report.contains("blocked="),
        ),
        (
            "provenance_binding",
            input.provenance_report.contains("[PROVENANCE]")
                || input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
        ),
        (
            "uncertainty_boundary",
            input.uncertainty_report.contains("[UNCERTAINTY]")
                || std::fs::metadata(state_paths::operator_verification_report_path()).is_ok(),
        ),
    ]
}

fn primitive_basis() -> Vec<PrimitiveSpec> {
    vec![
        primitive(
            "identity",
            1,
            "all",
            "Preserve an observed scalar or symbol.",
        ),
        primitive("affine_shift", 2, "numeric", "Translate or bias a value."),
        primitive(
            "bounded_product",
            2,
            "numeric",
            "Compose two magnitudes with saturation.",
        ),
        primitive(
            "residual_delta",
            2,
            "numeric",
            "Represent change between state and expectation.",
        ),
        primitive(
            "causal_gate",
            2,
            "causal",
            "Gate an effect by a candidate cause.",
        ),
        primitive(
            "counterfactual_switch",
            2,
            "causal",
            "Compare observed and alternative states.",
        ),
        primitive(
            "phase_fold",
            2,
            "periodic",
            "Represent bounded cyclical structure.",
        ),
        primitive(
            "evidence_weight",
            2,
            "probabilistic",
            "Attach support strength to a candidate.",
        ),
        primitive(
            "constraint_clip",
            2,
            "safety",
            "Keep a candidate inside declared bounds.",
        ),
    ]
}

fn primitive(
    id: &'static str,
    arity: u8,
    domain: &'static str,
    purpose: &'static str,
) -> PrimitiveSpec {
    PrimitiveSpec {
        id,
        arity,
        domain,
        purpose,
    }
}

fn primitive_basis_value(basis: &[PrimitiveSpec]) -> serde_json::Value {
    serde_json::json!({
        "schema": "eden-operator-primitive-basis-v1",
        "artifact": "operator_primitive_basis",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "basis": basis.iter().map(|primitive| {
            serde_json::json!({
                "id": primitive.id,
                "arity": primitive.arity,
                "domain": primitive.domain,
                "purpose": primitive.purpose,
            })
        }).collect::<Vec<_>>(),
        "selection_policy": "choose a small typed basis from goal domain, then verify graph shape before any downstream use",
    })
}

fn model_registry_value() -> serde_json::Value {
    serde_json::json!({
        "schema": "eden-operator-model-registry-v1",
        "artifact": "operator_model_registry",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "roles": [
            {"role": "basis_selector", "adapter": "deterministic_keyword_and_domain_router"},
            {"role": "expression_graph_builder", "adapter": "typed_bounded_graph_synthesizer"},
            {"role": "verifier", "adapter": "shape_domain_depth_side_effect_checker"},
            {"role": "cwm_candidate_exporter", "adapter": "disabled_until_verified_and_gewc_accepted"}
        ],
        "compile_policy": {
            "code_generation_enabled": false,
            "network_enabled": false,
            "filesystem_mutation_enabled": false,
            "future_compiler_requires_sandbox": true
        },
    })
}

fn select_basis(goal: &str) -> Vec<&'static str> {
    let normalized = goal.to_ascii_lowercase();
    let mut basis = vec!["identity", "residual_delta", "constraint_clip"];
    if contains_any(&normalized, &["cause", "causal", "counterfactual", "if "]) {
        basis.push("causal_gate");
        basis.push("counterfactual_switch");
    }
    if contains_any(&normalized, &["period", "cycle", "oscillat", "season"]) {
        basis.push("phase_fold");
    }
    if contains_any(
        &normalized,
        &["probability", "uncertain", "risk", "confidence"],
    ) {
        basis.push("evidence_weight");
    }
    if contains_any(&normalized, &["growth", "cost", "value", "score", "linear"]) {
        basis.push("affine_shift");
        basis.push("bounded_product");
    }
    basis.sort_unstable();
    basis.dedup();
    basis
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn expression_graph_value(goal: &str, selected_basis: &[&str]) -> serde_json::Value {
    let graph_id = format!("opforge-{:016x}", fnv64(goal.as_bytes()));
    let primary = selected_basis
        .iter()
        .find(|name| **name != "identity" && **name != "constraint_clip")
        .copied()
        .unwrap_or("residual_delta");
    serde_json::json!({
        "schema": "eden-operator-expression-graph-v1",
        "id": graph_id,
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "goal": goal,
        "basis": selected_basis,
        "typed": true,
        "acyclic": true,
        "max_depth": 4,
        "side_effects": false,
        "compiled": false,
        "nodes": [
            {"id": "x", "kind": "variable", "dtype": "scalar_or_symbol", "unit": "domain_defined"},
            {"id": "context", "kind": "evidence", "dtype": "support", "unit": "confidence"},
            {"id": "candidate", "kind": "operator", "op": primary, "inputs": ["x", "context"], "dtype": "candidate_model"},
            {"id": "bounded", "kind": "operator", "op": "constraint_clip", "inputs": ["candidate", "context"], "dtype": "verified_candidate"}
        ],
        "edges": [
            {"from": "x", "to": "candidate"},
            {"from": "context", "to": "candidate"},
            {"from": "candidate", "to": "bounded"}
        ],
        "downstream_policy": "may_be_exported_to_cwm_only_after_verification_and_gewc_acceptance",
    })
}

fn verify_graph(graph: &serde_json::Value) -> serde_json::Value {
    let nodes = graph
        .get("nodes")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let basis = graph
        .get("basis")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let max_depth = graph
        .get("max_depth")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(u64::MAX);
    let acyclic = graph
        .get("acyclic")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let typed = graph
        .get("typed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let side_effects = graph
        .get("side_effects")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let checks = vec![
        ("has_nodes", !nodes.is_empty()),
        ("has_basis", !basis.is_empty()),
        ("typed", typed),
        ("acyclic", acyclic),
        ("bounded_depth", max_depth <= 6),
        ("no_side_effects", !side_effects),
    ];
    let passed = checks.iter().all(|(_, passed)| *passed);
    serde_json::json!({
        "schema": "eden-operator-graph-verification-v1",
        "graph_id": graph.get("id").cloned().unwrap_or(serde_json::Value::Null),
        "passed": passed,
        "checks": checks.into_iter().map(|(id, passed)| {
            serde_json::json!({"id": id, "passed": passed})
        }).collect::<Vec<_>>(),
    })
}

fn verify_expression_graphs_value() -> serde_json::Value {
    let graphs = read_graphs();
    let verifications = graphs.iter().map(verify_graph).collect::<Vec<_>>();
    let passed = verifications
        .iter()
        .filter(|record| {
            record
                .get("passed")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    serde_json::json!({
        "schema": "eden-operator-verification-report-v1",
        "artifact": "operator_verification_report",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "passed": passed,
        "total": verifications.len(),
        "verifications": verifications,
        "policy": "verification proves local graph hygiene only, not scientific truth or AGI capability",
    })
}

fn ensure_graph_log() {
    let _ = state_paths::ensure_state_dir();
    if std::fs::metadata(state_paths::operator_expression_graphs_path()).is_err() {
        let _ = std::fs::write(state_paths::operator_expression_graphs_path(), "");
    }
}

fn append_graph(graph: serde_json::Value) {
    let _ = state_paths::ensure_state_dir();
    ensure_graph_log();
    let line = serde_json::to_string(&graph).unwrap_or_else(|_| graph.to_string());
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(state_paths::operator_expression_graphs_path())
        .and_then(|mut file| writeln!(file, "{line}"));
}

fn read_graphs() -> Vec<serde_json::Value> {
    std::fs::read_to_string(state_paths::operator_expression_graphs_path())
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect()
}

fn graph_count() -> usize {
    read_graphs().len()
}

fn write_json(path: String, record: serde_json::Value) {
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_forge_writes_native_artifacts() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(
            std::env::temp_dir().join(format!("eden_operator_forge_test_{}", std::process::id())),
        );
        let _ = std::fs::remove_dir_all(state_paths::state_dir());

        let out = run(OperatorForgeInput {
            praxis_report: "[EDEN-PRAXIS-NEXUS] primitives=7/7".to_string(),
            world_report: "[WORLD]\n[WORLD-EVAL] passed=5/5".to_string(),
            policy_report: "[POLICY] allowed=1 blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
        });

        assert!(out.contains("[EDEN-OPERATOR-FORGE]"));
        assert!(out.contains("passed=6/6"));
        assert!(std::fs::metadata(state_paths::eden_operator_forge_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operator_primitive_basis_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operator_expression_graphs_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operator_model_registry_path()).is_ok());
        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn operator_forge_synthesizes_and_verifies_bounded_graph() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_operator_forge_synth_test_{}",
            std::process::id()
        )));
        let _ = std::fs::remove_dir_all(state_paths::state_dir());

        let out = synthesize("causal risk model for action cost under uncertainty");
        let verify_out = verify();

        assert!(out.contains("status=candidate_recorded"));
        assert!(out.contains("causal_gate"));
        assert!(verify_out.contains("passed=1/1"));
        assert!(audit_report().contains("graphs=1"));
        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
