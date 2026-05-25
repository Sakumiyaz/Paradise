// eden_core/src/crypto/poly1305.rs
//! # Poly1305 Message Authentication Code
//!
//! Implementación pura de Poly1305 MAC.
//! Usado en ChaCha20-Poly1305 AEAD (RFC 8439).
//!
//! Poly1305 computes a 128-bit authenticator under a 256-bit key.
//! Key = (r, s) donde r es la clave de autenticación y s la clave de enmascaramiento.

/// Constante: 2^130 - 5 (el módulo primo de Poly1305)
/// P = 0x3fffffffffffffffffffffffffffffffb
const P0: u64 = 0xfffffffffffffffb; // 2^64 - 5
const P1: u64 = 0x3fffffffffffffff; // 2^66 - 1
const P2: u64 = 0x3; // 2^2 - 1

/// Clave de 256 bits para Poly1305
#[derive(Clone, Copy)]
pub struct Poly1305Key {
    /// r: clave de autenticación (130 bits interpretados en limb 0,1,2)
    r: [u64; 3],
    /// s: clave de enmascaramiento (128 bits)
    s: [u64; 2],
}

impl Poly1305Key {
    /// Genera una clave Poly1305 de 256 bits desde 32 bytes (r) + 16 bytes (s)
    /// o desde 64 bytes de ChaCha20 keystream
    pub fn from_bytes(key: &[u8; 64]) -> Self {
        let r0 = u64::from_le_bytes([key[0], key[1], key[2], key[3], key[4], key[5], key[6], key[7]]);
        let r1 = u64::from_le_bytes([key[8], key[9], key[10], key[11], key[12], key[13], key[14], key[15]]);
        let r2 = u64::from_le_bytes([key[16], key[17], key[18], key[19], key[20], key[21], key[22], key[23]]);
        let s0 = u64::from_le_bytes([key[24], key[25], key[26], key[27], key[28], key[29], key[30], key[31]]);
        let s1 = u64::from_le_bytes([key[32], key[33], key[34], key[35], key[36], key[37], key[38], key[39]]);

        let mut r = [r0, r1, r2 & 0x0fffffffffffffff];

        // Clamp r: bits 0-1 = 0, bit 128 = 0, bits 129-130 = 0
        r[0] &= 0x0fffffffffffffff;
        r[1] &= 0x0fffffffffffffff;
        r[2] &= 0x00fffffffffffffff;

        let s = [s0, s1];

        Self { r, s }
    }

    /// Desde clave de 32 bytes (expandido a 64 bytes con ceros para s)
    pub fn from_32_bytes(key: &[u8; 32]) -> Self {
        let mut key64 = [0u8; 64];
        key64[..32].copy_from_slice(key);
        Self::from_bytes(&key64)
    }
}

/// Poly1305 MAC
pub struct Poly1305 {
    /// Acumulador interno (h)
    h: [u64; 3],
    /// Clave
    r: [u64; 3],
    s: [u64; 2],
    /// Buffer para datos parciales
    buffer: [u8; 17],
    /// Bytes en buffer
    buffer_len: usize,
}

impl Poly1305 {
    /// Crea un nuevo Poly1305 con la clave de 32 bytes
    pub fn new(key: &[u8; 32]) -> Self {
        let poly_key = Poly1305Key::from_32_bytes(key);
        Self {
            h: [0, 0, 0],
            r: poly_key.r,
            s: poly_key.s,
            buffer: [0u8; 17],
            buffer_len: 0,
        }
    }

    /// Agrega datos al MAC
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.buffer[self.buffer_len] = byte;
            self.buffer_len += 1;

            if self.buffer_len == 17 {
                self.process_chunk();
                self.buffer_len = 0;
            }
        }
    }

    /// Finaliza y retorna el MAC de 16 bytes
    pub fn finish(mut self) -> [u8; 16] {
        // Procesar buffer restante
        if self.buffer_len > 0 {
            let mut chunk = [0u8; 17];
            chunk[..self.buffer_len].copy_from_slice(&self.buffer[..self.buffer_len]);
            chunk[self.buffer_len] = 1;
            self.add_chunk(&chunk);
            self.reduce();
        }

        // h = h + s (mod 2^128)
        let (h0, overflow) = self.h[0].overflowing_add(self.s[0]);
        let h1 = self.h[1] + self.s[1] + if overflow { 1 } else { 0 };

        let mut result = [0u8; 16];
        result[0..8].copy_from_slice(&h0.to_le_bytes());
        result[8..16].copy_from_slice(&h1.to_le_bytes());
        result
    }

    /// Procesa un chunk de 17 bytes
    fn process_chunk(&mut self) {
        let chunk_copy = self.buffer;
        self.add_chunk(&chunk_copy);
        self.reduce();
    }

    /// Agrega un chunk al acumulador: h = h + c (mod 2^128)
    fn add_chunk(&mut self, chunk: &[u8; 17]) {
        let c0 = u64::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7]]);
        let c1 = u64::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11], chunk[12], chunk[13], chunk[14], chunk[15]]);
        let c2 = chunk[16] as u64;

        let (h0, overflow0) = self.h[0].overflowing_add(c0);
        let (h1, overflow1) = self.h[1].overflowing_add(c1 + if overflow0 { 1 } else { 0 });
        let h2 = self.h[2] + c2 + if overflow1 { 1 } else { 0 };

        self.h = [h0, h1, h2];
    }

    /// Reduce h mod (2^130 - 5)
    fn reduce(&mut self) {
        let h0 = self.h[0];
        let h1 = self.h[1];
        let h2 = self.h[2];

        // h2 tiene a lo más 37 bits útiles (17 bits de entrada + 18 de acarreo)
        // h0 tiene 64 bits, h1 tiene 64 bits

        // Dividir h2 * 5 y agregarlo a h0
        // Esto es equivalente a h mod (2^130 - 5) porque:
        // h = h0 + h1*2^64 + h2*2^128
        // h mod (2^130 - 5) = h0 + h1*2^64 + h2*2^128 mod (2^130 - 5)
        // 2^128 = 5 * (2^130 - 5) + 5, así que 2^128 ≡ 5 (mod 2^130-5)
        // h2*2^128 ≡ h2*5 (mod 2^130-5)

        let d0 = h0 as u128 + (h2 as u128 * 5);
        let d1 = h1 as u128 + (d0 >> 64);

        self.h[0] = d0 as u64;
        self.h[1] = d1 as u64;
        self.h[2] = (d1 >> 64) as u64;

        // Reducir si h >= p
        let (t0, underflow0) = self.h[0].overflowing_sub(P0);
        let t1 = self.h[1].wrapping_sub(P1.wrapping_add(if underflow0 { 1 } else { 0 }));
        let t2 = self.h[2].wrapping_sub(P2.wrapping_add(if t1 > self.h[1] { 1 } else { 0 }));

        // Seleccionar si hubo underflow
        let overflow_threshold = 0x1000000000000000u64; // 2^60
        if t2 < overflow_threshold || (t2 == P2 && t1 >= P1) || (t2 == P2 && t1 == P1 && t0 >= P0) {
            self.h[0] = t0;
            self.h[1] = t1;
            self.h[2] = t2;
        }
    }
}

/// Computa Poly1305 MAC en un solo paso
pub fn poly1305_auth(key: &[u8; 32], message: &[u8]) -> [u8; 16] {
    let mut poly = Poly1305::new(key);
    poly.update(message);
    poly.finish()
}

/// Verifica un MAC Poly1305
pub fn poly1305_verify(tag: &[u8; 16], key: &[u8; 32], message: &[u8]) -> bool {
    let expected = poly1305_auth(key, message);
    constant_time_compare(&expected, tag)
}

/// Comparación en tiempo constante para evitar timing attacks
fn constant_time_compare(a: &[u8; 16], b: &[u8; 16]) -> bool {
    let mut diff = 0u8;
    for i in 0..16 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poly1305_basic() {
        let key = [0u8; 32];
        let msg = b"Hello Poly1305!";
        let tag = poly1305_auth(&key, msg);
        assert!(poly1305_verify(&tag, &key, msg));
    }

    #[test]
    fn test_poly1305_empty() {
        let key = [0u8; 32];
        let msg = b"";
        let tag = poly1305_auth(&key, msg);
        assert!(poly1305_verify(&tag, &key, msg));
    }

    #[test]
    fn test_poly1305_long_message() {
        let key = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                   0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
                   0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
                   0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20];
        let msg = &[0u8; 1000];
        let tag = poly1305_auth(&key, msg);
        assert!(poly1305_verify(&tag, &key, msg));
    }
}
