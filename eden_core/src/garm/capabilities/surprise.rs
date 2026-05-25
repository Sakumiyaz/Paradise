// EDEN GARM SurpriseDetector — Sorpresa computacional global (Agujero 9)
// Un evento es sorprendente si el predictor del bus NO lo predijo.
// La sorpresa dispara aprendizaje prioritario.

#[derive(Clone, Debug)]
pub struct SurpriseDetector {
    pub surprise_threshold: f32,
    pub last_surprise_tick: u64,
    pub n_surprises: u64,
    pub cumulative_surprise: f32,
    pub surprise_ema: f32,
    pub learning_boost_active: bool,
    pub learning_boost_ticks: u64,
}

impl SurpriseDetector {
    pub fn new() -> Self {
        SurpriseDetector {
            surprise_threshold: 0.3,
            last_surprise_tick: 0,
            n_surprises: 0,
            cumulative_surprise: 0.0,
            surprise_ema: 0.1,
            learning_boost_active: false,
            learning_boost_ticks: 0,
        }
    }

    /// Feed prediction error from bus_predictor. Returns true if surprise detected.
    pub fn observe(&mut self, tick: u64, pred_error: f32) -> bool {
        let surprise = pred_error - self.surprise_ema;
        self.surprise_ema = self.surprise_ema * 0.95 + pred_error * 0.05;
        if surprise > self.surprise_threshold {
            self.n_surprises += 1;
            self.last_surprise_tick = tick;
            self.cumulative_surprise += surprise;
            self.learning_boost_active = true;
            self.learning_boost_ticks = 10;
            true
        } else {
            if self.learning_boost_ticks > 0 {
                self.learning_boost_ticks -= 1;
            } else {
                self.learning_boost_active = false;
            }
            false
        }
    }

    /// Returns learning rate multiplier during surprise boost.
    pub fn lr_multiplier(&self) -> f32 {
        if self.learning_boost_active {
            2.0
        } else {
            1.0
        }
    }

    pub fn status(&self) -> String {
        format!(
            "Surprise | thr={:.2} | count={} | last={} | ema={:.3} | boost={} | boost_ticks={}",
            self.surprise_threshold,
            self.n_surprises,
            self.last_surprise_tick,
            self.surprise_ema,
            self.learning_boost_active,
            self.learning_boost_ticks
        )
    }
}
