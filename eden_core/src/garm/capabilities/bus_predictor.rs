// EDEN GARM BusPredictor — Modelo de transicion del bus (Agujero 4)
// Predice bus[t+1] dado bus[t] + instruccion ejecutada.
// Red online pequena: input=bus_dim + instr_dim, hidden=64, output=bus_dim

#[derive(Clone, Debug)]
pub struct BusPredictor {
    pub bus_dim: usize,
    pub instr_dim: usize,
    pub hidden_dim: usize,
    pub w1: Vec<Vec<f32>>, // (bus+instr) x hidden
    pub b1: Vec<f32>,
    pub w2: Vec<Vec<f32>>, // hidden x bus
    pub b2: Vec<f32>,
    pub lr: f32,
    pub n_train: u64,
    pub pred_error: f32,
    pub pred_error_ema: f32,
}

fn xavier(rows: usize, cols: usize) -> Vec<Vec<f32>> {
    let scale = (2.0 / cols as f32).sqrt();
    let mut m = vec![vec![0.0f32; cols]; rows];
    let mut seed: u64 = 777;
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

impl BusPredictor {
    pub fn new(bus_dim: usize, instr_dim: usize) -> Self {
        let hidden_dim = 64usize;
        let input_dim = bus_dim + instr_dim;
        BusPredictor {
            bus_dim,
            instr_dim,
            hidden_dim,
            w1: xavier(input_dim, hidden_dim),
            b1: vec![0.0f32; hidden_dim],
            w2: xavier(hidden_dim, bus_dim),
            b2: vec![0.0f32; bus_dim],
            lr: 0.01,
            n_train: 0,
            pred_error: 0.0,
            pred_error_ema: 0.5,
        }
    }

    /// Encode instruction as one-hot-ish vector.
    pub fn encode_instruction(&self, instr_type: usize, n_types: usize) -> Vec<f32> {
        let mut v = vec![0.0f32; self.instr_dim];
        if n_types > 0 && self.instr_dim >= n_types {
            let idx = instr_type % n_types;
            v[idx] = 1.0;
        }
        v
    }

    fn forward(&self, input: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut h = vec![0.0f32; self.hidden_dim];
        for j in 0..self.hidden_dim {
            let mut sum = self.b1[j];
            for i in 0..input.len() {
                sum += input[i] * self.w1[i][j];
            }
            h[j] = sum;
        }
        relu(&mut h);
        let mut out = vec![0.0f32; self.bus_dim];
        for j in 0..self.bus_dim {
            let mut sum = self.b2[j];
            for i in 0..self.hidden_dim {
                sum += h[i] * self.w2[i][j];
            }
            out[j] = sum;
        }
        (out, h)
    }

    /// Predict next bus state given current bus + instruction.
    pub fn predict(&self, bus: &[f32], instr_vec: &[f32]) -> Vec<f32> {
        let mut input = bus.iter().copied().take(self.bus_dim).collect::<Vec<f32>>();
        input.extend_from_slice(instr_vec);
        self.forward(&input).0
    }

    /// Train one step: compare prediction with actual next bus state.
    pub fn train_step(&mut self, bus: &[f32], instr_vec: &[f32], next_bus: &[f32]) -> f32 {
        let mut input = bus.iter().copied().take(self.bus_dim).collect::<Vec<f32>>();
        input.extend_from_slice(instr_vec);
        let (pred, h) = self.forward(&input);
        // MSE loss
        let mut loss = 0.0f32;
        let mut d_out = vec![0.0f32; self.bus_dim];
        for j in 0..self.bus_dim {
            let err = pred[j] - next_bus[j];
            loss += err * err;
            d_out[j] = 2.0 * err;
        }
        // Backprop through w2/b2
        let mut d_h = vec![0.0f32; self.hidden_dim];
        for i in 0..self.hidden_dim {
            let mut grad = 0.0f32;
            for j in 0..self.bus_dim {
                grad += d_out[j] * self.w2[i][j];
                self.w2[i][j] -= self.lr * d_out[j] * h[i];
            }
            d_h[i] = grad;
        }
        for j in 0..self.bus_dim {
            self.b2[j] -= self.lr * d_out[j];
        }
        // ReLU derivative
        for i in 0..self.hidden_dim {
            if h[i] <= 0.0 {
                d_h[i] = 0.0;
            }
        }
        // Backprop through w1/b1
        for i in 0..input.len() {
            for j in 0..self.hidden_dim {
                self.w1[i][j] -= self.lr * d_h[j] * input[i];
            }
        }
        for j in 0..self.hidden_dim {
            self.b1[j] -= self.lr * d_h[j];
        }
        self.n_train += 1;
        let mse = loss / self.bus_dim as f32;
        self.pred_error = mse.sqrt();
        self.pred_error_ema = self.pred_error_ema * 0.99 + self.pred_error * 0.01;
        mse
    }

    pub fn status(&self) -> String {
        format!(
            "BusPredictor | dim={} | hidden={} | train={} | pred_err={:.4} | ema={:.4}",
            self.bus_dim, self.hidden_dim, self.n_train, self.pred_error, self.pred_error_ema
        )
    }
}
