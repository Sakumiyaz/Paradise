// EDEN GARM Semantics — Fase 1A: Distributional Semantics via Online Co-occurrence + Power-Iteration SVD
// Builds its OWN word embeddings from experience. No pre-trained vectors. No LLM.

use crate::eden_garm::capabilities::nlp::{is_stopword, stem, tokenize};
use std::collections::HashMap;

pub struct DistributionalSemantics {
    pub embed_dim: usize,
    pub max_vocab: usize,
    pub vocab: HashMap<String, usize>,      // word -> index
    pub index_to_word: Vec<String>,         // index -> word
    pub word_counts: Vec<f32>,              // raw frequency per index
    pub cooc: HashMap<(usize, usize), f32>, // sparse PPMI-raw counts
    pub total_windows: u64,
    pub embeddings: Vec<Vec<f32>>, // per-word dense vector
    pub window: usize,
    pub tick_since_compute: u64,
    pub compute_every: u64,
    pub vocab_size: usize,
    /// Bigram counts: (w_prev, w_curr) -> count
    pub bigrams: HashMap<(usize, usize), f32>,
    pub total_bigrams: u64,
}

impl DistributionalSemantics {
    pub fn new(embed_dim: usize, max_vocab: usize) -> Self {
        DistributionalSemantics {
            embed_dim,
            max_vocab,
            vocab: HashMap::new(),
            index_to_word: Vec::new(),
            word_counts: Vec::new(),
            cooc: HashMap::new(),
            total_windows: 0,
            embeddings: Vec::new(),
            window: 5,
            tick_since_compute: 0,
            compute_every: 50,
            vocab_size: 0,
            bigrams: HashMap::new(),
            total_bigrams: 0,
        }
    }

    /// Observe a raw sentence (or any text fragment). Updates co-occurrence counts online.
    /// Bulk-add raw words without stemming or stopword filtering.
    /// Used to pre-seed vocabulary from corpus generators.
    pub fn ensure_raw_words(&mut self, words: &[String]) {
        for w in words {
            if w.is_empty() || self.vocab.contains_key(w) {
                continue;
            }
            if self.vocab_size >= self.max_vocab {
                continue;
            }
            let idx = self.vocab_size;
            self.vocab.insert(w.clone(), idx);
            self.index_to_word.push(w.clone());
            self.word_counts.push(1.0);
            // Random small embedding
            let mut emb = vec![0.0f32; self.embed_dim];
            let mut seed: u64 = (idx as u64).wrapping_mul(1664525).wrapping_add(1013904223);
            for d in 0..self.embed_dim {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                emb[d] = ((seed % 1000) as f32 / 1000.0 - 0.5) * 0.2;
            }
            self.embeddings.push(emb);
            self.vocab_size += 1;
        }
    }

    pub fn observe(&mut self, text: &str) {
        let tokens: Vec<String> = tokenize(text)
            .into_iter()
            .map(|t| stem(&t))
            .filter(|t| !is_stopword(t) && t.len() > 1)
            .collect();
        if tokens.len() < 2 {
            return;
        }

        // Ensure all tokens have an index
        for t in &tokens {
            if !self.vocab.contains_key(t) {
                if self.vocab_size >= self.max_vocab {
                    // Vocab full: skip rare words (heuristic: only add if > min count later)
                    continue;
                }
                let idx = self.vocab_size;
                self.vocab.insert(t.clone(), idx);
                self.index_to_word.push(t.clone());
                self.word_counts.push(0.0);
                self.embeddings.push(vec![0.0f32; self.embed_dim]);
                self.vocab_size += 1;
            }
        }

        // Update co-occurrence counts with symmetric window
        for (i, w) in tokens.iter().enumerate() {
            let wi = match self.vocab.get(w) {
                Some(&x) => x,
                None => continue,
            };
            let start = if i >= self.window { i - self.window } else { 0 };
            let end = (i + self.window + 1).min(tokens.len());
            for j in start..end {
                if i == j {
                    continue;
                }
                let wj = match self.vocab.get(&tokens[j]) {
                    Some(&x) => x,
                    None => continue,
                };
                let key = if wi <= wj { (wi, wj) } else { (wj, wi) };
                *self.cooc.entry(key).or_insert(0.0) += 1.0;
                self.total_windows += 1;
            }
            self.word_counts[wi] += 1.0;
        }

        // Track adjacent bigrams (w_i, w_{i+1}) for sequential statistics
        for i in 0..tokens.len().saturating_sub(1) {
            let a = match self.vocab.get(&tokens[i]) {
                Some(&x) => x,
                None => continue,
            };
            let b = match self.vocab.get(&tokens[i + 1]) {
                Some(&x) => x,
                None => continue,
            };
            *self.bigrams.entry((a, b)).or_insert(0.0) += 1.0;
            self.total_bigrams += 1;
        }
    }

    /// Ingest an entire file line-by-line.
    pub fn ingest_file(&mut self, path: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        for line in content.lines() {
            self.observe(line);
        }
        Ok(())
    }

    /// Compute dense word embeddings via sparse PPMI + power-iteration SVD.
    /// Called periodically (every `compute_every` observations or ticks).
    pub fn compute_embeddings(&mut self) {
        if self.vocab_size < 2 || self.embed_dim == 0 {
            return;
        }

        // Build sparse PPMI matrix entries
        let mut ppmi_entries: Vec<(usize, usize, f32)> = Vec::new();
        let total_pairs = self.total_windows.max(1) as f32;
        let total_words = self.word_counts.iter().sum::<f32>().max(1.0);

        for (&(i, j), &count) in &self.cooc {
            let p_ij = count / total_pairs;
            let p_i = self.word_counts[i] / total_words;
            let p_j = self.word_counts[j] / total_words;
            if p_i > 0.0 && p_j > 0.0 {
                let ppmi = (p_ij / (p_i * p_j)).max(1.0).ln();
                if ppmi > 0.0 {
                    ppmi_entries.push((i, j, ppmi));
                    if i != j {
                        ppmi_entries.push((j, i, ppmi));
                    }
                }
            }
        }

        // Group by row for fast sparse multiplication
        let mut rows: Vec<Vec<(usize, f32)>> = vec![Vec::new(); self.vocab_size];
        for (i, j, v) in ppmi_entries {
            rows[i].push((j, v));
        }

        // Power iteration to extract top `embed_dim` eigenvectors
        let mut components: Vec<Vec<f32>> = Vec::with_capacity(self.embed_dim);
        let mut scratch = vec![0.0f32; self.vocab_size];

        for d in 0..self.embed_dim {
            // Random-ish init using word index
            let mut v: Vec<f32> = (0..self.vocab_size)
                .map(|i| ((i * 7 + d * 13) as f32).sin() * 0.5 + 0.5)
                .collect();
            normalize(&mut v);

            for _ in 0..20 {
                // Multiply sparse matrix into scratch
                for i in 0..self.vocab_size {
                    scratch[i] = 0.0;
                    for &(j, val) in &rows[i] {
                        scratch[i] += val * v[j];
                    }
                }
                // Deflate against previous components
                for prev in &components {
                    let proj = dot(&scratch, prev);
                    for i in 0..self.vocab_size {
                        scratch[i] -= proj * prev[i];
                    }
                }
                v.copy_from_slice(&scratch);
                normalize(&mut v);
            }
            components.push(v);
        }

        // Assign embeddings: each word gets [comp0[i], comp1[i], ...]
        for i in 0..self.vocab_size {
            let mut emb = vec![0.0f32; self.embed_dim];
            for d in 0..self.embed_dim {
                emb[d] = components[d][i];
            }
            self.embeddings[i] = emb;
        }
        self.tick_since_compute = 0;
    }

    /// Get the embedding vector for a word, if known.
    pub fn embedding(&self, word: &str) -> Option<&[f32]> {
        let s = stem(word);
        self.vocab
            .get(&s)
            .and_then(|&idx| self.embeddings.get(idx))
            .map(|v| v.as_slice())
    }

    /// Compute a sentence embedding by averaging word vectors.
    pub fn sentence_embedding(&self, text: &str) -> Vec<f32> {
        let tokens: Vec<String> = tokenize(text)
            .into_iter()
            .map(|t| stem(&t))
            .filter(|t| !is_stopword(t) && t.len() > 1)
            .collect();
        let mut sum = vec![0.0f32; self.embed_dim];
        let mut count = 0usize;
        for t in &tokens {
            if let Some(vec) = self.embedding(t) {
                for d in 0..self.embed_dim {
                    sum[d] += vec[d];
                }
                count += 1;
            }
        }
        if count > 0 {
            for d in 0..self.embed_dim {
                sum[d] /= count as f32;
            }
        }
        sum
    }

    /// Bigram-aware sentence embedding: blends unigram avg + bigram-weighted unigram avg.
    /// Words that appear in frequent bigrams get extra weight, capturing phrase context.
    pub fn bigram_embedding(&self, text: &str) -> Vec<f32> {
        let tokens: Vec<String> = tokenize(text)
            .into_iter()
            .map(|t| stem(&t))
            .filter(|t| !is_stopword(t) && t.len() > 1)
            .collect();
        let mut sum = vec![0.0f32; self.embed_dim];
        let mut total_weight = 0.0f32;
        // unigram contribution (weight=1)
        for t in &tokens {
            if let Some(vec) = self.embedding(t) {
                for d in 0..self.embed_dim {
                    sum[d] += vec[d];
                }
                total_weight += 1.0;
            }
        }
        // bigram contribution: for each adjacent pair, add (avg of their embeddings) * sqrt(bigram_count)
        for i in 0..tokens.len().saturating_sub(1) {
            let (a_idx, b_idx) = match (self.vocab.get(&tokens[i]), self.vocab.get(&tokens[i + 1]))
            {
                (Some(&a), Some(&b)) => (a, b),
                _ => continue,
            };
            let count = self.bigrams.get(&(a_idx, b_idx)).copied().unwrap_or(0.0);
            if count <= 0.0 {
                continue;
            }
            let weight = count.sqrt(); // dampen with sqrt
            let emb_a = self.embedding(&tokens[i]);
            let emb_b = self.embedding(&tokens[i + 1]);
            if let (Some(ea), Some(eb)) = (emb_a, emb_b) {
                for d in 0..self.embed_dim {
                    sum[d] += weight * (ea[d] + eb[d]) * 0.5;
                }
                total_weight += weight;
            }
        }
        if total_weight > 0.0 {
            for d in 0..self.embed_dim {
                sum[d] /= total_weight;
            }
        }
        sum
    }

    /// Find nearest neighbors by cosine similarity.
    pub fn nearest(&self, word: &str, k: usize) -> Vec<(String, f32)> {
        let s = stem(word);
        let q = match self.embedding(&s) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let mut scored: Vec<(String, f32)> = Vec::new();
        for i in 0..self.vocab_size {
            let w = &self.index_to_word[i];
            if w == &s {
                continue;
            }
            let v = &self.embeddings[i];
            let sim = cosine(q, v);
            scored.push((w.clone(), sim));
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    pub fn status(&self) -> String {
        format!(
            "Semantics | vocab={}/{} | cooc={} | dim={} | last_compute={} ticks ago",
            self.vocab_size,
            self.max_vocab,
            self.cooc.len(),
            self.embed_dim,
            self.tick_since_compute
        )
    }

    pub fn n_embeddings(&self) -> usize {
        self.embeddings.len()
    }
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn normalize(v: &mut [f32]) {
    let mag = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag > 1e-8 {
        for x in v.iter_mut() {
            *x /= mag;
        }
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let d = dot(a, b);
    let ma = dot(a, a).sqrt();
    let mb = dot(b, b).sqrt();
    if ma > 1e-8 && mb > 1e-8 {
        d / (ma * mb)
    } else {
        0.0
    }
}
