// EDEN GARM — MDL Pruner (Minimum Description Length)
// Podar conceptos cuya complejidad de descripción supera su utilidad predictiva.
// MDL = L(model) + L(data|model). Conceptos con alta MDL se eliminan.

#[derive(Clone, Debug)]
pub struct MDLPruner {
    pub threshold_bits: f32,
    pub n_pruned: u64,
    pub n_evaluated: u64,
    pub last_avg_mdl: f32,
}

impl MDLPruner {
    pub fn new() -> Self {
        MDLPruner {
            threshold_bits: 12.0,
            n_pruned: 0,
            n_evaluated: 0,
            last_avg_mdl: 0.0,
        }
    }

    /// Estimate MDL cost of a concept: complexity of description + error in predictions.
    /// Lower is better. Returns (should_prune, mdl_estimate).
    pub fn evaluate(
        &mut self,
        concept_label: &str,
        n_instances: u32,
        n_correct_preds: u32,
        n_edges: usize,
    ) -> (bool, f32) {
        self.n_evaluated += 1;
        // Description length: log2 of tree depth + edges + label length
        let label_bits = (concept_label.len() as f32) * 8.0;
        let structure_bits = (n_edges as f32) * 4.0; // ~4 bits per edge pointer
        let l_model = label_bits + structure_bits + 16.0; // overhead
                                                          // Data cost: negative log likelihood of predictions
        let n = n_instances.max(1) as f32;
        let accuracy = n_correct_preds as f32 / n;
        let l_data = -n * (accuracy.max(1e-6).ln() + (1.0 - accuracy).max(1e-6).ln());
        let mdl = l_model + l_data.abs();
        self.last_avg_mdl =
            (self.last_avg_mdl * (self.n_evaluated - 1) as f32 + mdl) / self.n_evaluated as f32;
        let should_prune = mdl > self.threshold_bits && n_instances < 3;
        if should_prune {
            self.n_pruned += 1;
        }
        (should_prune, mdl)
    }

    /// Run over a set of concepts and return IDs to prune.
    pub fn prune_batch(&mut self, concepts: &[(String, u32, u32, usize)]) -> Vec<usize> {
        let mut to_prune = Vec::new();
        for (idx, (label, n_inst, n_corr, n_edges)) in concepts.iter().enumerate() {
            let (prune, _mdl) = self.evaluate(label, *n_inst, *n_corr, *n_edges);
            if prune {
                to_prune.push(idx);
            }
        }
        to_prune
    }

    pub fn status(&self) -> String {
        format!(
            "MDL | thr={:.1}bits | eval={} | pruned={} | avg_mdl={:.1}",
            self.threshold_bits, self.n_evaluated, self.n_pruned, self.last_avg_mdl
        )
    }
}
