// paradigms/autograd.rs — AutoGrad en Rust puro (ARM64 compatible)
// v2: Tensor 2D + MatMul + Softmax + LayerNorm for true deep learning

/// Tensor con gradiente automático. Forward + backward en 1 paso.
pub struct Var {
    pub data: Vec<f32>,
    pub grad: Vec<f32>,
}

impl Var {
    pub fn new(data: &[f32]) -> Self {
        Var {
            data: data.to_vec(),
            grad: vec![0.0; data.len()],
        }
    }
    pub fn zero(n: usize) -> Self {
        Var::new(&vec![0.0; n])
    }
    pub fn ones(n: usize) -> Self {
        Var::new(&vec![1.0; n])
    }
    pub fn rand(n: usize) -> Self {
        Var {
            data: (0..n).map(|i| ((i * 7 + 3) as f32 * 0.01).sin()).collect(),
            grad: vec![0.0; n],
        }
    }
}

/// Capa lineal: y = xW + b. Forward + backward automático.
pub struct Linear {
    pub w: Var,
    pub b: Var,
}

impl Linear {
    pub fn new(in_dim: usize, out_dim: usize) -> Self {
        Linear {
            w: Var::rand(in_dim * out_dim),
            b: Var::zero(out_dim),
        }
    }
    pub fn forward(&mut self, x: &[f32]) -> Vec<f32> {
        let out_dim = self.b.data.len();
        let in_dim = self.w.data.len() / out_dim.max(1);
        let max_j = x.len().min(in_dim);
        let mut y = vec![0.0; out_dim];
        for i in 0..out_dim {
            let mut s = self.b.data[i];
            for j in 0..max_j {
                s += self.w.data[j * out_dim + i] * x[j];
            }
            y[i] = s.max(0.0);
        }
        y
    }
    pub fn backward(&mut self, x: &[f32], y_pred: &[f32], y_true: &[f32], lr: f32) -> f32 {
        let out_dim = self.b.data.len();
        if y_true.len() < out_dim || y_pred.len() < out_dim {
            return 0.0;
        }
        let in_dim = self.w.data.len() / out_dim.max(1);
        let mut loss = 0.0;
        for i in 0..out_dim {
            let err = y_true[i] - y_pred[i];
            loss += err * err;
            let grad = if y_pred[i] > 0.0 { err } else { 0.0 };
            self.b.grad[i] += grad;
            for j in 0..x.len().min(in_dim) {
                self.w.grad[j * out_dim + i] += grad * x[j];
            }
        }
        for i in 0..self.w.data.len() {
            self.w.data[i] += lr * self.w.grad[i];
            self.w.grad[i] = 0.0;
        }
        for i in 0..self.b.data.len() {
            self.b.data[i] += lr * self.b.grad[i];
            self.b.grad[i] = 0.0;
        }
        loss / out_dim as f32
    }
    /// Batched forward: input is [batch × in_dim] → output [batch × out_dim]
    pub fn forward_batch(&mut self, x: &[Vec<f32>]) -> Vec<Vec<f32>> {
        x.iter().map(|xi| self.forward(xi)).collect()
    }
}

/// Adam optimizer: momentum + adaptive learning rate per parameter
pub struct Adam {
    pub m: Vec<f32>, // first moment (momentum)
    pub v: Vec<f32>, // second moment (RMSprop)
    pub beta1: f32,
    pub beta2: f32,
    pub eps: f32,
    pub t: u32,
}

impl Adam {
    pub fn new(n_params: usize) -> Self {
        Adam {
            m: vec![0.0; n_params],
            v: vec![0.0; n_params],
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            t: 0,
        }
    }
    pub fn step(&mut self, params: &mut [f32], grads: &[f32], lr: f32) {
        self.t += 1;
        let bias_corr1 = 1.0 - self.beta1.powi(self.t as i32);
        let bias_corr2 = 1.0 - self.beta2.powi(self.t as i32);
        for i in 0..params.len().min(grads.len()) {
            self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * grads[i];
            self.v[i] = self.beta2 * self.v[i] + (1.0 - self.beta2) * grads[i] * grads[i];
            let m_hat = self.m[i] / bias_corr1;
            let v_hat = self.v[i] / bias_corr2;
            params[i] -= lr * m_hat / (v_hat.sqrt() + self.eps);
        }
    }
}

/// Fallback optimizer when Adam state is not initialized.
/// Simple SGD with momentum. Used by GNN as fallback.
pub struct SGDOptimizer {
    pub velocity: Vec<f32>,
    pub momentum: f32,
}

impl SGDOptimizer {
    pub fn new(n_params: usize) -> Self {
        SGDOptimizer {
            velocity: vec![0.0; n_params],
            momentum: 0.9,
        }
    }
    /// Fallback step for GNN when Adam state is not initialized.
    pub fn fallback_step(&mut self, params: &mut [f32], grads: &[f32], lr: f32) {
        for i in 0..params.len().min(grads.len()) {
            self.velocity[i] = self.momentum * self.velocity[i] + lr * grads[i];
            params[i] -= self.velocity[i];
        }
    }
}

/// MSE loss: (pred - true)². Used by CNN1D training in deep.rs.
pub fn mse(pred: &[f32], target: &[f32]) -> f32 {
    pred.iter()
        .zip(target)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        / pred.len().max(1) as f32
}

// ═══════════════════════════════════════════════════════════════
// TENSOR 2D: Batched matrix operations for true deep learning
// ═══════════════════════════════════════════════════════════════

/// 2D Tensor: shape = [rows, cols], flat storage in data/grad.
/// Lighter CPU-only 2D tensor for small ops in ZeroShot/FewShot.
#[derive(Clone)]
pub struct Tensor {
    pub data: Vec<f32>,
    pub grad: Vec<f32>,
    pub rows: usize,
    pub cols: usize,
}

impl Tensor {
    pub fn new(rows: usize, cols: usize) -> Self {
        let n = rows * cols;
        Tensor {
            data: vec![0.0; n],
            grad: vec![0.0; n],
            rows,
            cols,
        }
    }
    pub fn from_vec(data: Vec<f32>, rows: usize, cols: usize) -> Self {
        Tensor {
            data,
            grad: vec![0.0; rows * cols],
            rows,
            cols,
        }
    }
    pub fn rand(rows: usize, cols: usize) -> Self {
        let n = rows * cols;
        let data: Vec<f32> = (0..n).map(|i| ((i * 7 + 3) as f32 * 0.01).sin()).collect();
        Tensor {
            data,
            grad: vec![0.0; n],
            rows,
            cols,
        }
    }
    pub fn get(&self, r: usize, c: usize) -> f32 {
        self.data[r * self.cols + c]
    }
    pub fn set(&mut self, r: usize, c: usize, v: f32) {
        self.data[r * self.cols + c] = v;
    }
    pub fn row(&self, r: usize) -> &[f32] {
        let start = r * self.cols;
        &self.data[start..start + self.cols]
    }
    pub fn row_mut(&mut self, r: usize) -> &mut [f32] {
        let start = r * self.cols;
        &mut self.data[start..start + self.cols]
    }
    /// Zero all gradients
    pub fn zero_grad(&mut self) {
        for g in &mut self.grad {
            *g = 0.0;
        }
    }

    /// MatMul: self[A×B] × other[B×C] → [A×C]. With gradient tracking.
    pub fn matmul(&self, other: &Self) -> Self {
        assert_eq!(self.cols, other.rows, "MatMul dim mismatch");
        let a = self.rows;
        let b = self.cols;
        let c = other.cols;
        let mut out = vec![0.0f32; a * c];
        for i in 0..a {
            for j in 0..c {
                let mut s = 0.0;
                for k in 0..b {
                    s += self.data[i * b + k] * other.data[k * c + j];
                }
                out[i * c + j] = s;
            }
        }
        Tensor {
            data: out,
            grad: vec![0.0; a * c],
            rows: a,
            cols: c,
        }
    }

    /// MatMul with gradient accumulation: dL/dself += grad_out × other^T, dL/dother += self^T × grad_out
    pub fn matmul_backward(&mut self, other: &mut Self, grad_out: &Tensor, lr: f32) {
        let a = self.rows;
        let b = self.cols;
        let c = other.cols;
        // dL/dself += grad_out @ other^T
        for i in 0..a {
            for k in 0..b {
                let mut s = 0.0;
                for j in 0..c {
                    s += grad_out.data[i * c + j] * other.data[k * c + j];
                }
                self.grad[i * b + k] += s;
            }
        }
        // dL/dother += self^T @ grad_out
        for k in 0..b {
            for j in 0..c {
                let mut s = 0.0;
                for i in 0..a {
                    s += self.data[i * b + k] * grad_out.data[i * c + j];
                }
                other.grad[k * c + j] += s;
            }
        }
        // Apply gradients
        for i in 0..self.data.len() {
            self.data[i] += lr * self.grad[i];
            self.grad[i] = 0.0;
        }
        for i in 0..other.data.len() {
            other.data[i] += lr * other.grad[i];
            other.grad[i] = 0.0;
        }
    }

    /// Softmax along last dimension (cols). Input [A×C], output [A×C].
    pub fn softmax(&self) -> Self {
        let mut out = self.data.clone();
        for i in 0..self.rows {
            let start = i * self.cols;
            let row = &self.data[start..start + self.cols];
            let max_val = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let sum: f32 = row.iter().map(|&x| (x - max_val).exp()).sum();
            for j in 0..self.cols {
                out[start + j] = ((self.data[start + j] - max_val).exp()) / sum.max(1e-8);
            }
        }
        Tensor {
            data: out,
            grad: vec![0.0; self.data.len()],
            rows: self.rows,
            cols: self.cols,
        }
    }

    /// ReLU activation
    pub fn relu(&self) -> Self {
        let out: Vec<f32> = self.data.iter().map(|&x| x.max(0.0)).collect();
        Tensor {
            data: out,
            grad: vec![0.0; self.data.len()],
            rows: self.rows,
            cols: self.cols,
        }
    }

    /// Layer Normalization over last dimension
    pub fn layer_norm(&self, eps: f32) -> Self {
        let mut out = self.data.clone();
        for i in 0..self.rows {
            let start = i * self.cols;
            let row = &self.data[start..start + self.cols];
            let mean: f32 = row.iter().sum::<f32>() / self.cols as f32;
            let var: f32 = row.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / self.cols as f32;
            let denom = (var + eps).sqrt();
            for j in 0..self.cols {
                out[start + j] = (self.data[start + j] - mean) / denom;
            }
        }
        Tensor {
            data: out,
            grad: vec![0.0; self.data.len()],
            rows: self.rows,
            cols: self.cols,
        }
    }

    /// Element-wise add. Both must have same shape.
    pub fn add(&self, other: &Self) -> Self {
        let out: Vec<f32> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();
        Tensor {
            data: out,
            grad: vec![0.0; self.data.len()],
            rows: self.rows,
            cols: self.cols,
        }
    }

    /// Scalar multiply
    pub fn scale(&self, s: f32) -> Self {
        let out: Vec<f32> = self.data.iter().map(|&x| x * s).collect();
        Tensor {
            data: out,
            grad: vec![0.0; self.data.len()],
            rows: self.rows,
            cols: self.cols,
        }
    }
}
