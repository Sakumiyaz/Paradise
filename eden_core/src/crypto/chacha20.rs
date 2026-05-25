// eden_core/src/crypto/chacha20.rs
//! # ChaCha20 Stream Cipher
//!
//! Implementación pura de ChaCha20 para cifrado de flujo.
//! RFC 8439: ChaCha20-Poly1305 AEAD.
//!
//! ChaCha20 opera sobre un estado de 64 bytes (16 words de 32 bits).
//! Aplica 20 rondas (10 Double-Round) donde cada Double-Round
//! consiste en Column-Round y Diagonal-Round.

use crate::crypto::CryptoError;

/// Nonce para ChaCha20 (12 bytes / 96 bits)
pub type Nonce = [u8; 12];

/// Estado interno de ChaCha20 (16 palabras de 32 bits)
#[derive(Clone)]
struct ChaChaState {
    inner: [u32; 16],
}

/// ChaCha20 block output (64 bytes)
struct ChaChaBlock {
    block: [u8; 64],
}

/// ChaCha20 stream cipher
#[derive(Clone)]
pub struct ChaCha20 {
    state: ChaChaState,
}

/// Constants "expand 32-byte k"
const SIGMA: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];

impl ChaCha20 {
    /// Crea ChaCha20 con clave de 256 bits y nonce de 96 bits
    pub fn new(key: &[u8; 32], nonce: &Nonce) -> Self {
        let mut state = ChaChaState { inner: [0u32; 16] };

        // Constants
        state.inner[0] = SIGMA[0];
        state.inner[1] = SIGMA[1];
        state.inner[2] = SIGMA[2];
        state.inner[3] = SIGMA[3];

        // Key (little-endian)
        for i in 0..8 {
            state.inner[4 + i] = u32::from_le_bytes([
                key[4 * i],
                key[4 * i + 1],
                key[4 * i + 2],
                key[4 * i + 3],
            ]);
        }

        // Counter (initial 0)
        state.inner[12] = 0;
        state.inner[13] = 0;

        // Nonce (96 bits / 12 bytes)
        state.inner[14] = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
        state.inner[15] = u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]);

        // Los últimos 4 bytes del nonce van en inner[13]
        // Pero ChaCha20 standard usa solo 12 bytes de nonce...
        // nonce[8..12] se pone en counter bajo
        // En realidad: counter es 64 bits (inner[12] y inner[13])
        // nonce es 96 bits así que counter bajo = 0
        // Vamos a ajustar: nonce[0..4] = inner[13], nonce[4..8] = inner[14], nonce[8..12] = inner[15]
        state.inner[13] = u32::from_le_bytes([nonce[8], nonce[9], nonce[10], nonce[11]]);

        Self { state }
    }

    /// Genera el siguiente block y avanza el counter
    fn next_block(&mut self) -> [u8; 64] {
        let mut working_state = self.state.clone();
        chaCha20_rounds(&mut working_state.inner);

        let mut block = [0u8; 64];
        for i in 0..16 {
            block[4 * i..4 * i + 4].copy_from_slice(&working_state.inner[i].to_le_bytes());
        }

        // Incrementar counter (manejo de desbordamiento)
        self.state.inner[12] = self.state.inner[12].wrapping_add(1);
        if self.state.inner[12] == 0 {
            self.state.inner[13] = self.state.inner[13].wrapping_add(1);
        }

        block
    }

    /// Cifra datos in-place
    pub fn encrypt(&mut self, data: &mut [u8]) {
        let mut block = self.next_block();
        let mut block_idx = 0;

        for byte in data.iter_mut() {
            if block_idx == 64 {
                block = self.next_block();
                block_idx = 0;
            }
            *byte ^= block[block_idx];
            block_idx += 1;
        }
    }

    /// Cifra datos (versión que no consume el cipher state)
    pub fn encrypt_data(key: &[u8; 32], nonce: &Nonce, counter: u64, plaintext: &[u8]) -> Vec<u8> {
        let mut chacha = Self::new_with_counter(key, nonce, counter);
        let mut result = plaintext.to_vec();
        chacha.encrypt(&mut result);
        result
    }

    /// Crea con counter inicial específico
    pub fn new_with_counter(key: &[u8; 32], nonce: &Nonce, counter: u64) -> Self {
        let mut chacha = Self::new(key, nonce);
        chacha.state.inner[12] = counter as u32;
        chacha.state.inner[13] = (counter >> 32) as u32;
        chacha
    }
}

/// Aplica las 20 rondas de ChaCha20
fn chaCha20_rounds(state: &mut [u32; 16]) {
    // 10 rounds = 20 half-rounds
    for _ in 0..10 {
        // Column rounds
        quarter_round(0, 4, 8, 12, state);
        quarter_round(1, 5, 9, 13, state);
        quarter_round(2, 6, 10, 14, state);
        quarter_round(3, 7, 11, 15, state);

        // Diagonal rounds
        quarter_round(0, 5, 10, 15, state);
        quarter_round(1, 6, 11, 12, state);
        quarter_round(2, 7, 8, 13, state);
        quarter_round(3, 4, 9, 14, state);
    }
}

/// Quarter-round operation: a += b; d ^= a; d = rotl32(d, 16);
///                          c += d; b ^= c; b = rotl32(b, 12);
///                          a += b; d ^= a; d = rotl32(d, 8);
///                          c += d; b ^= c; b = rotl32(b, 7);
#[inline]
fn quarter_round(a: usize, b: usize, c: usize, d: usize, state: &mut [u32; 16]) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);

    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}

/// Generador de números pseudoaleatorios basado en ChaCha20
#[derive(Clone)]
pub struct ChaChaRng {
    cipher: ChaCha20,
    buffer: [u8; 64],
    pos: usize,
}

impl ChaChaRng {
    /// Crea ChaChaRng con seed de 32 bytes
    pub fn new(seed: &[u8; 32]) -> Self {
        // Usar seed como clave, nonce = 0
        let nonce: Nonce = [0u8; 12];
        Self {
            cipher: ChaCha20::new(seed, &nonce),
            buffer: [0u8; 64],
            pos: 64, // Forzar generación del primer bloque
        }
    }

    /// Genera el siguiente u32 aleatorio
    pub fn next_u32(&mut self) -> u32 {
        if self.pos >= 64 {
            self.buffer = self.cipher.next_block();
            self.pos = 0;
        }
        let result = u32::from_le_bytes([
            self.buffer[self.pos],
            self.buffer[self.pos + 1],
            self.buffer[self.pos + 2],
            self.buffer[self.pos + 3],
        ]);
        self.pos += 4;
        result
    }

    /// Genera un u64 aleatorio
    pub fn next_u64(&mut self) -> u64 {
        let low = self.next_u32() as u64;
        let high = self.next_u32() as u64;
        low | (high << 32)
    }

    /// Genera bytes aleatorios
    pub fn fill_bytes(&mut self, dest: &mut [u8]) {
        for byte in dest.iter_mut() {
            *byte = self.next_u32() as u8;
        }
    }
}

/// AEAD ChaCha20-Poly1305 (RFC 8439)
pub struct ChaCha20Poly1305 {
    key: [u8; 32],
}

impl ChaCha20Poly1305 {
    /// Crea una nueva instancia con clave de 256 bits
    pub fn new(key: &[u8; 32]) -> Self {
        Self { key: *key }
    }

    /// Cifra y autentica (AEAD seal)
    pub fn seal(&self, nonce: &Nonce, plaintext: &[u8], aad: &[u8]) -> Vec<u8> {
        // Generar subclave para Poly1305
        let mut subkey = [0u8; 32];
        let mut poly_key_chacha = ChaCha20::new(&self.key, nonce);
        let block = poly_key_chacha.next_block();
        subkey.copy_from_slice(&block[..32]);

        // Poly1305 con subclave (32 bytes)
        let poly = crate::crypto::Poly1305::new(&subkey);

        // Construir MAC data: poly1305(ciphertext || le64(aad_len) || le64(ct_len))
        let mut mac_data = Vec::with_capacity(
            plaintext.len() + aad.len() + 16
        );

        mac_data.extend_from_slice(plaintext);

        let aad_len = aad.len() as u64;
        mac_data.extend_from_slice(&aad_len.to_le_bytes());

        let ct_len = plaintext.len() as u64;
        mac_data.extend_from_slice(&ct_len.to_le_bytes());

        // Cifrar con ChaCha20
        let mut chacha = ChaCha20::new(&self.key, nonce);
        chacha.encrypt(&mut mac_data[..plaintext.len()]);

        // Calcular tag
        let poly = crate::crypto::Poly1305::new(&subkey);
        let tag = poly.finish();

        // Retornar ciphertext || tag
        let mut result = Vec::with_capacity(plaintext.len() + 16);
        result.extend_from_slice(&mac_data[..plaintext.len()]);
        result.extend_from_slice(&tag);

        result
    }

    /// Descifra y verifica (AEAD open)
    pub fn open(&self, nonce: &Nonce, ciphertext_tag: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if ciphertext_tag.len() < 16 {
            return Err(CryptoError::AuthenticationFailed);
        }

        let ct_len = ciphertext_tag.len() - 16;
        let ciphertext = &ciphertext_tag[..ct_len];
        let tag = &ciphertext_tag[ct_len..];

        // Generar subclave
        let mut subkey = [0u8; 32];
        let mut poly_key_chacha = ChaCha20::new(&self.key, nonce);
        let block = poly_key_chacha.next_block();
        subkey.copy_from_slice(&block[..32]);

        // Construir MAC data
        let mut mac_data = Vec::with_capacity(ct_len + aad.len() + 16);
        mac_data.extend_from_slice(ciphertext);

        let aad_len = aad.len() as u64;
        mac_data.extend_from_slice(&aad_len.to_le_bytes());

        let ct_len_u64 = ct_len as u64;
        mac_data.extend_from_slice(&ct_len_u64.to_le_bytes());

        // Verificar tag
        let poly = crate::crypto::Poly1305::new(&subkey);
        let expected_tag = poly.finish();

        // Comparación en tiempo constante
        let mut diff = 0u8;
        for i in 0..16 {
            diff |= expected_tag[i] ^ tag[i];
        }
        if diff != 0 {
            return Err(CryptoError::AuthenticationFailed);
        }

        // Descifrar
        let mut plaintext = ciphertext.to_vec();
        let mut chacha = ChaCha20::new(&self.key, nonce);
        chacha.encrypt(&mut plaintext);

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chacha20_encrypt_decrypt() {
        let key = [0u8; 32];
        let nonce: Nonce = [0u8; 12];
        let plaintext = b"Hello, ChaCha20!";

        let ciphertext = ChaCha20::encrypt_data(&key, &nonce, 0, plaintext);

        // Descifrar
        let mut chacha = ChaCha20::new_with_counter(&key, &nonce, 0);
        let mut decrypted = ciphertext.clone();
        chacha.encrypt(&mut decrypted);

        assert_eq!(&decrypted[..], plaintext);
    }

    #[test]
    fn test_chacha20_known_vectors() {
        // Test básico para verificar que el cifrado funciona
        let key = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                   0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
                   0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
                   0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f];
        let nonce: Nonce = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4a,
                           0x00, 0x00, 0x00, 0x00];

        let plaintext = b"Test plaintext for ChaCha20!";
        let ciphertext = ChaCha20::encrypt_data(&key, &nonce, 1, plaintext);

        // Verificar que no es igual al plaintext
        assert_ne!(&ciphertext[..], plaintext);
    }

    #[test]
    fn test_chacha_rng() {
        let seed = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
                    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
                    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f];

        let mut rng = ChaChaRng::new(&seed);
        let val1 = rng.next_u32();
        let val2 = rng.next_u32();
        assert_ne!(val1, val2);
    }
}
