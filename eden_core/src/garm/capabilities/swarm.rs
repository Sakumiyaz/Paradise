// EDEN GARM Swarm — Cognitive Ensemble of neural specialists
// 3 parallel networks with different architectures vote on outcomes.
// Emergence: the ensemble is more robust than any single network.

use super::neural::OnlineNetwork;

pub struct Specialist {
    pub name: &'static str,
    pub net: OnlineNetwork,
    pub recent_error: f32,
    pub target_mask: Vec<bool>,
}

pub struct CognitiveEnsemble {
    pub specialists: Vec<Specialist>,
    pub best_idx: usize,
}

impl CognitiveEnsemble {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        let mut specialists = Vec::with_capacity(3);
        // Conservative: small hidden, low lr — predicts first 3 dimensions
        specialists.push(Specialist {
            name: "conservative",
            net: OnlineNetwork::new(input_size, 6, output_size, 0.02),
            recent_error: 1.0,
            target_mask: vec![true, true, true, false, false, false],
        });
        // Balanced: medium hidden, medium lr — generalist, all dimensions
        specialists.push(Specialist {
            name: "balanced",
            net: OnlineNetwork::new(input_size, 12, output_size, 0.05),
            recent_error: 1.0,
            target_mask: vec![true, true, true, true, true, true],
        });
        // Explorer: large hidden, high lr — predicts last 3 dimensions
        specialists.push(Specialist {
            name: "explorer",
            net: OnlineNetwork::new(input_size, 20, output_size, 0.08),
            recent_error: 1.0,
            target_mask: vec![false, false, false, true, true, true],
        });
        CognitiveEnsemble {
            specialists,
            best_idx: 1,
        }
    }

    /// Ensemble prediction: weighted average of all specialists, weighted by inverse error.
    pub fn predict(&mut self, input: &[f32]) -> Vec<f32> {
        let mut outputs: Vec<Vec<f32>> = Vec::new();
        let mut weights = Vec::new();
        for s in &mut self.specialists {
            let out = s.net.predict(input);
            outputs.push(out);
            let w = 1.0 / (s.recent_error + 0.01);
            weights.push(w);
        }
        let total_w: f32 = weights.iter().sum();
        if total_w <= 0.0 || outputs.is_empty() {
            return vec![0.5f32; self.specialists[0].net.output_size];
        }
        let n_out = outputs[0].len();
        let mut ensemble = vec![0.5f32; n_out];
        for j in 0..n_out {
            let mut sum = 0.0f32;
            let mut mask_w = 0.0f32;
            for i in 0..outputs.len() {
                if self.specialists[i]
                    .target_mask
                    .get(j)
                    .copied()
                    .unwrap_or(false)
                {
                    sum += outputs[i][j] * weights[i];
                    mask_w += weights[i];
                }
            }
            if mask_w > 0.0 {
                ensemble[j] = sum / mask_w;
            }
        }
        ensemble
    }

    /// Train the specialist with lowest recent_error (competitive learning).
    pub fn train(&mut self, input: &[f32], target: &[f32]) -> f32 {
        let mut min_err = f32::MAX;
        let mut min_idx = 0usize;
        for (i, s) in self.specialists.iter().enumerate() {
            if s.recent_error < min_err {
                min_err = s.recent_error;
                min_idx = i;
            }
        }
        let loss = self.specialists[min_idx].net.train(input, target);
        self.specialists[min_idx].recent_error =
            self.specialists[min_idx].recent_error * 0.9 + loss * 0.1;
        self.best_idx = min_idx;
        loss
    }

    /// Get embedding from the currently best specialist.
    pub fn get_hidden(&self) -> Vec<f32> {
        self.specialists[self.best_idx].net.get_hidden()
    }

    pub fn status(&self) -> String {
        let names: Vec<String> = self
            .specialists
            .iter()
            .map(|s| format!("{}:{:.3}", s.name, s.recent_error))
            .collect();
        format!(
            "Swarm | best={} | specialists: {}",
            self.specialists[self.best_idx].name,
            names.join(" ")
        )
    }
}
