use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_LEARNING_ENTRIES: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearningEntry {
    pub id: u64,
    pub source: String,
    pub hypothesis: String,
    pub evidence: String,
    pub outcome: String,
    pub confidence: u8,
    pub status: String,
}

#[derive(Clone, Debug)]
struct LearningLedgerState {
    entries: VecDeque<LearningEntry>,
    next_id: u64,
    recorded: u64,
    consolidated: u64,
    contradicted: u64,
}

impl Default for LearningLedgerState {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            next_id: 1,
            recorded: 0,
            consolidated: 0,
            contradicted: 0,
        }
    }
}

static LEARNING_STATE: OnceLock<Mutex<LearningLedgerState>> = OnceLock::new();

fn learning_state() -> &'static Mutex<LearningLedgerState> {
    LEARNING_STATE.get_or_init(|| Mutex::new(LearningLedgerState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = learning_state().lock() {
        *state = LearningLedgerState::default();
    }
}

pub fn record(source: &str, hypothesis: &str, evidence: &str, outcome: &str) -> String {
    let clean_hypothesis = hypothesis.trim();
    let hypothesis = if clean_hypothesis.is_empty() {
        "aprendizaje sin hipotesis especifica"
    } else {
        clean_hypothesis
    };
    let mut state = learning_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    state.recorded += 1;
    let confidence = confidence_from(evidence, outcome);
    let status = if outcome.contains("blocked") || outcome.contains("failed") {
        state.contradicted += 1;
        "contradicted"
    } else if confidence >= 60 {
        state.consolidated += 1;
        "consolidated"
    } else {
        "observed"
    };
    push_entry(
        &mut state,
        LearningEntry {
            id,
            source: source.to_string(),
            hypothesis: hypothesis.to_string(),
            evidence: evidence.to_string(),
            outcome: outcome.to_string(),
            confidence,
            status: status.to_string(),
        },
    );
    format!(
        "[LEARNING-RECORD] id={} source={} status={} confidence={} hypothesis='{}' evidence='{}' outcome='{}'\n",
        id, source, status, confidence, hypothesis, evidence, outcome
    )
}

pub fn consolidate() -> String {
    let mut state = learning_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut changed = 0usize;
    for entry in &mut state.entries {
        if entry.status == "observed" && entry.confidence >= 50 {
            entry.status = "consolidated".to_string();
            changed += 1;
        }
    }
    state.consolidated += changed as u64;
    format!(
        "[LEARNING-CONSOLIDATE] changed={} consolidated={} contradicted={}\n{}",
        changed,
        state.consolidated,
        state.contradicted,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = learning_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = learning_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for entry in state.entries.iter().rev().take(8) {
        out.push_str(&format!(
            "- learning={} source={} status={} confidence={} hypothesis='{}' evidence='{}' outcome='{}'\n",
            entry.id,
            entry.source,
            entry.status,
            entry.confidence,
            entry.hypothesis,
            entry.evidence,
            entry.outcome
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = learning_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let entries: Vec<_> = state
        .entries
        .iter()
        .map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "source": entry.source,
                "hypothesis": entry.hypothesis,
                "evidence": entry.evidence,
                "outcome": entry.outcome,
                "confidence": entry.confidence,
                "status": entry.status,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "learning-ledger-v1",
        "next_id": state.next_id,
        "recorded": state.recorded,
        "consolidated": state.consolidated,
        "contradicted": state.contradicted,
        "entries": entries,
    });
    std::fs::write(
        state_paths::learning_ledger_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write learning ledger state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::learning_ledger_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read learning ledger state: {}", e))?;
    let snapshot: serde_json::Value =
        serde_json::from_str(&data).map_err(|e| format!("failed to parse learning JSON: {}", e))?;
    let mut state = LearningLedgerState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.recorded = snapshot
        .get("recorded")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.consolidated = snapshot
        .get("consolidated")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.contradicted = snapshot
        .get("contradicted")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(entries) = snapshot.get("entries").and_then(|v| v.as_array()) {
        for entry in entries {
            push_entry(
                &mut state,
                LearningEntry {
                    id: entry.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    source: json_string(entry, "source"),
                    hypothesis: json_string(entry, "hypothesis"),
                    evidence: json_string(entry, "evidence"),
                    outcome: json_string(entry, "outcome"),
                    confidence: entry
                        .get("confidence")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u8,
                    status: json_string(entry, "status"),
                },
            );
        }
    }
    *learning_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &LearningLedgerState) -> String {
    let last = state
        .entries
        .back()
        .map(|entry| format!("{}:{}:{}", entry.id, entry.status, entry.hypothesis))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "[LEARNING] schema=learning-ledger-v1 entries={} recorded={} consolidated={} contradicted={} last={}\n",
        state.entries.len(),
        state.recorded,
        state.consolidated,
        state.contradicted,
        last
    )
}

fn push_entry(state: &mut LearningLedgerState, entry: LearningEntry) {
    state.entries.push_back(entry);
    while state.entries.len() > MAX_LEARNING_ENTRIES {
        state.entries.pop_front();
    }
}

fn confidence_from(evidence: &str, outcome: &str) -> u8 {
    let mut confidence = 25u8;
    if !evidence.trim().is_empty() {
        confidence += 20;
    }
    if evidence.contains("eval") || evidence.contains("audit") || evidence.contains("kg") {
        confidence += 15;
    }
    if outcome.contains("completed") || outcome.contains("improving") || outcome.contains("ready") {
        confidence += 25;
    } else if outcome.contains("blocked") || outcome.contains("failed") {
        confidence = confidence.saturating_sub(20);
    }
    confidence.min(100)
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
    fn records_and_audits_learning_entries() {
        let _state_guard = state_paths::test_state_guard();
        reset_for_tests();

        let out = record("test", "goals improve planning", "eval audit", "completed");
        let audit = audit_report();

        assert!(out.contains("[LEARNING-RECORD] id=1"));
        assert!(audit.contains("schema=learning-ledger-v1"));
        assert!(audit.contains("status=consolidated"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_learning_ledger_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_learning_ledger_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = record("test", "persist learning", "kg evidence", "ready");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("entries=1"));
        assert!(report.contains("recorded=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
