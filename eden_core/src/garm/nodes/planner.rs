//! GARM Node: Planner — Planificacion de objetivos y ejecucion
//!
//! Extrae el planner de GarmCapabilityState como nodo independiente.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct PlannerNode {
    id: usize,
    horizon: usize,
    simulations: u64,
    best_score: f32,
    active_goals: Vec<String>,
}

impl PlannerNode {
    pub fn new(id: usize) -> Self {
        PlannerNode {
            id,
            horizon: 3,
            simulations: 0,
            best_score: 0.0,
            active_goals: Vec::new(),
        }
    }

    pub fn add_goal(&mut self, goal: String) {
        if !self.active_goals.contains(&goal) {
            self.active_goals.push(goal);
        }
    }
}

impl GARMNode for PlannerNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "planner"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        // FE alta cuando hay muchos goals pendientes y baja simulacion
        let goal_pressure = self.active_goals.len() as f32 * 0.5;
        let simulation_deficit = (10.0 - self.simulations as f32).max(0.0) * 0.1;
        goal_pressure + simulation_deficit + 0.4
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.best_score,
            self.active_goals.len() as f32,
            self.simulations as f32,
        ]
    }

    fn act(&mut self, _ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        if !self.active_goals.is_empty() {
            // Simular un plan (placeholder)
            self.simulations += 1;
            self.best_score = (self.best_score * 0.9 + 0.05).min(1.0);
            return NodeAction::Output(vec![self.best_score, self.active_goals.len() as f32]);
        }
        NodeAction::None
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost: f32 = 6.0;
        cost.min(energy_in)
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        80.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
