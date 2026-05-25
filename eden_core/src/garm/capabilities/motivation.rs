// EDEN GARM Motivation — Fase 4: Learning Progress + Autonomous Curriculum
// Homeostatic drives + per-domain learning progress tracking.
// The system monitors its own error curves and switches domains when stagnating.
// No external commands: the system "wants" to minimize its own discomfort.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Drives {
    pub curiosity: f32,
    pub efficiency: f32,
    pub stability: f32,
    pub competence: f32, // Fase 4: satisfaction from mastering domains
}

/// Tracks error history for a single cognitive domain.
#[derive(Clone, Debug)]
pub struct DomainTracker {
    pub name: String,
    pub errors: Vec<f32>, // rolling window of recent errors
    pub max_window: usize,
    pub ticks_in_domain: u64,
}

impl DomainTracker {
    pub fn new(name: &str, max_window: usize) -> Self {
        DomainTracker {
            name: name.to_string(),
            errors: Vec::with_capacity(max_window),
            max_window,
            ticks_in_domain: 0,
        }
    }

    pub fn observe_error(&mut self, err: f32) {
        self.errors.push(err.clamp(0.0, 1.0));
        if self.errors.len() > self.max_window {
            self.errors.remove(0);
        }
        self.ticks_in_domain += 1;
    }

    /// Learning progress = negative derivative of error.
    /// Positive means error is decreasing (learning). Negative means stagnation.
    pub fn learning_progress(&self) -> f32 {
        if self.errors.len() < 10 {
            return 0.0;
        }
        let mid = self.errors.len() / 2;
        let early: f32 = self.errors[..mid].iter().sum::<f32>() / mid.max(1) as f32;
        let late: f32 =
            self.errors[mid..].iter().sum::<f32>() / (self.errors.len() - mid).max(1) as f32;
        early - late // positive = error went down = learning
    }

    pub fn current_error(&self) -> f32 {
        self.errors.last().copied().unwrap_or(0.5)
    }

    pub fn is_stagnating(&self, threshold: f32) -> bool {
        self.learning_progress() < threshold && self.errors.len() >= self.max_window
    }
}

#[derive(Clone, Debug)]
pub struct MotivationEngine {
    pub drives: Drives,
    pub discomfort: f32,
    pub history: Vec<f32>,
    pub max_history: usize,
    pub domains: HashMap<String, DomainTracker>,
    pub current_focus: String,
    pub stagnation_threshold: f32,
    pub switch_cooldown: u64,
    pub ticks_since_switch: u64,
    pub competence_targets: HashMap<String, f32>, // target error per domain
}

impl MotivationEngine {
    pub fn new() -> Self {
        let mut domains = HashMap::new();
        domains.insert(
            "prediction".to_string(),
            DomainTracker::new("prediction", 50),
        );
        domains.insert("vision".to_string(), DomainTracker::new("vision", 50));
        domains.insert("semantics".to_string(), DomainTracker::new("semantics", 50));
        domains.insert("world".to_string(), DomainTracker::new("world", 50));
        let mut competence_targets = HashMap::new();
        competence_targets.insert("prediction".to_string(), 0.1);
        competence_targets.insert("vision".to_string(), 0.2);
        competence_targets.insert("semantics".to_string(), 0.3);
        competence_targets.insert("world".to_string(), 0.25);
        MotivationEngine {
            drives: Drives {
                curiosity: 0.5,
                efficiency: 0.5,
                stability: 0.5,
                competence: 0.5,
            },
            discomfort: 0.5,
            history: Vec::with_capacity(1000),
            max_history: 1000,
            domains,
            current_focus: "prediction".to_string(),
            stagnation_threshold: 0.005,
            switch_cooldown: 20,
            ticks_since_switch: 0,
            competence_targets,
        }
    }

    /// Update drives based on recent experience and learning progress across domains.
    pub fn update(&mut self, prediction_error: f32, n_actions: usize, tick_time_ms: f32) {
        // Curiosity: grows when global error is high, but modulated by stagnation
        self.drives.curiosity =
            (self.drives.curiosity * 0.9 + prediction_error.clamp(0.0, 1.0) * 0.1).clamp(0.0, 1.0);
        // Efficiency: inverse of actions + time
        let load = ((n_actions as f32 + tick_time_ms / 100.0) / 20.0).clamp(0.0, 1.0);
        self.drives.efficiency = (1.0 - load).clamp(0.0, 1.0);
        // Stability: high when error is low
        self.drives.stability = (1.0 - prediction_error.clamp(0.0, 1.0)).clamp(0.0, 1.0);
        // Competence: average progress across all domains (0.5 = neutral)
        let avg_progress: f32 = self
            .domains
            .values()
            .map(|d| d.learning_progress())
            .sum::<f32>()
            / self.domains.len().max(1) as f32;
        self.drives.competence = (0.5 + avg_progress * 2.0).clamp(0.0, 1.0);

        // Discomfort: weighted deficits + stagnation penalty
        let stagnation_penalty: f32 = if self
            .current_focus_domain()
            .is_stagnating(self.stagnation_threshold)
        {
            0.2
        } else {
            0.0
        };
        self.discomfort = ((1.0 - self.drives.curiosity) * 0.25
            + (1.0 - self.drives.efficiency) * 0.2
            + (1.0 - self.drives.stability) * 0.2
            + (1.0 - self.drives.competence) * 0.15
            + stagnation_penalty)
            .clamp(0.0, 1.0);

        self.history.push(self.discomfort);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        self.ticks_since_switch += 1;
    }

    /// Observe domain-specific error. Called by each subsystem.
    pub fn observe_domain(&mut self, domain: &str, error: f32) {
        if let Some(tracker) = self.domains.get_mut(domain) {
            tracker.observe_error(error);
        }
    }

    /// Auto-switch focus domain if current one stagnates.
    /// Returns the new focus domain name if switched.
    pub fn maybe_switch_focus(&mut self) -> Option<String> {
        if self.ticks_since_switch < self.switch_cooldown {
            return None;
        }
        if !self
            .current_focus_domain()
            .is_stagnating(self.stagnation_threshold)
        {
            return None;
        }
        // Find domain with highest learning progress
        let best = self
            .domains
            .iter()
            .filter(|(name, _)| *name != &self.current_focus)
            .max_by(|(_, a), (_, b)| {
                a.learning_progress()
                    .partial_cmp(&b.learning_progress())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        if let Some((name, tracker)) = best {
            if tracker.learning_progress() > self.current_focus_domain().learning_progress() {
                let old = self.current_focus.clone();
                self.current_focus = name.clone();
                self.ticks_since_switch = 0;
                return Some(old);
            }
        }
        None
    }

    pub fn current_focus_domain(&self) -> &DomainTracker {
        self.domains
            .get(&self.current_focus)
            .unwrap_or_else(|| self.domains.values().next().unwrap())
    }

    pub fn should_act_autonomously(&self, idle_ticks: u64) -> bool {
        self.discomfort > 0.6 && idle_ticks > 5
    }

    pub fn dominant_drive(&self) -> &'static str {
        let d_cur = 1.0 - self.drives.curiosity;
        let d_eff = 1.0 - self.drives.efficiency;
        let d_stab = 1.0 - self.drives.stability;
        let d_comp = 1.0 - self.drives.competence;
        if d_cur >= d_eff && d_cur >= d_stab && d_cur >= d_comp {
            "curiosity"
        } else if d_eff >= d_stab && d_eff >= d_comp {
            "efficiency"
        } else if d_stab >= d_comp {
            "stability"
        } else {
            "competence"
        }
    }

    /// Generate a synthetic training target: domain with highest error = best learning opportunity.
    pub fn best_learning_opportunity(&self) -> (String, f32) {
        let mut best_name = "prediction".to_string();
        let mut best_err = 0.0f32;
        for (name, tracker) in &self.domains {
            let err = tracker.current_error();
            if err > best_err {
                best_err = err;
                best_name = name.clone();
            }
        }
        (best_name, best_err)
    }

    pub fn status(&self) -> String {
        let focus = self.current_focus_domain();
        format!("Motivation | curiosity={:.2} | efficiency={:.2} | stability={:.2} | competence={:.2} | discomfort={:.2} | focus={} | progress={:.4} | dominant={}",
            self.drives.curiosity, self.drives.efficiency, self.drives.stability, self.drives.competence,
            self.discomfort, self.current_focus, focus.learning_progress(), self.dominant_drive())
    }
}
