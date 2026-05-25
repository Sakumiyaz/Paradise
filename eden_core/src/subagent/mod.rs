//! # SUBAGENT - Sistema de subagentes con aprendizaje desde cero
//!
//! Subagentes autónomos que pueden aprender de sus experiencias.
//! 100% Rust puro - sin dependencias externas.
#![allow(dead_code)]
#![allow(non_snake_case)]

mod genetic;
mod learning;

pub use genetic::{Chromosome, FitnessFunction, GeneticOptimizer};
pub use learning::{Experience, ExperienceBuffer, Subagent, SubagentConfig};

use std::collections::HashMap;

/// Sistema de subagentes con aprendizaje
#[derive(Debug, Clone)]
pub struct SubagentSystem {
    subagents: HashMap<String, Subagent>,
    max_subagents: usize,
}

impl SubagentSystem {
    pub fn new(max_subagents: usize) -> Self {
        Self {
            subagents: HashMap::new(),
            max_subagents,
        }
    }

    /// Crear nuevo subagente
    pub fn create_subagent(&mut self, id: &str, role: &str) -> Option<&mut Subagent> {
        if self.subagents.len() >= self.max_subagents {
            return None;
        }

        let config = SubagentConfig {
            role: role.to_string(),
            learning_rate: 0.01,
            discount_factor: 0.95,
            exploration_rate: 0.1,
        };

        self.subagents.insert(id.to_string(), Subagent::new(config));
        self.subagents.get_mut(id)
    }

    /// Entrenar subagente con experiencia
    pub fn train(&mut self, id: &str, experience: Experience) {
        if let Some(agent) = self.subagents.get_mut(id) {
            agent.add_experience(experience);
            agent.learn();
        }
    }

    /// Obtener acción del subagente
    pub fn act(&self, id: &str, state: &[f32]) -> Option<usize> {
        self.subagents.get(id).map(|a| a.choose_action(state))
    }

    /// Obtener información del subagente
    pub fn get_info(&self, id: &str) -> Option<String> {
        self.subagents.get(id).map(|a| a.info())
    }

    /// Listar todos los subagentes
    pub fn list_agents(&self) -> Vec<String> {
        self.subagents.keys().cloned().collect()
    }

    /// Tamaño del sistema
    pub fn len(&self) -> usize {
        self.subagents.len()
    }

    /// Obtener conocimiento compartido de todos los subagentes (Hive Mind)
    pub fn get_shared_knowledge(&self) -> Vec<String> {
        self.subagents
            .values()
            .map(|a| a.get_knowledge())
            .filter(|k| !k.is_empty())
            .collect()
    }

    /// Comunicación activa entre subagentes - compartir insights
    pub fn active_communication(&mut self) -> Vec<(String, String, f32)> {
        let mut messages = Vec::new();

        // Cada subagente envía insight a los demás
        let agent_ids: Vec<String> = self.subagents.keys().cloned().collect();
        let mut insights_from_all: Vec<(String, String, f32)> = Vec::new();

        for id in &agent_ids {
            if let Some(agent) = self.subagents.get(id) {
                if let Some((insight, usefulness)) = agent.send_insight() {
                    insights_from_all.push((id.clone(), insight, usefulness));
                }
            }
        }

        // Cada subagente recibe conocimiento de los demás
        for (source_id, insight, usefulness) in &insights_from_all {
            for target_id in &agent_ids {
                if target_id != source_id {
                    if let Some(agent) = self.subagents.get_mut(target_id) {
                        agent.receive_knowledge(source_id, insight, *usefulness);
                    }
                }
            }
            messages.push((source_id.clone(), insight.clone(), *usefulness));
        }

        messages
    }
}
