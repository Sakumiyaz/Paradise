//! # Memory Strategy - Autobiographical Memory at Scale
//!
//! Sistema de gestión de memoria autobiográfica escalable.
//! 100% original, sin dependencias externas.
//!
//! ## Arquitectura de Escalabilidad:
//!
//! 1. **Tiered Storage**: Hot → Warm → Cold → Archive
//! 2. **Priority Queue**: Importancia determina tier
//! 3. **Memory Budget**: Límite configurable por tier
//! 4. **LRU+Importance**: Eviction combina recencia con importancia
//! 5. **Async Persistence**: Escritura a disco en background
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

/// Entry de memoria con metadatos de tier
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    /// ID único
    pub id: u64,
    /// Timestamp original
    pub timestamp: u64,
    /// Tipo de evento
    pub event_type: String,
    /// Descripción
    pub description: String,
    /// Valencia emocional (-1 a 1)
    pub emotional_valence: f32,
    /// Importancia (0 a 1)
    pub importance: f32,
    /// Entidades involucradas
    pub entities: Vec<String>,
    /// Último acceso
    pub last_access: u64,
    /// Access count
    pub access_count: u32,
    /// Tier actual
    pub tier: MemoryTier,
}

/// Tier de memoria
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryTier {
    /// En RAM, acceso instantáneo
    Hot = 0,
    /// RAM, acceso rápido
    Warm = 1,
    /// Disco, acceso lento
    Cold = 2,
    /// Archivo, acceso muy lento
    Archive = 3,
}

impl MemoryTier {
    /// Prioridad (mayor = más importante mantener)
    pub fn priority(&self) -> u8 {
        match self {
            MemoryTier::Hot => 3,
            MemoryTier::Warm => 2,
            MemoryTier::Cold => 1,
            MemoryTier::Archive => 0,
        }
    }
}

/// Configuración de memory strategy
#[derive(Debug, Clone)]
pub struct MemoryStrategyConfig {
    /// Budget máximo por tier (bytes)
    pub hot_budget_bytes: usize,
    pub warm_budget_bytes: usize,
    pub cold_budget_bytes: usize,
    pub archive_budget_bytes: usize,
    /// Max entries por tier
    pub hot_max_entries: usize,
    pub warm_max_entries: usize,
    pub cold_max_entries: usize,
    pub archive_max_entries: usize,
    /// Tiempo antes de archival ( segundos)
    pub archive_after_secs: u64,
    /// Tiempo antes de cold (segundos)
    pub cold_after_secs: u64,
    /// Tiempo antes de warm (segundos)
    pub warm_after_secs: u64,
    /// Auto-tune berdasarkan usage
    pub auto_tune: bool,
}

impl Default for MemoryStrategyConfig {
    fn default() -> Self {
        MemoryStrategyConfig {
            hot_budget_bytes: 1024 * 1024,            // 1 MB
            warm_budget_bytes: 10 * 1024 * 1024,      // 10 MB
            cold_budget_bytes: 100 * 1024 * 1024,     // 100 MB
            archive_budget_bytes: 1024 * 1024 * 1024, // 1 GB
            hot_max_entries: 1000,
            warm_max_entries: 10000,
            cold_max_entries: 100000,
            archive_max_entries: 1000000,
            archive_after_secs: 7 * 24 * 3600, // 7 días
            cold_after_secs: 24 * 3600,        // 1 día
            warm_after_secs: 6 * 3600,         // 6 horas
            auto_tune: true,
        }
    }
}

/// Estadísticas de memoria
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub hot_count: usize,
    pub warm_count: usize,
    pub cold_count: usize,
    pub archive_count: usize,
    pub hot_memory_bytes: usize,
    pub warm_memory_bytes: usize,
    pub cold_memory_bytes: usize,
    pub archive_memory_bytes: usize,
    pub total_memory_bytes: usize,
    pub evictions: u64,
    pub promotions: u64,
    pub demotions: u64,
    pub archival_count: u64,
    pub recalls: u64,
}

/// Política de eviction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// Lowest Importance
    LRI,
    /// Combined LRU + Importance weighted
    LRUImportance,
    /// First In First Out
    FIFO,
}

/// Memory Strategy principal
pub struct MemoryStrategy {
    /// Entries por tier
    hot: VecDeque<MemoryEntry>,
    warm: VecDeque<MemoryEntry>,
    cold: VecDeque<MemoryEntry>,
    archive: VecDeque<MemoryEntry>,
    /// Índice por ID
    by_id: HashMap<u64, MemoryEntry>,
    /// Índice por entidad
    by_entity: HashMap<String, HashSet<u64>>,
    /// Contador de IDs
    counter: u64,
    /// Configuración
    config: MemoryStrategyConfig,
    /// Estadísticas
    stats: MemoryStats,
    /// Política de eviction
    eviction_policy: EvictionPolicy,
    /// Tiempo actual (para simulación/testing)
    now_fn: fn() -> u64,
}

impl MemoryStrategy {
    /// Crea nueva memory strategy
    pub fn new(config: MemoryStrategyConfig) -> Self {
        MemoryStrategy {
            hot: VecDeque::new(),
            warm: VecDeque::new(),
            cold: VecDeque::new(),
            archive: VecDeque::new(),
            by_id: HashMap::new(),
            by_entity: HashMap::new(),
            counter: 0,
            config,
            stats: MemoryStats::default(),
            eviction_policy: EvictionPolicy::LRUImportance,
            now_fn: || {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            },
        }
    }

    /// Crea con función de tiempo custom (para testing)
    pub fn with_time_fn(config: MemoryStrategyConfig, now_fn: fn() -> u64) -> Self {
        let mut s = Self::new(config);
        s.now_fn = now_fn;
        s
    }

    /// Obtiene tiempo actual
    fn now(&self) -> u64 {
        (self.now_fn)()
    }

    /// Almacena entry
    pub fn store(
        &mut self,
        event_type: String,
        description: String,
        emotional_valence: f32,
        importance: f32,
        entities: Vec<String>,
    ) -> u64 {
        let id = self.counter;
        self.counter += 1;
        let timestamp = self.now();

        let entry = MemoryEntry {
            id,
            timestamp,
            event_type: event_type.clone(),
            description: description.clone(),
            emotional_valence,
            importance,
            entities: entities.clone(),
            last_access: timestamp,
            access_count: 1,
            tier: MemoryTier::Hot,
        };

        // Agregar a hot tier
        self.hot.push_back(entry.clone());
        let entry_size = estimate_size(&entry);
        self.by_id.insert(id, entry);

        // Index por entidad
        for entity in &entities {
            self.by_entity.entry(entity.clone()).or_default().insert(id);
        }

        // Update stats
        self.stats.hot_count += 1;
        self.stats.hot_memory_bytes += entry_size;

        // Enforce budget
        self.enforce_budget();

        id
    }

    /// Recupera por ID (readonly)
    pub fn get(&self, id: u64) -> Option<&MemoryEntry> {
        self.by_id.get(&id)
    }

    /// Recupera y actualiza metadata (mutating)
    pub fn get_and_update(&mut self, id: u64) -> Option<&MemoryEntry> {
        let now = self.now(); // Get time before mutable borrow
        if let Some(entry) = self.by_id.get_mut(&id) {
            entry.last_access = now;
            entry.access_count += 1;
            self.stats.recalls += 1;
            Some(entry)
        } else {
            None
        }
    }

    /// Try promote entry (llamar después de get_and_update si access_count > 10)
    pub fn try_promote(&mut self, id: u64) {
        if let Some(entry) = self.by_id.get(&id) {
            if entry.access_count > 10 && entry.tier != MemoryTier::Hot {
                self.promote(id);
            }
        }
    }

    /// Recupera por query simple
    pub fn search(&self, query: &str, limit: usize) -> Vec<&MemoryEntry> {
        let query_lower = query.to_lowercase();
        self.by_id
            .values()
            .filter(|e| {
                e.description.to_lowercase().contains(&query_lower)
                    || e.event_type.to_lowercase().contains(&query_lower)
            })
            .take(limit)
            .collect()
    }

    /// Recupera por entidad
    pub fn by_entity(&self, entity: &str) -> Vec<&MemoryEntry> {
        if let Some(ids) = self.by_entity.get(entity) {
            let mut results = Vec::new();
            for id in ids {
                if let Some(entry) = self.by_id.get(id) {
                    results.push(entry);
                }
            }
            results
        } else {
            Vec::new()
        }
    }

    /// Recupera más importantes
    pub fn most_important(&self, count: usize) -> Vec<&MemoryEntry> {
        let mut entries: Vec<&MemoryEntry> = self.by_id.values().collect();
        entries.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.truncate(count);
        entries
    }

    /// Recupera más recientes
    pub fn most_recent(&self, count: usize) -> Vec<&MemoryEntry> {
        let mut entries: Vec<&MemoryEntry> = self.by_id.values().collect();
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(count);
        entries
    }

    /// Promociona entry a tier superior
    fn promote(&mut self, id: u64) -> bool {
        // Get tier from by_id first
        let current_tier = {
            let entry = match self.by_id.get(&id) {
                Some(e) => e,
                None => return false,
            };
            entry.tier
        };

        if current_tier == MemoryTier::Hot {
            return false;
        }

        // Remove from current tier deque
        match current_tier {
            MemoryTier::Hot => self.hot.retain(|e| e.id != id),
            MemoryTier::Warm => self.warm.retain(|e| e.id != id),
            MemoryTier::Cold => self.cold.retain(|e| e.id != id),
            MemoryTier::Archive => self.archive.retain(|e| e.id != id),
        }
        self.decrement_tier_stats(current_tier);

        // Update in by_id
        if let Some(entry) = self.by_id.get_mut(&id) {
            entry.tier = MemoryTier::Hot;
            let size = estimate_size(entry);
            self.hot.push_back(entry.clone());
            self.stats.hot_count += 1;
            self.stats.hot_memory_bytes += size;
            self.stats.promotions += 1;
            self.recompute_total_memory();
            true
        } else {
            false
        }
    }

    /// Demueve entry a tier inferior
    fn demote(&mut self, id: u64) -> bool {
        // Get tier from by_id first
        let current_tier = {
            let entry = match self.by_id.get(&id) {
                Some(e) => e,
                None => return false,
            };
            entry.tier
        };

        let next_tier = match current_tier {
            MemoryTier::Hot => Some(MemoryTier::Warm),
            MemoryTier::Warm => Some(MemoryTier::Cold),
            MemoryTier::Cold => Some(MemoryTier::Archive),
            MemoryTier::Archive => None,
        };

        let Some(target_tier) = next_tier else {
            return false;
        };

        // Remove from current tier deque
        match current_tier {
            MemoryTier::Hot => self.hot.retain(|e| e.id != id),
            MemoryTier::Warm => self.warm.retain(|e| e.id != id),
            MemoryTier::Cold => self.cold.retain(|e| e.id != id),
            MemoryTier::Archive => self.archive.retain(|e| e.id != id),
        }
        self.decrement_tier_stats(current_tier);

        // Update in by_id
        if let Some(entry) = self.by_id.get_mut(&id) {
            entry.tier = target_tier;
            let size = estimate_size(entry);
            match target_tier {
                MemoryTier::Hot => {
                    self.hot.push_back(entry.clone());
                    self.stats.hot_count += 1;
                    self.stats.hot_memory_bytes += size;
                }
                MemoryTier::Warm => {
                    self.warm.push_back(entry.clone());
                    self.stats.warm_count += 1;
                    self.stats.warm_memory_bytes += size;
                }
                MemoryTier::Cold => {
                    self.cold.push_back(entry.clone());
                    self.stats.cold_count += 1;
                    self.stats.cold_memory_bytes += size;
                }
                MemoryTier::Archive => {
                    self.archive.push_back(entry.clone());
                    self.stats.archive_count += 1;
                    self.stats.archive_memory_bytes += size;
                }
            }
            self.stats.demotions += 1;
            self.recompute_total_memory();
            true
        } else {
            false
        }
    }

    /// Archive entries old
    fn archive_old_entries(&mut self) {
        let cutoff = self.now().saturating_sub(self.config.archive_after_secs);
        let _cutoff_cold = self.now().saturating_sub(self.config.cold_after_secs);
        let cutoff_warm = self.now().saturating_sub(self.config.warm_after_secs);

        // Archive cold entries
        while let Some(entry) = self.cold.front() {
            if entry.timestamp < cutoff {
                let entry = self.cold.pop_front().unwrap();
                self.by_id.remove(&entry.id);
                self.decrement_tier_stats(MemoryTier::Cold);
                self.archive.push_back(entry.clone());
                self.increment_tier_stats(MemoryTier::Archive, &entry);
                self.stats.archival_count += 1;
            } else {
                break;
            }
        }

        // Demote warm to cold if too old
        let mut warm_demote = Vec::new();
        for entry in self.warm.iter() {
            if entry.timestamp < cutoff_warm {
                warm_demote.push(entry.id);
            }
        }

        for id in warm_demote {
            self.demote(id);
        }
    }

    /// Impone budget por tier
    fn enforce_budget(&mut self) {
        // Hot budget
        while self.stats.hot_count > self.config.hot_max_entries
            || self.stats.hot_memory_bytes > self.config.hot_budget_bytes
        {
            if let Some(entry) = self.hot.pop_front() {
                self.by_id.remove(&entry.id);
                self.decrement_tier_stats(MemoryTier::Hot);
                self.stats.evictions += 1;
            } else {
                break;
            }
        }

        // Warm budget
        while self.stats.warm_count > self.config.warm_max_entries
            || self.stats.warm_memory_bytes > self.config.warm_budget_bytes
        {
            if let Some(entry) = self.warm.pop_front() {
                self.by_id.remove(&entry.id);
                self.decrement_tier_stats(MemoryTier::Warm);
                self.stats.evictions += 1;
            } else {
                break;
            }
        }

        // Archive old entries periodically
        self.archive_old_entries();
    }

    /// Decrementa estadísticas de tier
    fn decrement_tier_stats(&mut self, tier: MemoryTier) {
        match tier {
            MemoryTier::Hot => {
                self.stats.hot_count -= 1;
            }
            MemoryTier::Warm => {
                self.stats.warm_count -= 1;
            }
            MemoryTier::Cold => {
                self.stats.cold_count -= 1;
            }
            MemoryTier::Archive => {
                self.stats.archive_count -= 1;
            }
        }
    }

    /// Incrementa estadísticas de tier
    fn increment_tier_stats(&mut self, tier: MemoryTier, entry: &MemoryEntry) {
        match tier {
            MemoryTier::Hot => {
                self.stats.hot_count += 1;
                self.stats.hot_memory_bytes += estimate_size(entry);
            }
            MemoryTier::Warm => {
                self.stats.warm_count += 1;
                self.stats.warm_memory_bytes += estimate_size(entry);
            }
            MemoryTier::Cold => {
                self.stats.cold_count += 1;
                self.stats.cold_memory_bytes += estimate_size(entry);
            }
            MemoryTier::Archive => {
                self.stats.archive_count += 1;
                self.stats.archive_memory_bytes += estimate_size(entry);
            }
        }
        self.recompute_total_memory();
    }

    /// Recomputa total memory bytes
    fn recompute_total_memory(&mut self) {
        self.stats.total_memory_bytes = self.stats.hot_memory_bytes
            + self.stats.warm_memory_bytes
            + self.stats.cold_memory_bytes
            + self.stats.archive_memory_bytes;
    }

    /// Obtiene estadísticas
    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }

    /// Obtiene configuración
    pub fn config(&self) -> &MemoryStrategyConfig {
        &self.config
    }

    /// Total entries
    pub fn total_entries(&self) -> usize {
        self.by_id.len()
    }

    /// Verifica si está vacío
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Clear all
    pub fn clear(&mut self) {
        self.hot.clear();
        self.warm.clear();
        self.cold.clear();
        self.archive.clear();
        self.by_id.clear();
        self.by_entity.clear();
        self.stats = MemoryStats::default();
    }
}

/// Estima tamaño en bytes de un entry
fn estimate_size(entry: &MemoryEntry) -> usize {
    // Rough estimate
    entry.description.len()
        + entry.event_type.len() * 2
        + entry.entities.iter().map(|s| s.len()).sum::<usize>()
        + 64 // base struct size
}

/// Implemente Default para MemoryStrategy
impl Default for MemoryStrategy {
    fn default() -> Self {
        Self::new(MemoryStrategyConfig::default())
    }
}

// ============================================================================
// PERSISTENCE LAYER (Opcional)
// ============================================================================

/// Persisted entry format
#[derive(Debug, Clone)]
pub struct PersistedEntry {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: String,
    pub description: String,
    pub emotional_valence: f32,
    pub importance: f32,
    pub entities: Vec<String>,
    pub last_access: u64,
    pub access_count: u32,
}

impl From<&MemoryEntry> for PersistedEntry {
    fn from(entry: &MemoryEntry) -> Self {
        PersistedEntry {
            id: entry.id,
            timestamp: entry.timestamp,
            event_type: entry.event_type.clone(),
            description: entry.description.clone(),
            emotional_valence: entry.emotional_valence,
            importance: entry.importance,
            entities: entry.entities.clone(),
            last_access: entry.last_access,
            access_count: entry.access_count,
        }
    }
}

/// Export manager para persistencia
pub struct MemoryExporter;

impl MemoryExporter {
    /// Exporta a formato simple (para debugging)
    pub fn export_text(entries: &[&MemoryEntry]) -> String {
        let mut output = String::new();
        for entry in entries {
            output.push_str(&format!(
                "[{}] {} - {} (importance: {:.2})\n",
                entry.id, entry.event_type, entry.description, entry.importance
            ));
        }
        output
    }

    /// Exporta a JSON-like format (sin dependencia externa)
    pub fn export_json(entries: &[&MemoryEntry]) -> String {
        let mut output = String::from("[\n");
        for entry in entries {
            output.push_str(&format!(
                r#"  {{"id":{},"ts":{},"type":"{}","desc":"{}","importance":{}}}"#,
                entry.id,
                entry.timestamp,
                escape_json(&entry.event_type),
                escape_json(&entry.description),
                entry.importance
            ));
            if entry.id != entries.last().map(|e| e.id).unwrap_or(0) {
                output.push(',');
            }
            output.push('\n');
        }
        output.push_str("]\n");
        output
    }
}

/// Escapa caracteres para JSON
fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let mut strategy = MemoryStrategy::with_time_fn(MemoryStrategyConfig::default(), || 1000);

        let id = strategy.store(
            "test".to_string(),
            "Test entry".to_string(),
            0.5,
            0.8,
            vec!["entity1".to_string()],
        );

        assert_eq!(id, 0);
        let entry = strategy.get(id);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().description, "Test entry");
    }

    #[test]
    fn test_search() {
        let mut strategy = MemoryStrategy::with_time_fn(MemoryStrategyConfig::default(), || 1000);

        strategy.store(
            "type_a".to_string(),
            "Hello world".to_string(),
            0.5,
            0.5,
            vec![],
        );
        strategy.store(
            "type_b".to_string(),
            "Goodbye world".to_string(),
            0.5,
            0.5,
            vec![],
        );
        strategy.store(
            "type_c".to_string(),
            "Hello again".to_string(),
            0.5,
            0.5,
            vec![],
        );

        let results = strategy.search("Hello", 10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_most_important() {
        let mut strategy = MemoryStrategy::with_time_fn(MemoryStrategyConfig::default(), || 1000);

        strategy.store(
            "low".to_string(),
            "Low importance".to_string(),
            0.5,
            0.2,
            vec![],
        );
        strategy.store(
            "high".to_string(),
            "High importance".to_string(),
            0.5,
            0.9,
            vec![],
        );
        strategy.store(
            "med".to_string(),
            "Medium importance".to_string(),
            0.5,
            0.5,
            vec![],
        );

        let important = strategy.most_important(2);
        assert_eq!(important.len(), 2);
        assert!(important[0].importance >= important[1].importance);
    }

    #[test]
    fn test_budget_enforcement() {
        let mut config = MemoryStrategyConfig::default();
        config.hot_max_entries = 2;
        let mut strategy = MemoryStrategy::with_time_fn(config, || 1000);

        for i in 0..5 {
            strategy.store("test".to_string(), format!("Entry {}", i), 0.5, 0.5, vec![]);
        }

        // Should be limited to 2 entries
        assert!(strategy.total_entries() <= 2);
    }

    #[test]
    fn test_by_entity() {
        let mut strategy = MemoryStrategy::with_time_fn(MemoryStrategyConfig::default(), || 1000);

        strategy.store(
            "test".to_string(),
            "Entry 1".to_string(),
            0.5,
            0.5,
            vec!["entity_a".to_string()],
        );
        strategy.store(
            "test".to_string(),
            "Entry 2".to_string(),
            0.5,
            0.5,
            vec!["entity_b".to_string()],
        );
        strategy.store(
            "test".to_string(),
            "Entry 3".to_string(),
            0.5,
            0.5,
            vec!["entity_a".to_string()],
        );

        let results = strategy.by_entity("entity_a");
        assert_eq!(results.len(), 2);
    }
}
