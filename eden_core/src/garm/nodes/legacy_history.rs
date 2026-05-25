use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct LegacyHistoryNode {
    id: usize,
    events: Vec<String>,
    internal_fe: f32,
}

impl LegacyHistoryNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            events: Vec::new(),
            internal_fe: 1.0,
        }
    }

    pub fn record_command(&mut self, raw: &str) {
        let clean = raw.trim();
        if clean.is_empty() {
            return;
        }
        self.events.push(format!("cmd: {}", clean));
        if self.events.len() > 100 {
            let overflow = self.events.len() - 100;
            self.events.drain(0..overflow);
        }
    }

    pub fn report(&self) -> String {
        if self.events.is_empty() {
            return "Historial GARM vacio.".to_string();
        }
        let mut out = String::from("Historial GARM:\n");
        let start = self.events.len().saturating_sub(10);
        for (idx, event) in self.events.iter().enumerate().skip(start) {
            out.push_str(&format!("{}. {}\n", idx + 1, event));
        }
        out.push_str(&format!("Total: {} eventos", self.events.len()));
        out
    }

    pub fn recent_events(&self, limit: usize) -> Vec<String> {
        let start = self.events.len().saturating_sub(limit);
        self.events.iter().skip(start).cloned().collect()
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({ "events": self.events });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.events = snapshot
            .get("events")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "missing events array".to_string())?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect();
        if self.events.len() > 100 {
            let overflow = self.events.len() - 100;
            self.events.drain(0..overflow);
        }
        Ok(())
    }

    pub fn history_snapshot(&self) -> String {
        format!(
            "history:events:{} internal_fe:{:.3}",
            self.events.len(),
            self.internal_fe
        )
    }
}

impl GARMNode for LegacyHistoryNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_history"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe + (self.events.len() as f32).ln_1p() * 0.01
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.events.len() as f32, self.internal_fe]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.events.len() as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
        0.2
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        10.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::LegacyHistoryNode;

    #[test]
    fn saves_loads_and_reports_events() {
        let path = std::env::temp_dir().join(format!(
            "eden_garm_legacy_history_test_{}.json",
            std::process::id()
        ));
        let path_str = path.to_string_lossy().to_string();

        let mut source = LegacyHistoryNode::new(20_001);
        source.record_command("hola");
        source.record_command("evolve");
        source.save_state(&path_str).unwrap();

        let mut restored = LegacyHistoryNode::new(20_002);
        restored.load_state(&path_str).unwrap();
        let report = restored.report();

        assert!(report.contains("cmd: hola"));
        assert!(report.contains("cmd: evolve"));
        assert!(report.contains("Total: 2 eventos"));

        let _ = std::fs::remove_file(path);
    }
}
