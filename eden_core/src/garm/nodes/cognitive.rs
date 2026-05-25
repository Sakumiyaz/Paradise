//! GARM Node: Cognitive — Razonamiento, inferencia, logica, analogia
//!
//! Meta-nodo que agrupa: inference, logic_reasoning, analogy, composition,
//  program_induction, counterfactual, causal_model.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct CognitiveNode {
    id: usize,
    inferences: u64,
    rules: u64,
    analogy_matches: u64,
    cognitive_load: f32,
}

impl CognitiveNode {
    pub fn new(id: usize) -> Self {
        CognitiveNode {
            id,
            inferences: 0,
            rules: 42, // seed
            analogy_matches: 0,
            cognitive_load: 0.0,
        }
    }
}

impl GARMNode for CognitiveNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "cognitive"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        let load_fe = self.cognitive_load * 2.0;
        let rule_complexity = (self.rules as f32).ln_1p() * 0.1;
        load_fe + rule_complexity + 0.5
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.cognitive_load,
            self.inferences as f32,
            self.rules as f32,
        ]
    }

    fn act(&mut self, _ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        if self.cognitive_load > 0.8 {
            // Descansar cognitivamente
            self.cognitive_load *= 0.5;
            return NodeAction::Output(vec![0.0, self.cognitive_load]);
        }
        // Razonar (simulado)
        self.inferences += 1;
        self.cognitive_load += 0.05;
        NodeAction::Output(vec![1.0, self.cognitive_load])
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost: f32 = 5.0;
        if energy_in < cost {
            self.cognitive_load += 0.1;
        } else {
            self.cognitive_load *= 0.98;
        }
        cost.min(energy_in)
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        70.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
