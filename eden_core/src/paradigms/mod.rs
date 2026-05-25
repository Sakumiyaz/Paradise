// paradigms/mod.rs — 43 paradigmas de IA en Rust puro
// Cada archivo contiene implementaciones al techo Pareto

pub mod agents;
pub mod autograd;
pub mod classic;
pub mod deep;
pub mod distributed;
pub mod frontier;
pub mod generative;
pub mod graph_v8;
pub mod learning;
pub mod models;
pub mod parser;
pub mod reasoning;
pub mod systems;

use std::collections::HashMap;

/// Cross-paradigm signals that flow between paradigms and influence EDEN's decisions.
/// Built during paradigm_tick, consumed to modify meta-params, models, crawl strategy, etc.
#[derive(Default, Clone)]
pub struct ParadigmSignals {
    // ── Embeddings (produced by GNN, consumed by RAG, Siamese, FewShot...) ──
    pub node_embeddings: Vec<Vec<f32>>,
    pub svd_embeddings: Vec<Vec<f32>>,
    pub cooc_matrix: Vec<Vec<f32>>,
    // ── Edge predictions (multi-paradigm consensus) ──
    pub edge_trust: HashMap<(usize, usize), f32>,
    pub novel_edges: Vec<(usize, usize, f32)>,
    pub contradicted_edges: Vec<(usize, usize)>,
    // ── Source trust (Bayesian updates) ──
    pub source_scores: HashMap<String, f32>,
    pub source_bf: HashMap<String, f32>, // Bayes Factors
    // ── Decision signals ──
    pub crawl_recommendations: Vec<(String, f32)>, // (URL, priority)
    pub sleep_recommendation: f32,                 // 0..1, higher = sleep more
    pub prune_threshold: f32,                      // 0..1
    pub explore_rate: f32,                         // 0..1
    pub pause_duration: u64,                       // cycles to pause
    // ── Meta-parameter adjustments ──
    pub cooc_boost: f32,
    pub embed_confidence: f32,
    pub random_pages: f32,
    pub learning_rate_factor: f32,
    // ── Model updates (name → weights) ──
    pub model_updates: HashMap<String, Vec<f32>>,
    // ── Novel concepts ──
    pub inferred_rules: Vec<String>,
    pub synthesized_templates: Vec<(String, f32)>,
    // ── Training signals ──
    pub prediction_targets: Vec<(Vec<f32>, Vec<f32>)>, // (features, targets) for Predictor
    pub edge_scorer_examples: Vec<(Vec<f32>, f32)>,    // for EdgeScorer model
    pub oracle_examples: Vec<(Vec<f32>, f32)>,         // for DeathOracle
    pub emotion_examples: Vec<(Vec<f32>, Vec<f32>)>,   // for EmotionModel
    // ── Paradigm health ──
    pub activations: Vec<String>, // which paradigms had meaningful output this tick
    pub warnings: Vec<String>,
}

impl ParadigmSignals {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn blend_embeddings(&mut self, alpha: f32) {
        if self.node_embeddings.len() != self.svd_embeddings.len() {
            return;
        }
        for i in 0..self.node_embeddings.len() {
            for d in 0..self.node_embeddings[i]
                .len()
                .min(self.svd_embeddings[i].len())
            {
                self.node_embeddings[i][d] =
                    self.node_embeddings[i][d] * alpha + self.svd_embeddings[i][d] * (1.0 - alpha);
            }
        }
    }
    pub fn top_edges(&self, top_k: usize) -> Vec<(usize, usize, f32)> {
        let mut v: Vec<_> = self
            .edge_trust
            .iter()
            .map(|((a, b), &c)| (*a, *b, c))
            .collect();
        v.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        v.truncate(top_k);
        v
    }
    pub fn top_crawl(&self, n: usize) -> Vec<String> {
        let mut v = self.crawl_recommendations.clone();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        v.truncate(n);
        v.iter().map(|(u, _)| u.clone()).collect()
    }
}
