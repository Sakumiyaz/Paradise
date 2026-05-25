//! GARM Node: Memory — Memoria episodica y buffer de experiencias
//!
//! Extrae hippocampus + working memory + experience buffer como nodos GARM.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use std::collections::VecDeque;

pub struct MemoryNode {
    id: usize,
    episodes: VecDeque<String>,
    capacity: usize,
    reads: u64,
    writes: u64,
    consolidation_counter: u32,
}

impl MemoryNode {
    pub fn new(id: usize) -> Self {
        MemoryNode {
            id,
            episodes: VecDeque::with_capacity(100),
            capacity: 100,
            reads: 0,
            writes: 0,
            consolidation_counter: 0,
        }
    }

    pub fn write_episode(&mut self, content: String) {
        if self.episodes.len() >= self.capacity {
            self.episodes.pop_front();
        }
        self.episodes.push_back(content);
        self.writes += 1;
    }

    pub fn read_recent(&self, n: usize) -> Vec<String> {
        self.episodes.iter().rev().take(n).cloned().collect()
    }
}

impl GARMNode for MemoryNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "memory"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        // FE alta cuando la memoria esta llena (necesita consolidacion)
        let fullness = self.episodes.len() as f32 / self.capacity as f32;
        let access_pressure = (self.reads as f32).ln_1p() * 0.1;
        fullness * 2.0 + access_pressure + 0.3
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.episodes.len() as f32,
            self.reads as f32,
            self.writes as f32,
        ]
    }

    fn act(&mut self, ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        // Consolidar episodios cada 50 ticks
        self.consolidation_counter += 1;
        if self.consolidation_counter >= 50 {
            self.consolidation_counter = 0;
            let n = self.episodes.len();
            return NodeAction::Output(vec![n as f32, self.reads as f32, self.writes as f32]);
        }
        // Si hay output de vecinos, intentar memorizarlo
        for (src, out) in &ctx.neighbor_outputs {
            let summary = format!("src={} len={}", src, out.len());
            self.write_episode(summary);
        }
        NodeAction::None
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost: f32 = 2.0;
        cost.min(energy_in)
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        40.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
