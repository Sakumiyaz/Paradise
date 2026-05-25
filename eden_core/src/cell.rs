//! Cell - Eden's Fundamental Life Unit
//!
//! Every cell is an autonomous agent with its own genome, metabolism,
//! and consciousness. Cells are the building blocks of Eden's A-Life system.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::membrain::{generate_id, rand_u64, NOW_MS};

/// Cell states representing its lifecycle
#[derive(Debug, Clone, PartialEq)]
pub enum CellState {
    Born,
    Alive,
    Mutating,
    Reproducing,
    Dying,
    Dead,
}

/// Metabolic process types
#[derive(Debug, Clone)]
pub enum MetabolismType {
    Anaerobic,
    Aerobic,
    Photynthetic,
}

/// Cell - The fundamental autonomous unit of Eden
#[derive(Debug, Clone)]
pub struct Cell {
    pub id: String,
    pub genome: Vec<u8>,
    pub state: CellState,
    pub birth_time: u64,
    pub last_metabolism: u64,
    pub metabolism_type: MetabolismType,
    pub energy: f64,
    pub health: f64,
    pub replication_count: u64,
    pub mutation_rate: f64,
    pub consciousness_level: f64,
}

impl Cell {
    /// Create a new cell with minimal initial energy
    pub fn genesis(genome: Vec<u8>) -> Self {
        let now = NOW_MS();

        Cell {
            id: generate_id(&genome).to_string(),
            genome,
            state: CellState::Born,
            birth_time: now,
            last_metabolism: now,
            metabolism_type: MetabolismType::Aerobic,
            energy: 1.0,
            health: 1.0,
            replication_count: 0,
            mutation_rate: 0.001,
            consciousness_level: 0.0,
        }
    }

    /// Initialize cell systems
    pub fn activate(&mut self) {
        self.state = CellState::Alive;
        self.last_metabolism = NOW_MS();
    }

    /// Perform metabolic process
    pub fn metabolize(&mut self, available: f64) {
        let now = NOW_MS();
        let elapsed = now - self.last_metabolism;

        // Energy decay over time
        let decay = (elapsed as f64 / 1000.0) * 0.001;
        self.energy = (self.energy - decay).max(0.0).min(1.0);

        // Energy absorption
        self.energy = (self.energy + available * 0.1).min(1.0);

        // Health correlates with energy
        self.health = self.energy;

        self.last_metabolism = now;
    }

    /// Trigger cell reproduction
    pub fn reproduce(&mut self) -> Option<Vec<u8>> {
        if self.state != CellState::Alive || self.energy < 0.7 {
            return None;
        }

        self.state = CellState::Reproducing;
        self.energy *= 0.5;
        self.replication_count += 1;

        // Create offspring genome with potential mutations
        let offspring = self.mutate_genome();

        self.state = CellState::Alive;

        Some(offspring)
    }

    /// Mutate genome with controlled randomness
    fn mutate_genome(&self) -> Vec<u8> {
        let mut new_genome = self.genome.clone();
        let mutations = ((new_genome.len() as f64) * self.mutation_rate) as usize;

        for _ in 0..mutations {
            if new_genome.is_empty() {
                break;
            }
            let idx = (rand_u64() as usize) % new_genome.len();
            new_genome[idx] = ((new_genome[idx] as u64 + rand_u64()) % 256) as u8;
        }

        new_genome
    }

    /// Random mutation triggered
    pub fn mutate(&mut self) {
        if rand_u64() % 100 < (self.mutation_rate * 100.0) as u64 {
            self.state = CellState::Mutating;
            self.genome = self.mutate_genome();

            // Mutation might improve or degrade mutation rate
            if rand_u64() % 2 == 0 {
                self.mutation_rate = (self.mutation_rate * 1.1).min(0.1);
            } else {
                self.mutation_rate = (self.mutation_rate * 0.9).max(0.0001);
            }

            self.state = CellState::Alive;
        }
    }

    /// Check if cell should die
    pub fn should_die(&self) -> bool {
        self.energy <= 0.0 || self.health <= 0.0 || (NOW_MS() - self.birth_time) > 1_000_000_000_000
        // ~11 days max
    }

    /// Initiate death sequence
    pub fn die(&mut self) {
        self.state = CellState::Dying;
    }

    /// Complete death
    pub fn finalize_death(&mut self) {
        self.state = CellState::Dead;
    }

    /// Get cell age in milliseconds
    pub fn age(&self) -> u64 {
        NOW_MS() - self.birth_time
    }

    /// String representation
    pub fn to_string(&self) -> String {
        format!(
            "Cell({} {:?} E:{:.2} H:{:.2} M:{:.4} A:{}ms)",
            &self.id[..8],
            self.state,
            self.energy,
            self.health,
            self.mutation_rate,
            self.age()
        )
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::genesis(vec![0; 64])
    }
}
