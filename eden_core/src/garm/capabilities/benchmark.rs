// EDEN GARM — Benchmark Suite
// Evalúa capacidades reales del sistema cada N ticks.
// 5 pruebas: memoria, razonamiento, tool_use, generalización, autonomía.
// Devuelve scores 0..1 y un summary string.

#[derive(Clone, Debug)]
pub struct BenchmarkSuite {
    pub memory_score: f32,
    pub reasoning_score: f32,
    pub tool_score: f32,
    pub generalization_score: f32,
    pub autonomy_score: f32,
    pub n_runs: u64,
    pub last_summary: String,
    pub train_loss_ema: f32,
}

impl BenchmarkSuite {
    pub fn new() -> Self {
        BenchmarkSuite {
            memory_score: 0.0,
            reasoning_score: 0.0,
            tool_score: 0.0,
            generalization_score: 0.0,
            autonomy_score: 0.0,
            n_runs: 0,
            last_summary: String::new(),
            train_loss_ema: 1.0,
        }
    }

    /// Run all benchmarks against the engine state.
    /// This is called externally by the engine every N ticks.
    pub fn run(
        &mut self,
        _tick_count: u64,
        concepts_count: usize,
        concepts_at_tick_0: usize,
        tool_successes: u64,
        tool_failures: u64,
        goals_generated: usize,
        goals_completed: usize,
        vocab_size: usize,
        vocab_at_tick_0: usize,
    ) -> Vec<String> {
        let mut results = Vec::new();

        // 1. MEMORY: concept retention + vocabulary growth
        let concept_retention = if concepts_at_tick_0 > 0 {
            (concepts_count as f32 / concepts_at_tick_0 as f32).min(2.0) / 2.0
        } else {
            (concepts_count as f32 / 10.0).min(1.0)
        };
        let vocab_growth = if vocab_at_tick_0 > 0 {
            ((vocab_size as f32 - vocab_at_tick_0 as f32) / vocab_at_tick_0 as f32).min(1.0)
        } else {
            (vocab_size as f32 / 100.0).min(1.0)
        };
        self.memory_score = (concept_retention * 0.5 + vocab_growth * 0.5).clamp(0.0, 1.0);
        results.push(format!(
            "[BENCH_MEM] concepts={}/{} vocab={}/{} | score={:.2}",
            concepts_count, concepts_at_tick_0, vocab_size, vocab_at_tick_0, self.memory_score
        ));

        // 2. REASONING: analogies via hub cross-domain similarity
        // (placeholder: if any cross-domain projection exists, score > 0)
        self.reasoning_score = 0.0; // populated externally if hub has projections
        results.push(format!("[BENCH_REASON] score={:.2}", self.reasoning_score));

        // 3. TOOL USE: success rate
        let total_tools = tool_successes + tool_failures;
        self.tool_score = if total_tools > 0 {
            (tool_successes as f32 / total_tools as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        results.push(format!(
            "[BENCH_TOOL] success={}/{} | score={:.2}",
            tool_successes, total_tools, self.tool_score
        ));

        // 4. GENERALIZATION: learning progress measured by real train loss EMA + concept formation
        let loss_progress = if self.train_loss_ema > 0.0 {
            (1.0 - self.train_loss_ema).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let concept_rate = (concepts_count as f32 / 50.0).min(1.0);
        self.generalization_score = (loss_progress * 0.6 + concept_rate * 0.4).clamp(0.0, 1.0);
        results.push(format!(
            "[BENCH_GEN] loss_ema={:.3} loss_prog={:.2} concept_rate={:.2} | score={:.2}",
            self.train_loss_ema, loss_progress, concept_rate, self.generalization_score
        ));

        // 5. AUTONOMY: goals generated and completed without external input
        let gen_rate = (goals_generated as f32 / 10.0).min(1.0);
        let comp_rate = if goals_generated > 0 {
            (goals_completed as f32 / goals_generated as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.autonomy_score = (gen_rate * 0.5 + comp_rate * 0.5).clamp(0.0, 1.0);
        results.push(format!(
            "[BENCH_AUTO] goals_gen={} comp={} | score={:.2}",
            goals_generated, goals_completed, self.autonomy_score
        ));

        self.n_runs += 1;
        let avg = (self.memory_score
            + self.reasoning_score
            + self.tool_score
            + self.generalization_score
            + self.autonomy_score)
            / 5.0;
        self.last_summary = format!(
            "Benchmark | mem={:.2} reason={:.2} tool={:.2} gen={:.2} auto={:.2} | avg={:.2}",
            self.memory_score,
            self.reasoning_score,
            self.tool_score,
            self.generalization_score,
            self.autonomy_score,
            avg
        );
        results.push(self.last_summary.clone());
        results
    }

    /// Feed external reasoning evidence (e.g., hub found cross-domain analogy).
    pub fn report_reasoning(&mut self, found_analogy: bool) {
        if found_analogy {
            self.reasoning_score = (self.reasoning_score * 0.9 + 0.1).min(1.0);
        }
    }

    pub fn status(&self) -> String {
        format!("Benchmark | runs={} | {}", self.n_runs, self.last_summary)
    }

    /// Report a training loss sample from any module.
    pub fn report_train_loss(&mut self, loss: f32) {
        if loss > 0.0 {
            let alpha = 0.05;
            self.train_loss_ema = self.train_loss_ema * (1.0 - alpha) + loss * alpha;
        }
    }
}
