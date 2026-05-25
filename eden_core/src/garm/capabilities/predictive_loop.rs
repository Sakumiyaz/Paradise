// EDEN GARM PredictiveLoop — Convierte el self_model de predictor sintetico
// a modelo entrenado por experiencia. 100% Rust puro, 0 LLM, 0 red.
//
// Antes: self_model.train recibia placeholders [0.5, 0.5, ...]
// Ahora: cada goal ejecutado genera un (state_pre, action, outcome_real)
//        donde outcome_real = [completed, duration_ticks, concept_growth]
//
// El loop:
//   1. encode_action_target convierte ActionTarget en one-hot vector
//   2. predict_outcome ANTES del dispatch
//   3. record_prediction guarda prediccion + outcome real
//   4. compute_calibration produce Brier score y mean prediction error
//   5. expose calibration para que metacognition la use

use crate::eden_garm::capabilities::goal_executor::ActionTarget;
use std::collections::VecDeque;

/// One-hot encoding de las 8 ActionTargets en un Vec<f32> de 8 elementos.
/// Esto es la "action" que el self_model recibe como input.
pub fn encode_action_target(target: &ActionTarget) -> Vec<f32> {
    let mut v = vec![0.0f32; 8];
    let idx = match target {
        ActionTarget::Exploration => 0,
        ActionTarget::Metacognition => 1,
        ActionTarget::Physics => 2,
        ActionTarget::Causality => 3,
        ActionTarget::Memory => 4,
        ActionTarget::Semantics => 5,
        ActionTarget::Perception => 6,
        ActionTarget::Goal => 7,
        ActionTarget::NoMatch => return v,
    };
    v[idx] = 1.0;
    v
}

#[derive(Clone, Debug)]
pub struct PredictionRecord {
    pub tick: u64,
    pub goal_id: u64,
    pub goal_label: String,
    pub action: ActionTarget,
    /// Predicted [success_prob, duration, concept_growth]
    pub predicted: Vec<f32>,
    /// Actual [success(0/1), duration, concept_delta]
    pub actual: Vec<f32>,
    /// |predicted - actual| per dimension
    pub error_per_dim: Vec<f32>,
    /// Brier score for the success dimension (squared error of probability)
    pub brier_success: f32,
}

#[derive(Clone, Debug)]
pub struct PredictiveLoop {
    pub history: VecDeque<PredictionRecord>,
    pub max_history: usize,
    pub n_predictions: u64,
    pub n_trainings: u64,
    /// Running mean of brier success score (lower = better calibration)
    pub mean_brier: f32,
    /// Running mean of total error
    pub mean_total_error: f32,
}

impl PredictiveLoop {
    pub fn new() -> Self {
        PredictiveLoop {
            history: VecDeque::with_capacity(256),
            max_history: 256,
            n_predictions: 0,
            n_trainings: 0,
            mean_brier: 0.0,
            mean_total_error: 0.0,
        }
    }

    pub fn record(&mut self, mut rec: PredictionRecord) {
        // Compute error_per_dim and brier_success
        let dims = rec.predicted.len().min(rec.actual.len());
        rec.error_per_dim = (0..dims)
            .map(|i| (rec.predicted[i] - rec.actual[i]).abs())
            .collect();
        rec.brier_success = if dims > 0 {
            (rec.predicted[0] - rec.actual[0]).powi(2)
        } else {
            0.0
        };

        self.n_predictions += 1;
        // Update running means with EMA
        let alpha = 0.1f32;
        self.mean_brier = self.mean_brier * (1.0 - alpha) + rec.brier_success * alpha;
        let total_err: f32 = rec.error_per_dim.iter().sum();
        self.mean_total_error = self.mean_total_error * (1.0 - alpha) + total_err * alpha;

        self.history.push_back(rec);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// Compute calibration: for predictions of success in bins, what's the actual rate?
    /// Returns Vec<(bin_low, bin_high, n_predictions, actual_success_rate)>.
    pub fn calibration_curve(&self, n_bins: usize) -> Vec<(f32, f32, usize, f32)> {
        let mut bins: Vec<(f32, f32, Vec<f32>)> = (0..n_bins)
            .map(|i| {
                let lo = i as f32 / n_bins as f32;
                let hi = (i + 1) as f32 / n_bins as f32;
                (lo, hi, Vec::new())
            })
            .collect();
        for rec in &self.history {
            if rec.predicted.is_empty() || rec.actual.is_empty() {
                continue;
            }
            let p = rec.predicted[0];
            let bin_idx = ((p * n_bins as f32) as usize).min(n_bins - 1);
            bins[bin_idx].2.push(rec.actual[0]);
        }
        bins.into_iter()
            .map(|(lo, hi, vals)| {
                let n = vals.len();
                let rate = if n == 0 {
                    0.0
                } else {
                    vals.iter().sum::<f32>() / n as f32
                };
                (lo, hi, n, rate)
            })
            .collect()
    }

    pub fn status(&self) -> String {
        format!(
            "PredictiveLoop | predictions={} | trainings={} | history={} | mean_brier={:.3} | mean_total_err={:.3}",
            self.n_predictions, self.n_trainings, self.history.len(),
            self.mean_brier, self.mean_total_error,
        )
    }

    pub fn report_calibration(&self, n_bins: usize) -> String {
        let curve = self.calibration_curve(n_bins);
        if self.history.is_empty() {
            return "Sin predicciones registradas (ejecutar goals primero)".to_string();
        }
        let mut out = format!(
            "Calibracion del self_model ({} predicciones registradas):\n",
            self.history.len(),
        );
        out.push_str(&format!(
            "  Mean Brier score (success): {:.3} (lower = better, perfect=0)\n",
            self.mean_brier,
        ));
        out.push_str(&format!(
            "  Mean total prediction error: {:.3}\n",
            self.mean_total_error,
        ));
        out.push_str("\n  Curva de calibracion:\n");
        out.push_str("  bin_predicted    n_samples  actual_rate  delta\n");
        for (lo, hi, n, rate) in curve {
            if n == 0 {
                continue;
            }
            let mid = (lo + hi) / 2.0;
            let delta = rate - mid;
            let delta_marker = if delta.abs() < 0.05 {
                "OK"
            } else if delta > 0.0 {
                "underconf"
            } else {
                "overconf"
            };
            out.push_str(&format!(
                "  [{:.2}-{:.2}]   {:>5}      {:.3}        {:+.3}  {}\n",
                lo, hi, n, rate, delta, delta_marker,
            ));
        }
        out
    }

    pub fn report_recent(&self, n: usize) -> String {
        if self.history.is_empty() {
            return "Sin predicciones".to_string();
        }
        let mut out = format!("Ultimas {} predicciones:\n", n.min(self.history.len()));
        for r in self.history.iter().rev().take(n) {
            let total_err: f32 = r.error_per_dim.iter().sum();
            out.push_str(&format!(
                "  [t={}] gid={} '{}' -> {:?}\n    predicted={:?}\n    actual={:?}\n    brier={:.3} total_err={:.3}\n",
                r.tick, r.goal_id, r.goal_label, r.action,
                r.predicted, r.actual, r.brier_success, total_err,
            ));
        }
        out
    }
}
