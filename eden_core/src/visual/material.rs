//! # Material - Surface materials for rendering

#![allow(dead_code)]

use super::Color;

/// Material de superficie
#[derive(Debug, Clone)]
pub struct Material {
    pub color: Color,
    pub reflectivity: f32,     // 0.0 = mate, 1.0 = espejo perfecto
    pub transparency: f32,     // 0.0 = opaco, 1.0 = completamente transparente
    pub refraction_index: f32, // Índice de refracción (1.0 = vacío, 1.33 = agua, 1.5 = vidrio)
    pub shininess: f32,        // Exponente para specular (Phong)
    pub specular: f32,         // Intensidad de reflexiones especulares
}

impl Material {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            reflectivity: 0.0,
            transparency: 0.0,
            refraction_index: 1.0,
            shininess: 32.0,
            specular: 0.5,
        }
    }

    /// Material tipo metal
    pub fn metal(color: Color) -> Self {
        Self {
            color,
            reflectivity: 0.8,
            transparency: 0.0,
            refraction_index: 1.0,
            shininess: 64.0,
            specular: 0.9,
        }
    }

    /// Material tipo vidrio
    pub fn glass(color: Color) -> Self {
        Self {
            color,
            reflectivity: 0.1,
            transparency: 0.9,
            refraction_index: 1.5,
            shininess: 128.0,
            specular: 1.0,
        }
    }

    /// Material tipo agua
    pub fn water() -> Self {
        Self {
            color: Color::new(0.7, 0.9, 1.0),
            reflectivity: 0.2,
            transparency: 0.8,
            refraction_index: 1.33,
            shininess: 64.0,
            specular: 0.8,
        }
    }

    /// Material tipo espejo
    pub fn mirror() -> Self {
        Self {
            color: Color::white(),
            reflectivity: 1.0,
            transparency: 0.0,
            refraction_index: 1.0,
            shininess: 128.0,
            specular: 1.0,
        }
    }

    /// Material tipo plástico
    pub fn plastic(color: Color) -> Self {
        Self {
            color,
            reflectivity: 0.3,
            transparency: 0.0,
            refraction_index: 1.0,
            shininess: 16.0,
            specular: 0.4,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: Color::gray(0.5),
            reflectivity: 0.0,
            transparency: 0.0,
            refraction_index: 1.0,
            shininess: 32.0,
            specular: 0.5,
        }
    }
}

impl Color {
    /// Color gris
    pub fn gray(g: f32) -> Self {
        Self::new(g, g, g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glass() {
        let glass = Material::glass(Color::blue());
        assert!(glass.transparency > 0.8);
    }
}
