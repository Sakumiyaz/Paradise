//! GARM Node: Evolution — Algoritmos geneticos, auto-modificacion, open-endedness
//!
//! Meta-nodo que agrupa: evolution, meta_evolution, self_modification,
//! continual_learning, mdl_pruner, meta_learning.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct EvolutionNode {
    id: usize,
    generations: u64,
    mutations: u64,
    fitness_ema: f32,
    diversity: f32,
}

impl EvolutionNode {
    pub fn new(id: usize) -> Self {
        EvolutionNode {
            id,
            generations: 0,
            mutations: 0,
            fitness_ema: 0.5,
            diversity: 1.0,
        }
    }
}

impl GARMNode for EvolutionNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "evolution"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Evolutionary
    }

    fn free_energy(&self) -> f32 {
        // FE alta cuando fitness estanca o diversidad baja
        let stagnation = (0.9 - self.fitness_ema).max(0.0) * 5.0;
        let diversity_cost = (1.0 - self.diversity).max(0.0) * 3.0;
        stagnation + diversity_cost + 0.5
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.fitness_ema, self.diversity, self.generations as f32]
    }

    fn act(&mut self, ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        self.generations += 1;
        if ctx.tick % 500 == 0 && ctx.tick > 0 {
            // Evento evolutivo: mutar
            self.mutations += 1;
            self.diversity = (self.diversity + 0.1).min(1.0);
            return NodeAction::Output(vec![1.0, self.fitness_ema, self.diversity]);
        }
        // Adaptar fitness_ema lentamente
        self.fitness_ema = self.fitness_ema * 0.995 + 0.5 * 0.005;
        NodeAction::Output(vec![0.0, self.fitness_ema])
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost: f32 = 7.0; // evolucionar es caro
        cost.min(energy_in)
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        150.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
