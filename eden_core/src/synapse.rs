//! # Synapse - Red Neural A-Life
//!
//! Sistema de sinapsis neural para organismos A-Life.
//! 100% Rust puro - sin dependencias externas.
//!
//! Usa MemBrain como motor de almacenamiento neural.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

// Importar el módulo membrain
use crate::membrain::MemBrain;

// ============================================================================
// HELPERS - Sin dependencias externas
// ============================================================================

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Lee u64 de bytes sin macros externas
fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    if bytes.len() < offset + 8 {
        return None;
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes[offset..offset + 8]);
    Some(u64::from_le_bytes(arr))
}

/// Lee u32 de bytes sin macros externas
fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    if bytes.len() < offset + 4 {
        return None;
    }
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&bytes[offset..offset + 4]);
    Some(u32::from_le_bytes(arr))
}

/// Lee f64 de bytes sin macros externas
fn read_f64(bytes: &[u8], offset: usize) -> f64 {
    if bytes.len() < offset + 8 {
        return 0.0;
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes[offset..offset + 8]);
    f64::from_le_bytes(arr)
}

/// Lee string de bytes
fn read_string(bytes: &[u8], offset: usize) -> Option<(String, usize)> {
    let len = read_u32(bytes, offset)? as usize;
    if bytes.len() < offset + 4 + len {
        return None;
    }
    let s = String::from_utf8(bytes[offset + 4..offset + 4 + len].to_vec()).ok()?;
    Some((s, offset + 4 + len))
}

// ============================================================================
// ESTRUCTURAS DE DATOS - Serialización manual
// ============================================================================

/// Configuración del synapsis
#[derive(Debug, Clone)]
pub struct Config {
    pub organism_id: u64,
    pub organism_type: String,
    pub age_cycles: u64,
    pub state: OrganismState,
    pub energy: f64,
    pub health: f64,
    pub pos_x: f64,
    pub pos_y: f64,
    pub generation: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrganismState {
    Alive,
    Dormant,
    Dead,
    Mutating,
}

impl Config {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.organism_id.to_le_bytes());

        let type_bytes = self.organism_type.as_bytes();
        bytes.extend_from_slice(&(type_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(type_bytes);

        bytes.extend_from_slice(&self.age_cycles.to_le_bytes());

        let state_byte = match self.state {
            OrganismState::Alive => 0u8,
            OrganismState::Dormant => 1,
            OrganismState::Dead => 2,
            OrganismState::Mutating => 3,
        };
        bytes.push(state_byte);

        bytes.extend_from_slice(&self.energy.to_le_bytes());
        bytes.extend_from_slice(&self.health.to_le_bytes());
        bytes.extend_from_slice(&self.pos_x.to_le_bytes());
        bytes.extend_from_slice(&self.pos_y.to_le_bytes());
        bytes.extend_from_slice(&self.generation.to_le_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut offset = 0;

        let organism_id = read_u64(bytes, offset)?;
        offset += 8;

        let (organism_type, new_offset) = read_string(bytes, offset)?;
        offset = new_offset;

        let age_cycles = read_u64(bytes, offset)?;
        offset += 8;

        if offset >= bytes.len() {
            return None;
        }
        let state_byte = bytes[offset];
        let state = match state_byte {
            0 => OrganismState::Alive,
            1 => OrganismState::Dormant,
            2 => OrganismState::Dead,
            3 => OrganismState::Mutating,
            _ => return None,
        };
        offset += 1;

        let energy = read_f64(bytes, offset);
        offset += 8;

        let health = read_f64(bytes, offset);
        offset += 8;

        let pos_x = read_f64(bytes, offset);
        offset += 8;

        let pos_y = read_f64(bytes, offset);
        offset += 8;

        let generation = read_u32(bytes, offset)?;

        Some(Self {
            organism_id,
            organism_type,
            age_cycles,
            state,
            energy,
            health,
            pos_x,
            pos_y,
            generation,
        })
    }
}

/// Genoma del organismo
#[derive(Debug, Clone)]
pub struct Genome {
    pub id: u64,
    pub parent_id: u64,
    pub genes: Vec<u8>,
    pub checksum: u64,
    pub mutations: u32,
    pub created_at: u64,
}

impl Genome {
    pub fn new(parent_id: u64, genes: Vec<u8>) -> Self {
        let id = Self::generate_id(&genes);
        let checksum = Self::calculate_checksum(&genes);

        Self {
            id,
            parent_id,
            genes,
            checksum,
            mutations: 0,
            created_at: now_millis(),
        }
    }

    fn generate_id(genes: &[u8]) -> u64 {
        let mut hash: u64 = 0x6a09e667f3bccf85;
        for (i, &byte) in genes.iter().enumerate() {
            hash = hash
                .rotate_left(7)
                .wrapping_add(byte as u64)
                .wrapping_mul(0xbf584c640c1de965);
            hash ^= (i as u64).wrapping_mul(0x9e3779b97f4a7c15).rotate_left(11);
        }
        hash ^ (hash >> 32)
    }

    fn calculate_checksum(genes: &[u8]) -> u64 {
        let mut checksum: u64 = 0x9e3779b97f4a7c15;
        for (i, &byte) in genes.iter().enumerate() {
            checksum = checksum.rotate_left(5)
                ^ (byte as u64)
                    .wrapping_mul(0x9e3779b97f4a7c15)
                    .wrapping_add(i as u64);
        }
        checksum
    }

    pub fn verify(&self) -> bool {
        Self::calculate_checksum(&self.genes) == self.checksum
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.id.to_le_bytes());
        bytes.extend_from_slice(&self.parent_id.to_le_bytes());
        bytes.extend_from_slice(&(self.genes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.genes);
        bytes.extend_from_slice(&self.checksum.to_le_bytes());
        bytes.extend_from_slice(&self.mutations.to_le_bytes());
        bytes.extend_from_slice(&self.created_at.to_le_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut offset = 0;

        let id = read_u64(bytes, offset)?;
        offset += 8;

        let parent_id = read_u64(bytes, offset)?;
        offset += 8;

        let genes_len = read_u32(bytes, offset)? as usize;
        offset += 4;

        if bytes.len() < offset + genes_len {
            return None;
        }
        let genes = bytes[offset..offset + genes_len].to_vec();
        offset += genes_len;

        let checksum = read_u64(bytes, offset)?;
        offset += 8;

        let mutations = read_u32(bytes, offset)?;
        offset += 4;

        let created_at = read_u64(bytes, offset)?;

        Some(Self {
            id,
            parent_id,
            genes,
            checksum,
            mutations,
            created_at,
        })
    }

    /// Mutación simple usando XOR (sin rand)
    pub fn mutate(&mut self, rate: u8) -> bool {
        let threshold = (rate as f64 * 255.0) as u8;
        let mut mutated = false;

        for gene in &mut self.genes {
            // Usar hash de tiempo como "random" source
            let time_byte =
                ((now_millis() ^ (*gene as u64).wrapping_mul(0x9e37_79b9_7f4_a7c5)) & 0xFF) as u8;
            if time_byte < threshold {
                *gene = (*gene ^ time_byte).wrapping_add(1);
                mutated = true;
            }
        }

        if mutated {
            self.mutations += 1;
            self.checksum = Self::calculate_checksum(&self.genes);
        }

        mutated
    }
}

/// Estado de la célula
#[derive(Debug, Clone)]
pub struct CellState {
    pub cell_id: u64,
    pub config: Config,
    pub genome: Genome,
    pub memory: Vec<u8>,
    pub metabolic_rate: f64,
    pub optimal_temp: f64,
    pub temp_tolerance: f64,
    pub flags: CellFlags,
}

#[derive(Debug, Clone)]
pub struct CellFlags {
    pub can_move: bool,
    pub can_reproduce: bool,
    pub is_reactive: bool,
    pub has_memory: bool,
}

impl Default for CellFlags {
    fn default() -> Self {
        Self {
            can_move: true,
            can_reproduce: true,
            is_reactive: true,
            has_memory: true,
        }
    }
}

impl CellState {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.cell_id.to_le_bytes());

        let config_bytes = self.config.to_bytes();
        bytes.extend_from_slice(&(config_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&config_bytes);

        let genome_bytes = self.genome.to_bytes();
        bytes.extend_from_slice(&(genome_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&genome_bytes);

        bytes.extend_from_slice(&(self.memory.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.memory);

        bytes.extend_from_slice(&self.metabolic_rate.to_le_bytes());
        bytes.extend_from_slice(&self.optimal_temp.to_le_bytes());
        bytes.extend_from_slice(&self.temp_tolerance.to_le_bytes());

        bytes.push(if self.flags.can_move { 1 } else { 0 });
        bytes.push(if self.flags.can_reproduce { 1 } else { 0 });
        bytes.push(if self.flags.is_reactive { 1 } else { 0 });
        bytes.push(if self.flags.has_memory { 1 } else { 0 });

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut offset = 0;

        let cell_id = read_u64(bytes, offset)?;
        offset += 8;

        let config_len = read_u32(bytes, offset)? as usize;
        offset += 4;
        if bytes.len() < offset + config_len {
            return None;
        }
        let config = Config::from_bytes(&bytes[offset..offset + config_len])?;
        offset += config_len;

        let genome_len = read_u32(bytes, offset)? as usize;
        offset += 4;
        if bytes.len() < offset + genome_len {
            return None;
        }
        let genome = Genome::from_bytes(&bytes[offset..offset + genome_len])?;
        offset += genome_len;

        let memory_len = read_u32(bytes, offset)? as usize;
        offset += 4;
        if bytes.len() < offset + memory_len {
            return None;
        }
        let memory = bytes[offset..offset + memory_len].to_vec();
        offset += memory_len;

        let metabolic_rate = read_f64(bytes, offset);
        offset += 8;

        let optimal_temp = read_f64(bytes, offset);
        offset += 8;

        let temp_tolerance = read_f64(bytes, offset);
        offset += 8;

        if offset + 4 > bytes.len() {
            return None;
        }
        let flags = CellFlags {
            can_move: bytes[offset] != 0,
            can_reproduce: bytes[offset + 1] != 0,
            is_reactive: bytes[offset + 2] != 0,
            has_memory: bytes[offset + 3] != 0,
        };

        Some(Self {
            cell_id,
            config,
            genome,
            memory,
            metabolic_rate,
            optimal_temp,
            temp_tolerance,
            flags,
        })
    }
}

/// Pesos neurales para red neuronal
#[derive(Debug, Clone)]
pub struct NeuralWeights {
    pub network_id: u64,
    pub input_weights: HashMap<u64, f64>,
    pub hidden_weights: HashMap<u64, f64>,
    pub output_weights: HashMap<u64, f64>,
    pub bias: f64,
    pub learning_rate: f64,
}

impl Default for NeuralWeights {
    fn default() -> Self {
        Self::new(0)
    }
}

impl NeuralWeights {
    pub fn new(network_id: u64) -> Self {
        Self {
            network_id,
            input_weights: HashMap::new(),
            hidden_weights: HashMap::new(),
            output_weights: HashMap::new(),
            bias: 0.0,
            learning_rate: 0.01,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.network_id.to_le_bytes());
        bytes.extend_from_slice(&self.bias.to_le_bytes());
        bytes.extend_from_slice(&self.learning_rate.to_le_bytes());

        bytes.extend_from_slice(&(self.input_weights.len() as u32).to_le_bytes());
        for (k, v) in &self.input_weights {
            bytes.extend_from_slice(&k.to_le_bytes());
            bytes.extend_from_slice(&v.to_le_bytes());
        }

        bytes.extend_from_slice(&(self.hidden_weights.len() as u32).to_le_bytes());
        for (k, v) in &self.hidden_weights {
            bytes.extend_from_slice(&k.to_le_bytes());
            bytes.extend_from_slice(&v.to_le_bytes());
        }

        bytes.extend_from_slice(&(self.output_weights.len() as u32).to_le_bytes());
        for (k, v) in &self.output_weights {
            bytes.extend_from_slice(&k.to_le_bytes());
            bytes.extend_from_slice(&v.to_le_bytes());
        }

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut offset = 0;

        let network_id = read_u64(bytes, offset)?;
        offset += 8;

        let bias = read_f64(bytes, offset);
        offset += 8;

        let learning_rate = read_f64(bytes, offset);
        offset += 8;

        let input_len = read_u32(bytes, offset)? as usize;
        offset += 4;
        let mut input_weights = HashMap::new();
        for _ in 0..input_len {
            if bytes.len() < offset + 16 {
                return None;
            }
            let k = read_u64(bytes, offset)?;
            offset += 8;
            let v = read_f64(bytes, offset);
            offset += 8;
            input_weights.insert(k, v);
        }

        let hidden_len = read_u32(bytes, offset)? as usize;
        offset += 4;
        let mut hidden_weights = HashMap::new();
        for _ in 0..hidden_len {
            if bytes.len() < offset + 16 {
                return None;
            }
            let k = read_u64(bytes, offset)?;
            offset += 8;
            let v = read_f64(bytes, offset);
            offset += 8;
            hidden_weights.insert(k, v);
        }

        let output_len = read_u32(bytes, offset)? as usize;
        offset += 4;
        let mut output_weights = HashMap::new();
        for _ in 0..output_len {
            if bytes.len() < offset + 16 {
                return None;
            }
            let k = read_u64(bytes, offset)?;
            offset += 8;
            let v = read_f64(bytes, offset);
            offset += 8;
            output_weights.insert(k, v);
        }

        Some(Self {
            network_id,
            input_weights,
            hidden_weights,
            output_weights,
            bias,
            learning_rate,
        })
    }
}

/// Estadísticas de la célula
#[derive(Debug, Clone)]
pub struct CellStats {
    pub life_cycles: u64,
    pub energy_consumed: f64,
    pub reproductions: u32,
    pub successful_mutations: u32,
    pub distance_traveled: f64,
    pub avg_response_time: f64,
    pub memory_used: u64,
    pub last_update: u64,
}

impl Default for CellStats {
    fn default() -> Self {
        Self {
            life_cycles: 0,
            energy_consumed: 0.0,
            reproductions: 0,
            successful_mutations: 0,
            distance_traveled: 0.0,
            avg_response_time: 0.0,
            memory_used: 0,
            last_update: now_millis(),
        }
    }
}

impl CellStats {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.life_cycles.to_le_bytes());
        bytes.extend_from_slice(&self.energy_consumed.to_le_bytes());
        bytes.extend_from_slice(&self.reproductions.to_le_bytes());
        bytes.extend_from_slice(&self.successful_mutations.to_le_bytes());
        bytes.extend_from_slice(&self.distance_traveled.to_le_bytes());
        bytes.extend_from_slice(&self.avg_response_time.to_le_bytes());
        bytes.extend_from_slice(&self.memory_used.to_le_bytes());
        bytes.extend_from_slice(&self.last_update.to_le_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 64 {
            return None;
        }

        Some(Self {
            life_cycles: read_u64(bytes, 0)?,
            energy_consumed: read_f64(bytes, 8),
            reproductions: read_u32(bytes, 16)?,
            successful_mutations: read_u32(bytes, 20)?,
            distance_traveled: read_f64(bytes, 24),
            avg_response_time: read_f64(bytes, 32),
            memory_used: read_u64(bytes, 40)?,
            last_update: read_u64(bytes, 48)?,
        })
    }
}

// ============================================================================
// SINAPSIS - Red Neural Principal
// ============================================================================

pub struct Synapse {
    brain: MemBrain,
    cache: RwLock<HashMap<String, Vec<u8>>>,
    stats: SynapseStats,
}

#[derive(Debug, Clone)]
pub struct SynapseStats {
    pub total_queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub brain_operations: u64,
}

impl Default for SynapseStats {
    fn default() -> Self {
        Self {
            total_queries: 0,
            cache_hits: 0,
            cache_misses: 0,
            brain_operations: 0,
        }
    }
}

impl Synapse {
    pub fn new(storage_path: &str) -> std::io::Result<Self> {
        let brain = MemBrain::new(storage_path)?;

        Ok(Self {
            brain,
            cache: RwLock::new(HashMap::new()),
            stats: SynapseStats::default(),
        })
    }

    /// Inserta o actualiza una célula
    pub fn upsert_cell(&mut self, cell: &CellState) -> std::io::Result<()> {
        let key = format!("cell:{}", cell.cell_id);
        let data = cell.to_bytes();

        self.brain.gaba(key.as_bytes(), data.clone());

        let mut cache = self.cache.write().unwrap();
        cache.insert(key, data);

        self.stats.brain_operations += 1;
        Ok(())
    }

    /// Recupera una célula por ID
    pub fn get_cell(&mut self, cell_id: u64) -> Option<CellState> {
        let key = format!("cell:{}", cell_id);

        {
            let cache = self.cache.read().unwrap();
            if let Some(data) = cache.get(&key) {
                self.stats.cache_hits += 1;
                self.stats.total_queries += 1;
                return CellState::from_bytes(data);
            }
        }

        self.stats.cache_misses += 1;
        self.stats.total_queries += 1;

        let results = self.brain.search(key.as_bytes());
        for data in results {
            if let Some(cell) = CellState::from_bytes(&data) {
                if cell.cell_id == cell_id {
                    return Some(cell);
                }
            }
        }

        None
    }

    /// Obtiene todas las células
    pub fn get_all_cells(&self) -> Vec<CellState> {
        let mut cells = Vec::new();
        let results = self.brain.search(b"cell:");
        for data in results {
            if let Some(cell) = CellState::from_bytes(&data) {
                cells.push(cell);
            }
        }
        cells
    }

    /// Elimina una célula
    pub fn delete_cell(&mut self, cell_id: u64) -> std::io::Result<()> {
        let key = format!("cell:{}", cell_id);

        let mut cache = self.cache.write().unwrap();
        cache.remove(&key);

        self.stats.brain_operations += 1;
        Ok(())
    }

    /// Guarda pesos neurales
    pub fn save_weights(&mut self, weights: &NeuralWeights) -> std::io::Result<()> {
        let key = format!("weights:{}", weights.network_id);
        let data = weights.to_bytes();

        self.brain.gaba(key.as_bytes(), data);

        self.stats.brain_operations += 1;
        Ok(())
    }

    /// Recupera pesos neurales
    pub fn get_weights(&self, network_id: u64) -> Option<NeuralWeights> {
        let key = format!("weights:{}", network_id);

        {
            let cache = self.cache.read().unwrap();
            if let Some(data) = cache.get(&key) {
                return NeuralWeights::from_bytes(data);
            }
        }

        let results = self.brain.search(key.as_bytes());
        for data in results {
            if let Some(weights) = NeuralWeights::from_bytes(&data) {
                if weights.network_id == network_id {
                    return Some(weights);
                }
            }
        }

        None
    }

    /// Guarda un genoma
    pub fn save_genome(&mut self, genome: &Genome) -> std::io::Result<()> {
        let key = format!("genome:{}", genome.id);
        let data = genome.to_bytes();

        self.brain.dopa(key.as_bytes(), data);

        self.stats.brain_operations += 1;
        Ok(())
    }

    /// Recupera un genoma
    pub fn get_genome(&self, genome_id: u64) -> Option<Genome> {
        let key = format!("genome:{}", genome_id);

        let results = self.brain.search(key.as_bytes());
        for data in results {
            if let Some(genome) = Genome::from_bytes(&data) {
                if genome.id == genome_id {
                    return Some(genome);
                }
            }
        }

        None
    }

    /// Guarda estadísticas
    pub fn save_stats(&mut self, cell_id: u64, stats: &CellStats) -> std::io::Result<()> {
        let key = format!("stats:{}", cell_id);
        let data = stats.to_bytes();

        self.brain.gluta(key.as_bytes(), data);

        self.stats.brain_operations += 1;
        Ok(())
    }

    /// Recupera estadísticas
    pub fn get_stats(&self, cell_id: u64) -> Option<CellStats> {
        let key = format!("stats:{}", cell_id);

        let results = self.brain.search(key.as_bytes());
        for data in results {
            if let Some(stats) = CellStats::from_bytes(&data) {
                return Some(stats);
            }
        }

        None
    }

    /// Ejecuta metabolismo
    pub fn metabolize(&mut self) {
        self.brain.metabolize();
    }

    /// Persiste estado
    pub fn persist(&self) -> std::io::Result<()> {
        self.brain.persist()
    }

    /// Dump de estado
    pub fn dump(&self) -> String {
        format!(
            "═══ SYNAPSIS STATUS ═══\n\
             Total Queries: {}\n\
             Cache Hits: {} ({}%)\n\
             Cache Misses: {}\n\
             Brain Operations: {}\n\
             \n{}",
            self.stats.total_queries,
            self.stats.cache_hits,
            if self.stats.total_queries > 0 {
                (self.stats.cache_hits as f64 / self.stats.total_queries as f64 * 100.0) as u32
            } else {
                0
            },
            self.stats.cache_misses,
            self.stats.brain_operations,
            self.brain.dump()
        )
    }

    pub fn get_synapse_stats(&self) -> SynapseStats {
        self.stats.clone()
    }

    pub fn clear_cache(&mut self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    pub fn autopoiesis(&mut self) -> crate::membrain::AutopoiesisReport {
        self.brain.autopoiesis()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_roundtrip() {
        let config = Config {
            organism_id: 12345,
            organism_type: "TestOrganism".to_string(),
            age_cycles: 100,
            state: OrganismState::Alive,
            energy: 0.85,
            health: 0.95,
            pos_x: 10.5,
            pos_y: 20.3,
            generation: 5,
        };

        let bytes = config.to_bytes();
        let restored = Config::from_bytes(&bytes).unwrap();

        assert_eq!(config.organism_id, restored.organism_id);
        assert_eq!(config.organism_type, restored.organism_type);
        assert_eq!(config.state, restored.state);
    }

    #[test]
    fn test_genome_creation() {
        let genes = b"ATGCATGCATGC".to_vec();
        let genome = Genome::new(0, genes.clone());

        assert_eq!(genome.parent_id, 0);
        assert!(genome.verify());
    }

    #[test]
    fn test_genome_mutation() {
        let genes = b"AAAAAAAAAAAAAAAA".to_vec();
        let mut genome = Genome::new(0, genes);

        genome.mutate(128); // 128/255 ≈ 0.5 mutation rate
        assert!(genome.verify());
    }

    #[test]
    fn test_cell_state_roundtrip() {
        let cell = CellState {
            cell_id: 1,
            config: Config {
                organism_id: 1,
                organism_type: "Test".to_string(),
                age_cycles: 10,
                state: OrganismState::Alive,
                energy: 0.8,
                health: 0.9,
                pos_x: 5.0,
                pos_y: 10.0,
                generation: 1,
            },
            genome: Genome::new(0, b"ATGC".to_vec()),
            memory: b"memory".to_vec(),
            metabolic_rate: 1.0,
            optimal_temp: 37.0,
            temp_tolerance: 5.0,
            flags: CellFlags::default(),
        };

        let bytes = cell.to_bytes();
        let restored = CellState::from_bytes(&bytes).unwrap();

        assert_eq!(cell.cell_id, restored.cell_id);
        assert_eq!(cell.config.organism_id, restored.config.organism_id);
    }

    #[test]
    fn test_weights_roundtrip() {
        let mut weights = NeuralWeights::new(1);
        weights.input_weights.insert(1, 0.5);
        weights.hidden_weights.insert(10, 0.7);
        weights.bias = 0.1;

        let bytes = weights.to_bytes();
        let restored = NeuralWeights::from_bytes(&bytes).unwrap();

        assert_eq!(weights.network_id, restored.network_id);
        assert_eq!(weights.input_weights.len(), restored.input_weights.len());
    }
}
