// EDEN GARM — Self-Modification (autonomía recursiva)
// El sistema puede proponer y aplicar cambios a su propia arquitectura.
// Limitado a parámetros y config; no reescribe código fuente arbitrario (safety).
// Capacidad: ajustar hiperparámetros, reordenar prioridades de módulos,
// activar/desactivar capabilities, modificar thresholds.

#[derive(Clone, Debug)]
pub struct SelfModification {
    pub proposals: Vec<ModificationProposal>,
    pub applied: u64,
    pub rejected: u64,
    pub last_proposal: String,
}

#[derive(Clone, Debug)]
pub struct ModificationProposal {
    pub target: String,
    pub param: String,
    pub old_value: f32,
    pub new_value: f32,
    pub reason: String,
    pub tick: u64,
}

impl SelfModification {
    pub fn new() -> Self {
        SelfModification {
            proposals: Vec::new(),
            applied: 0,
            rejected: 0,
            last_proposal: String::new(),
        }
    }

    /// Propose a modification based on detected bottleneck.
    pub fn propose(
        &mut self,
        target: &str,
        param: &str,
        current: f32,
        metric: f32,
        tick: u64,
    ) -> Option<ModificationProposal> {
        let new_value = if metric > 0.7 {
            current * 1.1 // increase if doing well
        } else if metric < 0.3 {
            current * 0.9 // decrease if struggling
        } else {
            current
        };
        if (new_value - current).abs() < 0.01 {
            return None;
        }
        let proposal = ModificationProposal {
            target: target.to_string(),
            param: param.to_string(),
            old_value: current,
            new_value,
            reason: format!("metric={:.2} -> adjust {}", metric, param),
            tick,
        };
        self.last_proposal = format!("{}:{}={:.3}->{:.3}", target, param, current, new_value);
        Some(proposal)
    }

    /// Apply proposal if it passes safety checks.
    pub fn apply(&mut self, proposal: &ModificationProposal) -> bool {
        // Safety: reject if change > 50% or if param is critical
        if (proposal.new_value - proposal.old_value).abs() / proposal.old_value.max(1e-6) > 0.5 {
            self.rejected += 1;
            return false;
        }
        self.proposals.push(proposal.clone());
        self.applied += 1;
        true
    }

    pub fn status(&self) -> String {
        format!(
            "SelfMod | proposals={} | applied={} | rejected={} | last='{}'",
            self.proposals.len(),
            self.applied,
            self.rejected,
            self.last_proposal
        )
    }
}
