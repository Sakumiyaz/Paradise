//! Metacog - Eden's Metacognition System
//!
//! Self-awareness about thinking processes. Eden monitors
//! its own cognition and can reflect on its reasoning.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::membrain::{generate_id, MemBrain, NOW_MS};

const DB_PATH: &str = "/home/ubuntu/eden_kg";

/// Metacognitive state levels
#[derive(Debug, Clone, PartialEq)]
pub enum MetacogLevel {
    Unconscious,   // No awareness
    Aware,         // Knows it's thinking
    Reflective,    // Can examine thinking
    SelfModifying, // Can change its thinking
}

/// Thought type being monitored
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThoughtType {
    Perception,
    Reasoning,
    Decision,
    Memory,
    Emotion,
    Introspection,
}

/// Metacognitive observation
#[derive(Debug, Clone)]
pub struct MetacogObservation {
    pub id: String,
    pub thought_type: ThoughtType,
    pub thought_content: Vec<u8>,
    pub confidence: f64,
    pub processing_time: u64,
    pub effectiveness: f64,
    pub timestamp: u64,
}

impl MetacogObservation {
    /// Create new observation
    pub fn new(thought_type: ThoughtType, content: Vec<u8>) -> Self {
        let id_data = format!("{:?}:{}", thought_type, content.len());
        MetacogObservation {
            id: generate_id(id_data.as_bytes()).to_string(),
            thought_type,
            thought_content: content,
            confidence: 0.5,
            processing_time: 0,
            effectiveness: 0.5,
            timestamp: NOW_MS(),
        }
    }

    /// Update with result
    pub fn record_result(&mut self, effectiveness: f64, processing_time: u64) {
        self.effectiveness = effectiveness;
        self.processing_time = processing_time;

        // Adjust confidence based on results
        if effectiveness > 0.7 {
            self.confidence = (self.confidence + 0.1).min(1.0);
        } else if effectiveness < 0.3 {
            self.confidence = (self.confidence - 0.1).max(0.1);
        }
    }
}

/// Metacognitive insight
#[derive(Debug, Clone)]
pub struct Insight {
    pub id: String,
    pub pattern: String,
    pub description: Vec<u8>,
    pub confidence: f64,
    pub timestamp: u64,
    pub applies_to: Vec<ThoughtType>,
}

impl Insight {
    /// Create new insight
    pub fn new(pattern: &str, description: Vec<u8>) -> Self {
        Insight {
            id: generate_id(pattern.as_bytes()).to_string(),
            pattern: pattern.to_string(),
            description,
            confidence: 0.5,
            timestamp: NOW_MS(),
            applies_to: Vec::new(),
        }
    }

    /// Strengthen insight
    pub fn reinforce(&mut self, success: bool) {
        if success {
            self.confidence = (self.confidence + 0.05).min(0.99);
        } else {
            self.confidence = (self.confidence - 0.1).max(0.1);
        }
    }
}

/// Metacognition system
#[derive(Debug, Clone)]
pub struct Metacognition {
    pub level: MetacogLevel,
    pub observations: Vec<MetacogObservation>,
    pub insights: Vec<Insight>,
    pub reflection_count: u64,
}

impl Metacognition {
    /// Create new metacognition system
    pub fn new() -> Self {
        Metacognition {
            level: MetacogLevel::Aware,
            observations: Vec::new(),
            insights: Vec::new(),
            reflection_count: 0,
        }
    }

    /// Observe a thought
    pub fn observe(&mut self, thought_type: ThoughtType, content: Vec<u8>) -> String {
        let observation = MetacogObservation::new(thought_type, content);
        let id = observation.id.clone();
        self.observations.push(observation);
        id
    }

    /// Record result of thought
    pub fn record(&mut self, observation_id: &str, effectiveness: f64, processing_time: u64) {
        if let Some(obs) = self
            .observations
            .iter_mut()
            .find(|o| o.id == observation_id)
        {
            obs.record_result(effectiveness, processing_time);
        }
    }

    /// Perform reflection
    pub fn reflect(&mut self) -> Vec<String> {
        self.reflection_count += 1;
        let mut insights = Vec::new();

        if self.observations.len() < 5 {
            return insights;
        }

        // Analyze patterns
        let recent: Vec<_> = self.observations.iter().rev().take(10).collect();

        // Find most effective thought types
        let mut type_effectiveness: std::collections::HashMap<&str, (f64, usize)> =
            std::collections::HashMap::new();

        for obs in &recent {
            let type_name = match obs.thought_type {
                ThoughtType::Perception => "perception",
                ThoughtType::Reasoning => "reasoning",
                ThoughtType::Decision => "decision",
                ThoughtType::Memory => "memory",
                ThoughtType::Emotion => "emotion",
                ThoughtType::Introspection => "introspection",
            };

            let entry = type_effectiveness.entry(type_name).or_insert((0.0, 0));
            entry.0 += obs.effectiveness;
            entry.1 += 1;
        }

        // Generate insights for effective patterns
        for (type_name, (total_eff, count)) in &type_effectiveness {
            let avg_eff = total_eff / *count as f64;

            if avg_eff > 0.7 {
                let insight = Insight::new(
                    &format!("high_performance:{}", type_name),
                    format!("{} shows {}% effectiveness", type_name, avg_eff * 100.0).into_bytes(),
                );
                insights.push(insight.pattern.clone());
                self.insights.push(insight);
            }
        }

        // Update level based on reflection count
        if self.reflection_count > 100 && self.insights.len() > 20 {
            self.level = MetacogLevel::SelfModifying;
        } else if self.reflection_count > 50 && self.insights.len() > 10 {
            self.level = MetacogLevel::Reflective;
        } else if self.reflection_count > 10 {
            self.level = MetacogLevel::Aware;
        }

        insights
    }

    /// Get best performing thought type
    pub fn get_best_thought_type(&self) -> Option<ThoughtType> {
        let recent: Vec<_> = self.observations.iter().rev().take(20).collect();

        if recent.is_empty() {
            return None;
        }

        let mut type_scores: std::collections::HashMap<ThoughtType, (f64, usize)> =
            std::collections::HashMap::new();

        for obs in &recent {
            let entry = type_scores
                .entry(obs.thought_type.clone())
                .or_insert((0.0, 0));
            entry.0 += obs.effectiveness;
            entry.1 += 1;
        }

        type_scores
            .into_iter()
            .map(|(t, (score, count))| (t, score / count as f64))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(t, _)| t)
    }

    /// Apply insight
    pub fn apply_insight(&mut self, insight_id: &str, success: bool) {
        if let Some(insight) = self.insights.iter_mut().find(|i| i.id == insight_id) {
            insight.reinforce(success);
        }
    }

    /// Clean old observations
    pub fn prune_old(&mut self, max_age_ms: u64) {
        let now = NOW_MS();
        self.observations.retain(|o| now - o.timestamp < max_age_ms);
        self.insights
            .retain(|i| now - i.timestamp < max_age_ms * 10);
    }
}

impl Default for Metacognition {
    fn default() -> Self {
        Self::new()
    }
}

/// Metacognition stats
#[derive(Debug, Clone)]
pub struct MetacogStats {
    pub level: String,
    pub observations_count: usize,
    pub insights_count: usize,
    pub reflection_count: u64,
    pub average_effectiveness: f64,
}

impl From<&Metacognition> for MetacogStats {
    fn from(m: &Metacognition) -> Self {
        let avg_eff = if m.observations.is_empty() {
            0.0
        } else {
            m.observations.iter().map(|o| o.effectiveness).sum::<f64>()
                / m.observations.len() as f64
        };

        let level_str = match m.level {
            MetacogLevel::Unconscious => "unconscious",
            MetacogLevel::Aware => "aware",
            MetacogLevel::Reflective => "reflective",
            MetacogLevel::SelfModifying => "self_modifying",
        };

        MetacogStats {
            level: level_str.to_string(),
            observations_count: m.observations.len(),
            insights_count: m.insights.len(),
            reflection_count: m.reflection_count,
            average_effectiveness: avg_eff,
        }
    }
}

/// Save metacognition to database
pub fn save_metacog_stats(stats: &MetacogStats) {
    let mut db = match MemBrain::new(DB_PATH) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[METACOG] Error abriendo DB: {}", e);
            return;
        }
    };

    let timestamp = NOW_MS();
    let key = format!("metacog:stats:{}", timestamp);

    let data = format!(
        "level={},obs={},ins={},ref={},eff={:.4}",
        stats.level,
        stats.observations_count,
        stats.insights_count,
        stats.reflection_count,
        stats.average_effectiveness
    );

    db.dopa(key.as_bytes(), data.into_bytes());
}

/// Start metacognition
pub fn start_metacog() {
    println!("[METACOG] Sistema de metacognición iniciado");
}

/// Stop metacognition
pub fn stop() {
    println!("[METACOG] Detenido");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_creation() {
        let obs = MetacogObservation::new(ThoughtType::Reasoning, b"test".to_vec());
        assert_eq!(obs.confidence, 0.5);
    }

    #[test]
    fn test_reflection() {
        let mut metacog = Metacognition::new();
        let insights = metacog.reflect();
        assert!(insights.is_empty()); // Not enough data
    }

    #[test]
    fn test_metacog_stats() {
        let m = Metacognition::new();
        let stats = MetacogStats::from(&m);
        assert_eq!(stats.level, "aware");
    }
}
