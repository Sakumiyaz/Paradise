// EDEN GARM ExplorationEngine — Exploracion dirigida por incertidumbre (Agujero 5)
// Prioriza donde el sistema mas necesita datos, no aleatorio uniforme.

#[derive(Clone, Debug)]
pub struct ExplorationEngine {
    pub transformer_entropy_threshold: f32,
    pub concept_count_threshold: u32,
    pub bus_variance_threshold: f32,
    pub n_exploration_directed: u64,
    pub n_exploration_random: u64,
    pub last_strategy: String,
}

impl ExplorationEngine {
    pub fn new() -> Self {
        ExplorationEngine {
            transformer_entropy_threshold: 1.5,
            concept_count_threshold: 3,
            bus_variance_threshold: 0.1,
            n_exploration_directed: 0,
            n_exploration_random: 0,
            last_strategy: "init".to_string(),
        }
    }

    /// Compute entropy of a probability distribution.
    pub fn entropy(probs: &[f32]) -> f32 {
        let mut h = 0.0f32;
        for &p in probs {
            if p > 1e-8 {
                h -= p * p.ln();
            }
        }
        h
    }

    fn _softmax_local(x: &mut [f32]) {
        let max = x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let mut sum = 0.0f32;
        for v in x.iter_mut() {
            *v = (*v - max).exp();
            sum += *v;
        }
        let denom = sum.max(1e-8);
        for v in x.iter_mut() {
            *v /= denom;
        }
    }

    /// Compute variance of a vector.
    pub fn variance(v: &[f32]) -> f32 {
        if v.is_empty() {
            return 0.0;
        }
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / v.len() as f32;
        var
    }

    /// Score a corpus sentence by how much it would help the system.
    /// Higher score = more valuable to process.
    pub fn score_sentence(&self, sentence: &str, vocab_size: usize) -> f32 {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }
        let unique_ratio = words.len() as f32 / words.len().max(1) as f32;
        let length_bonus = (words.len() as f32 / 10.0).min(1.0);
        let rarity = if vocab_size > 0 {
            1.0 / (vocab_size as f32 / 1000.0 + 1.0)
        } else {
            1.0
        };
        unique_ratio * 0.3 + length_bonus * 0.3 + rarity * 0.4
    }

    /// Decide exploration strategy based on system state.
    pub fn choose_strategy(
        &mut self,
        transformer_entropy: f32,
        min_concept_count: u32,
        bus_variance: f32,
        pred_error: f32,
    ) -> ExplorationStrategy {
        let mut reasons = Vec::new();
        if transformer_entropy > self.transformer_entropy_threshold {
            reasons.push("high_transformer_entropy");
        }
        if min_concept_count < self.concept_count_threshold {
            reasons.push("low_concept_count");
        }
        if bus_variance > self.bus_variance_threshold {
            reasons.push("high_bus_variance");
        }
        if pred_error > 0.1 {
            reasons.push("high_prediction_error");
        }
        if reasons.is_empty() {
            self.n_exploration_random += 1;
            self.last_strategy = "random".to_string();
            ExplorationStrategy::RandomUniform
        } else {
            self.n_exploration_directed += 1;
            self.last_strategy = reasons.join(",");
            ExplorationStrategy::Directed {
                reasons: reasons.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    pub fn status(&self) -> String {
        format!(
            "Exploration | directed={} | random={} | last={} | entropy_thr={:.1} | count_thr={}",
            self.n_exploration_directed,
            self.n_exploration_random,
            self.last_strategy,
            self.transformer_entropy_threshold,
            self.concept_count_threshold
        )
    }
}

#[derive(Clone, Debug)]
pub enum ExplorationStrategy {
    RandomUniform,
    Directed { reasons: Vec<String> },
}
