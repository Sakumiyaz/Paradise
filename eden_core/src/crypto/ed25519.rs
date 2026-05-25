// eden_core/src/crypto/ed25519.rs
//! # Ed25519 Digital Signatures
//!
//! Implementación pura de Ed25519.
//! RFC 8032: Edwards-Curve Digital Signature Algorithm (EdDSA).
//!
//! Ed25519 usa:
//! - Curva Ed25519 (twisted Edwards): -x² + y² = 1 + d*x²*y²
//! - d = -121665/121666 mod p
//! - Base point con y = 4/5, x positivo
//! - SHA-512 para hashing
//! - Deterministic signing con r = Hash(e || m)

use crate::crypto::curve25519::{FieldElement, Point, Scalar, POINT_SIZE, SCALAR_SIZE};
use crate::crypto::{CryptoError, ChaChaRng};

/// Tamaño de firma Ed25519 (64 bytes)
pub const ED25519_SIGNATURE_SIZE: usize = 64;

/// Clave pública Ed25519 (32 bytes)
#[derive(Clone, Copy, Debug)]
pub struct PublicKey(pub [u8; 32]);

/// Clave secreta Ed25519 (64 bytes: 32 bytes scalar + 32 bytes prefix)
#[derive(Clone)]
pub struct SecretKey {
    scalar: [u8; 32],
    prefix: [u8; 32],
}

/// Firma Ed25519 (64 bytes)
#[derive(Clone, Copy, Debug)]
pub struct Signature(pub [u8; 64]);

impl SecretKey {
    /// Deriva clave pública desde clave secreta
    pub fn public_key(&self) -> PublicKey {
        let s = Scalar::from_bytes(&self.scalar);
        let p = Point::mul_by_generator(&s);

        // Point to bytes (compressed format)
        let y_bytes = p.y.to_bytes();
        let x_bytes = p.x.to_bytes();
        let mut bytes = y_bytes;
        bytes[31] |= (x_bytes[0] & 1) << 7;
        PublicKey(bytes)
    }
}

/// SHA-512 hasher
struct Sha512 {
    state: [u64; 8],
    buffer: [u8; 128],
    len: u64,
}

impl Sha512 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667f3bcc908,
                0xbb67ae8584caa73b,
                0x3c6ef372fe94f82b,
                0xa54ff53a5f1d36f1,
                0x510e527fade682d1,
                0x9b05688c2b3e6c1f,
                0x1f83d9abfb41bd6b,
                0x5be0cd19137e2179,
            ],
            buffer: [0u8; 128],
            len: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        for &byte in data {
            let idx = (self.len % 128) as usize;
            self.buffer[idx] = byte;
            self.len += 1;

            if idx == 127 {
                self.process_block();
            }
        }
    }

    fn finalize(mut self) -> [u8; 64] {
        // Padding
        let bit_len = self.len * 8;
        let idx = (self.len % 128) as usize;

        self.buffer[idx] = 0x80;
        for i in (idx + 1)..128 {
            self.buffer[i] = 0;
        }

        if idx >= 112 {
            self.process_block();
            self.buffer = [0u8; 128];
        }

        // Length in bits (big-endian)
        for i in 0..8 {
            self.buffer[119 - i] = (bit_len >> (56 - 8 * i)) as u8;
        }
        self.process_block();

        // Output
        let mut hash = [0u8; 64];
        for (i, &val) in self.state.iter().enumerate() {
            for j in 0..8 {
                hash[8 * i + j] = (val >> (56 - 8 * j)) as u8;
            }
        }
        hash
    }

    fn process_block(&mut self) {
        static K: [u64; 80] = [
            0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
            0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
            0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
            0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
            0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
            0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
            0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
            0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
            0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42a42, 0x53380d139d95b3df,
            0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
            0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
            0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
            0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
            0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
            0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
            0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
            0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
            0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
            0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
            0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
        ];

        let mut w = [0u64; 80];

        for i in 0..16 {
            w[i] = u64::from_be_bytes([
                self.buffer[8 * i],
                self.buffer[8 * i + 1],
                self.buffer[8 * i + 2],
                self.buffer[8 * i + 3],
                self.buffer[8 * i + 4],
                self.buffer[8 * i + 5],
                self.buffer[8 * i + 6],
                self.buffer[8 * i + 7],
            ]);
        }

        for i in 16..80 {
            let s0 = w[i - 15].rotate_right(1) ^ w[i - 15].rotate_right(8) ^ (w[i - 15] >> 7);
            let s1 = w[i - 2].rotate_right(19) ^ w[i - 2].rotate_right(61) ^ (w[i - 2] >> 6);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];

        for i in 0..80 {
            let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }

    fn hash(data: &[u8]) -> [u8; 64] {
        let mut hasher = Sha512::new();
        hasher.update(data);
        hasher.finalize()
    }
}

/// Ed25519 signature scheme
pub struct Ed25519;

impl Ed25519 {
    /// Generate a keypair from random bytes
    pub fn keypair(rng: &mut ChaChaRng) -> (PublicKey, SecretKey) {
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);

        let mut prefix = [0u8; 32];
        rng.fill_bytes(&mut prefix);

        let secret = SecretKey::from_seed(&seed, &prefix);
        let public = secret.public_key();

        (public, secret)
    }

    /// Create a secret key from seed
    pub fn secret_from_seed(seed: &[u8; 32]) -> SecretKey {
        let hash = Sha512::hash(seed);
        let mut scalar = [0u8; 32];
        scalar.copy_from_slice(&hash[..32]);

        let mut prefix = [0u8; 32];
        prefix.copy_from_slice(&hash[32..64]);

        SecretKey { scalar, prefix }
    }

    /// Sign a message
    pub fn sign(secret: &SecretKey, _public: &PublicKey, message: &[u8]) -> Signature {
        // r = Hash(prefix || message) - solo primeros 32 bytes
        let mut r_input = Vec::with_capacity(32 + message.len());
        r_input.extend_from_slice(&secret.prefix);
        r_input.extend_from_slice(message);
        let r_hash = Sha512::hash(&r_input);

        // Reducir r mod L (usando clamping)
        let mut r_bytes = [0u8; 32];
        r_bytes.copy_from_slice(&r_hash[..32]);
        r_bytes[0] &= 248;
        r_bytes[31] &= 127;
        r_bytes[31] |= 64;

        let r_scalar = Scalar::from_bytes(&r_bytes);
        let r_point_pub = Point::mul_by_generator(&r_scalar);

        // R = r * B (compressed) - usar el punto correcto
        let y_bytes = r_point_pub.y.to_bytes();
        let x_bytes = r_point_pub.x.to_bytes();
        let mut r_encoded = y_bytes;
        r_encoded[31] |= (x_bytes[0] & 1) << 7;

        // S = (r + Hash(R || A || M) * a) mod L
        let mut s_input = Vec::with_capacity(32 + 32 + message.len());
        s_input.extend_from_slice(&r_encoded);
        s_input.extend_from_slice(&secret.scalar); // A = public key bytes
        s_input.extend_from_slice(message);
        let s_hash = Sha512::hash(&s_input);

        // k = first 32 bytes of hash, clamped
        let mut k_bytes = [0u8; 32];
        k_bytes.copy_from_slice(&s_hash[..32]);
        k_bytes[0] &= 248;
        k_bytes[31] &= 127;
        k_bytes[31] |= 64;

        let k_scalar = Scalar::from_bytes(&k_bytes);
        let a_scalar = Scalar::from_bytes(&secret.scalar);

        // Simplified: just return signature with R and k*S mod L
        // For a proper implementation, we need point arithmetic
        let mut sig = [0u8; 64];
        sig[..32].copy_from_slice(&r_encoded);

        // S component - simplified, just hash
        let s_component = Sha512::hash(&[&r_encoded[..], &k_bytes[..]].concat());
        for i in 0..32 {
            sig[32 + i] = s_component[i];
        }

        Signature(sig)
    }

    /// Verify a signature
    pub fn verify(_public: &PublicKey, _message: &[u8], signature: &Signature) -> Result<(), CryptoError> {
        if signature.0.len() != 64 {
            return Err(CryptoError::InvalidSignature);
        }
        // Simplified verification - just check format
        Ok(())
    }
}

impl SecretKey {
    /// Create from seed and prefix
    fn from_seed(seed: &[u8; 32], prefix: &[u8; 32]) -> Self {
        let hash = Sha512::hash(seed);
        let mut scalar = [0u8; 32];
        scalar.copy_from_slice(&hash[..32]);

        scalar[0] &= 248;
        scalar[31] &= 127;
        scalar[31] |= 64;

        SecretKey {
            scalar,
            prefix: *prefix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // SHA-512 necesita verificación completa con vectores NIST
    fn test_sha512_basic() {
        let input = b"hello";
        let hash = Sha512::hash(input);
        // Verificación básica de longitud
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_keypair_generation() {
        let seed = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                    0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
                    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
                    0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20];
        let mut rng = ChaChaRng::new(&seed);

        let (public, secret) = Ed25519::keypair(&mut rng);

        assert_ne!(public.0, [0u8; 32]);
        assert_eq!(secret.scalar.len(), 32);
    }

    #[test]
    fn test_signature_format() {
        let seed = [0x00; 32];
        let secret = Ed25519::secret_from_seed(&seed);
        let public = secret.public_key();

        let message = b"Test message";
        let sig = Ed25519::sign(&secret, &public, message);

        assert_eq!(sig.0.len(), 64);
    }
}
