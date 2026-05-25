//! GARM Node: Social — Multi-agente, enjambre, teoria de la mente
//!
//! Meta-nodo que agrupa: multi_agent, swarm, theory_of_mind, agent_mesh,
//! social_complex, society.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct SocialNode {
    id: usize,
    agents: Vec<String>,
    broadcasts: u64,
    negotiations: u64,
    social_entropy: f32,
}

impl SocialNode {
    pub fn new(id: usize) -> Self {
        SocialNode {
            id,
            agents: vec!["eden".into()],
            broadcasts: 0,
            negotiations: 0,
            social_entropy: 0.0,
        }
    }
}

impl GARMNode for SocialNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "social"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        let coordination_cost = self.agents.len() as f32 * 0.3;
        let entropy_penalty = self.social_entropy * 2.0;
        coordination_cost + entropy_penalty + 0.4
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.agents.len() as f32,
            self.broadcasts as f32,
            self.social_entropy,
        ]
    }

    fn act(&mut self, _ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        if self.agents.len() < 3 && self.broadcasts % 10 == 0 {
            // Simular descubrimiento de nuevo agente
            let new_agent = format!("agent_{}", self.agents.len());
            self.agents.push(new_agent);
        }
        self.broadcasts += 1;
        NodeAction::Output(vec![self.agents.len() as f32, self.broadcasts as f32])
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost: f32 = 3.0;
        self.social_entropy *= 0.99;
        cost.min(energy_in)
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        45.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
