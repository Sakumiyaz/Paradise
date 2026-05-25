// EDEN GARM Gate — Veto / Inhibición (Agujero 6)
// Cada acción propuesta pasa por un gate de confianza antes de ejecutarse.
// Si confianza < threshold, la acción se inhibe (no se ejecuta).
// El gate aprende de los resultados: si inhibir fue correcto, refuerza el umbral.

#[derive(Clone, Debug)]
pub struct ActionProposal {
    pub source: String,
    pub action_label: String,
    pub confidence: f32,
    pub cost_estimate: f32,
}

#[derive(Clone, Debug)]
pub struct VetoGate {
    pub threshold: f32,
    pub lr: f32,
    pub n_vetoed: u64,
    pub n_allowed: u64,
    pub last_veto_reason: String,
    pub veto_history: Vec<(String, String, f32, bool)>, // source, action, confidence, was_correct
}

impl VetoGate {
    pub fn new() -> Self {
        VetoGate {
            threshold: 0.35,
            lr: 0.05,
            n_vetoed: 0,
            n_allowed: 0,
            last_veto_reason: String::new(),
            veto_history: Vec::new(),
        }
    }

    /// Evaluate a proposed action. Returns true if allowed, false if vetoed.
    pub fn evaluate(&mut self, proposal: &ActionProposal) -> bool {
        let effective_threshold = self.threshold + proposal.cost_estimate * 0.1;
        if proposal.confidence < effective_threshold {
            self.n_vetoed += 1;
            self.last_veto_reason = format!(
                "{}: conf={:.2} < thr={:.2}",
                proposal.source, proposal.confidence, effective_threshold
            );
            self.veto_history.push((
                proposal.source.clone(),
                proposal.action_label.clone(),
                proposal.confidence,
                false,
            ));
            false
        } else {
            self.n_allowed += 1;
            self.veto_history.push((
                proposal.source.clone(),
                proposal.action_label.clone(),
                proposal.confidence,
                true,
            ));
            true
        }
    }

    /// Feedback after seeing outcome. If vetoed but outcome would have been good,
    /// lower threshold. If allowed but outcome was bad, raise threshold.
    pub fn feedback(&mut self, _proposal: &ActionProposal, outcome_good: bool) {
        if let Some(last) = self.veto_history.last() {
            let was_vetoed = !last.3;
            if was_vetoed && outcome_good {
                // Veto was wrong: lower threshold
                self.threshold -= self.lr;
            } else if !was_vetoed && !outcome_good {
                // Allow was wrong: raise threshold
                self.threshold += self.lr;
            }
            self.threshold = self.threshold.clamp(0.1, 0.9);
        }
    }

    pub fn status(&self) -> String {
        format!(
            "Gate | thr={:.2} | vetoed={} | allowed={} | last='{}'",
            self.threshold, self.n_vetoed, self.n_allowed, self.last_veto_reason
        )
    }
}
