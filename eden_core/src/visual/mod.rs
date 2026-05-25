//! # VISUAL - Rendering System
//!
//! Sistema de renderizado visual avanzado que deja a Vision atrás.
//! GPU software con ray tracing, framebuffers, compositor.
//!
//! ## Componentes
//!
//! - **Renderer**: Motor de renderizado principal
//! - **RayTracer**: Ray tracing básico para escenas 3D
//! - **FrameBuffer**: Buffer de pixels hacia display
//! - **Compositor**: Composición de capas visuales

#![allow(dead_code)]

mod compositor;
mod framebuffer;
mod material;
mod ray_tracer;
mod renderer;
mod scene;

pub use compositor::Compositor;
pub use framebuffer::FrameBuffer;
pub use material::Material;
pub use ray_tracer::RayTracer;
pub use renderer::Renderer;
pub use scene::{Camera, Hit, Ray, Scene, Vec3D};

/// Color RGBA básico
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn with_alpha(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
    pub fn red() -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
    pub fn green() -> Self {
        Self {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }
    pub fn blue() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }

    /// Color desde HSV
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self::new(r + m, g + m, b + m)
    }

    /// Blend con otro color (alpha compositing)
    pub fn blend(&self, other: &Color) -> Color {
        let out_alpha = self.a + other.a * (1.0 - self.a);
        if out_alpha == 0.0 {
            return Color::black();
        }

        let out_r = (self.r * self.a + other.r * other.a * (1.0 - self.a)) / out_alpha;
        let out_g = (self.g * self.a + other.g * other.a * (1.0 - self.a)) / out_alpha;
        let out_b = (self.b * self.a + other.b * other.a * (1.0 - self.a)) / out_alpha;

        Color {
            r: out_r,
            g: out_g,
            b: out_b,
            a: out_alpha,
        }
    }

    /// A gamma correction
    pub fn gamma_correct(&self, gamma: f32) -> Color {
        let inv_gamma = 1.0 / gamma;
        Color {
            r: self.r.powf(inv_gamma),
            g: self.g.powf(inv_gamma),
            b: self.b.powf(inv_gamma),
            a: self.a,
        }
    }

    /// Convierte a packed u32 (BGRA)
    pub fn to_u32(&self) -> u32 {
        let r = (self.r.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (self.g.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (self.b.clamp(0.0, 1.0) * 255.0) as u8;
        let a = (self.a.clamp(0.0, 1.0) * 255.0) as u8;
        (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32
    }

    /// Desde u32 (BGRA)
    pub fn from_u32(c: u32) -> Self {
        let a = ((c >> 24) & 0xFF) as f32 / 255.0;
        let r = ((c >> 16) & 0xFF) as f32 / 255.0;
        let g = ((c >> 8) & 0xFF) as f32 / 255.0;
        let b = (c & 0xFF) as f32 / 255.0;
        Self { r, g, b, a }
    }
}

impl std::ops::Add for Color {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a,
        }
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;
    fn mul(self, factor: f32) -> Self {
        Color {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
            a: self.a,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_blend() {
        let c1 = Color::with_alpha(1.0, 0.0, 0.0, 0.5);
        let c2 = Color::new(0.0, 0.0, 1.0);
        let blended = c1.blend(&c2);
        assert!((blended.r - 0.5).abs() < f32::EPSILON);
        assert!((blended.b - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hsv() {
        let red = Color::from_hsv(0.0, 1.0, 1.0);
        assert!((red.r - 1.0).abs() < 0.01);
    }
}
