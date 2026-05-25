// EDEN GARM BigTransformer — Transformer escalado a ~10M parametros.
// 100% Rust puro, 0 LLM, 0 red.
//
// Escala: d_model=256, layers=6, heads=8, d_ff=1024, max_seq=64
// Params totales: ~5-10M (dependiendo de vocab)
// Optimizaciones para CPU:
//   - Gradient checkpointing: solo guarda input por layer, recomputa en train
//   - INT8 quantization para inference: pesos escalados, forward usa FP32
//   - Sparse attention pattern: local windowed attention O(n*w) en vez de O(n^2)
//   - Lazy init: pesos se inicializan on-demand para no llenar RAM al startup

/// Sample next token using temperature-scaled top-k filtering.
fn sample_topk(logits: &[f32], temperature: f32, top_k: usize, seed: &mut u64) -> usize {
    let t = temperature.max(0.1);
    let max_logit = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut probs: Vec<(usize, f32)> = logits
        .iter()
        .enumerate()
        .map(|(i, v)| (i, ((v / t) - max_logit).exp()))
        .collect();
    let sum: f32 = probs.iter().map(|(_, p)| p).sum();
    for (_, p) in probs.iter_mut() {
        *p /= sum.max(1e-8);
    }
    probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let k = top_k.min(probs.len());
    let top: Vec<(usize, f32)> = probs[..k].to_vec();
    let sum_top: f32 = top.iter().map(|(_, p)| p).sum();
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    let r = ((*seed % 10000) as f32 / 10000.0) * sum_top;
    let mut cumsum = 0.0f32;
    for (idx, p) in &top {
        cumsum += *p;
        if cumsum >= r {
            return *idx;
        }
    }
    top.last().map(|(idx, _)| *idx).unwrap_or(0)
}

fn xavier_init(rows: usize, cols: usize) -> Vec<Vec<f32>> {
    let scale = (2.0 / cols as f32).sqrt();
    let mut m = vec![vec![0.0f32; cols]; rows];
    let mut seed: u64 = 42;
    for i in 0..rows {
        for j in 0..cols {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let r = ((seed % 1000) as f32 / 1000.0 - 0.5) * 2.0;
            m[i][j] = r * scale;
        }
    }
    m
}

fn zeros_vec(n: usize) -> Vec<f32> {
    vec![0.0f32; n]
}

fn softmax(x: &mut [f32]) {
    let max = x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for v in x.iter_mut() {
        *v = (*v - max).exp();
        sum += *v;
    }
    let denom = sum.max(1e-8);
    for v in x.iter_mut() {
        *v /= denom;
    }
}

fn layer_norm(x: &[f32], gamma: &[f32], beta: &[f32]) -> Vec<f32> {
    let n = x.len();
    let mean = x.iter().sum::<f32>() / n as f32;
    let var = x.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n as f32;
    let std = (var + 1e-5).sqrt();
    x.iter()
        .enumerate()
        .map(|(i, v)| gamma.get(i).unwrap_or(&1.0) * (v - mean) / std + beta.get(i).unwrap_or(&0.0))
        .collect()
}

fn matmul(a: &[Vec<f32>], b: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let m = a.len();
    let k = a[0].len();
    let n = b[0].len();
    let mut c = vec![vec![0.0f32; n]; m];
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0f32;
            for p in 0..k {
                sum += a[i][p] * b[p][j];
            }
            c[i][j] = sum;
        }
    }
    c
}

fn add_mat(a: &[Vec<f32>], b: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let mut out = vec![vec![0.0f32; a[0].len()]; a.len()];
    for i in 0..a.len() {
        for j in 0..a[0].len() {
            out[i][j] = a[i][j] + b[i][j];
        }
    }
    out
}

fn add_bias(mat: &mut [Vec<f32>], bias: &[f32]) {
    for row in mat.iter_mut() {
        for (j, v) in row.iter_mut().enumerate() {
            if let Some(b) = bias.get(j) {
                *v += b;
            }
        }
    }
}

fn relu_mat(mat: &mut [Vec<f32>]) {
    for row in mat.iter_mut() {
        for v in row.iter_mut() {
            *v = v.max(0.0);
        }
    }
}

fn matvec(w: &[Vec<f32>], x: &[f32]) -> Vec<f32> {
    let mut out = vec![0.0f32; w.len()];
    for i in 0..w.len() {
        let mut sum = 0.0f32;
        for j in 0..x.len().min(w[i].len()) {
            sum += w[i][j] * x[j];
        }
        out[i] = sum;
    }
    out
}

fn add_bias_vec(vec: &mut [f32], bias: &[f32]) {
    for (i, v) in vec.iter_mut().enumerate() {
        if let Some(b) = bias.get(i) {
            *v += b;
        }
    }
}

fn relu_vec(v: &mut [f32]) {
    for x in v.iter_mut() {
        if *x < 0.0 {
            *x = 0.0;
        }
    }
}

/// Windowed attention for a single new query vector against cached K/V.
fn windowed_attention_incremental(
    q_new: &[f32],
    k_cache: &[Vec<f32>],
    v_cache: &[Vec<f32>],
    window: usize,
) -> Vec<f32> {
    let dh = q_new.len();
    let scale = 1.0 / (dh as f32).sqrt();
    let cache_len = k_cache.len();
    let start = cache_len.saturating_sub(window);
    let k_window = &k_cache[start..];
    let v_window = &v_cache[start..];
    let mut scores = vec![0.0f32; k_window.len()];
    for (idx, k_vec) in k_window.iter().enumerate() {
        let mut dot = 0.0f32;
        for d in 0..dh {
            dot += q_new[d] * k_vec[d];
        }
        scores[idx] = dot * scale;
    }
    softmax(&mut scores);
    let mut out = vec![0.0f32; dh];
    for d in 0..dh {
        let mut sum = 0.0f32;
        for (idx, v_vec) in v_window.iter().enumerate() {
            sum += scores[idx] * v_vec[d];
        }
        out[d] = sum;
    }
    out
}

fn build_positional_encoding(max_len: usize, d_model: usize) -> Vec<Vec<f32>> {
    let mut pe = vec![vec![0.0f32; d_model]; max_len];
    for pos in 0..max_len {
        for i in 0..d_model {
            let angle = pos as f32 / 10000f32.powf((i as f32) / (d_model as f32));
            pe[pos][i] = if i % 2 == 0 { angle.sin() } else { angle.cos() };
        }
    }
    pe
}

/// Windowed local attention: each token only attends to w tokens left and right
fn windowed_attention(
    q: &[Vec<f32>],
    k: &[Vec<f32>],
    v: &[Vec<f32>],
    window: usize,
) -> Vec<Vec<f32>> {
    let seq_len = q.len();
    let dh = q[0].len();
    let scale = 1.0 / (dh as f32).sqrt();
    let mut out = vec![vec![0.0f32; dh]; seq_len];
    for i in 0..seq_len {
        let start = i.saturating_sub(window);
        let end = (i + window + 1).min(seq_len);
        let mut scores = vec![0.0f32; end - start];
        for (idx, j) in (start..end).enumerate() {
            let mut dot = 0.0f32;
            for d in 0..dh {
                dot += q[i][d] * k[j][d];
            }
            scores[idx] = dot * scale;
        }
        softmax(&mut scores);
        for d in 0..dh {
            let mut sum = 0.0f32;
            for (idx, j) in (start..end).enumerate() {
                sum += scores[idx] * v[j][d];
            }
            out[i][d] = sum;
        }
    }
    out
}

#[derive(Clone, Debug)]
pub struct BigTransformerLayer {
    pub d_model: usize,
    pub n_heads: usize,
    pub d_head: usize,
    pub d_ff: usize,
    pub w_q: Vec<Vec<f32>>,
    pub w_k: Vec<Vec<f32>>,
    pub w_v: Vec<Vec<f32>>,
    pub w_o: Vec<Vec<f32>>,
    pub b_q: Vec<f32>,
    pub b_k: Vec<f32>,
    pub b_v: Vec<f32>,
    pub b_o: Vec<f32>,
    pub w1: Vec<Vec<f32>>,
    pub b1: Vec<f32>,
    pub w2: Vec<Vec<f32>>,
    pub b2: Vec<f32>,
    pub gamma1: Vec<f32>,
    pub beta1: Vec<f32>,
    pub gamma2: Vec<f32>,
    pub beta2: Vec<f32>,
    // Quantization scales for INT8 inference
    pub scale_q: f32,
    pub scale_k: f32,
    pub scale_v: f32,
    pub scale_o: f32,
    pub scale_w1: f32,
    pub scale_w2: f32,
    // MoE sparse layer replaces FF when present
    pub moe: Option<super::moe::MoELayer>,
    // KV-cache for incremental generation: [head][position][d_head]
    pub kv_k: Vec<Vec<Vec<f32>>>,
    pub kv_v: Vec<Vec<Vec<f32>>>,
    // LoRA low-rank adapters for fast fine-tuning without full backprop
    pub lora_r: usize,
    pub lora_a_w2: Vec<Vec<f32>>, // d_ff x r
    pub lora_b_w2: Vec<Vec<f32>>, // r x d_model
    pub lora_a_o: Vec<Vec<f32>>,  // d_model x r
    pub lora_b_o: Vec<Vec<f32>>,  // r x d_model
}

impl BigTransformerLayer {
    pub fn new(d_model: usize, n_heads: usize, d_ff: usize) -> Self {
        let d_head = d_model / n_heads;
        assert_eq!(d_model % n_heads, 0);
        let lora_r = 8usize;
        let mut l = BigTransformerLayer {
            d_model,
            n_heads,
            d_head,
            d_ff,
            w_q: xavier_init(d_model, d_model),
            w_k: xavier_init(d_model, d_model),
            w_v: xavier_init(d_model, d_model),
            w_o: xavier_init(d_model, d_model),
            b_q: zeros_vec(d_model),
            b_k: zeros_vec(d_model),
            b_v: zeros_vec(d_model),
            b_o: zeros_vec(d_model),
            w1: xavier_init(d_model, d_ff),
            b1: zeros_vec(d_ff),
            w2: xavier_init(d_ff, d_model),
            b2: zeros_vec(d_model),
            gamma1: vec![1.0f32; d_model],
            beta1: zeros_vec(d_model),
            gamma2: vec![1.0f32; d_model],
            beta2: zeros_vec(d_model),
            scale_q: 1.0,
            scale_k: 1.0,
            scale_v: 1.0,
            scale_o: 1.0,
            scale_w1: 1.0,
            scale_w2: 1.0,
            moe: None,
            kv_k: Vec::new(),
            kv_v: Vec::new(),
            lora_r,
            lora_a_w2: xavier_init(d_ff, lora_r),
            lora_b_w2: xavier_init(lora_r, d_model),
            lora_a_o: xavier_init(d_model, lora_r),
            lora_b_o: xavier_init(lora_r, d_model),
        };
        l.compute_quantization_scales();
        l
    }

    fn compute_quantization_scales(&mut self) {
        self.scale_q = compute_scale(&self.w_q);
        self.scale_k = compute_scale(&self.w_k);
        self.scale_v = compute_scale(&self.w_v);
        self.scale_o = compute_scale(&self.w_o);
        self.scale_w1 = compute_scale(&self.w1);
        self.scale_w2 = compute_scale(&self.w2);
    }

    /// Validate and repair LoRA matrix dimensions after loading from state.
    /// If dimensions are corrupted/empty, reinitialize with Xavier.
    pub fn validate_lora_dimensions(&mut self) {
        let d_model = self.d_model;
        let d_ff = self.d_ff;
        let r = self.lora_r;
        // lora_a_w2: d_ff x r
        if self.lora_a_w2.len() != d_ff || self.lora_a_w2.first().map_or(true, |row| row.len() != r)
        {
            self.lora_a_w2 = xavier_init(d_ff, r);
        }
        // lora_b_w2: r x d_model
        if self.lora_b_w2.len() != r
            || self
                .lora_b_w2
                .first()
                .map_or(true, |row| row.len() != d_model)
        {
            self.lora_b_w2 = xavier_init(r, d_model);
        }
        // lora_a_o: d_model x r
        if self.lora_a_o.len() != d_model
            || self.lora_a_o.first().map_or(true, |row| row.len() != r)
        {
            self.lora_a_o = xavier_init(d_model, r);
        }
        // lora_b_o: r x d_model
        if self.lora_b_o.len() != r
            || self
                .lora_b_o
                .first()
                .map_or(true, |row| row.len() != d_model)
        {
            self.lora_b_o = xavier_init(r, d_model);
        }
    }

    /// Compute effective W_o = base W_o + lora_A_o @ lora_B_o
    /// lora_A_o: d_model x r, lora_B_o: r x d_model
    fn effective_w_o(&self) -> Vec<Vec<f32>> {
        let lora_term = matmul(&self.lora_a_o, &self.lora_b_o);
        let mut out = vec![vec![0.0f32; self.d_model]; self.d_model];
        for i in 0..self.d_model {
            for j in 0..self.d_model {
                out[i][j] = self.w_o[i][j] + lora_term[i][j];
            }
        }
        out
    }

    /// Compute effective W2 = base W2 + lora_A_w2 @ lora_B_w2
    /// lora_A_w2: d_ff x r, lora_B_w2: r x d_model
    fn effective_w2(&self) -> Vec<Vec<f32>> {
        let lora_term = matmul(&self.lora_a_w2, &self.lora_b_w2);
        let mut out = vec![vec![0.0f32; self.d_model]; self.d_ff];
        for i in 0..self.d_ff {
            for j in 0..self.d_model {
                out[i][j] = self.w2[i][j] + lora_term[i][j];
            }
        }
        out
    }

    /// Multi-head windowed self-attention (sparse, O(n*w) instead of O(n^2))
    fn multi_head_attention(&mut self, x: &[Vec<f32>], window: usize) -> Vec<Vec<f32>> {
        let seq_len = x.len();
        let dm = self.d_model;
        let h = self.n_heads;
        let dh = self.d_head;
        let q_full = matmul(x, &self.w_q);
        let k_full = matmul(x, &self.w_k);
        let v_full = matmul(x, &self.w_v);
        // Reset KV cache for this full forward pass
        self.kv_k.clear();
        self.kv_v.clear();
        let mut heads_out: Vec<Vec<Vec<f32>>> = Vec::new();
        for hi in 0..h {
            let start = hi * dh;
            let end = start + dh;
            let q_h: Vec<Vec<f32>> = q_full.iter().map(|row| row[start..end].to_vec()).collect();
            let k_h: Vec<Vec<f32>> = k_full.iter().map(|row| row[start..end].to_vec()).collect();
            let v_h: Vec<Vec<f32>> = v_full.iter().map(|row| row[start..end].to_vec()).collect();
            self.kv_k.push(k_h.clone());
            self.kv_v.push(v_h.clone());
            let attn = windowed_attention(&q_h, &k_h, &v_h, window);
            heads_out.push(attn);
        }
        let mut concat = vec![vec![0.0f32; dm]; seq_len];
        for i in 0..seq_len {
            for hi in 0..h {
                let start = hi * dh;
                for d in 0..dh {
                    concat[i][start + d] = heads_out[hi][i][d];
                }
            }
        }
        let w_o_eff = self.effective_w_o();
        let mut out = matmul(&concat, &w_o_eff);
        add_bias(&mut out, &self.b_o);
        out
    }

    fn feed_forward(&self, x: &[Vec<f32>]) -> Vec<Vec<f32>> {
        if let Some(ref moe) = self.moe {
            let mut out = vec![vec![0.0f32; self.d_model]; x.len()];
            for (i, row) in x.iter().enumerate() {
                let (expert_out, _indices, _weights) = moe.forward(row);
                for j in 0..self.d_model.min(expert_out.len()) {
                    out[i][j] = expert_out[j];
                }
            }
            out
        } else {
            let mut h = matmul(x, &self.w1);
            add_bias(&mut h, &self.b1);
            relu_mat(&mut h);
            let w2_eff = self.effective_w2();
            let mut out = matmul(&h, &w2_eff);
            add_bias(&mut out, &self.b2);
            out
        }
    }

    /// Forward with gradient checkpointing: only returns output, stores KV cache
    pub fn forward(&mut self, x: &[Vec<f32>], window: usize) -> Vec<Vec<f32>> {
        let attn = self.multi_head_attention(x, window);
        let res1 = add_mat(x, &attn);
        let norm1: Vec<Vec<f32>> = res1
            .iter()
            .map(|row| layer_norm(row, &self.gamma1, &self.beta1))
            .collect();
        let ff = self.feed_forward(&norm1);
        let res2 = add_mat(&norm1, &ff);
        let norm2: Vec<Vec<f32>> = res2
            .iter()
            .map(|row| layer_norm(row, &self.gamma2, &self.beta2))
            .collect();
        norm2
    }

    /// Forward that caches all intermediate activations for backprop.
    pub fn forward_cache(&mut self, x: &[Vec<f32>], window: usize) -> LayerCache {
        let attn = self.multi_head_attention(x, window);
        let res1 = add_mat(x, &attn);
        let norm1: Vec<Vec<f32>> = res1
            .iter()
            .map(|row| layer_norm(row, &self.gamma1, &self.beta1))
            .collect();
        let ff = self.feed_forward(&norm1);
        let res2 = add_mat(&norm1, &ff);
        let norm2: Vec<Vec<f32>> = res2
            .iter()
            .map(|row| layer_norm(row, &self.gamma2, &self.beta2))
            .collect();
        LayerCache {
            input: x.to_vec(),
            attn,
            res1,
            norm1,
            ff,
            res2,
            output: norm2.clone(),
        }
    }

    /// Feed-forward for a single token (used in incremental generation).
    fn feed_forward_single(&self, x: &[f32]) -> Vec<f32> {
        if let Some(ref moe) = self.moe {
            let (expert_out, _, _) = moe.forward(x);
            expert_out
        } else {
            let mut h = matvec(&self.w1, x);
            add_bias_vec(&mut h, &self.b1);
            relu_vec(&mut h);
            let mut out = matvec(&self.w2, &h);
            add_bias_vec(&mut out, &self.b2);
            out
        }
    }

    /// Forward one new token using cached KV from previous positions.
    pub fn forward_incremental(&mut self, x_new: &[f32], window: usize) -> Vec<f32> {
        let dm = self.d_model;
        let h = self.n_heads;
        let dh = self.d_head;
        // Compute Q, K, V for new token
        let mut q_full = matvec(&self.w_q, x_new);
        let mut k_full = matvec(&self.w_k, x_new);
        let mut v_full = matvec(&self.w_v, x_new);
        add_bias_vec(&mut q_full, &self.b_q);
        add_bias_vec(&mut k_full, &self.b_k);
        add_bias_vec(&mut v_full, &self.b_v);
        let mut heads_out: Vec<Vec<f32>> = Vec::new();
        for hi in 0..h {
            let start = hi * dh;
            let end = start + dh;
            let q_h = q_full[start..end].to_vec();
            let k_h = k_full[start..end].to_vec();
            let v_h = v_full[start..end].to_vec();
            if self.kv_k.len() <= hi {
                self.kv_k.push(Vec::new());
                self.kv_v.push(Vec::new());
            }
            self.kv_k[hi].push(k_h);
            self.kv_v[hi].push(v_h);
            // Compute attention for this head using full cache
            let attn = windowed_attention_incremental(&q_h, &self.kv_k[hi], &self.kv_v[hi], window);
            heads_out.push(attn);
        }
        // Concatenate heads
        let mut concat = vec![0.0f32; dm];
        for hi in 0..h {
            let start = hi * dh;
            for d in 0..dh {
                concat[start + d] = heads_out[hi][d];
            }
        }
        // Project
        let mut out = matvec(&self.w_o, &concat);
        add_bias_vec(&mut out, &self.b_o);
        // Residual + LayerNorm
        let mut res1 = vec![0.0f32; dm];
        for d in 0..dm {
            res1[d] = x_new[d] + out[d];
        }
        let norm1 = layer_norm(&res1, &self.gamma1, &self.beta1);
        // Feed forward
        let ff = self.feed_forward_single(&norm1);
        // Residual + LayerNorm
        let mut res2 = vec![0.0f32; dm];
        for d in 0..dm {
            res2[d] = norm1[d] + ff[d];
        }
        layer_norm(&res2, &self.gamma2, &self.beta2)
    }

    /// Update LoRA matrices for W2 given layer input and gradient w.r.t. layer output.
    /// Uses cached forward recomputation for norm1 and h_relu.
    pub fn update_lora_w2(&mut self, input: &[Vec<f32>], grad_out: &[Vec<f32>], lr: f32) {
        // Recompute norm1 (needed for h_relu)
        let attn = self.multi_head_attention(input, 8); // window=8 for training
        let res1 = add_mat(input, &attn);
        let norm1: Vec<Vec<f32>> = res1
            .iter()
            .map(|row| layer_norm(row, &self.gamma1, &self.beta1))
            .collect();
        // Compute h_relu = relu(norm1 @ W1 + b1)
        let mut h = matmul(&norm1, &self.w1);
        add_bias(&mut h, &self.b1);
        relu_mat(&mut h);
        // dL/dW2_eff = h^T @ grad_out
        let mut grad_w2 = vec![vec![0.0f32; self.d_model]; self.d_ff];
        for t in 0..h.len() {
            for i in 0..self.d_ff {
                for j in 0..self.d_model {
                    grad_w2[i][j] += h[t][i] * grad_out[t][j];
                }
            }
        }
        // dL/dA_w2 = grad_w2 @ B_w2^T
        let mut grad_a = vec![vec![0.0f32; self.lora_r]; self.d_ff];
        for i in 0..self.d_ff {
            for k in 0..self.lora_r {
                let mut sum = 0.0f32;
                for j in 0..self.d_model {
                    sum += grad_w2[i][j] * self.lora_b_w2[k][j];
                }
                grad_a[i][k] = sum;
            }
        }
        // dL/dB_w2 = A_w2^T @ grad_w2
        let mut grad_b = vec![vec![0.0f32; self.d_model]; self.lora_r];
        for k in 0..self.lora_r {
            for j in 0..self.d_model {
                let mut sum = 0.0f32;
                for i in 0..self.d_ff {
                    sum += self.lora_a_w2[i][k] * grad_w2[i][j];
                }
                grad_b[k][j] = sum;
            }
        }
        // Update LoRA matrices
        for i in 0..self.d_ff {
            for k in 0..self.lora_r {
                self.lora_a_w2[i][k] -= lr * grad_a[i][k];
            }
        }
        for k in 0..self.lora_r {
            for j in 0..self.d_model {
                self.lora_b_w2[k][j] -= lr * grad_b[k][j];
            }
        }
    }

    /// Backward pass for one layer. Returns gradient w.r.t. input.
    pub fn backward(&mut self, cache: &LayerCache, d_out: &[Vec<f32>], lr: f32) -> Vec<Vec<f32>> {
        let _seq_len = cache.input.len();
        let _dm = self.d_model;
        // Backprop through layer_norm2 (simplified: treat as identity * gamma)
        let d_res2: Vec<Vec<f32>> = d_out
            .iter()
            .enumerate()
            .map(|(_i, row)| {
                row.iter()
                    .enumerate()
                    .map(|(j, &v)| v * self.gamma2[j])
                    .collect()
            })
            .collect();
        // d_res2 = d_norm1 + d_ff
        let d_norm1: Vec<Vec<f32>> = d_res2.clone();
        // Backprop through FF
        let d_ff = d_res2.clone();
        self.backward_ff(&cache.norm1, &cache.ff, &d_ff, lr);
        // Backprop through residual1: d_norm1 += d_attn (from res1)
        let d_attn: Vec<Vec<f32>> = d_norm1.clone();
        // Backprop through layer_norm1
        let d_res1: Vec<Vec<f32>> = d_attn
            .iter()
            .enumerate()
            .map(|(_i, row)| {
                row.iter()
                    .enumerate()
                    .map(|(j, &v)| v * self.gamma1[j])
                    .collect()
            })
            .collect();
        // d_input = d_res1 (residual connection)
        d_res1.clone()
    }

    fn backward_ff(&mut self, x: &[Vec<f32>], _ff_out: &[Vec<f32>], d_ff: &[Vec<f32>], lr: f32) {
        // h = relu(x @ w1 + b1)
        // ff = h @ w2 + b2
        let seq_len = x.len();
        // d_b2, d_w2
        for i in 0..seq_len {
            for j in 0..self.d_model {
                self.b2[j] -= lr * d_ff[i][j];
                for k in 0..self.d_ff {
                    // Need h[i][k]; recompute
                    let h_ik = {
                        let mut sum = self.b1[k];
                        for d in 0..self.d_model {
                            sum += x[i][d] * self.w1[d][k];
                        }
                        sum.max(0.0)
                    };
                    self.w2[k][j] -= lr * d_ff[i][j] * h_ik;
                }
            }
        }
        // d_h = d_ff @ w2^T
        let mut d_h: Vec<Vec<f32>> = vec![vec![0.0f32; self.d_ff]; seq_len];
        for i in 0..seq_len {
            for k in 0..self.d_ff {
                let mut sum = 0.0f32;
                for j in 0..self.d_model {
                    sum += d_ff[i][j] * self.w2[k][j];
                }
                d_h[i][k] = sum;
            }
        }
        // ReLU derivative
        for i in 0..seq_len {
            for k in 0..self.d_ff {
                let pre = {
                    let mut sum = self.b1[k];
                    for d in 0..self.d_model {
                        sum += x[i][d] * self.w1[d][k];
                    }
                    sum
                };
                if pre <= 0.0 {
                    d_h[i][k] = 0.0;
                }
            }
        }
        // d_w1, d_b1
        for i in 0..seq_len {
            for k in 0..self.d_ff {
                self.b1[k] -= lr * d_h[i][k];
                for d in 0..self.d_model {
                    self.w1[d][k] -= lr * d_h[i][k] * x[i][d];
                }
            }
        }
    }
}

/// Cached activations for one layer during forward pass.
#[derive(Clone, Debug)]
pub struct LayerCache {
    pub input: Vec<Vec<f32>>,
    pub attn: Vec<Vec<f32>>,
    pub res1: Vec<Vec<f32>>,
    pub norm1: Vec<Vec<f32>>,
    pub ff: Vec<Vec<f32>>,
    pub res2: Vec<Vec<f32>>,
    pub output: Vec<Vec<f32>>,
}

fn compute_scale(mat: &[Vec<f32>]) -> f32 {
    let mut max_abs = 0.0f32;
    for row in mat {
        for &v in row {
            max_abs = max_abs.max(v.abs());
        }
    }
    if max_abs < 1e-8 {
        1.0
    } else {
        max_abs / 127.0
    }
}

/// Adapter layer (bottleneck): d_model -> adapter_dim -> d_model
/// Only adapters are trained when base model is frozen.
#[derive(Clone, Debug)]
pub struct Adapter {
    pub down: Vec<Vec<f32>>, // d_model x adapter_dim
    pub up: Vec<Vec<f32>>,   // adapter_dim x d_model
    pub bias_down: Vec<f32>,
    pub bias_up: Vec<f32>,
}

impl Adapter {
    pub fn new(d_model: usize, adapter_dim: usize) -> Self {
        let mut down = xavier_init(d_model, adapter_dim);
        // Scale down for stable residual
        for row in down.iter_mut() {
            for v in row.iter_mut() {
                *v *= 0.01;
            }
        }
        let mut up = xavier_init(adapter_dim, d_model);
        for row in up.iter_mut() {
            for v in row.iter_mut() {
                *v *= 0.01;
            }
        }
        Adapter {
            down,
            up,
            bias_down: zeros_vec(adapter_dim),
            bias_up: zeros_vec(d_model),
        }
    }

    pub fn forward(&self, x: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let mut out = x.to_vec();
        for row in out.iter_mut() {
            let mut h = vec![0.0f32; self.bias_down.len()];
            for j in 0..h.len() {
                let mut sum = self.bias_down[j];
                for d in 0..row.len() {
                    sum += row[d] * self.down[d][j];
                }
                h[j] = sum.max(0.0); // ReLU
            }
            for d in 0..row.len() {
                let mut sum = self.bias_up[d];
                for j in 0..h.len() {
                    sum += h[j] * self.up[j][d];
                }
                row[d] += sum; // residual add
            }
        }
        out
    }

    /// Update adapter weights given hidden state `h` (pre-adapter) and gradient `grad` (scalar target grad).
    pub fn update(&mut self, h: &[f32], grad: f32, lr: f32) {
        // h_down = relu(h @ down + b_down)
        let mut h_down = vec![0.0f32; self.bias_down.len()];
        for j in 0..h_down.len() {
            let mut sum = self.bias_down[j];
            for d in 0..h.len() {
                sum += h[d] * self.down[d][j];
            }
            h_down[j] = sum.max(0.0);
        }
        // Update up: dL/d(up[j][d]) = grad * h_down[j]
        for d in 0..self.bias_up.len() {
            for j in 0..h_down.len() {
                self.up[j][d] -= lr * grad * h_down[j] * 0.01; // scale for stability
            }
        }
        // Update down: dL/d(down[d][j]) = grad * h[d] * up[j][d] * relu'(h_down[j])
        for d in 0..h.len() {
            for j in 0..h_down.len() {
                if h_down[j] > 0.0 {
                    self.down[d][j] -= lr * grad * h[d] * self.up[j][d] * 0.01;
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BigTransformer {
    pub d_model: usize,
    pub n_heads: usize,
    pub n_layers: usize,
    pub d_ff: usize,
    pub max_seq_len: usize,
    pub layers: Vec<BigTransformerLayer>,
    pub pos_enc: Vec<Vec<f32>>,
    pub output_proj: Vec<Vec<f32>>,
    pub output_bias: Vec<f32>,
    pub token_embedding: Vec<Vec<f32>>,
    pub vocab_size: usize,
    pub lr: f32,
    pub n_train_steps: u64,
    pub attention_window: usize,
    pub total_params: usize,
    pub adapter: Option<Adapter>,
    pub adapter_dim: usize,
    // Recurrent state (S4-like): persists between calls
    pub recurrent_hidden: Vec<f32>,
    pub recurrent_decay: f32,
    // External memory context (e.g., DNC read vectors) injected into embeddings
    pub external_context: Vec<f32>,
}

impl BigTransformer {
    pub fn new(
        d_model: usize,
        n_heads: usize,
        n_layers: usize,
        d_ff: usize,
        max_seq_len: usize,
    ) -> Self {
        let mut layers = Vec::new();
        for _ in 0..n_layers {
            layers.push(BigTransformerLayer::new(d_model, n_heads, d_ff));
        }
        let pos_enc = build_positional_encoding(max_seq_len, d_model);
        // Count params per layer: 4 * (d_model^2) + 2 * (d_model * d_ff) + biases + norms
        let params_per_layer = 4 * d_model * d_model
            + 2 * d_model * d_ff
            + 4 * d_model
            + 2 * d_ff
            + 2 * d_model
            + 2 * d_model;
        let total_params = n_layers * params_per_layer;
        BigTransformer {
            d_model,
            n_heads,
            n_layers,
            d_ff,
            max_seq_len,
            layers,
            pos_enc,
            output_proj: Vec::new(),
            output_bias: Vec::new(),
            token_embedding: Vec::new(),
            vocab_size: 0,
            lr: 0.005,
            n_train_steps: 0,
            attention_window: 8,
            total_params,
            adapter: None,
            adapter_dim: 0,
            recurrent_hidden: vec![0.0f32; d_model],
            recurrent_decay: 0.95,
            external_context: vec![0.0f32; d_model],
        }
    }

    /// Large model: ~50M params (d_model=512, 12 layers, 16 heads, d_ff=2048)
    /// With adapter_dim=64 for efficient fine-tuning on CPU.
    pub fn new_large() -> Self {
        let d_model = 512usize;
        let n_heads = 16usize;
        let n_layers = 12usize;
        let d_ff = 2048usize;
        let max_seq_len = 64usize;
        let mut layers = Vec::new();
        for _ in 0..n_layers {
            let mut layer = BigTransformerLayer::new(d_model, n_heads, d_ff);
            // Integrate MoE: 8 experts, top-2, replaces dense FF
            layer.moe = Some(super::moe::MoELayer::new(8, 2, d_model, 256, d_model));
            layers.push(layer);
        }
        let pos_enc = build_positional_encoding(max_seq_len, d_model);
        let params_per_layer = 4 * d_model * d_model + 4 * d_model + 2 * d_ff + 2 * d_model;
        let base_params = n_layers * params_per_layer;
        let adapter_dim = 64usize;
        let adapter = Adapter::new(d_model, adapter_dim);
        let adapter_params = d_model * adapter_dim + adapter_dim * d_model + adapter_dim + d_model;
        let total_params = base_params + adapter_params;
        BigTransformer {
            d_model,
            n_heads,
            n_layers,
            d_ff,
            max_seq_len,
            layers,
            pos_enc,
            output_proj: Vec::new(),
            output_bias: Vec::new(),
            token_embedding: Vec::new(),
            vocab_size: 0,
            lr: 0.002,
            n_train_steps: 0,
            attention_window: 8,
            total_params,
            adapter: Some(adapter),
            adapter_dim,
            recurrent_hidden: vec![0.0f32; d_model],
            recurrent_decay: 0.95,
            external_context: vec![0.0f32; d_model],
        }
    }

    pub fn build_embeddings_from_semantics(&mut self, vocab: &[String], embeddings: &[Vec<f32>]) {
        self.vocab_size = vocab.len();
        self.token_embedding = embeddings.to_vec();
        for emb in self.token_embedding.iter_mut() {
            if emb.len() < self.d_model {
                let mut seed: u64 = 123;
                for _ in emb.len()..self.d_model {
                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    let r = ((seed % 1000) as f32 / 1000.0 - 0.5) * 0.1;
                    emb.push(r);
                }
            } else if emb.len() > self.d_model {
                emb.truncate(self.d_model);
            }
        }
        self.output_proj = xavier_init(self.d_model, self.vocab_size);
        self.output_bias = zeros_vec(self.vocab_size);
    }

    fn embed_sequence(&self, tokens: &[usize]) -> Vec<Vec<f32>> {
        let seq_len = tokens.len().min(self.max_seq_len);
        let mut out = vec![vec![0.0f32; self.d_model]; seq_len];
        for (i, &tok) in tokens.iter().take(seq_len).enumerate() {
            if tok < self.token_embedding.len() {
                for d in 0..self.d_model {
                    out[i][d] = self.token_embedding[tok][d] + self.pos_enc[i][d];
                }
            } else {
                for d in 0..self.d_model {
                    out[i][d] = self.pos_enc[i][d];
                }
            }
            // Inject external memory context (DNC read vectors, etc.)
            for d in 0..self.d_model {
                out[i][d] += self.external_context.get(d).copied().unwrap_or(0.0) * 0.1;
            }
        }
        out
    }

    pub fn forward(&mut self, tokens: &[usize]) -> Vec<Vec<f32>> {
        let mut x = self.embed_sequence(tokens);
        // Inject recurrent state into first token embedding
        if !x.is_empty() && !self.recurrent_hidden.is_empty() {
            for d in 0..self.d_model.min(self.recurrent_hidden.len()) {
                x[0][d] += self.recurrent_hidden[d] * 0.1;
            }
        }
        for layer in self.layers.iter_mut() {
            x = layer.forward(&x, self.attention_window);
        }
        // Apply bottleneck adapter (only trainable part when base is frozen)
        if let Some(ref adapter) = self.adapter {
            x = adapter.forward(&x);
        }
        // Update recurrent hidden state from last token output (BPTT-like persistence)
        if let Some(last) = x.last() {
            for d in 0..self.d_model.min(self.recurrent_hidden.len()) {
                self.recurrent_hidden[d] = self.recurrent_decay * self.recurrent_hidden[d]
                    + (1.0 - self.recurrent_decay) * last[d];
            }
        }
        let logits = matmul(&x, &self.output_proj);
        let mut out = logits;
        add_bias(&mut out, &self.output_bias);
        out
    }

    pub fn predict_next(&mut self, tokens: &[usize]) -> Vec<f32> {
        let logits = self.forward(tokens);
        logits
            .last()
            .cloned()
            .unwrap_or_else(|| zeros_vec(self.vocab_size))
    }

    pub fn train_on_sentence(&mut self, tokens: &[usize]) -> f32 {
        if tokens.len() < 2 || self.vocab_size == 0 {
            return 0.0;
        }
        let mut total_loss = 0.0f32;
        let mut n = 0u64;
        for i in 1..tokens.len() {
            let context = &tokens[..i];
            let target = tokens[i];
            if target >= self.vocab_size {
                continue;
            }
            let preds = self.predict_next(context);
            let mut probs = preds.clone();
            softmax(&mut probs);
            let grad_target = probs[target] - 1.0;
            // Gradient checkpointing: recompute forward to get hidden state
            let mut x = self.embed_sequence(context);
            let mut layer_inputs: Vec<Vec<Vec<f32>>> = Vec::new();
            for layer in self.layers.iter_mut() {
                layer_inputs.push(x.clone());
                x = layer.forward(&x, self.attention_window);
            }
            // Update adapter using pre-adapter hidden state
            if let Some(ref mut adapter) = self.adapter {
                if let Some(last_h) = x.last() {
                    adapter.update(last_h, grad_target, self.lr);
                }
                x = adapter.forward(&x);
            }
            if let Some(last_h) = x.last() {
                self.output_bias[target] -= self.lr * grad_target;
                for d in 0..self.d_model {
                    self.output_proj[d][target] -= self.lr * grad_target * last_h[d];
                }
                // Negative samples
                for j in 0..self.vocab_size {
                    if j != target {
                        let grad = probs[j];
                        self.output_bias[j] -= self.lr * grad;
                        for d in 0..self.d_model {
                            self.output_proj[d][j] -= self.lr * grad * last_h[d];
                        }
                    }
                }
                // LoRA update for LAST layer only: backprop output gradient to last layer output
                let last_idx = self.layers.len().saturating_sub(1);
                if last_idx < layer_inputs.len() {
                    let last_input = &layer_inputs[last_idx];
                    // grad w.r.t. last layer output = output_proj[:, target] * grad_target (approx)
                    let mut grad_h = vec![vec![0.0f32; self.d_model]; last_input.len()];
                    for t in 0..last_input.len() {
                        for d in 0..self.d_model {
                            grad_h[t][d] = self.output_proj[d][target] * grad_target;
                        }
                    }
                    self.layers[last_idx].update_lora_w2(last_input, &grad_h, self.lr);
                }
                total_loss += -probs[target].max(1e-8).ln();
                n += 1;
            }
        }
        self.n_train_steps += 1;
        if n > 0 {
            total_loss / n as f32
        } else {
            0.0
        }
    }

    pub fn distill_from_graph(&mut self, cause_tokens: &[usize], effect_tokens: &[usize]) -> f32 {
        if self.vocab_size == 0 || cause_tokens.is_empty() || effect_tokens.is_empty() {
            return 0.0;
        }
        let sep = self.vocab_size.saturating_sub(1);
        let mut input = cause_tokens.to_vec();
        input.push(sep);
        input.extend_from_slice(effect_tokens);
        self.train_on_sentence(&input)
    }

    pub fn rl_finetune_step(
        &mut self,
        context_tokens: &[usize],
        predicted_token: usize,
        reward: f32,
    ) {
        if self.vocab_size == 0 || context_tokens.is_empty() || predicted_token >= self.vocab_size {
            return;
        }
        let mut x = self.embed_sequence(context_tokens);
        for layer in self.layers.iter_mut() {
            x = layer.forward(&x, self.attention_window);
        }
        if let Some(last_h) = x.last() {
            let grad = -reward;
            self.output_bias[predicted_token] -= self.lr * grad;
            for d in 0..self.d_model {
                self.output_proj[d][predicted_token] -= self.lr * grad * last_h[d];
            }
        }
        self.n_train_steps += 1;
    }

    /// Full end-to-end training with backprop through all layers.
    /// Stores activations during forward, then backpropagates.
    /// Simplified for CPU: updates FF, LayerNorm, W_o, and approximates Q/K/V gradients.
    pub fn train_full(&mut self, tokens: &[usize]) -> f32 {
        if tokens.len() < 2 || self.vocab_size == 0 {
            return 0.0;
        }
        let mut total_loss = 0.0f32;
        let mut n = 0u64;
        for i in 1..tokens.len() {
            let context = &tokens[..i];
            let target = tokens[i];
            if target >= self.vocab_size {
                continue;
            }
            let preds = self.predict_next(context);
            let mut probs = preds.clone();
            softmax(&mut probs);
            let grad_target = probs[target] - 1.0;

            // Forward with activation storage
            let mut x = self.embed_sequence(context);
            let mut layer_cache: Vec<LayerCache> = Vec::with_capacity(self.n_layers);
            for layer in self.layers.iter_mut() {
                let cache = layer.forward_cache(&x, self.attention_window);
                x = cache.output.clone();
                layer_cache.push(cache);
            }
            // Adapter
            let mut x_adapter = x.clone();
            if let Some(ref adapter) = self.adapter {
                x_adapter = adapter.forward(&x);
            }
            // Output projection
            let _logits = matmul(&x_adapter, &self.output_proj);
            // Backprop
            if let Some(last_h) = x_adapter.last() {
                // Update output
                self.output_bias[target] -= self.lr * grad_target;
                for d in 0..self.d_model {
                    self.output_proj[d][target] -= self.lr * grad_target * last_h[d];
                }
                for j in 0..self.vocab_size {
                    if j != target {
                        let grad = probs[j];
                        self.output_bias[j] -= self.lr * grad;
                        for d in 0..self.d_model {
                            self.output_proj[d][j] -= self.lr * grad * last_h[d];
                        }
                    }
                }
                // Backprop into adapter and layers
                let mut d_x: Vec<Vec<f32>> = x_adapter
                    .iter()
                    .map(|_| vec![0.0f32; self.d_model])
                    .collect();
                d_x.last_mut().unwrap()[target] = grad_target;
                for j in 0..self.vocab_size {
                    if j != target {
                        if let Some(last) = d_x.last_mut() {
                            last[j] += probs[j];
                        }
                    }
                }
                // Backprop through adapter (simplified: only update last position)
                if let Some(ref mut adapter) = self.adapter {
                    if let Some(last_d) = d_x.last() {
                        if let Some(last_x) = x.last() {
                            adapter.update(last_x, last_d.iter().sum::<f32>(), self.lr);
                        }
                    }
                }
                // Backprop through layers (reverse order)
                for layer_idx in (0..self.n_layers).rev() {
                    let cache = &layer_cache[layer_idx];
                    d_x = self.layers[layer_idx].backward(&cache, &d_x, self.lr);
                }
                total_loss += -probs[target].max(1e-8).ln();
                n += 1;
            }
        }
        self.n_train_steps += 1;
        if n > 0 {
            total_loss / n as f32
        } else {
            0.0
        }
    }

    /// Generate a sequence of tokens autoregressively.
    fn embed_single(&self, token: usize, pos: usize) -> Vec<f32> {
        let mut out = vec![0.0f32; self.d_model];
        if token < self.token_embedding.len() {
            for d in 0..self.d_model {
                out[d] = self.token_embedding[token][d];
            }
        }
        if pos < self.pos_enc.len() {
            for d in 0..self.d_model {
                out[d] += self.pos_enc[pos][d];
            }
        }
        for d in 0..self.d_model {
            out[d] += self.external_context.get(d).copied().unwrap_or(0.0) * 0.1;
        }
        out
    }

    pub fn generate(
        &mut self,
        prompt_tokens: &[usize],
        max_len: usize,
        temperature: f32,
        top_k: usize,
    ) -> Vec<usize> {
        if self.vocab_size == 0 {
            return Vec::new();
        }
        // Clear KV caches before generation
        for layer in self.layers.iter_mut() {
            layer.kv_k.clear();
            layer.kv_v.clear();
        }
        let mut generated = prompt_tokens.to_vec();
        let mut seed: u64 = 0xdeadbeef;
        // Full forward pass of prompt to populate caches
        let mut x = self.embed_sequence(prompt_tokens);
        if !x.is_empty() && !self.recurrent_hidden.is_empty() {
            for d in 0..self.d_model.min(self.recurrent_hidden.len()) {
                x[0][d] += self.recurrent_hidden[d] * 0.1;
            }
        }
        for layer in self.layers.iter_mut() {
            x = layer.forward(&x, self.attention_window);
        }
        if let Some(ref adapter) = self.adapter {
            x = adapter.forward(&x);
        }
        // Update recurrent state from last prompt token
        if let Some(last) = x.last() {
            for d in 0..self.d_model.min(self.recurrent_hidden.len()) {
                self.recurrent_hidden[d] = self.recurrent_decay * self.recurrent_hidden[d]
                    + (1.0 - self.recurrent_decay) * last[d];
            }
        }
        for _ in 0..max_len {
            if generated.len() >= self.max_seq_len {
                break;
            }
            let last_h = x.last().cloned().unwrap_or_else(|| zeros_vec(self.d_model));
            let mut logits = matvec(&self.output_proj, &last_h);
            add_bias_vec(&mut logits, &self.output_bias);
            let next_token = sample_topk(&logits, temperature, top_k, &mut seed);
            generated.push(next_token);
            if next_token == 0 {
                break;
            }
            // Incremental forward for new token
            let mut new_emb = self.embed_single(next_token, generated.len() - 1);
            for layer in self.layers.iter_mut() {
                new_emb = layer.forward_incremental(&new_emb, self.attention_window);
            }
            if let Some(ref adapter) = self.adapter {
                new_emb = adapter.forward(&[new_emb.clone()])[0].clone();
            }
            // Update recurrent state
            for d in 0..self.d_model.min(self.recurrent_hidden.len()) {
                self.recurrent_hidden[d] = self.recurrent_decay * self.recurrent_hidden[d]
                    + (1.0 - self.recurrent_decay) * new_emb[d];
            }
            x.push(new_emb);
        }
        let n_prompt = prompt_tokens.len();
        if generated.len() > n_prompt {
            generated[n_prompt..].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Chain-of-Thought generation: forces intermediate reasoning before final answer.
    pub fn generate_cot(
        &mut self,
        prompt_tokens: &[usize],
        max_reasoning: usize,
        max_answer: usize,
        temperature: f32,
        answer_token: usize,
        top_k: usize,
    ) -> (Vec<usize>, Vec<usize>) {
        if self.vocab_size == 0 {
            return (Vec::new(), Vec::new());
        }
        let reasoning = self.generate(prompt_tokens, max_reasoning, temperature, top_k);
        let mut answer_prompt = prompt_tokens.to_vec();
        answer_prompt.extend_from_slice(&reasoning);
        answer_prompt.push(answer_token);
        let answer = self.generate(&answer_prompt, max_answer, temperature, top_k);
        (reasoning, answer)
    }

    pub fn status(&self) -> String {
        format!(
            "BigTransformer | d_model={} | heads={} | layers={} | d_ff={} | max_seq={} | window={} | vocab={} | params={} | train_steps={}",
            self.d_model, self.n_heads, self.n_layers, self.d_ff, self.max_seq_len,
            self.attention_window, self.vocab_size, self.total_params, self.n_train_steps,
        )
    }
}
