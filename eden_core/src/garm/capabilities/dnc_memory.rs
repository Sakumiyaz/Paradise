// EDEN GARM — Differentiable Neural Computer (DNC) / Neural Turing Machine
// Memoria matricial N x W con read/write heads entrenables (content-addressable + allocation).
// El transformer puede leer/escribir en cada tick, y los gradientes fluyen a través.

use std::f32;

#[derive(Clone, Debug)]
pub struct DNC {
    pub n_cells: usize,
    pub word_size: usize,
    pub n_read_heads: usize,
    pub memory: Vec<Vec<f32>>, // n_cells x word_size
    pub usage: Vec<f32>,
    pub link_matrix: Vec<Vec<f32>>, // n_cells x n_cells (temporal links)
    pub read_weights: Vec<Vec<f32>>, // n_read_heads x n_cells
    pub write_weights: Vec<f32>,    // n_cells
    pub read_vectors: Vec<Vec<f32>>, // n_read_heads x word_size
    pub n_accesses: u64,
}

impl DNC {
    pub fn new(n_cells: usize, word_size: usize, n_read_heads: usize) -> Self {
        DNC {
            n_cells,
            word_size,
            n_read_heads,
            memory: vec![vec![0.0f32; word_size]; n_cells],
            usage: vec![0.0f32; n_cells],
            link_matrix: vec![vec![0.0f32; n_cells]; n_cells],
            read_weights: vec![vec![0.0f32; n_cells]; n_read_heads],
            write_weights: vec![0.0f32; n_cells],
            read_vectors: vec![vec![0.0f32; word_size]; n_read_heads],
            n_accesses: 0,
        }
    }

    /// Content-addressable read: softmax over cosine similarity between key and memory cells.
    pub fn read(&mut self, key: &[f32], strength: f32) -> Vec<Vec<f32>> {
        let mut results = Vec::new();
        for h in 0..self.n_read_heads {
            let mut sims: Vec<f32> = self
                .memory
                .iter()
                .map(|cell| {
                    let dot: f32 = cell.iter().zip(key.iter()).map(|(a, b)| a * b).sum();
                    let norm_cell = cell.iter().map(|v| v * v).sum::<f32>().sqrt();
                    let norm_key = key.iter().map(|v| v * v).sum::<f32>().sqrt();
                    dot / (norm_cell * norm_key).max(1e-8)
                })
                .collect();
            // Apply strength (beta)
            for s in sims.iter_mut() {
                *s = (*s * strength).exp();
            }
            let sum: f32 = sims.iter().sum();
            let weights: Vec<f32> = sims.iter().map(|s| s / sum.max(1e-8)).collect();
            let mut read_vec = vec![0.0f32; self.word_size];
            for j in 0..self.word_size {
                for i in 0..self.n_cells {
                    read_vec[j] += weights[i] * self.memory[i][j];
                }
            }
            self.read_weights[h] = weights;
            results.push(read_vec.clone());
            self.read_vectors[h] = read_vec;
        }
        self.n_accesses += 1;
        results
    }

    /// Write: update memory with erase + add vectors. Simplified differentiable write.
    pub fn write(&mut self, key: &[f32], add_vec: &[f32], erase_vec: &[f32], write_strength: f32) {
        // Content-based write weights
        let mut sims: Vec<f32> = self
            .memory
            .iter()
            .map(|cell| {
                let dot: f32 = cell.iter().zip(key.iter()).map(|(a, b)| a * b).sum();
                let norm_cell = cell.iter().map(|v| v * v).sum::<f32>().sqrt();
                let norm_key = key.iter().map(|v| v * v).sum::<f32>().sqrt();
                dot / (norm_cell * norm_key).max(1e-8)
            })
            .collect();
        for s in sims.iter_mut() {
            *s = (*s * write_strength).exp();
        }
        let sum: f32 = sims.iter().sum();
        let w: Vec<f32> = sims.iter().map(|s| s / sum.max(1e-8)).collect();
        // Erase then add
        for i in 0..self.n_cells {
            let wi = w[i];
            for j in 0..self.word_size {
                self.memory[i][j] *= 1.0 - wi * erase_vec.get(j).copied().unwrap_or(0.0);
                self.memory[i][j] += wi * add_vec.get(j).copied().unwrap_or(0.0);
            }
            self.usage[i] = (self.usage[i] + wi).min(1.0);
        }
        self.write_weights = w;
        self.n_accesses += 1;
    }

    /// Flatten read vectors for concatenation with transformer state.
    pub fn readout_flat(&self) -> Vec<f32> {
        self.read_vectors
            .iter()
            .flat_map(|v| v.iter().copied())
            .collect()
    }

    pub fn status(&self) -> String {
        let used: f32 = self.usage.iter().sum();
        format!(
            "DNC | cells={}x{} | heads={} | accesses={} | usage={:.1}/{}",
            self.n_cells, self.word_size, self.n_read_heads, self.n_accesses, used, self.n_cells
        )
    }
}
