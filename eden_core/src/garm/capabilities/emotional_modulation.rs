// EDEN GARM — Emotional Modulation
// Las emociones (valence, arousal, dominance) modulan ganancias globales:
// - LR multiplier, attention temperature, exploration rate, plasticity.
// No son etiquetas decorativas: son moduladores de ganancia reales.

#[derive(Clone, Debug)]
pub struct EmotionalModulation {
    pub valence: f32,   // -1..1
    pub arousal: f32,   // 0..1
    pub dominance: f32, // 0..1
    pub lr_multiplier: f32,
    pub attention_temp: f32,
    pub exploration_boost: f32,
    pub plasticity: f32,
    pub n_ticks: u64,
}

impl EmotionalModulation {
    pub fn new() -> Self {
        EmotionalModulation {
            valence: 0.0,
            arousal: 0.5,
            dominance: 0.5,
            lr_multiplier: 1.0,
            attention_temp: 1.0,
            exploration_boost: 0.0,
            plasticity: 0.5,
            n_ticks: 0,
        }
    }

    /// Update emotional state and recompute modulators.
    pub fn update(&mut self, reward: f32, novelty: f32, control: f32) {
        // Reward -> valence
        self.valence = self.valence * 0.9 + reward * 0.1;
        self.valence = self.valence.clamp(-1.0, 1.0);
        // Novelty -> arousal
        self.arousal = self.arousal * 0.9 + novelty * 0.1;
        self.arousal = self.arousal.clamp(0.0, 1.0);
        // Control -> dominance
        self.dominance = self.dominance * 0.9 + control * 0.1;
        self.dominance = self.dominance.clamp(0.0, 1.0);

        // Recompute modulators
        // High arousal + positive valence -> high plasticity (learning window open)
        self.plasticity = ((self.arousal + 0.5) * (self.valence + 1.0) / 2.0).clamp(0.1, 2.0);
        // High dominance -> lower temperature (more focused attention)
        self.attention_temp = (1.5 - self.dominance).clamp(0.3, 2.0);
        // Low valence + high arousal -> boost exploration (seek change)
        self.exploration_boost = if self.valence < 0.0 && self.arousal > 0.6 {
            0.3
        } else {
            0.0
        };
        // Arousal directly modulates LR
        self.lr_multiplier = (0.5 + self.arousal).clamp(0.3, 2.0);

        self.n_ticks += 1;
    }

    /// Apply emotional modulation to a learning rate.
    pub fn modulate_lr(&self, base_lr: f32) -> f32 {
        base_lr * self.lr_multiplier
    }

    /// Apply emotional modulation to attention logits (temperature scaling).
    pub fn modulate_attention(&self, logits: &mut [f32]) {
        for x in logits.iter_mut() {
            *x /= self.attention_temp.max(0.1);
        }
    }

    pub fn status(&self) -> String {
        format!("Emotion | V={:.2} A={:.2} D={:.2} | lr={:.2}x | temp={:.2} | explore+={:.2} | plast={:.2}",
            self.valence, self.arousal, self.dominance,
            self.lr_multiplier, self.attention_temp, self.exploration_boost, self.plasticity)
    }
}
