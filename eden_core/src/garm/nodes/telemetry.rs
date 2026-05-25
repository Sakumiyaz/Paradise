use crate::eden_garm::capabilities::GarmCapabilityState;
use crate::eden_garm::hypergraph::HyperGraph;
use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use std::sync::{Arc, Mutex};

pub struct TelemetryNode {
    id: usize,
    reports: u64,
    internal_fe: f32,
}

impl TelemetryNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            reports: 0,
            internal_fe: 1.0,
        }
    }

    pub fn status(&mut self, engine: &Arc<Mutex<GarmCapabilityState>>) -> String {
        self.reports += 1;
        status(engine)
    }

    pub fn auto_progress(
        &mut self,
        pulse: usize,
        engine: &Arc<Mutex<GarmCapabilityState>>,
    ) -> String {
        self.reports += 1;
        auto_progress(pulse, engine)
    }

    pub fn periodic(
        &mut self,
        elapsed: f64,
        graph: &HyperGraph,
        engine: &Arc<Mutex<GarmCapabilityState>>,
    ) -> String {
        self.reports += 1;
        periodic(elapsed, graph, engine)
    }

    pub fn refresh_snapshot(&mut self, alive_nodes: usize, tick: u64) -> String {
        self.reports += 1;
        format!(
            "[TELEMETRY-AUTO] tick={} alive_nodes={} reports={}",
            tick, alive_nodes, self.reports
        )
    }

    pub fn telemetry_snapshot(&self) -> String {
        format!(
            "telemetry:reports:{} internal_fe:{:.3}",
            self.reports, self.internal_fe
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "reports": self.reports,
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
        self.reports = snapshot
            .get("reports")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.internal_fe = snapshot
            .get("internal_fe")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        Ok(())
    }
}

pub fn status(engine: &Arc<Mutex<GarmCapabilityState>>) -> String {
    let guard = engine.lock().unwrap();
    guard.status_summary()
}

pub fn auto_progress(pulse: usize, engine: &Arc<Mutex<GarmCapabilityState>>) -> String {
    let guard = engine.lock().unwrap();
    format!(
        "  pulse {:4} | garm_ticks={} | parse={:.3} | reward_ema={:.3} | energy={:.1}",
        pulse,
        guard.state.tick_count,
        guard.gen_metrics.parse_rate(),
        guard.gen_metrics.reward_ema,
        guard.metabolism.energy,
    )
}

pub fn periodic(
    elapsed: f64,
    graph: &HyperGraph,
    engine: &Arc<Mutex<GarmCapabilityState>>,
) -> String {
    let guard = engine.lock().unwrap();
    format!(
        "[GARM] t={:.0}s | alive={} | garm_ticks={} | parse={:.3} | reward={:.3} | energy={:.1}",
        elapsed,
        graph.alive_node_count(),
        guard.state.tick_count,
        guard.gen_metrics.parse_rate(),
        guard.gen_metrics.reward_ema,
        guard.metabolism.energy,
    )
}

impl GARMNode for TelemetryNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "telemetry"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.reports as f32]
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
