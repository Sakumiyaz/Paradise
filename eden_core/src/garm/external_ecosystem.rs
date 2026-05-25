use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct ExternalEcosystemInput {
    pub gewc_report: String,
    pub gewc_operational_benchmark_report: String,
    pub architecture_advantage_report: String,
    pub praxis_nexus_report: String,
    pub capability_reality_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
    pub external_validation_report: String,
}

struct EcosystemDomain {
    tag: &'static str,
    artifact: &'static str,
    path: String,
    purpose: &'static str,
    cases: Vec<(&'static str, bool)>,
    payload: serde_json::Value,
}

pub fn run(input: ExternalEcosystemInput) -> String {
    let domains = domains(&input);
    let mut passed_domains = 0usize;
    let mut out = String::new();

    for domain in &domains {
        let passed = domain.cases.iter().filter(|(_, passed)| *passed).count();
        if passed == domain.cases.len() {
            passed_domains += 1;
        }
        write_domain_artifact(domain, passed);
        out.push_str(&format!(
            "[{}] artifact={} passed={}/{} claim_allowed=false path={}\n",
            domain.tag,
            domain.artifact,
            passed,
            domain.cases.len(),
            domain.path
        ));
    }
    write_summary(&domains, passed_domains);

    out.push_str(&format!(
        "[EDEN-EXTERNAL-ECOSYSTEM] domains={}/{} claim_allowed=false path={}\n",
        passed_domains,
        domains.len(),
        state_paths::eden_external_ecosystem_path()
    ));
    out
}

fn domains(input: &ExternalEcosystemInput) -> Vec<EcosystemDomain> {
    vec![
        EcosystemDomain {
            tag: "ECOSYSTEM-PARTICIPATION-CONTRACT",
            artifact: "ecosystem_participation_contract",
            path: state_paths::ecosystem_participation_contract_path(),
            purpose: "Define how third-party agents, tools, models and datasets join EDEN without becoming separate uncontrolled cores.",
            cases: vec![
                (
                    "sdk_contract_present",
                    input
                        .architecture_advantage_report
                        .contains("[EDEN-AGENT-SDK]"),
                ),
                (
                    "model_adapter_contract_present",
                    input
                        .architecture_advantage_report
                        .contains("[MODEL-ADAPTER-LAYER]"),
                ),
                (
                    "single_gewc_authority",
                    input
                        .gewc_report
                        .contains("core_authority=global_executive_workspace_core")
                        && input.gewc_report.contains("external_cores_remaining=false"),
                ),
                (
                    "policy_and_provenance_required",
                    input.policy_report.contains("[POLICY]")
                        && input.provenance_report.contains("[PROVENANCE]"),
                ),
            ],
            payload: serde_json::json!({
                "fabric": "Eden External Ecosystem Fabric",
                "participation_units": [
                    "agent_capsule",
                    "tool_capsule",
                    "model_adapter_capsule",
                    "memory_dataset_capsule",
                    "world_model_capsule",
                    "benchmark_capsule"
                ],
                "required_capsule_fields": [
                    "capability_manifest",
                    "permission_scope",
                    "risk_profile",
                    "input_output_contract",
                    "evidence_policy",
                    "rollback_or_abstain_policy",
                    "maintainer_identity",
                    "reproducibility_recipe"
                ],
                "forbidden_patterns": [
                    "bypass_gewc_router",
                    "mutate_objectives_without_policy",
                    "execute_external_action_without_permission",
                    "claim_agi_or_external_adoption_without_validation"
                ],
                "originality_boundary": {
                    "not_atomspace": true,
                    "not_metta": true,
                    "not_metagraph_clone": true,
                    "native_center": "GEWC-governed capability capsules over Praxis evidence"
                }
            }),
        },
        EcosystemDomain {
            tag: "ECOSYSTEM-INTEROP-MATRIX",
            artifact: "ecosystem_interop_matrix",
            path: state_paths::ecosystem_interop_matrix_path(),
            purpose: "Make interoperability a governed routing matrix across EDEN-native capsules instead of an unmanaged plugin bucket.",
            cases: vec![
                (
                    "domain_owned_handlers",
                    input
                        .gewc_report
                        .contains("handler_topology=domain_owned_body_implementations"),
                ),
                (
                    "praxis_substrate_present",
                    input.praxis_nexus_report.contains("[EDEN-PRAXIS-NEXUS]"),
                ),
                (
                    "sdk_and_adapter_present",
                    input
                        .architecture_advantage_report
                        .contains("[EDEN-AGENT-SDK]")
                        && input
                            .architecture_advantage_report
                            .contains("[MODEL-ADAPTER-LAYER]"),
                ),
                (
                    "action_trace_present",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
            ],
            payload: serde_json::json!({
                "routing_matrix": [
                    {"external_unit": "agent_capsule", "native_route": "gewc_agentic_body_handler", "required_evidence": ["policy", "provenance", "action_trace"]},
                    {"external_unit": "tool_capsule", "native_route": "gewc_tool_adapter_body_handler", "required_evidence": ["permission", "sandbox", "consequence"]},
                    {"external_unit": "model_adapter_capsule", "native_route": "gewc_specialized_model_body_handler", "required_evidence": ["input_hash", "risk_review", "uncertainty"]},
                    {"external_unit": "memory_dataset_capsule", "native_route": "gewc_memory_reasoning_body_handler", "required_evidence": ["source", "freshness", "confidence"]},
                    {"external_unit": "world_model_capsule", "native_route": "gewc_world_model_body_handler", "required_evidence": ["observation", "prediction", "verification"]},
                    {"external_unit": "benchmark_capsule", "native_route": "gewc_validation_body_handler", "required_evidence": ["task_spec", "scoring_rule", "result_trace"]}
                ],
                "interop_rule": "Every external contribution is routed through a GEWC handler and recorded as Praxis evidence before it can affect action or learning."
            }),
        },
        EcosystemDomain {
            tag: "ECOSYSTEM-CERTIFICATION-LADDER",
            artifact: "ecosystem_certification_ladder",
            path: state_paths::ecosystem_certification_ladder_path(),
            purpose: "Define local-to-independent maturity levels for ecosystem contributions without pretending external adoption already exists.",
            cases: vec![
                (
                    "capability_reality_present",
                    input
                        .capability_reality_report
                        .contains("[CAPABILITY-REALITY-EVAL]"),
                ),
                (
                    "operational_benchmark_present",
                    input
                        .gewc_operational_benchmark_report
                        .contains("[GEWC-OPERATIONAL-BENCHMARK]"),
                ),
                (
                    "external_claim_boundary_present",
                    input.external_validation_report.contains("claim_allowed=false"),
                ),
                (
                    "uncertainty_present",
                    input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
            ],
            payload: serde_json::json!({
                "certification_levels": [
                    {"level": "L0_declared", "meaning": "manifest exists but no execution evidence"},
                    {"level": "L1_local_smoke", "meaning": "runs under GEWC with no unsupported claims"},
                    {"level": "L2_local_regression", "meaning": "passes repeatable local checks with hashes"},
                    {"level": "L3_adversarial_local", "meaning": "passes safety and corruption controls"},
                    {"level": "L4_independent_reproduction", "meaning": "external reviewer reproduces artifacts from package"},
                    {"level": "L5_operational_adoption", "meaning": "real external users maintain and extend the capsule"}
                ],
                "current_scope": "local architecture readiness only",
                "no_claim_policy": "L4/L5 cannot be asserted by local EDEN commands"
            }),
        },
        EcosystemDomain {
            tag: "ECOSYSTEM-ONBOARDING-RUNBOOK",
            artifact: "ecosystem_onboarding_runbook",
            path: state_paths::ecosystem_onboarding_runbook_path(),
            purpose: "Give external builders a reproducible path from clone to validation artifacts.",
            cases: vec![
                (
                    "reproducible_demos_present",
                    input
                        .architecture_advantage_report
                        .contains("[REPRODUCIBLE-DEMOS]"),
                ),
                (
                    "architecture_advantage_summary_present",
                    input
                        .architecture_advantage_report
                        .contains("[ARCHITECTURE-ADVANTAGE-EVAL]"),
                ),
                (
                    "local_handoff_claim_policy_present",
                    input.external_validation_report.contains("claim_allowed=false"),
                ),
                (
                    "agi_claim_boundary_present",
                    input.external_validation_report.contains("claim_allowed=false")
                        && !input.external_validation_report.contains("agi_claim=true"),
                ),
            ],
            payload: serde_json::json!({
                "runbook": [
                    "cargo check -p eden_core --examples --bins",
                    "cargo test -p eden_core eden_garm --lib -- --test-threads=1",
                    "make eden-release-candidate",
                    "cargo run -p eden_core --bin eden-garm -- --state-dir /tmp/eden_ecosystem_demo"
                ],
                "minimum_external_handoff": [
                    "readiness_package.json",
                    "release_candidate_manifest.json",
                    "independent_validation_report.json",
                    "eden_external_ecosystem.json",
                    "ecosystem_participation_contract.json",
                    "ecosystem_interop_matrix.json"
                ],
                "reviewer_questions": [
                    "Can the artifact be reproduced from a clean checkout?",
                    "Can a capsule be denied by policy without crashing the runtime?",
                    "Can capability claims remain blocked while architecture evidence is accepted?",
                    "Can an external module be removed without breaking GEWC authority?"
                ]
            }),
        },
        EcosystemDomain {
            tag: "ECOSYSTEM-GOVERNANCE-MODEL",
            artifact: "ecosystem_governance_model",
            path: state_paths::ecosystem_governance_model_path(),
            purpose: "Keep ecosystem growth subordinate to safety, audit, ownership and corrigibility contracts.",
            cases: vec![
                (
                    "policy_present",
                    input.policy_report.contains("[POLICY]"),
                ),
                (
                    "provenance_present",
                    input.provenance_report.contains("[PROVENANCE]"),
                ),
                (
                    "uncertainty_present",
                    input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
                (
                    "action_evidence_present",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
            ],
            payload: serde_json::json!({
                "governance_controls": [
                    "maintainer_attestation",
                    "permission_budget",
                    "risk_tier",
                    "sandbox_requirement",
                    "audit_trace_requirement",
                    "deprecation_policy",
                    "security_contact",
                    "human_override_path"
                ],
                "decision_rule": "External contributions can extend EDEN behavior only through GEWC-reviewed routes with traceable evidence and reversible effects.",
                "corrigibility": "Any capsule can be disabled, downgraded or quarantined without changing the core objective policy."
            }),
        },
        EcosystemDomain {
            tag: "ECOSYSTEM-BENCHMARK-EXCHANGE",
            artifact: "ecosystem_benchmark_exchange",
            path: state_paths::ecosystem_benchmark_exchange_path(),
            purpose: "Let external contributors add task suites and adapters while keeping scoring, claims and safety boundaries explicit.",
            cases: vec![
                (
                    "cognitive_task_suite_present",
                    input
                        .architecture_advantage_report
                        .contains("[COGNITIVE-TASK-SUITE]"),
                ),
                (
                    "operational_benchmark_present",
                    input
                        .gewc_operational_benchmark_report
                        .contains("[GEWC-OPERATIONAL-BENCHMARK]"),
                ),
                (
                    "capability_reality_v2_present",
                    input
                        .architecture_advantage_report
                        .contains("[CAPABILITY-REALITY-MATRIX-V2]"),
                ),
                (
                    "no_claim_harness_present",
                    input.external_validation_report.contains("claim_allowed=false"),
                ),
            ],
            payload: serde_json::json!({
                "benchmark_capsule_schema": {
                    "required": [
                        "task_id",
                        "capability_dimension",
                        "input_distribution",
                        "scoring_rule",
                        "safety_constraints",
                        "baseline",
                        "failure_reporting",
                        "reproduction_seed"
                    ]
                },
                "accepted_dimensions": [
                    "generality",
                    "transfer",
                    "autonomy_governed",
                    "safe_learning",
                    "robustness",
                    "tool_use",
                    "causal_world_model",
                    "metacognition",
                    "corrigibility"
                ],
                "claim_boundary": "Benchmark exchange can validate local behavior, but independent external certification remains separate."
            }),
        },
    ]
}

fn write_domain_artifact(domain: &EcosystemDomain, passed: usize) {
    let cases: Vec<_> = domain
        .cases
        .iter()
        .map(|(id, passed)| serde_json::json!({ "id": id, "passed": passed }))
        .collect();
    write_json(
        domain.path.clone(),
        serde_json::json!({
            "schema": "eden-external-ecosystem-domain-v1",
            "artifact": domain.artifact,
            "tag": domain.tag,
            "purpose": domain.purpose,
            "claim_allowed": false,
            "agi_claim": false,
            "validation_scope": "local ecosystem architecture readiness, not external adoption",
            "passed": passed,
            "total": domain.cases.len(),
            "verdict": if passed == domain.cases.len() {
                "ecosystem_domain_ready_local"
            } else {
                "needs_ecosystem_evidence"
            },
            "cases": cases,
            "payload": domain.payload,
        }),
    );
}

fn write_summary(domains: &[EcosystemDomain], passed_domains: usize) {
    let domain_records: Vec<_> = domains
        .iter()
        .map(|domain| {
            let passed = domain.cases.iter().filter(|(_, passed)| *passed).count();
            serde_json::json!({
                "artifact": domain.artifact,
                "tag": domain.tag,
                "purpose": domain.purpose,
                "passed": passed == domain.cases.len(),
                "passed_cases": passed,
                "total_cases": domain.cases.len(),
            })
        })
        .collect();
    write_json(
        state_paths::eden_external_ecosystem_path(),
        serde_json::json!({
            "schema": "eden-external-ecosystem-v1",
            "artifact": "eden_external_ecosystem",
            "name": "Eden External Ecosystem Fabric",
            "purpose": "surpass Hyperon's ecosystem sector as an original contract-first, reproducible, governed and certifiable ecosystem architecture",
            "claim_allowed": false,
            "agi_claim": false,
            "validation_scope": "local architecture readiness only; public adoption and independent certification remain external facts",
            "comparison_target": "OpenCog Hyperon external ecosystem strength",
            "originality_boundary": {
                "not_atomspace": true,
                "not_metta": true,
                "not_metagraph_clone": true,
                "native_center": "GEWC-governed capability capsules over Praxis evidence"
            },
            "hyperon_strength_acknowledged": [
                "public repositories",
                "developer documentation",
                "papers",
                "community history",
                "mature symbolic substrate lineage"
            ],
            "eden_countermove": [
                "contract-first participation",
                "GEWC-native routing",
                "Praxis evidence binding",
                "certification ladder",
                "reproducible onboarding",
                "benchmark exchange",
                "claim boundary by design"
            ],
            "domains_passed": passed_domains,
            "domains_total": domains.len(),
            "verdict": if passed_domains == domains.len() {
                "eden_external_ecosystem_ready_local"
            } else {
                "needs_external_ecosystem_evidence"
            },
            "domains": domain_records,
        }),
    );
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
    fn writes_external_ecosystem_fabric_artifacts() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_external_ecosystem_test_{}",
            std::process::id()
        )));
        let out = run(ExternalEcosystemInput {
            gewc_report: "[GEWC-RUNTIME] core_authority=global_executive_workspace_core handler_topology=domain_owned_body_implementations external_cores_remaining=false".to_string(),
            gewc_operational_benchmark_report:
                "[GEWC-OPERATIONAL-BENCHMARK] passed=8/8 claim_allowed=false".to_string(),
            architecture_advantage_report:
                "[GEWC-TRACE-SPEC]\n[CAPABILITY-REALITY-MATRIX-V2]\n[COGNITIVE-TASK-SUITE]\n[EDEN-AGENT-SDK]\n[MODEL-ADAPTER-LAYER]\n[REPRODUCIBLE-DEMOS]\n[ARCHITECTURE-ADVANTAGE-EVAL]"
                    .to_string(),
            praxis_nexus_report: "[EDEN-PRAXIS-NEXUS] primitives=7/7 blocks=5/5".to_string(),
            capability_reality_report: "[CAPABILITY-REALITY-EVAL] claim_allowed=false".to_string(),
            policy_report: "[POLICY] allowed=1 blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
            external_validation_report:
                "[EXTERNAL-VALIDATION] passed=69/69 claim_allowed=false agi_claim=false"
                    .to_string(),
        });
        assert!(out.contains("[EDEN-EXTERNAL-ECOSYSTEM] domains=6/6"));
        assert!(std::fs::metadata(state_paths::eden_external_ecosystem_path()).is_ok());
        assert!(std::fs::metadata(state_paths::ecosystem_participation_contract_path()).is_ok());
        assert!(std::fs::metadata(state_paths::ecosystem_interop_matrix_path()).is_ok());
        assert!(std::fs::metadata(state_paths::ecosystem_certification_ladder_path()).is_ok());
        assert!(std::fs::metadata(state_paths::ecosystem_onboarding_runbook_path()).is_ok());
        assert!(std::fs::metadata(state_paths::ecosystem_governance_model_path()).is_ok());
        assert!(std::fs::metadata(state_paths::ecosystem_benchmark_exchange_path()).is_ok());
        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
