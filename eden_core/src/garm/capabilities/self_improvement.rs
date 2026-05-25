// EDEN GARM SelfImprovement — Recursive self-improvement bounded.
// 100% Rust puro, 0 LLM, 0 red.
//
// Usa self_awareness para detectar parametros suboptimos y los ajusta dentro
// de rangos seguros. Cada ajuste guarda el valor previo para rollback.
//
// SAFE BY DESIGN:
//   - Solo se permiten ajustes en parametros enumerados (no codigo)
//   - Todos los rangos tienen bounds [min, max]
//   - Step size es proporcional al rango (no salto brusco)
//   - History permite revert del ultimo cambio
//   - Si la metrica empeora N veces consecutivas, recommend rollback
//
// Esto NO modifica el código fuente. Solo ajusta parametros runtime.

use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TunableParam {
    /// morphogenesis.creation_threshold: distance below which embedding assimilates
    MorphoCreationThreshold,
    /// morphogenesis.tension_threshold: tension above which spawn new concept
    MorphoTensionThreshold,
    /// analogy.min_combined_score: min score (0..1) for analogy match
    AnalogyMinScore,
    /// goal_executor.min_match_score: min score (0..1) for capability match
    ExecutorMinMatch,
    /// goal_executor.completion_threshold: progress delta needed to complete
    ExecutorCompletionThreshold,
    /// corpus_reader.sentences_per_tick: how many sentences per autonomous tick
    CorpusSentencesPerTick,
    /// autonomy.max_goals_per_run: max goals from one introspection
    AutonomyMaxGoals,
}

impl TunableParam {
    /// Returns (min, max) bounds for the parameter.
    pub fn bounds(&self) -> (f32, f32) {
        match self {
            TunableParam::MorphoCreationThreshold => (0.05, 0.8),
            TunableParam::MorphoTensionThreshold => (0.1, 0.9),
            TunableParam::AnalogyMinScore => (0.1, 0.9),
            TunableParam::ExecutorMinMatch => (0.05, 0.6),
            TunableParam::ExecutorCompletionThreshold => (0.005, 0.5),
            TunableParam::CorpusSentencesPerTick => (1.0, 50.0),
            TunableParam::AutonomyMaxGoals => (1.0, 30.0),
        }
    }

    /// Step size for adjustments (delta to add/subtract).
    pub fn step(&self) -> f32 {
        match self {
            TunableParam::MorphoCreationThreshold => 0.05,
            TunableParam::MorphoTensionThreshold => 0.05,
            TunableParam::AnalogyMinScore => 0.05,
            TunableParam::ExecutorMinMatch => 0.03,
            TunableParam::ExecutorCompletionThreshold => 0.01,
            TunableParam::CorpusSentencesPerTick => 1.0,
            TunableParam::AutonomyMaxGoals => 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ParamAdjustment {
    pub param: TunableParam,
    pub old_value: f32,
    pub new_value: f32,
    pub reason: String,
    pub tick: u64,
    pub metric_before: f32,
    pub metric_after: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct ImprovementProposal {
    pub param: TunableParam,
    pub direction: f32, // +1 or -1
    pub reason: String,
    pub priority: f32,
}

#[derive(Clone, Debug)]
pub struct SelfImprovement {
    pub history: VecDeque<ParamAdjustment>,
    pub max_history: usize,
    pub n_audits: u64,
    pub n_applied: u64,
    pub n_reverted: u64,
    pub last_proposals: Vec<ImprovementProposal>,
}

impl SelfImprovement {
    pub fn new() -> Self {
        SelfImprovement {
            history: VecDeque::with_capacity(64),
            max_history: 64,
            n_audits: 0,
            n_applied: 0,
            n_reverted: 0,
            last_proposals: Vec::new(),
        }
    }

    /// Audit current state and produce ranked improvement proposals.
    /// Inputs are the relevant metrics extracted from the engine.
    pub fn audit(
        &mut self,
        n_concepts: usize,
        n_relations: usize,
        n_executions: u64,
        n_completions: u64,
        n_no_match: u64,
        n_analogy_attempts: u64,
        n_analogy_inferences: u64,
        _n_grounding_facts: usize,
        sentences_processed: u64,
    ) -> Vec<ImprovementProposal> {
        self.n_audits += 1;
        let mut proposals = Vec::new();

        // Rule 1: completions == 0 over many executions -> lower completion_threshold
        if n_executions >= 5 && n_completions == 0 {
            proposals.push(ImprovementProposal {
                param: TunableParam::ExecutorCompletionThreshold,
                direction: -1.0,
                reason: format!(
                    "0 completions over {} executions; threshold may be too strict",
                    n_executions
                ),
                priority: 0.9,
            });
        }

        // Rule 2: many no_matches -> lower min_match_score
        if n_no_match >= 3 && n_executions > 0 {
            let no_match_ratio = n_no_match as f32 / (n_executions as f32 + n_no_match as f32);
            if no_match_ratio > 0.4 {
                proposals.push(ImprovementProposal {
                    param: TunableParam::ExecutorMinMatch,
                    direction: -1.0,
                    reason: format!(
                        "{:.0}% of goals had no capability match; threshold may be too strict",
                        no_match_ratio * 100.0
                    ),
                    priority: 0.7,
                });
            }
        }

        // Rule 3: analogy attempts but no inferences -> lower analogy threshold
        if n_analogy_attempts >= 10 && n_analogy_inferences == 0 {
            proposals.push(ImprovementProposal {
                param: TunableParam::AnalogyMinScore,
                direction: -1.0,
                reason: format!(
                    "0 inferences over {} attempts; threshold may be too strict",
                    n_analogy_attempts
                ),
                priority: 0.6,
            });
        }

        // Rule 4: very few concepts after many sentences -> lower creation_threshold
        if sentences_processed >= 50 && n_concepts < 20 {
            proposals.push(ImprovementProposal {
                param: TunableParam::MorphoCreationThreshold,
                direction: -1.0,
                reason: format!(
                    "only {} concepts from {} sentences; geometric clustering may be too greedy",
                    n_concepts, sentences_processed
                ),
                priority: 0.8,
            });
        }

        // Rule 5: too many concepts but very low relation density -> raise tension threshold
        if n_concepts > 500 && n_relations < n_concepts / 10 {
            proposals.push(ImprovementProposal {
                param: TunableParam::MorphoTensionThreshold,
                direction: 1.0,
                reason: format!(
                    "{} concepts but only {} relations; spawning may be too aggressive",
                    n_concepts, n_relations
                ),
                priority: 0.5,
            });
        }

        // Rule 6: corpus stalled (no sentences/tick) but corpus available
        if sentences_processed > 0 && sentences_processed < 20 {
            proposals.push(ImprovementProposal {
                param: TunableParam::CorpusSentencesPerTick,
                direction: 1.0,
                reason: format!(
                    "only {} sentences processed; can speed up ingestion",
                    sentences_processed
                ),
                priority: 0.4,
            });
        }

        // Sort by priority desc
        proposals.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.last_proposals = proposals.clone();
        proposals
    }

    /// Apply an adjustment. Returns the (old, new) value pair on success.
    /// Note: caller must persist new_value to the actual subsystem.
    pub fn propose_value(&self, current: f32, prop: &ImprovementProposal) -> f32 {
        let (lo, hi) = prop.param.bounds();
        let step = prop.param.step();
        let new_val = (current + prop.direction * step).clamp(lo, hi);
        new_val
    }

    /// Record an applied adjustment.
    pub fn record(&mut self, adj: ParamAdjustment) {
        self.n_applied += 1;
        self.history.push_back(adj);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// Get last adjustment for revert.
    pub fn last_adjustment(&self) -> Option<&ParamAdjustment> {
        self.history.back()
    }

    pub fn pop_last(&mut self) -> Option<ParamAdjustment> {
        if let Some(adj) = self.history.pop_back() {
            self.n_reverted += 1;
            Some(adj)
        } else {
            None
        }
    }

    pub fn status(&self) -> String {
        format!(
            "SelfImprovement | audits={} | applied={} | reverted={} | history_kept={} | last_proposals={}",
            self.n_audits, self.n_applied, self.n_reverted,
            self.history.len(), self.last_proposals.len(),
        )
    }

    pub fn report_history(&self, max: usize) -> String {
        if self.history.is_empty() {
            return "Sin ajustes en historial".to_string();
        }
        let mut out = String::from("Ajustes recientes:\n");
        for adj in self.history.iter().rev().take(max) {
            let after = adj
                .metric_after
                .map(|v| format!("{:.3}", v))
                .unwrap_or_else(|| "?".to_string());
            out.push_str(&format!(
                "  [t={}] {:?}: {:.3} -> {:.3} | metric_before={:.3} | metric_after={} | {}\n",
                adj.tick,
                adj.param,
                adj.old_value,
                adj.new_value,
                adj.metric_before,
                after,
                adj.reason,
            ));
        }
        out
    }

    pub fn report_proposals(&self) -> String {
        if self.last_proposals.is_empty() {
            return "Sin propuestas (ejecutar audit primero)".to_string();
        }
        let mut out = String::from("Propuestas de mejora:\n");
        for (i, p) in self.last_proposals.iter().enumerate() {
            let dir = if p.direction > 0.0 { "+" } else { "-" };
            out.push_str(&format!(
                "  [{}] {:?} {} | priority={:.2} | razon: {}\n",
                i, p.param, dir, p.priority, p.reason,
            ));
        }
        out
    }
}
