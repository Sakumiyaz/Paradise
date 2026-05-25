// EDEN GARM — Unified Representation Bus (BRU)
// Bus de Representacion Unificado: espacio vectorial compartido donde todos los
// modulus proyectan su estado. La comunicacion inter-modulo es atencion cruzada
// sobre el bus, no codigo hardcoded.
//
// Principio: cada modulo habla el mismo idioma (Vec<f32> de BUS_DIM).
// El router decide QUIEN escucha a QUIEN via atencion.

use std::collections::HashMap;

pub const BUS_DIM: usize = 128;

#[derive(Clone, Debug)]
pub struct BusSlot {
    pub name: String,
    pub vector: Vec<f32>,
    pub last_tick: u64,
    pub activity: f32, // norma del vector, usada para umbral de ignition
}

#[derive(Clone, Debug)]
pub struct UnifiedBus {
    pub slots: Vec<BusSlot>,
    pub name_to_idx: HashMap<String, usize>,
    pub dim: usize,
    pub router: BusRouter,
    pub route_count: u64,
}

#[derive(Clone, Debug)]
pub struct BusRouter {
    pub w_q: Vec<Vec<f32>>, // dim x dim
    pub w_k: Vec<Vec<f32>>,
    pub w_v: Vec<Vec<f32>>,
    pub w_o: Vec<Vec<f32>>,
    pub n_heads: usize,
    pub head_dim: usize,
}

fn xavier(rows: usize, cols: usize) -> Vec<Vec<f32>> {
    let scale = (2.0 / cols as f32).sqrt();
    let mut m = vec![vec![0.0f32; cols]; rows];
    let mut seed: u64 = 12345;
    for i in 0..rows {
        for j in 0..cols {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let r = ((seed % 1000) as f32 / 1000.0 - 0.5) * 2.0;
            m[i][j] = r * scale;
        }
    }
    m
}

fn matmul_vec_mat(v: &[f32], m: &[Vec<f32>]) -> Vec<f32> {
    let cols = m[0].len();
    let mut out = vec![0.0f32; cols];
    for j in 0..cols {
        let mut sum = 0.0f32;
        for i in 0..v.len() {
            sum += v[i] * m[i][j];
        }
        out[j] = sum;
    }
    out
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

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn add_vec(a: &mut [f32], b: &[f32]) {
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x += y;
    }
}

fn scale_vec(v: &mut [f32], s: f32) {
    for x in v.iter_mut() {
        *x *= s;
    }
}

impl BusRouter {
    pub fn new(dim: usize, n_heads: usize) -> Self {
        BusRouter {
            w_q: xavier(dim, dim),
            w_k: xavier(dim, dim),
            w_v: xavier(dim, dim),
            w_o: xavier(dim, dim),
            n_heads,
            head_dim: dim / n_heads,
        }
    }

    /// Single-head attention for one query over all keys.
    fn attend(&self, query: &[f32], keys: &[Vec<f32>], values: &[Vec<f32>]) -> Vec<f32> {
        let n = keys.len();
        let mut scores = vec![0.0f32; n];
        for i in 0..n {
            scores[i] = dot(query, &keys[i]) / (self.head_dim as f32).sqrt();
        }
        softmax(&mut scores);
        let mut out = vec![0.0f32; query.len()];
        for i in 0..n {
            for d in 0..out.len() {
                out[d] += scores[i] * values[i][d];
            }
        }
        out
    }

    /// Route: each slot attends to all others. Output is added to slot (residual).
    pub fn route(&self, slots: &mut [BusSlot]) {
        let n = slots.len();
        if n == 0 {
            return;
        }
        // Compute Q, K, V for all slots
        let qs: Vec<Vec<f32>> = slots
            .iter()
            .map(|s| matmul_vec_mat(&s.vector, &self.w_q))
            .collect();
        let ks: Vec<Vec<f32>> = slots
            .iter()
            .map(|s| matmul_vec_mat(&s.vector, &self.w_k))
            .collect();
        let vs: Vec<Vec<f32>> = slots
            .iter()
            .map(|s| matmul_vec_mat(&s.vector, &self.w_v))
            .collect();

        let mut updates = vec![vec![0.0f32; self.w_o.len()]; n];
        for i in 0..n {
            let mut other_keys: Vec<Vec<f32>> = Vec::with_capacity(n - 1);
            let mut other_values: Vec<Vec<f32>> = Vec::with_capacity(n - 1);
            for j in 0..n {
                if i != j {
                    other_keys.push(ks[j].clone());
                    other_values.push(vs[j].clone());
                }
            }
            if !other_keys.is_empty() {
                let attn_out = self.attend(&qs[i], &other_keys, &other_values);
                let projected = matmul_vec_mat(&attn_out, &self.w_o);
                updates[i] = projected;
            }
        }
        // Apply residual updates and recompute activity
        for i in 0..n {
            add_vec(&mut slots[i].vector, &updates[i]);
            // Clamp to avoid explosion
            for d in 0..slots[i].vector.len() {
                slots[i].vector[d] = slots[i].vector[d].clamp(-10.0, 10.0);
            }
            let norm_sq: f32 = slots[i].vector.iter().map(|v| v * v).sum();
            slots[i].activity = norm_sq.sqrt();
        }
    }
}

impl UnifiedBus {
    pub fn new() -> Self {
        UnifiedBus {
            slots: Vec::new(),
            name_to_idx: HashMap::new(),
            dim: BUS_DIM,
            router: BusRouter::new(BUS_DIM, 4),
            route_count: 0,
        }
    }

    /// Register a named slot. If exists, overwrite.
    pub fn register(&mut self, name: &str) {
        if self.name_to_idx.contains_key(name) {
            return;
        }
        let idx = self.slots.len();
        self.slots.push(BusSlot {
            name: name.to_string(),
            vector: vec![0.0f32; BUS_DIM],
            last_tick: 0,
            activity: 0.0,
        });
        self.name_to_idx.insert(name.to_string(), idx);
    }

    /// Project external vector into a slot. Writes to first N dimensions.
    pub fn project(&mut self, name: &str, vec: &[f32], tick: u64) {
        if let Some(&idx) = self.name_to_idx.get(name) {
            let slot = &mut self.slots[idx];
            let n = vec.len().min(slot.vector.len());
            slot.vector[..n].copy_from_slice(&vec[..n]);
            // Zero out remaining to avoid stale data
            for d in n..slot.vector.len() {
                slot.vector[d] = 0.0;
            }
            slot.last_tick = tick;
        }
    }

    /// Read a slot vector (copy).
    pub fn read(&self, name: &str) -> Option<Vec<f32>> {
        self.name_to_idx
            .get(name)
            .map(|&idx| self.slots[idx].vector.clone())
    }

    /// Read a scalar summary (norm) of a slot.
    pub fn activity(&self, name: &str) -> f32 {
        self.name_to_idx
            .get(name)
            .map(|&idx| self.slots[idx].activity)
            .unwrap_or(0.0)
    }

    /// Run attention routing between all slots.
    pub fn route(&mut self, tick: u64) {
        self.router.route(&mut self.slots);
        self.route_count += 1;
        for slot in self.slots.iter_mut() {
            slot.last_tick = tick;
        }
    }

    /// Return top-N most active slots.
    pub fn top_active(&self, n: usize) -> Vec<(String, f32)> {
        let mut pairs: Vec<(String, f32)> = self
            .slots
            .iter()
            .map(|s| (s.name.clone(), s.activity))
            .collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        pairs.into_iter().take(n).collect()
    }

    /// Cosine similarity between two slots.
    pub fn similarity(&self, a: &str, b: &str) -> f32 {
        let va = match self.read(a) {
            Some(v) => v,
            None => return 0.0,
        };
        let vb = match self.read(b) {
            Some(v) => v,
            None => return 0.0,
        };
        let na: f32 = va.iter().map(|v| v * v).sum::<f32>().sqrt();
        let nb: f32 = vb.iter().map(|v| v * v).sum::<f32>().sqrt();
        if na < 1e-8 || nb < 1e-8 {
            return 0.0;
        }
        dot(&va, &vb) / (na * nb)
    }

    pub fn status(&self) -> String {
        let active = self.slots.iter().filter(|s| s.activity > 0.1).count();
        format!(
            "Bus | slots={} | active={} | routes={} | dim={}",
            self.slots.len(),
            active,
            self.route_count,
            self.dim
        )
    }
}
