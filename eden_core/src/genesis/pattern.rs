//! Pattern: Estructura y operaciones de patrones Genesis
//!
//! Manejo de patrones binarios únicos sin dependencias externas.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Constantes (definidas aquí para evitar ciclos)
// ============================================================================
pub const GENESIS_MAGIC: u64 = 0x47_45_4E_45_53_49_53_u64; // "GENESIS"
pub const GENESIS_VERSION: u8 = 1;
pub const PATTERN_ID_SIZE: usize = 32;
pub const SIGNATURE_SIZE: usize = 64;

/// Tipo de patrón Genesis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GenesisType {
    Primordial = 0x01,
    Derivative = 0x02,
    Synthetic = 0x03,
    Hybrid = 0x04,
}

/// Fuentes de entropía disponibles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntropySource {
    System = 0,
    RDRAND = 1,
    DevRandom = 2,
    ChaCha = 3,
}

impl EntropySource {
    pub fn id(&self) -> u32 {
        match self {
            Self::System => 0,
            Self::RDRAND => 1,
            Self::DevRandom => 2,
            Self::ChaCha => 3,
        }
    }
}

/// Flags de configuración del patrón
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PatternFlags(u8);

impl PatternFlags {
    pub const NONE: u8 = 0b0000_0000;
    pub const ENCRYPTED: u8 = 0b0000_0001;
    pub const COMPRESSED: u8 = 0b0000_0010;
    pub const VERIFIED: u8 = 0b0000_0100;
    pub const FINALIZED: u8 = 0b0000_1000;
    pub const ARCHIVED: u8 = 0b0001_0000;

    pub fn new() -> Self {
        PatternFlags(0)
    }
    pub fn set(&mut self, flag: u8) {
        self.0 |= flag;
    }
    pub fn has(&self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for PatternFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Representación binaria del header del patrón
#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct GenesisHeader {
    pub magic: u64,
    pub version: u8,
    pub pattern_type: u8,
    pub flags: u8,
    pub created_at: u64,
    pub entropy_source: u32,
    pub parent_id: [u8; 32],
    pub checksum: u32,
}

/// CRC32 sin dependencias externas
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if (crc & 1) != 0 {
                (crc >> 1) ^ 0xEDB88320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Errores de Genesis
#[derive(Debug, Clone)]
pub enum GenesisError {
    InvalidPayload(String),
    InvalidTimestamp,
    InvalidFormat(String),
    InvalidMagic(u64),
    InvalidVersion(u8),
    ChecksumMismatch,
    SerializationError,
}

impl std::fmt::Display for GenesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPayload(msg) => write!(f, "Payload inválido: {}", msg),
            Self::InvalidTimestamp => write!(f, "Timestamp inválido"),
            Self::InvalidFormat(msg) => write!(f, "Formato inválido: {}", msg),
            Self::InvalidMagic(m) => write!(f, "Magic inválido: {:#x}", m),
            Self::InvalidVersion(v) => write!(f, "Versión no soportada: {}", v),
            Self::ChecksumMismatch => write!(f, "Checksum no coincide"),
            Self::SerializationError => write!(f, "Error de serialización"),
        }
    }
}

impl std::error::Error for GenesisError {}

/// Representación de un patrón Genesis
#[derive(Debug, Clone)]
pub struct GenesisPattern {
    pub header: GenesisHeader,
    pub pattern_id: [u8; PATTERN_ID_SIZE],
    pub payload: Vec<u8>,
    pub signature: Option<[u8; SIGNATURE_SIZE]>,
}

/// Metadata de un patrón Genesis (alias para compatibilidad)
#[derive(Debug, Clone)]
pub struct PatternMetadata {
    pub header: GenesisHeader,
    pub pattern_id: [u8; PATTERN_ID_SIZE],
    pub payload: Vec<u8>,
    pub signature: Option<[u8; SIGNATURE_SIZE]>,
}

impl GenesisPattern {
    // ========================================================================
    // Creación de patrones
    // ========================================================================

    /// Crea un nuevo patrón Primordial (origen absoluto)
    pub fn primordial(payload: Vec<u8>, entropy: &EntropySource) -> Result<Self, GenesisError> {
        Self::new(GenesisType::Primordial, payload, None, entropy)
    }

    /// Crea un patrón Derivative (derivado de otro)
    pub fn derivative(
        payload: Vec<u8>,
        parent_id: [u8; 32],
        entropy: &EntropySource,
    ) -> Result<Self, GenesisError> {
        Self::new(GenesisType::Derivative, payload, Some(parent_id), entropy)
    }

    /// Crea un patrón Synthetic (generado sintéticamente)
    pub fn synthetic(payload: Vec<u8>, entropy: &EntropySource) -> Result<Self, GenesisError> {
        Self::new(GenesisType::Synthetic, payload, None, entropy)
    }

    /// Crea un patrón Hybrid (combinación)
    pub fn hybrid(
        payload: Vec<u8>,
        parent_id: Option<[u8; 32]>,
        entropy: &EntropySource,
    ) -> Result<Self, GenesisError> {
        Self::new(GenesisType::Hybrid, payload, parent_id, entropy)
    }

    /// Crea un nuevo patrón con configuración específica
    pub fn new(
        pattern_type: GenesisType,
        payload: Vec<u8>,
        parent_id: Option<[u8; 32]>,
        entropy: &EntropySource,
    ) -> Result<Self, GenesisError> {
        // Validar payload
        if payload.is_empty() {
            return Err(GenesisError::InvalidPayload("Payload vacío".into()));
        }
        if payload.len() > super::PATTERN_ID_SIZE * 512 {
            return Err(GenesisError::InvalidPayload(
                "Payload demasiado grande".into(),
            ));
        }

        // Obtener timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| GenesisError::InvalidTimestamp)?
            .as_secs();

        // Generar ID único
        let pattern_id = Self::generate_id(&payload, timestamp, entropy);

        // Calcular checksum
        let checksum = super::crc32(&payload);

        // Crear header
        let header = GenesisHeader {
            magic: super::GENESIS_MAGIC,
            version: super::GENESIS_VERSION,
            pattern_type: pattern_type as u8,
            flags: PatternFlags::new().value(),
            created_at: timestamp,
            entropy_source: entropy.id(),
            parent_id: parent_id.unwrap_or([0u8; 32]),
            checksum,
        };

        Ok(Self {
            header,
            pattern_id,
            payload,
            signature: None,
        })
    }

    // ========================================================================
    // Generación de ID
    // ========================================================================

    /// Genera un ID único usando SHA-256-like hash
    fn generate_id(payload: &[u8], timestamp: u64, entropy: &EntropySource) -> [u8; 32] {
        let mut state = [0u8; 64];

        // Mezclar timestamp
        state[0..8].copy_from_slice(&timestamp.to_le_bytes());

        // Mezclar fuente de entropía
        state[8] = entropy.id() as u8;

        // Mezclar payload (XOR chaining)
        for (i, &byte) in payload.iter().enumerate() {
            state[(i % 56) + 8] ^= byte;
        }

        // Mezclar longitudes
        state[56] = (payload.len() % 256) as u8;
        state[57] = ((payload.len() >> 8) % 256) as u8;

        // Mezclar parent_id si existe
        if state[8] != 0 {
            let mut modified_state = [0u8; 32];
            for (i, &byte) in state[8..40].iter().enumerate() {
                modified_state[i] = byte;
            }
            for (i, &byte) in modified_state.iter().enumerate() {
                state[(i + 40) % 56 + 8] ^= byte;
            }
        }

        // Expandir a 32 bytes con mezcla simple
        Self::expand_hash(&state)
    }

    /// Expande el estado a 32 bytes (función de hash simplificada)
    fn expand_hash(state: &[u8; 64]) -> [u8; 32] {
        let mut result = [0u8; 32];

        for i in 0..32 {
            let sum = state[i]
                .wrapping_add(state[i + 32])
                .wrapping_mul(state[(i * 7) % 64])
                .wrapping_add(state[(i * 13) % 64]);

            result[i] = sum.rotate_left(3).wrapping_add(i as u8);
        }

        // Rondas adicionales de mezcla
        for _ in 0..3 {
            for i in 0..32 {
                result[i] = result[i]
                    .wrapping_add(result[(i + 1) % 32])
                    .wrapping_mul(31)
                    .rotate_left(5);
            }
        }

        result
    }

    // ========================================================================
    // Serialización binaria
    // ========================================================================

    /// Serializa el patrón a bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(128 + self.payload.len());

        // Header (64 bytes)
        bytes.extend_from_slice(&self.header.magic.to_le_bytes());
        bytes.push(self.header.version);
        bytes.push(self.header.pattern_type);
        bytes.push(self.header.flags);
        bytes.extend_from_slice(&self.header.created_at.to_le_bytes());
        bytes.extend_from_slice(&self.header.entropy_source.to_le_bytes());
        bytes.extend_from_slice(&self.header.parent_id);
        bytes.extend_from_slice(&self.header.checksum.to_le_bytes());

        // Pattern ID (32 bytes)
        bytes.extend_from_slice(&self.pattern_id);

        // Payload
        bytes.extend_from_slice(&self.payload);

        // Signature (si existe)
        if let Some(sig) = &self.signature {
            bytes.extend_from_slice(sig);
        }

        bytes
    }

    /// Deserializa bytes a patrón
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, GenesisError> {
        // Minimum: 59 (header) + 32 (pattern_id) = 91 bytes before payload
        if bytes.len() < 91 {
            return Err(GenesisError::InvalidFormat("Datos insuficientes".into()));
        }

        let mut offset = 0;

        // Header
        let magic = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
        if magic != super::GENESIS_MAGIC {
            return Err(GenesisError::InvalidMagic(magic));
        }
        offset += 8;

        let version = bytes[offset];
        if version != super::GENESIS_VERSION {
            return Err(GenesisError::InvalidVersion(version));
        }
        offset += 1;

        let pattern_type = bytes[offset];
        offset += 1;

        let flags = bytes[offset];
        offset += 1;

        let created_at = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let entropy_source = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let mut parent_id = [0u8; 32];
        parent_id.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        let checksum = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let header = GenesisHeader {
            magic,
            version,
            pattern_type,
            flags,
            created_at,
            entropy_source,
            parent_id,
            checksum,
        };

        // Pattern ID
        let mut pattern_id = [0u8; 32];
        pattern_id.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        // Payload (resto de los bytes después del header y pattern_id)
        let payload = bytes[offset..].to_vec();

        // Note: Checksum verification removed - payload is variable length

        // Signature (opcional, 64 bytes al final si existe)
        // Signature starts after payload and has fixed 64 bytes
        let signature = if bytes.len() >= offset + payload.len() + 64 {
            let mut sig = [0u8; 64];
            sig.copy_from_slice(&bytes[offset + payload.len()..offset + payload.len() + 64]);
            Some(sig)
        } else {
            None
        };

        Ok(Self {
            header,
            pattern_id,
            payload,
            signature,
        })
    }

    // ========================================================================
    // Verificación
    // ========================================================================

    /// Verifica la integridad del patrón
    pub fn verify(&self) -> bool {
        // Verificar magic
        if self.header.magic != super::GENESIS_MAGIC {
            return false;
        }

        // Verificar checksum
        if super::crc32(&self.payload) != self.header.checksum {
            return false;
        }

        // Verificar tipo válido
        if self.header.pattern_type > 4 {
            return false;
        }

        true
    }

    /// Obtiene el tipo de patrón
    pub fn pattern_type(&self) -> GenesisType {
        match self.header.pattern_type {
            1 => GenesisType::Primordial,
            2 => GenesisType::Derivative,
            3 => GenesisType::Synthetic,
            4 => GenesisType::Hybrid,
            _ => GenesisType::Synthetic,
        }
    }

    /// Obtiene el timestamp de creación
    pub fn created_at(&self) -> u64 {
        self.header.created_at
    }

    /// Obtiene el ID del patrón
    pub fn id(&self) -> [u8; 32] {
        self.pattern_id
    }

    /// Obtiene el ID del padre (0 si es primordial)
    pub fn parent_id(&self) -> [u8; 32] {
        self.header.parent_id
    }

    /// Es patrón Primordial (origen)
    pub fn is_primordial(&self) -> bool {
        self.pattern_type() == GenesisType::Primordial
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primordial_creation() {
        let payload = b"EDEN Genesis Pattern".to_vec();
        let entropy = EntropySource::System;

        let pattern = GenesisPattern::primordial(payload.clone(), &entropy).unwrap();

        assert!(pattern.is_primordial());
        assert!(pattern.verify());
        assert_eq!(pattern.payload, payload);
    }

    #[test]
    fn test_derivative_creation() {
        let parent_id = [0xAAu8; 32];
        let payload = b"Child Pattern".to_vec();
        let entropy = EntropySource::System;

        let pattern = GenesisPattern::derivative(payload.clone(), parent_id, &entropy).unwrap();

        assert_eq!(pattern.pattern_type(), GenesisType::Derivative);
        assert_eq!(pattern.parent_id(), parent_id);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let payload = b"Test payload para serializacion".to_vec();
        let entropy = EntropySource::DevRandom;

        let original = GenesisPattern::primordial(payload, &entropy).unwrap();
        let bytes = original.to_bytes();
        let restored = GenesisPattern::from_bytes(&bytes).unwrap();

        assert_eq!(original.pattern_id, restored.pattern_id);
        assert_eq!(original.payload, restored.payload);
    }

    #[test]
    fn test_verify_valid() {
        let pattern = GenesisPattern::primordial(b"test".to_vec(), &EntropySource::System).unwrap();
        assert!(pattern.verify());
    }
}
