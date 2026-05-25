// EDEN GARM — Critic Network (Actor-Critic RL)
// Estimates V(state) and computes advantage A = R + gamma*V(next) - V(curr).
// Reduces variance of policy gradients vs REINFORCE puro.

pub fn xavier_vec(rows: usize, cols: usize) -> Vec<Vec<f32>> {
    let scale = (2.0 / cols as f32).sqrt();
    let mut m = vec![vec![0.0f32; cols]; rows];
    let mut seed: u64 = 131;
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
pub struct Critic {
    pub state_dim: usize,
    pub hidden_dim: usize,
    pub w1: Vec<Vec<f32>>,
    pub b1: Vec<f32>,
    pub w2: Vec<f32>,
    pub b2: f32,
    pub gamma: f32,
    pub lr: f32,
    pub n_updates: u64,
    pub last_v: f32,
    pub td_error_ema: f32,
}

impl Critic {
    pub fn new(state_dim: usize) -> Self {
        let hidden = 64usize;
        Critic {
            state_dim,
            hidden_dim: hidden,
            w1: xavier_vec(state_dim, hidden),
            b1: vec![0.0f32; hidden],
            w2: {
                let scale = (2.0 / hidden as f32).sqrt();
                let mut v = vec![0.0f32; hidden];
                let mut seed: u64 = 131;
                for i in 0..hidden {
                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    let r = ((seed % 1000) as f32 / 1000.0 - 0.5) * 2.0;
                    v[i] = r * scale;
                }
                v
            },
            b2: 0.0,
            gamma: 0.95,
            lr: 0.005,
            n_updates: 0,
            last_v: 0.0,
            td_error_ema: 0.0,
        }
    }

    fn forward(&self, state: &[f32]) -> (f32, Vec<f32>) {
        let mut h = vec![0.0f32; self.hidden_dim];
        for j in 0..self.hidden_dim {
            let mut sum = self.b1[j];
            for i in 0..state.len().min(self.state_dim) {
                sum += state[i] * self.w1[i][j];
            }
            h[j] = sum;
        }
        relu(&mut h);
        let mut v = self.b2;
        for j in 0..self.hidden_dim {
            v += h[j] * self.w2[j];
        }
        (v, h)
    }

    /// Estimate V(s)
    pub fn value(&self, state: &[f32]) -> f32 {
        self.forward(state).0
    }

    /// Update critic with TD target. Returns TD error (advantage).
    pub fn update(&mut self, state: &[f32], reward: f32, next_state: &[f32]) -> f32 {
        let (v, h) = self.forward(state);
        let v_next = self.value(next_state);
        let target = reward + self.gamma * v_next;
        let td_err = target - v;
        self.td_error_ema = self.td_error_ema * 0.95 + td_err.abs() * 0.05;
        // Grad w2/b2
        for j in 0..self.hidden_dim {
            self.w2[j] += self.lr * td_err * h[j];
        }
        self.b2 += self.lr * td_err;
        // Grad w1/b1 (backprop through ReLU)
        for j in 0..self.hidden_dim {
            let grad_h = td_err * self.w2[j] * if h[j] > 0.0 { 1.0 } else { 0.0 };
            for i in 0..state.len().min(self.state_dim) {
                self.w1[i][j] += self.lr * grad_h * state[i];
            }
            self.b1[j] += self.lr * grad_h;
        }
        self.last_v = v;
        self.n_updates += 1;
        td_err
    }

    pub fn status(&self) -> String {
        format!(
            "Critic | dim={} | n_upd={} | last_v={:.3} | td_ema={:.3}",
            self.state_dim, self.n_updates, self.last_v, self.td_error_ema
        )
    }
}
