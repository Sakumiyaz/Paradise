// EDEN GARM — Reward Oracle (recompensa externa consistente)
// La recompensa NO viene de feelings internos. Viene de un ORÁCULO externo
// que observa cambios verificables en el estado del mundo/engine.
//
// Señales ground-truth:
// - Build exitoso (cargo check sin errores)
// - Descubrimiento de concepto nuevo
// - Tool call exitoso (resultado correcto verificable)
// - Corpus consumido (progreso medible)
// - Predicción correcta del world model
// - Mejora de métricas de seguridad/observability
//
// Reward es escalar, acumulativo, y tiene varianza controlada.

#[derive(Clone, Debug)]
pub struct RewardOracle {
    pub last_reward: f32,
    pub reward_ema: f32,
    pub n_evaluations: u64,
    pub baseline: f32,
    pub history: Vec<f32>,
    pub last_explanation: String,
}

impl RewardOracle {
    pub fn new() -> Self {
        RewardOracle {
            last_reward: 0.0,
            reward_ema: 0.0,
            n_evaluations: 0,
            baseline: 0.0,
            history: Vec::new(),
            last_explanation: String::new(),
        }
    }

    /// Evaluate the engine state and return a scalar reward with explanation.
    pub fn evaluate(
        &mut self,
        prev_errors: usize,
        curr_errors: usize,
        prev_concepts: usize,
        curr_concepts: usize,
        prev_corpus: u64,
        curr_corpus: u64,
        tool_success: bool,
        wm_pred_err: f32,
    ) -> (f32, String) {
        let mut reward = 0.0f32;
        let mut reasons = Vec::new();

        // 1. Build quality: -5 per error, +10 for clean build
        let error_delta = curr_errors as f32 - prev_errors as f32;
        if error_delta < 0.0 {
            reward += error_delta.abs() * 5.0;
            reasons.push(format!("errors_down:{}", error_delta.abs()));
        } else if error_delta > 0.0 {
            reward -= error_delta * 5.0;
            reasons.push(format!("errors_up:{}", error_delta));
        } else if curr_errors == 0 {
            reward += 10.0;
            reasons.push("build_clean".to_string());
        }

        // 2. Concept discovery: +5 per new concept
        let concept_delta = curr_concepts as f32 - prev_concepts as f32;
        if concept_delta > 0.0 {
            reward += concept_delta * 5.0;
            reasons.push(format!("new_concepts:{}", concept_delta));
        }

        // 3. Corpus progress: +0.001 per sentence consumed
        let corpus_delta = curr_corpus as f32 - prev_corpus as f32;
        if corpus_delta > 0.0 {
            reward += corpus_delta * 0.001;
            reasons.push(format!("corpus:{}", corpus_delta));
        }

        // 4. Tool success: +3
        if tool_success {
            reward += 3.0;
            reasons.push("tool_success".to_string());
        }

        // 5. World model prediction accuracy: +1 if error decreased
        if wm_pred_err < 0.1 {
            reward += 1.0;
            reasons.push("wm_accurate".to_string());
        } else if wm_pred_err > 0.5 {
            reward -= 1.0;
            reasons.push("wm_bad".to_string());
        }

        // 6. Anti-stagnation: -0.1 if nothing changed
        if reward == 0.0 {
            reward -= 0.1;
            reasons.push("stagnation".to_string());
        }

        reward = reward.clamp(-10.0, 20.0);
        self.last_reward = reward;
        self.reward_ema = self.reward_ema * 0.95 + reward * 0.05;
        self.n_evaluations += 1;
        self.history.push(reward);
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
        self.baseline = self.reward_ema;
        self.last_explanation = reasons.join(" | ");
        (reward, self.last_explanation.clone())
    }

    /// Advantage = reward - baseline (for policy gradient).
    pub fn advantage(&self) -> f32 {
        self.last_reward - self.baseline
    }

    pub fn status(&self) -> String {
        format!(
            "Oracle | last={:.2} | ema={:.2} | adv={:.2} | n={} | last='{}'",
            self.last_reward,
            self.reward_ema,
            self.advantage(),
            self.n_evaluations,
            self.last_explanation
        )
    }
}
