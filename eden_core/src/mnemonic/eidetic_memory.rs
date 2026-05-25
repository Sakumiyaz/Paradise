//! # Eidetic Memory Module
//!
//! Real perfect memory system that stores and retrieves information
//! without loss. Implements byte-exact storage with integrity verification.
//!
//! ## Architecture
//!
//! - `EideticMemory`: Core memory engine with perfect recall
//! - `MemoriaPerfecta`: Individual memory unit with byte-exact storage
//! - `IndiceAsociativo`: Real associative indexing with scoring
//! - `EncodingPerfecto`: Real lossless encoding with checksums
//! - `ForgettingConfig`: Optional decay mechanism
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{BTreeMap, HashMap};
use std::hash::{DefaultHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
// ============================================================================
// HASH VERIFICATION
// ============================================================================

/// Computes a 64-bit hash for data integrity verification
fn compute_hash(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(data);
    hasher.finish()
}

/// Computes a 32-bit checksum for quick integrity checks
fn compute_checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    for chunk in data.chunks(4) {
        let mut word: u32 = 0;
        for (i, &byte) in chunk.iter().enumerate() {
            word |= (byte as u32) << (i * 8);
        }
        sum = sum.wrapping_add(word);
    }
    sum
}

/// Verifies data integrity using hash and checksum
fn verify_integrity(data: &[u8], expected_hash: u64, expected_checksum: u32) -> bool {
    compute_hash(data) == expected_hash && compute_checksum(data) == expected_checksum
}

// ============================================================================
// CODEC TRAIT
// ============================================================================

/// Trait for types that can be losslessly encoded/decoded
pub trait LosslessCodec: Clone {
    /// Encodes the value to bytes without any loss
    fn encode(&self) -> Vec<u8>;

    /// Decodes bytes back to the original value
    fn decode(data: &[u8]) -> Result<Self, CodecError>
    where
        Self: Sized;
}

/// Errors that can occur during encoding/decoding
#[derive(Debug, Clone, PartialEq)]
pub enum CodecError {
    InsufficientData { expected: usize, actual: usize },
    InvalidFormat(String),
    ChecksumMismatch { expected: u32, actual: u32 },
    HashMismatch { expected: u64, actual: u64 },
}

// Implement LosslessCodec for common types
impl LosslessCodec for String {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.len());
        bytes.extend((self.len() as u64).to_be_bytes());
        bytes.extend(self.as_bytes());
        bytes
    }

    fn decode(data: &[u8]) -> Result<Self, CodecError> {
        if data.len() < 8 {
            return Err(CodecError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }
        let mut len_bytes = [0u8; 8];
        len_bytes.copy_from_slice(&data[..8]);
        let len = u64::from_be_bytes(len_bytes) as usize;

        if data.len() < 8 + len {
            return Err(CodecError::InsufficientData {
                expected: 8 + len,
                actual: data.len(),
            });
        }

        String::from_utf8(data[8..8 + len].to_vec())
            .map_err(|e| CodecError::InvalidFormat(e.to_string()))
    }
}

impl LosslessCodec for Vec<u8> {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.len());
        bytes.extend((self.len() as u64).to_be_bytes());
        bytes.extend(self);
        bytes
    }

    fn decode(data: &[u8]) -> Result<Self, CodecError> {
        if data.len() < 8 {
            return Err(CodecError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }
        let mut len_bytes = [0u8; 8];
        len_bytes.copy_from_slice(&data[..8]);
        let len = u64::from_be_bytes(len_bytes) as usize;

        if data.len() < 8 + len {
            return Err(CodecError::InsufficientData {
                expected: 8 + len,
                actual: data.len(),
            });
        }

        Ok(data[8..8 + len].to_vec())
    }
}

impl LosslessCodec for u64 {
    fn encode(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn decode(data: &[u8]) -> Result<Self, CodecError> {
        if data.len() != 8 {
            return Err(CodecError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(data);
        Ok(u64::from_be_bytes(bytes))
    }
}

impl LosslessCodec for i64 {
    fn encode(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn decode(data: &[u8]) -> Result<Self, CodecError> {
        if data.len() != 8 {
            return Err(CodecError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(data);
        Ok(i64::from_be_bytes(bytes))
    }
}

impl LosslessCodec for f64 {
    fn encode(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn decode(data: &[u8]) -> Result<Self, CodecError> {
        if data.len() != 8 {
            return Err(CodecError::InsufficientData {
                expected: 8,
                actual: data.len(),
            });
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(data);
        Ok(f64::from_be_bytes(bytes))
    }
}

impl LosslessCodec for u32 {
    fn encode(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn decode(data: &[u8]) -> Result<Self, CodecError> {
        if data.len() != 4 {
            return Err(CodecError::InsufficientData {
                expected: 4,
                actual: data.len(),
            });
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(data);
        Ok(u32::from_be_bytes(bytes))
    }
}

// ============================================================================
// PERFECT ENCODING SYSTEM
// ============================================================================

/// Lossless encoding system with integrity verification
#[derive(Debug, Clone)]
pub struct EncodingPerfecto {
    /// Enable integrity verification
    verify_integrity: bool,
}

impl Default for EncodingPerfecto {
    fn default() -> Self {
        Self::new()
    }
}

impl EncodingPerfecto {
    /// Creates a new encoding system
    pub fn new() -> Self {
        Self {
            verify_integrity: true,
        }
    }

    /// Encodes data with integrity verification
    /// Format: [4 bytes checksum][8 bytes hash][4 bytes length][data]
    pub fn encode<T: LosslessCodec>(&self, value: &T) -> Vec<u8> {
        let data = value.encode();
        let checksum = compute_checksum(&data);
        let hash = compute_hash(&data);

        let mut encoded = Vec::with_capacity(16 + data.len());
        encoded.extend(checksum.to_be_bytes());
        encoded.extend(hash.to_be_bytes());
        encoded.extend((data.len() as u32).to_be_bytes());
        encoded.extend(data);

        encoded
    }

    /// Decodes data and verifies integrity
    pub fn decode<T: LosslessCodec>(&self, encoded: &[u8]) -> Result<T, CodecError> {
        if encoded.len() < 16 {
            return Err(CodecError::InsufficientData {
                expected: 16,
                actual: encoded.len(),
            });
        }

        // Extract components
        let mut checksum_bytes = [0u8; 4];
        checksum_bytes.copy_from_slice(&encoded[..4]);
        let expected_checksum = u32::from_be_bytes(checksum_bytes);

        let mut hash_bytes = [0u8; 8];
        hash_bytes.copy_from_slice(&encoded[4..12]);
        let expected_hash = u64::from_be_bytes(hash_bytes);

        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&encoded[12..16]);
        let data_len = u32::from_be_bytes(len_bytes) as usize;

        if encoded.len() < 16 + data_len {
            return Err(CodecError::InsufficientData {
                expected: 16 + data_len,
                actual: encoded.len(),
            });
        }

        let data = &encoded[16..16 + data_len];

        // Verify integrity if enabled
        if self.verify_integrity {
            let actual_checksum = compute_checksum(data);
            if actual_checksum != expected_checksum {
                return Err(CodecError::ChecksumMismatch {
                    expected: expected_checksum,
                    actual: actual_checksum,
                });
            }

            let actual_hash = compute_hash(data);
            if actual_hash != expected_hash {
                return Err(CodecError::HashMismatch {
                    expected: expected_hash,
                    actual: actual_hash,
                });
            }
        }

        T::decode(data)
    }

    /// Encodes raw bytes without wrapping
    pub fn encode_raw(data: &[u8]) -> Vec<u8> {
        let checksum = compute_checksum(data);
        let hash = compute_hash(data);

        let mut encoded = Vec::with_capacity(12 + data.len());
        encoded.extend(checksum.to_be_bytes());
        encoded.extend(hash.to_be_bytes());
        encoded.extend((data.len() as u32).to_be_bytes());
        encoded.extend(data);

        encoded
    }

    /// Decodes raw bytes and verifies integrity
    pub fn decode_raw(encoded: &[u8]) -> Result<Vec<u8>, CodecError> {
        if encoded.len() < 12 {
            return Err(CodecError::InsufficientData {
                expected: 12,
                actual: encoded.len(),
            });
        }

        let mut checksum_bytes = [0u8; 4];
        checksum_bytes.copy_from_slice(&encoded[..4]);
        let expected_checksum = u32::from_be_bytes(checksum_bytes);

        let mut hash_bytes = [0u8; 8];
        hash_bytes.copy_from_slice(&encoded[4..12]);
        let expected_hash = u64::from_be_bytes(hash_bytes);

        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&encoded[12..16]);
        let data_len = u32::from_be_bytes(len_bytes) as usize;

        if encoded.len() < 16 + data_len {
            return Err(CodecError::InsufficientData {
                expected: 16 + data_len,
                actual: encoded.len(),
            });
        }

        let data = encoded[16..16 + data_len].to_vec();

        // Always verify for raw decoding
        let actual_checksum = compute_checksum(&data);
        if actual_checksum != expected_checksum {
            return Err(CodecError::ChecksumMismatch {
                expected: expected_checksum,
                actual: actual_checksum,
            });
        }

        let actual_hash = compute_hash(&data);
        if actual_hash != expected_hash {
            return Err(CodecError::HashMismatch {
                expected: expected_hash,
                actual: actual_hash,
            });
        }

        Ok(data)
    }
}

// ============================================================================
// ASSOCIATIVE INDEX
// ============================================================================

/// Real associative indexing with relevance scoring
#[derive(Debug, Clone)]
pub struct IndiceAsociativo {
    /// Tag to memory IDs mapping
    tag_index: HashMap<String, Vec<u64>>,
    /// Temporal index: timestamp -> memory IDs
    temporal_index: BTreeMap<u64, Vec<u64>>,
    /// Content hash to memory ID
    hash_index: HashMap<u64, u64>,
    /// Tag frequency for scoring
    tag_frequency: HashMap<String, u32>,
    /// Total searches performed
    searches_performed: u64,
}

impl Default for IndiceAsociativo {
    fn default() -> Self {
        Self::new()
    }
}

impl IndiceAsociativo {
    /// Creates a new associative index
    pub fn new() -> Self {
        Self {
            tag_index: HashMap::new(),
            temporal_index: BTreeMap::new(),
            hash_index: HashMap::new(),
            tag_frequency: HashMap::new(),
            searches_performed: 0,
        }
    }

    /// Indexes a memory entry
    pub fn index(&mut self, memory_id: u64, tags: &[String], timestamp: u64, content_hash: u64) {
        // Index by tags
        for tag in tags {
            let tag_lower = tag.to_lowercase();
            self.tag_index
                .entry(tag_lower.clone())
                .or_insert_with(Vec::new)
                .push(memory_id);

            *self.tag_frequency.entry(tag_lower).or_insert(0) += 1;
        }

        // Index by timestamp
        self.temporal_index
            .entry(timestamp)
            .or_insert_with(Vec::new)
            .push(memory_id);

        // Index by content hash
        self.hash_index.insert(content_hash, memory_id);
    }

    /// Searches by tags with relevance scoring
    /// Returns (memory_id, score) pairs sorted by relevance
    pub fn search_by_tags(&mut self, query_tags: &[&str]) -> Vec<(u64, f32)> {
        self.searches_performed += 1;

        if query_tags.is_empty() {
            return vec![];
        }

        // Count occurrences across query tags
        let mut candidate_scores: HashMap<u64, (u32, f32)> = HashMap::new();

        for query_tag in query_tags {
            let tag_lower = query_tag.to_lowercase();
            if let Some(ids) = self.tag_index.get(&tag_lower) {
                // IDF-like scoring: rare tags weight more
                let total_docs = self.hash_index.len() as f32;
                let doc_freq = ids.len() as f32;
                let idf = (total_docs / doc_freq.max(1.0)).ln() + 1.0;

                for &id in ids {
                    let entry = candidate_scores.entry(id).or_insert((0, 0.0));
                    entry.0 += 1; // Term frequency
                    entry.1 += idf; // IDF weight
                }
            }
        }

        // Convert to scored vector with normalized score
        let query_len = query_tags.len() as f32;
        let mut results: Vec<(u64, f32)> = candidate_scores
            .into_iter()
            .filter(|(_, (tf, _))| *tf == query_len as u32) // AND semantics: all tags must match
            .map(|(id, (tf, idf))| {
                // TF-IDF score
                let tf_norm = tf as f32 / query_len;
                let score = tf_norm * idf;
                (id, score)
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Searches by tags with OR semantics (any tag matches)
    pub fn search_by_tags_or(&mut self, query_tags: &[&str]) -> Vec<u64> {
        self.searches_performed += 1;

        let mut results = Vec::new();
        let mut seen = HashMap::new();

        for query_tag in query_tags {
            let tag_lower = query_tag.to_lowercase();
            if let Some(ids) = self.tag_index.get(&tag_lower) {
                for &id in ids {
                    if !seen.contains_key(&id) {
                        seen.insert(id, true);
                        results.push(id);
                    }
                }
            }
        }

        results
    }

    /// Finds memories by exact timestamp
    pub fn find_at_timestamp(&mut self, timestamp: u64) -> Vec<u64> {
        self.searches_performed += 1;

        self.temporal_index
            .get(&timestamp)
            .cloned()
            .unwrap_or_default()
    }

    /// Finds memories within a time range
    pub fn find_in_range(&mut self, start: u64, end: u64) -> Vec<u64> {
        self.searches_performed += 1;

        let mut results = Vec::new();
        for (_, ids) in self.temporal_index.range(start..=end) {
            results.extend(ids);
        }
        results
    }

    /// Finds memory by exact content hash
    pub fn find_by_hash(&mut self, hash: u64) -> Option<u64> {
        self.searches_performed += 1;
        self.hash_index.get(&hash).copied()
    }

    /// Gets the number of indexed tags
    pub fn tag_count(&self) -> usize {
        self.tag_index.len()
    }

    /// Gets total searches performed
    pub fn search_count(&self) -> u64 {
        self.searches_performed
    }
}

// ============================================================================
// PERFECT MEMORY UNIT
// ============================================================================

/// Individual memory unit with byte-exact storage
#[derive(Debug, Clone)]
pub struct MemoriaPerfecta {
    /// Unique identifier
    id: u64,
    /// Raw encoded data
    data: Vec<u8>,
    /// Associated tags
    tags: Vec<String>,
    /// Creation timestamp
    timestamp: u64,
    /// Original type name
    type_name: String,
    /// Content hash for verification
    content_hash: u64,
    /// Integrity checksum
    checksum: u32,
    /// Memory strength (1.0 = perfect, <1.0 = decayed)
    strength: f32,
    /// Access count
    access_count: u32,
    /// Last access timestamp
    last_access: Option<u64>,
}

impl MemoriaPerfecta {
    /// Creates a new memory unit
    fn new(id: u64, data: Vec<u8>, tags: Vec<String>, type_name: String) -> Self {
        let content_hash = compute_hash(&data);
        let checksum = compute_checksum(&data);

        Self {
            id,
            data,
            tags,
            timestamp: current_timestamp(),
            type_name,
            content_hash,
            checksum,
            strength: 1.0,
            access_count: 0,
            last_access: None,
        }
    }

    /// Verifies data integrity
    pub fn verify(&self) -> bool {
        verify_integrity(&self.data, self.content_hash, self.checksum)
    }

    /// Records an access
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_access = Some(current_timestamp());
        // Optionally reinforce strength
        self.strength = (self.strength + 0.01).min(1.0);
    }

    /// Applies decay if configured
    pub fn apply_decay(&mut self, decay_factor: f32) {
        self.strength = (self.strength * decay_factor).max(0.0);
    }

    /// Gets the raw data (only if verified)
    pub fn get_data(&self) -> &Vec<u8> {
        debug_assert!(self.verify(), "Memory integrity compromised");
        &self.data
    }

    /// Gets the ID
    pub fn get_id(&self) -> u64 {
        self.id
    }

    /// Gets the hash
    pub fn get_hash(&self) -> u64 {
        self.content_hash
    }

    /// Gets the timestamp
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Gets the tags
    pub fn get_tags(&self) -> &[String] {
        &self.tags
    }

    /// Gets the strength
    pub fn get_strength(&self) -> f32 {
        self.strength
    }

    /// Gets the access count
    pub fn get_access_count(&self) -> u32 {
        self.access_count
    }
}

// ============================================================================
// FORGETTING CONFIGURATION
// ============================================================================

/// Configuration for optional memory decay
#[derive(Debug, Clone)]
pub struct ForgettingConfig {
    /// Whether forgetting is enabled
    enabled: bool,
    /// Maximum retention time in seconds
    max_retention_secs: u64,
    /// Decay factor per period (0.0 to 1.0)
    decay_factor: f32,
    /// Decay period in seconds
    decay_period_secs: u64,
    /// Minimum strength threshold
    min_strength_threshold: f32,
}

impl Default for ForgettingConfig {
    fn default() -> Self {
        Self::perfect()
    }
}

impl ForgettingConfig {
    /// Creates a perfect memory configuration (no forgetting)
    pub fn perfect() -> Self {
        Self {
            enabled: false,
            max_retention_secs: u64::MAX,
            decay_factor: 1.0,
            decay_period_secs: u64::MAX,
            min_strength_threshold: 0.0,
        }
    }

    /// Creates a forgetting configuration
    pub fn with_forgetting(
        max_retention_secs: u64,
        decay_factor: f32,
        decay_period_secs: u64,
        min_strength_threshold: f32,
    ) -> Self {
        Self {
            enabled: true,
            max_retention_secs,
            decay_factor,
            decay_period_secs,
            min_strength_threshold,
        }
    }

    /// Checks if forgetting is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// ============================================================================
// EIDETIC MEMORY ENGINE
// ============================================================================

/// Core engine for perfect memory storage and retrieval
#[derive(Debug, Clone)]
pub struct EideticMemory {
    /// Storage for memory units
    storage: Vec<MemoriaPerfecta>,
    /// Associative index
    index: IndiceAsociativo,
    /// Encoding system
    encoder: EncodingPerfecto,
    /// Forgetting configuration
    forgetting: Option<ForgettingConfig>,
    /// Next memory ID
    next_id: u64,
    /// Creation timestamp
    created_at: u64,
    /// Last maintenance timestamp
    last_maintenance: u64,
}

impl Default for EideticMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl EideticMemory {
    /// Creates a new eidetic memory engine
    pub fn new() -> Self {
        Self {
            storage: Vec::with_capacity(100_000),
            index: IndiceAsociativo::new(),
            encoder: EncodingPerfecto::new(),
            forgetting: None,
            next_id: 1,
            created_at: current_timestamp(),
            last_maintenance: current_timestamp(),
        }
    }

    /// Creates a new engine with forgetting enabled
    pub fn with_forgetting(config: ForgettingConfig) -> Self {
        Self {
            storage: Vec::with_capacity(100_000),
            index: IndiceAsociativo::new(),
            encoder: EncodingPerfecto::new(),
            forgetting: Some(config),
            next_id: 1,
            created_at: current_timestamp(),
            last_maintenance: current_timestamp(),
        }
    }

    // ========================================================================
    // STORAGE OPERATIONS
    // ========================================================================

    /// Stores data with perfect encoding
    /// Returns the memory ID
    pub fn store<T: LosslessCodec>(&mut self, value: &T, tags: Vec<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        // Encode with integrity verification
        let encoded = self.encoder.encode(value);

        // Create memory unit
        let type_name = std::any::type_name::<T>().to_string();
        let memory = MemoriaPerfecta::new(id, encoded, tags.clone(), type_name);
        let hash = memory.get_hash();

        // Index the memory
        self.index.index(id, &tags, memory.get_timestamp(), hash);

        // Store
        self.storage.push(memory);

        id
    }

    /// Stores raw bytes
    pub fn store_raw(&mut self, data: &[u8], tags: Vec<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        // Encode raw data with integrity
        let encoded = EncodingPerfecto::encode_raw(data);

        // Create memory unit for raw storage
        let hash = compute_hash(data);
        let checksum = compute_checksum(data);

        let memory = MemoriaPerfecta {
            id,
            data: encoded,
            tags: tags.clone(),
            timestamp: current_timestamp(),
            type_name: "raw_bytes".to_string(),
            content_hash: hash,
            checksum,
            strength: 1.0,
            access_count: 0,
            last_access: None,
        };

        // Index
        self.index.index(id, &tags, memory.get_timestamp(), hash);

        // Store
        self.storage.push(memory);

        id
    }

    // ========================================================================
    // RETRIEVAL OPERATIONS
    // ========================================================================

    /// Retrieves memory by ID and decodes to original type
    pub fn retrieve<T: LosslessCodec>(&mut self, id: u64) -> Option<T> {
        let memory = self.storage.iter_mut().find(|m| m.id == id)?;

        // Record access
        memory.record_access();

        // Decode and return
        self.encoder.decode(&memory.data).ok()
    }

    /// Retrieves raw bytes by ID
    pub fn retrieve_raw(&mut self, id: u64) -> Option<Vec<u8>> {
        let memory = self.storage.iter_mut().find(|m| m.id == id)?;
        memory.record_access();

        EncodingPerfecto::decode_raw(&memory.data).ok()
    }

    /// Gets memory metadata without decoding
    pub fn get_metadata(&self, id: u64) -> Option<MemoryMetadata> {
        self.storage
            .iter()
            .find(|m| m.id == id)
            .map(|m| MemoryMetadata {
                id: m.id,
                tags: m.tags.clone(),
                timestamp: m.timestamp,
                type_name: m.type_name.clone(),
                content_hash: m.content_hash,
                strength: m.strength,
                access_count: m.access_count,
                last_access: m.last_access,
                size_bytes: m.data.len() as u64,
                verified: m.verify(),
            })
    }

    /// Gets memory by exact hash
    pub fn get_by_hash(&self, hash: u64) -> Option<&MemoriaPerfecta> {
        self.storage.iter().find(|m| m.content_hash == hash)
    }

    // ========================================================================
    // SEARCH OPERATIONS
    // ========================================================================

    /// Searches by tags with relevance ranking
    pub fn search(&mut self, tags: &[&str]) -> Vec<SearchResult> {
        let scored = self.index.search_by_tags(tags);
        scored
            .into_iter()
            .map(|(id, score)| {
                let memory = self.storage.iter().find(|m| m.id == id).unwrap();
                SearchResult {
                    id,
                    score,
                    tags: memory.tags.clone(),
                    timestamp: memory.timestamp,
                    strength: memory.strength,
                }
            })
            .collect()
    }

    /// Searches by tags with OR semantics
    pub fn search_or(&mut self, tags: &[&str]) -> Vec<u64> {
        self.index.search_by_tags_or(tags)
    }

    /// Finds memories by timestamp
    pub fn find_at_time(&mut self, timestamp: u64) -> Vec<u64> {
        self.index.find_at_timestamp(timestamp)
    }

    /// Finds memories in time range
    pub fn find_in_range(&mut self, start: u64, end: u64) -> Vec<u64> {
        self.index.find_in_range(start, end)
    }

    /// Finds exact duplicate by content
    pub fn find_duplicate(&mut self, data: &[u8]) -> Option<u64> {
        let hash = compute_hash(data);
        self.index.find_by_hash(hash)
    }

    // ========================================================================
    // MAINTENANCE OPERATIONS
    // ========================================================================

    /// Performs maintenance: decay and cleanup
    pub fn maintain(&mut self) -> MaintenanceResult {
        let before_bytes = self.total_bytes();

        if let Some(ref config) = self.forgetting {
            if !config.enabled {
                return MaintenanceResult::no_op();
            }

            let now = current_timestamp();
            let elapsed = now.saturating_sub(self.last_maintenance);

            if elapsed >= config.decay_period_secs {
                self.last_maintenance = now;

                // Apply decay
                for m in &mut self.storage {
                    m.apply_decay(config.decay_factor);
                }

                // Remove weak memories
                let initial_len = self.storage.len();
                self.storage
                    .retain(|m| m.strength >= config.min_strength_threshold);
                let removed = initial_len - self.storage.len();

                // Clean up index
                self.rebuild_index();

                return MaintenanceResult {
                    memories_decayed: self.storage.len(),
                    memories_removed: removed,
                    bytes_freed: before_bytes.saturating_sub(self.total_bytes()),
                    current_count: self.storage.len(),
                };
            }
        }

        MaintenanceResult::no_op()
    }

    /// Rebuilds the index after removals
    fn rebuild_index(&mut self) {
        self.index = IndiceAsociativo::new();
        for memory in &self.storage {
            self.index.index(
                memory.id,
                &memory.tags,
                memory.timestamp,
                memory.content_hash,
            );
        }
    }

    /// Verifies integrity of all memories
    pub fn verify_all(&self) -> VerificationResult {
        let mut verified = 0;
        let mut failed = Vec::new();

        for memory in &self.storage {
            if memory.verify() {
                verified += 1;
            } else {
                failed.push(memory.id);
            }
        }

        VerificationResult {
            total: self.storage.len(),
            verified,
            failed,
        }
    }

    /// Deletes a memory by ID
    pub fn delete(&mut self, id: u64) -> bool {
        let initial_len = self.storage.len();
        self.storage.retain(|m| m.id != id);

        if self.storage.len() < initial_len {
            self.rebuild_index();
            true
        } else {
            false
        }
    }

    /// Clears all memories
    pub fn clear(&mut self) {
        self.storage.clear();
        self.index = IndiceAsociativo::new();
        self.next_id = 1;
    }

    // ========================================================================
    // STATISTICS
    // ========================================================================

    /// Total number of stored memories
    pub fn count(&self) -> usize {
        self.storage.len()
    }

    /// Total bytes stored
    pub fn total_bytes(&self) -> u64 {
        self.storage.iter().map(|m| m.data.len() as u64).sum()
    }

    /// Gets comprehensive statistics
    pub fn stats(&self) -> EideticStats {
        let strong_count = self.storage.iter().filter(|m| m.strength >= 0.9).count();

        EideticStats {
            total_memories: self.storage.len() as u64,
            total_bytes: self.total_bytes(),
            indexed_tags: self.index.tag_count() as u64,
            searches_performed: self.index.search_count(),
            perfect_memory: self.forgetting.is_none()
                || self.forgetting.as_ref().map(|f| !f.enabled).unwrap_or(true),
            average_strength: if self.storage.is_empty() {
                1.0
            } else {
                self.storage.iter().map(|m| m.strength as f64).sum::<f64>()
                    / self.storage.len() as f64
            } as f32,
            strong_memories: strong_count,
        }
    }
}

// ============================================================================
// RESULT TYPES
// ============================================================================

/// Memory metadata
#[derive(Debug, Clone)]
pub struct MemoryMetadata {
    pub id: u64,
    pub tags: Vec<String>,
    pub timestamp: u64,
    pub type_name: String,
    pub content_hash: u64,
    pub strength: f32,
    pub access_count: u32,
    pub last_access: Option<u64>,
    pub size_bytes: u64,
    pub verified: bool,
}

/// Search result with relevance score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: u64,
    pub score: f32,
    pub tags: Vec<String>,
    pub timestamp: u64,
    pub strength: f32,
}

/// Maintenance result
#[derive(Debug, Clone)]
pub struct MaintenanceResult {
    pub memories_decayed: usize,
    pub memories_removed: usize,
    pub bytes_freed: u64,
    pub current_count: usize,
}

impl MaintenanceResult {
    fn no_op() -> Self {
        Self {
            memories_decayed: 0,
            memories_removed: 0,
            bytes_freed: 0,
            current_count: 0,
        }
    }
}

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub total: usize,
    pub verified: usize,
    pub failed: Vec<u64>,
}

/// Statistics
#[derive(Debug, Clone)]
pub struct EideticStats {
    pub total_memories: u64,
    pub total_bytes: u64,
    pub indexed_tags: u64,
    pub searches_performed: u64,
    pub perfect_memory: bool,
    pub average_strength: f32,
    pub strong_memories: usize,
}

// ============================================================================
// UTILITIES
// ============================================================================

/// Gets current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve_string() {
        let mut memory = EideticMemory::new();

        let id = memory.store(&"Hello, World!".to_string(), vec!["greeting".to_string()]);
        assert_eq!(id, 1);

        let retrieved: Option<String> = memory.retrieve(id);
        assert_eq!(retrieved, Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_store_and_retrieve_numbers() {
        let mut memory = EideticMemory::new();

        let id1 = memory.store(&42u64, vec!["number".to_string()]);
        let id2 = memory.store(&-1234i64, vec!["negative".to_string()]);
        let id3 = memory.store(&3.14159f64, vec!["pi".to_string()]);

        assert_eq!(memory.retrieve::<u64>(id1), Some(42));
        assert_eq!(memory.retrieve::<i64>(id2), Some(-1234));
        assert!((memory.retrieve::<f64>(id3).unwrap() - 3.14159).abs() < 0.0001);
    }

    #[test]
    fn test_raw_storage() {
        let mut memory = EideticMemory::new();

        let data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE];
        let id = memory.store_raw(&data, vec!["binary".to_string()]);

        let retrieved = memory.retrieve_raw(id);
        assert_eq!(retrieved, Some(data));
    }

    #[test]
    fn test_search_by_tags() {
        let mut memory = EideticMemory::new();

        memory.store(
            &"Apple".to_string(),
            vec!["fruit".to_string(), "red".to_string()],
        );
        memory.store(
            &"Banana".to_string(),
            vec!["fruit".to_string(), "yellow".to_string()],
        );
        memory.store(
            &"Carrot".to_string(),
            vec!["vegetable".to_string(), "orange".to_string()],
        );

        let results = memory.search(&["fruit"]);
        assert_eq!(results.len(), 2);

        let and_results = memory.search(&["fruit", "red"]);
        assert_eq!(and_results.len(), 1);
    }

    #[test]
    fn test_temporal_queries() {
        let mut memory = EideticMemory::new();

        memory.store(&"First".to_string(), vec![]);
        memory.store(&"Second".to_string(), vec![]);

        let count = memory.count();
        assert_eq!(count, 2);

        let range = memory.find_in_range(0, u64::MAX);
        assert_eq!(range.len(), 2);
    }

    #[test]
    fn test_integrity_verification() {
        let mut memory = EideticMemory::new();

        let id = memory.store(&"Test data".to_string(), vec![]);

        let meta = memory.get_metadata(id).unwrap();
        assert!(meta.verified);
        assert!(memory.get_by_hash(meta.content_hash).is_some());
        assert_eq!(memory.retrieve::<String>(id).unwrap(), "Test data");
    }

    #[test]
    fn test_duplicate_detection() {
        let mut memory = EideticMemory::new();

        let data = b"Duplicate content";
        let id1 = memory.store_raw(data, vec![]);
        let id2 = memory.store_raw(data, vec![]);

        // Should detect duplicate
        let duplicate = memory.find_duplicate(data);
        assert!(duplicate.is_some());

        // Both IDs should exist
        assert!(memory.get_metadata(id1).is_some());
        assert!(memory.get_metadata(id2).is_some());
    }

    #[test]
    fn test_forgetting_config() {
        // Perfect memory (default)
        let perfect = ForgettingConfig::perfect();
        assert!(!perfect.is_enabled());

        // With forgetting
        let forgetting = ForgettingConfig::with_forgetting(3600, 0.9, 60, 0.1);
        assert!(forgetting.is_enabled());
    }

    #[test]
    fn test_encoding_checksum() {
        let encoder = EncodingPerfecto::new();

        let data = vec![1u8, 2, 3, 4, 5];
        let encoded = encoder.encode(&data);

        let decoded: Vec<u8> = encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_delete() {
        let mut memory = EideticMemory::new();

        let id1 = memory.store(&"First".to_string(), vec![]);
        let id2 = memory.store(&"Second".to_string(), vec![]);

        assert!(memory.delete(id1));
        assert_eq!(memory.count(), 1);
        assert!(memory.retrieve::<String>(id2).is_some());
    }

    #[test]
    fn test_verify_all() {
        let mut memory = EideticMemory::new();

        memory.store(&"Test".to_string(), vec![]);
        memory.store(&123u64, vec![]);

        let result = memory.verify_all();
        assert_eq!(result.total, 2);
        assert_eq!(result.verified, 2);
        assert!(result.failed.is_empty());
    }

    #[test]
    fn test_stats() {
        let mut memory = EideticMemory::new();

        memory.store(
            &"Data".to_string(),
            vec!["tag1".to_string(), "tag2".to_string()],
        );

        let stats = memory.stats();
        assert_eq!(stats.total_memories, 1);
        assert!(stats.perfect_memory);
        assert_eq!(stats.indexed_tags, 2);
    }
}
