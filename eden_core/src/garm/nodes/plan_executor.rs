use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_PLANS: usize = 128;

#[derive(Clone, Debug, PartialEq)]
struct ExecutionPlan {
    id: u64,
    query: String,
    status: String,
    score: f32,
    steps: Vec<String>,
    rollback_note: String,
}

#[derive(Clone, Debug)]
struct PlanExecutorState {
    plans: VecDeque<ExecutionPlan>,
    next_id: u64,
    completed: u64,
    rolled_back: u64,
}

impl Default for PlanExecutorState {
    fn default() -> Self {
        Self {
            plans: VecDeque::new(),
            next_id: 1,
            completed: 0,
            rolled_back: 0,
        }
    }
}

static PLAN_EXECUTOR_STATE: OnceLock<Mutex<PlanExecutorState>> = OnceLock::new();

fn executor_state() -> &'static Mutex<PlanExecutorState> {
    PLAN_EXECUTOR_STATE.get_or_init(|| Mutex::new(PlanExecutorState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = executor_state().lock() {
        *state = PlanExecutorState::default();
    }
}

pub fn plan(query: &str) -> String {
    let trimmed = query.trim();
    let mut state = executor_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    let steps = vec![
        format!("collect_context:{}", bounded_fragment(trimmed, 48)),
        "check_organs:safe_local_only".to_string(),
        "score_evidence:reports_only".to_string(),
        "commit_or_rollback:state_log_only".to_string(),
    ];
    push_plan(
        &mut state,
        ExecutionPlan {
            id,
            query: trimmed.to_string(),
            status: "planned".to_string(),
            score: 0.0,
            steps: steps.clone(),
            rollback_note: String::new(),
        },
    );
    format!(
        "[EXEC-PLAN] id={} status=planned steps={} query={}\n{}",
        id,
        steps.len(),
        trimmed,
        report_locked(&state)
    )
}

pub fn run_next(local_evidence: &str) -> String {
    let mut state = executor_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(pos) = state.plans.iter().position(|plan| plan.status == "planned") else {
        return format!(
            "[EXEC-RUN] status=no_planned_plan\n{}",
            report_locked(&state)
        );
    };
    let score = score_evidence(local_evidence);
    let plan_id = state.plans[pos].id;
    let status = if score >= 0.50 {
        state.completed += 1;
        "completed"
    } else {
        state.rolled_back += 1;
        "rolled_back"
    };
    state.plans[pos].score = score;
    state.plans[pos].status = status.to_string();
    if status == "rolled_back" {
        state.plans[pos].rollback_note =
            "insufficient local evidence; no external side effects".to_string();
    }
    format!(
        "[EXEC-RUN] id={} status={} score={:.1}% rollback={}\n{}",
        plan_id,
        status,
        score * 100.0,
        state.plans[pos].rollback_note,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = executor_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = executor_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for plan in state.plans.iter().rev().take(8) {
        out.push_str(&format!(
            "- plan={} status={} score={:.3} query={} rollback={} steps={}\n",
            plan.id,
            plan.status,
            plan.score,
            plan.query,
            plan.rollback_note,
            plan.steps.join(",")
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = executor_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let plans: Vec<_> = state
        .plans
        .iter()
        .map(|plan| {
            serde_json::json!({
                "id": plan.id,
                "query": plan.query,
                "status": plan.status,
                "score": plan.score,
                "steps": plan.steps,
                "rollback_note": plan.rollback_note,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "plan-executor-v1",
        "next_id": state.next_id,
        "completed": state.completed,
        "rolled_back": state.rolled_back,
        "plans": plans,
    });
    std::fs::write(
        state_paths::plan_executor_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write plan executor state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::plan_executor_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read plan executor state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse plan executor JSON: {}", e))?;
    let mut state = PlanExecutorState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.completed = snapshot
        .get("completed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.rolled_back = snapshot
        .get("rolled_back")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(plans) = snapshot.get("plans").and_then(|v| v.as_array()) {
        for plan in plans {
            push_plan(
                &mut state,
                ExecutionPlan {
                    id: plan.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    query: json_string(plan, "query"),
                    status: json_string(plan, "status"),
                    score: plan.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                    steps: plan
                        .get("steps")
                        .and_then(|v| v.as_array())
                        .map(|steps| {
                            steps
                                .iter()
                                .filter_map(|v| v.as_str().map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default(),
                    rollback_note: json_string(plan, "rollback_note"),
                },
            );
        }
    }
    *executor_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &PlanExecutorState) -> String {
    let planned = state
        .plans
        .iter()
        .filter(|plan| plan.status == "planned")
        .count();
    format!(
        "[EXEC] schema=plan-executor-v1 plans={} planned={} completed={} rolled_back={}\n",
        state.plans.len(),
        planned,
        state.completed,
        state.rolled_back
    )
}

fn score_evidence(local_evidence: &str) -> f32 {
    let mut score: f32 = 0.0;
    if local_evidence.contains("[CAG]") || local_evidence.contains("[CAG-REPORT]") {
        score += 0.25;
    }
    if local_evidence.contains("total=32") && local_evidence.contains("missing=0") {
        score += 0.25;
    }
    if local_evidence.contains("[GOALS]") || local_evidence.contains("goals=") {
        score += 0.20;
    }
    if local_evidence.contains("[BENCH]") || local_evidence.contains("[EVAL]") {
        score += 0.20;
    }
    if local_evidence.contains("[WORLD]") || local_evidence.contains("[LEARNING]") {
        score += 0.10;
    }
    score.min(1.0)
}

fn push_plan(state: &mut PlanExecutorState, plan: ExecutionPlan) {
    state.plans.push_back(plan);
    while state.plans.len() > MAX_PLANS {
        state.plans.pop_front();
    }
}

fn bounded_fragment(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
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
    fn plans_scores_and_completes_with_local_evidence() {
        reset_for_tests();

        let planned = plan("mejorar evaluacion local");
        let run = run_next("[CAG] total=32 missing=0 [GOALS] [BENCH] [WORLD]");

        assert!(planned.contains("[EXEC-PLAN] id=1"));
        assert!(run.contains("status=completed"));
        assert!(run.contains("score=100.0%"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_plan_executor_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_plan_executor_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = plan("persistir plan local");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("plans=1"));
        assert!(report.contains("planned=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
