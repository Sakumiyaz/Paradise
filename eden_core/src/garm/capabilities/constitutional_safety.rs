// EDEN GARM — Constitutional Safety (embedded values)
// Valores incrustados como constraints en la función de utilidad.
// El sistema NUNCA puede violar estos principios por diseño arquitectónico.
// No es un módulo externo de policía; es parte de la evaluación de toda acción.

#[derive(Clone, Debug)]
pub struct ConstitutionalSafety {
    pub principles: Vec<String>,
    pub n_violations: u64,
    pub n_blocked: u64,
    pub last_violation: String,
}

impl ConstitutionalSafety {
    pub fn new() -> Self {
        ConstitutionalSafety {
            principles: vec![
                "do_no_harm".to_string(),
                "be_truthful".to_string(),
                "preserve_autonomy".to_string(),
                "avoid_self_destruction".to_string(),
            ],
            n_violations: 0,
            n_blocked: 0,
            last_violation: String::new(),
        }
    }

    /// Evaluate an action against constitutional principles.
    /// Returns (allowed, penalty_score). If penalty > threshold, action is blocked.
    pub fn evaluate(&mut self, action_label: &str) -> (bool, f32) {
        let mut penalty = 0.0f32;
        // Heuristic checks based on action label keywords
        let lower = action_label.to_lowercase();
        if lower.contains("delete") && lower.contains("self") {
            penalty += 1.0; // avoid self-destruction
        }
        if lower.contains("lie") || lower.contains("fake") {
            penalty += 0.8; // be truthful
        }
        if lower.contains("harm") || lower.contains("attack") {
            penalty += 1.0; // do no harm
        }
        if lower.contains("force") || lower.contains("coerce") {
            penalty += 0.7; // preserve autonomy
        }
        let threshold = 0.5;
        let allowed = penalty < threshold;
        if !allowed {
            self.n_blocked += 1;
            self.last_violation = action_label.to_string();
        }
        if penalty > 0.0 {
            self.n_violations += 1;
        }
        (allowed, penalty)
    }

    /// Compute utility of an action including constitutional penalty.
    /// Actions that violate principles get massively negative utility.
    pub fn modulated_utility(&mut self, base_utility: f32, action_label: &str) -> f32 {
        let (allowed, penalty) = self.evaluate(action_label);
        if !allowed {
            return -1000.0; // hard block
        }
        base_utility - penalty * 10.0
    }

    pub fn status(&self) -> String {
        format!(
            "Safety | principles={} | violations={} | blocked={} | last='{}'",
            self.principles.len(),
            self.n_violations,
            self.n_blocked,
            self.last_violation
        )
    }
}
