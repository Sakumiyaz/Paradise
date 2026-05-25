use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct LegacyEvolutionNode {
    id: usize,
    requests: u64,
    last_summary: String,
    internal_fe: f32,
}

impl LegacyEvolutionNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            requests: 0,
            last_summary: String::new(),
            internal_fe: 1.0,
        }
    }

    pub fn summarize(
        &mut self,
        pulses: usize,
        ticks_before: u64,
        ticks_after: u64,
        energy_before: f32,
        energy_after: f32,
    ) -> String {
        self.requests += 1;
        self.last_summary = format!(
            "[EVOLVE] Evolucion acotada completada | pulses={} | ticks {}->{} | energy {:.1}->{:.1} | requests={}",
            pulses,
            ticks_before,
            ticks_after,
            energy_before,
            energy_after,
            self.requests,
        );
        self.last_summary.clone()
    }

    pub fn propose_bounded_improvement(&mut self, tick: u64) -> String {
        self.requests += 1;
        self.last_summary = format!(
            "[EVOLVE-AUTO] bounded improvement proposal recorded at tick={} requests={} guard=no_code_mutation",
            tick, self.requests
        );
        self.last_summary.clone()
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "requests": self.requests,
            "last_summary": self.last_summary,
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
        self.last_summary = snapshot
            .get("last_summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Ok(())
    }

    pub fn evolution_snapshot(&self) -> String {
        format!(
            "evolution:requests:{} last_summary_len:{} internal_fe:{:.3}",
            self.requests,
            self.last_summary.len(),
            self.internal_fe
        )
    }
}

impl GARMNode for LegacyEvolutionNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_evolution"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe + self.requests as f32 * 0.01
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.requests as f32, self.internal_fe]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.requests as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
        0.3
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
