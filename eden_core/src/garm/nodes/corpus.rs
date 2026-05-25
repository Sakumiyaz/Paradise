//! GARM Node: Corpus — Generador de entrenamiento para el transformer
//!
//! Procedura corpus con meta-curriculum adaptativo.

use crate::eden_garm::capabilities::corpus_massive::CorpusMassive;
use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct CorpusNode {
    id: usize,
    corpus: CorpusMassive,
    sentences_generated: u64,
    program_bias_history: Vec<f32>,
}

impl CorpusNode {
    pub fn new(id: usize) -> Self {
        CorpusNode {
            id,
            corpus: CorpusMassive::new(),
            sentences_generated: 0,
            program_bias_history: vec![0.5],
        }
    }
}

impl GARMNode for CorpusNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "corpus"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        // FE alta cuando no hay suficiente diversidad generada
        let diversity = self.program_bias_history.len() as f32;
        (100.0 / diversity.max(1.0)) * 0.1 + 0.5
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.sentences_generated as f32, self.corpus.program_bias]
    }

    fn act(&mut self, ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        // Adaptar program_bias segun parse_rate global (inyectado por contexto)
        let global_parse = ctx.global_free_energy; // proxy: baja FE = alto parse
        let target_bias = if global_parse < 0.5 {
            0.8
        } else if global_parse < 2.0 {
            0.6
        } else {
            0.3
        };
        self.corpus.program_bias = target_bias;
        self.program_bias_history.push(target_bias);
        if self.program_bias_history.len() > 50 {
            self.program_bias_history.remove(0);
        }
        NodeAction::Output(vec![target_bias, self.sentences_generated as f32])
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost = 4.0;
        if energy_in >= cost {
            // Generar sentences cuando hay energia
            let _sents = self.corpus.generate_n(2);
            self.sentences_generated += 2;
        }
        cost.min(energy_in)
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        30.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
