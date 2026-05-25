//! Wisdom - Eden's Accumulated Knowledge System
//!
//! Wisdom represents the crystallized experience of the organism,
//! encoded as neural patterns and survival heuristics.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::membrain::{generate_id, NOW_MS};

/// Types of wisdom patterns
#[derive(Debug, Clone, PartialEq)]
pub enum WisdomType {
    Heuristic, // Rule of thumb
    Strategic, // Long-term planning
    Tactical,  // Short-term decisions
    Intuitive, // Pattern recognition
    Learned,   // From experience
    Innate,    // Born with it
}

/// Pattern strength levels
#[derive(Debug, Clone, PartialEq)]
pub enum PatternStrength {
    Weak,       // < 0.3
    Developing, // 0.3 - 0.6
    Strong,     // 0.6 - 0.85
    Mastered,   // > 0.85
}

/// Wisdom pattern - A learned or innate cognitive pattern
#[derive(Debug, Clone)]
pub struct WisdomPattern {
    pub id: String,
    pub pattern_type: WisdomType,
    pub pattern_hash: String,
    pub content: Vec<u8>,
    pub strength: f64,
    pub reinforcement_count: u64,
    pub decay_rate: f64,
    pub birth_time: u64,
    pub last_used: u64,
    pub source: String, // Where this wisdom came from
    pub confidence: f64,
    pub applicability_context: Vec<String>,
}

impl WisdomPattern {
    /// Create a new wisdom pattern
    pub fn new(pattern_type: WisdomType, content: Vec<u8>, source: &str) -> Self {
        let now = NOW_MS();
        let pattern_hash = Self::hash_content(&content);

        WisdomPattern {
            id: generate_id(&content).to_string(),
            pattern_type,
            pattern_hash,
            content,
            strength: 0.1, // Starts weak, must be reinforced
            reinforcement_count: 0,
            decay_rate: 0.00001,
            birth_time: now,
            last_used: now,
            source: source.to_string(),
            confidence: 0.5,
            applicability_context: Vec::new(),
        }
    }

    /// Hash content for identification
    fn hash_content(content: &[u8]) -> String {
        let mut hash: u64 = 0x5A3C9F;
        for (i, &byte) in content.iter().enumerate() {
            hash ^= (byte as u64).wrapping_mul(0x9E3779B9);
            hash = hash.rotate_left(7);
            hash ^= (i as u64).wrapping_mul(0x85EBCA6B);
        }
        format!("{:016x}", hash)
    }

    /// Reinforce this pattern (successful use)
    pub fn reinforce(&mut self, success: f64) {
        self.reinforcement_count += 1;

        // Non-linear reinforcement - harder to reach higher strengths
        let gain = success * (1.0 - self.strength) * 0.2;
        self.strength = (self.strength + gain).min(1.0);

        // Confidence increases with successful uses
        if self.reinforcement_count > 5 {
            self.confidence = (self.confidence + 0.05).min(0.99);
        }

        self.last_used = NOW_MS();
    }

    /// Decay this pattern (time without use)
    pub fn decay(&mut self) {
        let now = NOW_MS();
        let time_since_use = now - self.last_used;

        // Exponential decay based on time
        let decay_factor = (-(time_since_use as f64) * self.decay_rate).exp();
        self.strength *= decay_factor;

        // Decay confidence too
        self.confidence *= 0.9999;

        // Pattern becomes weak if decayed too much
        if self.strength < 0.05 {
            self.strength = 0.05;
        }
    }

    /// Get pattern strength category
    pub fn get_strength_level(&self) -> PatternStrength {
        if self.strength < 0.3 {
            PatternStrength::Weak
        } else if self.strength < 0.6 {
            PatternStrength::Developing
        } else if self.strength < 0.85 {
            PatternStrength::Strong
        } else {
            PatternStrength::Mastered
        }
    }

    /// Check if pattern is applicable in context
    pub fn is_applicable(&self, context: &[String]) -> bool {
        if self.applicability_context.is_empty() {
            return true; // Universal pattern
        }

        context
            .iter()
            .any(|c| self.applicability_context.iter().any(|apc| apc == c))
    }

    /// Add applicability context
    pub fn add_context(&mut self, context: &str) {
        if !self.applicability_context.contains(&context.to_string()) {
            self.applicability_context.push(context.to_string());
        }
    }

    /// Fuse two patterns (crossover)
    pub fn fuse_with(&self, other: &WisdomPattern) -> WisdomPattern {
        let min_len = self.content.len().min(other.content.len());
        let mut new_content = Vec::with_capacity(min_len);

        for i in 0..min_len {
            if i % 2 == 0 {
                new_content.push(self.content[i]);
            } else {
                new_content.push(other.content[i]);
            }
        }

        let mut new_pattern = Self::new(WisdomType::Learned, new_content, "fusion");

        // Average strength
        new_pattern.strength = (self.strength + other.strength) / 2.0;
        new_pattern.confidence = (self.confidence + other.confidence) / 2.0;

        new_pattern
    }

    /// Get age in milliseconds
    pub fn age(&self) -> u64 {
        NOW_MS() - self.birth_time
    }

    /// Time since last use
    pub fn idle_time(&self) -> u64 {
        NOW_MS() - self.last_used
    }

    /// String representation
    pub fn to_string(&self) -> String {
        format!(
            "Wisdom({} {:?} S:{:.2} C:{:.2} R:{} I:{}ms)",
            &self.id[..8],
            self.pattern_type,
            self.strength,
            self.confidence,
            self.reinforcement_count,
            self.idle_time()
        )
    }
}

impl Default for WisdomPattern {
    fn default() -> Self {
        Self::new(WisdomType::Intuitive, vec![0; 32], "default")
    }
}

/// Wisdom repository - collection of wisdom patterns
#[derive(Debug, Clone)]
pub struct WisdomRepository {
    pub patterns: Vec<WisdomPattern>,
    pub capacity: usize,
}

impl WisdomRepository {
    /// Create new wisdom repository
    pub fn new(capacity: usize) -> Self {
        WisdomRepository {
            patterns: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Add a new pattern
    pub fn add(&mut self, pattern: WisdomPattern) {
        if self.patterns.len() >= self.capacity {
            self.prune_weakest();
        }
        self.patterns.push(pattern);
    }

    /// Remove weakest patterns to make room
    fn prune_weakest(&mut self) {
        if self.patterns.is_empty() {
            return;
        }

        // Find weakest
        let mut weakest_idx = 0;
        let mut weakest_strength = f64::MAX;

        for (i, p) in self.patterns.iter().enumerate() {
            if p.strength < weakest_strength {
                weakest_strength = p.strength;
                weakest_idx = i;
            }
        }

        self.patterns.remove(weakest_idx);
    }

    /// Get strongest pattern matching type
    pub fn get_strongest(&self, pattern_type: WisdomType) -> Option<&WisdomPattern> {
        self.patterns
            .iter()
            .filter(|p| p.pattern_type == pattern_type)
            .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap())
    }

    /// Apply decay to all patterns
    pub fn decay_all(&mut self) {
        for pattern in &mut self.patterns {
            pattern.decay();
        }
    }

    /// Count patterns by type
    pub fn count_by_type(&self, pattern_type: &WisdomType) -> usize {
        self.patterns
            .iter()
            .filter(|p| p.pattern_type == *pattern_type)
            .count()
    }

    /// Get average strength
    pub fn average_strength(&self) -> f64 {
        if self.patterns.is_empty() {
            return 0.0;
        }
        self.patterns.iter().map(|p| p.strength).sum::<f64>() / self.patterns.len() as f64
    }
}
