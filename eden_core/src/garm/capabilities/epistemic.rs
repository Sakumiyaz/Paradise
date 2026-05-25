// EDEN GARM EpistemicConfidence — Confianza epistémica por creencia (Agujero 10)
// Cada módulo trackea la confianza de sus outputs.
// Permite saber qué ignorar cuando hay conflicto.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BeliefConfidence {
    pub confidence: f32,
    pub n_observations: u32,
    pub last_error: f32,
    pub error_ema: f32,
}

#[derive(Clone, Debug)]
pub struct EpistemicConfidence {
    pub beliefs: HashMap<String, BeliefConfidence>,
    pub default_confidence: f32,
    pub global_uncertainty: f32,
}

impl EpistemicConfidence {
    pub fn new() -> Self {
        EpistemicConfidence {
            beliefs: HashMap::new(),
            default_confidence: 0.5,
            global_uncertainty: 1.0,
        }
    }

    /// Register a belief with its confidence.
    pub fn report(&mut self, source: &str, confidence: f32, error: f32) {
        let entry = self
            .beliefs
            .entry(source.to_string())
            .or_insert(BeliefConfidence {
                confidence: self.default_confidence,
                n_observations: 0,
                last_error: 0.0,
                error_ema: 0.5,
            });
        entry.confidence = (entry.confidence * 0.9 + confidence * 0.1).clamp(0.0, 1.0);
        entry.last_error = error;
        entry.error_ema = entry.error_ema * 0.9 + error * 0.1;
        entry.n_observations += 1;
        // Update global uncertainty
        let avg_error: f32 = self.beliefs.values().map(|b| b.error_ema).sum::<f32>()
            / self.beliefs.len().max(1) as f32;
        self.global_uncertainty = avg_error;
    }

    /// Get confidence for a specific source.
    pub fn get(&self, source: &str) -> f32 {
        self.beliefs
            .get(source)
            .map(|b| b.confidence)
            .unwrap_or(self.default_confidence)
    }

    /// Returns sources sorted by confidence descending.
    pub fn ranked_sources(&self) -> Vec<(String, f32)> {
        let mut v: Vec<(String, f32)> = self
            .beliefs
            .iter()
            .map(|(k, b)| (k.clone(), b.confidence))
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v
    }

    pub fn status(&self) -> String {
        let sources = self.beliefs.len();
        let top = self.ranked_sources().first().cloned();
        format!(
            "Epistemic | sources={} | global_unc={:.2} | top={}",
            sources,
            self.global_uncertainty,
            top.map(|(s, c)| format!("{}:{:.2}", s, c))
                .unwrap_or_else(|| "none".to_string())
        )
    }
}
