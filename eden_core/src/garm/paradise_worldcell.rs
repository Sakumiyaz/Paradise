use crate::eden_garm::nodes::{learning_ledger, world_model_core};
use crate::eden_garm::{
    action_evidence, eden_locus_layer, eden_operator_forge, operational_api, runtime_spine,
    state_paths,
};

pub const PARADISE_WORLDCELL_SCHEMA: &str = "eden-paradise-worldcell-runtime-v1";
pub const PARADISE_WORLDCELL_SESSION_SCHEMA: &str = "eden-paradise-worldcell-session-v1";

#[derive(Clone)]
pub struct ParadiseWorldcellInput {
    pub gewc_report: String,
    pub praxis_report: String,
    pub locus_report: String,
    pub forge_report: String,
    pub operational_report: String,
    pub model_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
}

struct WorldcellCheck {
    id: &'static str,
    component: &'static str,
    purpose: &'static str,
    passed: bool,
    evidence: &'static str,
}

pub struct ParadiseExecutionRequest {
    pub session_id: String,
    pub raw_command: String,
}

pub enum ParadiseExecutionPreparation {
    Ready(ParadiseExecutionRequest),
    Blocked(String),
}

pub fn run(input: ParadiseWorldcellInput) -> String {
    let checks = checks(&input);
    let passed = checks.iter().filter(|check| check.passed).count();
    let total = checks.len();
    let record = record(&checks, passed, total);
    let _ = state_paths::ensure_state_dir();
    let path = state_paths::paradise_worldcell_runtime_path();
    let _ = std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
    format!(
        "[PARADISE-WORLDCELL-RUNTIME] passed={}/{} claim_allowed=false path={}\n",
        passed, total, path
    )
}

pub fn record_intent(intent: &str) -> String {
    let intent = intent.trim();
    if intent.is_empty() {
        return "[PARADISE-INTENT] status=rejected reason=empty_intent\n".to_string();
    }
    let mut store = read_session_store();
    ensure_sessions_array(&mut store);
    let session_index = sessions(&store).len() + 1;
    let session_id = format!(
        "paradise-{:016x}",
        fnv64(format!("{}|{}", intent, session_index).as_bytes())
    );
    let locus_ingest = eden_locus_layer::ingest(&format!("operator intent :: {}", intent));
    let locus_context = eden_locus_layer::context_packet(intent);
    let evidence = action_evidence::record_attempt(
        "paradise_worldcell",
        intent,
        "allowed",
        "intent_recorded",
        "worldcell_session_created",
        "paradise_worldcell_sessions_json",
        "low",
    );
    let spine_event = runtime_spine::publish_event(
        "paradise_worldcell",
        "planning_goals",
        "intent_recorded",
        "low",
        serde_json::json!({
            "session_id": session_id,
            "intent": intent,
            "stage": "intent"
        }),
    );
    let spine_state = runtime_spine::record_state_mutation(
        "paradise_worldcell",
        "paradise_latest_session",
        "create",
        "intent_recorded",
        serde_json::json!({
            "session_id": session_id,
            "intent": intent
        }),
    );
    let session = serde_json::json!({
        "id": session_id,
        "intent": intent,
        "status": "intent_recorded",
        "current_stage": "intent",
        "authority": "global_executive_workspace_core",
        "created_by": "paradise_intent_command",
        "claim_allowed": false,
        "agi_claim": false,
        "candidate_command": null,
        "approval": {
            "status": "not_requested",
            "scope": "none",
            "required_by_paradise": true
        },
        "worldcell_loop": worldcell_loop_status("intent"),
        "steps": [
            step_record("intent", "completed", "operator_intent_admitted", intent),
            step_record("locus_context", "completed", "locus_authority_context_prepared", &format!("{}{}", locus_ingest, locus_context))
        ],
        "runtime_boundary": {
            "external_action_executed": false,
            "model_weights_mutated": false,
            "direct_tool_call_allowed": false
        }
    });
    if let Some(records) = store
        .get_mut("sessions")
        .and_then(serde_json::Value::as_array_mut)
    {
        records.push(session);
    }
    store["latest_session_id"] = serde_json::json!(session_id);
    write_session_store(&store);
    format!(
        "[PARADISE-INTENT] id={} status=intent_recorded path={}\n{}{}{}",
        store
            .get("latest_session_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown"),
        state_paths::paradise_worldcell_sessions_path(),
        evidence,
        spine_event,
        spine_state
    )
}

pub fn plan_session(spec: &str) -> String {
    let mut store = read_session_store();
    let Some(index) = resolve_session_index(&store, spec) else {
        return format!(
            "[PARADISE-PLAN] status=rejected reason=session_not_found spec=\"{}\" path={}\n",
            compact(spec, 80),
            state_paths::paradise_worldcell_sessions_path()
        );
    };
    let intent = store["sessions"][index]
        .get("intent")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unspecified intent")
        .to_string();
    let candidate = candidate_command_for(&intent);
    let dry_run_text = operational_api::action_dry_run_json(&candidate);
    let dry_run = serde_json::from_str::<serde_json::Value>(&dry_run_text).unwrap_or_else(|_| {
        serde_json::json!({
            "schema": "eden-action-dry-run-v1",
            "raw_command": candidate,
            "parse_error": true
        })
    });
    let observation = world_model_core::observe(
        "paradise_worldcell_plan",
        &format!(
            "intent '{}' maps to candidate command '{}'",
            intent, candidate
        ),
    );
    let prediction = world_model_core::predict(&candidate);
    let forge = eden_operator_forge::synthesize(&format!(
        "paradise action contract for intent '{}' using command '{}'",
        intent, candidate
    ));
    let verification = eden_operator_forge::verify();
    let risk = dry_run
        .get("risk_level")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let permission_key = dry_run
        .get("persistent_permission")
        .and_then(|value| value.get("permission_key"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let requires_human = dry_run
        .get("requires_human_approval")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let mutates_runtime = dry_run
        .get("mutates_runtime")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let requires_supervision = dry_run
        .get("requires_supervision")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let standalone_execution_allowed = dry_run
        .get("standalone_execution_allowed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let session = &mut store["sessions"][index];
    session["status"] = serde_json::json!("planned");
    session["current_stage"] = serde_json::json!("dry_run_complete");
    session["candidate_command"] = serde_json::json!(candidate);
    session["dry_run"] = dry_run;
    session["plan"] = serde_json::json!({
        "schema": "eden-paradise-worldcell-plan-v1",
        "intent": intent,
        "candidate_command": session.get("candidate_command").cloned().unwrap_or_default(),
        "risk_level": risk,
        "permission_key": permission_key,
        "requires_human_approval_by_dry_run": requires_human,
        "requires_supervision": requires_supervision,
        "mutates_runtime": mutates_runtime,
        "standalone_execution_allowed": standalone_execution_allowed,
        "paradise_approval_required": true,
        "stages": [
            "intent",
            "locus_context_authority",
            "dry_run",
            "world_model_simulation",
            "operator_forge_contract",
            "permission_gate",
            "runtime_body_execution",
            "evidence_memory"
        ],
        "world_model": {
            "observation": observation.trim(),
            "prediction": prediction.trim()
        },
        "operator_forge": {
            "synthesis": forge.trim(),
            "verification": verification.trim()
        }
    });
    session["approval"] = serde_json::json!({
        "status": "pending",
        "scope": "once",
        "required_by_paradise": true,
        "required_by_dry_run": requires_human,
        "permission_key": permission_key,
    });
    session["worldcell_loop"] = serde_json::Value::Array(worldcell_loop_status("plan"));
    push_step(
        session,
        step_record(
            "plan_dry_run",
            "completed",
            "candidate_command_classified_without_execution",
            &dry_run_text,
        ),
    );
    push_step(
        session,
        step_record(
            "world_model_simulation",
            "completed",
            "consequence_simulated_before_action",
            &format!("{}{}", observation, prediction),
        ),
    );
    push_step(
        session,
        step_record(
            "operator_forge_contract",
            "completed",
            "action_contract_synthesized_and_verified",
            &format!("{}{}", forge, verification),
        ),
    );
    let evidence = action_evidence::record_attempt(
        "paradise_worldcell",
        &intent,
        "allowed",
        "planned",
        "dry_run_world_model_and_action_contract_written",
        "paradise_worldcell_sessions_json",
        if risk == "high" { "high" } else { "medium" },
    );
    let spine_event = runtime_spine::publish_event(
        "paradise_worldcell",
        "planning_goals",
        "session_planned",
        if risk == "high" { "high" } else { "medium" },
        serde_json::json!({
            "session_id": session_id_at(&store, index),
            "intent": intent,
            "candidate_command": session_candidate_at(&store, index),
            "risk": risk,
            "permission_key": permission_key
        }),
    );
    let spine_state = runtime_spine::record_state_mutation(
        "paradise_worldcell",
        "paradise_latest_plan",
        "write",
        "planned",
        serde_json::json!({
            "session_id": session_id_at(&store, index),
            "candidate_command": session_candidate_at(&store, index),
            "risk": risk
        }),
    );
    write_session_store(&store);
    format!(
        "[PARADISE-PLAN] id={} command=\"{}\" risk={} approval=pending path={}\n{}{}{}",
        session_id_at(&store, index),
        session_candidate_at(&store, index),
        risk,
        state_paths::paradise_worldcell_sessions_path(),
        evidence,
        spine_event,
        spine_state
    )
}

pub fn approve_session(spec: &str) -> String {
    let mut store = read_session_store();
    let Some(index) = resolve_session_index(&store, spec) else {
        return format!(
            "[PARADISE-APPROVE] status=rejected reason=session_not_found spec=\"{}\" path={}\n",
            compact(spec, 80),
            state_paths::paradise_worldcell_sessions_path()
        );
    };
    let candidate = session_candidate_at(&store, index);
    if candidate == "none" {
        return format!(
            "[PARADISE-APPROVE] id={} status=rejected reason=session_not_planned path={}\n",
            session_id_at(&store, index),
            state_paths::paradise_worldcell_sessions_path()
        );
    }
    let intent = store["sessions"][index]
        .get("intent")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unspecified intent")
        .to_string();
    let session = &mut store["sessions"][index];
    session["status"] = serde_json::json!("approved");
    session["current_stage"] = serde_json::json!("permission_gate_passed");
    session["approval"]["status"] = serde_json::json!("approved");
    session["approval"]["approved_by"] = serde_json::json!("operator");
    session["approval"]["approval_mode"] = serde_json::json!("explicit_once");
    session["worldcell_loop"] = serde_json::Value::Array(worldcell_loop_status("approve"));
    push_step(
        session,
        step_record(
            "permission_gate",
            "approved",
            "operator_explicit_once_approval_recorded",
            &candidate,
        ),
    );
    let evidence = action_evidence::record_attempt(
        "paradise_worldcell",
        &intent,
        "allowed",
        "approved",
        "operator_permission_gate_recorded",
        "paradise_worldcell_sessions_json",
        "medium",
    );
    let spine_event = runtime_spine::publish_event(
        "paradise_worldcell",
        "planning_goals",
        "session_approved",
        "medium",
        serde_json::json!({
            "session_id": session_id_at(&store, index),
            "candidate_command": candidate,
            "approval_mode": "explicit_once"
        }),
    );
    let spine_state = runtime_spine::record_state_mutation(
        "paradise_worldcell",
        "paradise_latest_approval",
        "write",
        "approved",
        serde_json::json!({
            "session_id": session_id_at(&store, index),
            "candidate_command": candidate
        }),
    );
    write_session_store(&store);
    format!(
        "[PARADISE-APPROVE] id={} status=approved scope=once path={}\n{}{}{}",
        session_id_at(&store, index),
        state_paths::paradise_worldcell_sessions_path(),
        evidence,
        spine_event,
        spine_state
    )
}

pub fn prepare_execution(spec: &str) -> ParadiseExecutionPreparation {
    let mut store = read_session_store();
    let Some(index) = resolve_session_index(&store, spec) else {
        return ParadiseExecutionPreparation::Blocked(format!(
            "[PARADISE-EXECUTE] status=rejected reason=session_not_found spec=\"{}\" path={}\n",
            compact(spec, 80),
            state_paths::paradise_worldcell_sessions_path()
        ));
    };
    let session_id = session_id_at(&store, index);
    let candidate = session_candidate_at(&store, index);
    if candidate == "none" {
        return ParadiseExecutionPreparation::Blocked(format!(
            "[PARADISE-EXECUTE] id={} status=rejected reason=session_not_planned path={}\n",
            session_id,
            state_paths::paradise_worldcell_sessions_path()
        ));
    }
    let approval_status = store["sessions"][index]
        .get("approval")
        .and_then(|approval| approval.get("status"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("not_requested")
        .to_string();
    if approval_status != "approved" {
        store["sessions"][index]["status"] = serde_json::json!("awaiting_approval");
        store["sessions"][index]["current_stage"] = serde_json::json!("permission_gate_pending");
        push_step(
            &mut store["sessions"][index],
            step_record(
                "permission_gate",
                "blocked",
                "execution_requires_explicit_paradise_approval",
                &approval_status,
            ),
        );
        write_session_store(&store);
        return ParadiseExecutionPreparation::Blocked(format!(
            "[PARADISE-EXECUTE] id={} status=blocked reason=approval_required path={}\n",
            session_id,
            state_paths::paradise_worldcell_sessions_path()
        ));
    }
    let dry_run = store["sessions"][index]
        .get("dry_run")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let standalone = dry_run
        .get("standalone_execution_allowed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let requires_supervision = dry_run
        .get("requires_supervision")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let mutates_runtime = dry_run
        .get("mutates_runtime")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    if !standalone || requires_supervision || mutates_runtime {
        store["sessions"][index]["status"] = serde_json::json!("blocked_by_safety_gate");
        store["sessions"][index]["current_stage"] = serde_json::json!("safety_permission_gate");
        store["sessions"][index]["execution_gate"] = serde_json::json!({
            "allowed": false,
            "reason": "candidate_command_not_standalone_safe",
            "standalone_execution_allowed": standalone,
            "requires_supervision": requires_supervision,
            "mutates_runtime": mutates_runtime,
        });
        push_step(
            &mut store["sessions"][index],
            step_record(
                "safety_permission_gate",
                "blocked",
                "candidate_command_requires_stronger_runtime_authority",
                &candidate,
            ),
        );
        write_session_store(&store);
        let evidence = action_evidence::record_attempt(
            "paradise_worldcell",
            &candidate,
            "blocked",
            "blocked",
            "safety_gate_prevented_non_standalone_action",
            "paradise_worldcell_sessions_json",
            "high",
        );
        let spine_event = runtime_spine::publish_event(
            "paradise_worldcell",
            "planning_goals",
            "session_execution_blocked",
            "high",
            serde_json::json!({
                "session_id": session_id,
                "candidate_command": candidate,
                "reason": "candidate_command_not_standalone_safe"
            }),
        );
        let spine_state = runtime_spine::record_state_mutation(
            "paradise_worldcell",
            "paradise_latest_execution_gate",
            "write",
            "blocked_by_safety_gate",
            serde_json::json!({
                "session_id": session_id,
                "candidate_command": candidate
            }),
        );
        return ParadiseExecutionPreparation::Blocked(format!(
            "[PARADISE-EXECUTE] id={} status=blocked reason=safety_gate command=\"{}\" path={}\n{}{}{}",
            session_id,
            candidate,
            state_paths::paradise_worldcell_sessions_path(),
            evidence,
            spine_event,
            spine_state
        ));
    }
    store["sessions"][index]["status"] = serde_json::json!("executing");
    store["sessions"][index]["current_stage"] = serde_json::json!("runtime_body_execution");
    store["sessions"][index]["execution_gate"] = serde_json::json!({
        "allowed": true,
        "reason": "approved_standalone_safe_action",
        "standalone_execution_allowed": standalone,
        "requires_supervision": requires_supervision,
        "mutates_runtime": mutates_runtime,
    });
    store["sessions"][index]["worldcell_loop"] =
        serde_json::Value::Array(worldcell_loop_status("execute"));
    push_step(
        &mut store["sessions"][index],
        step_record(
            "runtime_body_execution",
            "started",
            "safe_candidate_dispatched_to_gewc_runtime_body",
            &candidate,
        ),
    );
    let _ = runtime_spine::publish_event(
        "paradise_worldcell",
        "planning_goals",
        "session_execution_started",
        "low",
        serde_json::json!({
            "session_id": session_id,
            "candidate_command": candidate
        }),
    );
    write_session_store(&store);
    ParadiseExecutionPreparation::Ready(ParadiseExecutionRequest {
        session_id,
        raw_command: candidate,
    })
}

pub fn complete_execution(
    session_id: &str,
    raw_command: &str,
    result_summary: &str,
    execution_status: &str,
    consequence: &str,
    should_continue: bool,
) -> String {
    let mut store = read_session_store();
    let Some(index) = resolve_session_index(&store, session_id) else {
        return format!(
            "[PARADISE-EXECUTE] id={} status=rejected reason=session_not_found_after_execution path={}\n",
            session_id,
            state_paths::paradise_worldcell_sessions_path()
        );
    };
    let completed = execution_status == "completed";
    let final_status = if completed {
        "completed"
    } else {
        "blocked_by_runtime"
    };
    let evidence = action_evidence::record_attempt(
        "paradise_worldcell",
        raw_command,
        if completed { "allowed" } else { "blocked" },
        execution_status,
        consequence,
        "paradise_worldcell_sessions_json",
        if completed { "low" } else { "medium" },
    );
    let learning = learning_ledger::record(
        "paradise_worldcell",
        raw_command,
        "worldcell_execution_feedback",
        final_status,
    );
    let spine_event = runtime_spine::publish_event(
        "paradise_worldcell",
        "planning_goals",
        "session_execution_completed",
        if completed { "low" } else { "medium" },
        serde_json::json!({
            "session_id": session_id,
            "raw_command": raw_command,
            "status": final_status,
            "consequence": consequence
        }),
    );
    let spine_state = runtime_spine::record_state_mutation(
        "paradise_worldcell",
        "paradise_latest_execution",
        "write",
        final_status,
        serde_json::json!({
            "session_id": session_id,
            "raw_command": raw_command,
            "result_hash": format!("{:016x}", fnv64(result_summary.as_bytes()))
        }),
    );
    let session = &mut store["sessions"][index];
    session["status"] = serde_json::json!(final_status);
    session["current_stage"] = serde_json::json!("evidence_memory");
    session["worldcell_loop"] = serde_json::Value::Array(worldcell_loop_status("complete"));
    session["execution"] = serde_json::json!({
        "raw_command": raw_command,
        "status": execution_status,
        "consequence": consequence,
        "result_summary": compact(result_summary, 360),
        "result_hash": format!("{:016x}", fnv64(result_summary.as_bytes())),
        "should_continue": should_continue,
        "evidence": evidence.trim(),
        "learning": learning.trim(),
    });
    push_step(
        session,
        step_record(
            "evidence_memory",
            if completed { "completed" } else { "blocked" },
            consequence,
            &format!("{}{}", evidence, learning),
        ),
    );
    write_session_store(&store);
    format!(
        "[PARADISE-EXECUTE] id={} status={} command=\"{}\" path={}\n{}{}{}{}",
        session_id,
        final_status,
        raw_command,
        state_paths::paradise_worldcell_sessions_path(),
        evidence,
        learning,
        spine_event,
        spine_state
    )
}

pub fn audit_sessions() -> String {
    let store = read_session_store();
    let records = sessions(&store);
    let planned = count_status(records, "planned");
    let approved = count_status(records, "approved");
    let executing = count_status(records, "executing");
    let completed = count_status(records, "completed");
    let blocked = records
        .iter()
        .filter(|session| {
            session
                .get("status")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|status| status.contains("blocked"))
        })
        .count();
    format!(
        "[PARADISE-SESSIONS] sessions={} planned={} approved={} executing={} completed={} blocked={} latest={} path={}\n",
        records.len(),
        planned,
        approved,
        executing,
        completed,
        blocked,
        store
            .get("latest_session_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("none"),
        state_paths::paradise_worldcell_sessions_path()
    )
}

pub fn sessions_json() -> String {
    std::fs::read_to_string(state_paths::paradise_worldcell_sessions_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&empty_session_store()).unwrap_or_else(|_| "{}".to_string())
    })
}

fn checks(input: &ParadiseWorldcellInput) -> Vec<WorldcellCheck> {
    vec![
        WorldcellCheck {
            id: "membrane",
            component: "permissioned_boundary",
            purpose: "Actions enter a bounded local membrane before touching the real world.",
            passed: input.policy_report.contains("[POLICY]")
                && input.policy_report.contains("blocked=")
                && input.uncertainty_report.contains("[UNCERTAINTY]"),
            evidence: "policy_guard + uncertainty_ledger",
        },
        WorldcellCheck {
            id: "locus",
            component: "context_authority",
            purpose: "Context, authority and trust are bound before any action contract.",
            passed: input.locus_report.contains("[EDEN-LOCUS-LAYER]")
                && input.locus_report.contains("authority_parser=native")
                && input.locus_report.contains("permission_matrix=local"),
            evidence: "eden_locus_layer",
        },
        WorldcellCheck {
            id: "executive_workspace",
            component: "gewc_control",
            purpose: "The executive workspace remains the authority over routing and action.",
            passed: input
                .gewc_report
                .contains("core_authority=global_executive_workspace_core")
                && input.gewc_report.contains("handler_dispatch=domain_handler_dispatch"),
            evidence: "global_executive_workspace_runtime",
        },
        WorldcellCheck {
            id: "praxis_space",
            component: "cognitive_records",
            purpose: "Intent, state, evidence, constraints, affordances, projections and traces share one governed space.",
            passed: input.praxis_report.contains("[EDEN-PRAXIS-NEXUS]")
                && input.praxis_report.contains("primitives=7/7")
                && input.praxis_report.contains("blocks=5/5"),
            evidence: "eden_praxis_nexus",
        },
        WorldcellCheck {
            id: "forge",
            component: "action_contracts",
            purpose: "Potential actions become typed contracts before execution.",
            passed: input.forge_report.contains("[EDEN-OPERATOR-FORGE]")
                && input.forge_report.contains("verifier=bounded"),
            evidence: "eden_operator_forge",
        },
        WorldcellCheck {
            id: "runtime_body",
            component: "governed_execution",
            purpose: "Execution is local, replayable and contract-gated.",
            passed: input.operational_report.contains("[OPERATIONAL-RUNTIME-PHASE]")
                && input.operational_report.contains("action_executor=contract_gated")
                && input.operational_report.contains("replay=evaluable"),
            evidence: "operational_runtime_phase",
        },
        WorldcellCheck {
            id: "evidence_memory",
            component: "audit_and_learning",
            purpose: "Actions leave evidence, provenance and reviewable memory instead of hidden state changes.",
            passed: input.action_evidence_report.contains("[ACTION-EVIDENCE]")
                && input.provenance_report.contains("[PROVENANCE]"),
            evidence: "action_evidence + provenance_ledger",
        },
        WorldcellCheck {
            id: "model_subordination",
            component: "model_governance",
            purpose: "Models remain tools inside the Worldcell rather than the final authority.",
            passed: input.model_report.contains("[MODEL-RUNTIME]")
                && input.model_report.contains("authority=global_executive_workspace_core")
                && input.model_report.contains("claim_allowed=false"),
            evidence: "model_runtime",
        },
    ]
}

fn record(checks: &[WorldcellCheck], passed: usize, total: usize) -> serde_json::Value {
    let check_records: Vec<_> = checks
        .iter()
        .map(|check| {
            serde_json::json!({
                "id": check.id,
                "component": check.component,
                "purpose": check.purpose,
                "passed": check.passed,
                "evidence": check.evidence,
            })
        })
        .collect();

    serde_json::json!({
        "schema": PARADISE_WORLDCELL_SCHEMA,
        "artifact": "paradise_worldcell_runtime",
        "name": "Paradise",
        "category": "worldcell_runtime",
        "claim_allowed": false,
        "agi_claim": false,
        "tagline": "Give agents a worldcell before they touch the world.",
        "public_positioning": "Paradise is the public Worldcell Runtime identity for EDEN: a bounded cognitive world where autonomous intent becomes simulated, permissioned, auditable action.",
        "internal_engine": {
            "runtime": "GARM",
            "executive_core": "GEWC",
            "context_authority": "Locus",
            "action_contracts": "Operator Forge",
            "cognitive_learning": "ELCP",
        },
        "worldcell_loop": [
            "intent",
            "locus_context_authority",
            "cognitive_field",
            "gewc_executive_decision",
            "operator_forge_contract",
            "world_model_simulation",
            "safety_permission_gate",
            "runtime_body_execution",
            "evidence_memory",
            "safe_learning"
        ],
        "membrane": {
            "local_first": true,
            "dry_run_first": true,
            "permissioned_actions": true,
            "sandbox_required_for_risky_actions": true,
            "rollback_required": true,
            "external_claims_blocked": true
        },
        "passed": passed,
        "total": total,
        "checks": check_records,
    })
}

fn empty_session_store() -> serde_json::Value {
    serde_json::json!({
        "schema": PARADISE_WORLDCELL_SESSION_SCHEMA,
        "artifact": "paradise_worldcell_sessions",
        "name": "Paradise",
        "category": "worldcell_operational_loop",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "global_executive_workspace_core",
        "latest_session_id": null,
        "sessions": [],
        "loop_contract": [
            "intent",
            "locus_context_authority",
            "dry_run",
            "world_model_simulation",
            "operator_forge_contract",
            "permission_gate",
            "runtime_body_execution",
            "evidence_memory",
            "safe_learning"
        ],
        "write_policy": "sessions mutate only through GEWC-routed Paradise commands",
    })
}

fn read_session_store() -> serde_json::Value {
    std::fs::read_to_string(state_paths::paradise_worldcell_sessions_path())
        .ok()
        .and_then(|body| serde_json::from_str::<serde_json::Value>(&body).ok())
        .unwrap_or_else(empty_session_store)
}

fn write_session_store(store: &serde_json::Value) {
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        state_paths::paradise_worldcell_sessions_path(),
        serde_json::to_string_pretty(store).unwrap_or_else(|_| store.to_string()),
    );
}

fn ensure_sessions_array(store: &mut serde_json::Value) {
    if !store
        .get("sessions")
        .is_some_and(serde_json::Value::is_array)
    {
        store["sessions"] = serde_json::json!([]);
    }
}

fn sessions(store: &serde_json::Value) -> &[serde_json::Value] {
    store
        .get("sessions")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn resolve_session_index(store: &serde_json::Value, spec: &str) -> Option<usize> {
    let trimmed = spec.trim();
    let target = if trimmed.is_empty() || trimmed == "latest" || trimmed == "last" {
        store
            .get("latest_session_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
    } else {
        trimmed
    };
    sessions(store).iter().position(|session| {
        session
            .get("id")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|id| id == target || id.starts_with(target))
    })
}

fn session_id_at(store: &serde_json::Value, index: usize) -> String {
    store["sessions"][index]
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn session_candidate_at(store: &serde_json::Value, index: usize) -> String {
    store["sessions"][index]
        .get("candidate_command")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none")
        .to_string()
}

fn step_record(stage: &str, status: &str, evidence: &str, output: &str) -> serde_json::Value {
    serde_json::json!({
        "stage": stage,
        "status": status,
        "evidence": evidence,
        "summary": compact(output, 280),
        "fnv64": format!("{:016x}", fnv64(output.as_bytes())),
    })
}

fn push_step(session: &mut serde_json::Value, step: serde_json::Value) {
    if !session
        .get("steps")
        .is_some_and(serde_json::Value::is_array)
    {
        session["steps"] = serde_json::json!([]);
    }
    if let Some(steps) = session
        .get_mut("steps")
        .and_then(serde_json::Value::as_array_mut)
    {
        steps.push(step);
    }
}

fn worldcell_loop_status(stage: &str) -> Vec<serde_json::Value> {
    let order = [
        "intent",
        "locus_context_authority",
        "dry_run",
        "world_model_simulation",
        "operator_forge_contract",
        "permission_gate",
        "runtime_body_execution",
        "evidence_memory",
        "safe_learning",
    ];
    let active_index = match stage {
        "intent" => 1,
        "plan" => 4,
        "approve" => 5,
        "execute" => 6,
        "complete" => 8,
        _ => 0,
    };
    order
        .iter()
        .enumerate()
        .map(|(index, name)| {
            serde_json::json!({
                "stage": name,
                "status": if index <= active_index { "completed_or_active" } else { "pending" },
            })
        })
        .collect()
}

fn candidate_command_for(intent: &str) -> String {
    let normalized = intent.to_ascii_lowercase();
    if contains_any(
        &normalized,
        &["status", "state", "estado", "health", "snapshot"],
    ) {
        "status".to_string()
    } else if contains_any(
        &normalized,
        &["permission", "permissions", "permiso", "permisos"],
    ) {
        "operational permissions audit".to_string()
    } else if contains_any(
        &normalized,
        &["world", "mundo", "simulate", "simular", "causal"],
    ) {
        "world audit".to_string()
    } else if contains_any(&normalized, &["memory", "memoria", "remember", "recuerdo"]) {
        "memory".to_string()
    } else if contains_any(&normalized, &["api", "contract", "schema", "contrato"]) {
        "operational api eval".to_string()
    } else if contains_any(
        &normalized,
        &[
            "clean", "fix", "write", "modify", "delete", "push", "open pr", "train", "evolve",
            "mejor", "archivo", "repo",
        ],
    ) {
        "evolve".to_string()
    } else {
        "status".to_string()
    }
}

fn contains_any(text: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| text.contains(term))
}

fn count_status(records: &[serde_json::Value], status: &str) -> usize {
    records
        .iter()
        .filter(|session| {
            session
                .get("status")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value == status)
        })
        .count()
}

fn compact(text: &str, limit: usize) -> String {
    let compacted = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compacted.chars().count() > limit {
        compacted.chars().take(limit).collect()
    } else {
        compacted
    }
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
    fn writes_paradise_worldcell_runtime_artifact() {
        let _guard = state_paths::test_state_guard();
        let temp = std::env::temp_dir().join(format!(
            "eden_paradise_worldcell_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        state_paths::set_state_dir(temp.clone());
        let report = run(ParadiseWorldcellInput {
            gewc_report: "[GEWC-RUNTIME] core_authority=global_executive_workspace_core handler_dispatch=domain_handler_dispatch".to_string(),
            praxis_report: "[EDEN-PRAXIS-NEXUS] primitives=7/7 blocks=5/5".to_string(),
            locus_report: "[EDEN-LOCUS-LAYER] authority_parser=native permission_matrix=local".to_string(),
            forge_report: "[EDEN-OPERATOR-FORGE] verifier=bounded".to_string(),
            operational_report: "[OPERATIONAL-RUNTIME-PHASE] action_executor=contract_gated replay=evaluable".to_string(),
            model_report: "[MODEL-RUNTIME] authority=global_executive_workspace_core claim_allowed=false".to_string(),
            policy_report: "[POLICY] blocked=1".to_string(),
            provenance_report: "[PROVENANCE]".to_string(),
            uncertainty_report: "[UNCERTAINTY]".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE]".to_string(),
        });

        assert!(report.contains("[PARADISE-WORLDCELL-RUNTIME] passed=8/8"));
        let body = std::fs::read_to_string(state_paths::paradise_worldcell_runtime_path())
            .expect("artifact");
        assert!(body.contains("\"name\": \"Paradise\""));
        assert!(body.contains("\"claim_allowed\": false"));
        let _ = std::fs::remove_dir_all(temp);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn paradise_session_loop_records_approval_and_completion() {
        let _guard = state_paths::test_state_guard();
        let temp = std::env::temp_dir().join(format!(
            "eden_paradise_session_loop_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        state_paths::set_state_dir(temp.clone());

        let intent = record_intent("inspect runtime status safely");
        assert!(intent.contains("[PARADISE-INTENT]"));
        let plan = plan_session("latest");
        assert!(plan.contains("command=\"status\""));
        let approval = approve_session("latest");
        assert!(approval.contains("status=approved"));
        let request = match prepare_execution("latest") {
            ParadiseExecutionPreparation::Ready(request) => request,
            ParadiseExecutionPreparation::Blocked(output) => panic!("{output}"),
        };
        assert_eq!(request.raw_command, "status");
        let completed = complete_execution(
            &request.session_id,
            &request.raw_command,
            "[STATUS] local runtime inspected",
            "completed",
            "safe_read_completed",
            true,
        );
        assert!(completed.contains("status=completed"));
        let body = sessions_json();
        assert!(body.contains("\"schema\": \"eden-paradise-worldcell-session-v1\""));
        assert!(body.contains("\"status\": \"completed\""));
        assert!(audit_sessions().contains("completed=1"));

        let _ = std::fs::remove_dir_all(temp);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn paradise_execution_blocks_risky_candidate_even_after_approval() {
        let _guard = state_paths::test_state_guard();
        let temp = std::env::temp_dir().join(format!(
            "eden_paradise_session_block_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp);
        state_paths::set_state_dir(temp.clone());

        let _ = record_intent("fix repo and push a change");
        let plan = plan_session("latest");
        assert!(plan.contains("command=\"evolve\""));
        let _ = approve_session("latest");
        let blocked = match prepare_execution("latest") {
            ParadiseExecutionPreparation::Ready(_) => panic!("risky action should stay blocked"),
            ParadiseExecutionPreparation::Blocked(output) => output,
        };
        assert!(blocked.contains("reason=safety_gate"));
        assert!(sessions_json().contains("\"status\": \"blocked_by_safety_gate\""));

        let _ = std::fs::remove_dir_all(temp);
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
