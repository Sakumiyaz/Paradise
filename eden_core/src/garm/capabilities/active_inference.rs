// EDEN GARM — Active Inference / Free Energy Principle (FEP)
// Minimiza Free Energy F = E_q[ln q(s) - ln p(o,s)]
// Simplificado: F ≈ pred_error + complexity (divergencia de KL sobre beliefs)
// El sistema elige acciones que minimicen F esperada.

#[derive(Clone, Debug)]
pub struct ActiveInference {
    pub beliefs: Vec<f32>, // posterior q(s)
    pub prior: Vec<f32>,   // p(s)
    pub sensory_precision: f32,
    pub complexity_weight: f32,
    pub n_steps: u64,
    pub last_free_energy: f32,
    pub action_preferences: Vec<f32>,
}

impl ActiveInference {
    pub fn new(state_dim: usize) -> Self {
        let prior = vec![0.0f32; state_dim];
        ActiveInference {
            beliefs: vec![0.0f32; state_dim],
            prior,
            sensory_precision: 1.0,
            complexity_weight: 0.5,
            n_steps: 0,
            last_free_energy: 0.0,
            action_preferences: Vec::new(),
        }
    }

    /// Update posterior given observation. Simplified variational update.
    pub fn perceive(&mut self, observation: &[f32]) {
        for i in 0..self.beliefs.len().min(observation.len()) {
            let prediction_error = observation[i] - self.beliefs[i];
            // Bayesian update: belief += precision * prediction_error
            self.beliefs[i] += self.sensory_precision * prediction_error * 0.1;
        }
        self.n_steps += 1;
    }

    /// Compute variational free energy: accuracy - complexity
    pub fn free_energy(&self, observation: &[f32]) -> f32 {
        let mut accuracy = 0.0f32;
        let mut complexity = 0.0f32;
        for i in 0..self.beliefs.len().min(observation.len()) {
            let pe = observation[i] - self.beliefs[i];
            accuracy += -0.5 * pe * pe; // log likelihood (Gaussian)
            let d = self.beliefs[i] - self.prior[i];
            complexity += 0.5 * d * d; // KL divergence from prior
        }
        let f = -accuracy + self.complexity_weight * complexity;
        f
    }

    /// Choose action that minimizes expected free energy.
    /// Given candidate next states, pick the one with lowest F.
    pub fn infer_action(&mut self, candidate_states: &[Vec<f32>]) -> Option<usize> {
        let mut best_idx = None;
        let mut min_fe = f32::INFINITY;
        for (i, cand) in candidate_states.iter().enumerate() {
            let fe = self.free_energy(cand);
            if fe < min_fe {
                min_fe = fe;
                best_idx = Some(i);
            }
        }
        self.last_free_energy = min_fe;
        if let Some(idx) = best_idx {
            self.action_preferences = candidate_states[idx].clone();
        }
        best_idx
    }

    /// Update prior toward current beliefs (slow drift, as in hierarchical models).
    pub fn update_prior(&mut self) {
        for i in 0..self.prior.len() {
            self.prior[i] += 0.01 * (self.beliefs[i] - self.prior[i]);
        }
    }

    pub fn status(&self) -> String {
        format!(
            "FEP | steps={} | F={:.3} | prec={:.1} | compl_w={:.1}",
            self.n_steps, self.last_free_energy, self.sensory_precision, self.complexity_weight
        )
    }
}
