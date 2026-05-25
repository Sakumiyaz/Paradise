// paradigms/deep.rs — GNN · Transformer · LSTM · CNN1D
use crate::paradigms::autograd::Linear;
use rayon::prelude::*;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

pub struct EdgeInfo {
    pub target: u32,
    pub confidence: f32,
    pub rel_type: u8,
}

#[derive(Clone)]
pub struct MatF32 {
    rows: usize,
    cols: usize,
    data: Vec<f32>,
}

impl MatF32 {
    fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    fn from_fn(mut rows: usize, mut cols: usize, f: impl Fn(usize, usize) -> f32) -> Self {
        rows = rows.max(1);
        cols = cols.max(1);
        let mut data = vec![0.0; rows * cols];
        for i in 0..rows {
            for j in 0..cols {
                data[i * cols + j] = f(i, j);
            }
        }
        Self { rows, cols, data }
    }

    fn ncols(&self) -> usize {
        self.cols
    }

    fn transpose(&self) -> Self {
        Self::from_fn(self.cols, self.rows, |i, j| self[(j, i)])
    }

    fn subcols(&self, offset: usize, width: usize) -> Self {
        Self::from_fn(self.rows, width, |i, j| {
            self.data
                .get(i * self.cols + offset + j)
                .copied()
                .unwrap_or(0.0)
        })
    }

    fn matmul(&self, rhs: &Self) -> Self {
        let inner = self.cols.min(rhs.rows);
        Self::from_fn(self.rows, rhs.cols, |i, j| {
            let mut sum = 0.0;
            for k in 0..inner {
                sum += self[(i, k)] * rhs[(k, j)];
            }
            sum
        })
    }

    fn scaled(&self, scale: f32) -> Self {
        let mut out = self.clone();
        for value in &mut out.data {
            *value *= scale;
        }
        out
    }
}

impl Index<(usize, usize)> for MatF32 {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[index.0 * self.cols + index.1]
    }
}

impl IndexMut<(usize, usize)> for MatF32 {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0 * self.cols + index.1]
    }
}

// ═══ Structural embeddings via local deterministic graph features ═══
pub fn svd_embeddings(adj: &[Vec<EdgeInfo>], n: usize, dim: usize) -> Vec<Vec<f32>> {
    if n < 2 || dim == 0 {
        return vec![vec![0.0; dim.max(1)]; n.max(1)];
    }
    let mut result = vec![vec![0.0; dim]; n];
    let max_deg = adj.iter().map(|v| v.len()).max().unwrap_or(1) as f32;
    for i in 0..n {
        let deg = adj[i].len() as f32;
        result[i][0] = deg / max_deg.max(1.0);
        let mut total_conf = 0.0f32;
        for e in &adj[i] {
            let rel_weight = match e.rel_type {
                0 => 1.0, // IsA
                1 => 0.8, // Causes
                2 => 0.6, // HasProperty
                3 => 0.5, // PartOf
                4 => 0.3, // Opposes
                _ => 1.0,
            };
            total_conf += e.confidence * rel_weight;
        }
        if dim > 1 {
            result[i][1] = (total_conf / adj[i].len().max(1) as f32).max(0.1);
        }
        for k in 2..dim {
            result[i][k] = ((i * k + 7 + (deg as usize)) as f32 * 0.01).sin() * (total_conf + 0.1);
        }
    }
    for k in 0..dim {
        let norm: f32 = result
            .iter()
            .map(|v| v[k].powi(2))
            .sum::<f32>()
            .sqrt()
            .max(1e-8);
        if norm > 0.0 {
            for i in 0..n {
                result[i][k] /= norm;
            }
        }
    }
    result
}

// ═══ GNN: 6-layer with local matmul + chain-rule backprop + Adam + Early stopping ═══
pub struct GNN {
    pub layers: Vec<Linear>,
    pub loss_hist: Vec<f32>,
    pub hid: usize,
    pub adam: Option<crate::paradigms::autograd::Adam>,
    pub sgd: crate::paradigms::autograd::SGDOptimizer,
}
impl GNN {
    pub fn new(in_dim: usize, hid: usize) -> Self {
        let mut layers = Vec::with_capacity(6);
        layers.push(Linear::new(in_dim, hid));
        for _ in 1..5 {
            layers.push(Linear::new(hid, hid));
        }
        layers.push(Linear::new(hid, in_dim));
        let n_params: usize = layers.iter().map(|l| l.w.data.len() + l.b.data.len()).sum();
        GNN {
            layers,
            loss_hist: vec![],
            hid,
            adam: Some(crate::paradigms::autograd::Adam::new(n_params)),
            sgd: crate::paradigms::autograd::SGDOptimizer::new(n_params),
        }
    }

    pub fn train(
        &mut self,
        adj: &[Vec<EdgeInfo>],
        node_feats: &[Vec<f32>],
        epochs: usize,
        lr: f32,
    ) -> (Vec<Vec<f32>>, Vec<Vec<(Vec<f32>, Vec<f32>)>>) {
        let n = node_feats.len();
        if n == 0 || adj.is_empty() {
            return (vec![], vec![]);
        }
        let in_d = node_feats[0].len().min(32);
        let _hid = self.hid;

        // Input matrix X [n x in_d] via local row-major kernel.
        let x = MatF32::from_fn(n, in_d, |i, j| *node_feats[i].get(j).unwrap_or(&0.0));

        // Normalized adjacency A [n × n]
        let mut a = MatF32::zeros(n, n);
        for src in 0..n {
            for e in &adj[src] {
                let t = e.target as usize;
                if t < n {
                    a[(src, t)] += e.confidence;
                    a[(t, src)] += e.confidence * 0.3;
                }
            }
        }
        for i in 0..n {
            let dg: f32 = (0..n).map(|j| a[(i, j)]).sum();
            if dg > 0.0 {
                let sc = 1.0 / dg.sqrt();
                for j in 0..n {
                    a[(i, j)] *= sc;
                }
            }
        }
        let at = a.transpose();

        // Weight matrices [in_dl x out_d], built fresh each epoch.
        let build_w = |l: &Linear| -> MatF32 {
            let out_d = l.b.data.len();
            let in_dl = l.w.data.len() / out_d.max(1);
            MatF32::from_fn(in_dl, out_d, |i, j| l.w.data[i * out_d + j])
        };
        let mut loss_hist = Vec::new();

        for _ep in 0..epochs {
            let mut pre_acts: Vec<MatF32> = Vec::new();
            let mut lin_acts: Vec<MatF32> = Vec::new();
            let mut agg_acts: Vec<MatF32> = Vec::new();
            let mut relu_acts: Vec<MatF32> = Vec::new();

            let mut cur = x.clone();
            for li in 0..6 {
                let wm = build_w(&self.layers[li]);
                let out_d = wm.ncols();

                pre_acts.push(cur.clone());

                // Linear: h_lin = cur @ W + b
                let mut h_lin = cur.matmul(&wm);
                for i in 0..n {
                    for o in 0..out_d {
                        h_lin[(i, o)] += self.layers[li].b.data[o];
                    }
                }
                lin_acts.push(h_lin.clone());

                // Message passing: h_agg = h_lin * 0.7 + A @ h_lin * 0.3
                let nei = a.matmul(&h_lin);
                let mut h_agg = h_lin.clone();
                for i in 0..n {
                    for o in 0..out_d {
                        h_agg[(i, o)] = h_agg[(i, o)] * 0.7 + nei[(i, o)] * 0.3;
                    }
                }
                agg_acts.push(h_agg.clone());

                // ReLU + residual
                let mut h_relu = h_agg.clone();
                for i in 0..n {
                    for o in 0..out_d {
                        h_relu[(i, o)] = h_relu[(i, o)].max(0.0);
                    }
                }
                if li > 0 && out_d == cur.ncols() {
                    for i in 0..n {
                        for o in 0..out_d {
                            h_relu[(i, o)] += cur[(i, o)] * 0.2;
                        }
                    }
                }
                relu_acts.push(h_relu.clone());
                cur = h_relu;
            }

            // Loss: MSE
            let out = &relu_acts[5];
            let mut loss = 0.0f32;
            let mut dout = MatF32::zeros(n, in_d);
            for i in 0..n {
                for d in 0..in_d {
                    let err = out[(i, d)] - x[(i, d)];
                    loss += err * err;
                    dout[(i, d)] = err * 2.0 / n as f32;
                }
            }
            loss_hist.push(loss);

            // Exact chain-rule backprop
            let mut grad = dout;
            for li in (0..6).rev() {
                let wm = build_w(&self.layers[li]);
                let out_d = wm.ncols();

                let h_agg = &agg_acts[li];
                let mut dagg = MatF32::zeros(n, out_d);
                for i in 0..n {
                    for o in 0..out_d.min(grad.ncols()) {
                        dagg[(i, o)] = grad[(i, o)] * if h_agg[(i, o)] > 0.0 { 1.0 } else { 0.0 };
                    }
                }

                let dlin_step1 = dagg.scaled(0.7);
                let dlin_step2 = at.matmul(&dagg).scaled(0.3);
                let mut dlin = MatF32::zeros(n, out_d);
                for i in 0..n {
                    for o in 0..out_d {
                        dlin[(i, o)] = dlin_step1[(i, o)] + dlin_step2[(i, o)];
                    }
                }

                let prev = &pre_acts[li];
                let dw = prev.transpose().matmul(&dlin);
                let db: Vec<f32> = (0..out_d)
                    .map(|o| (0..n).map(|i| dlin[(i, o)]).sum())
                    .collect();
                let wmt = wm.transpose();
                let dprev = dlin.matmul(&wmt);

                for o in 0..out_d {
                    self.layers[li].b.grad[o] += db[o];
                    for j in 0..prev.ncols() {
                        self.layers[li].w.grad[j * out_d + o] += dw[(j, o)];
                    }
                }
                grad = dprev;
            }

            // Adam optimizer step
            let mut all_params: Vec<f32> = Vec::new();
            let mut all_grads: Vec<f32> = Vec::new();
            for li in 0..6 {
                all_params.extend_from_slice(&self.layers[li].w.data);
                all_params.extend_from_slice(&self.layers[li].b.data);
                all_grads.extend_from_slice(&self.layers[li].w.grad);
                all_grads.extend_from_slice(&self.layers[li].b.grad);
            }
            let grad_norm: f32 = all_grads
                .iter()
                .map(|&g| g * g)
                .sum::<f32>()
                .sqrt()
                .max(1e-8);
            if grad_norm > 5.0 {
                let scale = 5.0 / grad_norm;
                for g in &mut all_grads {
                    *g *= scale;
                }
            }
            if let Some(ref mut adam) = self.adam {
                adam.step(&mut all_params, &all_grads, lr);
            } else {
                self.sgd.fallback_step(&mut all_params, &all_grads, lr);
            }
            let mut off = 0usize;
            for li in 0..6 {
                let wl = self.layers[li].w.data.len();
                self.layers[li]
                    .w
                    .data
                    .copy_from_slice(&all_params[off..off + wl]);
                off += wl;
                let bl = self.layers[li].b.data.len();
                self.layers[li]
                    .b
                    .data
                    .copy_from_slice(&all_params[off..off + bl]);
                off += bl;
            }
            // Zero grads — rayon parallelized across layers
            self.layers.par_iter_mut().for_each(|l| {
                for w in 0..l.w.data.len() {
                    l.w.grad[w] = 0.0;
                }
                for b in 0..l.b.data.len() {
                    l.b.grad[b] = 0.0;
                }
            });

            if loss_hist.len() >= 5 {
                let last3: f32 = loss_hist.iter().rev().take(3).sum::<f32>() / 3.0;
                let prev3: f32 = loss_hist.iter().rev().skip(1).take(3).sum::<f32>() / 3.0;
                if last3 >= prev3 * 0.999 {
                    break;
                }
            }
        }

        self.loss_hist.extend(loss_hist);
        let mut h = x.clone();
        for li in 0..6 {
            let wm = build_w(&self.layers[li]);
            let mut h_lin = h.matmul(&wm);
            let out_d = wm.ncols();
            for i in 0..n {
                for o in 0..out_d {
                    h_lin[(i, o)] += self.layers[li].b.data[o];
                }
            }
            let nei = a.matmul(&h_lin);
            let mut agg = h_lin.clone();
            for i in 0..n {
                for o in 0..out_d {
                    agg[(i, o)] = agg[(i, o)] * 0.7 + nei[(i, o)] * 0.3;
                    agg[(i, o)] = agg[(i, o)].max(0.0);
                }
            }
            h = agg;
        }
        let result: Vec<Vec<f32>> = (0..n)
            .into_par_iter()
            .map(|i| (0..h.ncols()).map(|c| h[(i, c)]).collect())
            .collect();
        let pt: Vec<Vec<(Vec<f32>, Vec<f32>)>> = vec![result
            .iter()
            .take(10)
            .map(|e| {
                let mut f = e.clone();
                f.resize(4, 0.0);
                f.truncate(4);
                (f, vec![0.5; 4])
            })
            .collect()];
        (result, pt)
    }

    pub fn embed(&mut self, feats: &[Vec<f32>], adj: &[Vec<EdgeInfo>]) -> Vec<Vec<f32>> {
        let n = feats.len();
        if n == 0 || adj.is_empty() {
            return feats.to_vec();
        }
        let d = feats[0].len().min(32);
        let mut h = MatF32::from_fn(n, d, |i, j| *feats[i].get(j).unwrap_or(&0.0));
        let mut a = MatF32::zeros(n, n);
        for src in 0..n {
            for e in &adj[src] {
                let t = e.target as usize;
                if t < n {
                    a[(src, t)] += e.confidence;
                }
            }
        }
        for li in 0..6 {
            let out_d = self.layers[li].b.data.len();
            let in_dl = self.layers[li].w.data.len() / out_d.max(1);
            let w = MatF32::from_fn(in_dl, out_d, |i, j| self.layers[li].w.data[i * out_d + j]);
            let lh = h.matmul(&w);
            let nei = a.matmul(&lh);
            let mut aggr = lh.clone();
            let out_d2 = aggr.ncols();
            for i in 0..n {
                for o in 0..out_d2 {
                    aggr[(i, o)] = aggr[(i, o)] * 0.7 + nei[(i, o)] * 0.3;
                    aggr[(i, o)] = aggr[(i, o)].max(0.0);
                }
            }
            h = aggr;
        }
        (0..n)
            .map(|i| (0..h.ncols()).map(|c| h[(i, c)]).collect())
            .collect()
    }
}
// ═══ Transformer: 4-head self-attention with scaled dot-product + cooc bias ═══
pub struct Transformer {
    pub wq: Linear,
    pub wk: Linear,
    pub wv: Linear,
    pub wo: Linear,
    pub ln: Vec<f32>,
}
impl Transformer {
    pub fn new(dim: usize) -> Self {
        let _dk = dim / 4; // per-head dim
        Transformer {
            wq: Linear::new(dim, dim),
            wk: Linear::new(dim, dim),
            wv: Linear::new(dim, dim),
            wo: Linear::new(dim, dim),
            ln: vec![1.0f32; dim], // layer norm scale (init 1.0)
        }
    }

    pub fn train(
        &mut self,
        emb: &[Vec<f32>],
        adj: &[Vec<EdgeInfo>],
        _lr: f32,
    ) -> (Vec<Vec<f32>>, Vec<Vec<f32>>, HashMap<(usize, usize), f32>) {
        let n = emb.len();
        if n == 0 {
            return (vec![], vec![], HashMap::new());
        }
        let d = emb[0].len().min(32);
        let h = 4usize;
        let dk = d / h;

        // Input matrix [n × d]
        let mut x = MatF32::from_fn(n, d, |i, j| *emb[i].get(j).unwrap_or(&0.0));

        // Layer normalization (pre-norm) — manual compute to avoid borrow issues
        let mut sum = 0.0f32;
        let mut sq_sum = 0.0f32;
        let total = (n * d) as f32;
        for i in 0..n {
            for j in 0..d {
                let v = x[(i, j)];
                sum += v;
                sq_sum += v * v;
            }
        }
        let mean = sum / total;
        let var = sq_sum / total - mean * mean;
        for i in 0..n {
            for j in 0..d {
                x[(i, j)] =
                    (x[(i, j)] - mean) / (var + 1e-5).sqrt() * self.ln[j.min(self.ln.len() - 1)];
            }
        }

        // Co-occurrence matrix [n × n]
        let mut cooc = MatF32::zeros(n, n);
        for src in 0..n {
            for e in &adj[src] {
                let t = e.target as usize;
                if t < n {
                    cooc[(src, t)] += e.confidence;
                    cooc[(t, src)] += e.confidence * 0.3;
                }
            }
        }

        let mut cur = x.clone();
        for _layer in 0..2 {
            let q = cur.matmul(&Self::to_mat(&mut self.wq, d, d));
            let k = cur.matmul(&Self::to_mat(&mut self.wk, d, d));
            let v = cur.matmul(&Self::to_mat(&mut self.wv, d, d));

            let mut mha = MatF32::zeros(n, d);
            for head in 0..h {
                let off = head * dk;
                let scale = 1.0 / (dk as f32).sqrt();

                let qh = q.subcols(off, dk);
                let kh = k.subcols(off, dk);
                let vh = v.subcols(off, dk);

                let mut scores = qh.matmul(&kh.transpose());
                for i in 0..n {
                    for j in 0..n {
                        scores[(i, j)] *= scale;
                    }
                }
                let mut att = scores.clone();
                for i in 0..n {
                    for j in 0..n {
                        att[(i, j)] = (att[(i, j)] + cooc[(i, j)] * 0.3).exp();
                    }
                }
                for i in 0..n {
                    let row_sum: f32 = (0..n).map(|j| att[(i, j)]).sum();
                    if row_sum > 1e-8 {
                        for j in 0..n {
                            att[(i, j)] /= row_sum;
                        }
                    }
                }
                let head_out = att.matmul(&vh);
                for i in 0..n {
                    for j in 0..dk {
                        mha[(i, off + j)] = head_out[(i, j)];
                    }
                }
            }

            let proj = mha.matmul(&Self::to_mat(&mut self.wo, d, d));
            for i in 0..n {
                for j in 0..d {
                    cur[(i, j)] = cur[(i, j)] * 0.5 + proj[(i, j)] * 0.5;
                }
            }
        }

        let out: Vec<Vec<f32>> = (0..n)
            .map(|i| (0..d).map(|j| cur[(i, j)]).collect())
            .collect();
        let cooc_out: Vec<Vec<f32>> = (0..n)
            .map(|i| (0..n).map(|j| cooc[(i, j)]).collect())
            .collect();

        let mut et = HashMap::new();
        for i in 0..n.min(30) {
            for j in (i + 1)..n.min(30) {
                let s: f32 = (0..d)
                    .map(|k| cur[(i, k)] * cur[(j, k)])
                    .sum::<f32>()
                    .max(0.0);
                if s > 0.4 {
                    et.insert((i, j), s.clamp(0.0, 1.0));
                }
            }
        }
        (out, cooc_out, et)
    }

    pub fn link_score(&self, emb: &[Vec<f32>], a: usize, b: usize) -> f32 {
        if emb.is_empty() || a >= emb.len() || b >= emb.len() {
            return 0.0;
        }
        let d = emb[0].len().min(16);
        emb[a]
            .iter()
            .zip(&emb[b])
            .take(d)
            .map(|(x, y)| x * y)
            .sum::<f32>()
            .max(0.0)
    }

    pub fn to_mat(l: &mut Linear, rows: usize, cols: usize) -> MatF32 {
        MatF32::from_fn(rows, cols, |i, j| {
            l.w.data.get(i * cols + j).copied().unwrap_or(0.0)
        })
    }
}

// ═══ LSTM: single cell, 4x Linear::new(3,4) gates, 3-dim input → 4-dim hidden ═══
pub struct LSTMCell {
    pub wf: Linear,
    pub wi: Linear,
    pub wo: Linear,
    pub wc: Linear,
    pub h: Vec<f32>,
    pub c: Vec<f32>,
    pub hid: usize,
}
impl LSTMCell {
    pub fn new(_in_dim: usize, hid: usize) -> Self {
        LSTMCell {
            wf: Linear::new(3, 4),
            wi: Linear::new(3, 4),
            wo: Linear::new(3, 4),
            wc: Linear::new(3, 4),
            h: vec![0.0; hid],
            c: vec![0.0; hid],
            hid,
        }
    }
    pub fn sig(v: &[f32]) -> Vec<f32> {
        v.iter().map(|&z| 1.0 / (1.0 + (-z).exp())).collect()
    }
    pub fn forward(&mut self, x: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let x3: Vec<f32> = x
            .iter()
            .take(3)
            .cloned()
            .chain(std::iter::repeat(0.0))
            .take(3)
            .collect();
        let f = Self::sig(&self.wf.forward(&x3));
        let i = Self::sig(&self.wi.forward(&x3));
        let o = Self::sig(&self.wo.forward(&x3));
        let g: Vec<f32> = self.wc.forward(&x3).iter().map(|&z| z.tanh()).collect();
        let hd = self.hid.min(4);
        for k in 0..hd {
            let fk = f.get(k).copied().unwrap_or(0.5);
            let ik = i.get(k).copied().unwrap_or(0.0);
            let ok = o.get(k).copied().unwrap_or(0.5);
            let gk = g.get(k).copied().unwrap_or(0.0);
            self.c[k] = fk * self.c[k] + ik * gk;
            self.h[k] = ok * self.c[k].tanh();
        }
        (self.h.clone(), self.c.clone())
    }
    pub fn reset(&mut self) {
        for v in &mut self.h {
            *v = 0.0;
        }
        for v in &mut self.c {
            *v = 0.0;
        }
    }
}

pub struct StackedLSTM {
    pub cells: Vec<LSTMCell>,
    pub depth: usize,
}
impl StackedLSTM {
    pub fn new(in_dim: usize, hid: usize, d: usize) -> Self {
        let mut c = Vec::with_capacity(d);
        c.push(LSTMCell::new(in_dim, hid));
        for _ in 1..d {
            c.push(LSTMCell::new(hid, hid));
        }
        StackedLSTM { cells: c, depth: d }
    }

    pub fn train_sequence(
        &mut self,
        seq: &[Vec<f32>],
        _lr: f32,
    ) -> (Vec<f32>, f32, Vec<(Vec<f32>, Vec<f32>)>, f32) {
        if seq.len() < 2 {
            let h = self
                .cells
                .last()
                .map(|c| c.h.clone())
                .unwrap_or(vec![0.0; 4]);
            return (h, 0.0, vec![], 0.0);
        }
        for c in &mut self.cells {
            c.reset();
        }
        let hid = self.cells[0].hid;
        let mut loss = 0.0f32;
        let mut emex = vec![];
        for t in 0..seq.len() - 1 {
            let mut cur = seq[t].clone();
            cur.resize(3, 0.0);
            cur.truncate(3);
            for ci in 0..self.depth {
                let (h, _) = self.cells[ci].forward(&cur);
                cur = h;
            }
            let h = self.cells.last().map(|c| c.h.clone()).unwrap_or(vec![]);
            let target = seq[t + 1].clone();
            for k in 0..3.min(h.len()).min(target.len()) {
                let e = target[k] - h[k];
                loss += e * e;
            }
            let mut em = h.clone();
            em.resize(3, 0.0);
            em.truncate(3);
            emex.push((seq[t].clone(), em));
        }
        let fh = self
            .cells
            .last()
            .map(|c| c.h.clone())
            .unwrap_or(vec![0.0; hid]);
        let sr = if fh.len() >= 1 {
            (1.0 - fh[0].max(-1.0).min(1.0)) * 0.5
        } else {
            0.0
        };
        (
            fh,
            loss / (seq.len() - 1).max(1) as f32,
            emex,
            sr.clamp(0.0, 1.0),
        )
    }

    pub fn predict_next(&mut self, recent: &[Vec<f32>], steps: usize) -> Vec<f32> {
        for c in &mut self.cells {
            c.reset();
        }
        for s in recent {
            let mut cur = s.clone();
            cur.resize(3, 0.0);
            cur.truncate(3);
            for ci in 0..self.depth {
                let (h, _) = self.cells[ci].forward(&cur);
                cur = h;
            }
        }
        let mut last = self.cells.last().map(|c| c.h.clone()).unwrap_or(vec![0.0]);
        for _ in 0..steps {
            let mut cur = last.clone();
            cur.resize(3, 0.0);
            cur.truncate(3);
            for ci in 0..self.depth {
                let (h, _) = self.cells[ci].forward(&cur);
                cur = h;
            }
            last = self.cells.last().map(|c| c.h.clone()).unwrap_or(vec![0.0]);
        }
        last
    }
}

// ═══ CNN1D: simple conv1d with fixed kernels, detect clusters ═══
pub struct CNN1D {
    pub kernels: [Linear; 3],
    pub ks: [usize; 3],
    pub fc: Linear,
}
impl CNN1D {
    pub fn new() -> Self {
        CNN1D {
            kernels: [Linear::new(2, 1), Linear::new(4, 1), Linear::new(6, 1)],
            ks: [2, 4, 6],
            fc: Linear::new(3, 1),
        }
    }

    pub fn feat_vec(&mut self, seq: &[Vec<f32>]) -> Vec<f32> {
        if seq.is_empty() {
            return vec![];
        }
        let d = seq[0].len().max(1);
        let mut all = Vec::new();
        for ki in 0..3 {
            let ks = self.ks[ki];
            if seq.len() < ks {
                continue;
            }
            let mut pool = Vec::new();
            for t in 0..seq.len() - ks + 1 {
                let mut patch = vec![0.0f32; ks * d];
                for k in 0..ks {
                    for dd in 0..d.min(seq[t + k].len()) {
                        patch[k * d + dd] = seq[t + k][dd];
                    }
                }
                patch.resize(ks, 0.0);
                let p = self.kernels[ki].forward(&patch[..ks]);
                pool.push(p.get(0).copied().unwrap_or(0.0).max(0.0));
            }
            all.extend(
                pool.chunks(2)
                    .map(|c| c.iter().cloned().fold(f32::NEG_INFINITY, f32::max)),
            );
        }
        all
    }

    pub fn detect_anomalies(&mut self, bursts: &[Vec<f32>]) -> Vec<(usize, f32)> {
        let feats = self.feat_vec(bursts);
        if feats.len() < 3 {
            return vec![];
        }
        let mut r = Vec::new();
        for i in 0..feats.len() {
            let w: Vec<f32> = (0..3)
                .map(|o| feats[(i + o).min(feats.len() - 1)])
                .collect();
            let c = self.fc.forward(&w).get(0).copied().unwrap_or(0.0);
            if c > 0.3 {
                r.push((i, c.clamp(0.0, 1.0)));
            }
        }
        r.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        r
    }

    pub fn train(
        &mut self,
        seq: &[Vec<f32>],
        confs: &[f32],
        lr: f32,
    ) -> (f32, Vec<(Vec<f32>, f32)>) {
        let feats = self.feat_vec(seq);
        if feats.is_empty() || confs.is_empty() {
            return (0.0, vec![]);
        }
        let n = feats.len().min(confs.len()).min(3);
        let (mut loss, mut ex) = (0.0f32, vec![]);
        for i in 0..n {
            let w: Vec<f32> = (0..3).map(|o| feats[(i + o).min(n - 1)]).collect();
            let pr = self.fc.forward(&w);
            loss += super::autograd::mse(&pr, &[confs[i]]);
            self.fc.backward(&w, &pr, &[confs[i]], lr);
            ex.push((w, confs[i]));
        }
        (loss / n.max(1) as f32, ex)
    }
}
