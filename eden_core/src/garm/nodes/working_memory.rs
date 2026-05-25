use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_ITEMS: usize = 64;

#[derive(Clone, Debug, PartialEq)]
struct AttentionItem {
    id: u64,
    text: String,
    source: String,
    weight: f32,
    status: String,
}

#[derive(Clone, Debug)]
struct WorkingMemoryState {
    items: VecDeque<AttentionItem>,
    next_id: u64,
    focus_shifts: u64,
    clears: u64,
}

impl Default for WorkingMemoryState {
    fn default() -> Self {
        Self {
            items: VecDeque::new(),
            next_id: 1,
            focus_shifts: 0,
            clears: 0,
        }
    }
}

static WORKING_MEMORY_STATE: OnceLock<Mutex<WorkingMemoryState>> = OnceLock::new();

fn memory_state() -> &'static Mutex<WorkingMemoryState> {
    WORKING_MEMORY_STATE.get_or_init(|| Mutex::new(WorkingMemoryState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = memory_state().lock() {
        *state = WorkingMemoryState::default();
    }
}

pub fn attend(source: &str, text: &str) -> String {
    let trimmed = text.trim();
    let mut state = memory_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    state.focus_shifts += 1;
    let weight = score_attention(trimmed);
    push_item(
        &mut state,
        AttentionItem {
            id,
            text: bounded_fragment(trimmed, 160),
            source: source.to_string(),
            weight,
            status: "focused".to_string(),
        },
    );
    format!(
        "[ATTEND] id={} source={} weight={:.2} text={}\n{}",
        id,
        source,
        weight,
        trimmed,
        report_locked(&state)
    )
}

pub fn clear() -> String {
    let mut state = memory_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let cleared = state.items.len();
    state.items.clear();
    state.clears += 1;
    format!(
        "[ATTENTION-CLEAR] cleared={} clears={}\n{}",
        cleared,
        state.clears,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = memory_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = memory_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for item in state.items.iter().rev().take(8) {
        out.push_str(&format!(
            "- item={} source={} weight={:.3} status={} text={}\n",
            item.id, item.source, item.weight, item.status, item.text
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = memory_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let items: Vec<_> = state
        .items
        .iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id,
                "text": item.text,
                "source": item.source,
                "weight": item.weight,
                "status": item.status,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "working-memory-v1",
        "next_id": state.next_id,
        "focus_shifts": state.focus_shifts,
        "clears": state.clears,
        "items": items,
    });
    std::fs::write(
        state_paths::working_memory_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write working memory state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::working_memory_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read working memory state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse working memory JSON: {}", e))?;
    let mut state = WorkingMemoryState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.focus_shifts = snapshot
        .get("focus_shifts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.clears = snapshot.get("clears").and_then(|v| v.as_u64()).unwrap_or(0);
    if let Some(items) = snapshot.get("items").and_then(|v| v.as_array()) {
        for item in items {
            push_item(
                &mut state,
                AttentionItem {
                    id: item.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    text: json_string(item, "text"),
                    source: json_string(item, "source"),
                    weight: item.get("weight").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                    status: json_string(item, "status"),
                },
            );
        }
    }
    *memory_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &WorkingMemoryState) -> String {
    let top = state
        .items
        .iter()
        .max_by(|left, right| left.weight.total_cmp(&right.weight))
        .map(|item| format!("{}:{:.2}:{}", item.id, item.weight, item.source))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "[ATTENTION] schema=working-memory-v1 items={} focus_shifts={} clears={} top={}\n",
        state.items.len(),
        state.focus_shifts,
        state.clears,
        top
    )
}

fn score_attention(text: &str) -> f32 {
    let mut score: f32 = 0.20;
    let lower = text.to_ascii_lowercase();
    if lower.contains("goal") || lower.contains("objetivo") || lower.contains("plan") {
        score += 0.25;
    }
    if lower.contains("risk") || lower.contains("rollback") || lower.contains("riesgo") {
        score += 0.20;
    }
    if lower.contains("eval") || lower.contains("bench") || lower.contains("evidence") {
        score += 0.20;
    }
    if text.len() > 40 {
        score += 0.15;
    }
    score.min(1.0)
}

fn push_item(state: &mut WorkingMemoryState, item: AttentionItem) {
    state.items.push_back(item);
    while state.items.len() > MAX_ITEMS {
        state.items.pop_front();
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
    fn attends_and_clears_focus_items() {
        reset_for_tests();

        let attended = attend("test", "goal plan with rollback evidence");
        let cleared = clear();

        assert!(attended.contains("[ATTEND] id=1"));
        assert!(attended.contains("items=1"));
        assert!(cleared.contains("cleared=1"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_working_memory_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_working_memory_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = attend("test", "persist focused plan evidence");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("items=1"));
        assert!(report.contains("focus_shifts=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
