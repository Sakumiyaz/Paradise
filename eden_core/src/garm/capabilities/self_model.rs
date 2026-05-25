// EDEN GARM Self-Model — Functional self-prediction and prediction-error minimization
// The system predicts its own next state and the outcome of its actions.
// Prediction error drives learning in neural, morphogenesis, and evolution modules.

use super::neural::OnlineNetwork;

pub struct SelfModel {
    pub outcome_net: OnlineNetwork, // predicts [mean_success, mean_duration, mean_resources, log_var_success, log_var_duration, log_var_resources]
    pub state_net: OnlineNetwork,   // predicts next state features
    pub last_prediction: Vec<f32>,
    pub last_state: Vec<f32>,
    pub last_action: Vec<f32>,
    pub prediction_errors: Vec<f32>, // history of prediction errors
    pub total_error: f32,
    pub prediction_count: u32,
}

impl SelfModel {
    pub fn new(state_dim: usize, action_dim: usize) -> Self {
        SelfModel {
            outcome_net: OnlineNetwork::new(state_dim + action_dim, 8, 6, 0.05),
            state_net: OnlineNetwork::new(state_dim + action_dim, 8, state_dim, 0.05),
            last_prediction: Vec::new(),
            last_state: Vec::new(),
            last_action: Vec::new(),
            prediction_errors: Vec::with_capacity(1000),
            total_error: 0.0,
            prediction_count: 0,
        }
    }

    /// Predict the outcome of taking `action` from `state`.
    /// Returns [success_probability, expected_duration, expected_resource_cost].
    pub fn predict_outcome(&mut self, state: &[f32], action: &[f32]) -> Vec<f32> {
        let mut input = state.to_vec();
        input.extend(action);
        let out = self.outcome_net.predict(&input);
        self.last_prediction = out.clone();
        self.last_state = state.to_vec();
        self.last_action = action.to_vec();
        out[..3].to_vec()
    }

    /// Counterfactual rollout: simulate an action without mutating last_prediction/last_state/last_action.
    /// Returns the predicted outcome means [success, duration, resources].
    pub fn simulate(&mut self, state: &[f32], action: &[f32]) -> Vec<f32> {
        let mut input = state.to_vec();
        input.extend(action);
        let out = self.outcome_net.predict(&input);
        out[..3].to_vec()
    }

    /// Evaluate multiple actions and return the index + predicted outcome of the best one
    /// (highest predicted success probability).
    pub fn best_action(&mut self, state: &[f32], actions: &[Vec<f32>]) -> (usize, Vec<f32>) {
        let mut best_idx = 0;
        let mut best_success = -1.0f32;
        let mut best_outcome = Vec::new();
        for (i, action) in actions.iter().enumerate() {
            let outcome = self.simulate(state, action);
            let success = outcome.get(0).copied().unwrap_or(0.0);
            if success > best_success {
                best_success = success;
                best_idx = i;
                best_outcome = outcome;
            }
        }
        (best_idx, best_outcome)
    }

    /// Predict the next state after taking `action` from `state`.
    pub fn predict_next_state(&mut self, state: &[f32], action: &[f32]) -> Vec<f32> {
        let mut input = state.to_vec();
        input.extend(action);
        self.state_net.predict(&input)
    }

    /// Train outcome_net with Gaussian Negative Log-Likelihood.
    /// Assumes outcome_net output_size = 6: [mean0, mean1, mean2, log_var0, log_var1, log_var2]
    fn train_outcome_net(&mut self, input: &[f32], target: &[f32]) -> f32 {
        let out = self.outcome_net.predict(input);
        let mut loss = 0.0f32;
        let mut d_out = vec![0.0f32; 6];

        for i in 0..3 {
            let mean = out[i];
            let log_var = out[i + 3];
            let var = log_var.exp();
            let err = target[i] - mean;

            loss += 0.5 * (log_var + err * err / var);

            // Gradient w.r.t. mean (through sigmoid)
            d_out[i] = (mean - target[i]) / var;
            d_out[i] *= mean * (1.0 - mean);

            // Gradient w.r.t. log_var (through sigmoid)
            d_out[i + 3] = 0.5 * (1.0 - err * err / var);
            d_out[i + 3] *= log_var * (1.0 - log_var);
        }

        let hidden_size = self.outcome_net.hidden_size;
        let lr = self.outcome_net.lr;

        // hidden gradient: d_h = W2^T @ d_out
        let mut d_hidden = vec![0.0f32; hidden_size];
        for j in 0..hidden_size {
            let mut sum = 0.0;
            for i in 0..6 {
                sum += self.outcome_net.w2[i][j] * d_out[i];
            }
            d_hidden[j] = sum;
        }

        // Recompute pre-activation values for backprop through Elman + Hebb
        let mut z = vec![0.0f32; hidden_size];
        for i in 0..hidden_size {
            let mut sum = self.outcome_net.b1[i];
            for k in 0..self.outcome_net.last_concat.len() {
                sum += self.outcome_net.w1[i][k] * self.outcome_net.last_concat[k];
            }
            z[i] = sum;
        }
        let mut z_prime = vec![0.0f32; hidden_size];
        for i in 0..hidden_size {
            let mut sum = z[i];
            for j in 0..hidden_size {
                sum += self.outcome_net.hebb[i][j] * z[j];
            }
            z_prime[i] = sum;
        }

        // d_zprime = d_hidden * relu'(z_prime)
        let mut d_zprime = vec![0.0f32; hidden_size];
        for i in 0..hidden_size {
            d_zprime[i] = d_hidden[i] * if z_prime[i] > 0.0 { 1.0 } else { 0.0 };
        }

        // d_z = d_zprime + hebb @ d_zprime (symmetric hebb)
        let mut d_z = vec![0.0f32; hidden_size];
        for i in 0..hidden_size {
            let mut sum = d_zprime[i];
            for j in 0..hidden_size {
                sum += self.outcome_net.hebb[i][j] * d_zprime[j];
            }
            d_z[i] = sum;
        }

        // update w2, b2
        for i in 0..6 {
            for j in 0..hidden_size {
                self.outcome_net.w2[i][j] -= lr * d_out[i] * self.outcome_net.last_hidden[j];
            }
            self.outcome_net.b2[i] -= lr * d_out[i];
        }

        // update w1, b1 using full concatenated input
        for j in 0..hidden_size {
            for k in 0..self.outcome_net.last_concat.len() {
                self.outcome_net.w1[j][k] -= lr * d_z[j] * self.outcome_net.last_concat[k];
            }
            self.outcome_net.b1[j] -= lr * d_z[j];
        }

        loss
    }

    /// After observing actual outcome and next state, train both predictors.
    /// Returns the total prediction error (higher = more surprise = more tension).
    pub fn train(&mut self, actual_outcome: &[f32], actual_next_state: &[f32]) -> f32 {
        // Guard: if predict_outcome was never called, last_state/last_action are empty.
        // Skip training instead of panicking inside the neural net.
        if self.last_state.is_empty() || self.last_action.is_empty() {
            return 0.0;
        }
        let mut input = self.last_state.clone();
        input.extend(&self.last_action);
        let loss_out = self.train_outcome_net(&input, actual_outcome);
        let loss_state = self.state_net.train(&input, actual_next_state);
        let total_err = loss_out + loss_state;
        self.total_error += total_err;
        self.prediction_count += 1;
        if self.prediction_errors.len() >= 1000 {
            self.prediction_errors.remove(0);
        }
        self.prediction_errors.push(total_err);
        total_err
    }

    pub fn mean_error(&self) -> f32 {
        if self.prediction_count == 0 {
            return 0.0;
        }
        self.total_error / self.prediction_count as f32
    }

    pub fn recent_error(&self, n: usize) -> f32 {
        let n = n.min(self.prediction_errors.len());
        if n == 0 {
            return 0.0;
        }
        self.prediction_errors[self.prediction_errors.len() - n..]
            .iter()
            .sum::<f32>()
            / n as f32
    }

    /// Returns the predicted variance [var_success, var_duration, var_resources] of the last prediction.
    pub fn uncertainty(&self) -> Vec<f32> {
        if self.last_prediction.len() < 6 {
            return vec![1.0f32; 3];
        }
        (0..3).map(|i| self.last_prediction[i + 3].exp()).collect()
    }

    /// The system can answer "what am I doing and why?" using the self-model.
    pub fn explain_last_action(&self) -> String {
        if self.last_prediction.is_empty() {
            return "No self-model prediction recorded yet.".to_string();
        }
        let unc = self.uncertainty();
        format!("SelfModel | Predicted success={:.2} | duration={:.2} | resource={:.2} | var=[{:.3},{:.3},{:.3}] | mean_error={:.4} | recent_err={:.4}",
            self.last_prediction.get(0).unwrap_or(&0.0),
            self.last_prediction.get(1).unwrap_or(&0.0),
            self.last_prediction.get(2).unwrap_or(&0.0),
            unc.get(0).unwrap_or(&1.0),
            unc.get(1).unwrap_or(&1.0),
            unc.get(2).unwrap_or(&1.0),
            self.mean_error(),
            self.recent_error(10))
    }

    pub fn status(&self) -> String {
        format!(
            "SelfModel | predictions: {} | mean_err: {:.4} | recent_err: {:.4}",
            self.prediction_count,
            self.mean_error(),
            self.recent_error(10)
        )
    }
}
