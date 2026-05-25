// EDEN GARM — Working Memory (addressable persistent memory slots)
// Matrix N x D. Read/write via content-addressable attention.
// El transformer puede leer/escribir slots en cada tick.

use std::f32;

#[derive(Clone, Debug)]
pub struct WorkingMemory {
    pub n_slots: usize,
    pub dim: usize,
    pub memory: Vec<Vec<f32>>, // n_slots x dim
    pub usage: Vec<f32>,       // 0..1 per slot
    pub n_reads: u64,
    pub n_writes: u64,
}

impl WorkingMemory {
    pub fn new(n_slots: usize, dim: usize) -> Self {
        let mut seed: u64 = 42;
        let memory: Vec<Vec<f32>> = (0..n_slots)
            .map(|_| {
                (0..dim)
                    .map(|_| {
                        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                        ((seed % 1000) as f32 / 1000.0 - 0.5) * 0.01
                    })
                    .collect()
            })
            .collect();
        WorkingMemory {
            n_slots,
            dim,
            memory,
            usage: vec![0.0f32; n_slots],
            n_reads: 0,
            n_writes: 0,
        }
    }

    /// Content-addressed read: softmax over query·slot similarity.
    pub fn read(&self, query: &[f32]) -> Vec<f32> {
        let sims: Vec<f32> = self
            .memory
            .iter()
            .enumerate()
            .map(|(i, slot)| {
                let dot: f32 = slot.iter().zip(query.iter()).map(|(a, b)| a * b).sum();
                dot * self.usage[i] // bias toward used slots
            })
            .collect();
        let max_sim = sims.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exps: Vec<f32> = sims.iter().map(|s| (s - max_sim).exp()).collect();
        let sum: f32 = exps.iter().sum();
        let weights: Vec<f32> = exps.iter().map(|e| e / sum.max(1e-8)).collect();
        let mut out = vec![0.0f32; self.dim];
        for j in 0..self.dim {
            for i in 0..self.n_slots {
                out[j] += weights[i] * self.memory[i][j];
            }
        }
        out
    }

    /// Write to the least-used slot (or most similar if high similarity).
    pub fn write(&mut self, value: &[f32]) {
        let sims: Vec<f32> = self
            .memory
            .iter()
            .map(|slot| {
                slot.iter()
                    .zip(value.iter())
                    .map(|(a, b)| a * b)
                    .sum::<f32>()
            })
            .collect();
        let best = sims
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let least_used = self
            .usage
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        // Write to best if similarity > 0.5, else to least used
        let target = if sims[best] > 0.5 { best } else { least_used };
        let lr = 0.3;
        for j in 0..self.dim {
            self.memory[target][j] = (1.0 - lr) * self.memory[target][j] + lr * value[j];
        }
        self.usage[target] = (self.usage[target] + 0.2).min(1.0);
        self.n_writes += 1;
    }

    /// Flatten all memory into a single vector (for feeding to transformer).
    pub fn flatten(&self) -> Vec<f32> {
        self.memory.iter().flat_map(|v| v.iter().copied()).collect()
    }

    pub fn status(&self) -> String {
        let avg_usage: f32 = self.usage.iter().sum::<f32>() / self.n_slots.max(1) as f32;
        format!(
            "WM | slots={}x{} | reads={} | writes={} | usage={:.2}",
            self.n_slots, self.dim, self.n_reads, self.n_writes, avg_usage
        )
    }
}
