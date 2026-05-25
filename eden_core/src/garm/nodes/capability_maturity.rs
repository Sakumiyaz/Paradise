use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_RECORDS: usize = 128;

#[derive(Clone, Debug, PartialEq)]
struct MaturityRecord {
    id: u64,
    capability: String,
    level: u8,
    evidence: String,
    status: String,
}

#[derive(Clone, Debug)]
struct MaturityState {
    records: VecDeque<MaturityRecord>,
    next_id: u64,
    assessments: u64,
}

impl Default for MaturityState {
    fn default() -> Self {
        Self {
            records: VecDeque::new(),
            next_id: 1,
            assessments: 0,
        }
    }
}

static MATURITY_STATE: OnceLock<Mutex<MaturityState>> = OnceLock::new();

fn maturity_state() -> &'static Mutex<MaturityState> {
    MATURITY_STATE.get_or_init(|| Mutex::new(MaturityState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = maturity_state().lock() {
        *state = MaturityState::default();
    }
}

pub fn assess(capability: &str, local_evidence: &str) -> String {
    let trimmed = capability.trim();
    let level = maturity_level(local_evidence);
    let status = match level {
        4..=5 => "operational",
        2..=3 => "developing",
        _ => "nascent",
    };
    let evidence = summarize_evidence(local_evidence);
    let mut state = maturity_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    state.assessments += 1;
    push_record(
        &mut state,
        MaturityRecord {
            id,
            capability: bounded_fragment(trimmed, 120),
            level,
            evidence,
            status: status.to_string(),
        },
    );
    format!(
        "[MATURITY-ASSESS] id={} capability={} level={} status={}\n{}",
        id,
        trimmed,
        level,
        status,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = maturity_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = maturity_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for record in state.records.iter().rev().take(8) {
        out.push_str(&format!(
            "- maturity={} capability={} level={} status={} evidence={}\n",
            record.id, record.capability, record.level, record.status, record.evidence
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = maturity_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let records: Vec<_> = state
        .records
        .iter()
        .map(|record| {
            serde_json::json!({
                "id": record.id,
                "capability": record.capability,
                "level": record.level,
                "evidence": record.evidence,
                "status": record.status,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "capability-maturity-v1",
        "next_id": state.next_id,
        "assessments": state.assessments,
        "records": records,
    });
    std::fs::write(
        state_paths::capability_maturity_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write capability maturity state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::capability_maturity_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read capability maturity state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse capability maturity JSON: {}", e))?;
    let mut state = MaturityState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.assessments = snapshot
        .get("assessments")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(records) = snapshot.get("records").and_then(|v| v.as_array()) {
        for record in records {
            push_record(
                &mut state,
                MaturityRecord {
                    id: record.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    capability: json_string(record, "capability"),
                    level: record.get("level").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                    evidence: json_string(record, "evidence"),
                    status: json_string(record, "status"),
                },
            );
        }
    }
    *maturity_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &MaturityState) -> String {
    let average = if state.records.is_empty() {
        0.0
    } else {
        state
            .records
            .iter()
            .map(|record| record.level as f32)
            .sum::<f32>()
            / state.records.len() as f32
    };
    format!(
        "[MATURITY] schema=capability-maturity-v1 records={} assessments={} average={:.2}\n",
        state.records.len(),
        state.assessments,
        average
    )
}

fn maturity_level(local_evidence: &str) -> u8 {
    let markers = [
        "[GOALS]",
        "[EVAL]",
        "[LEARNING]",
        "[BENCH]",
        "[EXEC]",
        "[POLICY]",
        "[PROVENANCE]",
        "[HRM-TEXT]",
    ];
    let count = markers
        .iter()
        .filter(|marker| local_evidence.contains(**marker))
        .count();
    count.min(5) as u8
}

fn summarize_evidence(local_evidence: &str) -> String {
    let mut parts = Vec::new();
    for marker in [
        "[GOALS]",
        "[EVAL]",
        "[LEARNING]",
        "[BENCH]",
        "[EXEC]",
        "[POLICY]",
        "[PROVENANCE]",
        "[HRM-TEXT]",
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

fn push_record(state: &mut MaturityState, record: MaturityRecord) {
    state.records.push_back(record);
    while state.records.len() > MAX_RECORDS {
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
    fn assesses_capability_maturity_from_local_evidence() {
        reset_for_tests();

        let out = assess("policy guard", "[GOALS] [EVAL] [LEARNING] [BENCH] [POLICY]");

        assert!(out.contains("[MATURITY-ASSESS] id=1"));
        assert!(out.contains("level=5"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_maturity_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_garm_maturity_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = assess("experiment", "[EVAL]");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("records=1"));
        assert!(report.contains("assessments=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
