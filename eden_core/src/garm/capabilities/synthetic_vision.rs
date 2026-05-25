// EDEN GARM SyntheticVision — Generacion procedural de bitmaps con captions.
// 100% Rust puro, 0 LLM, 0 red.
//
// Genera escenas simples (formas geometricas en grid de bytes) emparejadas con
// texto descriptivo. Usado para entrenar grounding cross-modal text<->vision:
// EDEN ve la imagen, lee el caption, y aprende que palabras corresponden a
// que patrones visuales.
//
// SceneType:
//   - FallingObject: forma centrada en X que cae verticalmente -> motion blur Y
//   - MovingObject: forma que se mueve horizontalmente -> motion blur X
//   - TwoObjects: dos formas en posiciones diferentes
//   - StaticObject: forma estatica en posicion aleatoria
//   - GrowingObject: forma que crece (multiples copias concentricas)

use crate::eden_garm::capabilities::vision::ImageBuffer;

#[derive(Clone, Debug, PartialEq)]
pub enum SceneType {
    FallingObject,
    MovingObject,
    TwoObjects,
    StaticObject,
    GrowingObject,
    StackedObjects,
    ChasingObjects,
    NestedObjects,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Circle,
    Square,
    Triangle,
    Diamond,
}

#[derive(Clone, Debug)]
pub struct SyntheticScene {
    pub image: ImageBuffer,
    pub caption: String,
    pub scene_type: SceneType,
    pub shape: Shape,
    pub position_label: String, // "left", "right", "center", "top", "bottom"
    pub motion_label: String,   // "falls", "moves", "grows", "static"
}

pub struct SyntheticVision {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub n_generated: u64,
}

impl SyntheticVision {
    pub fn new(width: u32, height: u32) -> Self {
        SyntheticVision {
            width,
            height,
            seed: 1,
            n_generated: 0,
        }
    }

    /// LCG random for reproducible scenes.
    fn rand(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223) & 0xFFFFFFFF;
        (self.seed & 0xFFFFFFFF) as u32
    }

    fn rand_range(&mut self, lo: u32, hi: u32) -> u32 {
        if hi <= lo {
            return lo;
        }
        lo + self.rand() % (hi - lo)
    }

    fn pick_shape(&mut self) -> Shape {
        match self.rand() % 4 {
            0 => Shape::Circle,
            1 => Shape::Square,
            2 => Shape::Triangle,
            _ => Shape::Diamond,
        }
    }

    /// Draw a filled shape with intensity 220 (bright) at (cx, cy) with radius r.
    fn draw_shape(
        &self,
        img: &mut ImageBuffer,
        cx: i32,
        cy: i32,
        r: i32,
        shape: &Shape,
        intensity: u8,
    ) {
        let w = img.width as i32;
        let h = img.height as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                let inside = match shape {
                    Shape::Circle => dx * dx + dy * dy <= r * r,
                    Shape::Square => dx.abs() <= r && dy.abs() <= r,
                    Shape::Triangle => {
                        // Triangle pointing up: y from -r (top) to +r (bottom), |x| <= scaled
                        let half_width = ((r - dy) * r) / (2 * r).max(1);
                        dx.abs() <= half_width && dy >= -r && dy <= r
                    }
                    Shape::Diamond => dx.abs() + dy.abs() <= r,
                };
                if inside {
                    let x = cx + dx;
                    let y = cy + dy;
                    if x >= 0 && x < w && y >= 0 && y < h {
                        img.pixels[(y * w + x) as usize] = intensity;
                    }
                }
            }
        }
    }

    /// Generate a scene of the given type. Returns image + caption + metadata.
    pub fn generate_scene(&mut self, scene_type: SceneType) -> SyntheticScene {
        self.n_generated += 1;
        let pixels = vec![20u8; (self.width * self.height) as usize]; // dark background
        let mut img = ImageBuffer {
            width: self.width,
            height: self.height,
            pixels,
        };
        let shape = self.pick_shape();
        let r = self.rand_range(4, 10) as i32;
        let w = self.width as i32;
        let h = self.height as i32;

        let (caption, position_label, motion_label) = match scene_type {
            SceneType::FallingObject => {
                let cx = self.rand_range(15, (self.width - 15).max(15)) as i32;
                // motion blur: draw shape at multiple Y positions with decreasing intensity
                for (i, y_off) in [-12, -6, 0].iter().enumerate() {
                    let intensity = 80 + (i as u8) * 70;
                    self.draw_shape(&mut img, cx, h / 2 + y_off, r, &shape, intensity);
                }
                let pos = if cx < w / 3 {
                    "left"
                } else if cx > 2 * w / 3 {
                    "right"
                } else {
                    "center"
                };
                let cap = format!(
                    "a {} object falls from the {} position",
                    Self::shape_name(&shape),
                    pos
                );
                (cap, pos.to_string(), "falls".to_string())
            }
            SceneType::MovingObject => {
                let cy = self.rand_range(10, (self.height - 10).max(10)) as i32;
                // horizontal motion blur
                for (i, x_off) in [-12, -6, 0].iter().enumerate() {
                    let intensity = 80 + (i as u8) * 70;
                    self.draw_shape(&mut img, w / 2 + x_off, cy, r, &shape, intensity);
                }
                let pos = if cy < h / 3 {
                    "top"
                } else if cy > 2 * h / 3 {
                    "bottom"
                } else {
                    "middle"
                };
                let cap = format!(
                    "a {} object moves through the {} of the scene",
                    Self::shape_name(&shape),
                    pos
                );
                (cap, pos.to_string(), "moves".to_string())
            }
            SceneType::TwoObjects => {
                let shape_b = match self.rand() % 4 {
                    0 => Shape::Circle,
                    1 => Shape::Square,
                    2 => Shape::Triangle,
                    _ => Shape::Diamond,
                };
                let cx_a = self.rand_range(8, (w / 2 - 5) as u32) as i32;
                let cx_b = self.rand_range((w / 2 + 5) as u32, (self.width - 8).max(20)) as i32;
                let cy_a = self.rand_range(10, (self.height - 10).max(15)) as i32;
                let cy_b = self.rand_range(10, (self.height - 10).max(15)) as i32;
                self.draw_shape(&mut img, cx_a, cy_a, r, &shape, 200);
                self.draw_shape(&mut img, cx_b, cy_b, r, &shape_b, 200);
                let cap = format!(
                    "a {} on the left and a {} on the right",
                    Self::shape_name(&shape),
                    Self::shape_name(&shape_b)
                );
                (cap, "left and right".to_string(), "static".to_string())
            }
            SceneType::StaticObject => {
                let cx = self.rand_range(8, (self.width - 8).max(15)) as i32;
                let cy = self.rand_range(8, (self.height - 8).max(15)) as i32;
                self.draw_shape(&mut img, cx, cy, r, &shape, 220);
                let pos = if cx < w / 3 {
                    "left"
                } else if cx > 2 * w / 3 {
                    "right"
                } else {
                    "center"
                };
                let cap = format!(
                    "a static {} object on the {}",
                    Self::shape_name(&shape),
                    pos
                );
                (cap, pos.to_string(), "static".to_string())
            }
            SceneType::GrowingObject => {
                let cx = (w / 2) as i32;
                let cy = (h / 2) as i32;
                // concentric rings to suggest growth
                for (i, scale) in [3, 6, 9, 12].iter().enumerate() {
                    let intensity = 60 + (i as u8) * 50;
                    self.draw_shape(&mut img, cx, cy, *scale, &shape, intensity);
                }
                let cap = format!(
                    "a {} object grows in the center of the scene",
                    Self::shape_name(&shape)
                );
                (cap, "center".to_string(), "grows".to_string())
            }
            SceneType::StackedObjects => {
                // Two objects vertically stacked, one above the other
                let cx = (w / 2) as i32;
                let r2 = self.rand_range(4, 8) as i32;
                let shape_top = self.pick_shape();
                self.draw_shape(&mut img, cx, h / 3, r2, &shape_top, 220);
                self.draw_shape(&mut img, cx, 2 * h / 3, r, &shape, 220);
                let cap = format!(
                    "a {} above a {} stacked vertically",
                    Self::shape_name(&shape_top),
                    Self::shape_name(&shape)
                );
                (cap, "stacked".to_string(), "static".to_string())
            }
            SceneType::ChasingObjects => {
                // Two objects with motion blur, one trailing the other
                let cy = (h / 2) as i32;
                let shape_b = self.pick_shape();
                // leader at right
                for (i, x_off) in [-12, -6, 0].iter().enumerate() {
                    let intensity = 80 + (i as u8) * 70;
                    self.draw_shape(&mut img, 3 * w / 4 + x_off, cy, r, &shape, intensity);
                }
                // chaser to the left, also moving
                for (i, x_off) in [-12, -6, 0].iter().enumerate() {
                    let intensity = 60 + (i as u8) * 60;
                    self.draw_shape(&mut img, w / 4 + x_off, cy, r, &shape_b, intensity);
                }
                let cap = format!(
                    "a {} chases a {} from the left to the right",
                    Self::shape_name(&shape_b),
                    Self::shape_name(&shape)
                );
                (cap, "horizontal".to_string(), "chases".to_string())
            }
            SceneType::NestedObjects => {
                // Inner shape inside outer larger shape (concentric)
                let cx = (w / 2) as i32;
                let cy = (h / 2) as i32;
                let r_outer = self.rand_range(10, 16) as i32;
                let r_inner = (r_outer / 3).max(2);
                let shape_inner = self.pick_shape();
                self.draw_shape(&mut img, cx, cy, r_outer, &shape, 140);
                self.draw_shape(&mut img, cx, cy, r_inner, &shape_inner, 230);
                let cap = format!(
                    "a {} inside a larger {} container",
                    Self::shape_name(&shape_inner),
                    Self::shape_name(&shape)
                );
                (cap, "nested".to_string(), "static".to_string())
            }
        };

        SyntheticScene {
            image: img,
            caption,
            scene_type,
            shape,
            position_label,
            motion_label,
        }
    }

    fn shape_name(s: &Shape) -> &'static str {
        match s {
            Shape::Circle => "circle",
            Shape::Square => "square",
            Shape::Triangle => "triangle",
            Shape::Diamond => "diamond",
        }
    }

    /// Convenience: pick a random scene type and generate.
    pub fn generate_random(&mut self) -> SyntheticScene {
        let t = match self.rand() % 8 {
            0 => SceneType::FallingObject,
            1 => SceneType::MovingObject,
            2 => SceneType::TwoObjects,
            3 => SceneType::GrowingObject,
            4 => SceneType::StackedObjects,
            5 => SceneType::ChasingObjects,
            6 => SceneType::NestedObjects,
            _ => SceneType::StaticObject,
        };
        self.generate_scene(t)
    }

    pub fn status(&self) -> String {
        format!(
            "SyntheticVision | {}x{} | n_generated={}",
            self.width, self.height, self.n_generated
        )
    }
}
