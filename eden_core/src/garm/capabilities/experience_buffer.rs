// EDEN GARM — Experience Buffer + Meta-RL
// Guarda (estado, programa, recompensa, siguiente_estado) para entrenar
// el transformer con recompensas reales de la ejecucion.
//
// Principio: el transformer genera programas. Si un programa tiene exito,
// todos sus tokens se refuerzan. Si falla, se penalizan.

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Experience {
    pub state_bus: Vec<f32>,      // snapshot del bus antes de ejecutar
    pub program_text: String,     // texto del programa
    pub reward: f32,              // recompensa observada
    pub next_state_bus: Vec<f32>, // snapshot del bus despues
    pub tick: u64,
    pub n_tokens: usize, // numero de tokens generados
}

#[derive(Clone, Debug)]
pub struct ExperienceBuffer {
    pub buffer: VecDeque<Experience>,
    pub max_size: usize,
    pub total_reward: f32,
    pub total_experiences: u64,
    pub avg_reward: f32,
}

impl ExperienceBuffer {
    pub fn new(max_size: usize) -> Self {
        ExperienceBuffer {
            buffer: VecDeque::with_capacity(max_size),
            max_size,
            total_reward: 0.0,
            total_experiences: 0,
            avg_reward: 0.0,
        }
    }

    pub fn add(&mut self, exp: Experience) {
        self.total_reward += exp.reward;
        self.total_experiences += 1;
        self.avg_reward = self.total_reward / self.total_experiences as f32;
        if self.buffer.len() >= self.max_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(exp);
    }

    pub fn sample_batch(&self, n: usize) -> Vec<&Experience> {
        let mut batch: Vec<&Experience> = self.buffer.iter().collect();
        // Sort by reward descending (prioritized experience replay primitive)
        batch.sort_by(|a, b| {
            b.reward
                .partial_cmp(&a.reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        batch.into_iter().take(n).collect()
    }

    pub fn latest(&self) -> Option<&Experience> {
        self.buffer.back()
    }

    pub fn best(&self) -> Option<&Experience> {
        self.buffer.iter().max_by(|a, b| {
            a.reward
                .partial_cmp(&b.reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn status(&self) -> String {
        format!(
            "ExpBuffer | size={}/{} | total_reward={:.2} | avg={:.3} | best={:.2}",
            self.buffer.len(),
            self.max_size,
            self.total_reward,
            self.avg_reward,
            self.best().map(|e| e.reward).unwrap_or(0.0)
        )
    }
}
