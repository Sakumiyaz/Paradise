use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_PLANS: usize = 128;
const MAX_TEXT_LEN: usize = 1024;

#[derive(Clone, Debug, PartialEq)]
struct HybridVoicePlan {
    id: u64,
    text: String,
    backbone: String,
    hierarchy: String,
    prosody: String,
    status: String,
    manifest_path: String,
}

#[derive(Clone, Debug)]
struct HybridVoiceState {
    plans: VecDeque<HybridVoicePlan>,
    next_id: u64,
    manifests: u64,
}

impl Default for HybridVoiceState {
    fn default() -> Self {
        Self {
            plans: VecDeque::new(),
            next_id: 1,
            manifests: 0,
        }
    }
}

static HYBRID_VOICE_STATE: OnceLock<Mutex<HybridVoiceState>> = OnceLock::new();

fn hybrid_state() -> &'static Mutex<HybridVoiceState> {
    HYBRID_VOICE_STATE.get_or_init(|| Mutex::new(HybridVoiceState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = hybrid_state().lock() {
        *state = HybridVoiceState::default();
    }
}

pub fn plan(text: &str) -> String {
    let clamped = clamp_text(text);
    let prosody = infer_prosody(&clamped);
    let mut state = hybrid_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    push_plan(
        &mut state,
        HybridVoicePlan {
            id,
            text: clamped.clone(),
            backbone: "stacked_transformer_backbone".to_string(),
            hierarchy: "garm_hierarchical_loop:text->intent->prosody->acoustic->verify".to_string(),
            prosody: prosody.clone(),
            status: "planned".to_string(),
            manifest_path: String::new(),
        },
    );
    format!(
        "[HYBRID-VOICE-PLAN] id={} backbone=stacked_transformer_backbone hierarchy=garm_hierarchical_loop prosody={} text_len={}\n{}",
        id,
        prosody,
        clamped.len(),
        report_locked(&state)
    )
}

pub fn synthesize_manifest(text: &str, tick: u64) -> String {
    let clamped = clamp_text(text);
    let prosody = infer_prosody(&clamped);
    let manifest_path = state_paths::hybrid_voice_manifest_path();
    let mut state = hybrid_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    let body = format!(
        "schema=hybrid-voice-manifest-v1\nid={}\ntick={}\nbackbone=stacked_transformer_backbone\nhierarchy=garm_hierarchical_loop\nstages=text,intent,prosody,acoustic,verify\nprosody={}\ntext={}\n",
        id, tick, prosody, clamped
    );
    let status = match std::fs::write(&manifest_path, body) {
        Ok(()) => {
            state.manifests += 1;
            "manifest_written"
        }
        Err(_) => "manifest_error",
    };
    push_plan(
        &mut state,
        HybridVoicePlan {
            id,
            text: clamped.clone(),
            backbone: "stacked_transformer_backbone".to_string(),
            hierarchy: "garm_hierarchical_loop:text->intent->prosody->acoustic->verify".to_string(),
            prosody: prosody.clone(),
            status: status.to_string(),
            manifest_path: manifest_path.clone(),
        },
    );
    format!(
        "[HYBRID-VOICE-SYNTH] id={} status={} manifest={} prosody={} text_len={}\n{}",
        id,
        status,
        manifest_path,
        prosody,
        clamped.len(),
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = hybrid_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = hybrid_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for plan in state.plans.iter().rev().take(8) {
        out.push_str(&format!(
            "- hybrid_voice={} status={} backbone={} hierarchy={} prosody={} manifest={} text_len={}\n",
            plan.id,
            plan.status,
            plan.backbone,
            plan.hierarchy,
            plan.prosody,
            plan.manifest_path,
            plan.text.len()
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = hybrid_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let plans: Vec<_> = state
        .plans
        .iter()
        .map(|plan| {
            serde_json::json!({
                "id": plan.id,
                "text": plan.text,
                "backbone": plan.backbone,
                "hierarchy": plan.hierarchy,
                "prosody": plan.prosody,
                "status": plan.status,
                "manifest_path": plan.manifest_path,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "hybrid-voice-v1",
        "next_id": state.next_id,
        "manifests": state.manifests,
        "plans": plans,
    });
    std::fs::write(state_paths::hybrid_voice_state_path(), snapshot.to_string())
        .map_err(|e| format!("failed to write hybrid voice state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::hybrid_voice_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read hybrid voice state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse hybrid voice JSON: {}", e))?;
    let mut state = HybridVoiceState::default();
    state.next_id = snapshot
        .get("next_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.manifests = snapshot
        .get("manifests")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(plans) = snapshot.get("plans").and_then(|v| v.as_array()) {
        for plan in plans {
            push_plan(
                &mut state,
                HybridVoicePlan {
                    id: plan.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    text: json_string(plan, "text"),
                    backbone: json_string(plan, "backbone"),
                    hierarchy: json_string(plan, "hierarchy"),
                    prosody: json_string(plan, "prosody"),
                    status: json_string(plan, "status"),
                    manifest_path: json_string(plan, "manifest_path"),
                },
            );
        }
    }
    *hybrid_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &HybridVoiceState) -> String {
    format!(
        "[HYBRID-VOICE] schema=hybrid-voice-v1 plans={} manifests={} architecture=stacked_transformer+garm_hierarchical_loop\n",
        state.plans.len(),
        state.manifests
    )
}

fn infer_prosody(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    if text.contains('!') || lower.contains("urgente") {
        "high_energy".to_string()
    } else if text.contains('?') {
        "rising_question".to_string()
    } else if lower.contains("calma") || lower.contains("suave") {
        "soft_calm".to_string()
    } else {
        "neutral_clear".to_string()
    }
}

fn push_plan(state: &mut HybridVoiceState, plan: HybridVoicePlan) {
    state.plans.push_back(plan);
    while state.plans.len() > MAX_PLANS {
        state.plans.pop_front();
    }
}

fn clamp_text(text: &str) -> String {
    text.trim().chars().take(MAX_TEXT_LEN).collect()
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
    fn plans_hybrid_voice_architecture() {
        reset_for_tests();

        let out = plan("hola voz clara");

        assert!(out.contains("[HYBRID-VOICE-PLAN] id=1"));
        assert!(out.contains("stacked_transformer_backbone"));
        assert!(out.contains("garm_hierarchical_loop"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_hybrid_voice_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_hybrid_voice_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        state_paths::ensure_state_dir().unwrap();
        reset_for_tests();

        let _ = synthesize_manifest("hola hibrido", 7);
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("plans=1"));
        assert!(report.contains("manifests=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
