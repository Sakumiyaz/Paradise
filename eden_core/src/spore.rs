//! Spore - Eden's Reproduction & Evolution System
//!
//! Spores carry genetic information between cells, enabling evolution
//! and the creation of new cell generations.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::membrain::{generate_id, rand_u64, NOW_MS};

/// Spore types based on transmission method
#[derive(Debug, Clone, PartialEq)]
pub enum SporeType {
    Airborne, // Random dispersion
    Directed, // Target specific cells
    Dormant,  // Waits for activation
    Hybrid,   // Combines multiple genomes
}

/// Spore state machine
#[derive(Debug, Clone, PartialEq)]
pub enum SporeState {
    Forming,
    Dormant,
    Active,
    Germinating,
    Absorbed,
    Degraded,
}

/// Spore - Genetic carrier for reproduction
#[derive(Debug, Clone)]
pub struct Spore {
    pub id: String,
    pub genome: Vec<u8>,
    pub spore_type: SporeType,
    pub state: SporeState,
    pub birth_time: u64,
    pub viability: f64,            // 0.0 - 1.0
    pub mutation_history: u64,     // Number of mutations
    pub generation: u64,           // Which generation this came from
    pub target_id: Option<String>, // For directed spores
    pub energy_reserve: f64,
}

impl Spore {
    /// Create a new spore from cell genome
    pub fn from_cell(genome: Vec<u8>, spore_type: SporeType, generation: u64) -> Self {
        Spore {
            id: generate_id(&genome).to_string(),
            genome,
            spore_type,
            state: SporeState::Forming,
            birth_time: NOW_MS(),
            viability: 1.0,
            mutation_history: 0,
            generation,
            target_id: None,
            energy_reserve: 0.5,
        }
    }

    /// Create a directed spore targeting a specific cell
    pub fn directed(genome: Vec<u8>, target_id: String, generation: u64) -> Self {
        let mut spore = Self::from_cell(genome, SporeType::Directed, generation);
        spore.target_id = Some(target_id);
        spore
    }

    /// Activate spore for dispersal
    pub fn activate(&mut self) {
        if self.state == SporeState::Dormant {
            self.state = SporeState::Active;
        }
    }

    /// Degrade over time
    pub fn degrade(&mut self, delta_time: u64) {
        // Viability decreases with time
        let decay = (delta_time as f64) / 100_000.0; // ~100 seconds to degrade 10%
        self.viability = (self.viability - decay).max(0.0);

        if self.viability <= 0.0 {
            self.state = SporeState::Degraded;
        }
    }

    /// Attempt germination
    pub fn germinate(&mut self) -> bool {
        if self.state != SporeState::Active {
            return false;
        }

        if self.viability < 0.3 {
            self.state = SporeState::Degraded;
            return false;
        }

        self.state = SporeState::Germinating;
        true
    }

    /// Create hybrid genome from two parents
    pub fn hybridize(genome_a: &[u8], genome_b: &[u8]) -> Vec<u8> {
        let min_len = genome_a.len().min(genome_b.len());
        let max_len = genome_a.len().max(genome_b.len());

        let mut hybrid = Vec::with_capacity(max_len);

        // Crossover at random point
        let crossover = (rand_u64() % (min_len as u64)) as usize;

        // Take genes from parent A before crossover
        hybrid.extend_from_slice(&genome_a[..crossover]);

        // Take genes from parent B after crossover
        hybrid.extend_from_slice(&genome_b[crossover..]);

        // Apply mutation to hybrid
        let mutation_count = ((hybrid.len() as f64) * 0.01) as usize;
        for _ in 0..mutation_count {
            if hybrid.is_empty() {
                break;
            }
            let idx = (rand_u64() as usize) % hybrid.len();
            hybrid[idx] = ((hybrid[idx] as u64 + rand_u64()) % 256) as u8;
        }

        hybrid
    }

    /// Apply mutation to genome
    pub fn mutate(&mut self) {
        if self.genome.is_empty() {
            return;
        }

        let mutation_count = ((self.genome.len() as f64) * 0.005) as usize;

        for _ in 0..mutation_count {
            let idx = (rand_u64() as usize) % self.genome.len();
            let mutation = ((self.genome[idx] as u64 + rand_u64()) % 256) as u8;
            self.genome[idx] = mutation;
            self.mutation_history += 1;
        }

        // Viability might be affected
        self.viability = (self.viability - 0.01).max(0.1);
    }

    /// Check if spore is viable
    pub fn is_viable(&self) -> bool {
        self.viability > 0.2 && self.state != SporeState::Degraded
    }

    /// Check if spore reached target
    pub fn reached_target(&self, cell_id: &str) -> bool {
        if let Some(ref target) = self.target_id {
            target == cell_id
        } else {
            // Airborne spores reach random targets
            rand_u64() % 100 < 30 // 30% chance per check
        }
    }

    /// Get spore age in milliseconds
    pub fn age(&self) -> u64 {
        NOW_MS() - self.birth_time
    }

    /// String representation
    pub fn to_string(&self) -> String {
        format!(
            "Spore({} {:?} V:{:.2} Gen:{} MH:{})",
            &self.id[..8],
            self.spore_type,
            self.viability,
            self.generation,
            self.mutation_history
        )
    }
}

impl Default for Spore {
    fn default() -> Self {
        Self::from_cell(vec![0; 64], SporeType::Airborne, 0)
    }
}
