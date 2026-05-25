//! GARM Node: Coordinator — Minimal clock for GARM capabilities
//!
//! Handles only pulse-level counters. Capability work lives in CapabilityNode.

use crate::eden_garm::capabilities::GarmCapabilityState;
use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use std::sync::{Arc, Mutex};

pub struct CoordinatorNode {
    id: usize,
    engine: Arc<Mutex<GarmCapabilityState>>,
    internal_fe: f32,
    tick_accumulator: f32,
    tick_interval: f32,
    observations: u64,
}

impl CoordinatorNode {
    pub fn new(id: usize, engine: Arc<Mutex<GarmCapabilityState>>) -> Self {
        CoordinatorNode {
            id,
            engine,
            internal_fe: 1.0,
            tick_accumulator: 0.0,
            tick_interval: 1.0,
            observations: 0,
        }
    }

    pub fn observe_capability_pressure(&mut self) -> String {
        self.observations += 1;
        let guard = self.engine.lock().unwrap();
        format!(
            "[COORDINATOR-AUTO] ticks={} idle_ticks={} observations={} internal_fe={:.2}",
            guard.state.tick_count, guard.state.idle_ticks, self.observations, self.internal_fe
        )
    }

    pub fn autonomy_snapshot(&self) -> String {
        let guard = self.engine.lock().unwrap();
        format!(
            "coordinator:ticks:{} idle:{} observations:{} internal_fe:{:.3}",
            guard.state.tick_count, guard.state.idle_ticks, self.observations, self.internal_fe
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "observations": self.observations,
            "internal_fe": self.internal_fe,
            "tick_accumulator": self.tick_accumulator,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.observations = snapshot
            .get("observations")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.internal_fe = snapshot
            .get("internal_fe")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        self.tick_accumulator = snapshot
            .get("tick_accumulator")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        Ok(())
    }
}

impl GARMNode for CoordinatorNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "coordinator"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        // Coordinator must run first in every pulse.
        1000.0
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.internal_fe]
    }

    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.1).min(10.0);
        }
        NodeAction::Output(vec![self.internal_fe])
    }

    fn update(&mut self, dt: f32, energy_in: f32) -> f32 {
        let cost: f32 = 2.0;
        self.tick_accumulator += dt;
        if energy_in > 0.0 && self.tick_accumulator >= self.tick_interval {
            self.tick_accumulator -= self.tick_interval;
            let mut guard = self.engine.lock().unwrap();
            guard.state.tick_count += 1;
            guard.state.idle_ticks += 1;
            guard.action_log.clear();
            drop(guard);
        }
        self.internal_fe *= 0.99;
        cost.max(0.0)
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
