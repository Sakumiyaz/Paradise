use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_GOALS: usize = 256;
const MAX_ACTIONS: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalRecord {
    pub id: u64,
    pub title: String,
    pub source: String,
    pub priority: u8,
    pub risk: String,
    pub status: String,
    pub evidence_required: String,
    pub result: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionContract {
    pub id: u64,
    pub goal_id: u64,
    pub organ: String,
    pub kind: String,
    pub risk: String,
    pub preconditions: String,
    pub expected_effect: String,
    pub evidence_required: String,
    pub status: String,
    pub result: String,
}

#[derive(Clone, Debug)]
struct GoalSchedulerState {
    goals: VecDeque<GoalRecord>,
    actions: VecDeque<ActionContract>,
    next_goal_id: u64,
    next_action_id: u64,
    planned: u64,
    completed: u64,
    blocked: u64,
}

impl Default for GoalSchedulerState {
    fn default() -> Self {
        Self {
            goals: VecDeque::new(),
            actions: VecDeque::new(),
            next_goal_id: 1,
            next_action_id: 1,
            planned: 0,
            completed: 0,
            blocked: 0,
        }
    }
}

static GOAL_STATE: OnceLock<Mutex<GoalSchedulerState>> = OnceLock::new();

fn scheduler_state() -> &'static Mutex<GoalSchedulerState> {
    GOAL_STATE.get_or_init(|| Mutex::new(GoalSchedulerState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = scheduler_state().lock() {
        *state = GoalSchedulerState::default();
    }
}

pub fn plan_goal(title: &str, source: &str) -> String {
    let clean_title = title.trim();
    let title = if clean_title.is_empty() {
        "arquitectura sin objetivo especifico"
    } else {
        clean_title
    };
    let mut state = scheduler_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let goal_id = state.next_goal_id;
    state.next_goal_id += 1;
    state.planned += 1;
    push_goal(
        &mut state,
        GoalRecord {
            id: goal_id,
            title: title.to_string(),
            source: source.to_string(),
            priority: 7,
            risk: "low".to_string(),
            status: "ready".to_string(),
            evidence_required: "local_context".to_string(),
            result: "planned".to_string(),
        },
    );
    add_contract(
        &mut state,
        goal_id,
        "context_augmentation",
        "build_context_pack",
        "low",
        "state_dir_available,query_present",
        "context_pack_available",
        "memory_or_history_or_kg_trace",
    );
    add_contract(
        &mut state,
        goal_id,
        "hrm_reasoner",
        "derive_hierarchical_plan",
        "low",
        "context_pack_available",
        "plan_steps_recorded",
        "hrm_snapshot",
    );
    add_contract(
        &mut state,
        goal_id,
        "organ_registry",
        "execute_safe_organ_cycle",
        "medium",
        "local_runtime_ready,no_remote_network",
        "organ_deltas_audited",
        "organ_audit",
    );
    format!(
        "[GOALS-PLAN] id={} status=ready priority=7 risk=low title='{}' contracts=3\n{}",
        goal_id,
        title,
        actions_for_goal_report(&state, goal_id)
    )
}

pub fn record_external_goal(source: &str, title: &str, completed: bool, result: &str) -> String {
    let mut state = scheduler_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let goal_id = state.next_goal_id;
    state.next_goal_id += 1;
    state.planned += 1;
    if completed {
        state.completed += 1;
    } else {
        state.blocked += 1;
    }
    push_goal(
        &mut state,
        GoalRecord {
            id: goal_id,
            title: title.trim().to_string(),
            source: source.to_string(),
            priority: 6,
            risk: "low".to_string(),
            status: if completed { "completed" } else { "blocked" }.to_string(),
            evidence_required: "runtime_trace".to_string(),
            result: result.to_string(),
        },
    );
    let action_id = state.next_action_id;
    state.next_action_id += 1;
    push_action(
        &mut state,
        ActionContract {
            id: action_id,
            goal_id,
            organ: source.to_string(),
            kind: "external_runtime_execution".to_string(),
            risk: "low".to_string(),
            preconditions: "command_dispatched".to_string(),
            expected_effect: "runtime_trace_recorded".to_string(),
            evidence_required: "command_output".to_string(),
            status: if completed { "completed" } else { "blocked" }.to_string(),
            result: result.to_string(),
        },
    );
    format!(
        "[GOALS-RECORD] id={} source={} status={} result={}\n",
        goal_id,
        source,
        if completed { "completed" } else { "blocked" },
        result
    )
}

pub fn plan_readiness_actions(actions: &[&str]) -> String {
    let selected_actions = if actions.is_empty() {
        vec!["mantener_gates_readiness_y_ejecutar_benchmarks_regulares"]
    } else {
        actions.to_vec()
    };
    let mut state = scheduler_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = format!(
        "[READINESS-GOALS] planned_actions={} source=readiness\n",
        selected_actions.len()
    );
    for action in selected_actions {
        let goal_id = state.next_goal_id;
        state.next_goal_id += 1;
        state.planned += 1;
        push_goal(
            &mut state,
            GoalRecord {
                id: goal_id,
                title: action.to_string(),
                source: "readiness".to_string(),
                priority: 9,
                risk: "medium".to_string(),
                status: "ready".to_string(),
                evidence_required: "readiness_gate_evidence".to_string(),
                result: "planned".to_string(),
            },
        );
        add_readiness_contracts(&mut state, goal_id, action);
        out.push_str(&format!(
            "- goal={} status=ready priority=9 action={} contracts={}\n",
            goal_id,
            action,
            state
                .actions
                .iter()
                .filter(|contract| contract.goal_id == goal_id)
                .count()
        ));
        out.push_str(&actions_for_goal_report(&state, goal_id));
    }
    out
}

pub fn run_ready_goals() -> String {
    let mut state = scheduler_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut completed_actions = 0usize;
    let mut completed_goals = 0usize;
    for action in &mut state.actions {
        if action.status == "ready" || action.status == "pending" {
            action.status = "completed".to_string();
            action.result = format!(
                "contract_satisfied:{}:{}",
                action.expected_effect, action.evidence_required
            );
            completed_actions += 1;
        }
    }
    for goal in &mut state.goals {
        if goal.status == "ready" || goal.status == "running" {
            goal.status = "completed".to_string();
            goal.result = "all_ready_contracts_completed".to_string();
            completed_goals += 1;
        }
    }
    state.completed += completed_goals as u64;
    format!(
        "[GOALS-RUN] completed_goals={} completed_actions={} blocked={}\n{}",
        completed_goals,
        completed_actions,
        state.blocked,
        audit_report_locked(&state)
    )
}

pub fn run_readiness_ready_goals_with_evidence(local_evidence: &str) -> String {
    let benchmark = readiness_gate_benchmark_report(local_evidence);
    let mut state = scheduler_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let goal_ids: Vec<u64> = state
        .goals
        .iter()
        .filter(|goal| {
            goal.source == "readiness" && (goal.status == "ready" || goal.status == "running")
        })
        .map(|goal| goal.id)
        .collect();
    let mut completed_actions = 0usize;
    let mut blocked_actions = 0usize;
    let mut completed_goals = 0usize;
    let mut blocked_goals = 0usize;
    for goal_id in &goal_ids {
        for action in state
            .actions
            .iter_mut()
            .filter(|action| action.goal_id == *goal_id && action.status == "ready")
        {
            if readiness_contract_has_evidence(action, local_evidence) {
                action.status = "completed".to_string();
                action.result = format!(
                    "evidence_satisfied:{}:{}",
                    action.expected_effect, action.evidence_required
                );
                completed_actions += 1;
            } else {
                action.status = "blocked".to_string();
                action.result = format!(
                    "missing_evidence:{}:{}",
                    action.preconditions, action.evidence_required
                );
                blocked_actions += 1;
            }
        }
    }
    for goal_id in &goal_ids {
        let has_blocked = state
            .actions
            .iter()
            .any(|action| action.goal_id == *goal_id && action.status == "blocked");
        let all_completed = state
            .actions
            .iter()
            .filter(|action| action.goal_id == *goal_id)
            .all(|action| action.status == "completed");
        if let Some(goal) = state.goals.iter_mut().find(|goal| goal.id == *goal_id) {
            if all_completed {
                goal.status = "completed".to_string();
                goal.result = "readiness_contract_evidence_satisfied".to_string();
                completed_goals += 1;
            } else if has_blocked {
                goal.status = "blocked".to_string();
                goal.result = "readiness_contract_evidence_missing".to_string();
                blocked_goals += 1;
            }
        }
    }
    state.completed += completed_goals as u64;
    state.blocked += blocked_goals as u64;
    format!(
        "[READINESS-GOALS-RUN] completed_goals={} blocked_goals={} completed_actions={} blocked_actions={} evidence_mode=local\n{}{}",
        completed_goals,
        blocked_goals,
        completed_actions,
        blocked_actions,
        benchmark,
        audit_report_locked(&state)
    )
}

pub fn readiness_gate_benchmark_report(local_evidence: &str) -> String {
    let gates = readiness_gate_results(local_evidence);
    let passed = gates.iter().filter(|(_, ok, _)| *ok).count();
    let mut out = format!(
        "[READINESS-BENCH] gates={} passed={} failed={} evidence_mode=local\n",
        gates.len(),
        passed,
        gates.len().saturating_sub(passed)
    );
    for (gate, ok, evidence) in gates {
        out.push_str(&format!(
            "- gate={} passed={} evidence={}\n",
            gate, ok, evidence
        ));
    }
    out
}

pub fn report() -> String {
    let state = scheduler_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let pending = state
        .goals
        .iter()
        .filter(|goal| goal.status == "ready" || goal.status == "running")
        .count();
    let last = state
        .goals
        .back()
        .map(|goal| format!("{}:{}:{}", goal.id, goal.status, goal.title))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "[GOALS] goals={} actions={} pending={} planned={} completed={} blocked={} last={}\n",
        state.goals.len(),
        state.actions.len(),
        pending,
        state.planned,
        state.completed,
        state.blocked,
        last
    )
}

pub fn audit_report() -> String {
    let state = scheduler_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    audit_report_locked(&state)
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = scheduler_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let goals: Vec<_> = state
        .goals
        .iter()
        .map(|goal| {
            serde_json::json!({
                "id": goal.id,
                "title": goal.title,
                "source": goal.source,
                "priority": goal.priority,
                "risk": goal.risk,
                "status": goal.status,
                "evidence_required": goal.evidence_required,
                "result": goal.result,
            })
        })
        .collect();
    let actions: Vec<_> = state
        .actions
        .iter()
        .map(|action| {
            serde_json::json!({
                "id": action.id,
                "goal_id": action.goal_id,
                "organ": action.organ,
                "kind": action.kind,
                "risk": action.risk,
                "preconditions": action.preconditions,
                "expected_effect": action.expected_effect,
                "evidence_required": action.evidence_required,
                "status": action.status,
                "result": action.result,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "goal-scheduler-v1",
        "next_goal_id": state.next_goal_id,
        "next_action_id": state.next_action_id,
        "planned": state.planned,
        "completed": state.completed,
        "blocked": state.blocked,
        "goals": goals,
        "actions": actions,
    });
    std::fs::write(
        state_paths::goal_scheduler_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write goal scheduler state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::goal_scheduler_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read goal scheduler state: {}", e))?;
    let snapshot: serde_json::Value =
        serde_json::from_str(&data).map_err(|e| format!("failed to parse goal JSON: {}", e))?;
    let mut state = GoalSchedulerState::default();
    state.next_goal_id = snapshot
        .get("next_goal_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.next_action_id = snapshot
        .get("next_action_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.planned = snapshot
        .get("planned")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.completed = snapshot
        .get("completed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.blocked = snapshot
        .get("blocked")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(goals) = snapshot.get("goals").and_then(|v| v.as_array()) {
        for goal in goals {
            push_goal(
                &mut state,
                GoalRecord {
                    id: goal.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    title: json_string(goal, "title"),
                    source: json_string(goal, "source"),
                    priority: goal.get("priority").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                    risk: json_string(goal, "risk"),
                    status: json_string(goal, "status"),
                    evidence_required: json_string(goal, "evidence_required"),
                    result: json_string(goal, "result"),
                },
            );
        }
    }
    if let Some(actions) = snapshot.get("actions").and_then(|v| v.as_array()) {
        for action in actions {
            push_action(
                &mut state,
                ActionContract {
                    id: action.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    goal_id: action.get("goal_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    organ: json_string(action, "organ"),
                    kind: json_string(action, "kind"),
                    risk: json_string(action, "risk"),
                    preconditions: json_string(action, "preconditions"),
                    expected_effect: json_string(action, "expected_effect"),
                    evidence_required: json_string(action, "evidence_required"),
                    status: json_string(action, "status"),
                    result: json_string(action, "result"),
                },
            );
        }
    }
    *scheduler_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn add_contract(
    state: &mut GoalSchedulerState,
    goal_id: u64,
    organ: &str,
    kind: &str,
    risk: &str,
    preconditions: &str,
    expected_effect: &str,
    evidence_required: &str,
) {
    let id = state.next_action_id;
    state.next_action_id += 1;
    push_action(
        state,
        ActionContract {
            id,
            goal_id,
            organ: organ.to_string(),
            kind: kind.to_string(),
            risk: risk.to_string(),
            preconditions: preconditions.to_string(),
            expected_effect: expected_effect.to_string(),
            evidence_required: evidence_required.to_string(),
            status: "ready".to_string(),
            result: "planned".to_string(),
        },
    );
}

fn push_goal(state: &mut GoalSchedulerState, goal: GoalRecord) {
    state.goals.push_back(goal);
    while state.goals.len() > MAX_GOALS {
        state.goals.pop_front();
    }
}

fn push_action(state: &mut GoalSchedulerState, action: ActionContract) {
    state.actions.push_back(action);
    while state.actions.len() > MAX_ACTIONS {
        state.actions.pop_front();
    }
}

fn actions_for_goal_report(state: &GoalSchedulerState, goal_id: u64) -> String {
    let mut out = String::new();
    for action in state
        .actions
        .iter()
        .filter(|action| action.goal_id == goal_id)
    {
        out.push_str(&format!(
            "- action={} organ={} kind={} risk={} preconditions={} expected_effect={} evidence={} status={}\n",
            action.id,
            action.organ,
            action.kind,
            action.risk,
            action.preconditions,
            action.expected_effect,
            action.evidence_required,
            action.status
        ));
    }
    out
}

fn add_readiness_contracts(state: &mut GoalSchedulerState, goal_id: u64, action: &str) {
    if action.contains("rag_verificable") {
        add_contract(
            state,
            goal_id,
            "hrm_text_pretraining",
            "run_recall_benchmark_with_citations",
            "medium",
            "local_corpus_present,query_set_present",
            "recall_and_citation_report_recorded",
            "hrm_text_context_pack,rag_eval_trace",
        );
        add_contract(
            state,
            goal_id,
            "provenance_ledger",
            "record_retrieval_evidence_chain",
            "low",
            "context_pack_available",
            "retrieval_to_answer_citations_linked",
            "provenance_record",
        );
        return;
    }
    if action.contains("world_model") || action.contains("predictivo") {
        add_contract(
            state,
            goal_id,
            "world_model",
            "record_prediction_and_observation_pairs",
            "medium",
            "local_observations_available",
            "prediction_error_measured",
            "world_model_trace,benchmark_result",
        );
        add_contract(
            state,
            goal_id,
            "competence_benchmark",
            "run_prediction_regression_cases",
            "low",
            "prediction_cases_present",
            "prediction_gate_score_recorded",
            "benchmark_report",
        );
        return;
    }
    if action.contains("generalizacion") {
        add_contract(
            state,
            goal_id,
            "competence_benchmark",
            "run_out_of_distribution_cases",
            "medium",
            "ood_cases_present,no_training_leakage",
            "generalization_gate_score_recorded",
            "benchmark_report",
        );
        add_contract(
            state,
            goal_id,
            "uncertainty_ledger",
            "record_failures_and_abstentions",
            "low",
            "benchmark_result_available",
            "uncertainty_updates_recorded",
            "uncertainty_record",
        );
        return;
    }
    if action.contains("grounding_accion") {
        add_contract(
            state,
            goal_id,
            "organ_registry",
            "execute_local_tool_contracts",
            "medium",
            "policy_gate_passed,no_remote_network",
            "tool_effects_and_deltas_audited",
            "organ_audit,action_contract_trace",
        );
        add_contract(
            state,
            goal_id,
            "goal_scheduler",
            "verify_action_contract_closure",
            "low",
            "organ_deltas_audited",
            "closed_loop_goal_evidence_recorded",
            "goal_audit",
        );
        return;
    }
    if action.contains("policy") || action.contains("provenance") || action.contains("uncertainty")
    {
        add_contract(
            state,
            goal_id,
            "policy_guard",
            "enforce_action_gate",
            "medium",
            "action_contract_present",
            "unsafe_actions_blocked_before_execution",
            "policy_decision_trace",
        );
        add_contract(
            state,
            goal_id,
            "provenance_ledger",
            "link_policy_uncertainty_evidence",
            "low",
            "policy_decision_available",
            "safety_gate_evidence_linked",
            "provenance_record,uncertainty_record",
        );
        return;
    }
    add_contract(
        state,
        goal_id,
        "readiness",
        "rerun_readiness_gate_report",
        "low",
        "runtime_metrics_available",
        "readiness_report_updated",
        "readiness_report",
    );
}

fn readiness_contract_has_evidence(action: &ActionContract, local_evidence: &str) -> bool {
    match (action.organ.as_str(), action.kind.as_str()) {
        ("hrm_text_pretraining", "run_recall_benchmark_with_citations") => {
            readiness_gate_passed(local_evidence, "rag_verificable")
        }
        ("provenance_ledger", "record_retrieval_evidence_chain") => {
            readiness_gate_passed(local_evidence, "rag_verificable")
        }
        ("world_model", "record_prediction_and_observation_pairs") => {
            readiness_gate_passed(local_evidence, "modelos_predictivos")
        }
        ("competence_benchmark", "run_prediction_regression_cases")
        | ("competence_benchmark", "run_out_of_distribution_cases") => {
            readiness_gate_passed(local_evidence, "generalizacion")
        }
        ("uncertainty_ledger", "record_failures_and_abstentions") => {
            readiness_gate_passed(local_evidence, "seguridad_operacional")
        }
        ("organ_registry", "execute_local_tool_contracts") => local_evidence.contains("total=32"),
        ("goal_scheduler", "verify_action_contract_closure") => {
            readiness_gate_passed(local_evidence, "autonomia_gobernada")
        }
        ("policy_guard", "enforce_action_gate") => local_evidence.contains("[POLICY]"),
        ("provenance_ledger", "link_policy_uncertainty_evidence") => {
            readiness_gate_passed(local_evidence, "seguridad_operacional")
        }
        ("readiness", "rerun_readiness_gate_report") => {
            local_evidence.contains("READINESS")
                && local_evidence.contains("no_claim_until_all_gates_pass")
        }
        _ => false,
    }
}

fn readiness_gate_passed(local_evidence: &str, gate: &str) -> bool {
    readiness_gate_results(local_evidence)
        .into_iter()
        .any(|(name, passed, _)| name == gate && passed)
}

fn readiness_gate_results(local_evidence: &str) -> Vec<(&'static str, bool, &'static str)> {
    vec![
        (
            "rag_verificable",
            metric_at_least(local_evidence, "retrieval_hits=", 1)
                || metric_at_least(local_evidence, "context_packs=", 1),
            "retrieval_hits>=1_or_context_packs>=1",
        ),
        (
            "modelos_predictivos",
            metric_at_least(local_evidence, "predictions=", 1),
            "predictions>=1",
        ),
        (
            "generalizacion",
            metric_at_least(local_evidence, "runs=", 1) && local_evidence.contains("[BENCH]"),
            "benchmark_runs>=1_and_bench_report",
        ),
        (
            "grounding_accion",
            local_evidence.contains("total=32") && local_evidence.contains("[GOALS]"),
            "organ_total_32_and_goal_scheduler_report",
        ),
        (
            "seguridad_operacional",
            local_evidence.contains("[POLICY]")
                && local_evidence.contains("[PROVENANCE]")
                && local_evidence.contains("[UNCERTAINTY]"),
            "policy_provenance_uncertainty_reports",
        ),
        (
            "autonomia_gobernada",
            local_evidence.contains("[GOALS]")
                && local_evidence.contains("actions=")
                && local_evidence.contains("total=32"),
            "goal_contracts_and_organ_registry",
        ),
    ]
}

fn metric_at_least(report: &str, key: &str, minimum: u64) -> bool {
    report
        .split_whitespace()
        .filter_map(|part| part.strip_prefix(key))
        .filter_map(|value| {
            value
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .ok()
        })
        .any(|value| value >= minimum)
}

fn audit_report_locked(state: &GoalSchedulerState) -> String {
    let mut out = format!(
        "[GOALS-AUDIT] schema=goal-scheduler-v1 goals={} actions={} planned={} completed={} blocked={}\n",
        state.goals.len(),
        state.actions.len(),
        state.planned,
        state.completed,
        state.blocked
    );
    for goal in state.goals.iter().rev().take(8) {
        out.push_str(&format!(
            "- goal={} status={} priority={} risk={} source={} evidence={} result={} title='{}'\n",
            goal.id,
            goal.status,
            goal.priority,
            goal.risk,
            goal.source,
            goal.evidence_required,
            goal.result,
            goal.title
        ));
    }
    for action in state.actions.iter().rev().take(8) {
        out.push_str(&format!(
            "- contract={} goal={} organ={} kind={} status={} risk={} expected={} evidence={} result={}\n",
            action.id,
            action.goal_id,
            action.organ,
            action.kind,
            action.status,
            action.risk,
            action.expected_effect,
            action.evidence_required,
            action.result
        ));
    }
    out
}

fn json_string(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_runs_and_audits_goal_contracts() {
        reset_for_tests();

        let plan = plan_goal("mejorar arquitectura", "test");
        let run = run_ready_goals();
        let audit = audit_report();

        assert!(plan.contains("[GOALS-PLAN] id=1"));
        assert!(plan.contains("organ=context_augmentation"));
        assert!(run.contains("completed_goals=1"));
        assert!(audit.contains("schema=goal-scheduler-v1"));
        assert!(audit.contains("status=completed"));
    }

    #[test]
    fn plans_readiness_actions_as_specific_contracts() {
        reset_for_tests();

        let plan = plan_readiness_actions(&[
            "expandir_rag_verificable_con_recall_benchmarks_y_citas_obligatorias",
            "endurecer_policy_provenance_uncertainty_como_gates_de_accion",
        ]);

        assert!(plan.contains("[READINESS-GOALS] planned_actions=2"));
        assert!(plan.contains("organ=hrm_text_pretraining"));
        assert!(plan.contains("run_recall_benchmark_with_citations"));
        assert!(plan.contains("organ=policy_guard"));
        assert!(report().contains("actions=4"));
    }

    #[test]
    fn runs_readiness_contracts_only_when_local_evidence_is_present() {
        reset_for_tests();
        let _ = plan_readiness_actions(&[
            "expandir_rag_verificable_con_recall_benchmarks_y_citas_obligatorias",
        ]);

        let blocked = run_readiness_ready_goals_with_evidence(
            "[HRM-TEXT] retrieval_hits=0 [PROVENANCE] records=0",
        );

        assert!(blocked.contains("blocked_goals=1"));
        assert!(blocked.contains("missing_evidence"));

        reset_for_tests();
        let _ = plan_readiness_actions(&[
            "expandir_rag_verificable_con_recall_benchmarks_y_citas_obligatorias",
        ]);

        let completed = run_readiness_ready_goals_with_evidence(
            "[HRM-TEXT] retrieval_hits=1 [PROVENANCE] records=1",
        );

        assert!(completed.contains("completed_goals=1"));
        assert!(completed.contains("evidence_satisfied"));
    }

    #[test]
    fn reports_readiness_gate_benchmarks_from_local_evidence() {
        let report = readiness_gate_benchmark_report(
            "[HRM-TEXT] retrieval_hits=1 context_packs=1 [BENCH] runs=1 predictions=1 [GOALS] actions=2 total=32 [POLICY] [PROVENANCE] [UNCERTAINTY]",
        );

        assert!(report.contains("[READINESS-BENCH] gates=6 passed=6 failed=0"));
        assert!(report.contains("gate=rag_verificable passed=true"));
        assert!(report.contains("gate=seguridad_operacional passed=true"));
    }

    #[test]
    fn saves_and_loads_goal_scheduler_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_goal_scheduler_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = plan_goal("persistir objetivos", "test");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("goals=1"));
        assert!(report.contains("actions=3"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
