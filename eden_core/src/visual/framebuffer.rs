//! # FrameBuffer - Pixel buffer management

#![allow(dead_code)]

use super::Color;

/// Framebuffer con pixel buffer
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pixels: Vec<u32>, // BGRA packed
    depth_buffer: Vec<f32>,
}

impl FrameBuffer {
    /// Crea framebuffer nuevo
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width: width as usize,
            height: height as usize,
            pixels: vec![0xFF000000; size], // Negro con alpha 1.0
            depth_buffer: vec![f32::INFINITY; size],
        }
    }

    /// Setea un pixel
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx = y * self.width + x;
        self.pixels[idx] = color.to_u32();
    }

    /// Obtiene un pixel
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = y * self.width + x;
        Some(Color::from_u32(self.pixels[idx]))
    }

    /// Setea depth buffer para un pixel
    pub fn set_depth(&mut self, x: usize, y: usize, depth: f32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let idx = y * self.width + x;
        if depth < self.depth_buffer[idx] {
            self.depth_buffer[idx] = depth;
            true
        } else {
            false
        }
    }

    /// Limpia el framebuffer
    pub fn clear(&mut self, color: Color) {
        let packed = color.to_u32();
        for pixel in &mut self.pixels {
            *pixel = packed;
        }
        for depth in &mut self.depth_buffer {
            *depth = f32::INFINITY;
        }
    }

    /// Limpia solo el depth buffer
    pub fn clear_depth(&mut self) {
        for depth in &mut self.depth_buffer {
            *depth = f32::INFINITY;
        }
    }

    /// Convierte a RGBA bytes (para PNG encoding)
    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 4);

        for &pixel in &self.pixels {
            // BGRA -> RGBA
            bytes.push((pixel & 0xFF) as u8); // B
            bytes.push(((pixel >> 8) & 0xFF) as u8); // G
            bytes.push(((pixel >> 16) & 0xFF) as u8); // R
            bytes.push(((pixel >> 24) & 0xFF) as u8); // A
        }

        bytes
    }

    /// Desde RGBA bytes
    pub fn from_rgba_bytes(&mut self, bytes: &[u8], width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.pixels.clear();
        self.depth_buffer = vec![f32::INFINITY; width * height];

        for chunk in bytes.chunks(4) {
            if chunk.len() >= 4 {
                let b = chunk[0] as u32;
                let g = chunk[1] as u32;
                let r = chunk[2] as u32;
                let a = chunk[3] as u32;
                self.pixels.push((a << 24) | (r << 16) | (g << 8) | b);
            }
        }
    }

    /// Aplica gamma correction a todo el framebuffer
    pub fn gamma_correct(&mut self, gamma: f32) {
        for pixel in &mut self.pixels {
            let color = Color::from_u32(*pixel);
            let corrected = color.gamma_correct(gamma);
            *pixel = corrected.to_u32();
        }
    }

    /// Resolución
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Dibuja una línea (Bresenham)
    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: Color) {
        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x0 >= 0 && (x0 as usize) < self.width && y0 >= 0 && (y0 as usize) < self.height {
                self.set_pixel(x0 as usize, y0 as usize, color);
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Dibuja un rectángulo
    pub fn draw_rect(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: Color,
        filled: bool,
    ) {
        if filled {
            for dy in 0..h {
                for dx in 0..w {
                    self.set_pixel(x + dx, y + dy, color);
                }
            }
        } else {
            // Bordes
            self.draw_line(x, y, x + w, y, color);
            self.draw_line(x + w, y, x + w, y + h, color);
            self.draw_line(x + w, y + h, x, y + h, color);
            self.draw_line(x, y + h, x, y, color);
        }
    }

    /// Dibuja un círculo
    pub fn draw_circle(&mut self, cx: usize, cy: usize, radius: usize, color: Color, filled: bool) {
        let cx = cx as i32;
        let cy = cy as i32;
        let r = radius as i32;
        let mut x = r;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            if filled {
                for dx in -x..=x {
                    let px = cx + dx;
                    self.set_pixel(px as usize, (cy + x).max(0) as usize, color);
                    self.set_pixel(px as usize, (cy - x).max(0) as usize, color);
                    self.set_pixel(px as usize, (cy + y).max(0) as usize, color);
                    self.set_pixel(px as usize, (cy - y).max(0) as usize, color);
                }
            } else {
                self.set_pixel((cx + x) as usize, (cy + y).max(0) as usize, color);
                self.set_pixel((cx - x) as usize, (cy + y).max(0) as usize, color);
                self.set_pixel((cx + x) as usize, (cy - y).max(0) as usize, color);
                self.set_pixel((cx - x) as usize, (cy - y).max(0) as usize, color);
                self.set_pixel((cx + y) as usize, (cy + x).max(0) as usize, color);
                self.set_pixel((cx - y) as usize, (cy + x).max(0) as usize, color);
                self.set_pixel((cx + y) as usize, (cy - x).max(0) as usize, color);
                self.set_pixel((cx - y) as usize, (cy - x).max(0) as usize, color);
            }

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear() {
        let mut fb = FrameBuffer::new(100, 100);
        fb.set_pixel(50, 50, Color::red());
        fb.clear(Color::blue());
        assert_eq!(fb.get_pixel(50, 50), Some(Color::blue()));
    }

    #[test]
    fn test_line() {
        let mut fb = FrameBuffer::new(100, 100);
        fb.draw_line(0, 0, 50, 50, Color::green());
        assert!(fb.get_pixel(0, 0).is_some());
    }
}
