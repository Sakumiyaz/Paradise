// EDEN GARM — Neural World Model (differentiable transition model)
// Predice s_{t+1} = f(s_t, a_t). Entrenado end-to-end con MSE.
// Habilita planificación en imaginación (rollouts sin actuar en el mundo real).

pub fn xavier(rows: usize, cols: usize) -> Vec<Vec<f32>> {
    let scale = (2.0 / cols as f32).sqrt();
    let mut m = vec![vec![0.0f32; cols]; rows];
    let mut seed: u64 = 999;
    for i in 0..rows {
        for j in 0..cols {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let r = ((seed % 1000) as f32 / 1000.0 - 0.5) * 2.0;
            m[i][j] = r * scale;
        }
    }
    m
}

fn relu(v: &mut [f32]) {
    for x in v.iter_mut() {
        if *x < 0.0 {
            *x = 0.0;
        }
    }
}

#[derive(Clone, Debug)]
pub struct NeuralWorldModel {
    pub state_dim: usize,
    pub action_dim: usize,
    pub hidden_dim: usize,
    pub w1: Vec<Vec<f32>>,
    pub b1: Vec<f32>,
    pub w2: Vec<Vec<f32>>,
    pub b2: Vec<f32>,
    pub lr: f32,
    pub n_train: u64,
    pub last_mse: f32,
    pub pred_err_ema: f32,
}

impl NeuralWorldModel {
    pub fn new(state_dim: usize, action_dim: usize) -> Self {
        let hidden = 128usize;
        let input_dim = state_dim + action_dim;
        NeuralWorldModel {
            state_dim,
            action_dim,
            hidden_dim: hidden,
            w1: xavier(input_dim, hidden),
            b1: vec![0.0f32; hidden],
            w2: xavier(hidden, state_dim),
            b2: vec![0.0f32; state_dim],
            lr: 0.001,
            n_train: 0,
            last_mse: 0.0,
            pred_err_ema: 1.0,
        }
    }

    fn forward(&self, state: &[f32], action: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut h = vec![0.0f32; self.hidden_dim];
        for j in 0..self.hidden_dim {
            let mut sum = self.b1[j];
            for i in 0..state.len().min(self.state_dim) {
                sum += state[i] * self.w1[i][j];
            }
            for i in 0..action.len().min(self.action_dim) {
                sum += action[i] * self.w1[self.state_dim + i][j];
            }
            h[j] = sum;
        }
        relu(&mut h);
        let mut out = vec![0.0f32; self.state_dim];
        for j in 0..self.state_dim {
            let mut sum = self.b2[j];
            for i in 0..self.hidden_dim {
                sum += h[i] * self.w2[i][j];
            }
            out[j] = sum;
        }
        (out, h)
    }

    /// Predict next state.
    pub fn predict(&self, state: &[f32], action: &[f32]) -> Vec<f32> {
        self.forward(state, action).0
    }

    /// Train one step. Returns MSE.
    pub fn train_step(&mut self, state: &[f32], action: &[f32], next_state: &[f32]) -> f32 {
        let (pred, h) = self.forward(state, action);
        let mut mse = 0.0f32;
        let mut delta = vec![0.0f32; self.state_dim];
        for j in 0..self.state_dim {
            let err = next_state[j] - pred[j];
            delta[j] = err;
            mse += err * err;
        }
        mse /= self.state_dim.max(1) as f32;
        self.pred_err_ema = self.pred_err_ema * 0.95 + mse * 0.05;
        self.last_mse = mse;
        // Backprop w2
        for i in 0..self.hidden_dim {
            for j in 0..self.state_dim {
                self.w2[i][j] += self.lr * delta[j] * h[i];
            }
        }
        for j in 0..self.state_dim {
            self.b2[j] += self.lr * delta[j];
        }
        // Backprop w1
        for k in 0..self.hidden_dim {
            let grad_h: f32 = (0..self.state_dim)
                .map(|j| delta[j] * self.w2[k][j])
                .sum::<f32>()
                * if h[k] > 0.0 { 1.0 } else { 0.0 };
            for i in 0..state.len().min(self.state_dim) {
                self.w1[i][k] += self.lr * grad_h * state[i];
            }
            for i in 0..action.len().min(self.action_dim) {
                self.w1[self.state_dim + i][k] += self.lr * grad_h * action[i];
            }
            self.b1[k] += self.lr * grad_h;
        }
        self.n_train += 1;
        mse
    }

    /// Rollout: simulate N steps from initial state + action sequence.
    pub fn rollout(&self, init_state: &[f32], actions: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let mut states = vec![init_state.to_vec()];
        for a in actions {
            let s_next = self.predict(&states.last().unwrap(), a);
            states.push(s_next);
        }
        states
    }

    pub fn status(&self) -> String {
        format!(
            "WorldModelNN | s={} a={} | n_train={} | mse={:.4} | ema={:.4}",
            self.state_dim, self.action_dim, self.n_train, self.last_mse, self.pred_err_ema
        )
    }
}
