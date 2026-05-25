// EDEN GARM — Hierarchical Attention
// 4 niveles: token -> objeto -> episodio -> concepto.
// Cada nivel produce un vector resumen que alimenta al siguiente.
// Simula la jerarquía cortical: V1 -> IT -> hipocampus -> PFC.

#[derive(Clone, Debug)]
pub struct HierarchicalAttention {
    pub token_dim: usize,
    pub object_dim: usize,
    pub episode_dim: usize,
    pub concept_dim: usize,
    pub token_summary: Vec<f32>,
    pub object_summary: Vec<f32>,
    pub episode_summary: Vec<f32>,
    pub concept_summary: Vec<f32>,
    pub n_passes: u64,
}

impl HierarchicalAttention {
    pub fn new(
        token_dim: usize,
        object_dim: usize,
        episode_dim: usize,
        concept_dim: usize,
    ) -> Self {
        HierarchicalAttention {
            token_dim,
            object_dim,
            episode_dim,
            concept_dim,
            token_summary: vec![0.0f32; token_dim],
            object_summary: vec![0.0f32; object_dim],
            episode_summary: vec![0.0f32; episode_dim],
            concept_summary: vec![0.0f32; concept_dim],
            n_passes: 0,
        }
    }

    /// Level 1: pool tokens into object representation (mean + max).
    pub fn tokens_to_objects(&mut self, tokens: &[Vec<f32>]) {
        if tokens.is_empty() {
            return;
        }
        let n = tokens.len() as f32;
        self.token_summary = vec![0.0f32; self.token_dim];
        for t in tokens {
            for j in 0..self.token_dim.min(t.len()) {
                self.token_summary[j] += t[j];
            }
        }
        for j in 0..self.token_dim {
            self.token_summary[j] /= n;
        }
        // Project token_summary -> object_summary via simple linear map (identity-ish)
        for j in 0..self.object_dim.min(self.token_dim) {
            self.object_summary[j] = self.token_summary[j];
        }
    }

    /// Level 2: pool objects into episode representation.
    pub fn objects_to_episodes(&mut self, objects: &[Vec<f32>]) {
        if objects.is_empty() {
            return;
        }
        let n = objects.len() as f32;
        let mut obj_pooled = vec![0.0f32; self.object_dim];
        for o in objects {
            for j in 0..self.object_dim.min(o.len()) {
                obj_pooled[j] += o[j];
            }
        }
        for j in 0..self.object_dim {
            obj_pooled[j] /= n;
        }
        // Project obj_pooled -> episode_summary
        for j in 0..self.episode_dim.min(self.object_dim) {
            self.episode_summary[j] = obj_pooled[j];
        }
    }

    /// Level 3: pool episodes into concept representation.
    pub fn episodes_to_concepts(&mut self, episodes: &[Vec<f32>]) {
        if episodes.is_empty() {
            return;
        }
        let n = episodes.len() as f32;
        let mut ep_pooled = vec![0.0f32; self.episode_dim];
        for e in episodes {
            for j in 0..self.episode_dim.min(e.len()) {
                ep_pooled[j] += e[j];
            }
        }
        for j in 0..self.episode_dim {
            ep_pooled[j] /= n;
        }
        for j in 0..self.concept_dim.min(self.episode_dim) {
            self.concept_summary[j] = ep_pooled[j];
        }
    }

    /// Top-down modulation: concept_summary gates lower levels.
    pub fn top_down_gate(&self, lower: &mut [f32]) {
        let gate = self
            .concept_summary
            .iter()
            .map(|v| v * v)
            .sum::<f32>()
            .sqrt()
            .min(1.0);
        for x in lower.iter_mut() {
            *x *= gate;
        }
    }

    pub fn full_pass(&mut self, tokens: &[Vec<f32>], objects: &[Vec<f32>], episodes: &[Vec<f32>]) {
        self.tokens_to_objects(tokens);
        self.objects_to_episodes(objects);
        self.episodes_to_concepts(episodes);
        self.n_passes += 1;
    }

    pub fn status(&self) -> String {
        let concept_activity = self
            .concept_summary
            .iter()
            .map(|v| v * v)
            .sum::<f32>()
            .sqrt();
        format!(
            "HierAttn | passes={} | tok={} obj={} ep={} conc={} | concept_act={:.3}",
            self.n_passes,
            self.token_dim,
            self.object_dim,
            self.episode_dim,
            self.concept_dim,
            concept_activity
        )
    }
}
