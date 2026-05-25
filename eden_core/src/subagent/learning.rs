//! # LEARNING - Sistema de aprendizaje por reforzamiento desde cero
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct SubagentConfig {
    pub role: String,
    pub learning_rate: f32,
    pub discount_factor: f32,
    pub exploration_rate: f32,
}

#[derive(Debug, Clone)]
pub struct Experience {
    pub state: Vec<f32>,
    pub action: usize,
    pub reward: f32,
    pub next_state: Vec<f32>,
    pub done: bool,
}

impl Experience {
    pub fn new(
        state: Vec<f32>,
        action: usize,
        reward: f32,
        next_state: Vec<f32>,
        done: bool,
    ) -> Self {
        Self {
            state,
            action,
            reward,
            next_state,
            done,
        }
    }
}

/// Buffer circular de experiencias
#[derive(Debug, Clone)]
pub struct ExperienceBuffer {
    buffer: VecDeque<Experience>,
    capacity: usize,
}

impl ExperienceBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, experience: Experience) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(experience);
    }

    pub fn sample(&self, batch_size: usize) -> Vec<Experience> {
        let mut rng = rand::thread_rng();
        let mut batch = Vec::new();
        let len = self.buffer.len();

        for _ in 0..batch_size.min(len) {
            let idx = rand::Rng::gen_range(&mut rng, 0..len);
            if let Some(exp) = self.buffer.get(idx) {
                batch.push(exp.clone());
            }
        }
        batch
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

/// Subagente con Q-learning
#[derive(Debug, Clone)]
pub struct Subagent {
    config: SubagentConfig,
    q_table: HashMap<String, Vec<f32>>, // state_key -> action values
    experiences: ExperienceBuffer,
    episode_count: usize,
    total_reward: f32,
}

impl Subagent {
    pub fn new(config: SubagentConfig) -> Self {
        Self {
            config,
            q_table: HashMap::new(),
            experiences: ExperienceBuffer::new(10000),
            episode_count: 0,
            total_reward: 0.0,
        }
    }

    /// Convertir estado a clave string
    fn state_key(&self, state: &[f32]) -> String {
        state
            .iter()
            .map(|v| format!("{:.2}", v))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Elegir acción (epsilon-greedy)
    pub fn choose_action(&self, state: &[f32]) -> usize {
        let key = self.state_key(state);
        let num_actions = self.q_table.get(&key).map(|v| v.len()).unwrap_or(4);

        // Exploración
        if rand::random::<f32>() < self.config.exploration_rate {
            return rand::random::<usize>() % num_actions;
        }

        // Explotación - mejor acción
        if let Some(q_values) = self.q_table.get(&key) {
            q_values
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Agregar experiencia al buffer
    pub fn add_experience(&mut self, experience: Experience) {
        let reward = experience.reward;
        // Actualizar Q-table con la experiencia
        self.update_q(&experience);
        self.experiences.push(experience);
        self.total_reward += reward;
    }

    /// Actualizar Q-table con Q-learning
    fn update_q(&mut self, experience: &Experience) {
        let state_key = self.state_key(&experience.state);
        let next_state_key = self.state_key(&experience.next_state);
        let action = experience.action;
        let reward = experience.reward;

        // Calcular max_next_q primero (inmutable)
        let max_next_q = self
            .q_table
            .get(&next_state_key)
            .map(|q| q.iter().cloned().fold(f32::NEG_INFINITY, f32::max))
            .unwrap_or(0.0);

        // Ahora hacer el mutable borrow
        let q_values = self
            .q_table
            .entry(state_key.clone())
            .or_insert_with(|| vec![0.0; 4]);
        let num_actions = q_values.len();

        // Asegurar que tenemos suficientes acciones
        if action >= num_actions {
            while q_values.len() <= action {
                q_values.push(0.0);
            }
        }

        let target = reward + self.config.discount_factor * max_next_q;
        let current_q = q_values[action];
        q_values[action] = current_q + self.config.learning_rate * (target - current_q);
    }

    /// Aprender del buffer de experiencias
    pub fn learn(&mut self) {
        let batch = self.experiences.sample(32);
        for exp in batch {
            self.update_q(&exp);
        }
    }

    /// Reiniciar para nuevo episodio
    pub fn new_episode(&mut self) {
        self.episode_count += 1;
        self.total_reward = 0.0;
        // Reducir exploración gradualmente
        self.config.exploration_rate = (self.config.exploration_rate * 0.99).max(0.01);
    }

    /// Información del subagente
    pub fn info(&self) -> String {
        format!(
            "Subagent(role={}, episodes={}, exploration={:.3}, total_states={})",
            self.config.role,
            self.episode_count,
            self.config.exploration_rate,
            self.q_table.len()
        )
    }

    /// Get Q table size
    pub fn q_table_size(&self) -> usize {
        self.q_table.len()
    }

    /// Obtener conocimiento del subagente para Hive Mind
    pub fn get_knowledge(&self) -> String {
        let mut knowledge = Vec::new();

        for (state, q_values) in &self.q_table {
            let max_q = q_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            if max_q > 0.5 {
                knowledge.push(format!("state={}, q_max={:.2}", state, max_q));
            }
        }

        knowledge.join("; ")
    }

    /// Recibir conocimiento de otro subagente (comunicación activa)
    pub fn receive_knowledge(&mut self, _from_agent: &str, _insight: &str, usefulness: f32) {
        // Crear nuevas experiencias basadas en conocimiento recibido
        if usefulness > 0.5 {
            // Alta utilidad = explorar esa dirección
            let experience = Experience {
                state: vec![usefulness * 10.0, 0.5, 0.5, 0.5], // Estado basado en insight
                action: 1,                                     // Acción de explorar
                reward: usefulness,
                next_state: vec![usefulness * 11.0, 0.6, 0.6, 0.6],
                done: false,
            };
            self.add_experience(experience);
        }
    }

    /// Enviar insight a otro subagente
    pub fn send_insight(&self) -> Option<(String, f32)> {
        if self.total_reward > 10.0 || self.q_table.len() > 5 {
            let insight = format!(
                "insight: reward={:.1}, states={}, role={}",
                self.total_reward,
                self.q_table.len(),
                self.config.role
            );
            Some((insight, self.total_reward / 100.0))
        } else {
            None
        }
    }
}
