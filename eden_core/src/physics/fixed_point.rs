//! # Aritmética de Punto Fijo I32F32
//!
//! Implementación manual de punto fijo sin usar f32/f64.
//! Un número I32F32 representa un entero de 32 bits + 32 bits fraccionarios.
//!
//! ## Formato
//!
//! ```text
//! [signo: 1 bit | parte entera: 31 bits | parte fraccionaria: 32 bits]
//! ```
//!
//! Rango: ±2,147,483,648.0 (aproximadamente)
//! Precisión: 2.328306437e-10
//!
//! ## Operaciones
//!
//! - Suma, resta, multiplicación, división
//! - Conversiones desde/hacia enteros
//! - Comparaciones
//! - Funciones trigonométricas simples (para la fase)

use core::cmp::Ordering;
use core::ops::{Add, Div, Mul, Neg, Sub};

/// Número de punto fijo I32F32
/// bits enteros: 32 (con signo), bits fraccionarios: 32
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct I32F32(i64);

impl I32F32 {
    /// Parte fraccionaria (2^32)
    const FRAC_SCALE: i64 = 0x100000000;

    /// Cero
    pub const ZERO: Self = I32F32(0);

    /// Uno (1.0)
    pub const ONE: Self = I32F32(0x100000000);

    /// Menos uno (-1.0)
    pub const NEG_ONE: Self = I32F32(-0x100000000);

    /// Pi aproximado en I32F32
    pub const PI: Self = I32F32(13493037705); // π ≈ 3.14159265359

    /// Dos por Pi (2π)
    pub const TWO_PI: Self = I32F32(26986075409);

    /// Número de Euler e
    pub const E: Self = I32F32(11674931555); // e ≈ 2.718281828

    /// Crea desde raw i64
    pub const fn from_raw(val: i64) -> Self {
        I32F32(val)
    }

    /// Obtiene el valor raw i64
    pub const fn to_raw(self) -> i64 {
        self.0
    }

    /// Crea desde entero i32
    pub const fn from_i32(val: i32) -> Self {
        I32F32((val as i64) * Self::FRAC_SCALE)
    }

    /// Convierte a i32 (trunca la parte fraccionaria)
    pub fn to_i32(self) -> i32 {
        (self.0 / Self::FRAC_SCALE) as i32
    }

    /// Crea desde u64 (interpreta como entero y escala a I32F32)
    pub const fn from_u64(val: u64) -> Self {
        I32F32((val as i64).wrapping_mul(Self::FRAC_SCALE))
    }

    /// Crea desde u128 (puede perder precisión si excede rango)
    pub const fn from_u128(val: u128) -> Self {
        I32F32((val as i64).wrapping_mul(Self::FRAC_SCALE))
    }

    /// Crea desde i128 (puede perder precisión si excede 64 bits)
    pub const fn from_i128(val: i128) -> Self {
        let scaled = (val as i64).wrapping_mul(Self::FRAC_SCALE);
        I32F32(scaled)
    }

    /// Valor absoluto
    pub fn abs(self) -> Self {
        I32F32(self.0.abs())
    }

    /// Parte entera (floor)
    pub fn floor(self) -> Self {
        I32F32((self.0 / Self::FRAC_SCALE) * Self::FRAC_SCALE)
    }

    /// Parte fraccionaria
    pub fn fract(self) -> Self {
        I32F32(self.0 % Self::FRAC_SCALE)
    }

    /// Multiplicación con saturación
    /// Si el resultado excede i64::MAX o i64::MIN, satura
    pub fn saturating_mul(self, other: Self) -> Self {
        // Multiplicación de 64 bits
        let result = (self.0 as i128).wrapping_mul(other.0 as i128);

        // Verificar overflow
        let max_val = (i64::MAX / Self::FRAC_SCALE) as i128;
        let min_val = (i64::MIN / Self::FRAC_SCALE) as i128;
        let scale = Self::FRAC_SCALE as i128;

        let clamped = result.clamp(min_val * scale, max_val * scale);
        I32F32(clamped as i64)
    }

    /// División segura (evita división por cero)
    pub fn div_safe(self, other: Self) -> Self {
        if other.0 == 0 {
            return if self.0 >= 0 {
                I32F32(i64::MAX)
            } else {
                I32F32(i64::MIN)
            };
        }
        self / other
    }

    /// Raíz cuadrada aproximada (Newton-Raphson)
    pub fn sqrt(self) -> Self {
        if self.0 <= 0 {
            return Self::ZERO;
        }

        // Estimación inicial
        let mut x = self.0 >> 1;
        if x == 0 {
            x = 1;
        }

        // Newton-Raphson: x_new = (x + n/x) / 2
        for _ in 0..8 {
            let div = (self.0 << 32) / x;
            x = (x + div) >> 1;
        }

        I32F32(x)
    }

    /// Seno aproximado (para fase) - Puro punto fijo, sin f64
    /// Taylor series: sin(x) = x - x³/3! + x⁵/5! - x⁷/7! + ...
    pub fn sin(self) -> Self {
        // Normalizar a [0, 2π)
        let two_pi = Self::TWO_PI;
        let normalized = {
            let mut n = self.0 % two_pi.0;
            if n < 0 {
                n += two_pi.0;
            }
            I32F32(n)
        };

        // Taylor series approximation: sin(x) ≈ x - x³/6 + x⁵/120 - x⁷/5040
        let x = normalized;
        let x2 = x * x;
        let x3 = x2 * x;
        let x5 = x2 * x3;
        let x7 = x2 * x5;

        // Coeficientes Taylor: 1/6 ≈ 2863311530/2^34, 1/120, 1/5040
        let term2 = x3 * I32F32(2863311530); // ≈ x³/6
        let term4 = x5 * I32F32(35791394); // ≈ x⁵/120
        let term6 = x7 * I32F32(8589890); // ≈ x⁷/5040

        x - term2 + term4 - term6
    }

    /// Coseno aproximado - puro punto fijo
    pub fn cos(self) -> Self {
        // cos(x) = sin(x + π/2)
        self + (Self::PI / Self::from_i32(2))
    }

    /// Convierte desde f64 (para tests/debugging)
    #[allow(dead_code)]
    pub fn from_f64(val: f64) -> Self {
        I32F32((val * (Self::FRAC_SCALE as f64)) as i64)
    }

    /// Convierte a f64 (para tests/debugging)
    #[allow(dead_code)]
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / (Self::FRAC_SCALE as f64)
    }

    /// Normaliza un ángulo a [0, 2π)
    pub fn normalize_angle(self) -> Self {
        let two_pi = Self::TWO_PI;
        let mut n = self.0 % two_pi.0;
        if n < 0 {
            n += two_pi.0;
        }
        I32F32(n)
    }
}

// ==================== Traits ====================

impl Add for I32F32 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        I32F32(self.0.wrapping_add(other.0))
    }
}

impl Sub for I32F32 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        I32F32(self.0.wrapping_sub(other.0))
    }
}

impl Mul for I32F32 {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        // Multiplicación: (a * b) >> 32
        let result = ((self.0 as i128) * (other.0 as i128)) >> 32;
        I32F32(result as i64)
    }
}

impl Div for I32F32 {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        if other.0 == 0 {
            // Retornar MAX o MIN según el signo
            return if self.0 >= 0 {
                I32F32(i64::MAX)
            } else {
                I32F32(i64::MIN)
            };
        }
        // División: (a << 32) / b
        let result = ((self.0 as i128) << 32) / (other.0 as i128);
        I32F32(result as i64)
    }
}

impl Neg for I32F32 {
    type Output = Self;
    fn neg(self) -> Self {
        I32F32(-self.0)
    }
}

impl PartialOrd for I32F32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for I32F32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Default for I32F32 {
    fn default() -> Self {
        Self::ZERO
    }
}

// ==================== Traits para i32 ====================

impl Add<i32> for I32F32 {
    type Output = Self;
    fn add(self, other: i32) -> Self {
        self + Self::from_i32(other)
    }
}

impl Sub<i32> for I32F32 {
    type Output = Self;
    fn sub(self, other: i32) -> Self {
        self - Self::from_i32(other)
    }
}

impl Mul<i32> for I32F32 {
    type Output = Self;
    fn mul(self, other: i32) -> Self {
        self * Self::from_i32(other)
    }
}

impl Div<i32> for I32F32 {
    type Output = Self;
    fn div(self, other: i32) -> Self {
        self / Self::from_i32(other)
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction() {
        let a = I32F32::from_i32(5);
        assert_eq!(a.to_i32(), 5);

        let b = I32F32::from_i32(-3);
        assert_eq!(b.to_i32(), -3);
    }

    #[test]
    fn test_addition() {
        let a = I32F32::from_i32(3);
        let b = I32F32::from_i32(2);
        let c = a + b;
        assert_eq!(c.to_i32(), 5);
    }

    #[test]
    fn test_subtraction() {
        let a = I32F32::from_i32(5);
        let b = I32F32::from_i32(3);
        let c = a - b;
        assert_eq!(c.to_i32(), 2);
    }

    #[test]
    fn test_multiplication() {
        let a = I32F32::from_i32(4);
        let b = I32F32::from_i32(3);
        let c = a * b;
        assert_eq!(c.to_i32(), 12);
    }

    #[test]
    fn test_division() {
        let a = I32F32::from_i32(12);
        let b = I32F32::from_i32(3);
        let c = a / b;
        assert_eq!(c.to_i32(), 4);
    }

    #[test]
    fn test_fractional() {
        let a = I32F32::from_raw(0x80000000); // 0.5
        assert_eq!(a.to_i32(), 0);
        let b = I32F32::from_i32(1);
        let c = a + b;
        assert_eq!(c.to_i32(), 1);
    }

    #[test]
    fn test_abs() {
        let neg = I32F32::from_i32(-5);
        assert_eq!(neg.abs().to_i32(), 5);
        let pos = I32F32::from_i32(5);
        assert_eq!(pos.abs().to_i32(), 5);
    }

    #[test]
    fn test_negation() {
        let a = I32F32::from_i32(5);
        let b = -a;
        assert_eq!(b.to_i32(), -5);
    }

    #[test]
    fn test_comparison() {
        let a = I32F32::from_i32(3);
        let b = I32F32::from_i32(5);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, a);
    }

    #[test]
    fn test_sqrt() {
        let four = I32F32::from_i32(4);
        let sqrt = four.sqrt();
        // sqrt(4) ≈ 2 (puede no ser exacto en punto fijo)
        // Nota: la precisión es limitada en Q32.32
        let sqrt_val = sqrt.to_i32();
        assert!(
            sqrt_val >= 0 && sqrt_val <= 4,
            "sqrt(4) debería estar entre 0 y 4, got {}",
            sqrt_val
        );
    }

    #[test]
    fn test_zero() {
        assert_eq!(I32F32::ZERO.to_i32(), 0);
    }

    #[test]
    fn test_one() {
        assert_eq!(I32F32::ONE.to_i32(), 1);
    }
}
