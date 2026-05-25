use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use crate::eden_garm::state_paths;

const MAX_TEXT_LEN: usize = 1024;

pub struct VoiceSynthesizerNode {
    id: usize,
    requests: u64,
    artifacts_written: u64,
    backend_available: bool,
    backend_name: String,
    last_text: String,
    last_artifact: String,
    internal_fe: f32,
}

impl VoiceSynthesizerNode {
    pub fn new(id: usize) -> Self {
        let backend_name = std::env::var("EDEN_GARM_TTS_BACKEND")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "text_manifest".to_string());
        let backend_available = backend_name != "text_manifest";
        Self {
            id,
            requests: 0,
            artifacts_written: 0,
            backend_available,
            backend_name,
            last_text: String::new(),
            last_artifact: String::new(),
            internal_fe: 1.0,
        }
    }

    pub fn synthesize_text(&mut self, text: &str, tick: u64) -> String {
        self.requests += 1;
        self.last_text = clamp_text(text);
        let artifact = state_paths::voice_last_artifact_path();
        let backend_request = state_paths::voice_backend_request_path();
        let backend_output = state_paths::voice_backend_output_path();
        let body = format!(
            "backend={}\nbackend_available={}\ntick={}\nrequests={}\ntext={}\nbackend_request={}\nbackend_output={}\n",
            self.backend_name,
            self.backend_available,
            tick,
            self.requests,
            self.last_text,
            backend_request,
            backend_output
        );
        match std::fs::write(&artifact, body) {
            Ok(()) => {
                if self.backend_available {
                    let request = format!(
                        "backend={}\ninput_artifact={}\noutput_artifact={}\ntext={}\n",
                        self.backend_name, artifact, backend_output, self.last_text
                    );
                    let _ = std::fs::write(&backend_request, request);
                }
                self.artifacts_written += 1;
                self.last_artifact = artifact;
                format!(
                    "[VOZ-TTS] backend={} available={} artifact={} requests={} text_len={}\n",
                    self.backend_name,
                    self.backend_available,
                    self.last_artifact,
                    self.requests,
                    self.last_text.len()
                )
            }
            Err(e) => format!(
                "[VOZ-TTS] backend={} available={} artifact_error={} requests={} text_len={}\n",
                self.backend_name,
                self.backend_available,
                e,
                self.requests,
                self.last_text.len()
            ),
        }
    }

    pub fn autodocument(&mut self, summary: &str, tick: u64) -> String {
        self.synthesize_text(summary, tick)
    }

    pub fn snapshot(&self) -> String {
        format!(
            "voice:requests:{} artifacts:{} backend:{} available:{} last_text_len:{} last_artifact_len:{} internal_fe:{:.3}",
            self.requests,
            self.artifacts_written,
            self.backend_name,
            self.backend_available,
            self.last_text.len(),
            self.last_artifact.len(),
            self.internal_fe
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "requests": self.requests,
            "artifacts_written": self.artifacts_written,
            "backend_available": self.backend_available,
            "backend_name": self.backend_name,
            "last_text": self.last_text,
            "last_artifact": self.last_artifact,
            "internal_fe": self.internal_fe,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.requests = snapshot
            .get("requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.artifacts_written = snapshot
            .get("artifacts_written")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.backend_available = snapshot
            .get("backend_available")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        self.backend_name = snapshot
            .get("backend_name")
            .and_then(|v| v.as_str())
            .unwrap_or("text_manifest")
            .to_string();
        self.last_text = snapshot
            .get("last_text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        self.last_artifact = snapshot
            .get("last_artifact")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        self.internal_fe = snapshot
            .get("internal_fe")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        Ok(())
    }
}

impl GARMNode for VoiceSynthesizerNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "voice_synthesizer"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.requests as f32, self.artifacts_written as f32]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.requests as f32, self.artifacts_written as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
        0.2
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        20.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn clamp_text(text: &str) -> String {
    text.trim().chars().take(MAX_TEXT_LEN).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_text_manifest_without_audio_backend() {
        let _state_guard = state_paths::test_state_guard();
        std::env::remove_var("EDEN_GARM_TTS_BACKEND");
        let dir = std::env::temp_dir().join(format!("eden_garm_voice_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        state_paths::ensure_state_dir().unwrap();
        let mut node = VoiceSynthesizerNode::new(1);

        let report = node.synthesize_text("hola voz", 3);

        assert!(report.contains("[VOZ-TTS]"));
        assert!(report.contains("backend=text_manifest"));
        assert!(std::fs::metadata(state_paths::voice_last_artifact_path()).is_ok());
        assert!(node.snapshot().contains("requests:1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn writes_backend_request_when_backend_is_configured() {
        let _state_guard = state_paths::test_state_guard();
        std::env::set_var("EDEN_GARM_TTS_BACKEND", "local_stub");
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_voice_backend_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        state_paths::ensure_state_dir().unwrap();
        let mut node = VoiceSynthesizerNode::new(1);

        let report = node.synthesize_text("hola backend", 4);
        let request = std::fs::read_to_string(state_paths::voice_backend_request_path()).unwrap();

        assert!(report.contains("backend=local_stub"));
        assert!(report.contains("available=true"));
        assert!(request.contains("backend=local_stub"));
        assert!(request.contains("output_artifact="));
        assert!(request.contains("text=hola backend"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        std::env::remove_var("EDEN_GARM_TTS_BACKEND");
    }
}
