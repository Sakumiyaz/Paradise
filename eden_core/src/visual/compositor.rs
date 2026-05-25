//! # Compositor - Layer composition and post-processing

#![allow(dead_code)]

use super::{Color, FrameBuffer};

/// Modo de blending
#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
}

/// Una capa visual
#[derive(Debug, Clone)]
pub struct Layer {
    pub framebuffer: FrameBuffer,
    pub opacity: f32,
    pub offset_x: i32,
    pub offset_y: i32,
}

impl Layer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            framebuffer: FrameBuffer::new(width, height),
            opacity: 1.0,
            offset_x: 0,
            offset_y: 0,
        }
    }
}

/// Compositor de capas
pub struct Compositor {
    output: FrameBuffer,
    layers: Vec<Layer>,
}

impl Compositor {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            output: FrameBuffer::new(width, height),
            layers: Vec::new(),
        }
    }

    /// Agrega una capa
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// Compone todas las capas
    pub fn compose(&mut self) -> &FrameBuffer {
        self.output.clear(Color::new(0.0, 0.0, 0.0));

        // Take layers temporarily to avoid borrow conflict
        let layers = std::mem::take(&mut self.layers);
        for layer in &layers {
            self.blend_layer(layer);
        }
        // Restore layers
        self.layers = layers;

        &self.output
    }

    fn blend_layer(&mut self, layer: &Layer) {
        let (w, h) = self.output.dimensions();

        for y in 0..h {
            for x in 0..w {
                // Calcular coordenadas en la capa
                let src_x = x as i32 - layer.offset_x;
                let src_y = y as i32 - layer.offset_y;

                if src_x >= 0
                    && (src_x as usize) < layer.framebuffer.width
                    && src_y >= 0
                    && (src_y as usize) < layer.framebuffer.height
                {
                    if let Some(src_color) =
                        layer.framebuffer.get_pixel(src_x as usize, src_y as usize)
                    {
                        let _alpha = src_color.a * layer.opacity;
                        let bg_color = self.output.get_pixel(x, y).unwrap_or(Color::black());

                        let blended = src_color.blend(&bg_color);
                        self.output.set_pixel(x, y, blended);
                    }
                }
            }
        }
    }

    /// Aplica post-procesamiento
    pub fn apply_post_process(&mut self, effect: PostEffect) {
        match effect {
            PostEffect::Blur(radius) => self.gaussian_blur(radius),
            PostEffect::Sharpen(factor) => self.sharpen(factor),
            PostEffect::Brightness(amount) => self.brightness(amount),
            PostEffect::Contrast(amount) => self.contrast(amount),
            PostEffect::Saturation(amount) => self.saturation(amount),
            PostEffect::Vignette(intensity) => self.vignette(intensity),
            PostEffect::Grain(amount) => self.grain(amount),
            PostEffect::ChromaticAberration(amount) => self.chromatic_aberration(amount),
        }
    }

    fn gaussian_blur(&mut self, radius: usize) {
        let (w, h) = self.output.dimensions();
        let mut temp = FrameBuffer::new(w as u32, h as u32);

        // Kernel gaussiano 3x3
        let kernel = Self::gaussian_kernel(radius);
        let k_size = kernel.len();
        let k_half = k_size / 2;

        // Pasada horizontal
        for y in 0..h {
            for x in 0..w {
                let mut r = 0.0;
                let mut g = 0.0;
                let mut b = 0.0;
                let mut a = 0.0;

                for ky in 0..k_size {
                    let sy = (y as i32 + ky as i32 - k_half as i32).clamp(0, h as i32 - 1) as usize;
                    for kx in 0..k_size {
                        let sx =
                            (x as i32 + kx as i32 - k_half as i32).clamp(0, w as i32 - 1) as usize;
                        if let Some(c) = self.output.get_pixel(sx, sy) {
                            let weight = kernel[ky][kx];
                            r += c.r * weight;
                            g += c.g * weight;
                            b += c.b * weight;
                            a += c.a * weight;
                        }
                    }
                }

                temp.set_pixel(x, y, Color { r, g, b, a });
            }
        }

        // Copiar de vuelta
        for y in 0..h {
            for x in 0..w {
                if let Some(c) = temp.get_pixel(x, y) {
                    self.output.set_pixel(x, y, c);
                }
            }
        }
    }

    fn gaussian_kernel(radius: usize) -> Vec<Vec<f32>> {
        let size = 2 * radius + 1;
        let sigma = radius as f32 / 2.0;
        let mut kernel = vec![vec![0.0; size]; size];

        let mut sum = 0.0;
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - radius as f32;
                let dy = y as f32 - radius as f32;
                let exp_val = -(dx * dx + dy * dy) / (2.0 * sigma * sigma);
                kernel[y][x] = exp_val.exp();
                sum += kernel[y][x];
            }
        }

        // Normalizar
        for y in 0..size {
            for x in 0..size {
                kernel[y][x] /= sum;
            }
        }

        kernel
    }

    fn sharpen(&mut self, factor: f32) {
        let (w, h) = self.output.dimensions();

        // Kernel sharpen
        let kernel = [[-1.0, -1.0, -1.0], [-1.0, 9.0, -1.0], [-1.0, -1.0, -1.0]];

        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let mut color = Color::black();

                for ky in 0..3 {
                    for kx in 0..3 {
                        if let Some(c) = self.output.get_pixel(x + kx - 1, y + ky - 1) {
                            color = color + c * kernel[ky][kx];
                        }
                    }
                }

                // Mezclar con original
                if let Some(orig) = self.output.get_pixel(x, y) {
                    let mixed = orig * (1.0 - factor) + color * factor;
                    self.output.set_pixel(x, y, mixed);
                }
            }
        }
    }

    fn brightness(&mut self, amount: f32) {
        for y in 0..self.output.dimensions().1 {
            for x in 0..self.output.dimensions().0 {
                if let Some(c) = self.output.get_pixel(x, y) {
                    self.output.set_pixel(
                        x,
                        y,
                        Color {
                            r: (c.r + amount).clamp(0.0, 1.0),
                            g: (c.g + amount).clamp(0.0, 1.0),
                            b: (c.b + amount).clamp(0.0, 1.0),
                            a: c.a,
                        },
                    );
                }
            }
        }
    }

    fn contrast(&mut self, amount: f32) {
        let factor = (259.0 * (amount + 255.0)) / (255.0 * (259.0 - amount));

        for y in 0..self.output.dimensions().1 {
            for x in 0..self.output.dimensions().0 {
                if let Some(c) = self.output.get_pixel(x, y) {
                    self.output.set_pixel(
                        x,
                        y,
                        Color {
                            r: ((factor * (c.r - 0.5) + 0.5).clamp(0.0, 1.0)),
                            g: ((factor * (c.g - 0.5) + 0.5).clamp(0.0, 1.0)),
                            b: ((factor * (c.b - 0.5) + 0.5).clamp(0.0, 1.0)),
                            a: c.a,
                        },
                    );
                }
            }
        }
    }

    fn saturation(&mut self, amount: f32) {
        for y in 0..self.output.dimensions().1 {
            for x in 0..self.output.dimensions().0 {
                if let Some(c) = self.output.get_pixel(x, y) {
                    let gray = 0.299 * c.r + 0.587 * c.g + 0.114 * c.b;
                    self.output.set_pixel(
                        x,
                        y,
                        Color {
                            r: (gray + amount * (c.r - gray)).clamp(0.0, 1.0),
                            g: (gray + amount * (c.g - gray)).clamp(0.0, 1.0),
                            b: (gray + amount * (c.b - gray)).clamp(0.0, 1.0),
                            a: c.a,
                        },
                    );
                }
            }
        }
    }

    fn vignette(&mut self, intensity: f32) {
        let (w, h) = self.output.dimensions();
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        let max_dist = (cx * cx + cy * cy).sqrt();

        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist = (dx * dx + dy * dy).sqrt() / max_dist;
                let factor = 1.0 - intensity * dist * dist;

                if let Some(c) = self.output.get_pixel(x, y) {
                    self.output.set_pixel(
                        x,
                        y,
                        Color {
                            r: c.r * factor,
                            g: c.g * factor,
                            b: c.b * factor,
                            a: c.a,
                        },
                    );
                }
            }
        }
    }

    fn grain(&mut self, amount: f32) {
        use std::time::SystemTime;
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u32;

        let (w, h) = self.output.dimensions();
        let mut rng = seed;

        for y in 0..h {
            for x in 0..w {
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                let noise = ((rng >> 16) & 0xFF) as f32 / 127.5 - 1.0;

                if let Some(c) = self.output.get_pixel(x, y) {
                    self.output.set_pixel(
                        x,
                        y,
                        Color {
                            r: (c.r + noise * amount).clamp(0.0, 1.0),
                            g: (c.g + noise * amount).clamp(0.0, 1.0),
                            b: (c.b + noise * amount).clamp(0.0, 1.0),
                            a: c.a,
                        },
                    );
                }
            }
        }
    }

    fn chromatic_aberration(&mut self, amount: f32) {
        let (w, h) = self.output.dimensions();
        let cx = w / 2;
        let cy = h / 2;

        let mut temp = FrameBuffer::new(w as u32, h as u32);

        for y in 0..h {
            for x in 0..w {
                let dx = (x as i32 - cx as i32) as f32;
                let dy = (y as i32 - cy as i32) as f32;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let offset = amount * dist / 100.0;

                // Canal rojo compensado
                let src_r = (x as f32 + dx / dist * offset) as i32;
                let src_b = (x as f32 - dx / dist * offset) as i32;

                let r = if src_r >= 0 && (src_r as usize) < w {
                    self.output.get_pixel(src_r as usize, y).map(|c| c.r)
                } else {
                    None
                };

                let b = if src_b >= 0 && (src_b as usize) < w {
                    self.output.get_pixel(src_b as usize, y).map(|c| c.b)
                } else {
                    None
                };

                if let Some(orig) = self.output.get_pixel(x, y) {
                    temp.set_pixel(
                        x,
                        y,
                        Color {
                            r: r.unwrap_or(orig.r),
                            g: orig.g,
                            b: b.unwrap_or(orig.b),
                            a: orig.a,
                        },
                    );
                }
            }
        }

        // Copiar de vuelta
        for y in 0..h {
            for x in 0..w {
                if let Some(c) = temp.get_pixel(x, y) {
                    self.output.set_pixel(x, y, c);
                }
            }
        }
    }

    /// Obtener output
    pub fn get_output(&self) -> &FrameBuffer {
        &self.output
    }
}

/// Efectos de post-procesamiento
#[derive(Debug, Clone, Copy)]
pub enum PostEffect {
    Blur(usize),
    Sharpen(f32),
    Brightness(f32),
    Contrast(f32),
    Saturation(f32),
    Vignette(f32),
    Grain(f32),
    ChromaticAberration(f32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vignette() {
        let mut comp = Compositor::new(100, 100);
        comp.apply_post_process(PostEffect::Vignette(0.5));
        // No crash = pass
    }
}
