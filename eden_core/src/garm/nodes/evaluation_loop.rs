use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_EVALUATIONS: usize = 128;

#[derive(Clone, Debug, PartialEq)]
pub struct EvaluationInput {
    pub tick: u64,
    pub memory_facts: usize,
    pub kg_edges: usize,
    pub alive_nodes: usize,
    pub graph_edges: usize,
    pub readiness_score: f32,
    pub goals_report: String,
    pub organ_audit: String,
    pub hrm_snapshot: String,
    pub benchmark_snapshot: String,
    pub attention_snapshot: String,
    pub uncertainty_snapshot: String,
    pub experiment_snapshot: String,
    pub provenance_snapshot: String,
    pub policy_snapshot: String,
    pub maturity_snapshot: String,
    pub hrm_text_snapshot: String,
}

#[derive(Clone, Debug, PartialEq)]
struct EvaluationRecord {
    id: u64,
    tick: u64,
    score: f32,
    verdict: String,
    architecture_score: f32,
    evidence_score: f32,
    execution_score: f32,
    learning_score: f32,
    regression_risk: String,
    recommendation: String,
}

#[derive(Clone, Debug)]
struct EvaluationLoopState {
    records: VecDeque<EvaluationRecord>,
    next_id: u64,
    runs: u64,
    improvements: u64,
    regressions: u64,
}

impl Default for EvaluationLoopState {
    fn default() -> Self {
        Self {
            records: VecDeque::new(),
            next_id: 1,
            runs: 0,
            improvements: 0,
            regressions: 0,
        }
    }
}

static EVALUATION_STATE: OnceLock<Mutex<EvaluationLoopState>> = OnceLock::new();

fn evaluation_state() -> &'static Mutex<EvaluationLoopState> {
    EVALUATION_STATE.get_or_init(|| Mutex::new(EvaluationLoopState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = evaluation_state().lock() {
        *state = EvaluationLoopState::default();
    }
}

pub fn run_evaluation(input: EvaluationInput) -> String {
    let architecture_score = score_ratio(input.alive_nodes, 120) * 0.30
        + score_ratio(input.graph_edges, 500) * 0.20
        + if input.goals_report.contains("goals=") {
            0.25
        } else {
            0.0
        }
        + if input.organ_audit.contains("total=32") {
            0.25
        } else {
            0.0
        };
    let evidence_score = score_ratio(input.memory_facts, 50) * 0.32
        + score_ratio(input.kg_edges, 100) * 0.33
        + if input.hrm_snapshot.contains("evidence:") {
            0.15
        } else {
            0.0
        }
        + if input.goals_report.contains("completed=") {
            0.15
        } else {
            0.0
        }
        + if input.provenance_snapshot.contains("verified=") {
            0.03
        } else {
            0.0
        }
        + if input.maturity_snapshot.contains("average=") {
            0.02
        } else {
            0.0
        }
        + if input.hrm_text_snapshot.contains("segments=") {
            0.03
        } else {
            0.0
        }
        + if input.hrm_text_snapshot.contains("retrieval_hits=")
            && !input.hrm_text_snapshot.contains("retrieval_hits=0")
        {
            0.04
        } else {
            0.0
        };
    let execution_score = if input.goals_report.contains("blocked=0") {
        0.32
    } else {
        0.15
    } + if input.organ_audit.contains("missing=0") {
        0.30
    } else {
        0.0
    } + if input.benchmark_snapshot.contains("tests:")
        || input.benchmark_snapshot.contains("[BENCH]")
    {
        0.20
    } else {
        0.0
    } + score_ratio(input.graph_edges, 500) * 0.15
        + if input.policy_snapshot.contains("blocked=") {
            0.03
        } else {
            0.0
        };
    let learning_score = input.readiness_score.clamp(0.0, 1.0) * 0.30
        + score_ratio(input.memory_facts, 100) * 0.25
        + score_ratio(input.kg_edges, 200) * 0.25
        + if input.goals_report.contains("planned=") {
            0.10
        } else {
            0.0
        }
        + if input.attention_snapshot.contains("items=") {
            0.05
        } else {
            0.0
        }
        + if input.uncertainty_snapshot.contains("open=") {
            0.03
        } else {
            0.0
        }
        + if input.experiment_snapshot.contains("experiments=") {
            0.02
        } else {
            0.0
        }
        + if input.hrm_text_snapshot.contains("corpora=") {
            0.03
        } else {
            0.0
        };
    let score = ((architecture_score + evidence_score + execution_score + learning_score) / 4.0)
        .clamp(0.0, 1.0);
    let verdict = if score >= 0.70 {
        "improving"
    } else if score >= 0.45 {
        "stable"
    } else {
        "needs_evidence"
    };
    let regression_risk = if input.organ_audit.contains("missing=0") && input.graph_edges > 0 {
        "low"
    } else {
        "medium"
    };
    let recommendation = if !input.hrm_text_snapshot.contains("retrieval_hits=")
        || input.hrm_text_snapshot.contains("retrieval_hits=0")
    {
        "run_hrm_text_search_before_regression_eval"
    } else if evidence_score < 0.35 {
        "add_memory_kg_evidence"
    } else if execution_score < 0.45 {
        "run_goals_and_organs"
    } else if learning_score < 0.35 {
        "connect_learning_ledger"
    } else {
        "continue_architecture_deepening"
    };
    let mut state = evaluation_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = state.records.back().map(|record| record.score);
    let id = state.next_id;
    state.next_id += 1;
    state.runs += 1;
    if let Some(previous) = previous {
        if score + 0.001 < previous {
            state.regressions += 1;
        } else if score > previous + 0.001 {
            state.improvements += 1;
        }
    }
    push_record(
        &mut state,
        EvaluationRecord {
            id,
            tick: input.tick,
            score,
            verdict: verdict.to_string(),
            architecture_score,
            evidence_score,
            execution_score,
            learning_score,
            regression_risk: regression_risk.to_string(),
            recommendation: recommendation.to_string(),
        },
    );
    format!(
        "[EVAL-RUN] id={} score={:.1}% verdict={} regression_risk={} recommendation={}\n- architecture={:.1}% evidence={:.1}% execution={:.1}% learning={:.1}%\n{}",
        id,
        score * 100.0,
        verdict,
        regression_risk,
        recommendation,
        architecture_score * 100.0,
        evidence_score * 100.0,
        execution_score * 100.0,
        learning_score * 100.0,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = evaluation_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = evaluation_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for record in state.records.iter().rev().take(8) {
        out.push_str(&format!(
            "- eval={} tick={} score={:.3} verdict={} arch={:.3} evidence={:.3} execution={:.3} learning={:.3} risk={} recommendation={}\n",
            record.id,
            record.tick,
            record.score,
            record.verdict,
            record.architecture_score,
            record.evidence_score,
            record.execution_score,
            record.learning_score,
            record.regression_risk,
            record.recommendation
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = evaluation_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let records: Vec<_> = state
        .records
        .iter()
        .map(|record| {
            serde_json::json!({
                "id": record.id,
                "tick": record.tick,
                "score": record.score,
                "verdict": record.verdict,
                "architecture_score": record.architecture_score,
                "evidence_score": record.evidence_score,
                "execution_score": record.execution_score,
                "learning_score": record.learning_score,
                "regression_risk": record.regression_risk,
                "recommendation": record.recommendation,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "evaluation-loop-v1",
        "next_id": state.next_id,
        "runs": state.runs,
        "improvements": state.improvements,
        "regressions": state.regressions,
        "records": records,
    });
    std::fs::write(
        state_paths::evaluation_loop_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write evaluation loop state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::evaluation_loop_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read evaluation loop state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse evaluation JSON: {}", e))?;
    let mut state = EvaluationLoopState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.runs = snapshot.get("runs").and_then(|v| v.as_u64()).unwrap_or(0);
    state.improvements = snapshot
        .get("improvements")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.regressions = snapshot
        .get("regressions")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(records) = snapshot.get("records").and_then(|v| v.as_array()) {
        for record in records {
            push_record(
                &mut state,
                EvaluationRecord {
                    id: record.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    tick: record.get("tick").and_then(|v| v.as_u64()).unwrap_or(0),
                    score: json_f32(record, "score"),
                    verdict: json_string(record, "verdict"),
                    architecture_score: json_f32(record, "architecture_score"),
                    evidence_score: json_f32(record, "evidence_score"),
                    execution_score: json_f32(record, "execution_score"),
                    learning_score: json_f32(record, "learning_score"),
                    regression_risk: json_string(record, "regression_risk"),
                    recommendation: json_string(record, "recommendation"),
                },
            );
        }
    }
    *evaluation_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &EvaluationLoopState) -> String {
    let last = state
        .records
        .back()
        .map(|record| {
            format!(
                "{}:{:.1}%:{}",
                record.id,
                record.score * 100.0,
                record.verdict
            )
        })
        .unwrap_or_else(|| "none".to_string());
    format!(
        "[EVAL] schema=evaluation-loop-v1 runs={} records={} improvements={} regressions={} last={}\n",
        state.runs,
        state.records.len(),
        state.improvements,
        state.regressions,
        last
    )
}

fn push_record(state: &mut EvaluationLoopState, record: EvaluationRecord) {
    state.records.push_back(record);
    while state.records.len() > MAX_EVALUATIONS {
        state.records.pop_front();
    }
}

fn score_ratio(value: usize, target: usize) -> f32 {
    if target == 0 {
        return 0.0;
    }
    (value as f32 / target as f32).clamp(0.0, 1.0)
}

fn json_f32(value: &serde_json::Value, key: &str) -> f32 {
    value.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32
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

    fn input(memory_facts: usize, kg_edges: usize) -> EvaluationInput {
        EvaluationInput {
            tick: 7,
            memory_facts,
            kg_edges,
            alive_nodes: 123,
            graph_edges: 510,
            readiness_score: 0.30,
            goals_report: "[GOALS] goals=1 planned=1 completed=1 blocked=0".to_string(),
            organ_audit: "[ORGANOS-AUDIT] total=32 missing=0".to_string(),
            hrm_snapshot: "hrm:runs:1 evidence:2".to_string(),
            benchmark_snapshot: "benchmark:tests:1 last:99.000 cumulative:9.900".to_string(),
            attention_snapshot: "[ATTENTION] items=1 focus_shifts=1".to_string(),
            uncertainty_snapshot: "[UNCERTAINTY] records=1 open=1".to_string(),
            experiment_snapshot: "[EXPERIMENT] experiments=1 completed=1".to_string(),
            provenance_snapshot: "[PROVENANCE] records=1 verified=1".to_string(),
            policy_snapshot: "[POLICY] decisions=1 allowed=1 blocked=0".to_string(),
            maturity_snapshot: "[MATURITY] records=1 average=5.00".to_string(),
            hrm_text_snapshot: "[HRM-TEXT] corpora=1 segments=2 retrieval_hits=1".to_string(),
        }
    }

    #[test]
    fn evaluates_architecture_evidence_execution_and_learning() {
        reset_for_tests();

        let out = run_evaluation(input(10, 20));
        let audit = audit_report();

        assert!(out.contains("[EVAL-RUN] id=1"));
        assert!(out.contains("architecture="));
        assert!(audit.contains("schema=evaluation-loop-v1"));
        assert!(audit.contains("recommendation="));
        reset_for_tests();
    }

    #[test]
    fn recommends_retrieval_before_regression_eval_when_hits_are_absent() {
        reset_for_tests();
        let mut input = input(10, 20);
        input.hrm_text_snapshot = "[HRM-TEXT] corpora=1 segments=2 retrieval_hits=0".to_string();

        let out = run_evaluation(input);

        assert!(out.contains("recommendation=run_hrm_text_search_before_regression_eval"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_evaluation_loop_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_evaluation_loop_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = run_evaluation(input(50, 80));
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("runs=1"));
        assert!(report.contains("records=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
