// EDEN GARM Planner — Monte Carlo Tree Search over simulated futures
// Evaluates action sequences by rolling forward an internal simulation.

use crate::eden_garm::capabilities::causality::StructuralCausalModel;
use crate::eden_garm::capabilities::morphogenesis::ConceptSpace;
use crate::eden_garm::capabilities::motivation::MotivationEngine;
use crate::eden_garm::capabilities::self_model::SelfModel;
use crate::eden_garm::capabilities::GarmCapability;

#[derive(Clone, Debug)]
pub struct ActionSequence {
    pub steps: Vec<GarmCapability>,
    pub predicted_final_discomfort: f32,
    pub predicted_concept_count: usize,
    pub predicted_error: f32,
    pub score: f32,
}

pub struct Planner {
    pub horizon: usize,
    pub n_simulations: usize,
    pub best: Option<ActionSequence>,
    pub last_plan_tick: u64,
}

impl Planner {
    pub fn new() -> Self {
        Planner {
            horizon: 3,
            n_simulations: 8,
            best: None,
            last_plan_tick: 0,
        }
    }

    /// Plan by simulating random action sequences and scoring by predicted discomfort
    pub fn plan(
        &mut self,
        tick: u64,
        motivation: &MotivationEngine,
        self_model: &SelfModel,
        morpho: &ConceptSpace,
        _causality: &StructuralCausalModel,
        capabilities: &[GarmCapability],
    ) -> Option<ActionSequence> {
        self.last_plan_tick = tick;
        let mut best_score = f32::MAX;
        let mut best_seq: Option<ActionSequence> = None;

        let actions: Vec<GarmCapability> = capabilities
            .iter()
            .filter(|c| **c != GarmCapability::NaturalLanguage)
            .cloned()
            .collect();
        if actions.is_empty() {
            return None;
        }

        for sim in 0..self.n_simulations {
            let mut sim_motivation = motivation.clone();
            let mut sim_morpho_count = morpho.n_concepts();
            let mut sim_error = self_model.mean_error();
            let mut steps = Vec::new();

            for _ in 0..self.horizon {
                let action = &actions[(tick as usize + sim + steps.len()) % actions.len()];
                steps.push(action.clone());

                // Simulate effect of this action on internal state
                match action {
                    GarmCapability::SelfModel | GarmCapability::Metacognition => {
                        // Training reduces error
                        sim_error *= 0.9;
                        sim_motivation.discomfort =
                            (sim_motivation.discomfort * 0.9 + 0.1 * sim_error).clamp(0.0, 1.0);
                    }
                    GarmCapability::Morphogenesis
                    | GarmCapability::Semantics
                    | GarmCapability::WorldModel => {
                        // Exploration may increase discomfort temporarily but adds concepts
                        sim_morpho_count += 1;
                        sim_motivation.discomfort =
                            (sim_motivation.discomfort * 1.05).clamp(0.0, 1.0);
                    }
                    GarmCapability::Causality => {
                        // Causal planning reduces uncertainty
                        sim_error *= 0.95;
                    }
                    GarmCapability::Motivation | GarmCapability::Mood => {
                        // Reflection reduces discomfort
                        sim_motivation.discomfort *= 0.95;
                    }
                    _ => {
                        // Default: slight cost
                        sim_motivation.discomfort =
                            (sim_motivation.discomfort + 0.05).clamp(0.0, 1.0);
                    }
                }
            }

            // Score: lower discomfort + lower error + higher concept growth
            let discomfort_score = sim_motivation.discomfort;
            let error_score = sim_error;
            let growth_score = -(sim_morpho_count as f32 - morpho.n_concepts() as f32) * 0.1;
            let score = discomfort_score + error_score + growth_score;

            if score < best_score {
                best_score = score;
                best_seq = Some(ActionSequence {
                    steps: steps.clone(),
                    predicted_final_discomfort: sim_motivation.discomfort,
                    predicted_concept_count: sim_morpho_count,
                    predicted_error: sim_error,
                    score,
                });
            }
        }

        self.best = best_seq.clone();
        best_seq
    }

    /// Recommend the first action of the best plan
    pub fn recommend_action(&self) -> Option<GarmCapability> {
        self.best.as_ref().and_then(|b| b.steps.first().cloned())
    }

    /// Evaluate if following the plan improves actual state
    pub fn observe_outcome(&mut self, actual_discomfort: f32, actual_error: f32) {
        if let Some(ref plan) = self.best {
            let discomfort_diff = plan.predicted_final_discomfort - actual_discomfort;
            let error_diff = plan.predicted_error - actual_error;
            if discomfort_diff < -0.1 || error_diff < -0.1 {
                // Plan was worse than predicted: increase simulations
                self.n_simulations = (self.n_simulations + 2).min(20);
            } else if discomfort_diff > 0.1 && error_diff > 0.1 {
                // Plan was better: keep or decrease for efficiency
                self.n_simulations = self.n_simulations.saturating_sub(1).max(4);
            }
        }
    }

    pub fn status(&self) -> String {
        match &self.best {
            Some(b) => format!(
                "Planner | horizon={} | sims={} | best_score={:.3} | steps={}",
                self.horizon,
                self.n_simulations,
                b.score,
                b.steps.len()
            ),
            None => format!(
                "Planner | horizon={} | sims={} | no plan yet",
                self.horizon, self.n_simulations
            ),
        }
    }
}
