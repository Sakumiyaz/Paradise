// EDEN GARM — Generative Controller Convergence Metrics
// Tracks whether the transformer is learning to emit parseable instructions.

use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct GenMetrics {
    pub n_generations: u64,
    pub n_parsed: u64,
    pub n_executed: u64,
    pub total_reward: f32,
    pub reward_ema: f32,
    pub unique_actions: HashSet<String>,
    pub last_gen_text: String,
    pub last_parse_success: bool,
    pub last_reward: f32,
    // Buffer of successful (prompt + generation) sequences for prioritized re-training
    pub success_buffer: Vec<String>,
    pub max_buffer: usize,
}

impl GenMetrics {
    pub fn new() -> Self {
        GenMetrics {
            n_generations: 0,
            n_parsed: 0,
            n_executed: 0,
            total_reward: 0.0,
            reward_ema: 0.0,
            unique_actions: HashSet::new(),
            last_gen_text: String::new(),
            last_parse_success: false,
            last_reward: 0.0,
            success_buffer: Vec::new(),
            max_buffer: 100,
        }
    }

    pub fn report_generation(
        &mut self,
        gen_text: &str,
        parsed: bool,
        n_instrs: usize,
        reward: f32,
        full_prompt: Option<&str>,
    ) {
        self.n_generations += 1;
        self.last_gen_text = gen_text.to_string();
        self.last_parse_success = parsed;
        self.last_reward = reward;
        if parsed {
            self.n_parsed += 1;
            self.n_executed += n_instrs as u64;
            // Track unique instruction types (first word of each line)
            for line in gen_text.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if let Some(first_word) = trimmed.split_whitespace().next() {
                        self.unique_actions.insert(first_word.to_uppercase());
                    }
                }
            }
            // Store successful sequence for prioritized re-training
            if let Some(prompt) = full_prompt {
                let seq = format!("{} {}", prompt, gen_text);
                if self.success_buffer.len() >= self.max_buffer {
                    self.success_buffer.remove(0);
                }
                self.success_buffer.push(seq);
            }
        }
        self.total_reward += reward;
        let alpha = 0.1;
        self.reward_ema = self.reward_ema * (1.0 - alpha) + reward * alpha;
    }

    /// Sample a random successful sequence from the buffer for training.
    pub fn sample_success(&self) -> Option<String> {
        if self.success_buffer.is_empty() {
            return None;
        }
        let idx = (self.n_generations as usize) % self.success_buffer.len();
        Some(self.success_buffer[idx].clone())
    }

    pub fn parse_rate(&self) -> f32 {
        if self.n_generations == 0 {
            0.0
        } else {
            self.n_parsed as f32 / self.n_generations as f32
        }
    }

    pub fn avg_reward(&self) -> f32 {
        if self.n_generations == 0 {
            0.0
        } else {
            self.total_reward / self.n_generations as f32
        }
    }

    pub fn diversity(&self) -> usize {
        self.unique_actions.len()
    }

    pub fn status(&self) -> String {
        format!("GenMetrics | gen={} parsed={} rate={:.2} exec={} diversity={} reward_ema={:.2} avg_r={:.2}",
            self.n_generations, self.n_parsed, self.parse_rate(), self.n_executed,
            self.diversity(), self.reward_ema, self.avg_reward())
    }
}
