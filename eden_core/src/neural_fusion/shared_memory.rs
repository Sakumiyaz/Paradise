//! # Shared Consciousness - Memory and pattern sharing between fused entities
#![allow(unused_imports)]
#![allow(dead_code)]
use std::time::Instant;

use std::collections::HashMap;

/// Shared memory block
#[derive(Debug, Clone)]
pub struct SharedBlock {
    pub id: String,
    pub data: Vec<u8>,
    pub owner: String, // Which entity originally owned it
    pub access: AccessLevel,
    pub created_at: u64,
    pub access_count: u32,
}

impl SharedBlock {
    pub fn new(id: String, data: Vec<u8>, owner: String) -> Self {
        Self {
            id,
            data,
            owner,
            access: AccessLevel::ReadWrite,
            created_at: current_time_ms(),
            access_count: 0,
        }
    }

    pub fn access(&mut self) {
        self.access_count += 1;
    }
}

/// Access level for shared memory
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessLevel {
    ReadOnly,
    ReadWrite,
    Execute,
}

impl Default for AccessLevel {
    fn default() -> Self {
        AccessLevel::ReadOnly
    }
}

/// Shared pattern (cognitive pattern, behavior, etc)
#[derive(Debug, Clone)]
pub struct SharedPattern {
    pub id: String,
    pub pattern_type: PatternType,
    pub data: Vec<u8>,
    pub strength: f32, // 0.0 - 1.0, how strongly embedded
    pub original_entity: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    Behavioral,
    Cognitive,
    Emotional,
    Sensory,
    Memory,
}

impl SharedPattern {
    pub fn new(id: String, pattern_type: PatternType, data: Vec<u8>, owner: &str) -> Self {
        Self {
            id,
            pattern_type,
            data,
            strength: 0.5,
            original_entity: owner.to_string(),
        }
    }

    /// Strengthen the pattern
    pub fn reinforce(&mut self, amount: f32) {
        self.strength = (self.strength + amount).min(1.0);
    }

    /// Weaken the pattern
    pub fn weaken(&mut self, amount: f32) {
        self.strength = (self.strength - amount).max(0.0);
    }
}

/// Shared consciousness space
pub struct SharedConsciousness {
    memories: HashMap<String, SharedBlock>,
    patterns: HashMap<String, SharedPattern>,
    connections: HashMap<String, Vec<String>>, // Which entities share what
}

impl SharedConsciousness {
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            patterns: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    /// Share a memory with another entity
    pub fn share_memory(&mut self, memory: SharedBlock, with_entities: &[String]) {
        let id = memory.id.clone();
        self.memories.insert(id.clone(), memory);

        for entity in with_entities {
            self.connections
                .entry(entity.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
    }

    /// Share a pattern
    pub fn share_pattern(&mut self, pattern: SharedPattern, with_entities: &[String]) {
        let id = pattern.id.clone();
        self.patterns.insert(id.clone(), pattern);

        for entity in with_entities {
            self.connections
                .entry(entity.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
    }

    /// Get shared memories for an entity
    pub fn get_shared_memories(&self, entity: &str) -> Vec<&SharedBlock> {
        self.connections
            .get(entity)
            .map(|ids| ids.iter().filter_map(|id| self.memories.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get shared patterns
    pub fn get_shared_patterns(&self, entity: &str) -> Vec<&SharedPattern> {
        self.connections
            .get(entity)
            .map(|ids| ids.iter().filter_map(|id| self.patterns.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get total shared memory size
    pub fn total_memory_bytes(&self) -> usize {
        self.memories.values().map(|m| m.data.len()).sum()
    }

    /// Consolidate - merge similar patterns
    pub fn consolidate(&mut self) {
        let patterns: Vec<(String, SharedPattern)> = self.patterns.drain().collect();

        for (id, mut pattern) in patterns {
            // Check for similar patterns
            let similar: Vec<String> = self
                .patterns
                .values()
                .filter(|p| {
                    p.pattern_type == pattern.pattern_type
                        && p.id != id
                        && Self::patterns_similar(&p.data, &pattern.data)
                })
                .map(|p| p.id.clone())
                .collect();

            // Merge similar patterns
            for s_id in similar {
                if let Some(other) = self.patterns.get_mut(&s_id) {
                    pattern.strength = (pattern.strength + other.strength) / 2.0;
                }
            }

            self.patterns.insert(id, pattern);
        }
    }

    fn patterns_similar(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() || a.is_empty() {
            return false;
        }

        let mut diff_count = 0;
        for (ai, bi) in a.iter().zip(b.iter()) {
            if (*ai as i16 - *bi as i16).abs() > 5 {
                diff_count += 1;
            }
        }

        (diff_count as f32 / (a.len() as f32)) < 0.2
    }

    /// Get statistics
    pub fn stats(&self) -> SharedStats {
        SharedStats {
            memory_count: self.memories.len(),
            pattern_count: self.patterns.len(),
            total_bytes: self.total_memory_bytes(),
            entity_count: self.connections.len(),
        }
    }
}

impl Default for SharedConsciousness {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics
#[derive(Debug, Clone)]
pub struct SharedStats {
    pub memory_count: usize,
    pub pattern_count: usize,
    pub total_bytes: usize,
    pub entity_count: usize,
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_share_memory() {
        let mut sc = SharedConsciousness::new();

        let memory = SharedBlock::new("mem1".to_string(), vec![1, 2, 3, 4], "entity_a".to_string());

        sc.share_memory(memory, &["entity_b".to_string()]);

        let shared = sc.get_shared_memories("entity_b");
        assert_eq!(shared.len(), 1);
    }
}
