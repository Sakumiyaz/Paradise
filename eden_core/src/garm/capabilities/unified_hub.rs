// EDEN GARM — Unified Hub (cross-domain representation transfer)
// Proyecta embeddings de todos los dominios (visión, lenguaje, acción, física)
// a un espacio común de baja dimensionalidad (hub_dim).
// Habilita analogías y transferencia entre dominios.

#[derive(Clone, Debug)]
pub struct UnifiedHub {
    pub hub_dim: usize,
    pub projectors: Vec<(String, Vec<Vec<f32>>)>, // domain -> linear projection matrix
    pub hub_cache: Vec<(String, Vec<f32>)>,       // last projected vectors
    pub n_projections: u64,
}

impl UnifiedHub {
    pub fn new(hub_dim: usize) -> Self {
        UnifiedHub {
            hub_dim,
            projectors: Vec::new(),
            hub_cache: Vec::new(),
            n_projections: 0,
        }
    }

    /// Register a domain with random projection.
    pub fn register_domain(&mut self, name: &str, input_dim: usize) {
        let scale = (2.0 / input_dim as f32).sqrt();
        let mut proj = vec![vec![0.0f32; self.hub_dim]; input_dim];
        let mut seed: u64 = 555;
        for i in 0..input_dim {
            for j in 0..self.hub_dim {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let r = ((seed % 1000) as f32 / 1000.0 - 0.5) * 2.0;
                proj[i][j] = r * scale;
            }
        }
        self.projectors.push((name.to_string(), proj));
    }

    /// Project vector from domain to hub space.
    pub fn project(&mut self, domain: &str, vec: &[f32]) -> Vec<f32> {
        let mut out = vec![0.0f32; self.hub_dim];
        if let Some((_, proj)) = self.projectors.iter().find(|(n, _)| n == domain) {
            for j in 0..self.hub_dim {
                let mut sum = 0.0f32;
                for i in 0..vec.len().min(proj.len()) {
                    sum += vec[i] * proj[i][j];
                }
                out[j] = sum;
            }
        }
        self.hub_cache.retain(|(d, _)| d != domain);
        self.hub_cache.push((domain.to_string(), out.clone()));
        self.n_projections += 1;
        out
    }

    /// Find nearest neighbor across domains (analogy transfer).
    pub fn nearest_cross_domain(
        &self,
        query_domain: &str,
        query_vec: &[f32],
    ) -> Option<(String, f32)> {
        let mut best = None;
        let mut best_sim = f32::NEG_INFINITY;
        for (domain, vec) in &self.hub_cache {
            if domain == query_domain {
                continue;
            }
            let sim: f32 = query_vec.iter().zip(vec.iter()).map(|(a, b)| a * b).sum();
            if sim > best_sim {
                best_sim = sim;
                best = Some((domain.clone(), sim));
            }
        }
        best
    }

    pub fn status(&self) -> String {
        format!(
            "Hub | dim={} | domains={} | projections={} | cache={}",
            self.hub_dim,
            self.projectors.len(),
            self.n_projections,
            self.hub_cache.len()
        )
    }
}
