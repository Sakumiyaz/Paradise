//! Registry: Persistencia en memoria de patrones Genesis
//!
//! Sistema de registro y búsqueda de patrones sin dependencias externas.
#![allow(dead_code)]
#![allow(non_snake_case)]

use super::{GenesisPattern, GenesisType, GENESIS_VERSION};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Índice de patrones para búsqueda rápida
#[derive(Debug, Clone, Default)]
pub struct GenesisIndex {
    /// Mapa de ID -> posición en registro
    by_id: HashMap<[u8; 32], usize>,

    /// Índice por tipo de patrón
    by_type: HashMap<u8, Vec<[u8; 32]>>,

    /// Índice por timestamp
    by_timestamp: Vec<(u64, [u8; 32])>,

    /// Patrones por padre
    by_parent: HashMap<[u8; 32], Vec<[u8; 32]>>,
}

impl GenesisIndex {
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            by_type: HashMap::new(),
            by_timestamp: Vec::new(),
            by_parent: HashMap::new(),
        }
    }

    /// Registra un patrón en el índice
    pub fn insert(&mut self, pattern: &GenesisPattern) {
        let id = pattern.id();
        let pos = self.by_id.len();

        // Por ID
        self.by_id.insert(id, pos);

        // Por tipo
        let type_key = pattern.header.pattern_type;
        self.by_type.entry(type_key).or_default().push(id);

        // Por timestamp
        self.by_timestamp.push((pattern.created_at(), id));

        // Por padre
        let parent = pattern.parent_id();
        if parent != [0u8; 32] {
            self.by_parent.entry(parent).or_default().push(id);
        }
    }

    /// Busca por ID exacto
    pub fn get_by_id(&self, id: &[u8; 32]) -> Option<usize> {
        self.by_id.get(id).copied()
    }

    /// Obtiene todos los IDs de un tipo
    pub fn get_by_type(&self, pattern_type: u8) -> Vec<[u8; 32]> {
        self.by_type.get(&pattern_type).cloned().unwrap_or_default()
    }

    /// Obtiene hijos de un patrón
    pub fn get_children(&self, parent_id: &[u8; 32]) -> Vec<[u8; 32]> {
        self.by_parent.get(parent_id).cloned().unwrap_or_default()
    }

    /// Busca patrones en rango de tiempo
    pub fn get_by_time_range(&self, start: u64, end: u64) -> Vec<[u8; 32]> {
        self.by_timestamp
            .iter()
            .filter(|(ts, _)| *ts >= start && *ts <= end)
            .map(|(_, id)| *id)
            .collect()
    }

    /// Cuenta patrones por tipo
    pub fn count_by_type(&self) -> HashMap<u8, usize> {
        self.by_type.iter().map(|(&k, v)| (k, v.len())).collect()
    }

    /// Total de patrones
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// Registro de patrones Genesis con persistencia en memoria
#[derive(Debug, Clone)]
pub struct GenesisRegistry {
    /// Nombre del registry
    name: String,

    /// Patrones almacenados
    patterns: Vec<GenesisPattern>,

    /// Índice para búsqueda
    index: GenesisIndex,

    /// Metadata del registry
    created_at: u64,
    version: u8,
    total_entropy: u64,
}

impl GenesisRegistry {
    // ========================================================================
    // Creación
    // ========================================================================

    /// Crea un nuevo registry
    pub fn new(name: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            name: name.to_string(),
            patterns: Vec::new(),
            index: GenesisIndex::new(),
            created_at: now,
            version: GENESIS_VERSION,
            total_entropy: 0,
        }
    }

    /// Crea un registry desde datos serializados
    pub fn from_bytes(data: &[u8]) -> Result<Self, RegistryError> {
        if data.len() < 128 {
            return Err(RegistryError::InvalidData("Datos insuficientes".into()));
        }

        let mut offset = 0;

        // Magic del registry
        let magic = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        if magic != 0x5245474953545259u64 {
            // "REGISTRY"
            return Err(RegistryError::InvalidMagic(magic));
        }
        offset += 8;

        // Version
        let version = data[offset];
        if version != GENESIS_VERSION {
            return Err(RegistryError::InvalidVersion(version));
        }
        offset += 1;

        // Nombre (64 bytes)
        let name_len = data[offset] as usize;
        offset += 1;
        let name = String::from_utf8(data[offset..offset + name_len].to_vec())
            .map_err(|_| RegistryError::InvalidData("Nombre inválido".into()))?;
        // Name field is at offset 10, fixed size of 64 bytes
        offset = 10 + 64;

        // Timestamps
        let created_at = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let total_entropy = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;

        // Count
        let count = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8;

        let mut registry = Self {
            name,
            patterns: Vec::with_capacity(count),
            index: GenesisIndex::new(),
            created_at,
            version,
            total_entropy,
        };

        // Cargar patrones
        for _ in 0..count {
            let pattern_len =
                u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;

            let pattern = GenesisPattern::from_bytes(&data[offset..offset + pattern_len])
                .map_err(|_| RegistryError::InvalidPattern)?;
            offset += pattern_len;

            registry.patterns.push(pattern.clone());
            registry.index.insert(&pattern);
        }

        Ok(registry)
    }

    // ========================================================================
    // Operaciones
    // ========================================================================

    /// Registra un nuevo patrón
    pub fn register(&mut self, pattern: GenesisPattern) -> Result<[u8; 32], RegistryError> {
        let id = pattern.id();

        // Verificar si ya existe
        if self.index.get_by_id(&id).is_some() {
            return Err(RegistryError::DuplicatePattern(id));
        }

        // Agregar
        let pos = self.patterns.len();
        self.patterns.push(pattern);
        self.index.insert(&self.patterns[pos]);
        self.total_entropy += self.patterns[pos].payload.len() as u64;

        Ok(id)
    }

    /// Obtiene un patrón por ID
    pub fn get(&self, id: &[u8; 32]) -> Option<&GenesisPattern> {
        self.index
            .get_by_id(id)
            .and_then(|pos| self.patterns.get(pos))
    }

    /// Obtiene todos los patrones de un tipo
    pub fn get_by_type(&self, pattern_type: GenesisType) -> Vec<&GenesisPattern> {
        self.index
            .get_by_type(pattern_type as u8)
            .iter()
            .filter_map(|id| self.get(id))
            .collect()
    }

    /// Obtiene patrones hijos de uno dado
    pub fn get_children(&self, parent_id: &[u8; 32]) -> Vec<&GenesisPattern> {
        self.index
            .get_children(parent_id)
            .iter()
            .filter_map(|id| self.get(id))
            .collect()
    }

    /// Obtiene todos los patrones Primordial
    pub fn primordial_roots(&self) -> Vec<&GenesisPattern> {
        self.get_by_type(GenesisType::Primordial)
    }

    /// Busca en rango de tiempo
    pub fn query_by_time(&self, start: u64, end: u64) -> Vec<&GenesisPattern> {
        self.index
            .get_by_time_range(start, end)
            .iter()
            .filter_map(|id| self.get(id))
            .collect()
    }

    /// Serializa el registry a bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Header del registry (variable)
        let mut header = Vec::new();

        // Magic "REGISTRY"
        header.extend_from_slice(&0x5245474953545259u64.to_le_bytes());
        header.push(GENESIS_VERSION);
        header.push(self.name.len() as u8);
        header.resize(header.len() + 64, 0);
        // Name field starts at offset 10 (after magic=8, version=1, name_len=1)
        header[10..10 + self.name.len()].copy_from_slice(self.name.as_bytes());
        header.extend_from_slice(&self.created_at.to_le_bytes());
        header.extend_from_slice(&self.total_entropy.to_le_bytes());
        header.extend_from_slice(&(self.patterns.len() as u64).to_le_bytes());

        data.extend_from_slice(&header);

        // Patrones
        for pattern in &self.patterns {
            let pattern_bytes = pattern.to_bytes();
            data.extend_from_slice(&(pattern_bytes.len() as u32).to_le_bytes());
            data.extend_from_slice(&pattern_bytes);
        }

        data
    }

    // ========================================================================
    // Información
    // ========================================================================

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    pub fn total_entropy(&self) -> u64 {
        self.total_entropy
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            total_patterns: self.patterns.len(),
            by_type: self.index.count_by_type(),
            total_entropy: self.total_entropy,
            memory_usage: self.to_bytes().len(),
        }
    }

    /// Itera sobre todos los patrones
    pub fn iter(&self) -> impl Iterator<Item = &GenesisPattern> {
        self.patterns.iter()
    }
}

/// Estadísticas del registry
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_patterns: usize,
    pub by_type: HashMap<u8, usize>,
    pub total_entropy: u64,
    pub memory_usage: usize,
}

// ========================================================================
// Errores
// ========================================================================

#[derive(Debug, Clone)]
pub enum RegistryError {
    InvalidData(String),
    InvalidMagic(u64),
    InvalidVersion(u8),
    InvalidPattern,
    DuplicatePattern([u8; 32]),
    NotFound,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidData(msg) => write!(f, "Datos inválidos: {}", msg),
            Self::InvalidMagic(m) => write!(f, "Magic inválido: {:#x}", m),
            Self::InvalidVersion(v) => write!(f, "Versión no soportada: {}", v),
            Self::InvalidPattern => write!(f, "Patrón inválido"),
            Self::DuplicatePattern(id) => write!(f, "Patrón duplicado: {:02x?}", &id[..8]),
            Self::NotFound => write!(f, "Patrón no encontrado"),
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = GenesisRegistry::new("test_registry");
        assert_eq!(registry.name(), "test_registry");
        assert!(registry.is_empty());
    }

    #[test]
    fn test_register_pattern() {
        use crate::genesis::{EntropySource, GenesisPattern};

        let mut registry = GenesisRegistry::new("test");

        let pattern =
            GenesisPattern::primordial(b"origin".to_vec(), &EntropySource::System).unwrap();
        let id = registry.register(pattern).unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.get(&id).is_some());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut registry = GenesisRegistry::new("persistence_test");

        use crate::genesis::{EntropySource, GenesisPattern};

        for i in 0..5 {
            let pattern = GenesisPattern::primordial(
                format!("pattern_{}", i).into_bytes(),
                &EntropySource::System,
            )
            .unwrap();
            registry.register(pattern).unwrap();
        }

        let bytes = registry.to_bytes();
        let restored = GenesisRegistry::from_bytes(&bytes).unwrap();

        assert_eq!(registry.len(), restored.len());
    }
}
