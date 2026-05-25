// EDEN GARM Neural — Real online learning MLP with backprop SGD + Elman RNN + Hebbian plasticity
// Zero external dependencies. Pure Rust. Learns from system experience in real-time.

use std::f32::consts::E;

pub struct OnlineNetwork {
    pub input_size: usize,
    pub hidden_size: usize,
    pub output_size: usize,
    pub w1: Vec<Vec<f32>>, // (input + hidden) -> hidden
    pub b1: Vec<f32>,      // hidden bias
    pub w2: Vec<Vec<f32>>, // hidden -> output
    pub b2: Vec<f32>,      // output bias
    pub lr: f32,
    pub lr_hebb: f32,
    pub context: Vec<f32>,
    pub hebb: Vec<Vec<f32>>,
    pub last_hidden: Vec<f32>,
    pub last_input: Vec<f32>,
    pub last_output: Vec<f32>,
    pub last_concat: Vec<f32>,
}

impl OnlineNetwork {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize, lr: f32) -> Self {
        Self::new_with_hebb(input_size, hidden_size, output_size, lr, 0.01)
    }

    pub fn new_with_hebb(
        input_size: usize,
        hidden_size: usize,
        output_size: usize,
        lr: f32,
        lr_hebb: f32,
    ) -> Self {
        let mut w1 = vec![vec![0.0f32; input_size + hidden_size]; hidden_size];
        let mut w2 = vec![vec![0.0f32; hidden_size]; output_size];
        // Xavier-like init
        let scale1 = (2.0 / (input_size + hidden_size + hidden_size) as f32).sqrt();
        let scale2 = (2.0 / (hidden_size + output_size) as f32).sqrt();
        for i in 0..hidden_size {
            for j in 0..(input_size + hidden_size) {
                w1[i][j] = (j as f32 * 0.1 - 0.5) * scale1;
            }
        }
        for i in 0..output_size {
            for j in 0..hidden_size {
                w2[i][j] = (j as f32 * 0.1 - 0.5) * scale2;
            }
        }
        OnlineNetwork {
            input_size,
            hidden_size,
            output_size,
            w1,
            b1: vec![0.0; hidden_size],
            w2,
            b2: vec![0.0; output_size],
            lr,
            lr_hebb,
            context: vec![0.0; hidden_size],
            hebb: vec![vec![0.0f32; hidden_size]; hidden_size],
            last_hidden: vec![0.0; hidden_size],
            last_input: vec![0.0; input_size],
            last_output: vec![0.0; output_size],
            last_concat: vec![0.0; input_size + hidden_size],
        }
    }

    fn relu(x: f32) -> f32 {
        x.max(0.0)
    }
    fn relu_deriv(x: f32) -> f32 {
        if x > 0.0 {
            1.0
        } else {
            0.0
        }
    }
    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + E.powf(-x))
    }

    pub fn predict(&mut self, input: &[f32]) -> Vec<f32> {
        let mut input: Vec<f32> = input.iter().map(|&v| v.clamp(-10.0, 10.0)).collect();
        // Pad/truncate input to expected input_size to prevent OOB on the matmul below
        if input.len() < self.input_size {
            input.resize(self.input_size, 0.0);
        } else if input.len() > self.input_size {
            input.truncate(self.input_size);
        }
        self.last_input = input.clone();
        // Concatenate input + context
        let mut concat = Vec::with_capacity(self.input_size + self.hidden_size);
        concat.extend_from_slice(&input);
        concat.extend_from_slice(&self.context);
        self.last_concat = concat.clone();
        // hidden_pre = W1 @ concat + b1
        let mut hidden_pre = vec![0.0; self.hidden_size];
        for i in 0..self.hidden_size {
            let mut sum = self.b1[i];
            for j in 0..(self.input_size + self.hidden_size) {
                sum += self.w1[i][j] * concat[j];
            }
            hidden_pre[i] = sum;
        }
        // Hebbian update (Oja rule)
        for i in 0..self.hidden_size {
            for j in 0..self.hidden_size {
                let delta = self.lr_hebb
                    * (hidden_pre[i] * hidden_pre[j]
                        - self.hebb[i][j] * hidden_pre[i] * hidden_pre[i]);
                self.hebb[i][j] += delta;
            }
        }
        // Symmetrize hebb
        for i in 0..self.hidden_size {
            for j in (i + 1)..self.hidden_size {
                let avg = (self.hebb[i][j] + self.hebb[j][i]) * 0.5;
                self.hebb[i][j] = avg;
                self.hebb[j][i] = avg;
            }
        }
        // Add hebb @ hidden_pre as recurrent term, then ReLU
        let mut hidden = vec![0.0; self.hidden_size];
        for i in 0..self.hidden_size {
            let mut sum = hidden_pre[i];
            for j in 0..self.hidden_size {
                sum += self.hebb[i][j] * hidden_pre[j];
            }
            hidden[i] = Self::relu(sum);
        }
        self.last_hidden = hidden.clone();
        self.context = hidden.clone();
        // output = sigmoid(W2 @ hidden + b2)
        let mut out = vec![0.0; self.output_size];
        for i in 0..self.output_size {
            let mut sum = self.b2[i];
            for j in 0..self.hidden_size {
                sum += self.w2[i][j] * hidden[j];
            }
            out[i] = Self::sigmoid(sum);
        }
        self.last_output = out.clone();
        out
    }

    /// Get the hidden layer representation (dense embedding) for downstream use
    pub fn get_hidden(&self) -> Vec<f32> {
        self.last_hidden.clone()
    }

    /// Attention gating: elementwise multiply last_hidden by active_mask
    pub fn gate(&mut self, active_mask: &[f32]) {
        assert_eq!(active_mask.len(), self.hidden_size);
        for i in 0..self.hidden_size {
            self.last_hidden[i] *= active_mask[i];
        }
    }

    /// Online SGD backprop. target must be in [0,1] for sigmoid outputs.
    pub fn train(&mut self, input: &[f32], target: &[f32]) -> f32 {
        let out = self.predict(input);
        let mut loss = 0.0f32;
        let mut d_out = vec![0.0; self.output_size];
        for i in 0..self.output_size {
            let err = out[i] - target[i];
            loss += err * err;
            d_out[i] = err * out[i] * (1.0 - out[i]); // sigmoid derivative
        }
        // hidden gradient
        let mut d_hidden = vec![0.0; self.hidden_size];
        for j in 0..self.hidden_size {
            let mut sum = 0.0;
            for i in 0..self.output_size {
                sum += self.w2[i][j] * d_out[i];
            }
            d_hidden[j] = sum;
        }
        // Recompute pre-activation values for backprop through Elman + Hebb
        let mut z = vec![0.0; self.hidden_size];
        for i in 0..self.hidden_size {
            let mut sum = self.b1[i];
            for j in 0..(self.input_size + self.hidden_size) {
                sum += self.w1[i][j] * self.last_concat[j];
            }
            z[i] = sum;
        }
        let mut z_prime = vec![0.0; self.hidden_size];
        for i in 0..self.hidden_size {
            let mut sum = z[i];
            for j in 0..self.hidden_size {
                sum += self.hebb[i][j] * z[j];
            }
            z_prime[i] = sum;
        }
        // d_zprime = d_hidden * relu'(z_prime)
        let mut d_zprime = vec![0.0; self.hidden_size];
        for i in 0..self.hidden_size {
            d_zprime[i] = d_hidden[i] * Self::relu_deriv(z_prime[i]);
        }
        // d_z = d_zprime + hebb @ d_zprime (symmetric hebb)
        let mut d_z = vec![0.0; self.hidden_size];
        for i in 0..self.hidden_size {
            let mut sum = d_zprime[i];
            for j in 0..self.hidden_size {
                sum += self.hebb[i][j] * d_zprime[j];
            }
            d_z[i] = sum;
        }
        // update w2, b2
        for i in 0..self.output_size {
            for j in 0..self.hidden_size {
                self.w2[i][j] -= self.lr * d_out[i] * self.last_hidden[j];
            }
            self.b2[i] -= self.lr * d_out[i];
        }
        // update w1, b1 using full concatenated input
        for j in 0..self.hidden_size {
            for k in 0..(self.input_size + self.hidden_size) {
                self.w1[j][k] -= self.lr * d_z[j] * self.last_concat[k];
            }
            self.b1[j] -= self.lr * d_z[j];
        }
        loss
    }

    pub fn weights(&self) -> Vec<f32> {
        let mut w = Vec::new();
        for row in &self.w1 {
            w.extend(row);
        }
        w.extend(&self.b1);
        for row in &self.w2 {
            w.extend(row);
        }
        w.extend(&self.b2);
        w
    }

    pub fn set_weights(&mut self, w: &[f32]) {
        let mut idx = 0usize;
        for i in 0..self.hidden_size {
            for j in 0..(self.input_size + self.hidden_size) {
                if idx >= w.len() {
                    return;
                }
                self.w1[i][j] = w[idx];
                idx += 1;
            }
        }
        for i in 0..self.hidden_size {
            if idx >= w.len() {
                return;
            }
            self.b1[i] = w[idx];
            idx += 1;
        }
        for i in 0..self.output_size {
            for j in 0..self.hidden_size {
                if idx >= w.len() {
                    return;
                }
                self.w2[i][j] = w[idx];
                idx += 1;
            }
        }
        for i in 0..self.output_size {
            if idx >= w.len() {
                return;
            }
            self.b2[i] = w[idx];
            idx += 1;
        }
    }

    pub fn n_weights(&self) -> usize {
        self.hidden_size * (self.input_size + self.hidden_size)
            + self.hidden_size
            + self.output_size * self.hidden_size
            + self.output_size
    }
}
