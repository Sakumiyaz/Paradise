// EDEN GARM Continuous — Modo continuo autónomo.
// 100% Rust puro, 0 LLM, 0 red.
//
// Loop que integra los modulos cognitivos en operacion continua sin
// intervencion del usuario. Cada iteracion ejecuta:
//   1. Corpus tick (procesa N oraciones nuevas si las hay)
//   2. Autonomy introspect + analogy resolve gaps
//   3. Autonomy generate goals desde gaps no resueltos
//   4. Goal executor consume el stack durante varios ticks
//   5. Self-improvement audit cada cierto numero de iteraciones
// Reporta evolucion de stats clave a lo largo del tiempo (concepts,
// relations, completions) para visibilizar comportamiento emergente.

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct IterationStats {
    pub iter: u64,
    pub tick: u64,
    pub n_concepts: usize,
    pub n_relations: usize,
    pub n_executions: u64,
    pub n_completions: u64,
    pub n_grounding_facts: usize,
    pub n_analogies: u64,
    pub remaining_corpus: usize,
    pub goals_pending: usize,
}

#[derive(Clone, Debug)]
pub struct ContinuousMode {
    pub history: VecDeque<IterationStats>,
    pub max_history: usize,
    pub n_runs: u64,
    pub n_iterations_total: u64,
    pub paused: bool,
    pub auto_improve_every: u64,
    pub last_audit_iter: u64,
}

impl ContinuousMode {
    pub fn new() -> Self {
        ContinuousMode {
            history: VecDeque::with_capacity(256),
            max_history: 256,
            n_runs: 0,
            n_iterations_total: 0,
            paused: false,
            auto_improve_every: 5,
            last_audit_iter: 0,
        }
    }

    pub fn record(&mut self, s: IterationStats) {
        self.history.push_back(s);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
        self.n_iterations_total += 1;
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }
    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn status(&self) -> String {
        format!(
            "Continuous | runs={} | iterations_total={} | history_kept={} | paused={}",
            self.n_runs,
            self.n_iterations_total,
            self.history.len(),
            self.paused,
        )
    }

    /// Compare first vs last sample in history to show net evolution.
    pub fn report_evolution(&self, max_recent: usize) -> String {
        if self.history.is_empty() {
            return "Sin historial continuo".to_string();
        }
        let first = self.history.front().unwrap().clone();
        let last = self.history.back().unwrap().clone();
        let mut out = String::from("Evolucion del modo continuo:\n");
        out.push_str(&format!(
            "  Iteraciones: {} -> {} (delta={})\n",
            first.iter,
            last.iter,
            last.iter - first.iter,
        ));
        out.push_str(&format!(
            "  Concepts: {} -> {} (delta={})\n",
            first.n_concepts,
            last.n_concepts,
            last.n_concepts as i64 - first.n_concepts as i64,
        ));
        out.push_str(&format!(
            "  Relations: {} -> {} (delta={})\n",
            first.n_relations,
            last.n_relations,
            last.n_relations as i64 - first.n_relations as i64,
        ));
        out.push_str(&format!(
            "  Executions: {} -> {} (delta={})\n",
            first.n_executions,
            last.n_executions,
            last.n_executions as i64 - first.n_executions as i64,
        ));
        out.push_str(&format!(
            "  Completions: {} -> {} (delta={})\n",
            first.n_completions,
            last.n_completions,
            last.n_completions as i64 - first.n_completions as i64,
        ));
        out.push_str(&format!(
            "  Grounding facts: {} -> {} (delta={})\n",
            first.n_grounding_facts,
            last.n_grounding_facts,
            last.n_grounding_facts as i64 - first.n_grounding_facts as i64,
        ));
        out.push_str(&format!(
            "  Analogies: {} -> {} (delta={})\n",
            first.n_analogies,
            last.n_analogies,
            last.n_analogies as i64 - first.n_analogies as i64,
        ));
        out.push_str(&format!(
            "  Corpus remaining: {} -> {} (delta={})\n",
            first.remaining_corpus,
            last.remaining_corpus,
            last.remaining_corpus as i64 - first.remaining_corpus as i64,
        ));
        out.push_str(&format!(
            "\nUltimas {} muestras:\n",
            max_recent.min(self.history.len())
        ));
        for s in self.history.iter().rev().take(max_recent) {
            out.push_str(&format!(
                "  iter={} tick={} | concepts={} rel={} exec={} comp={} ground={} analog={} pending_goals={} corpus_left={}\n",
                s.iter, s.tick, s.n_concepts, s.n_relations,
                s.n_executions, s.n_completions, s.n_grounding_facts,
                s.n_analogies, s.goals_pending, s.remaining_corpus,
            ));
        }
        out
    }
}
