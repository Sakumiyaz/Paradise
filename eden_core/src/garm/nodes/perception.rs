//! GARM Node: Perception — Vision, voz, NLP, percepcion unificada
//!
//! Meta-nodo que agrupa: vision, voice, nlp, perception, multimodal.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct PerceptionNode {
    id: usize,
    modalities: Vec<String>,
    binding_count: u64,
    attention_focus: String,
    sensory_surprise: f32,
}

impl PerceptionNode {
    pub fn new(id: usize) -> Self {
        PerceptionNode {
            id,
            modalities: vec!["text".into(), "vision".into(), "voice".into()],
            binding_count: 0,
            attention_focus: "none".into(),
            sensory_surprise: 1.0,
        }
    }
}

impl GARMNode for PerceptionNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "perception"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Fast
    }

    fn free_energy(&self) -> f32 {
        // Los sentidos SIEMPRE tienen sorpresa (el mundo es impredecible)
        self.sensory_surprise * 3.0 + self.binding_count as f32 * 0.01
    }

    fn predict(&mut self, ctx: &NodeContext) -> Vec<f32> {
        let sensor_norm = ctx.sensor_input.iter().map(|x| x.abs()).sum::<f32>()
            / ctx.sensor_input.len().max(1) as f32;
        vec![sensor_norm, self.sensory_surprise]
    }

    fn act(&mut self, ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.sensory_surprise = (self.sensory_surprise * 0.9 + err.abs() * 0.1).min(5.0);
        }
        // Intentar vincular inputs sensoriales
        if !ctx.sensor_input.is_empty() {
            self.binding_count += 1;
            self.attention_focus = "multimodal".into();
        }
        NodeAction::Output(ctx.sensor_input.clone())
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost: f32 = 2.5;
        cost.min(energy_in)
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        60.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
