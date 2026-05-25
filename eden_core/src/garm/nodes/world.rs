//! GARM Node: World — Fisica, mundo 3D, gridworld, modelo interno
//!
//! Meta-nodo que agrupa: physics, world3d, gridworld, world_model,
//! world_model_nn, grounding, scene_parser.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct WorldNode {
    id: usize,
    objects: u64,
    collisions: u64,
    total_ke: f32,
    gravity: f32,
}

impl WorldNode {
    pub fn new(id: usize) -> Self {
        WorldNode {
            id,
            objects: 0,
            collisions: 0,
            total_ke: 0.0,
            gravity: 9.81,
        }
    }
}

impl GARMNode for WorldNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "world"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        // FE alta cuando hay muchas colisiones o energia cinetica inestable
        let chaos = self.collisions as f32 * 0.1 + self.total_ke.abs() * 0.01;
        chaos + 0.3
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.objects as f32, self.total_ke, self.collisions as f32]
    }

    fn act(&mut self, _ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        // Simular un paso de fisica
        if self.objects > 0 {
            self.total_ke *= 0.99; // friccion
        }
        NodeAction::Output(vec![self.total_ke, self.objects as f32])
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost: f32 = 4.0;
        cost.min(energy_in)
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        55.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
