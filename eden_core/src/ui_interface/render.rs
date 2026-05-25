//! # Render - Vector Rendering Engine
//!
//! Motor de renderizado vectorial 2D/3D 100% original.
//! Sin dependencias de GPU - todo calculado en CPU.
//!
//! ## Primitive Types
//!
//! - Point, Line, Rect, Circle, Path, Text
//! - Transformaciones afínes (translate, rotate, scale)
//! - Blend modes (normal, multiply, screen, overlay)
//! - Clipping regions
//!
//! ## Usage
//!
//! ```rust
//! use eden_core::ui_interface::render::{Color, Rect, VectorRenderer};
//!
//! let mut renderer = VectorRenderer::new(800, 600);
//! renderer.set_color(Color::rgb(255, 0, 0));
//! renderer.draw_rect(Rect::new(100.0, 100.0, 200.0, 100.0));
//! let buffer = renderer.finish();
//! assert_eq!(buffer.width, 800);
//! ```

#![allow(dead_code)]

// ============================================================================
// TIPOS BÁSICOS
// ============================================================================

/// Punto 2D
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn distance_to(self, other: Point) -> f32 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }

    pub fn lerp(self, other: Point, t: f32) -> Point {
        Point::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

/// Tamaño 2D
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn area(self) -> f32 {
        self.width * self.height
    }
}

/// Rectángulo
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn from_points(p1: Point, p2: Point) -> Self {
        Self {
            origin: Point::new(p1.x.min(p2.x), p1.y.min(p2.y)),
            size: Size::new((p2.x - p1.x).abs(), (p2.y - p1.y).abs()),
        }
    }

    pub fn zero() -> Self {
        Self {
            origin: Point::zero(),
            size: Size::zero(),
        }
    }

    pub fn center(self) -> Point {
        Point::new(
            self.origin.x + self.size.width / 2.0,
            self.origin.y + self.size.height / 2.0,
        )
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.x <= self.origin.x + self.size.width
            && point.y >= self.origin.y
            && point.y <= self.origin.y + self.size.height
    }

    pub fn intersects(self, other: Rect) -> bool {
        self.origin.x < other.origin.x + other.size.width
            && self.origin.x + self.size.width > other.origin.x
            && self.origin.y < other.origin.y + other.size.height
            && self.origin.y + self.size.height > other.origin.y
    }

    pub fn intersection(self, other: Rect) -> Rect {
        let x1 = self.origin.x.max(other.origin.x);
        let y1 = self.origin.y.max(other.origin.y);
        let x2 = (self.origin.x + self.size.width).min(other.origin.x + other.size.width);
        let y2 = (self.origin.y + self.size.height).min(other.origin.y + other.size.height);
        Rect::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
    }
}

/// Color RGBA
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn black() -> Self {
        Self::rgb(0, 0, 0)
    }
    pub fn white() -> Self {
        Self::rgb(255, 255, 255)
    }
    pub fn red() -> Self {
        Self::rgb(255, 0, 0)
    }
    pub fn green() -> Self {
        Self::rgb(0, 255, 0)
    }
    pub fn blue() -> Self {
        Self::rgb(0, 0, 255)
    }
    pub fn yellow() -> Self {
        Self::rgb(255, 255, 0)
    }
    pub fn cyan() -> Self {
        Self::rgb(0, 255, 255)
    }
    pub fn magenta() -> Self {
        Self::rgb(255, 0, 255)
    }
    pub fn transparent() -> Self {
        Self::rgba(0, 0, 0, 0)
    }

    pub fn with_alpha(self, alpha: f32) -> Self {
        Self { a: alpha, ..self }
    }

    pub fn lerp(self, other: Color, t: f32) -> Color {
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    pub fn blend(self, other: Color, mode: BlendMode) -> Color {
        let a_u8 = (self.a * 255.0) as u8;
        match mode {
            BlendMode::Normal => self.lerp(other, other.a),
            BlendMode::Multiply => Color::rgba(
                (self.r * other.r * 255.0) as u8,
                (self.g * other.g * 255.0) as u8,
                (self.b * other.b * 255.0) as u8,
                a_u8,
            ),
            BlendMode::Screen => Color::rgba(
                ((1.0 - (1.0 - self.r) * (1.0 - other.r)) * 255.0) as u8,
                ((1.0 - (1.0 - self.g) * (1.0 - other.g)) * 255.0) as u8,
                ((1.0 - (1.0 - self.b) * (1.0 - other.b)) * 255.0) as u8,
                a_u8,
            ),
            BlendMode::Overlay => {
                let blend_component = |s: f32, o: f32| -> f32 {
                    if s < 0.5 {
                        2.0 * s * o
                    } else {
                        1.0 - 2.0 * (1.0 - s) * (1.0 - o)
                    }
                };
                Color::rgba(
                    (blend_component(self.r, other.r) * 255.0) as u8,
                    (blend_component(self.g, other.g) * 255.0) as u8,
                    (blend_component(self.b, other.b) * 255.0) as u8,
                    a_u8,
                )
            }
            BlendMode::Add => Color::rgba(
                (self.r.min(1.0) + other.r.min(1.0 - self.r) * other.a) as u8,
                (self.g.min(1.0) + other.g.min(1.0 - self.g) * other.a) as u8,
                (self.b.min(1.0) + other.b.min(1.0 - self.b) * other.a) as u8,
                a_u8,
            ),
        }
    }
}

/// Modos de blend
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
}

// ============================================================================
// TRANSFORMACIONES
// ============================================================================

/// Transformación afín 2D
#[derive(Clone, Debug, PartialEq)]
pub struct Transform2D {
    /// Factores de escala [scale_x, scale_y]
    pub scale: Point,
    /// Ángulo de rotación en radianes
    pub rotation: f32,
    /// Traslación [translate_x, translate_y]
    pub translation: Point,
    /// Punto de pivote para rotación y escala
    pub pivot: Point,
}

impl Transform2D {
    pub fn identity() -> Self {
        Self {
            scale: Point::new(1.0, 1.0),
            rotation: 0.0,
            translation: Point::new(0.0, 0.0),
            pivot: Point::new(0.0, 0.0),
        }
    }

    pub fn translate(self, tx: f32, ty: f32) -> Self {
        Self {
            translation: Point::new(self.translation.x + tx, self.translation.y + ty),
            ..self
        }
    }

    pub fn scale(self, sx: f32, sy: f32) -> Self {
        Self {
            scale: Point::new(self.scale.x * sx, self.scale.y * sy),
            ..self
        }
    }

    pub fn rotate(self, angle: f32) -> Self {
        Self {
            rotation: self.rotation + angle,
            ..self
        }
    }

    pub fn set_pivot(self, px: f32, py: f32) -> Self {
        Self {
            pivot: Point::new(px, py),
            ..self
        }
    }

    /// Aplica la transformación a un punto
    pub fn apply(self, point: Point) -> Point {
        // 1. Trasladar al sistema de coordenadas del pivote
        let px = point.x - self.pivot.x;
        let py = point.y - self.pivot.y;

        // 2. Rotar
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let rx = px * cos_r - py * sin_r;
        let ry = px * sin_r + py * cos_r;

        // 3. Escalar
        let sx = rx * self.scale.x;
        let sy = ry * self.scale.y;

        // 4. Trasladar de vuelta y aplicar traslación global
        Point::new(
            sx + self.pivot.x + self.translation.x,
            sy + self.pivot.y + self.translation.y,
        )
    }

    /// Calcula la transformación inversa
    pub fn inverse(self) -> Self {
        let inv_scale = Point::new(1.0 / self.scale.x, 1.0 / self.scale.y);
        let inv_rotation = -self.rotation;

        let _cos_r = inv_rotation.cos();
        let _sin_r = inv_rotation.sin();

        // Invertir traducción
        let tx = -self.translation.x;
        let ty = -self.translation.y;

        Self {
            scale: inv_scale,
            rotation: inv_rotation,
            translation: Point::new(tx, ty),
            pivot: self.pivot,
        }
    }

    /// Compose: aplica otra_transformación después de self
    pub fn compose(self, other: Transform2D) -> Self {
        Transform2D {
            scale: Point::new(self.scale.x * other.scale.x, self.scale.y * other.scale.y),
            rotation: self.rotation + other.rotation,
            translation: self.apply(other.translation),
            pivot: other.pivot,
        }
    }
}

// ============================================================================
// PRIMITIVAS
// ============================================================================

/// Primitiva geométrica
#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Point(Point),
    Line {
        start: Point,
        end: Point,
    },
    Rect(Rect),
    Circle {
        center: Point,
        radius: f32,
    },
    Arc {
        center: Point,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    },
    Path(Vec<PathCommand>),
    Text {
        position: Point,
        text: String,
        font_size: f32,
    },
    Image {
        rect: Rect,
        data: Vec<u8>,
    },
    Empty,
}

/// Comando de trazado (Bezier paths)
#[derive(Clone, Debug, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),         // Quadratic bezier
    CubicTo(Point, Point, Point), // Cubic bezier
    ArcTo(Point, Point, f32),     // Arc
    Close,
}

/// Glyph para renderizado de texto
#[derive(Clone, Debug)]
pub struct Glyph {
    pub char: char,
    pub codepoint: u32,
    pub advance: f32,
    pub bounds: Rect,
    pub path: Vec<PathCommand>,
}

// ============================================================================
// FRAMEBUFFER
// ============================================================================

/// Framebuffer simple para renderizado
#[derive(Clone, Debug)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>, // RGBA
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height * 4;
        Self {
            width,
            height,
            pixels: vec![0u8; size],
        }
    }

    pub fn clear(&mut self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) * 4;
                self.pixels[idx] = (color.r * 255.0) as u8;
                self.pixels[idx + 1] = (color.g * 255.0) as u8;
                self.pixels[idx + 2] = (color.b * 255.0) as u8;
                self.pixels[idx + 3] = (color.a * 255.0) as u8;
            }
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * 4;
            // Alpha blending simple
            let dest = Color::rgba(
                self.pixels[idx],
                self.pixels[idx + 1],
                self.pixels[idx + 2],
                self.pixels[idx + 3],
            );
            let blended = dest.blend(color, BlendMode::Normal);
            self.pixels[idx] = (blended.r * 255.0) as u8;
            self.pixels[idx + 1] = (blended.g * 255.0) as u8;
            self.pixels[idx + 2] = (blended.b * 255.0) as u8;
            self.pixels[idx + 3] = (blended.a * 255.0) as u8;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * 4;
            Color::rgba(
                self.pixels[idx],
                self.pixels[idx + 1],
                self.pixels[idx + 2],
                self.pixels[idx + 3],
            )
        } else {
            Color::transparent()
        }
    }

    /// Dibuja una línea usando algoritmo de Bresenham
    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: Color) {
        let dx = (x1 as i32 - x0 as i32).abs() as i32;
        let dy = (y1 as i32 - y0 as i32).abs() as i32;
        let sx = if (x0 as i32) < x1 as i32 { 1 } else { -1 };
        let sy = if (y0 as i32) < y1 as i32 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x0 as i32;
        let mut y = y0 as i32;

        loop {
            self.set_pixel(x as usize, y as usize, color);

            if x == x1 as i32 && y == y1 as i32 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Dibuja un rectángulo
    pub fn draw_rect(&mut self, rect: Rect, color: Color, filled: bool) {
        let x0 = rect.origin.x as usize;
        let y0 = rect.origin.y as usize;
        let x1 = (rect.origin.x + rect.size.width) as usize;
        let y1 = (rect.origin.y + rect.size.height) as usize;

        if filled {
            for y in y0..y1 {
                for x in x0..x1 {
                    self.set_pixel(x, y, color);
                }
            }
        } else {
            // Top and bottom
            for x in x0..x1 {
                self.set_pixel(x, y0, color);
                self.set_pixel(x, y1 - 1, color);
            }
            // Left and right
            for y in y0..y1 {
                self.set_pixel(x0, y, color);
                self.set_pixel(x1 - 1, y, color);
            }
        }
    }

    /// Dibuja un círculo (algoritmo midpoint)
    pub fn draw_circle(&mut self, cx: usize, cy: usize, radius: usize, color: Color, filled: bool) {
        let mut x = radius as i32;
        let mut y = 0i32;
        let mut err = 0i32;

        while x >= y {
            if filled {
                for i in -x..=x {
                    self.set_pixel((cx as i32 + i) as usize, (cy as i32 + y) as usize, color);
                    self.set_pixel((cx as i32 + i) as usize, (cy as i32 - y) as usize, color);
                    self.set_pixel((cx as i32 + i) as usize, (cy as i32 + x) as usize, color);
                    self.set_pixel((cx as i32 + i) as usize, (cy as i32 - x) as usize, color);
                    self.set_pixel((cx as i32 + i) as usize, (cy as i32 + y) as usize, color);
                    self.set_pixel((cx as i32 - i) as usize, (cy as i32 + y) as usize, color);
                    self.set_pixel((cx as i32 - i) as usize, (cy as i32 + x) as usize, color);
                    self.set_pixel((cx as i32 - i) as usize, (cy as i32 - x) as usize, color);
                }
            } else {
                self.set_pixel((cx as i32 + x) as usize, (cy as i32 + y) as usize, color);
                self.set_pixel((cx as i32 - x) as usize, (cy as i32 + y) as usize, color);
                self.set_pixel((cx as i32 + x) as usize, (cy as i32 - y) as usize, color);
                self.set_pixel((cx as i32 - x) as usize, (cy as i32 - y) as usize, color);
                self.set_pixel((cx as i32 + y) as usize, (cy as i32 + x) as usize, color);
                self.set_pixel((cx as i32 - y) as usize, (cy as i32 + x) as usize, color);
                self.set_pixel((cx as i32 + y) as usize, (cy as i32 - x) as usize, color);
                self.set_pixel((cx as i32 - y) as usize, (cy as i32 - x) as usize, color);
            }

            err += 1 + 2 * y;
            y += 1;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    /// Copia otra región del framebuffer con blending
    pub fn blit(
        &mut self,
        src: &FrameBuffer,
        dest_x: usize,
        dest_y: usize,
        src_rect: Rect,
        blend: BlendMode,
    ) {
        let sx0 = src_rect.origin.x as usize;
        let sy0 = src_rect.origin.y as usize;
        let _sx1 = (sx0 as f32 + src_rect.size.width) as usize;
        let _sy1 = (sy0 as f32 + src_rect.size.height) as usize;

        for y in 0..src_rect.size.height as usize {
            for x in 0..src_rect.size.width as usize {
                let src_x = sx0 + x;
                let src_y = sy0 + y;
                let dest_x = dest_x + x;
                let dest_y = dest_y + y;

                if src_x < src.width
                    && src_y < src.height
                    && dest_x < self.width
                    && dest_y < self.height
                {
                    let color = src.get_pixel(src_x, src_y);
                    let idx = (dest_y * self.width + dest_x) * 4;
                    let dest = Color::rgba(
                        self.pixels[idx],
                        self.pixels[idx + 1],
                        self.pixels[idx + 2],
                        self.pixels[idx + 3],
                    );
                    let blended = dest.blend(color, blend);
                    self.pixels[idx] = (blended.r * 255.0) as u8;
                    self.pixels[idx + 1] = (blended.g * 255.0) as u8;
                    self.pixels[idx + 2] = (blended.b * 255.0) as u8;
                    self.pixels[idx + 3] = (blended.a * 255.0) as u8;
                }
            }
        }
    }
}

// ============================================================================
// VECTOR RENDERER
// ============================================================================

/// Motor de renderizado vectorial
pub struct VectorRenderer {
    width: usize,
    height: usize,
    buffer: FrameBuffer,
    transform: Transform2D,
    clip_rect: Option<Rect>,
    current_color: Color,
    blend_mode: BlendMode,
}

impl VectorRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = FrameBuffer::new(width, height);
        buffer.clear(Color::transparent());
        Self {
            width,
            height,
            buffer,
            transform: Transform2D::identity(),
            clip_rect: None,
            current_color: Color::black(),
            blend_mode: BlendMode::Normal,
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn set_transform(&mut self, transform: Transform2D) {
        self.transform = transform;
    }

    pub fn set_clip(&mut self, rect: Option<Rect>) {
        self.clip_rect = rect;
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    /// Limpia el buffer
    pub fn clear(&mut self, color: Color) {
        self.buffer.clear(color);
    }

    /// Dibuja un punto
    pub fn draw_point(&mut self, point: Point) {
        let transformed = self.transform.clone().apply(point);
        let x = transformed.x as usize;
        let y = transformed.y as usize;
        self.buffer.set_pixel(x, y, self.current_color);
    }

    /// Dibuja una línea
    pub fn draw_line(&mut self, start: Point, end: Point) {
        let start_t = self.transform.clone().apply(start);
        let end_t = self.transform.clone().apply(end);

        // Algoritmo de Bresenham
        let x0 = start_t.x as i32;
        let y0 = start_t.y as i32;
        let x1 = end_t.x as i32;
        let y1 = end_t.y as i32;

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            // Check clipping
            if let Some(clip) = self.clip_rect {
                if !clip.contains(Point::new(x as f32, y as f32)) {
                    if x == x1 && y == y1 {
                        break;
                    }
                }
            }

            self.buffer
                .set_pixel(x as usize, y as usize, self.current_color);

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Dibuja un rectángulo
    pub fn draw_rect(&mut self, rect: Rect) {
        let p1 = self.transform.clone().apply(rect.origin);
        let p2 = self
            .transform
            .clone()
            .apply(Point::new(rect.origin.x + rect.size.width, rect.origin.y));
        let p3 = self
            .transform
            .clone()
            .apply(Point::new(rect.origin.x, rect.origin.y + rect.size.height));
        let p4 = self.transform.clone().apply(Point::new(
            rect.origin.x + rect.size.width,
            rect.origin.y + rect.size.height,
        ));

        self.draw_line(p1, p2);
        self.draw_line(p2, p4);
        self.draw_line(p4, p3);
        self.draw_line(p3, p1);
    }

    /// Dibuja un círculo
    pub fn draw_circle(&mut self, center: Point, radius: f32) {
        let transform = self.transform.clone();
        let center_t = transform.apply(center);
        let cx = center_t.x as i32;
        let cy = center_t.y as i32;
        let r = (radius * self.transform.scale.x) as i32;

        let mut x = r;
        let mut y = 0i32;
        let mut err = 0i32;

        while x >= y {
            let points = [
                (cx + x, cy + y),
                (cx - x, cy + y),
                (cx + x, cy - y),
                (cx - x, cy - y),
                (cx + y, cy + x),
                (cx - y, cy + x),
                (cx + y, cy - x),
                (cx - y, cy - x),
            ];

            for (px, py) in points.iter() {
                if let Some(clip) = self.clip_rect {
                    if clip.contains(Point::new(*px as f32, *py as f32)) {
                        self.buffer
                            .set_pixel(*px as usize, *py as usize, self.current_color);
                    }
                } else {
                    self.buffer
                        .set_pixel(*px as usize, *py as usize, self.current_color);
                }
            }

            err += 1 + 2 * y;
            y += 1;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    /// Dibuja un caminho (Bezier paths)
    pub fn draw_path(&mut self, commands: &[PathCommand]) {
        let mut current = Point::zero();
        let mut start = Point::zero();

        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(p) => {
                    let t = self.transform.clone();
                    current = t.apply(*p);
                    start = current;
                }
                PathCommand::LineTo(p) => {
                    let t = self.transform.clone();
                    let end = t.apply(*p);
                    self.draw_line(current, end);
                    current = end;
                }
                PathCommand::QuadTo(cp, end) => {
                    let cp_t = self.transform.clone().apply(*cp);
                    let end_t = self.transform.clone().apply(*end);
                    self.draw_quad_bezier(current, cp_t, end_t);
                    current = end_t;
                }
                PathCommand::CubicTo(cp1, cp2, end) => {
                    let cp1_t = self.transform.clone().apply(*cp1);
                    let cp2_t = self.transform.clone().apply(*cp2);
                    let end_t = self.transform.clone().apply(*end);
                    self.draw_cubic_bezier(current, cp1_t, cp2_t, end_t);
                    current = end_t;
                }
                PathCommand::Close => {
                    self.draw_line(current, start);
                    current = start;
                }
                _ => {}
            }
        }
    }

    fn draw_quad_bezier(&mut self, p0: Point, p1: Point, p2: Point) {
        // Subdividir en segmentos lineales
        let steps = 20;
        let mut prev = p0;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let t1 = 1.0 - t;
            let x = t1 * t1 * p0.x + 2.0 * t1 * t * p1.x + t * t * p2.x;
            let y = t1 * t1 * p0.y + 2.0 * t1 * t * p1.y + t * t * p2.y;
            self.draw_line(prev, Point::new(x, y));
            prev = Point::new(x, y);
        }
    }

    fn draw_cubic_bezier(&mut self, p0: Point, p1: Point, p2: Point, p3: Point) {
        let steps = 30;
        let mut prev = p0;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let t1 = 1.0 - t;
            let x = t1 * t1 * t1 * p0.x
                + 3.0 * t1 * t1 * t * p1.x
                + 3.0 * t1 * t * t * p2.x
                + t * t * t * p3.x;
            let y = t1 * t1 * t1 * p0.y
                + 3.0 * t1 * t1 * t * p1.y
                + 3.0 * t1 * t * t * p2.y
                + t * t * t * p3.y;
            self.draw_line(prev, Point::new(x, y));
            prev = Point::new(x, y);
        }
    }

    /// Dibuja texto simple (mono-spaced)
    pub fn draw_text(&mut self, position: Point, text: &str, font_size: f32) {
        let pos = self.transform.clone().apply(position);
        let char_w = font_size * 0.6;
        let char_h = font_size;

        for (i, c) in text.chars().enumerate() {
            let x = (pos.x + i as f32 * char_w) as usize;
            let y = pos.y as usize;

            // Draw simple bitmap for each character
            self.draw_char_simple(x, y, c, char_h);
        }
    }

    fn draw_char_simple(&mut self, x: usize, y: usize, ch: char, _height: f32) {
        // Simple 5x7 bitmap para caracteres básicos ASCII
        let bitmap: [u8; 7] = match ch {
            'A' => [
                0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
            'B' => [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
            ],
            'C' => [
                0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
            ],
            'D' => [
                0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
            ],
            'E' => [
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
            ],
            'F' => [
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
            'G' => [
                0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
            ],
            'H' => [
                0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
            'I' => [
                0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
            ],
            'J' => [
                0b00111, 0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b01110,
            ],
            'K' => [
                0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
            ],
            'L' => [
                0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
            ],
            'M' => [
                0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
            ],
            'N' => [
                0b10001, 0b11001, 0b10101, 0b10101, 0b10011, 0b10001, 0b10001,
            ],
            'O' => [
                0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
            'P' => [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
            'Q' => [
                0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
            ],
            'R' => [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
            ],
            'S' => [
                0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110,
            ],
            'T' => [
                0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
            'U' => [
                0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
            'V' => [
                0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
            ],
            'W' => [
                0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
            ],
            'X' => [
                0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
            ],
            'Y' => [
                0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
            'Z' => [
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
            ],
            '0' => [
                0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
            '1' => [
                0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
            ],
            '2' => [
                0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111,
            ],
            '3' => [
                0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110,
            ],
            '4' => [
                0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
            ],
            '5' => [
                0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b10001, 0b01110,
            ],
            '6' => [
                0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
            ],
            '7' => [
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
            ],
            '8' => [
                0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
            ],
            '9' => [
                0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
            ],
            ' ' => [
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
            '.' => [
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
            ],
            '!' => [
                0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
            ],
            _ => [
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
        };

        for row in 0..7 {
            let bits = bitmap[row];
            for col in 0..5 {
                if (bits & (0b10000 >> col)) != 0 {
                    self.buffer.set_pixel(x + col, y + row, self.current_color);
                }
            }
        }
    }

    /// Finaliza el renderizado y devuelve el framebuffer
    pub fn finish(self) -> FrameBuffer {
        self.buffer
    }

    /// Obtiene referencia al buffer durante el renderizado
    pub fn buffer(&self) -> &FrameBuffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut FrameBuffer {
        &mut self.buffer
    }
}

// ============================================================================
// FUNCIONES helper
// ============================================================================

pub fn vector_render_point(renderer: &mut VectorRenderer, point: Point) {
    renderer.draw_point(point);
}

pub fn vector_render_line(renderer: &mut VectorRenderer, start: Point, end: Point) {
    renderer.draw_line(start, end);
}

pub fn vector_render_rect(renderer: &mut VectorRenderer, rect: Rect, filled: bool) {
    if filled {
        // Fill using scanline
        for y in 0..rect.size.height as usize {
            let y_coord = rect.origin.y + y as f32;
            renderer.draw_line(
                Point::new(rect.origin.x, y_coord),
                Point::new(rect.origin.x + rect.size.width, y_coord),
            );
        }
    } else {
        renderer.draw_rect(rect);
    }
}

pub fn vector_render_circle(
    renderer: &mut VectorRenderer,
    center: Point,
    radius: f32,
    filled: bool,
) {
    if filled {
        // Fill using scanline
        let r = radius as i32;
        let cx = center.x as i32;
        let cy = center.y as i32;
        let color = renderer.current_color;

        for y in -r..=r {
            let width = ((radius.powi(2) - (y as f32).powi(2)).sqrt()) as i32;
            for x in -width..=width {
                renderer
                    .buffer_mut()
                    .set_pixel((cx + x) as usize, (cy + y) as usize, color);
            }
        }
    } else {
        renderer.draw_circle(center, radius);
    }
}

pub fn vector_render_path(renderer: &mut VectorRenderer, commands: &[PathCommand]) {
    renderer.draw_path(commands);
}

pub fn vector_render_text(
    renderer: &mut VectorRenderer,
    position: Point,
    text: &str,
    font_size: f32,
) {
    renderer.draw_text(position, text, font_size);
}
