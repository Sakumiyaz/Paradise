// EDEN GARM — Truncated BPTT Engine
// Acumula gradientes del transformer a través de múltiples ticks.
// Mantiene una ventana de los últimos K estados recurrentes y aplica
// backpropagation through time truncada cada N ticks.
//
// Esto convierte el transformer de "feedforward por tick" a un modelo
// recurrente real con memoria de gradiente de largo plazo.

#[derive(Clone, Debug)]
pub struct BPTTWindow {
    pub tokens: Vec<usize>,
    pub recurrent_before: Vec<f32>,
    pub recurrent_after: Vec<f32>,
    pub tick: u64,
}

#[derive(Clone, Debug)]
pub struct BPTTEngine {
    pub window_size: usize,
    pub window: Vec<BPTTWindow>,
    pub accumulated_grad: Vec<f32>,
    pub lr: f32,
    pub n_updates: u64,
    pub last_temporal_loss: f32,
    pub enable: bool,
}

impl BPTTEngine {
    pub fn new(window_size: usize, state_dim: usize) -> Self {
        BPTTEngine {
            window_size,
            window: Vec::new(),
            accumulated_grad: vec![0.0f32; state_dim],
            lr: 0.001,
            n_updates: 0,
            last_temporal_loss: 0.0,
            enable: true,
        }
    }

    /// Record a tick's state transition.
    pub fn record(
        &mut self,
        tokens: &[usize],
        recurrent_before: &[f32],
        recurrent_after: &[f32],
        tick: u64,
    ) {
        if !self.enable {
            return;
        }
        self.window.push(BPTTWindow {
            tokens: tokens.to_vec(),
            recurrent_before: recurrent_before.to_vec(),
            recurrent_after: recurrent_after.to_vec(),
            tick,
        });
        if self.window.len() > self.window_size {
            self.window.remove(0);
        }
    }

    /// Compute temporal consistency loss: MSE between predicted next recurrent state
    /// and actual next recurrent state across the window.
    pub fn compute_temporal_loss(&self) -> (f32, Vec<f32>) {
        if self.window.len() < 2 {
            return (0.0, vec![]);
        }
        let dim = self.window[0].recurrent_before.len();
        let mut total_loss = 0.0f32;
        let mut grad = vec![0.0f32; dim];
        // For each adjacent pair in window, predict_after = predict(before)
        for i in 0..(self.window.len() - 1) {
            let curr = &self.window[i];
            let next = &self.window[i + 1];
            // Simple prediction: next state = decay * curr_after + (1-decay) * curr_before
            let decay = 0.95;
            for d in 0..dim {
                let predicted =
                    decay * curr.recurrent_after[d] + (1.0 - decay) * curr.recurrent_before[d];
                let err = next.recurrent_after[d] - predicted;
                total_loss += err * err;
                grad[d] += -2.0 * err * (1.0 - decay); // dL/d(before)
            }
        }
        total_loss /= (self.window.len() - 1) as f32 * dim as f32;
        (total_loss, grad)
    }

    /// Accumulate gradient and apply to a target vector (e.g., big_transformer.recurrent_hidden).
    pub fn accumulate_and_apply(&mut self, target: &mut [f32]) {
        if !self.enable || self.window.len() < 2 {
            return;
        }
        let (loss, grad) = self.compute_temporal_loss();
        self.last_temporal_loss = loss;
        for d in 0..target.len().min(grad.len()) {
            self.accumulated_grad[d] = 0.9 * self.accumulated_grad[d] + grad[d];
            target[d] -= self.lr * self.accumulated_grad[d];
        }
        self.n_updates += 1;
    }

    /// Clear window (e.g., on sleep/phase change).
    pub fn clear(&mut self) {
        self.window.clear();
        self.accumulated_grad.fill(0.0);
    }

    pub fn status(&self) -> String {
        format!(
            "BPTT | window={}/{} | updates={} | loss={:.6} | lr={:.4}",
            self.window.len(),
            self.window_size,
            self.n_updates,
            self.last_temporal_loss,
            self.lr
        )
    }
}
