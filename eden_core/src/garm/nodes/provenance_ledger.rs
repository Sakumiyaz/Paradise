use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_RECORDS: usize = 128;

#[derive(Clone, Debug, PartialEq)]
struct ProvenanceRecord {
    id: u64,
    claim: String,
    source: String,
    evidence_kind: String,
    trust: f32,
    status: String,
}

#[derive(Clone, Debug)]
struct ProvenanceState {
    records: VecDeque<ProvenanceRecord>,
    next_id: u64,
    verified: u64,
}

impl Default for ProvenanceState {
    fn default() -> Self {
        Self {
            records: VecDeque::new(),
            next_id: 1,
            verified: 0,
        }
    }
}

static PROVENANCE_STATE: OnceLock<Mutex<ProvenanceState>> = OnceLock::new();

fn provenance_state() -> &'static Mutex<ProvenanceState> {
    PROVENANCE_STATE.get_or_init(|| Mutex::new(ProvenanceState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = provenance_state().lock() {
        *state = ProvenanceState::default();
    }
}

pub fn record(source: &str, claim: &str) -> String {
    let trimmed = claim.trim();
    let evidence_kind = classify_evidence(trimmed);
    let trust = estimate_trust(source, trimmed, evidence_kind);
    let status = if trust >= 0.70 { "verified" } else { "pending" };
    let mut state = provenance_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    if status == "verified" {
        state.verified += 1;
    }
    push_record(
        &mut state,
        ProvenanceRecord {
            id,
            claim: bounded_fragment(trimmed, 180),
            source: source.to_string(),
            evidence_kind: evidence_kind.to_string(),
            trust,
            status: status.to_string(),
        },
    );
    format!(
        "[PROVENANCE-RECORD] id={} status={} trust={:.2} kind={} source={} claim={}\n{}",
        id,
        status,
        trust,
        evidence_kind,
        source,
        trimmed,
        report_locked(&state)
    )
}

pub fn verify_next() -> String {
    let mut state = provenance_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(pos) = state
        .records
        .iter()
        .position(|record| record.status == "pending")
    else {
        return format!(
            "[PROVENANCE-VERIFY] status=no_pending_records\n{}",
            report_locked(&state)
        );
    };
    let id = state.records[pos].id;
    state.records[pos].status = "verified".to_string();
    state.records[pos].trust = state.records[pos].trust.max(0.70);
    state.verified += 1;
    format!(
        "[PROVENANCE-VERIFY] id={} status=verified trust={:.2}\n{}",
        id,
        state.records[pos].trust,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = provenance_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = provenance_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for record in state.records.iter().rev().take(8) {
        out.push_str(&format!(
            "- provenance={} status={} trust={:.3} kind={} source={} claim={}\n",
            record.id,
            record.status,
            record.trust,
            record.evidence_kind,
            record.source,
            record.claim
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = provenance_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let records: Vec<_> = state
        .records
        .iter()
        .map(|record| {
            serde_json::json!({
                "id": record.id,
                "claim": record.claim,
                "source": record.source,
                "evidence_kind": record.evidence_kind,
                "trust": record.trust,
                "status": record.status,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "provenance-ledger-v1",
        "next_id": state.next_id,
        "verified": state.verified,
        "records": records,
    });
    std::fs::write(
        state_paths::provenance_ledger_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write provenance ledger state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::provenance_ledger_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read provenance ledger state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse provenance ledger JSON: {}", e))?;
    let mut state = ProvenanceState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.verified = snapshot
        .get("verified")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(records) = snapshot.get("records").and_then(|v| v.as_array()) {
        for record in records {
            push_record(
                &mut state,
                ProvenanceRecord {
                    id: record.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    claim: json_string(record, "claim"),
                    source: json_string(record, "source"),
                    evidence_kind: json_string(record, "evidence_kind"),
                    trust: record.get("trust").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                    status: json_string(record, "status"),
                },
            );
        }
    }
    *provenance_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &ProvenanceState) -> String {
    let pending = state
        .records
        .iter()
        .filter(|record| record.status == "pending")
        .count();
    format!(
        "[PROVENANCE] schema=provenance-ledger-v1 records={} verified={} pending={}\n",
        state.records.len(),
        state.verified,
        pending
    )
}

fn classify_evidence(claim: &str) -> &'static str {
    let lower = claim.to_ascii_lowercase();
    if lower.contains("test") || lower.contains("passed") || lower.contains("verification") {
        "test_result"
    } else if lower.contains("report") || lower.contains("audit") {
        "runtime_report"
    } else if lower.contains("operator") || lower.contains("manual") {
        "operator_note"
    } else {
        "local_claim"
    }
}

fn estimate_trust(source: &str, claim: &str, evidence_kind: &str) -> f32 {
    let mut trust: f32 = match evidence_kind {
        "test_result" => 0.80,
        "runtime_report" => 0.70,
        "operator_note" => 0.45,
        _ => 0.35,
    };
    if source.contains("garm") || source.contains("runtime") {
        trust += 0.10;
    }
    if claim.to_ascii_lowercase().contains("unknown") {
        trust -= 0.20;
    }
    trust.clamp(0.05, 0.95)
}

fn push_record(state: &mut ProvenanceState, record: ProvenanceRecord) {
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
    fn records_and_verifies_provenance() {
        reset_for_tests();

        let record = record("runtime", "test passed for benchmark verification");
        let verify = verify_next();

        assert!(record.contains("[PROVENANCE-RECORD] id=1"));
        assert!(record.contains("kind=test_result"));
        assert!(verify.contains("[PROVENANCE-VERIFY]"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_provenance_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_garm_provenance_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = record("test", "persist provenance claim");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("records=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
