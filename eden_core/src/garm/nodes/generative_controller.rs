//! GARM Node: GenerativeController — Motor de acciones autonomas
//!
//! Extrae el generative controller de GarmCapabilityState como nodo independiente.
//! Legacy specialized node retained for module compatibility.
//! Runtime generation now lives in CapabilityId::GenController.

use crate::eden_garm::capabilities::corpus_massive::CorpusMassive;
use crate::eden_garm::capabilities::gen_metrics::GenMetrics;
use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct GenerativeControllerNode {
    id: usize,
    gen_metrics: GenMetrics,
    corpus: CorpusMassive,
    n_executions: u64,
    total_reward: f32,
    parse_rate_ema: f32,
}

impl GenerativeControllerNode {
    pub fn new(id: usize) -> Self {
        GenerativeControllerNode {
            id,
            gen_metrics: GenMetrics::new(),
            corpus: CorpusMassive::new(),
            n_executions: 0,
            total_reward: 0.0,
            parse_rate_ema: 0.0,
        }
    }
}

impl GARMNode for GenerativeControllerNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "generative_controller"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        // Alta FE cuando parse_rate es bajo (incertidumbre alta)
        let parse_uncertainty = 1.0 - self.gen_metrics.parse_rate();
        let reward_deficit = (10.0 - self.total_reward).max(0.0) * 0.1;
        parse_uncertainty * 2.0 + reward_deficit + 0.5
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.gen_metrics.parse_rate(),
            self.total_reward,
            self.parse_rate_ema,
        ]
    }

    fn act(&mut self, ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        // Adaptar top_k y temperatura segun parse_rate
        let parse_rate = self.gen_metrics.parse_rate();
        let temp = if parse_rate < 0.05 {
            0.2
        } else if parse_rate < 0.15 {
            0.3
        } else if parse_rate < 0.30 {
            0.5
        } else {
            0.7
        };

        // En fase 1, solo emitimos metadatos del estado del controller
        // En fase 2, aqui generariamos programs reales
        let out = vec![parse_rate, temp, self.total_reward, ctx.global_free_energy];

        if let Some(err) = prediction_error.first() {
            self.parse_rate_ema = self.parse_rate_ema * 0.95 + err.abs() * 0.05;
        }

        NodeAction::Output(out)
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost = 8.0; // generar programs es caro
        if energy_in >= cost {
            self.n_executions += 1;
        }
        cost
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        200.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
