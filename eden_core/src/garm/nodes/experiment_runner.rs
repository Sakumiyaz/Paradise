use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_EXPERIMENTS: usize = 128;

#[derive(Clone, Debug, PartialEq)]
struct ExperimentRecord {
    id: u64,
    hypothesis: String,
    status: String,
    score: f32,
    evidence: String,
    outcome: String,
}

#[derive(Clone, Debug)]
struct ExperimentState {
    records: VecDeque<ExperimentRecord>,
    next_id: u64,
    completed: u64,
    inconclusive: u64,
}

impl Default for ExperimentState {
    fn default() -> Self {
        Self {
            records: VecDeque::new(),
            next_id: 1,
            completed: 0,
            inconclusive: 0,
        }
    }
}

static EXPERIMENT_STATE: OnceLock<Mutex<ExperimentState>> = OnceLock::new();

fn experiment_state() -> &'static Mutex<ExperimentState> {
    EXPERIMENT_STATE.get_or_init(|| Mutex::new(ExperimentState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = experiment_state().lock() {
        *state = ExperimentState::default();
    }
}

pub fn plan(hypothesis: &str) -> String {
    let trimmed = hypothesis.trim();
    let mut state = experiment_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    push_record(
        &mut state,
        ExperimentRecord {
            id,
            hypothesis: bounded_fragment(trimmed, 180),
            status: "planned".to_string(),
            score: 0.0,
            evidence: "pending_local_evidence".to_string(),
            outcome: "pending".to_string(),
        },
    );
    format!(
        "[EXPERIMENT-PLAN] id={} status=planned hypothesis={}\n{}",
        id,
        trimmed,
        report_locked(&state)
    )
}

pub fn run_next(local_evidence: &str) -> String {
    let mut state = experiment_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(pos) = state
        .records
        .iter()
        .position(|record| record.status == "planned")
    else {
        return format!(
            "[EXPERIMENT-RUN] status=no_planned_experiment\n{}",
            report_locked(&state)
        );
    };
    let score = score_evidence(local_evidence);
    let id = state.records[pos].id;
    let status = if score >= 0.55 {
        state.completed += 1;
        "completed"
    } else {
        state.inconclusive += 1;
        "inconclusive"
    };
    state.records[pos].status = status.to_string();
    state.records[pos].score = score;
    state.records[pos].evidence = summarize_evidence(local_evidence);
    state.records[pos].outcome = if status == "completed" {
        "supports_hypothesis"
    } else {
        "needs_more_local_evidence"
    }
    .to_string();
    format!(
        "[EXPERIMENT-RUN] id={} status={} score={:.1}% outcome={}\n{}",
        id,
        status,
        score * 100.0,
        state.records[pos].outcome,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = experiment_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = experiment_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for record in state.records.iter().rev().take(8) {
        out.push_str(&format!(
            "- experiment={} status={} score={:.3} outcome={} evidence={} hypothesis={}\n",
            record.id,
            record.status,
            record.score,
            record.outcome,
            record.evidence,
            record.hypothesis
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = experiment_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let records: Vec<_> = state
        .records
        .iter()
        .map(|record| {
            serde_json::json!({
                "id": record.id,
                "hypothesis": record.hypothesis,
                "status": record.status,
                "score": record.score,
                "evidence": record.evidence,
                "outcome": record.outcome,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "experiment-runner-v1",
        "next_id": state.next_id,
        "completed": state.completed,
        "inconclusive": state.inconclusive,
        "records": records,
    });
    std::fs::write(
        state_paths::experiment_runner_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write experiment runner state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::experiment_runner_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read experiment runner state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse experiment runner JSON: {}", e))?;
    let mut state = ExperimentState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.completed = snapshot
        .get("completed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.inconclusive = snapshot
        .get("inconclusive")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(records) = snapshot.get("records").and_then(|v| v.as_array()) {
        for record in records {
            push_record(
                &mut state,
                ExperimentRecord {
                    id: record.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    hypothesis: json_string(record, "hypothesis"),
                    status: json_string(record, "status"),
                    score: record.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                    evidence: json_string(record, "evidence"),
                    outcome: json_string(record, "outcome"),
                },
            );
        }
    }
    *experiment_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &ExperimentState) -> String {
    let planned = state
        .records
        .iter()
        .filter(|record| record.status == "planned")
        .count();
    format!(
        "[EXPERIMENT] schema=experiment-runner-v1 experiments={} planned={} completed={} inconclusive={}\n",
        state.records.len(),
        planned,
        state.completed,
        state.inconclusive
    )
}

fn score_evidence(local_evidence: &str) -> f32 {
    let mut score: f32 = 0.0;
    if local_evidence.contains("[EVAL]") || local_evidence.contains("[EVAL-RUN]") {
        score += 0.25;
    }
    if local_evidence.contains("[BENCH]") || local_evidence.contains("[BENCH-RUN]") {
        score += 0.25;
    }
    if local_evidence.contains("[LEARNING]") || local_evidence.contains("[LEARNING-RECORD]") {
        score += 0.20;
    }
    if local_evidence.contains("[UNCERTAINTY]") || local_evidence.contains("high_risk=0") {
        score += 0.15;
    }
    if local_evidence.contains("[EXEC]") || local_evidence.contains("[ATTENTION]") {
        score += 0.15;
    }
    if local_evidence.contains("[POLICY]") {
        score += 0.05;
    }
    score.min(1.0)
}

fn summarize_evidence(local_evidence: &str) -> String {
    let mut parts = Vec::new();
    for marker in [
        "[EVAL]",
        "[BENCH]",
        "[LEARNING]",
        "[UNCERTAINTY]",
        "[EXEC]",
        "[ATTENTION]",
        "[POLICY]",
    ] {
        if local_evidence.contains(marker) {
            parts.push(marker.trim_matches(['[', ']']));
        }
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join("+")
    }
}

fn push_record(state: &mut ExperimentState, record: ExperimentRecord) {
    state.records.push_back(record);
    while state.records.len() > MAX_EXPERIMENTS {
        state.records.pop_front();
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
    fn plans_and_runs_local_experiment() {
        reset_for_tests();

        let planned = plan("benchmark improves evaluation calibration");
        let run = run_next("[EVAL] [BENCH] [LEARNING] [UNCERTAINTY] [EXEC] [POLICY]");

        assert!(planned.contains("[EXPERIMENT-PLAN] id=1"));
        assert!(run.contains("status=completed"));
        assert!(run.contains("outcome=supports_hypothesis"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_experiment_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_experiment_runner_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = plan("persist experiment");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("experiments=1"));
        assert!(report.contains("planned=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
