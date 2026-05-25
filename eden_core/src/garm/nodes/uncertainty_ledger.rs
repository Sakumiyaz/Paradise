use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_RECORDS: usize = 128;

#[derive(Clone, Debug, PartialEq)]
struct UncertaintyRecord {
    id: u64,
    claim: String,
    source: String,
    confidence: f32,
    risk: String,
    mitigation: String,
    status: String,
}

#[derive(Clone, Debug)]
struct UncertaintyState {
    records: VecDeque<UncertaintyRecord>,
    next_id: u64,
    resolved: u64,
}

impl Default for UncertaintyState {
    fn default() -> Self {
        Self {
            records: VecDeque::new(),
            next_id: 1,
            resolved: 0,
        }
    }
}

static UNCERTAINTY_STATE: OnceLock<Mutex<UncertaintyState>> = OnceLock::new();

fn uncertainty_state() -> &'static Mutex<UncertaintyState> {
    UNCERTAINTY_STATE.get_or_init(|| Mutex::new(UncertaintyState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = uncertainty_state().lock() {
        *state = UncertaintyState::default();
    }
}

pub fn record(source: &str, claim: &str) -> String {
    let trimmed = claim.trim();
    let confidence = estimate_confidence(trimmed);
    let risk = classify_risk(trimmed, confidence);
    let mitigation = mitigation_for(&risk);
    let mut state = uncertainty_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    push_record(
        &mut state,
        UncertaintyRecord {
            id,
            claim: bounded_fragment(trimmed, 180),
            source: source.to_string(),
            confidence,
            risk: risk.to_string(),
            mitigation: mitigation.to_string(),
            status: "open".to_string(),
        },
    );
    format!(
        "[UNCERTAINTY-RECORD] id={} confidence={:.2} risk={} mitigation={} claim={}\n{}",
        id,
        confidence,
        risk,
        mitigation,
        trimmed,
        report_locked(&state)
    )
}

pub fn resolve_next() -> String {
    let mut state = uncertainty_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(pos) = state
        .records
        .iter()
        .position(|record| record.status == "open")
    else {
        return format!(
            "[UNCERTAINTY-RESOLVE] status=no_open_records\n{}",
            report_locked(&state)
        );
    };
    let id = state.records[pos].id;
    state.records[pos].status = "mitigated".to_string();
    state.resolved += 1;
    format!(
        "[UNCERTAINTY-RESOLVE] id={} status=mitigated mitigation={}\n{}",
        id,
        state.records[pos].mitigation,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = uncertainty_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = uncertainty_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for record in state.records.iter().rev().take(8) {
        out.push_str(&format!(
            "- uncertainty={} status={} confidence={:.3} risk={} source={} mitigation={} claim={}\n",
            record.id,
            record.status,
            record.confidence,
            record.risk,
            record.source,
            record.mitigation,
            record.claim
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = uncertainty_state()
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
                "confidence": record.confidence,
                "risk": record.risk,
                "mitigation": record.mitigation,
                "status": record.status,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "uncertainty-ledger-v1",
        "next_id": state.next_id,
        "resolved": state.resolved,
        "records": records,
    });
    std::fs::write(
        state_paths::uncertainty_ledger_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write uncertainty ledger state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::uncertainty_ledger_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read uncertainty ledger state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse uncertainty ledger JSON: {}", e))?;
    let mut state = UncertaintyState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.resolved = snapshot
        .get("resolved")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(records) = snapshot.get("records").and_then(|v| v.as_array()) {
        for record in records {
            push_record(
                &mut state,
                UncertaintyRecord {
                    id: record.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    claim: json_string(record, "claim"),
                    source: json_string(record, "source"),
                    confidence: record
                        .get("confidence")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32,
                    risk: json_string(record, "risk"),
                    mitigation: json_string(record, "mitigation"),
                    status: json_string(record, "status"),
                },
            );
        }
    }
    *uncertainty_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &UncertaintyState) -> String {
    let open = state
        .records
        .iter()
        .filter(|record| record.status == "open")
        .count();
    let high = state
        .records
        .iter()
        .filter(|record| record.risk == "high")
        .count();
    format!(
        "[UNCERTAINTY] schema=uncertainty-ledger-v1 records={} open={} high_risk={} resolved={}\n",
        state.records.len(),
        open,
        high,
        state.resolved
    )
}

fn estimate_confidence(text: &str) -> f32 {
    let lower = text.to_ascii_lowercase();
    let mut confidence: f32 = 0.40;
    if lower.contains("evidence") || lower.contains("verified") || lower.contains("verificado") {
        confidence += 0.25;
    }
    if lower.contains("unknown") || lower.contains("uncertain") || lower.contains("incierto") {
        confidence -= 0.20;
    }
    if lower.contains("risk") || lower.contains("riesgo") || lower.contains("rollback") {
        confidence -= 0.10;
    }
    confidence.clamp(0.05, 0.95)
}

fn classify_risk(text: &str, confidence: f32) -> &'static str {
    let lower = text.to_ascii_lowercase();
    if confidence < 0.30 || lower.contains("unsafe") || lower.contains("peligro") {
        "high"
    } else if confidence < 0.55 || lower.contains("risk") || lower.contains("riesgo") {
        "medium"
    } else {
        "low"
    }
}

fn mitigation_for(risk: &str) -> &'static str {
    match risk {
        "high" => "block_until_verified",
        "medium" => "require_local_evidence",
        _ => "monitor",
    }
}

fn push_record(state: &mut UncertaintyState, record: UncertaintyRecord) {
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
    fn records_and_resolves_uncertainty() {
        reset_for_tests();

        let record = record("test", "unknown risk needs evidence");
        let resolved = resolve_next();

        assert!(record.contains("[UNCERTAINTY-RECORD] id=1"));
        assert!(record.contains("risk=high") || record.contains("risk=medium"));
        assert!(resolved.contains("status=mitigated"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_uncertainty_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_garm_uncertainty_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = record("test", "persist uncertain claim");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("records=1"));
        assert!(report.contains("open=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
