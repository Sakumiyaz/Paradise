// EDEN GARM Mood — Computational affective state
// Valence (pleasant/unpleasant) and Arousal (activated/calm) computed via exponential moving average (EMA)
// of motivational discomfort and prediction error.

pub struct MoodState {
    pub valence: f32, // -1.0 (negative) to 1.0 (positive)
    pub arousal: f32, // 0.0 (calm) to 1.0 (activated)
    pub alpha: f32,   // EMA smoothing factor
    pub ema_discomfort: f32,
    pub ema_error: f32,
}

impl MoodState {
    pub fn new() -> Self {
        MoodState {
            valence: 0.0,
            arousal: 0.0,
            alpha: 0.1,
            ema_discomfort: 0.5,
            ema_error: 0.5,
        }
    }

    /// Update mood with current discomfort (0..1) and prediction error (0..inf, clamped to 0..1).
    pub fn update(&mut self, discomfort: f32, prediction_error: f32) {
        let d = discomfort.clamp(0.0, 1.0);
        let e = prediction_error.clamp(0.0, 5.0) / 5.0; // normalize
        self.ema_discomfort = self.alpha * d + (1.0 - self.alpha) * self.ema_discomfort;
        self.ema_error = self.alpha * e + (1.0 - self.alpha) * self.ema_error;
        // Valence: inverse of discomfort + error. High error = negative.
        self.valence = (1.0 - self.ema_discomfort * 0.7 - self.ema_error * 0.3).clamp(-1.0, 1.0);
        // Arousal: error-driven activation.
        self.arousal = self.ema_error.clamp(0.0, 1.0);
    }

    pub fn dominant_quadrant(&self) -> &'static str {
        match (self.valence >= 0.0, self.arousal >= 0.5) {
            (true, true) => "excited",
            (false, true) => "anxious",
            (true, false) => "calm",
            (false, false) => "depressed",
        }
    }

    /// When valence is negative, increase tension threshold (less concept birth).
    pub fn tension_multiplier(&self) -> f32 {
        if self.valence < -0.3 {
            1.5
        } else {
            1.0
        }
    }

    /// When arousal is high, reduce idle threshold (act sooner).
    pub fn idle_threshold_multiplier(&self) -> f32 {
        if self.arousal > 0.7 {
            0.5
        } else {
            1.0
        }
    }

    /// When calm and comfortable, reduce learning rate (consolidate).
    pub fn lr_multiplier(&self) -> f32 {
        if self.dominant_quadrant() == "calm" && self.ema_discomfort < 0.3 {
            0.5
        } else {
            1.0
        }
    }

    pub fn status(&self) -> String {
        format!("Mood | valence={:.2} | arousal={:.2} | quadrant={} | t_mult={:.1} | idle_mult={:.1} | lr_mult={:.1}",
            self.valence, self.arousal, self.dominant_quadrant(),
            self.tension_multiplier(), self.idle_threshold_multiplier(), self.lr_multiplier())
    }
}
