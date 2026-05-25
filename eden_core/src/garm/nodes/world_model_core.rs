use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_WORLD_RECORDS: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldObservation {
    pub id: u64,
    pub source: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub confidence: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldPrediction {
    pub id: u64,
    pub query: String,
    pub prediction: String,
    pub evidence: String,
    pub status: String,
    pub confidence: u8,
}

#[derive(Clone, Debug)]
struct WorldModelState {
    observations: VecDeque<WorldObservation>,
    predictions: VecDeque<WorldPrediction>,
    next_observation_id: u64,
    next_prediction_id: u64,
    observations_recorded: u64,
    predictions_made: u64,
    predictions_verified: u64,
}

impl Default for WorldModelState {
    fn default() -> Self {
        Self {
            observations: VecDeque::new(),
            predictions: VecDeque::new(),
            next_observation_id: 1,
            next_prediction_id: 1,
            observations_recorded: 0,
            predictions_made: 0,
            predictions_verified: 0,
        }
    }
}

static WORLD_STATE: OnceLock<Mutex<WorldModelState>> = OnceLock::new();

fn world_state() -> &'static Mutex<WorldModelState> {
    WORLD_STATE.get_or_init(|| Mutex::new(WorldModelState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = world_state().lock() {
        *state = WorldModelState::default();
    }
}

pub fn observe(source: &str, text: &str) -> String {
    let (subject, relation, object) = parse_observation(text);
    let mut state = world_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = state.next_observation_id;
    state.next_observation_id += 1;
    state.observations_recorded += 1;
    push_observation(
        &mut state,
        WorldObservation {
            id,
            source: source.to_string(),
            subject: subject.clone(),
            relation: relation.clone(),
            object: object.clone(),
            confidence: confidence_for(&subject, &relation, &object),
        },
    );
    format!(
        "[WORLD-OBSERVE] id={} source={} subject='{}' relation='{}' object='{}' confidence={}\n",
        id,
        source,
        subject,
        relation,
        object,
        confidence_for(&subject, &relation, &object)
    )
}

pub fn predict(query: &str) -> String {
    let clean_query = query.trim();
    let mut state = world_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let related: Vec<WorldObservation> = state
        .observations
        .iter()
        .filter(|obs| {
            clean_query.is_empty()
                || obs.subject.contains(clean_query)
                || obs.object.contains(clean_query)
                || clean_query.contains(&obs.subject)
                || clean_query.contains(&obs.object)
        })
        .cloned()
        .take(4)
        .collect();
    let evidence = if related.is_empty() {
        "none".to_string()
    } else {
        related
            .iter()
            .map(|obs| {
                format!(
                    "{}:{}:{}'.{}'",
                    obs.id, obs.subject, obs.relation, obs.object
                )
            })
            .collect::<Vec<_>>()
            .join("|")
    };
    let prediction = if related.is_empty() {
        "insufficient_world_evidence".to_string()
    } else {
        format!(
            "{} likely relates_to {}",
            related[0].subject, related[0].object
        )
    };
    let confidence = if related.is_empty() {
        10
    } else {
        (related.iter().map(|obs| obs.confidence as u64).sum::<u64>() / related.len() as u64)
            .min(100) as u8
    };
    let supported = !related.is_empty();
    let id = state.next_prediction_id;
    state.next_prediction_id += 1;
    state.predictions_made += 1;
    push_prediction(
        &mut state,
        WorldPrediction {
            id,
            query: clean_query.to_string(),
            prediction: prediction.clone(),
            evidence: evidence.clone(),
            status: if supported { "supported" } else { "unverified" }.to_string(),
            confidence,
        },
    );
    format!(
        "[WORLD-PREDICT] id={} query='{}' status={} confidence={} prediction='{}' evidence='{}'\n",
        id,
        clean_query,
        if supported { "supported" } else { "unverified" },
        confidence,
        prediction,
        evidence
    )
}

pub fn verify_predictions() -> String {
    let mut state = world_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut verified = 0usize;
    for prediction in &mut state.predictions {
        if prediction.status == "supported" {
            prediction.status = "verified_local".to_string();
            verified += 1;
        }
    }
    state.predictions_verified += verified as u64;
    format!(
        "[WORLD-VERIFY] verified={} total_verified={}\n{}",
        verified,
        state.predictions_verified,
        report_locked(&state)
    )
}

pub fn report() -> String {
    let state = world_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = world_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for obs in state.observations.iter().rev().take(6) {
        out.push_str(&format!(
            "- observation={} source={} confidence={} subject='{}' relation='{}' object='{}'\n",
            obs.id, obs.source, obs.confidence, obs.subject, obs.relation, obs.object
        ));
    }
    for pred in state.predictions.iter().rev().take(6) {
        out.push_str(&format!(
            "- prediction={} status={} confidence={} query='{}' prediction='{}' evidence='{}'\n",
            pred.id, pred.status, pred.confidence, pred.query, pred.prediction, pred.evidence
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = world_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let observations: Vec<_> = state
        .observations
        .iter()
        .map(|obs| {
            serde_json::json!({
                "id": obs.id,
                "source": obs.source,
                "subject": obs.subject,
                "relation": obs.relation,
                "object": obs.object,
                "confidence": obs.confidence,
            })
        })
        .collect();
    let predictions: Vec<_> = state
        .predictions
        .iter()
        .map(|pred| {
            serde_json::json!({
                "id": pred.id,
                "query": pred.query,
                "prediction": pred.prediction,
                "evidence": pred.evidence,
                "status": pred.status,
                "confidence": pred.confidence,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "world-model-core-v1",
        "next_observation_id": state.next_observation_id,
        "next_prediction_id": state.next_prediction_id,
        "observations_recorded": state.observations_recorded,
        "predictions_made": state.predictions_made,
        "predictions_verified": state.predictions_verified,
        "observations": observations,
        "predictions": predictions,
    });
    std::fs::write(
        state_paths::world_model_core_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write world model state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::world_model_core_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read world model state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse world model JSON: {}", e))?;
    let mut state = WorldModelState::default();
    state.next_observation_id = snapshot
        .get("next_observation_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.next_prediction_id = snapshot
        .get("next_prediction_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    state.observations_recorded = snapshot
        .get("observations_recorded")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.predictions_made = snapshot
        .get("predictions_made")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.predictions_verified = snapshot
        .get("predictions_verified")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(observations) = snapshot.get("observations").and_then(|v| v.as_array()) {
        for obs in observations {
            push_observation(
                &mut state,
                WorldObservation {
                    id: obs.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    source: json_string(obs, "source"),
                    subject: json_string(obs, "subject"),
                    relation: json_string(obs, "relation"),
                    object: json_string(obs, "object"),
                    confidence: obs.get("confidence").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                },
            );
        }
    }
    if let Some(predictions) = snapshot.get("predictions").and_then(|v| v.as_array()) {
        for pred in predictions {
            push_prediction(
                &mut state,
                WorldPrediction {
                    id: pred.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    query: json_string(pred, "query"),
                    prediction: json_string(pred, "prediction"),
                    evidence: json_string(pred, "evidence"),
                    status: json_string(pred, "status"),
                    confidence: pred.get("confidence").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                },
            );
        }
    }
    *world_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn parse_observation(text: &str) -> (String, String, String) {
    let clean = text.trim();
    for marker in [" causes ", " causa ", " is ", " es ", " relates_to "] {
        if let Some((left, right)) = clean.split_once(marker) {
            return (
                left.trim().to_string(),
                marker.trim().to_string(),
                right.trim().to_string(),
            );
        }
    }
    (
        clean.to_string(),
        "observed_as".to_string(),
        clean.to_string(),
    )
}

fn confidence_for(subject: &str, relation: &str, object: &str) -> u8 {
    let mut confidence = 30u8;
    if !subject.is_empty() && !object.is_empty() {
        confidence += 25;
    }
    if relation != "observed_as" {
        confidence += 20;
    }
    confidence.min(100)
}

fn report_locked(state: &WorldModelState) -> String {
    let last_prediction = state
        .predictions
        .back()
        .map(|pred| format!("{}:{}:{}", pred.id, pred.status, pred.query))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "[WORLD] schema=world-model-core-v1 observations={} predictions={} verified={} last_prediction={}\n",
        state.observations.len(),
        state.predictions.len(),
        state.predictions_verified,
        last_prediction
    )
}

fn push_observation(state: &mut WorldModelState, observation: WorldObservation) {
    state.observations.push_back(observation);
    while state.observations.len() > MAX_WORLD_RECORDS {
        state.observations.pop_front();
    }
}

fn push_prediction(state: &mut WorldModelState, prediction: WorldPrediction) {
    state.predictions.push_back(prediction);
    while state.predictions.len() > MAX_WORLD_RECORDS {
        state.predictions.pop_front();
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
    fn observes_predicts_and_verifies_world_records() {
        let _state_guard = state_paths::test_state_guard();
        reset_for_tests();

        let observed = observe("test", "rain causes wet_ground");
        let predicted = predict("rain");
        let verified = verify_predictions();

        assert!(observed.contains("[WORLD-OBSERVE] id=1"));
        assert!(predicted.contains("[WORLD-PREDICT] id=1"));
        assert!(predicted.contains("status=supported"));
        assert!(verified.contains("verified=1"));
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_world_model_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_garm_world_model_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        reset_for_tests();

        let _ = observe("test", "goal is planned_action");
        let _ = predict("goal");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("observations=1"));
        assert!(report.contains("predictions=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
