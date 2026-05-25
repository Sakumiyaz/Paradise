// EDEN GARM — Recurrent State (S4-like simplified)
// Un estado oculto persistente entre ticks que evoluciona linealmente.
// Es la "memoria de corto plazo" del sistema: no se reinicia cada tick.
// d_h(t+1) = A * h(t) + B * x(t)  (simplificado como decay + input)

#[derive(Clone, Debug)]
pub struct RecurrentState {
    pub dim: usize,
    pub state: Vec<f32>,
    pub decay: f32,      // A diagonal: how much state persists
    pub input_gain: f32, // B: how much input affects state
    pub n_steps: u64,
    pub mean_activation: f32,
}

impl RecurrentState {
    pub fn new(dim: usize) -> Self {
        RecurrentState {
            dim,
            state: vec![0.0f32; dim],
            decay: 0.95,
            input_gain: 0.1,
            n_steps: 0,
            mean_activation: 0.0,
        }
    }

    /// Step the recurrent state with new input.
    pub fn step(&mut self, input: &[f32]) {
        for i in 0..self.dim {
            let inp = input.get(i).copied().unwrap_or(0.0);
            self.state[i] = self.decay * self.state[i] + self.input_gain * inp;
        }
        self.mean_activation = self.state.iter().map(|v| v.abs()).sum::<f32>() / self.dim as f32;
        self.n_steps += 1;
    }

    /// Project state to output dimension (simple linear readout).
    pub fn readout(&self, out_dim: usize) -> Vec<f32> {
        let mut out = vec![0.0f32; out_dim];
        for j in 0..out_dim {
            let src = j % self.dim;
            out[j] = self.state[src];
        }
        out
    }

    /// Reset to zero (e.g., on sleep or phase change).
    pub fn reset(&mut self) {
        self.state.fill(0.0);
    }

    pub fn status(&self) -> String {
        let act = self.state.iter().map(|v| v * v).sum::<f32>().sqrt();
        format!(
            "Recurrent | dim={} | steps={} | decay={:.2} | act={:.3} | mean_abs={:.3}",
            self.dim, self.n_steps, self.decay, act, self.mean_activation
        )
    }
}
