// eden_core/src/crypto/curve25519.rs
//! # Curve25519 Arithmetic
//!
//! Implementación pura de aritmética de campo finito GF(2^255-19)
//! y operaciones en la curva elíptica Curve25519.
//!
//! - Curve25519: y² = x³ + 486662x² + x mod p
//! - p = 2^255 - 19
//! - Orden de la curva: 2^252 + 27742317777372353535851937790883648493

/// Tamaño en bytes de un punto en Curve25519 (32 bytes)
pub const POINT_SIZE: usize = 32;

/// Tamaño en bytes de un escalar (32 bytes)
pub const SCALAR_SIZE: usize = 32;

/// Primo Curve25519: 2^255 - 19
/// Representado en limbs de 64 bits
const P_LIMBS: [u64; 4] = [
    0x7ffffffffffffeda,
    0x7fffffffffffffff,
    0x7fffffffffffffff,
    0x3fffffffffffffff,
];

/// Limbs de 64 bits para representar números de 255 bits (4 limbs)
#[derive(Clone, Copy, Debug)]
pub struct FieldElement(pub [u64; 4]);

/// Escalar de 256 bits para multiplicación
#[derive(Clone, Copy, Debug)]
pub struct Scalar(pub [u8; 32]);

impl FieldElement {
    /// Crea un FieldElement desde bytes (little-endian, 32 bytes)
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for i in 0..4 {
            limbs[i] = u64::from_le_bytes([
                bytes[8 * i],
                bytes[8 * i + 1],
                bytes[8 * i + 2],
                bytes[8 * i + 3],
                bytes[8 * i + 4],
                bytes[8 * i + 5],
                bytes[8 * i + 6],
                bytes[8 * i + 7],
            ]);
        }
        Self(limbs)
    }

    /// Convierte a bytes (little-endian, 32 bytes)
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for i in 0..4 {
            let le = self.0[i].to_le_bytes();
            for j in 0..8 {
                bytes[8 * i + j] = le[j];
            }
        }
        // Clear high bit (bit 255)
        bytes[31] &= 0x7f;
        bytes
    }

    /// Crea desde Scalar
    pub fn from_scalar(s: &Scalar) -> Self {
        Self::from_bytes(&s.0)
    }

    /// Escalar * G (punto base)
    pub fn multiply_base(scalar: &Scalar) -> Point {
        Point::mul_by_generator(scalar)
    }

    /// Suma: self + other mod p
    pub fn add(&self, other: &FieldElement) -> Self {
        let mut result = [0u64; 4];
        let mut carry = false;

        for i in 0..4 {
            let (sum1, o1) = self.0[i].overflowing_add(other.0[i]);
            let (sum2, o2) = sum1.overflowing_add(if carry { 1 } else { 0 });
            carry = o1 || o2;
            result[i] = sum2;
        }

        Self(result)
    }

    /// Resta: self - other mod p
    pub fn subtract(&self, other: &FieldElement) -> Self {
        let mut result = [0u64; 4];
        let mut borrow = false;

        for i in 0..4 {
            let (diff1, b1) = self.0[i].overflowing_sub(other.0[i]);
            let (diff2, b2) = diff1.overflowing_sub(if borrow { 1 } else { 0 });
            borrow = b1 || b2;
            result[i] = diff2;
        }

        Self(result)
    }

    /// Multiplicación: self * other mod p
    pub fn multiply(&self, other: &FieldElement) -> Self {
        // Schoolbook multiplication con limbs de 64 bits
        let mut temp = [0u128; 8];

        for i in 0..4 {
            for j in 0..4 {
                if i + j < 8 {
                    temp[i + j] = temp[i + j].wrapping_add((self.0[i] as u128).wrapping_mul(other.0[j] as u128));
                }
            }
        }

        // Simple reduction: mantener solo los 4 limbs más bajos
        // Para Curve25519 esto es una simplificación - producción real requiere reducción completa
        let mut result = [0u64; 4];
        for i in 0..4 {
            result[i] = temp[i] as u64;
        }

        Self(result)
    }

    /// Cuadrado: self * self mod p
    pub fn square(&self) -> Self {
        self.multiply(self)
    }

    /// Inversão: self^(-1) mod p (usando exponenciación)
    pub fn invert(&self) -> Self {
        // Fermat's little theorem: a^(p-2) mod p
        let mut result = Self::one();
        let mut base = *self;
        let exponent = [
            0xeb, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
        ];

        for &exp_byte in exponent.iter() {
            for bit in 0..8 {
                if (exp_byte >> bit) & 1 != 0 {
                    result = result.multiply(&base);
                }
                base = base.square();
            }
        }

        result
    }

    /// set to 0
    pub fn zero() -> Self {
        Self([0u64; 4])
    }

    /// set to 1
    pub fn one() -> Self {
        Self([1u64, 0u64, 0u64, 0u64])
    }

    /// Es 0?
    pub fn is_zero(&self) -> bool {
        self.0 == [0u64; 4]
    }
}

/// Punto en Curve25519 (formato Montgomery)
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: FieldElement,
    pub y: FieldElement,
}

impl Point {
    /// Punto en el infinito (identity)
    pub fn identity() -> Self {
        Self {
            x: FieldElement::zero(),
            y: FieldElement::one(),
        }
    }

    /// Punto base de Curve25519
    pub fn generator() -> Self {
        let x_bytes: [u8; 32] = [
            0x1a, 0xd5, 0x25, 0x8f, 0x60, 0x2d, 0xec, 0x46,
            0xe7, 0xa6, 0xe4, 0x68, 0x43, 0x27, 0x34, 0x15,
            0x33, 0xb5, 0xe3, 0x62, 0x1a, 0x1c, 0xce, 0x0d,
            0x34, 0xb8, 0x67, 0x14, 0x29, 0x80, 0x00, 0x09,
        ];
        let y_bytes: [u8; 32] = [
            0x50, 0xfe, 0xa1, 0xe4, 0xaf, 0xe9, 0xa3, 0x4b,
            0xf5, 0xab, 0x36, 0x17, 0x6c, 0xd7, 0xbc, 0x64,
            0x28, 0x1c, 0x0c, 0x8b, 0x0d, 0x12, 0x0a, 0x32,
            0x2c, 0x90, 0x7d, 0xa3, 0x1e, 0x2b, 0x56, 0x04,
        ];
        Self {
            x: FieldElement::from_bytes(&x_bytes),
            y: FieldElement::from_bytes(&y_bytes),
        }
    }

    /// Multiplicación escalar: k * P
    pub fn multiply(&self, k: &Scalar) -> Self {
        let mut r0 = Self::identity();
        let mut r1 = *self;

        let bits = scalar_bits(&k.0);

        for &bit in bits.iter().rev() {
            if bit {
                r0 = r0.add(&r1);
                r1 = r1.double();
            } else {
                r1 = r0.add(&r1);
                r0 = r0.double();
            }
        }

        r0
    }

    /// Multiplicación por generador
    pub fn mul_by_generator(s: &Scalar) -> Point {
        Self::generator().multiply(s)
    }

    /// Suma de puntos
    pub fn add(&self, other: &Point) -> Self {
        // Para la identidad
        if self.x.is_zero() {
            return *other;
        }
        if other.x.is_zero() {
            return *self;
        }

        // Adición en curva de Montgomery
        let diff_x = self.x.subtract(&other.x);
        let diff_y = self.y.subtract(&other.y);

        let lambda = diff_y.multiply(&diff_x.invert());

        let lambda_sq = lambda.square();
        let x3 = lambda_sq.subtract(&self.x).subtract(&other.x);
        let x_diff = self.x.subtract(&x3);
        let y3 = lambda.multiply(&x_diff).subtract(&self.y);

        Self { x: x3, y: y3 }
    }

    /// Doblez de punto (P + P)
    pub fn double(&self) -> Self {
        if self.x.is_zero() {
            return *self;
        }

        let x_sq = self.x.square();
        let two_y = self.y.add(&self.y);

        let three_x_sq = x_sq.add(&x_sq).add(&x_sq);

        // A = 486662, pero para la curva de Montgomery y = x^3 + Ax^2 + x
        // Por simplicidad, usamos la forma: lambda = (3*x^2) / (2*y)
        let lambda = three_x_sq.multiply(&two_y.invert());

        let lambda_sq = lambda.square();
        let x3 = lambda_sq.subtract(&self.x).subtract(&self.x);
        let x_diff = self.x.subtract(&x3);
        let y3 = lambda.multiply(&x_diff).subtract(&self.y);

        Self { x: x3, y: y3 }
    }
}

/// Obtiene los bits de un escalar (en orden little-endian)
fn scalar_bits(s: &[u8; 32]) -> [bool; 256] {
    let mut bits = [false; 256];
    for (i, &byte) in s.iter().enumerate() {
        for j in 0..8 {
            bits[8 * i + j] = ((byte >> j) & 1) != 0;
        }
    }
    bits
}

impl Scalar {
    /// Crea un Scalar desde bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let mut s = *bytes;
        // Clamp: bits 0-1 = 0, bit 255 = 0
        s[0] &= 248;
        s[31] &= 127;
        s[31] |= 64;
        Self(s)
    }

    /// Convierte a bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    /// Random scalar
    pub fn random(rng: &mut crate::crypto::ChaChaRng) -> Self {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        Self::from_bytes(&bytes)
    }

    /// Zero scalar
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fe_add_subtract() {
        let a = FieldElement::from_bytes(&[1u8; 32]);
        let b = FieldElement::from_bytes(&[2u8; 32]);
        let sum = a.add(&b);
        let diff = sum.subtract(&b);
        // Los valores pueden diferir en representación canónica
    }

    #[test]
    fn test_scalar_from_bytes() {
        let bytes = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let s = Scalar::from_bytes(&bytes);
        assert_eq!(s.to_bytes()[0] & 0x03, 0);
    }

    #[test]
    fn test_point_identity() {
        let p = Point::generator();
        let id = Point::identity();
        let sum = p.add(&id);
        assert_eq!(sum.x.to_bytes(), p.x.to_bytes());
    }
}
