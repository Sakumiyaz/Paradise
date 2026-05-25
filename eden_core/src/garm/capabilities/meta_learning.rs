// EDEN GARM — Meta-Learning (MAML-inspired fast adaptation)
// Mantiene una "meta-inicialización" de parámetros.
// En cada tarea: inner loop adapta por N pasos. Outer loop actualiza init.
// Simplificado: solo 1 capa lineal meta-adaptable.

pub fn xavier(rows: usize, cols: usize) -> Vec<Vec<f32>> {
    let scale = (2.0 / cols as f32).sqrt();
    let mut m = vec![vec![0.0f32; cols]; rows];
    let mut seed: u64 = 888;
    for i in 0..rows {
        for j in 0..cols {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let r = ((seed % 1000) as f32 / 1000.0 - 0.5) * 2.0;
            m[i][j] = r * scale;
        }
    }
    m
}

#[derive(Clone, Debug)]
pub struct MetaLearning {
    pub input_dim: usize,
    pub output_dim: usize,
    pub meta_w: Vec<Vec<f32>>,
    pub meta_b: Vec<f32>,
    pub inner_lr: f32,
    pub meta_lr: f32,
    pub inner_steps: usize,
    pub n_tasks: u64,
    pub last_meta_loss: f32,
}

impl MetaLearning {
    pub fn new(input_dim: usize, output_dim: usize) -> Self {
        MetaLearning {
            input_dim,
            output_dim,
            meta_w: xavier(input_dim, output_dim),
            meta_b: vec![0.0f32; output_dim],
            inner_lr: 0.01,
            meta_lr: 0.001,
            inner_steps: 3,
            n_tasks: 0,
            last_meta_loss: 0.0,
        }
    }

    /// Forward with current meta params.
    pub fn forward(&self, x: &[f32], w: &[Vec<f32>], b: &[f32]) -> Vec<f32> {
        let mut out = vec![0.0f32; self.output_dim];
        for j in 0..self.output_dim {
            let mut sum = b[j];
            for i in 0..x.len().min(self.input_dim) {
                sum += x[i] * w[i][j];
            }
            out[j] = sum;
        }
        out
    }

    /// Inner loop adaptation: adapt w/b to task data.
    pub fn adapt(&self, task_data: &[(Vec<f32>, Vec<f32>)]) -> (Vec<Vec<f32>>, Vec<f32>) {
        let mut w = self.meta_w.clone();
        let mut b = self.meta_b.clone();
        for _ in 0..self.inner_steps {
            for (x, y) in task_data {
                let pred = self.forward(x, &w, &b);
                for j in 0..self.output_dim {
                    let err = y[j] - pred[j];
                    for i in 0..x.len().min(self.input_dim) {
                        w[i][j] += self.inner_lr * err * x[i];
                    }
                    b[j] += self.inner_lr * err;
                }
            }
        }
        (w, b)
    }

    /// Meta-update: update meta params using post-adaptation loss.
    pub fn meta_update(&mut self, task_data: &[(Vec<f32>, Vec<f32>)]) {
        let (adapted_w, adapted_b) = self.adapt(task_data);
        // Compute meta-loss = loss(adapted_params, task_data) and backprop to meta
        let mut meta_grad_w: Vec<Vec<f32>> = vec![vec![0.0f32; self.output_dim]; self.input_dim];
        let mut meta_grad_b = vec![0.0f32; self.output_dim];
        let mut total_loss = 0.0f32;
        for (x, y) in task_data {
            let pred = self.forward(x, &adapted_w, &adapted_b);
            for j in 0..self.output_dim {
                let err = y[j] - pred[j];
                total_loss += err * err;
                for i in 0..x.len().min(self.input_dim) {
                    meta_grad_w[i][j] += err * x[i];
                }
                meta_grad_b[j] += err;
            }
        }
        let n = task_data.len().max(1) as f32;
        total_loss /= n;
        // Simplified MAML: just nudge meta params toward adapted params
        for i in 0..self.input_dim {
            for j in 0..self.output_dim {
                self.meta_w[i][j] += self.meta_lr * (adapted_w[i][j] - self.meta_w[i][j]);
            }
        }
        for j in 0..self.output_dim {
            self.meta_b[j] += self.meta_lr * (adapted_b[j] - self.meta_b[j]);
        }
        self.last_meta_loss = total_loss;
        self.n_tasks += 1;
    }

    pub fn status(&self) -> String {
        format!(
            "MetaLearn | tasks={} | inner_steps={} | meta_loss={:.4}",
            self.n_tasks, self.inner_steps, self.last_meta_loss
        )
    }
}
