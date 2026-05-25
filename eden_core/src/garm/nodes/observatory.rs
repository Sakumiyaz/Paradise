use crate::eden_garm::capabilities::GarmCapabilityState;
use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct ObservatoryNode {
    id: usize,
    reports: u64,
    internal_fe: f32,
}

impl ObservatoryNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            reports: 0,
            internal_fe: 1.0,
        }
    }

    pub fn report(
        &mut self,
        engine: &GarmCapabilityState,
        alive_nodes: usize,
        edge_count: usize,
        memory_facts: usize,
        uptime_sec: u64,
        ready: bool,
        autonomous: bool,
    ) -> String {
        self.reports += 1;
        format!(
            "OBSERVATORIO GARM\n- runtime: ready={} autonomous={} uptime={}s\n- graph: alive_nodes={} edges={}\n- capabilities: ticks={} parse_rate={:.3} reward_ema={:.3} energy={:.1}\n- memory: facts={}\n- reports={}",
            ready,
            autonomous,
            uptime_sec,
            alive_nodes,
            edge_count,
            engine.state.tick_count,
            engine.gen_metrics.parse_rate(),
            engine.gen_metrics.reward_ema,
            engine.metabolism.energy,
            memory_facts,
            self.reports,
        )
    }

    pub fn autonomy_snapshot(
        &mut self,
        alive_nodes: usize,
        edge_count: usize,
        memory_facts: usize,
        tick: u64,
    ) -> String {
        self.reports += 1;
        format!(
            "OBSERVATORIO AUTONOMO\n- tick={} alive_nodes={} edges={} memory_facts={} reports={}",
            tick, alive_nodes, edge_count, memory_facts, self.reports
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({ "reports": self.reports });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.reports = snapshot
            .get("reports")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        Ok(())
    }

    pub fn observatory_snapshot(&self) -> String {
        format!(
            "observatory:reports:{} internal_fe:{:.3}",
            self.reports, self.internal_fe
        )
    }
}

impl GARMNode for ObservatoryNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "observatory"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.reports as f32, self.internal_fe]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.reports as f32])
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
