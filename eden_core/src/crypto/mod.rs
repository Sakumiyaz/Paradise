// eden_core/src/crypto/mod.rs
//! # EDEN Cryptographic Primitives
//!
//! Implementaciones 100% Rust puro de primitivas criptográficas:
//! - ChaCha20-Poly1305 AEAD (RFC 8439)
//! - Poly1305 MAC
//! - X25519 Key Exchange (RFC 7748)
//! - Ed25519 Signatures (RFC 8032)
//!
//! Sin dependencias externas - todo desde cero.

pub mod chacha20;
pub mod poly1305;
pub mod curve25519;
pub mod x25519;
pub mod ed25519;

pub use chacha20::{ChaCha20, ChaChaRng,Nonce};
pub use poly1305::Poly1305;
pub use curve25519::{FieldElement, Scalar, POINT_SIZE, SCALAR_SIZE};
pub use x25519::X25519;
pub use ed25519::{Ed25519, PublicKey, SecretKey, Signature};

// ============================================================================
// Tipos compartidos
// ============================================================================

/// Resultado de operaciones criptográficas
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Errores criptográficos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoError {
    InvalidKey,
    InvalidNonce,
    InvalidSignature,
    InvalidPoint,
    InvalidScalar,
    AuthenticationFailed,
    DerivationFailed,
    RandomnessFailed,
}

impl core::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryptoError::InvalidKey => write!(f, "Clave inválida"),
            CryptoError::InvalidNonce => write!(f, "Nonce inválido"),
            CryptoError::InvalidSignature => write!(f, "Firma inválida"),
            CryptoError::InvalidPoint => write!(f, "Punto inválido en curva"),
            CryptoError::InvalidScalar => write!(f, "Escalar inválido"),
            CryptoError::AuthenticationFailed => write!(f, "Autenticación fallida"),
            CryptoError::DerivationFailed => write!(f, "Derivación de clave fallida"),
            CryptoError::RandomnessFailed => write!(f, "Error de aleatoriedad"),
        }
    }
}
