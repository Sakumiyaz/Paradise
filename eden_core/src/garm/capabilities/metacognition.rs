// EDEN GARM Metacognition — Monitors subsystem errors and adjusts global parameters
// The system knows when its own modules are failing and compensates.

pub struct Metacognition {
    pub swarm_error_ema: f32,
    pub self_model_error_ema: f32,
    pub morpho_concept_delta: i32, // change in concept count over window
    pub last_concept_count: usize,
    pub window_ticks: u32,
    pub alpha: f32,
}

impl Metacognition {
    pub fn new() -> Self {
        Metacognition {
            swarm_error_ema: 1.0,
            self_model_error_ema: 1.0,
            morpho_concept_delta: 0,
            last_concept_count: 0,
            window_ticks: 0,
            alpha: 0.1,
        }
    }

    pub fn observe(&mut self, swarm_err: f32, self_err: f32, concept_count: usize) {
        self.swarm_error_ema = self.alpha * swarm_err + (1.0 - self.alpha) * self.swarm_error_ema;
        self.self_model_error_ema =
            self.alpha * self_err + (1.0 - self.alpha) * self.self_model_error_ema;
        self.window_ticks += 1;
        if self.window_ticks >= 10 {
            self.morpho_concept_delta = concept_count as i32 - self.last_concept_count as i32;
            self.last_concept_count = concept_count;
            self.window_ticks = 0;
        }
    }

    /// Returns recommended mutation_strength increase if swarm is stagnating.
    pub fn recommended_mutation_boost(&self) -> f32 {
        if self.swarm_error_ema > 2.0 {
            0.05
        } else {
            0.0
        }
    }

    /// Returns true if a specialist should be disabled (error > 2x ensemble).
    pub fn should_disable_specialist(&self, specialist_error: f32) -> bool {
        specialist_error > self.swarm_error_ema * 2.0 && self.swarm_error_ema > 1.0
    }

    /// Returns true if the system is in "exploration mode" (high self-model error).
    pub fn exploration_mode(&self) -> bool {
        self.self_model_error_ema > 1.5
    }

    pub fn status(&self) -> String {
        format!(
            "Meta | swarm_err={:.3} | self_err={:.3} | concept_delta={} | explore={}",
            self.swarm_error_ema,
            self.self_model_error_ema,
            self.morpho_concept_delta,
            self.exploration_mode()
        )
    }
}
