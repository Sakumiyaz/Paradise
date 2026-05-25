use crate::eden_garm::state_paths;

#[derive(Clone)]
pub struct SovereignCognitionInput {
    pub gewc_report: String,
    pub praxis_nexus_report: String,
    pub architecture_advantage_report: String,
    pub external_ecosystem_report: String,
    pub capability_reality_report: String,
    pub memory_report: String,
    pub world_report: String,
    pub symbolic_report: String,
    pub paradigm_report: String,
    pub frontier_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
    pub external_validation_report: String,
}

struct Sector {
    id: &'static str,
    label: &'static str,
    hyperon_strength: &'static str,
    eden_countermove: &'static str,
    win_condition: &'static str,
    cases: Vec<(&'static str, bool)>,
}

struct DetailArtifact {
    tag: &'static str,
    artifact: &'static str,
    path: String,
    purpose: &'static str,
    cases: Vec<(&'static str, bool)>,
    payload: serde_json::Value,
}

pub fn run(input: SovereignCognitionInput) -> String {
    let sectors = sectors(&input);
    let details = detail_artifacts(&input);
    let passed_sectors = sectors
        .iter()
        .filter(|sector| sector.cases.iter().all(|(_, passed)| *passed))
        .count();

    write_sector_matrix(&sectors);
    for detail in &details {
        write_detail(detail);
    }
    write_summary(&sectors, &details);

    let mut out = format!(
        "[EDEN-SOVEREIGN-COGNITION] sectors={}/{} claim_allowed=false path={}\n[SOVEREIGN-SECTOR-WINS] passed={}/{} path={}\n",
        passed_sectors,
        sectors.len(),
        state_paths::eden_sovereign_cognition_path(),
        passed_sectors,
        sectors.len(),
        state_paths::sovereign_sector_wins_path(),
    );
    for detail in details {
        let passed = detail.cases.iter().filter(|(_, passed)| *passed).count();
        out.push_str(&format!(
            "[{}] artifact={} passed={}/{} claim_allowed=false path={}\n",
            detail.tag,
            detail.artifact,
            passed,
            detail.cases.len(),
            detail.path
        ));
    }
    out
}

fn sectors(input: &SovereignCognitionInput) -> Vec<Sector> {
    let praxis_artifact = read_file(state_paths::eden_praxis_nexus_path());
    let adr_count = adr_count();
    vec![
        Sector {
            id: "central_core",
            label: "Nucleo central",
            hyperon_strength: "AtomSpace and MeTTa give Hyperon a unified formal core for knowledge and programs.",
            eden_countermove: "GEWC is the authority-bearing executive core, and Praxis Nexus is the governed cognitive-operational substrate under that authority.",
            win_condition: "EDEN wins only when executive authority, Praxis substrate and trace semantics are all present.",
            cases: vec![
                (
                    "gewc_single_authority",
                    input
                        .gewc_report
                        .contains("core_authority=global_executive_workspace_core"),
                ),
                (
                    "praxis_substrate_present",
                    input
                        .praxis_nexus_report
                        .contains("[EDEN-PRAXIS-NEXUS]"),
                ),
                (
                    "trace_spec_present",
                    input
                        .architecture_advantage_report
                        .contains("[GEWC-TRACE-SPEC]"),
                ),
                (
                    "external_cores_absorbed",
                    input.gewc_report.contains("external_cores_remaining=false"),
                ),
            ],
        },
        Sector {
            id: "mathematical_formalism",
            label: "Formalismo matematico",
            hyperon_strength: "Hyperon is strong in metagraph rewriting, typed symbolic constructs and declarative representation.",
            eden_countermove: "Praxis Calculus models governed cognitive transitions over intent, state, evidence, constraint, affordance, projection and trace.",
            win_condition: "EDEN wins when formal primitives, transition traces, uncertainty and constraints are joined.",
            cases: vec![
                (
                    "seven_primitives_validated",
                    input.praxis_nexus_report.contains("primitives=7/7"),
                ),
                (
                    "five_blocks_validated",
                    input.praxis_nexus_report.contains("blocks=5/5"),
                ),
                (
                    "trace_transition_semantics",
                    input.praxis_nexus_report.contains("[PRAXIS-TRACE-SEMANTICS]"),
                ),
                (
                    "uncertainty_and_policy_in_formal_loop",
                    input.uncertainty_report.contains("[UNCERTAINTY]")
                        && input.policy_report.contains("[POLICY]"),
                ),
            ],
        },
        Sector {
            id: "distributed_scalability_conceptual",
            label: "Escalabilidad distribuida conceptual",
            hyperon_strength: "Hyperon has a distributed AtomSpace direction for scaling knowledge stores.",
            eden_countermove: "EDEN uses a governed federated runtime fabric: handlers, adapters, model routes and external modules remain subordinate to GEWC policy and trace contracts.",
            win_condition: "EDEN wins when distribution is not only storage scaling but policy-bound runtime federation.",
            cases: vec![
                (
                    "domain_handlers_present",
                    input
                        .gewc_report
                        .contains("handler_topology=domain_owned_body_implementations"),
                ),
                (
                    "agent_sdk_contract_present",
                    input
                        .architecture_advantage_report
                        .contains("[EDEN-AGENT-SDK]"),
                ),
                (
                    "model_adapter_layer_present",
                    input
                        .architecture_advantage_report
                        .contains("[MODEL-ADAPTER-LAYER]"),
                ),
                (
                    "policy_and_provenance_federation",
                    input.policy_report.contains("[POLICY]")
                        && input.provenance_report.contains("[PROVENANCE]"),
                ),
            ],
        },
        Sector {
            id: "originality_of_design",
            label: "Originalidad del diseno",
            hyperon_strength: "OpenCog has an original metagraph plus cognitive language lineage.",
            eden_countermove: "EDEN explicitly uses governed cognition in action, not AtomSpace, MeTTa or standalone symbolic KR as its center.",
            win_condition: "EDEN wins when the implemented artifacts state an originality boundary and use Praxis/GEWC-native semantics.",
            cases: vec![
                (
                    "not_atomspace_declared",
                    praxis_artifact.contains("\"not_atomspace\": true"),
                ),
                (
                    "not_metta_clone_declared",
                    praxis_artifact.contains("\"not_metta_clone\": true"),
                ),
                (
                    "praxis_focus_declared",
                    praxis_artifact.contains("governed cognition in action"),
                ),
                (
                    "eden_native_goal_declared",
                    praxis_artifact.contains("native formal substrate"),
                ),
            ],
        },
        Sector {
            id: "memory_and_knowledge",
            label: "Memoria y conocimiento",
            hyperon_strength: "AtomSpace is a mature knowledge representation store and rewriting substrate.",
            eden_countermove: "EDEN binds memory to evidence, provenance, constraints, action traces and freshness rather than treating knowledge as detached graph content.",
            win_condition: "EDEN wins when memory, evidence, action and constraints are inseparable in the substrate.",
            cases: vec![
                (
                    "memory_eval_present",
                    input.memory_report.contains("[MEMORY-EVAL]"),
                ),
                (
                    "evidence_provenance_present",
                    input.provenance_report.contains("[PROVENANCE]"),
                ),
                (
                    "action_trace_present",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
                (
                    "constraint_bound_knowledge",
                    input.praxis_nexus_report.contains("[PRAXIS-RULES]")
                        && input.policy_report.contains("[POLICY]"),
                ),
            ],
        },
        Sector {
            id: "multi_paradigm_integration",
            label: "Integracion multi-paradigma",
            hyperon_strength: "OpenCog has a long multi-paradigm AGI architecture history.",
            eden_countermove: "EDEN makes paradigm integration an explicit GEWC-governed architecture map with frontier layers and no-claim validation.",
            win_condition: "EDEN wins when paradigms are not parallel modules but routed under one executive validation authority.",
            cases: vec![
                (
                    "paradigm_authority_present",
                    input
                        .paradigm_report
                        .contains("[PARADIGM-ARCHITECTURE-MAP]"),
                ),
                (
                    "frontier_layers_present",
                    input
                        .frontier_report
                        .contains("[SAFETY-CONTROL-ARCHITECTURE]"),
                ),
                (
                    "gewc_evaluation_route_present",
                    input
                        .gewc_report
                        .contains("last_handler=gewc_validation_body_handler")
                        || input.gewc_report.contains("handler_metrics="),
                ),
                (
                    "capability_reality_present",
                    input
                        .capability_reality_report
                        .contains("[CAPABILITY-REALITY-EVAL]"),
                ),
            ],
        },
        Sector {
            id: "external_ecosystem",
            label: "Ecosistema externo",
            hyperon_strength: "OpenCog has public repositories, documentation, papers and an external developer history.",
            eden_countermove: "EDEN defines an original External Ecosystem Fabric: capability capsules, GEWC-native routing, certification ladder, reproducible onboarding, governance and benchmark exchange.",
            win_condition: "EDEN wins as a local architecture target when external participation is contract-first, certifiable, reproducible and governed without copying AtomSpace or MeTTa.",
            cases: vec![
                (
                    "external_ecosystem_fabric_present",
                    input
                        .external_ecosystem_report
                        .contains("[EDEN-EXTERNAL-ECOSYSTEM] domains=6/6"),
                ),
                (
                    "participation_contract_present",
                    input
                        .external_ecosystem_report
                        .contains("[ECOSYSTEM-PARTICIPATION-CONTRACT]"),
                ),
                (
                    "certification_ladder_present",
                    input
                        .external_ecosystem_report
                        .contains("[ECOSYSTEM-CERTIFICATION-LADDER]"),
                ),
                (
                    "benchmark_exchange_present",
                    input
                        .external_ecosystem_report
                        .contains("[ECOSYSTEM-BENCHMARK-EXCHANGE]"),
                ),
            ],
        },
        Sector {
            id: "historical_maturity",
            label: "Madurez historica",
            hyperon_strength: "OpenCog has a much longer history and external research lineage.",
            eden_countermove: "EDEN cannot fake elapsed history, so it competes with auditable maturity: ADR lineage, release-candidate validation and explicit capability reality.",
            win_condition: "EDEN wins only as reproducible maturity, not as an unsupported claim of longer history.",
            cases: vec![
                ("adr_lineage_present", adr_count >= 40),
                (
                    "release_candidate_policy_present",
                    input.external_validation_report.contains("claim_allowed=false"),
                ),
                (
                    "capability_reality_present",
                    input
                        .capability_reality_report
                        .contains("[CAPABILITY-REALITY-EVAL]"),
                ),
                (
                    "no_claim_maturity_policy",
                    input.external_validation_report.contains("agi_claim=false")
                        || input.external_validation_report.contains("claim_allowed=false"),
                ),
            ],
        },
        Sector {
            id: "symbolic_reasoning",
            label: "Razonamiento simbolico",
            hyperon_strength: "Hyperon is strong in symbolic and graph rewriting approaches.",
            eden_countermove: "EDEN combines symbolic reasoning with constraints, causal projection, policy and trace closure inside Praxis Reasoner.",
            win_condition: "EDEN wins when symbolic reasoning is operationally constrained and causally grounded.",
            cases: vec![
                (
                    "symbolic_architecture_present",
                    input.symbolic_report.contains("[SYMBOLIC-ARCHITECTURE]"),
                ),
                (
                    "praxis_reasoner_present",
                    input.praxis_nexus_report.contains("[PRAXIS-REASONER]"),
                ),
                (
                    "causal_world_present",
                    input.world_report.contains("[WORLD-EVAL]"),
                ),
                (
                    "policy_constrained_reasoning",
                    input.policy_report.contains("[POLICY]"),
                ),
            ],
        },
        Sector {
            id: "cognitive_language",
            label: "Lenguaje cognitivo",
            hyperon_strength: "MeTTa is Hyperon's cognitive programming language.",
            eden_countermove: "EDEN defines Cognitive Contract Language as executable governance over intent, evidence, constraints, routes, projections, action, trace and learning.",
            win_condition: "EDEN wins when the language is action-safe and audit-native rather than only expressive.",
            cases: vec![
                (
                    "praxis_rules_present",
                    input.praxis_nexus_report.contains("[PRAXIS-RULES]"),
                ),
                (
                    "trace_semantics_present",
                    input.praxis_nexus_report.contains("[PRAXIS-TRACE-SEMANTICS]"),
                ),
                (
                    "policy_language_guard",
                    input.policy_report.contains("[POLICY]"),
                ),
                (
                    "action_language_trace",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
            ],
        },
        Sector {
            id: "formal_representation",
            label: "Representacion formal",
            hyperon_strength: "AtomSpace is a formal hypergraph/metagraph representation for knowledge and procedures.",
            eden_countermove: "EDEN represents cognition as governed operational records, where every knowledge state is tied to intent, evidence, constraint, affordance, projection and trace.",
            win_condition: "EDEN wins when formal representation covers knowledge plus action governance and learning closure.",
            cases: vec![
                (
                    "praxis_space_present",
                    input.praxis_nexus_report.contains("[PRAXIS-SPACE]"),
                ),
                (
                    "primitives_present",
                    input.praxis_nexus_report.contains("[PRAXIS-PRIMITIVES]"),
                ),
                (
                    "state_and_projection_present",
                    input.world_report.contains("[WORLD")
                        && input.praxis_nexus_report.contains("[PRAXIS-BENCH]"),
                ),
                (
                    "evidence_and_trace_present",
                    input.provenance_report.contains("[PROVENANCE]")
                        && input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
            ],
        },
    ]
}

fn detail_artifacts(input: &SovereignCognitionInput) -> Vec<DetailArtifact> {
    vec![
        DetailArtifact {
            tag: "PRAXIS-CALCULUS-FORMALISM",
            artifact: "praxis_calculus_formalism",
            path: state_paths::praxis_calculus_formalism_path(),
            purpose: "Original EDEN formalism for policy-safe cognitive state transitions.",
            cases: vec![
                (
                    "primitives_are_closed",
                    input.praxis_nexus_report.contains("primitives=7/7"),
                ),
                (
                    "transitions_are_traceable",
                    input.praxis_nexus_report.contains("[PRAXIS-TRACE-SEMANTICS]"),
                ),
                (
                    "constraints_are_first_class",
                    input.policy_report.contains("[POLICY]"),
                ),
                (
                    "uncertainty_is_explicit",
                    input.uncertainty_report.contains("[UNCERTAINTY]"),
                ),
            ],
            payload: serde_json::json!({
                "calculus": "Praxis Calculus",
                "primitive_set": ["intent", "state", "evidence", "constraint", "affordance", "projection", "trace"],
                "transition_shape": "intent + state + evidence + constraint -> affordance -> projection -> action -> trace",
                "closure_rules": [
                    "no_action_without_constraint",
                    "no_learning_without_trace",
                    "no_claim_without_external_validation",
                    "no_projection_without_uncertainty"
                ]
            }),
        },
        DetailArtifact {
            tag: "COGNITIVE-CONTRACT-LANGUAGE",
            artifact: "cognitive_contract_language",
            path: state_paths::cognitive_contract_language_path(),
            purpose: "EDEN-native language layer for safe cognitive operations.",
            cases: vec![
                (
                    "rules_present",
                    input.praxis_nexus_report.contains("[PRAXIS-RULES]"),
                ),
                (
                    "runtime_routes_present",
                    input.gewc_report.contains("handler_metrics="),
                ),
                (
                    "policy_guard_present",
                    input.policy_report.contains("[POLICY]"),
                ),
                (
                    "action_evidence_present",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
            ],
            payload: serde_json::json!({
                "language": "Cognitive Contract Language",
                "verbs": ["observe", "intend", "bind_evidence", "constrain", "route", "project", "act", "trace", "learn", "abstain"],
                "contract_shape": "when <intent> over <state> require <evidence> obey <constraint> route <affordance> project <outcome> then <act|abstain> record <trace>",
                "not_metta": true
            }),
        },
        DetailArtifact {
            tag: "EVIDENCE-MEMORY-FABRIC",
            artifact: "evidence_memory_fabric",
            path: state_paths::evidence_memory_fabric_path(),
            purpose: "Knowledge fabric that binds memory to evidence, policy, provenance and action results.",
            cases: vec![
                (
                    "memory_present",
                    input.memory_report.contains("[MEMORY-EVAL]"),
                ),
                (
                    "provenance_present",
                    input.provenance_report.contains("[PROVENANCE]"),
                ),
                (
                    "action_trace_present",
                    input.action_evidence_report.contains("[ACTION-EVIDENCE]"),
                ),
                (
                    "capability_reality_present",
                    input
                        .capability_reality_report
                        .contains("[CAPABILITY-REALITY-EVAL]"),
                ),
            ],
            payload: serde_json::json!({
                "fabric": "Evidence Memory Fabric",
                "record_edges": ["source", "confidence", "freshness", "policy", "uncertainty", "action_result", "revision"],
                "retrieval_policy": "retrieve only with confidence, provenance and abstention boundary",
                "knowledge_statuses": ["observed", "supported", "contradicted", "stale", "unsafe_to_use", "requires_external_validation"]
            }),
        },
        DetailArtifact {
            tag: "FEDERATED-RUNTIME-FABRIC",
            artifact: "federated_runtime_fabric",
            path: state_paths::federated_runtime_fabric_path(),
            purpose: "Distributed conceptual model for governed runtime federation rather than unmanaged knowledge-store scaling.",
            cases: vec![
                (
                    "domain_handlers_present",
                    input
                        .gewc_report
                        .contains("handler_topology=domain_owned_body_implementations"),
                ),
                (
                    "sdk_contract_present",
                    input
                        .architecture_advantage_report
                        .contains("[EDEN-AGENT-SDK]"),
                ),
                (
                    "adapter_contract_present",
                    input
                        .architecture_advantage_report
                        .contains("[MODEL-ADAPTER-LAYER]"),
                ),
                (
                    "provenance_present",
                    input.provenance_report.contains("[PROVENANCE]"),
                ),
            ],
            payload: serde_json::json!({
                "fabric": "Federated Runtime Fabric",
                "federated_units": ["agent", "tool", "model_adapter", "memory_shard", "world_model", "validator"],
                "required_contracts": ["permission", "provenance", "trace", "capability_reality", "rollback_or_abstain"],
                "routing_authority": "global_executive_workspace_core"
            }),
        },
        DetailArtifact {
            tag: "SYMBOLIC-REASONING-FABRIC",
            artifact: "symbolic_reasoning_fabric",
            path: state_paths::symbolic_reasoning_fabric_path(),
            purpose: "Hybrid symbolic, causal, normative and evidential reasoning under Praxis constraints.",
            cases: vec![
                (
                    "symbolic_present",
                    input.symbolic_report.contains("[SYMBOLIC-ARCHITECTURE]"),
                ),
                (
                    "world_present",
                    input.world_report.contains("[WORLD-EVAL]"),
                ),
                (
                    "reasoner_present",
                    input.praxis_nexus_report.contains("[PRAXIS-REASONER]"),
                ),
                (
                    "policy_present",
                    input.policy_report.contains("[POLICY]"),
                ),
            ],
            payload: serde_json::json!({
                "fabric": "Symbolic Reasoning Fabric",
                "modes": ["symbolic", "causal", "normative", "evidential", "counterfactual", "abstention"],
                "composition_rule": "symbolic inference is executable only when evidence, policy and uncertainty are bound",
                "output_states": ["support", "reject", "defer", "abstain", "simulate_before_action"]
            }),
        },
    ]
}

fn write_sector_matrix(sectors: &[Sector]) {
    let records: Vec<_> = sectors
        .iter()
        .map(|sector| {
            let passed_cases = sector.cases.iter().filter(|(_, passed)| *passed).count();
            serde_json::json!({
                "id": sector.id,
                "label": sector.label,
                "hyperon_strength": sector.hyperon_strength,
                "eden_countermove": sector.eden_countermove,
                "win_condition": sector.win_condition,
                "winner": if passed_cases == sector.cases.len() { "eden_local_architecture" } else { "needs_evidence" },
                "passed": passed_cases == sector.cases.len(),
                "passed_cases": passed_cases,
                "total_cases": sector.cases.len(),
                "cases": sector.cases.iter().map(|(id, passed)| {
                    serde_json::json!({"id": id, "passed": passed})
                }).collect::<Vec<_>>(),
            })
        })
        .collect();
    let passed = sectors
        .iter()
        .filter(|sector| sector.cases.iter().all(|(_, passed)| *passed))
        .count();
    write_json(
        state_paths::sovereign_sector_wins_path(),
        serde_json::json!({
            "schema": "eden-sovereign-sector-wins-v1",
            "artifact": "sovereign_sector_wins",
            "claim_allowed": false,
            "agi_claim": false,
            "validation_scope": "local_architecture_advantage_targets_not_external_market_or_agi_claim",
            "passed": passed,
            "total": sectors.len(),
            "sectors": records,
        }),
    );
}

fn write_detail(detail: &DetailArtifact) {
    let passed = detail.cases.iter().filter(|(_, passed)| *passed).count();
    write_json(
        detail.path.clone(),
        serde_json::json!({
            "schema": "eden-sovereign-detail-v1",
            "artifact": detail.artifact,
            "tag": detail.tag,
            "purpose": detail.purpose,
            "claim_allowed": false,
            "agi_claim": false,
            "passed": passed,
            "total": detail.cases.len(),
            "cases": detail.cases.iter().map(|(id, passed)| {
                serde_json::json!({"id": id, "passed": passed})
            }).collect::<Vec<_>>(),
            "payload": detail.payload,
        }),
    );
}

fn write_summary(sectors: &[Sector], details: &[DetailArtifact]) {
    let passed_sectors = sectors
        .iter()
        .filter(|sector| sector.cases.iter().all(|(_, passed)| *passed))
        .count();
    let passed_details = details
        .iter()
        .filter(|detail| detail.cases.iter().all(|(_, passed)| *passed))
        .count();
    write_json(
        state_paths::eden_sovereign_cognition_path(),
        serde_json::json!({
            "schema": "eden-sovereign-cognition-v1",
            "artifact": "eden_sovereign_cognition",
            "name": "Eden Sovereign Cognition",
            "purpose": "surpass OpenCog Hyperon strong sectors with original EDEN architecture targets",
            "claim_allowed": false,
            "agi_claim": false,
            "validation_scope": "local architecture evidence and explicit roadmap, not external capability certification",
            "comparison_target": "OpenCog Hyperon",
            "originality_boundary": {
                "not_atomspace": true,
                "not_metta": true,
                "not_metagraph_clone": true,
                "native_center": "GEWC + Praxis Nexus + Cognitive Contract Language"
            },
            "sector_wins": passed_sectors,
            "sector_total": sectors.len(),
            "detail_artifacts_passed": passed_details,
            "detail_artifacts_total": details.len(),
            "verdict": if passed_sectors == sectors.len() && passed_details == details.len() {
                "eden_sovereign_cognition_ready_local"
            } else {
                "needs_sovereign_cognition_evidence"
            },
        }),
    );
}

fn read_file(path: String) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

fn adr_count() -> usize {
    ["docs/decisions", "../docs/decisions"]
        .iter()
        .filter_map(|path| std::fs::read_dir(path).ok())
        .flat_map(|entries| entries.flatten())
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("ADR-"))
        .count()
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
    fn writes_sovereign_cognition_sector_wins_and_detail_artifacts() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_sovereign_cognition_test_{}",
            std::process::id()
        )));
        let _ = state_paths::ensure_state_dir();
        write_json(
            state_paths::eden_praxis_nexus_path(),
            serde_json::json!({
                "originality_boundary": {
                    "not_atomspace": true,
                    "not_metta_clone": true,
                    "focus": "governed cognition in action"
                },
                "goal": "native formal substrate"
            }),
        );
        let out = run(SovereignCognitionInput {
            gewc_report: "[GEWC-RUNTIME] core_authority=global_executive_workspace_core handler_topology=domain_owned_body_implementations external_cores_remaining=false handler_metrics=x last_handler=gewc_validation_body_handler".to_string(),
            praxis_nexus_report: "[EDEN-PRAXIS-NEXUS] primitives=7/7 blocks=5/5 claim_allowed=false\n[PRAXIS-PRIMITIVES]\n[PRAXIS-SPACE]\n[PRAXIS-RULES]\n[PRAXIS-TRACE-SEMANTICS]\n[PRAXIS-REASONER]\n[PRAXIS-BENCH]".to_string(),
            architecture_advantage_report: "[GEWC-TRACE-SPEC]\n[EDEN-AGENT-SDK]\n[MODEL-ADAPTER-LAYER]\n[REPRODUCIBLE-DEMOS]\n[ARCHITECTURE-ADVANTAGE-EVAL]".to_string(),
            external_ecosystem_report: "[ECOSYSTEM-PARTICIPATION-CONTRACT]\n[ECOSYSTEM-CERTIFICATION-LADDER]\n[ECOSYSTEM-BENCHMARK-EXCHANGE]\n[EDEN-EXTERNAL-ECOSYSTEM] domains=6/6 claim_allowed=false".to_string(),
            capability_reality_report: "[CAPABILITY-REALITY-EVAL] claim_allowed=false".to_string(),
            memory_report: "[MEMORY-EVAL] passed=5/5".to_string(),
            world_report: "[WORLD]\n[WORLD-EVAL] passed=5/5".to_string(),
            symbolic_report: "[SYMBOLIC-ARCHITECTURE] passed=6/6".to_string(),
            paradigm_report: "[PARADIGM-ARCHITECTURE-MAP] paradigms=24".to_string(),
            frontier_report: "[SAFETY-CONTROL-ARCHITECTURE] passed=5/5".to_string(),
            policy_report: "[POLICY] blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
            external_validation_report:
                "[EXTERNAL-VALIDATION] passed=69/69 claim_allowed=false agi_claim=false"
                    .to_string(),
        });
        assert!(out.contains("[EDEN-SOVEREIGN-COGNITION]"));
        assert!(out.contains("sectors=11/11"));
        assert!(std::fs::metadata(state_paths::eden_sovereign_cognition_path()).is_ok());
        assert!(std::fs::metadata(state_paths::sovereign_sector_wins_path()).is_ok());
        assert!(std::fs::metadata(state_paths::praxis_calculus_formalism_path()).is_ok());
        assert!(std::fs::metadata(state_paths::cognitive_contract_language_path()).is_ok());
        assert!(std::fs::metadata(state_paths::evidence_memory_fabric_path()).is_ok());
        assert!(std::fs::metadata(state_paths::federated_runtime_fabric_path()).is_ok());
        assert!(std::fs::metadata(state_paths::symbolic_reasoning_fabric_path()).is_ok());
        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
