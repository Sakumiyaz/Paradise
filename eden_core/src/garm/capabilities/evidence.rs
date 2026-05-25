// EDEN GARM EvidenceAccumulator — Competencia entre representaciones (Agujero 8)
// Para un input ambiguo, acumula evidencia a favor de distintas interpretaciones.
// La primera que cruza threshold gana (decision con umbral).

#[derive(Clone, Debug)]
pub struct Interpretation {
    pub label: String,
    pub evidence: f32,
    pub source: String,
}

#[derive(Clone, Debug)]
pub struct EvidenceAccumulator {
    pub interpretations: Vec<Interpretation>,
    pub threshold: f32,
    pub decay: f32,
    pub n_decisions: u64,
}

impl EvidenceAccumulator {
    pub fn new() -> Self {
        EvidenceAccumulator {
            interpretations: Vec::new(),
            threshold: 0.6,
            decay: 0.95,
            n_decisions: 0,
        }
    }

    /// Add evidence for an interpretation. Creates if not exists.
    pub fn add_evidence(&mut self, label: &str, amount: f32, source: &str) {
        if let Some(interp) = self.interpretations.iter_mut().find(|i| i.label == label) {
            interp.evidence += amount;
        } else {
            self.interpretations.push(Interpretation {
                label: label.to_string(),
                evidence: amount,
                source: source.to_string(),
            });
        }
    }

    /// Apply decay and check if any interpretation crossed threshold.
    pub fn tick(&mut self) -> Option<Interpretation> {
        for interp in self.interpretations.iter_mut() {
            interp.evidence *= self.decay;
        }
        // Remove weak interpretations
        self.interpretations.retain(|i| i.evidence > 0.05);
        // Find winner
        if let Some(winner) = self.interpretations.iter().max_by(|a, b| {
            a.evidence
                .partial_cmp(&b.evidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            if winner.evidence >= self.threshold {
                self.n_decisions += 1;
                return Some(winner.clone());
            }
        }
        None
    }

    pub fn reset(&mut self) {
        self.interpretations.clear();
    }

    pub fn status(&self) -> String {
        let top = self.interpretations.iter().max_by(|a, b| {
            a.evidence
                .partial_cmp(&b.evidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        format!(
            "Evidence | n={} | thr={:.2} | decisions={} | top={}",
            self.interpretations.len(),
            self.threshold,
            self.n_decisions,
            top.map(|t| format!("{}:{:.2}", t.label, t.evidence))
                .unwrap_or_else(|| "none".to_string())
        )
    }
}
