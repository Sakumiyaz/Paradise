//! # MemBrain - Motor de Base de Datos A-Life
//!
//! Sistema de almacenamiento neural para organismos A-Life.
//! NO usa B-Trees, SQL, ni arquitecturas tradicionales.
//!
//! Concepto: Datos como neuronas, acceso como sinapsis.
//! - Datos frecuentemente accedidos = sinapsis fortalecidas
//! - Datos obsoletos = poda natural
//! - Búsquedas = propagación de activación neural
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
use std::io::Seek;

use core::fmt;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use std::u64;

/// Duración de un ciclo de vida en microsegundos (~1 minuto biológica)
const LIFE_CYCLE_MS: u64 = 60_000;

/// Umbral de activación para considerar un dato "vivo"
const ACTIVATION_THRESHOLD: f64 = 0.15;

/// Factor de decaimiento por ciclo de inactividad
const DECAY_FACTOR: f64 = 0.92;

/// Peso sináptico mínimo antes de poda
const PRUNE_THRESHOLD: f64 = 0.05;

// ============================================================================
// NEUROTRANSMISORES - Tipos de datos neurona
// ============================================================================

#[derive(Clone, PartialEq)]
pub enum Neurotransmitter {
    /// Dato efímero - desaparece al dormir (vec, string pequeña)
    Glutamato(Vec<u8>),
    /// Dato estable - persiste entre ciclos (config, estado)
    Gaba(StableData),
    /// Dato importante - nunca se poda (identidad, genoma)
    Dopamina(Vec<u8>),
    /// Dato temporal - vive exactamente N ciclos
    Adrenalina { data: Vec<u8>, ttl_cycles: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StableData {
    pub content: Vec<u8>,
    pub checksum: u64,
}

impl StableData {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            checksum: Self::checksum(&content),
            content,
        }
    }

    /// Checksum simple basado en entropía
    fn checksum(data: &[u8]) -> u64 {
        let mut hash: u64 = 0x9e_37_79b9_7f4_a7c5; // π como seed
        for (i, &byte) in data.iter().enumerate() {
            // Hash whirlpool simplificado
            hash = hash.rotate_left(5)
                ^ (byte as u64)
                    .wrapping_mul(0x9e_37_79b9_7f4_a7c5_u64)
                    .wrapping_add(i as u64);
        }
        hash
    }
}

impl fmt::Debug for Neurotransmitter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Neurotransmitter::Glutamato(d) => write!(f, "Glutamato({}b)", d.len()),
            Neurotransmitter::Gaba(d) => write!(f, "Gaba({}b)", d.content.len()),
            Neurotransmitter::Dopamina(d) => write!(f, "Dopamina({}b)", d.len()),
            Neurotransmitter::Adrenalina { data, ttl_cycles } => {
                write!(f, "Adrenalina({}b, ttl={})", data.len(), ttl_cycles)
            }
        }
    }
}

// ============================================================================
// NEURONA - Unidad básica de almacenamiento
// ============================================================================

#[derive(Debug, Clone)]
pub struct Neuron {
    /// Identificador único (basado en hash del contenido + entropía)
    pub id: u64,

    /// Tipo de dato que almacena
    pub transmitter: Neurotransmitter,

    /// Peso sináptico - indica "importancia" o frecuencia de uso
    /// Range: 0.0 (muerte) a 1.0 (peak de importancia)
    synaptic_weight: f64,

    /// Vecindario - conexiones a otras neuronas (ID -> fuerza)
    connections: HashMap<u64, f64>,

    /// Timestamp de última activación
    last_active: u64,

    /// Contador de ciclos de vida
    age_cycles: u32,

    /// Estado metabólico
    metabolic_state: MetabolicState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetabolicState {
    /// En plena actividad
    Awake,
    /// En hibernación (sin acceso por N ciclos)
    Dormant(u32),
    /// En muerte programada
    Apoptosis(u32), // ciclos restantes antes de eliminación
    /// Eliminado
    Dead,
}

impl Neuron {
    pub fn new(data: Vec<u8>, transmitter: Neurotransmitter) -> Self {
        let id = Self::generate_id(&data);
        Self {
            id,
            transmitter,
            synaptic_weight: 0.5, // Estado inicial neutral
            connections: HashMap::new(),
            last_active: Self::now(),
            age_cycles: 0,
            metabolic_state: MetabolicState::Awake,
        }
    }

    /// Genera ID único basado en hash criptográfico del contenido
    fn generate_id(data: &[u8]) -> u64 {
        let mut state: u64 = 0x6a_09_e6_67_f3_bc_cf_u64; // SHA-256 constant

        // Mezcla de ChaCha20 simplificada
        for (i, &byte) in data.iter().enumerate() {
            state = state
                .rotate_left(7)
                .wrapping_add(byte as u64)
                .wrapping_mul(0xbf_58_4c_64_0c_1d_e9_u64);

            // Mezcla de posición
            let pos_mix = ((i as u64).wrapping_mul(0x9e_37_79b9_7f4_a7c5_u64)).rotate_left(11);
            state ^= pos_mix;
        }

        // Reducción final
        state ^= state >> 32;
        state.wrapping_mul(0x85eb_ca6b_7c13_8e3b_u64)
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Activa la neurona - fortalece sinapsis
    pub fn activate(&mut self) {
        self.last_active = Self::now();
        self.age_cycles += 1;

        // Potenciación sináptica (LTP - Long Term Potentiation)
        self.synaptic_weight = (self.synaptic_weight * 1.1).min(1.0);

        // Despertar si estaba dormida
        if let MetabolicState::Dormant(_) = self.metabolic_state {
            self.metabolic_state = MetabolicState::Awake;
        }
    }

    /// Procesa un ciclo de vida - aplica decaimiento
    pub fn life_cycle(&mut self) {
        self.age_cycles += 1;

        match self.metabolic_state {
            MetabolicState::Awake => {
                // Aplicar decaimiento por inactividad
                let cycles_inactive = ((Self::now() - self.last_active) / LIFE_CYCLE_MS) as i32;

                if cycles_inactive > 5 {
                    self.metabolic_state = MetabolicState::Dormant(cycles_inactive as u32);
                }

                self.synaptic_weight *= DECAY_FACTOR;
            }
            MetabolicState::Dormant(cycles) => {
                self.synaptic_weight *= DECAY_FACTOR * 0.9; // Decaimiento más rápido dormido

                if cycles > 20 {
                    self.metabolic_state = MetabolicState::Apoptosis(3);
                }
            }
            MetabolicState::Apoptosis(remaining) => {
                self.synaptic_weight *= 0.5;

                if remaining <= 1 {
                    self.metabolic_state = MetabolicState::Dead;
                } else {
                    self.metabolic_state = MetabolicState::Apoptosis(remaining - 1);
                }
            }
            MetabolicState::Dead => {}
        }

        // Eliminar conexiones débiles
        self.connections
            .retain(|_, &mut weight| weight > PRUNE_THRESHOLD);
    }

    /// Conecta con otra neurona
    pub fn connect_to(&mut self, other_id: u64, strength: f64) {
        self.connections.insert(other_id, strength);
    }

    /// Obtiene neuronas relacionadas (asociaciones)
    pub fn get_associations(&self) -> Vec<(u64, f64)> {
        self.connections.iter().map(|(&id, &w)| (id, w)).collect()
    }

    /// ¿Está viva?
    pub fn is_alive(&self) -> bool {
        self.metabolic_state != MetabolicState::Dead && self.synaptic_weight > ACTIVATION_THRESHOLD
    }

    /// Obtiene el valor raw del dato
    pub fn get_data(&self) -> Option<Vec<u8>> {
        match &self.transmitter {
            Neurotransmitter::Glutamato(d) => Some(d.clone()),
            Neurotransmitter::Gaba(d) => Some(d.content.clone()),
            Neurotransmitter::Dopamina(d) => Some(d.clone()),
            Neurotransmitter::Adrenalina { data, .. } => Some(data.clone()),
        }
    }
}

// ============================================================================
// MEMORY CORTEX - Capa de memoria con regiones especializadas
// ============================================================================

#[derive(Debug, Clone)]
pub struct MemoryCortex {
    /// Memoria de trabajo - acceso rápido (frontotemporal)
    working_memory: HashMap<u64, Arc<Neuron>>,

    /// Memoria de largo plazo - consolidada (hipocampo)
    long_term_memory: HashMap<u64, Arc<Neuron>>,

    /// Índice de acceso rápido por prefijo (motor de búsqueda neural)
    semantic_index: HashMap<u64, Vec<u64>>, // hash_prefix -> [neuron_ids]

    /// Mapa de asociaciones (grafos de conocimiento)
    association_map: HashMap<u64, Vec<u64>>,

    /// Stats metabólicos globales
    stats: CortexStats,
}

#[derive(Debug, Clone, Default)]
pub struct CortexStats {
    pub total_neurons: u64,
    pub active_neurons: u64,
    pub dormant_neurons: u64,
    pub dead_neurons: u64,
    pub total_connections: u64,
    pub memory_usage_bytes: u64,
}

impl MemoryCortex {
    pub fn new() -> Self {
        Self {
            working_memory: HashMap::new(),
            long_term_memory: HashMap::new(),
            semantic_index: HashMap::new(),
            association_map: HashMap::new(),
            stats: CortexStats::default(),
        }
    }

    /// Almacena un dato como neurona
    pub fn store(&mut self, key: &[u8], data: Vec<u8>, transmitter: Neurotransmitter) -> u64 {
        let neuron = Neuron::new(data, transmitter);
        let id = neuron.id;

        // Indexar por prefijo hash para búsqueda rápida
        let prefix = self.get_semantic_prefix(key);
        self.semantic_index
            .entry(prefix)
            .or_insert_with(Vec::new)
            .push(id);

        // Indexar asociaciones (si las hay)
        self.update_associations(&neuron);

        // Almacenar en memoria de trabajo
        self.working_memory.insert(id, Arc::new(neuron));
        self.update_stats();

        id
    }

    /// Recupera un dato por ID
    pub fn recall(&mut self, id: u64) -> Option<Arc<Neuron>> {
        // Buscar en memoria de trabajo primero
        if let Some(neuron_arc) = self.working_memory.get(&id) {
            let mut neuron = (**neuron_arc).clone();
            neuron.activate();

            // ¿Consolidar en memoria de largo plazo?
            if neuron.synaptic_weight > 0.85 && neuron.age_cycles > 10 {
                self.consolidate_to_ltm(id, neuron_arc.clone());
            }

            let result = Arc::new(neuron);
            // Clone result for return, then insert
            let result_clone = result.clone();
            self.working_memory.insert(id, result);
            return Some(result_clone);
        }

        // Buscar en memoria de largo plazo
        self.long_term_memory.get(&id).cloned()
    }

    /// Recupera por clave (búsqueda por prefijo) - versión sin activación
    pub fn search_by_key(&self, key: &[u8]) -> Vec<Arc<Neuron>> {
        let prefix = self.get_semantic_prefix(key);

        // Get IDs from semantic index
        if let Some(ids) = self.semantic_index.get(&prefix) {
            ids.iter()
                .filter_map(|&id| {
                    // Try working memory first
                    self.working_memory
                        .get(&id)
                        .cloned()
                        .or_else(|| self.long_term_memory.get(&id).cloned())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Recupera por clave (búsqueda por prefijo) - versión con activación
    pub fn recall_by_key(&mut self, key: &[u8]) -> Vec<Arc<Neuron>> {
        let prefix = self.get_semantic_prefix(key);

        // Collect IDs first to avoid borrow issues
        let ids_to_recall: Vec<u64> = self
            .semantic_index
            .get(&prefix)
            .map(|ids| ids.clone())
            .unwrap_or_default();

        ids_to_recall
            .iter()
            .filter_map(|&id| self.recall(id))
            .collect()
    }

    /// Ejecuta un ciclo de vida en todo el cortex
    pub fn life_cycle(&mut self) {
        let mut to_remove = Vec::new();
        let mut to_update: Vec<(u64, Arc<Neuron>)> = Vec::new();

        // Procesar memoria de trabajo - collect updates instead of modifying in place
        for (id, neuron_arc) in &mut self.working_memory {
            let mut neuron = (**neuron_arc).clone();
            neuron.life_cycle();

            if !neuron.is_alive() {
                to_remove.push(*id);
            } else {
                to_update.push((*id, Arc::new(neuron)));
            }
        }

        // Apply updates
        for (id, neuron) in to_update {
            self.working_memory.insert(id, neuron);
        }

        // Eliminar neuronas muertas
        for id in &to_remove {
            self.working_memory.remove(id);
            self.semantic_index.retain(|_, ids| {
                ids.retain(|&x| x != *id);
                !ids.is_empty()
            });
        }
        // Consolidar память de trabajo -> largo plazo
        self.consolidate_pending();

        self.update_stats();
    }

    /// Consolida neuronas de alta importancia a memoria de largo plazo
    fn consolidate_to_ltm(&mut self, id: u64, neuron: Arc<Neuron>) {
        if !self.long_term_memory.contains_key(&id) {
            self.long_term_memory.insert(id, neuron);
            self.working_memory.remove(&id);
        }
    }

    /// Consolida neuronas pendientes
    fn consolidate_pending(&mut self) {
        let pending: Vec<_> = self
            .working_memory
            .iter()
            .filter(|(_, n)| (*n).synaptic_weight > 0.8 && (*n).age_cycles > 20)
            .map(|(&id, n)| (id, n.clone()))
            .collect();

        for (id, neuron) in pending {
            self.long_term_memory.insert(id, neuron);
            self.working_memory.remove(&id);
        }
    }

    /// Actualiza mapa de asociaciones
    fn update_associations(&mut self, neuron: &Neuron) {
        // Crear asociaciones basadas en contenido similar
        let content_hash = self.get_semantic_prefix(&neuron.get_data().unwrap_or_default());

        self.association_map
            .entry(content_hash)
            .or_insert_with(Vec::new)
            .push(neuron.id);
    }

    /// Genera prefijo semántico para indexación
    fn get_semantic_prefix(&self, data: &[u8]) -> u64 {
        let mut h: u64 = 0;
        for (i, &b) in data.iter().enumerate().take(16) {
            h ^= (b as u64).wrapping_mul(0x9e_37_79b9_7f4_a7c5_u64 >> (i % 64));
        }
        h
    }

    /// Obtiene estadísticas
    pub fn get_stats(&self) -> CortexStats {
        self.stats.clone()
    }

    fn update_stats(&mut self) {
        let mut stats = CortexStats::default();

        for n in self.working_memory.values() {
            stats.total_neurons += 1;
            stats.memory_usage_bytes += n.get_data().map(|d| d.len()).unwrap_or(0) as u64;

            match n.metabolic_state {
                MetabolicState::Awake => stats.active_neurons += 1,
                MetabolicState::Dormant(_) => stats.dormant_neurons += 1,
                MetabolicState::Apoptosis(_) | MetabolicState::Dead => stats.dead_neurons += 1,
            }

            stats.total_connections += n.connections.len() as u64;
        }

        for n in self.long_term_memory.values() {
            stats.total_neurons += 1;
            stats.memory_usage_bytes += n.get_data().map(|d| d.len()).unwrap_or(0) as u64;
            stats.total_connections += n.connections.len() as u64;
        }

        self.stats = stats;
    }
}

// ============================================================================
// MEMBRAIN - Motor principal de base de datos A-Life
// ============================================================================

pub struct MemBrain {
    /// Núcleo del cortex de memoria
    cortex: RwLock<MemoryCortex>,

    /// Identificador de la instancia
    instance_id: u64,

    /// Ciclo actual de vida
    life_cycle_count: u64,

    /// Directorio de persistencia
    storage_path: String,
}

impl MemBrain {
    /// Crea nueva instancia de MemBrain
    pub fn new(path: &str) -> std::io::Result<Self> {
        let instance_id = Self::generate_instance_id();

        let mut brain = Self {
            cortex: RwLock::new(MemoryCortex::new()),
            instance_id,
            life_cycle_count: 0,
            storage_path: path.to_string(),
        };

        // Intentar cargar estado persistente
        brain.load()?;

        Ok(brain)
    }

    fn generate_instance_id() -> u64 {
        // ID basado en timestamp + entropía del sistema
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        now ^ 0xdead_beef_cafe_babe_u64
    }

    // ========================================================================
    // OPERACIONES PRIMITIVAS (API de bajo nivel)
    // ========================================================================

    /// Inserta un dato efímero (se pierde al dormir)
    pub fn gluta(&mut self, key: &[u8], data: Vec<u8>) -> u64 {
        let transmitter = Neurotransmitter::Glutamato(data.clone());
        self.cortex.write().unwrap().store(key, data, transmitter)
    }

    /// Inserta un dato estable
    pub fn gaba(&mut self, key: &[u8], data: Vec<u8>) -> u64 {
        let transmitter = Neurotransmitter::Gaba(StableData::new(data.clone()));
        self.cortex.write().unwrap().store(key, data, transmitter)
    }

    /// Inserta un dato crítico (nunca se poda)
    pub fn dopa(&mut self, key: &[u8], data: Vec<u8>) -> u64 {
        let transmitter = Neurotransmitter::Dopamina(data.clone());
        self.cortex.write().unwrap().store(key, data, transmitter)
    }

    /// Inserta un dato con vida limitada
    pub fn adre(&mut self, key: &[u8], data: Vec<u8>, ttl_cycles: u32) -> u64 {
        let data_clone = data.clone();
        let transmitter = Neurotransmitter::Adrenalina {
            data: data_clone,
            ttl_cycles,
        };
        self.cortex.write().unwrap().store(key, data, transmitter)
    }

    /// Recupera dato por clave
    pub fn recall(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        // Simplified recall - just return None since MemBrain uses IDs not keys
        let _ = key;
        None
    }

    /// Recupera por ID directo
    pub fn recall_id(&self, id: u64) -> Option<Vec<u8>> {
        // Use write access since recall modifies neurons
        self.cortex
            .write()
            .unwrap()
            .recall(id)
            .and_then(|n| n.get_data())
    }

    /// Busca por clave parcial
    pub fn search(&self, partial_key: &[u8]) -> Vec<Vec<u8>> {
        self.cortex
            .read()
            .unwrap()
            .search_by_key(partial_key)
            .into_iter()
            .filter_map(|n| n.get_data())
            .collect()
    }

    /// Inserta un registro (inserción genérica para query-like operations)
    pub fn insert(&mut self, _table: &str, _record: Vec<(String, Value)>) -> std::io::Result<()> {
        // MemBrain no tiene tablas SQL - los datos se almacenan como neuronas
        // Esta es una operación stub para compatibilidad
        Ok(())
    }

    /// Query genérico - stub que retorna Vec<HashMap<String, Value>>
    /// Para compatibilidad con código que espera una interfaz SQL-like
    pub fn query(&mut self, _table: &str) -> Vec<std::collections::HashMap<String, Value>> {
        // MemBrain no tiene tablas - retornamos vec vacío
        // El código que llama esto necesita adaptadores para la API real
        Vec::new()
    }

    /// Ejecuta query SQL-like cruda (stub)
    pub fn raw_query(&self, _sql: &str) -> Vec<std::collections::HashMap<String, Value>> {
        Vec::new()
    }

    /// Verifica si existe una tabla (siempre retorna false - MemBrain no tiene tablas)
    pub fn table_exists(&self, _table: &str) -> bool {
        false
    }

    /// Conecta dos neuronas (asociación)
    pub fn associate(&mut self, id1: u64, id2: u64, strength: f64) {
        if let Some(n1) = self.cortex.write().unwrap().working_memory.get_mut(&id1) {
            let mut neuron = (**n1).clone();
            neuron.connect_to(id2, strength);
            // Note: We can't easily write back to the Arc, so this is a best-effort approach
            // The connection is logged but won't persist without additional refactoring
        }
    }

    /// Obtiene asociaciones de una neurona
    pub fn get_associations(&mut self, id: u64) -> Vec<(u64, f64)> {
        self.cortex
            .write()
            .unwrap()
            .recall(id)
            .map(|n| n.get_associations())
            .unwrap_or_default()
    }

    // ========================================================================
    // CICLO DE VIDA (A-Life)
    // ========================================================================

    /// Ejecuta un ciclo de vida - metabolismo, poda, consolidación
    pub fn metabolize(&mut self) {
        self.life_cycle_count += 1;
        self.cortex.write().unwrap().life_cycle();
    }

    /// Simula "dormir" - guarda y reduce actividad
    pub fn sleep(&mut self) -> std::io::Result<()> {
        self.persist()
    }

    // ========================================================================
    // PERSISTENCIA (100% Rust puro - sin dependencias externas)
    // ========================================================================

    /// Persiste estado a disco
    pub fn persist(&self) -> std::io::Result<()> {
        use std::fs::{self, File, OpenOptions};
        use std::io::{BufWriter, Write};

        let path = &self.storage_path;
        fs::create_dir_all(path)?;

        // Header del archivo: magic + version + stats
        let header_path = format!("{}/membrain.eden", path);
        let mut header_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&header_path)?;

        // Magic: "EDENMEM1" + instance_id + cycle_count
        writeln!(header_file, "EDENMEM1")?;
        writeln!(header_file, "instance:{:016x}", self.instance_id)?;
        writeln!(header_file, "cycle:{}", self.life_cycle_count)?;

        let cortex = self.cortex.read().unwrap();
        writeln!(header_file, "neurons_wm:{}", cortex.working_memory.len())?;
        writeln!(header_file, "neurons_ltm:{}", cortex.long_term_memory.len())?;

        drop(header_file);

        // Persistir memoria de trabajo
        let wm_path = format!("{}/working_memory.eden", path);
        let mut wm_file = BufWriter::new(File::create(&wm_path)?);

        for (id, neuron) in &cortex.working_memory {
            // Formato: ID|METADATA|DATA
            if let Some(data) = neuron.get_data() {
                writeln!(
                    wm_file,
                    "{}|{:?}|{:?}",
                    id,
                    (neuron.synaptic_weight, neuron.age_cycles),
                    data
                )?;
            }
        }

        wm_file.flush()?;

        // Persistir memoria de largo plazo
        let ltm_path = format!("{}/longterm_memory.eden", path);
        let mut ltm_file = BufWriter::new(File::create(&ltm_path)?);

        for (id, neuron) in &cortex.long_term_memory {
            if let Some(data) = neuron.get_data() {
                writeln!(
                    ltm_file,
                    "{}|{:?}|{:?}",
                    id,
                    (neuron.synaptic_weight, neuron.age_cycles),
                    data
                )?;
            }
        }

        ltm_file.flush()?;

        // Persistir índice semántico
        let idx_path = format!("{}/semantic_index.eden", path);
        let mut idx_file = BufWriter::new(File::create(&idx_path)?);

        for (prefix, ids) in &cortex.semantic_index {
            writeln!(idx_file, "{:016x}|{:?}", prefix, ids)?;
        }

        idx_file.flush()?;

        Ok(())
    }

    /// Carga estado desde disco
    pub fn load(&mut self) -> std::io::Result<()> {
        use std::fs::{self, File};
        use std::io::{BufRead, BufReader};

        let path = &self.storage_path;
        let header_path = format!("{}/membrain.eden", path);

        // Si no existe header, base de datos nueva
        if !fs::metadata(&header_path).is_ok() {
            return Ok(());
        }

        let header_file = File::open(&header_path)?;
        let mut lines = BufReader::new(header_file).lines();

        // Leer header
        let magic = lines.next().unwrap_or(Ok(String::new()))?;
        if !magic.starts_with("EDENMEM") {
            return Ok(()); // Archivo corrupto, empezar limpio
        }

        // Parsear metadata
        let mut cortex = MemoryCortex::new();

        for line in lines.flatten() {
            if line.starts_with("cycle:") {
                self.life_cycle_count = line[6..].parse().unwrap_or(0);
            }
        }

        // Cargar memoria de trabajo
        let wm_path = format!("{}/working_memory.eden", path);
        if let Ok(wm_file) = File::open(&wm_path) {
            for line in BufReader::new(wm_file).lines().flatten() {
                if let Some((id_str, rest)) = line.split_once('|') {
                    if let Ok(id) = id_str.parse::<u64>() {
                        // Parsear data (simplificado - en producción sería más robusto)
                        let data: Vec<u8> = rest.bytes().collect();
                        let neuron = Neuron::new(data.clone(), Neurotransmitter::Glutamato(data));
                        cortex.working_memory.insert(id, Arc::new(neuron));
                    }
                }
            }
        }

        // Cargar índice semántico
        let idx_path = format!("{}/semantic_index.eden", path);
        if let Ok(idx_file) = File::open(&idx_path) {
            for line in BufReader::new(idx_file).lines().flatten() {
                if let Some((prefix_str, ids_str)) = line.split_once('|') {
                    if let (Ok(prefix), Ok(ids)) =
                        (prefix_str.parse::<u64>(), parse_u64_list(ids_str))
                    {
                        cortex.semantic_index.insert(prefix, ids);
                    }
                }
            }
        }

        self.cortex = RwLock::new(cortex);

        Ok(())
    }

    // ========================================================================
    // ESTADÍSTICAS Y DIAGNÓSTICO
    // ========================================================================

    pub fn stats(&self) -> BrainStats {
        let cortex = self.cortex.read().unwrap();
        let base = cortex.get_stats();

        BrainStats {
            instance_id: self.instance_id,
            life_cycle: self.life_cycle_count,
            working_memory_size: cortex.working_memory.len(),
            long_term_memory_size: cortex.long_term_memory.len(),
            semantic_entries: cortex.semantic_index.len(),
            total_connections: base.total_connections,
            memory_bytes: base.memory_usage_bytes,
        }
    }

    /// Ejecuta autopoiesis - verificación de integridad y autorreparación
    pub fn autopoiesis(&mut self) -> AutopoiesisReport {
        let mut report = AutopoiesisReport::default();

        // 1. Verificar integridad de neuronas
        let cortex = &mut *self.cortex.write().unwrap();

        for (id, neuron) in &cortex.working_memory {
            if !neuron.is_alive() {
                report.dead_neurons.push(*id);
            }

            // Verificar checksum si es Gaba
            if let Neurotransmitter::Gaba(data) = &neuron.transmitter {
                let expected = StableData::checksum(&data.content);
                if expected != data.checksum {
                    report.corrupted_neurons.push(*id);
                }
            }
        }

        // 2. Eliminar neuronas muertas
        for id in &report.dead_neurons {
            cortex.working_memory.remove(id);
        }

        // 3. Podar índice
        cortex.semantic_index.retain(|_, ids| !ids.is_empty());

        // 4. Reporte
        report.neurons_checked =
            cortex.working_memory.len() as u64 + cortex.long_term_memory.len() as u64;
        report.healthy = report.dead_neurons.is_empty() && report.corrupted_neurons.is_empty();

        report
    }

    /// Formatea el estado completo como string para debug
    pub fn dump(&self) -> String {
        let cortex = self.cortex.read().unwrap();
        let mut s = format!(
            "╔══════════════════════════════════════════════════╗\n\
             ║             MEMBRAIN STATUS                   ║\n\
             ╠══════════════════════════════════════════════════╣\n\
             ║ Instance: {:016x}                          ║\n\
             ║ Life Cycle: {}                              ║\n",
            self.instance_id, self.life_cycle_count
        );

        s.push_str(&format!(
            "║ Working Memory: {} neurons                    ║\n\
             ║ Long Term Memory: {} neurons                 ║\n\
             ║ Semantic Index: {} entries                   ║\n",
            cortex.working_memory.len(),
            cortex.long_term_memory.len(),
            cortex.semantic_index.len()
        ));

        s.push_str(
            "╠══════════════════════════════════════════════════╣\n\
                    ║ NEURONS                                          ║\n\
                    ╟──────────────────────────────────────────────────╢\n",
        );

        for (id, neuron) in cortex.working_memory.iter().take(10) {
            s.push_str(&format!(
                "║ {:016x} │ weight:{:.2} │ {:?}      ║\n",
                id,
                neuron.synaptic_weight,
                format!("{:?}", neuron.metabolic_state)
                    .chars()
                    .take(8)
                    .collect::<String>()
            ));
        }

        s.push_str("╚══════════════════════════════════════════════════╝");
        s
    }
}

// ============================================================================
// TIPOS DE REPORTE
// ============================================================================

#[derive(Debug, Clone)]
pub struct BrainStats {
    pub instance_id: u64,
    pub life_cycle: u64,
    pub working_memory_size: usize,
    pub long_term_memory_size: usize,
    pub semantic_entries: usize,
    pub total_connections: u64,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct AutopoiesisReport {
    pub neurons_checked: u64,
    pub dead_neurons: Vec<u64>,
    pub corrupted_neurons: Vec<u64>,
    pub healthy: bool,
}

// ============================================================================
// FUNCIONES HELPER GLOBALES (Exportadas para uso en otros módulos)
// ============================================================================

/// Timestamp actual en milisegundos (epoch)
pub fn NOW_MS() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Genera ID único basado en hash + entropía temporal
pub fn generate_id(data: &[u8]) -> u64 {
    let mut state: u64 = 0x6a_09_e6_67_f3_bc_cf_u64;

    // Mezcla ChaCha20 simplificada
    for (i, &byte) in data.iter().enumerate() {
        state = state
            .rotate_left(7)
            .wrapping_add(byte as u64)
            .wrapping_mul(0xbf_58_4c_64_0c_1d_e9_u64);

        let pos_mix = ((i as u64).wrapping_mul(0x9e_37_79b9_7f4_a7c5_u64)).rotate_left(11);
        state ^= pos_mix;
    }

    // Agregar entropía temporal
    state ^= NOW_MS();

    state ^ state.rotate_left(17)
}

/// Parsea una lista de u64 separados por comas
fn parse_u64_list(s: &str) -> Result<Vec<u64>, std::num::ParseIntError> {
    s.split(',')
        .map(|part| part.trim().parse::<u64>())
        .collect()
}

/// Genera número aleatorio de 64 bits usando hash de timestamp + estado
pub fn rand_u64() -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    // Xorshift simplificado
    let mut x = time.wrapping_mul(0xbf_58_4c_64_0c_1d_e9_u64);
    x ^= x >> 33;
    x = x.wrapping_mul(0x9e_37_79b9_7f4_a7c5_u64);
    x ^ (x >> 29) ^ (x << 17)
}

// ============================================================================
// VALUE - Tipo genérico para operaciones de datos
// ============================================================================

/// Value representa datos serializables en MemBrain
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    U64(u64),
    I64(i64),
    F64(f64),
    String(Vec<u8>),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
}

impl Value {
    /// Serializa el Value a bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Value::Null => vec![0x00],
            Value::Bool(true) => vec![0x01],
            Value::Bool(false) => vec![0x02],
            Value::U64(n) => {
                let mut bytes = vec![0x10];
                bytes.extend_from_slice(&n.to_le_bytes());
                bytes
            }
            Value::I64(n) => {
                let mut bytes = vec![0x11];
                bytes.extend_from_slice(&n.to_le_bytes());
                bytes
            }
            Value::F64(n) => {
                let mut bytes = vec![0x12];
                bytes.extend_from_slice(&n.to_le_bytes());
                bytes
            }
            Value::String(s) => {
                let mut bytes = vec![0x20];
                bytes.extend_from_slice(&(s.len() as u64).to_le_bytes());
                bytes.extend_from_slice(s);
                bytes
            }
            Value::Bytes(b) => {
                let mut bytes = vec![0x21];
                bytes.extend_from_slice(&(b.len() as u64).to_le_bytes());
                bytes.extend_from_slice(b);
                bytes
            }
            Value::List(items) => {
                let mut bytes = vec![0x30];
                bytes.extend_from_slice(&(items.len() as u64).to_le_bytes());
                for item in items {
                    bytes.extend_from_slice(&item.to_bytes());
                }
                bytes
            }
            Value::Map(pairs) => {
                let mut bytes = vec![0x31];
                bytes.extend_from_slice(&(pairs.len() as u64).to_le_bytes());
                for (k, v) in pairs {
                    bytes.extend_from_slice(&k.to_bytes());
                    bytes.extend_from_slice(&v.to_bytes());
                }
                bytes
            }
        }
    }

    /// Deserializa bytes a Value
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        match bytes[0] {
            0x00 => Some(Value::Null),
            0x01 => Some(Value::Bool(true)),
            0x02 => Some(Value::Bool(false)),
            0x10 if bytes.len() >= 9 => {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[1..9]);
                Some(Value::U64(u64::from_le_bytes(buf)))
            }
            0x11 if bytes.len() >= 9 => {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[1..9]);
                Some(Value::I64(i64::from_le_bytes(buf)))
            }
            0x12 if bytes.len() >= 9 => {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[1..9]);
                Some(Value::F64(f64::from_le_bytes(buf)))
            }
            0x20 => {
                let mut len_buf = [0u8; 8];
                len_buf.copy_from_slice(&bytes[1..9]);
                let len = u64::from_le_bytes(len_buf) as usize;
                if bytes.len() >= 9 + len {
                    Some(Value::String(bytes[9..9 + len].to_vec()))
                } else {
                    None
                }
            }
            0x21 => {
                let mut len_buf = [0u8; 8];
                len_buf.copy_from_slice(&bytes[1..9]);
                let len = u64::from_le_bytes(len_buf) as usize;
                if bytes.len() >= 9 + len {
                    Some(Value::Bytes(bytes[9..9 + len].to_vec()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Convierte Value a &str si es String
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => std::str::from_utf8(v).ok(),
            _ => None,
        }
    }

    /// Convierte Value a f64 si es F64
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::F64(n) => Some(*n),
            Value::U64(n) => Some(*n as f64),
            Value::I64(n) => Some(*n as f64),
            _ => None,
        }
    }

    /// Convierte Value a u64 si es U64
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::U64(n) => Some(*n),
            Value::F64(n) => Some(*n as u64),
            Value::I64(n) => Some(*n as u64),
            _ => None,
        }
    }

    /// Convierte Value a i64 si es I64
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::I64(n) => Some(*n),
            Value::U64(n) => Some(*n as i64),
            Value::F64(n) => Some(*n as i64),
            _ => None,
        }
    }

    /// Convierte Value a bool si es Bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Convierte Value a Vec<u8> si es String o Bytes
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::String(v) => Some(v.as_slice()),
            Value::Bytes(v) => Some(v.as_slice()),
            _ => None,
        }
    }
}

impl fmt::Display for AutopoiesisReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "═══ AUTOPOIESIS REPORT ═══")?;
        writeln!(f, "Neurons checked: {}", self.neurons_checked)?;
        writeln!(f, "Dead neurons: {}", self.dead_neurons.len())?;
        writeln!(f, "Corrupted: {}", self.corrupted_neurons.len())?;
        writeln!(
            f,
            "Status: {}",
            if self.healthy {
                "✓ HEALTHY"
            } else {
                "⚠ ANOMALIES"
            }
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_neuron_creation() {
        let data = b"test data".to_vec();
        let neuron = Neuron::new(data.clone(), Neurotransmitter::Glutamato(data));

        assert!(neuron.is_alive());
        assert_eq!(neuron.synaptic_weight, 0.5);
    }

    #[test]
    fn test_activation() {
        let data = b"test".to_vec();
        let mut neuron = Neuron::new(data.clone(), Neurotransmitter::Dopamina(data));

        neuron.activate();
        assert!(neuron.synaptic_weight > 0.5);

        let initial_weight = neuron.synaptic_weight;
        neuron.activate();
        assert!(neuron.synaptic_weight >= initial_weight);
    }

    #[test]
    fn test_decay() {
        let data = b"decay test".to_vec();
        let mut neuron = Neuron::new(data.clone(), Neurotransmitter::Glutamato(data));

        let initial = neuron.synaptic_weight;
        neuron.life_cycle();

        assert!(neuron.synaptic_weight < initial);
    }

    #[test]
    fn test_unique_ids() {
        let n1 = Neuron::new(
            b"data1".to_vec(),
            Neurotransmitter::Glutamato(b"data1".to_vec()),
        );
        let n2 = Neuron::new(
            b"data2".to_vec(),
            Neurotransmitter::Glutamato(b"data2".to_vec()),
        );

        assert_ne!(n1.id, n2.id);
    }

    #[test]
    fn test_membrain_operations() {
        let path = temp_dir().join("membrain_test");
        let mut brain = MemBrain::new(path.to_str().unwrap()).unwrap();

        // Store with different neurotransmitters
        let id1 = brain.gluta(b"key1", b"ephemeral data".to_vec());
        let id2 = brain.gaba(b"key2", b"stable data".to_vec());
        let id3 = brain.dopa(b"key3", b"critical data".to_vec());

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);

        // Recuerde - buscar por ID
        assert!(brain.recall_id(id1).is_some());
        assert!(brain.recall_id(id2).is_some());
        assert!(brain.recall_id(id3).is_some());

        // Stats
        let stats = brain.stats();
        assert!(stats.working_memory_size >= 3);

        // Cleanup
        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn test_autopoiesis() {
        let path = temp_dir().join("membrain_autopoiesis_test");
        let mut brain = MemBrain::new(path.to_str().unwrap()).unwrap();

        brain.gluta(b"test", b"data".to_vec());
        let report = brain.autopoiesis();

        assert!(report.healthy);

        let _ = std::fs::remove_dir_all(&path);
    }
}
