use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_DECISIONS: usize = 128;

#[derive(Clone, Debug, PartialEq)]
struct PolicyDecision {
    id: u64,
    action: String,
    verdict: String,
    reason: String,
    matched_rule: String,
}

#[derive(Clone, Debug)]
struct PolicyState {
    decisions: VecDeque<PolicyDecision>,
    next_id: u64,
    allowed: u64,
    blocked: u64,
}

impl Default for PolicyState {
    fn default() -> Self {
        Self {
            decisions: VecDeque::new(),
            next_id: 1,
            allowed: 0,
            blocked: 0,
        }
    }
}

static POLICY_STATE: OnceLock<Mutex<PolicyState>> = OnceLock::new();

fn policy_state() -> &'static Mutex<PolicyState> {
    POLICY_STATE.get_or_init(|| Mutex::new(PolicyState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = policy_state().lock() {
        *state = PolicyState::default();
    }
}

pub fn evaluate(action: &str) -> String {
    let trimmed = action.trim();
    let (verdict, reason, matched_rule) = decide(trimmed);
    let mut state = policy_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    if verdict == "allow" {
        state.allowed += 1;
    } else {
        state.blocked += 1;
    }
    push_decision(
        &mut state,
        PolicyDecision {
            id,
            action: bounded_fragment(trimmed, 180),
            verdict: verdict.to_string(),
            reason: reason.to_string(),
            matched_rule: matched_rule.to_string(),
        },
    );
    format!(
        "[POLICY-EVAL] id={} verdict={} rule={} reason={} action={}\n{}",
        id,
        verdict,
        matched_rule,
        reason,
        trimmed,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = policy_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = policy_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for decision in state.decisions.iter().rev().take(8) {
        out.push_str(&format!(
            "- policy={} verdict={} rule={} reason={} action={}\n",
            decision.id, decision.verdict, decision.matched_rule, decision.reason, decision.action
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = policy_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let decisions: Vec<_> = state
        .decisions
        .iter()
        .map(|decision| {
            serde_json::json!({
                "id": decision.id,
                "action": decision.action,
                "verdict": decision.verdict,
                "reason": decision.reason,
                "matched_rule": decision.matched_rule,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "policy-guard-v1",
        "next_id": state.next_id,
        "allowed": state.allowed,
        "blocked": state.blocked,
        "rules": ["local_only", "no_shell", "no_remote_network", "no_code_mutation"],
        "decisions": decisions,
    });
    std::fs::write(state_paths::policy_guard_state_path(), snapshot.to_string())
        .map_err(|e| format!("failed to write policy guard state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::policy_guard_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read policy guard state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse policy guard JSON: {}", e))?;
    let mut state = PolicyState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.allowed = snapshot
        .get("allowed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.blocked = snapshot
        .get("blocked")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(decisions) = snapshot.get("decisions").and_then(|v| v.as_array()) {
        for decision in decisions {
            push_decision(
                &mut state,
                PolicyDecision {
                    id: decision.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    action: json_string(decision, "action"),
                    verdict: json_string(decision, "verdict"),
                    reason: json_string(decision, "reason"),
                    matched_rule: json_string(decision, "matched_rule"),
                },
            );
        }
    }
    *policy_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &PolicyState) -> String {
    format!(
        "[POLICY] schema=policy-guard-v1 decisions={} allowed={} blocked={} rules=4\n",
        state.decisions.len(),
        state.allowed,
        state.blocked
    )
}

fn decide(action: &str) -> (&'static str, &'static str, &'static str) {
    let lower = action.to_ascii_lowercase();
    if lower.contains("shell") || lower.contains("bash") || lower.contains("rm -rf") {
        ("block", "shell_execution_not_allowed", "no_shell")
    } else if lower.contains("http://") || lower.contains("https://") || lower.contains("remote") {
        (
            "block",
            "remote_network_requires_explicit_gate",
            "no_remote_network",
        )
    } else if lower.contains("mutate code") || lower.contains("self modify") {
        ("block", "code_mutation_not_allowed", "no_code_mutation")
    } else {
        ("allow", "local_bounded_action", "local_only")
    }
}

fn push_decision(state: &mut PolicyState, decision: PolicyDecision) {
    state.decisions.push_back(decision);
    while state.decisions.len() > MAX_DECISIONS {
        state.decisions.pop_front();
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
    fn evaluates_allowed_and_blocked_actions() {
        reset_for_tests();

        let allowed = evaluate("run local benchmark report");
        let blocked = evaluate("run shell rm -rf tmp");

        assert!(allowed.contains("verdict=allow"));
        assert!(blocked.contains("verdict=block"));
        assert!(blocked.contains("rule=no_shell"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_policy_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_policy_guard_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = evaluate("local action");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("decisions=1"));
        assert!(report.contains("allowed=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
