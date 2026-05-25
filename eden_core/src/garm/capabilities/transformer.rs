// EDEN GARM Transformer — Self-attention mechanism propio, multi-layer, 100% Rust puro.
//
// Arquitectura simplificada pero real:
//   - Self-attention: Q, K, V matrices manual
//   - Multi-head: 4 heads
//   - Feed-forward: 2 capas lineales + ReLU
//   - LayerNorm: mean/var normalization
//   - Positional encoding: sinusoidal
//   - Stack: 2 layers
//
// Parametros: d_model=64, n_heads=4, d_ff=128, max_seq=32
// Entrenable online via backprop manual.

/// Xavier-like init: scale by sqrt(2/fan_in)
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
    // Keep top_k
    probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let k = top_k.min(probs.len());
    let top: Vec<(usize, f32)> = probs[..k].to_vec();
    let sum_top: f32 = top.iter().map(|(_, p)| p).sum();
    // Weighted random sample via LCG
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

fn zeros(rows: usize, cols: usize) -> Vec<Vec<f32>> {
    vec![vec![0.0f32; cols]; rows]
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

/// Sinusoidal positional encoding
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

/// Matrix multiplication: A (m x k) @ B (k x n) -> C (m x n)
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

/// Add two matrices elementwise
fn add_mat(a: &[Vec<f32>], b: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let mut out = vec![vec![0.0f32; a[0].len()]; a.len()];
    for i in 0..a.len() {
        for j in 0..a[0].len() {
            out[i][j] = a[i][j] + b[i][j];
        }
    }
    out
}

/// Add vector to each row of matrix
fn add_bias(mat: &mut [Vec<f32>], bias: &[f32]) {
    for row in mat.iter_mut() {
        for (j, v) in row.iter_mut().enumerate() {
            if let Some(b) = bias.get(j) {
                *v += b;
            }
        }
    }
}

/// Apply ReLU to matrix
fn relu_mat(mat: &mut [Vec<f32>]) {
    for row in mat.iter_mut() {
        for v in row.iter_mut() {
            *v = v.max(0.0);
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransformerLayer {
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
    pub w1: Vec<Vec<f32>>, // d_model x d_ff
    pub b1: Vec<f32>,
    pub w2: Vec<Vec<f32>>, // d_ff x d_model
    pub b2: Vec<f32>,
    pub gamma1: Vec<f32>,
    pub beta1: Vec<f32>,
    pub gamma2: Vec<f32>,
    pub beta2: Vec<f32>,
    // Cached for backprop
    pub cache_input: Vec<Vec<f32>>,
    pub cache_attn_out: Vec<Vec<f32>>,
    pub cache_ff_out: Vec<Vec<f32>>,
}

impl TransformerLayer {
    pub fn new(d_model: usize, n_heads: usize, d_ff: usize) -> Self {
        let d_head = d_model / n_heads;
        assert_eq!(d_model % n_heads, 0, "d_model must be divisible by n_heads");
        TransformerLayer {
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
            cache_input: Vec::new(),
            cache_attn_out: Vec::new(),
            cache_ff_out: Vec::new(),
        }
    }

    /// Single-head attention: Q(seq x d_head) @ K^T -> softmax -> @ V
    fn single_head_attention(
        &self,
        q: &[Vec<f32>],
        k: &[Vec<f32>],
        v: &[Vec<f32>],
    ) -> Vec<Vec<f32>> {
        let seq_len = q.len();
        let dh = q[0].len();
        let scale = 1.0 / (dh as f32).sqrt();
        let mut out = vec![vec![0.0f32; dh]; seq_len];
        for i in 0..seq_len {
            let mut scores = vec![0.0f32; seq_len];
            for j in 0..seq_len {
                let mut dot = 0.0f32;
                for d in 0..dh {
                    dot += q[i][d] * k[j][d];
                }
                scores[j] = dot * scale;
            }
            softmax(&mut scores);
            for d in 0..dh {
                let mut sum = 0.0f32;
                for j in 0..seq_len {
                    sum += scores[j] * v[j][d];
                }
                out[i][d] = sum;
            }
        }
        out
    }

    /// Multi-head self-attention
    fn multi_head_attention(&self, x: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let seq_len = x.len();
        let dm = self.d_model;
        let h = self.n_heads;
        let dh = self.d_head;

        // Project to Q, K, V
        let q_full = matmul(x, &self.w_q);
        let k_full = matmul(x, &self.w_k);
        let v_full = matmul(x, &self.w_v);

        let mut heads_out: Vec<Vec<Vec<f32>>> = Vec::new();
        for hi in 0..h {
            let start = hi * dh;
            let end = start + dh;
            let q_h: Vec<Vec<f32>> = q_full.iter().map(|row| row[start..end].to_vec()).collect();
            let k_h: Vec<Vec<f32>> = k_full.iter().map(|row| row[start..end].to_vec()).collect();
            let v_h: Vec<Vec<f32>> = v_full.iter().map(|row| row[start..end].to_vec()).collect();
            let attn = self.single_head_attention(&q_h, &k_h, &v_h);
            heads_out.push(attn);
        }

        // Concatenate heads
        let mut concat = vec![vec![0.0f32; dm]; seq_len];
        for i in 0..seq_len {
            for hi in 0..h {
                let start = hi * dh;
                for d in 0..dh {
                    concat[i][start + d] = heads_out[hi][i][d];
                }
            }
        }

        // Output projection
        let mut out = matmul(&concat, &self.w_o);
        add_bias(&mut out, &self.b_o);
        out
    }

    /// Feed-forward: linear -> ReLU -> linear
    fn feed_forward(&self, x: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let mut h = matmul(x, &self.w1);
        add_bias(&mut h, &self.b1);
        relu_mat(&mut h);
        let mut out = matmul(&h, &self.w2);
        add_bias(&mut out, &self.b2);
        out
    }

    /// Forward pass for this layer
    pub fn forward(&mut self, x: &[Vec<f32>]) -> Vec<Vec<f32>> {
        self.cache_input = x.to_vec();
        // Self-attention + residual + layer norm
        let attn = self.multi_head_attention(x);
        let res1 = add_mat(x, &attn);
        let norm1: Vec<Vec<f32>> = res1
            .iter()
            .map(|row| layer_norm(row, &self.gamma1, &self.beta1))
            .collect();
        // Feed-forward + residual + layer norm
        let ff = self.feed_forward(&norm1);
        let res2 = add_mat(&norm1, &ff);
        let norm2: Vec<Vec<f32>> = res2
            .iter()
            .map(|row| layer_norm(row, &self.gamma2, &self.beta2))
            .collect();
        self.cache_attn_out = attn.clone();
        self.cache_ff_out = ff.clone();
        norm2
    }
}

#[derive(Clone, Debug)]
pub struct EdenTransformer {
    pub d_model: usize,
    pub n_heads: usize,
    pub n_layers: usize,
    pub d_ff: usize,
    pub max_seq_len: usize,
    pub layers: Vec<TransformerLayer>,
    pub pos_enc: Vec<Vec<f32>>,
    pub output_proj: Vec<Vec<f32>>, // d_model x vocab_size (initially identity-like, filled at runtime)
    pub output_bias: Vec<f32>,
    // Token embedding: maps token index -> dense vector. Built from semantics or learned.
    pub token_embedding: Vec<Vec<f32>>,
    pub vocab_size: usize,
    pub lr: f32,
    pub n_train_steps: u64,
}

impl EdenTransformer {
    pub fn new(
        d_model: usize,
        n_heads: usize,
        n_layers: usize,
        d_ff: usize,
        max_seq_len: usize,
    ) -> Self {
        let mut layers = Vec::new();
        for _ in 0..n_layers {
            layers.push(TransformerLayer::new(d_model, n_heads, d_ff));
        }
        let pos_enc = build_positional_encoding(max_seq_len, d_model);
        EdenTransformer {
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
            lr: 0.01,
            n_train_steps: 0,
        }
    }

    /// Build token embeddings from semantics module (PPMI/SVD vectors)
    pub fn build_embeddings_from_semantics(&mut self, vocab: &[String], embeddings: &[Vec<f32>]) {
        self.vocab_size = vocab.len();
        self.token_embedding = embeddings.to_vec();
        // If embeddings are smaller than d_model, pad with small noise
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
        // Output projection: d_model -> vocab_size
        self.output_proj = xavier_init(self.d_model, self.vocab_size);
        self.output_bias = zeros_vec(self.vocab_size);
    }

    /// Convert sequence of token indices to embeddings + positional encoding
    fn embed_sequence(&self, tokens: &[usize]) -> Vec<Vec<f32>> {
        let seq_len = tokens.len().min(self.max_seq_len);
        let mut out = vec![vec![0.0f32; self.d_model]; seq_len];
        for (i, &tok) in tokens.iter().take(seq_len).enumerate() {
            if tok < self.token_embedding.len() {
                for d in 0..self.d_model {
                    out[i][d] = self.token_embedding[tok][d] + self.pos_enc[i][d];
                }
            } else {
                // Unknown token: just positional encoding
                for d in 0..self.d_model {
                    out[i][d] = self.pos_enc[i][d];
                }
            }
        }
        out
    }

    /// Forward pass: tokens -> logits over vocab
    pub fn forward(&mut self, tokens: &[usize]) -> Vec<Vec<f32>> {
        let mut x = self.embed_sequence(tokens);
        for layer in self.layers.iter_mut() {
            x = layer.forward(&x);
        }
        // Project to vocab logits
        let logits = matmul(&x, &self.output_proj);
        let mut out = logits;
        add_bias(&mut out, &self.output_bias);
        out
    }

    /// Forward pass but return only last position logits (for next-token prediction)
    pub fn predict_next(&mut self, tokens: &[usize]) -> Vec<f32> {
        let logits = self.forward(tokens);
        if let Some(last) = logits.last() {
            last.clone()
        } else {
            zeros_vec(self.vocab_size)
        }
    }

    /// Simple SGD training step: tokens -> target_probs at each position
    pub fn train_step(&mut self, tokens: &[usize], target_indices: &[usize]) -> f32 {
        if tokens.len() < 2 || self.vocab_size == 0 {
            return 0.0;
        }
        let logits = self.forward(tokens);
        let seq_len = logits.len().min(target_indices.len());
        let mut loss = 0.0f32;
        // Compute softmax cross-entropy and gradients (simplified: only update output projection)
        let mut d_logits = vec![vec![0.0f32; self.vocab_size]; seq_len];
        for i in 0..seq_len {
            let mut probs = logits[i].clone();
            softmax(&mut probs);
            let target = target_indices[i].min(self.vocab_size - 1);
            for j in 0..self.vocab_size {
                let grad = probs[j] - if j == target { 1.0 } else { 0.0 };
                d_logits[i][j] = grad;
                if j == target {
                    loss -= (probs[target].max(1e-8)).ln();
                }
            }
        }
        // Use the final hidden state for gradient update
        let final_hidden = self.layers.last().unwrap().cache_input.clone();
        let seq_h = final_hidden.len();
        // Gradient w.r.t output_proj: hidden^T @ d_logits
        for i in 0..seq_h {
            for j in 0..self.vocab_size {
                let grad = d_logits[i][j];
                self.output_bias[j] -= self.lr * grad;
                for d in 0..self.d_model {
                    if i < final_hidden.len() && d < final_hidden[i].len() {
                        self.output_proj[d][j] -= self.lr * grad * final_hidden[i][d];
                    }
                }
            }
        }
        self.n_train_steps += 1;
        loss / seq_len as f32
    }

    /// Train on a single sentence: tokenize -> predict each next word
    pub fn train_on_sentence(&mut self, tokens: &[usize]) -> f32 {
        if tokens.len() < 2 {
            return 0.0;
        }
        let mut total_loss = 0.0f32;
        let mut n = 0u64;
        for i in 1..tokens.len() {
            let context = &tokens[..i];
            let target = tokens[i];
            let preds = self.predict_next(context);
            if target < self.vocab_size {
                let mut probs = preds.clone();
                softmax(&mut probs);
                // Update output bias and projection for this target
                let grad_target = probs[target] - 1.0;
                let final_hidden = self.layers.last().unwrap().cache_input.clone();
                if let Some(last_h) = final_hidden.last() {
                    self.output_bias[target] -= self.lr * grad_target;
                    for d in 0..self.d_model {
                        self.output_proj[d][target] -= self.lr * grad_target * last_h[d];
                    }
                }
                // Update other vocab items too (negative samples)
                for j in 0..self.vocab_size {
                    if j != target {
                        let grad = probs[j];
                        self.output_bias[j] -= self.lr * grad;
                        if let Some(last_h) = final_hidden.last() {
                            for d in 0..self.d_model {
                                self.output_proj[d][j] -= self.lr * grad * last_h[d];
                            }
                        }
                    }
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

    /// E) Distillation from symbolic causal graph.
    /// Given (cause_tokens, effect_tokens), train the transformer to predict effect given cause.
    /// This "distills" the symbolic causal knowledge into neural weights.
    pub fn distill_from_graph(&mut self, cause_tokens: &[usize], effect_tokens: &[usize]) -> f32 {
        if self.vocab_size == 0 || cause_tokens.is_empty() || effect_tokens.is_empty() {
            return 0.0;
        }
        // Build input: cause + "->" separator (use vocab_size as separator, clamped)
        let sep = self.vocab_size.saturating_sub(1);
        let mut input = cause_tokens.to_vec();
        input.push(sep);
        input.extend_from_slice(effect_tokens);
        // Train: predict each next token
        self.train_on_sentence(&input)
    }

    /// G) RL-style fine-tuning using outcome reward.
    /// After making a prediction, compare with actual outcome.
    /// If reward > 0 (good outcome), reinforce the weights that led to the prediction.
    /// If reward < 0 (bad outcome), penalize.
    pub fn rl_finetune_step(
        &mut self,
        context_tokens: &[usize],
        predicted_token: usize,
        reward: f32,
    ) {
        if self.vocab_size == 0 || context_tokens.is_empty() || predicted_token >= self.vocab_size {
            return;
        }
        // Forward to get hidden state for this context
        let _logits = self.forward(context_tokens);
        let final_hidden = self.layers.last().unwrap().cache_input.clone();
        if let Some(last_h) = final_hidden.last() {
            // Update output projection: reinforce/penalize the predicted token
            let grad = -reward; // negative because we minimize loss; positive reward -> negative grad -> increase weight
            self.output_bias[predicted_token] -= self.lr * grad;
            for d in 0..self.d_model {
                self.output_proj[d][predicted_token] -= self.lr * grad * last_h[d];
            }
        }
        self.n_train_steps += 1;
    }

    /// Generate a sequence of tokens autoregressively.
    /// Returns only the newly generated tokens (excludes prompt).
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
        let mut generated = prompt_tokens.to_vec();
        let mut seed: u64 = 0xdeadbeef;
        for _ in 0..max_len {
            if generated.len() >= self.max_seq_len {
                break;
            }
            let logits = self.predict_next(&generated);
            let next_token = sample_topk(&logits, temperature, top_k, &mut seed);
            generated.push(next_token);
            // Stop if we hit a likely end-of-sentence token (period '.' or pad 0)
            if next_token == 0 {
                break;
            }
        }
        let n_prompt = prompt_tokens.len();
        if generated.len() > n_prompt {
            generated[n_prompt..].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Chain-of-Thought generation: forces intermediate reasoning before final answer.
    /// Returns (reasoning_tokens, answer_tokens).
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
            "Transformer | d_model={} | heads={} | layers={} | d_ff={} | max_seq={} | vocab={} | train_steps={}",
            self.d_model, self.n_heads, self.n_layers, self.d_ff, self.max_seq_len,
            self.vocab_size, self.n_train_steps,
        )
    }
}
