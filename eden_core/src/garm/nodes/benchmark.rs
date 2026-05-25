//! GARM Node: Benchmark — Evaluacion continua del sistema
//!
//! Corre periodicamente y mide convergencia, generalizacion, etc.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct BenchmarkNode {
    id: usize,
    tests_run: u64,
    last_score: f32,
    cumulative_score: f32,
}

impl BenchmarkNode {
    pub fn new(id: usize) -> Self {
        BenchmarkNode {
            id,
            tests_run: 0,
            last_score: 0.0,
            cumulative_score: 0.0,
        }
    }

    pub fn sample_runtime_cost(&mut self, global_fe: f32) -> String {
        self.tests_run += 1;
        self.last_score = (100.0 - global_fe.min(100.0)).max(0.0);
        self.cumulative_score = self.cumulative_score * 0.9 + self.last_score * 0.1;
        format!(
            "[BENCHMARK-AUTO] score={:.2} cumulative={:.2} tests={}",
            self.last_score, self.cumulative_score, self.tests_run
        )
    }

    pub fn benchmark_snapshot(&self) -> String {
        format!(
            "benchmark:tests:{} last:{:.3} cumulative:{:.3}",
            self.tests_run, self.last_score, self.cumulative_score
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "tests_run": self.tests_run,
            "last_score": self.last_score,
            "cumulative_score": self.cumulative_score,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.tests_run = snapshot
            .get("tests_run")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.last_score = snapshot
            .get("last_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.cumulative_score = snapshot
            .get("cumulative_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        Ok(())
    }
}

impl GARMNode for BenchmarkNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "benchmark"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        // Alta FE cuando el score es bajo (necesitamos mejorar)
        let score_deficit = (100.0 - self.last_score).max(0.0) / 100.0;
        score_deficit * 3.0 + 0.3
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.last_score, self.cumulative_score]
    }

    fn act(&mut self, ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        // Benchmark corre cada 100 ticks del contexto global
        if ctx.tick % 100 == 0 && ctx.tick > 0 {
            self.tests_run += 1;
            // Score sintetico basado en FE global (inverso)
            let score = (100.0 - ctx.global_free_energy.min(100.0)).max(0.0);
            self.last_score = score;
            self.cumulative_score = self.cumulative_score * 0.9 + score * 0.1;
            return NodeAction::Output(vec![score, self.cumulative_score]);
        }
        NodeAction::None
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost: f32 = 3.0;
        cost.min(energy_in)
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        50.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
