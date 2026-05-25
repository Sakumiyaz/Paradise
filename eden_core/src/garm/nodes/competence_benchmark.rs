use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_BENCHMARK_RUNS: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BenchmarkCase {
    pub name: &'static str,
    pub dimension: &'static str,
    pub evidence: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
struct BenchmarkRun {
    id: u64,
    score: f32,
    passed: usize,
    total: usize,
    verdict: String,
    details: String,
}

#[derive(Clone, Debug)]
struct CompetenceBenchmarkState {
    runs: VecDeque<BenchmarkRun>,
    next_id: u64,
    best_score: f32,
}

impl Default for CompetenceBenchmarkState {
    fn default() -> Self {
        Self {
            runs: VecDeque::new(),
            next_id: 1,
            best_score: 0.0,
        }
    }
}

pub const BENCHMARK_CASES: &[BenchmarkCase] = &[
    BenchmarkCase {
        name: "goal_contracts",
        dimension: "planning",
        evidence: "goals=",
    },
    BenchmarkCase {
        name: "evaluation_loop",
        dimension: "measurement",
        evidence: "records=",
    },
    BenchmarkCase {
        name: "learning_ledger",
        dimension: "learning",
        evidence: "entries=",
    },
    BenchmarkCase {
        name: "world_model",
        dimension: "prediction",
        evidence: "predictions=",
    },
    BenchmarkCase {
        name: "organ_autonomy",
        dimension: "execution",
        evidence: "total=32",
    },
    BenchmarkCase {
        name: "hrm_reasoning",
        dimension: "reasoning",
        evidence: "hrm:runs:",
    },
    BenchmarkCase {
        name: "plan_executor",
        dimension: "rollback_execution",
        evidence: "[EXEC]",
    },
    BenchmarkCase {
        name: "working_memory",
        dimension: "attention",
        evidence: "[ATTENTION]",
    },
    BenchmarkCase {
        name: "uncertainty_ledger",
        dimension: "risk_calibration",
        evidence: "[UNCERTAINTY]",
    },
    BenchmarkCase {
        name: "experiment_runner",
        dimension: "experimentation",
        evidence: "[EXPERIMENT]",
    },
    BenchmarkCase {
        name: "provenance_ledger",
        dimension: "evidence_provenance",
        evidence: "[PROVENANCE]",
    },
    BenchmarkCase {
        name: "policy_guard",
        dimension: "constraint_checking",
        evidence: "[POLICY]",
    },
    BenchmarkCase {
        name: "capability_maturity",
        dimension: "maturity_tracking",
        evidence: "[MATURITY]",
    },
    BenchmarkCase {
        name: "hrm_text_ingestion",
        dimension: "text_prior_ingestion",
        evidence: "[HRM-TEXT]",
    },
    BenchmarkCase {
        name: "hrm_text_segments",
        dimension: "segment_indexing",
        evidence: "segments=",
    },
    BenchmarkCase {
        name: "hrm_text_retrieval",
        dimension: "text_retrieval",
        evidence: "retrieval_hits=",
    },
];

static BENCHMARK_STATE: OnceLock<Mutex<CompetenceBenchmarkState>> = OnceLock::new();

fn benchmark_state() -> &'static Mutex<CompetenceBenchmarkState> {
    BENCHMARK_STATE.get_or_init(|| Mutex::new(CompetenceBenchmarkState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = benchmark_state().lock() {
        *state = CompetenceBenchmarkState::default();
    }
}

pub fn run(local_evidence: &str) -> String {
    let mut passed = 0usize;
    let mut details = String::new();
    for case in BENCHMARK_CASES {
        let ok = local_evidence.contains(case.evidence);
        if ok {
            passed += 1;
        }
        details.push_str(&format!(
            "- case={} dimension={} passed={} evidence={}\n",
            case.name, case.dimension, ok, case.evidence
        ));
    }
    let total = BENCHMARK_CASES.len();
    let score = passed as f32 / total as f32;
    let verdict = if score >= 0.85 {
        "strong"
    } else if score >= 0.60 {
        "developing"
    } else {
        "insufficient"
    };
    let mut state = benchmark_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    state.best_score = state.best_score.max(score);
    push_run(
        &mut state,
        BenchmarkRun {
            id,
            score,
            passed,
            total,
            verdict: verdict.to_string(),
            details: details.clone(),
        },
    );
    format!(
        "[BENCH-RUN] id={} score={:.1}% passed={}/{} verdict={}\n{}{}",
        id,
        score * 100.0,
        passed,
        total,
        verdict,
        details,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = benchmark_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = benchmark_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for run in state.runs.iter().rev().take(8) {
        out.push_str(&format!(
            "- run={} score={:.3} passed={}/{} verdict={}\n{}",
            run.id, run.score, run.passed, run.total, run.verdict, run.details
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = benchmark_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let runs: Vec<_> = state
        .runs
        .iter()
        .map(|run| {
            serde_json::json!({
                "id": run.id,
                "score": run.score,
                "passed": run.passed,
                "total": run.total,
                "verdict": run.verdict,
                "details": run.details,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "competence-benchmark-v1",
        "next_id": state.next_id,
        "best_score": state.best_score,
        "runs": runs,
    });
    std::fs::write(
        state_paths::competence_benchmark_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write competence benchmark state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::competence_benchmark_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read competence benchmark state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse benchmark JSON: {}", e))?;
    let mut state = CompetenceBenchmarkState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.best_score = snapshot
        .get("best_score")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    if let Some(runs) = snapshot.get("runs").and_then(|v| v.as_array()) {
        for run in runs {
            push_run(
                &mut state,
                BenchmarkRun {
                    id: run.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    score: run.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                    passed: run.get("passed").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                    total: run.get("total").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                    verdict: json_string(run, "verdict"),
                    details: json_string(run, "details"),
                },
            );
        }
    }
    *benchmark_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &CompetenceBenchmarkState) -> String {
    let last = state
        .runs
        .back()
        .map(|run| format!("{}:{:.1}%:{}", run.id, run.score * 100.0, run.verdict))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "[BENCH] schema=competence-benchmark-v1 runs={} cases={} best={:.1}% last={}\n",
        state.runs.len(),
        BENCHMARK_CASES.len(),
        state.best_score * 100.0,
        last
    )
}

fn push_run(state: &mut CompetenceBenchmarkState, run: BenchmarkRun) {
    state.runs.push_back(run);
    while state.runs.len() > MAX_BENCHMARK_RUNS {
        state.runs.pop_front();
    }
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
    fn runs_local_competence_cases() {
        reset_for_tests();

        let out = run(
            "goals=1 records=1 entries=1 predictions=1 total=32 hrm:runs:1 [EXEC] [ATTENTION] [UNCERTAINTY] [EXPERIMENT] [PROVENANCE] [POLICY] [MATURITY] [HRM-TEXT] segments=2 retrieval_hits=1",
        );

        assert!(out.contains("[BENCH-RUN] id=1"));
        assert!(out.contains("passed=16/16"));
        assert!(out.contains("verdict=strong"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_competence_benchmark_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_competence_benchmark_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = run("goals=1 total=32");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("runs=1"));
        assert!(report.contains("cases=16"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
