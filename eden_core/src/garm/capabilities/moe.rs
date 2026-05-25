// EDEN GARM — Mixture of Experts (MoE) sparse layer
// Top-k gating: solo k expertos se activan por token, reduciendo costo compute.
// Habilita escala a 1B+ params en CPU (sparse inference).

use std::f32;

pub fn xavier(rows: usize, cols: usize) -> Vec<Vec<f32>> {
    let scale = (2.0 / cols as f32).sqrt();
    let mut m = vec![vec![0.0f32; cols]; rows];
    let mut seed: u64 = 2024;
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
pub struct Expert {
    pub w1: Vec<Vec<f32>>,
    pub b1: Vec<f32>,
    pub w2: Vec<Vec<f32>>,
    pub b2: Vec<f32>,
}

impl Expert {
    pub fn new(input_dim: usize, hidden_dim: usize, output_dim: usize) -> Self {
        Expert {
            w1: xavier(input_dim, hidden_dim),
            b1: vec![0.0f32; hidden_dim],
            w2: xavier(hidden_dim, output_dim),
            b2: vec![0.0f32; output_dim],
        }
    }

    pub fn forward(&self, x: &[f32]) -> Vec<f32> {
        let mut h = vec![0.0f32; self.b1.len()];
        for j in 0..h.len() {
            let mut sum = self.b1[j];
            for i in 0..x.len() {
                sum += x[i] * self.w1[i][j];
            }
            h[j] = sum;
        }
        relu(&mut h);
        let mut out = vec![0.0f32; self.b2.len()];
        for j in 0..out.len() {
            let mut sum = self.b2[j];
            for i in 0..h.len() {
                sum += h[i] * self.w2[i][j];
            }
            out[j] = sum;
        }
        out
    }
}

#[derive(Clone, Debug)]
pub struct MoELayer {
    pub n_experts: usize,
    pub top_k: usize,
    pub input_dim: usize,
    pub output_dim: usize,
    pub experts: Vec<Expert>,
    pub gate_w: Vec<Vec<f32>>, // input_dim x n_experts
    pub gate_b: Vec<f32>,
    pub n_calls: u64,
    pub total_flops: u64,
}

impl MoELayer {
    pub fn new(
        n_experts: usize,
        top_k: usize,
        input_dim: usize,
        hidden_dim: usize,
        output_dim: usize,
    ) -> Self {
        let experts: Vec<Expert> = (0..n_experts)
            .map(|_| Expert::new(input_dim, hidden_dim, output_dim))
            .collect();
        MoELayer {
            n_experts,
            top_k: top_k.min(n_experts),
            input_dim,
            output_dim,
            experts,
            gate_w: xavier(input_dim, n_experts),
            gate_b: vec![0.0f32; n_experts],
            n_calls: 0,
            total_flops: 0,
        }
    }

    /// Forward with top-k sparse routing. Returns (output, expert_indices, gate_weights).
    pub fn forward(&self, x: &[f32]) -> (Vec<f32>, Vec<usize>, Vec<f32>) {
        // Compute gate logits
        let mut logits = vec![0.0f32; self.n_experts];
        for j in 0..self.n_experts {
            let mut sum = self.gate_b[j];
            for i in 0..x.len().min(self.input_dim) {
                sum += x[i] * self.gate_w[i][j];
            }
            logits[j] = sum;
        }
        // Top-k selection + softmax over selected
        let mut indexed: Vec<(usize, f32)> =
            logits.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top = indexed
            .into_iter()
            .take(self.top_k)
            .collect::<Vec<(usize, f32)>>();
        let max_logit = top
            .iter()
            .map(|(_, v)| *v)
            .fold(f32::NEG_INFINITY, f32::max);
        let exps: Vec<f32> = top.iter().map(|(_, v)| (v - max_logit).exp()).collect();
        let sum_exp: f32 = exps.iter().sum();
        let weights: Vec<f32> = exps.iter().map(|e| e / sum_exp.max(1e-8)).collect();
        let indices: Vec<usize> = top.iter().map(|(i, _)| *i).collect();
        // Run selected experts and combine
        let mut out = vec![0.0f32; self.output_dim];
        for (idx, &wi) in indices.iter().zip(weights.iter()) {
            let expert_out = self.experts[*idx].forward(x);
            for j in 0..self.output_dim {
                out[j] += wi * expert_out[j];
            }
        }
        self.n_calls; // read-only in shared borrow; tracking done in wrapper
        (out, indices, weights)
    }

    pub fn status(&self) -> String {
        let active_ratio = (self.top_k as f32 / self.n_experts as f32) * 100.0;
        format!(
            "MoE | experts={} | top_k={} | active={:.1}% | dim={} -> {}",
            self.n_experts, self.top_k, active_ratio, self.input_dim, self.output_dim
        )
    }
}
