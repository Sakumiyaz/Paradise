//! # Renderer - Main rendering engine
//!
//! Motor de renderizado principal que coordina ray tracing y rasterización.

#![allow(dead_code)]

use super::{Color, FrameBuffer, Hit, Ray, Scene};
/// Modo de renderizado
#[derive(Debug, Clone, Copy)]
pub enum RenderMode {
    RayTrace, // Ray tracing completo
    Raster,   // Rasterización rápida
    Hybrid,   // Híbrido
}

/// Opciones de renderizado
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    pub max_bounces: u32,
    pub shadow_rays: u32,
    pub anti_aliasing: u32, // 1 = none, 2 = 4x, 3 = 8x
    pub ambient_occlusion: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            max_bounces: 4,
            shadow_rays: 1,
            anti_aliasing: 2,
            ambient_occlusion: true,
        }
    }
}

/// Renderer principal
pub struct Renderer {
    options: RenderOptions,
    framebuffer: FrameBuffer,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            options: RenderOptions {
                width,
                height,
                ..Default::default()
            },
            framebuffer: FrameBuffer::new(width, height),
        }
    }

    pub fn with_options(options: RenderOptions) -> Self {
        Self {
            framebuffer: FrameBuffer::new(options.width, options.height),
            options,
        }
    }

    /// Renderiza una escena completa
    pub fn render(&mut self, scene: &Scene) -> &FrameBuffer {
        let samples_per_pixel = self.options.anti_aliasing as usize;

        for y in 0..self.options.height {
            for x in 0..self.options.width {
                let mut color = Color::black();

                // Super sampling para anti-aliasing
                for sy in 0..samples_per_pixel {
                    for sx in 0..samples_per_pixel {
                        let u = (x as f32 + sx as f32 / samples_per_pixel as f32)
                            / self.options.width as f32;
                        let v = 1.0
                            - (y as f32 + sy as f32 / samples_per_pixel as f32)
                                / self.options.height as f32;

                        let ray = scene.camera.generate_ray(u, v);
                        let sample_color = self.trace_ray(scene, &ray, 0);
                        color = color + sample_color;
                    }
                }

                // Promedio de samples
                let num_samples = (samples_per_pixel * samples_per_pixel) as f32;
                color = color * (1.0 / num_samples);

                self.framebuffer.set_pixel(x as usize, y as usize, color);
            }
        }

        &self.framebuffer
    }

    /// Trace un rayo individual con recursion
    fn trace_ray(&self, scene: &Scene, ray: &Ray, depth: u32) -> Color {
        if depth > self.options.max_bounces {
            return scene.background_color;
        }

        // Encontrar intersección más cercana
        if let Some(hit) = scene.intersect(ray) {
            // Iluminación directa
            let direct = self.compute_lighting(scene, &hit);

            // Reflexiones
            let mut reflected = Color::black();
            if hit.material.reflectivity > 0.0 && depth < self.options.max_bounces {
                let reflect_dir = Self::reflect(&ray.direction, &hit.normal);
                let reflect_ray = Ray {
                    origin: hit.position,
                    direction: reflect_dir,
                    ..*ray
                };
                reflected = self.trace_ray(scene, &reflect_ray, depth + 1);
            }

            // Reflexión difusa (subsurface scattering simplificado)
            let mut refracted = Color::black();
            if hit.material.transparency > 0.0 {
                let refract_dir = Self::refract(
                    &ray.direction,
                    &hit.normal,
                    1.0 / hit.material.refraction_index,
                );
                if let Some(dir) = refract_dir {
                    let refract_ray = Ray {
                        origin: hit.position,
                        direction: dir,
                        ..*ray
                    };
                    refracted = self.trace_ray(scene, &refract_ray, depth + 1);
                }
            }

            // Combinar
            let reflectivity = hit.material.reflectivity;
            let transparency = hit.material.transparency;
            let base = direct;
            let refl = reflected * reflectivity;
            let refr = refracted * transparency;

            return base + refl + refr;
        }

        scene.background_color
    }

    /// Computa iluminación directa (Phong + sombras)
    fn compute_lighting(&self, scene: &Scene, hit: &Hit) -> Color {
        let mut total_light = Color::new(0.05, 0.05, 0.1); // Luz ambiental base

        for light in &scene.lights {
            let light_dir = (light.position - hit.position).normalized();

            // Sombras
            let shadow_ray = Ray {
                origin: hit.position + hit.normal * 0.001,
                direction: light_dir,
                ..*hit.ray
            };

            let in_shadow = scene.intersect(&shadow_ray).is_some();

            if !in_shadow || !self.options.shadow_rays > 0 {
                // Diffuse (Lambert)
                let diff = hit.normal.dot(&light_dir).max(0.0);
                let diffuse = hit.material.color * diff * light.intensity;

                // Specular (Phong)
                let view_dir = -hit.ray.direction;
                let reflect_dir = Self::reflect(&light_dir, &hit.normal);
                let spec = view_dir
                    .dot(&reflect_dir)
                    .max(0.0)
                    .powf(hit.material.shininess);
                let specular = Color::new(1.0, 1.0, 1.0) * spec * hit.material.specular;

                total_light = total_light + diffuse + specular;
            }
        }

        total_light
    }

    /// Calcula dirección de reflexión
    fn reflect(dir: &super::scene::Vec3D, normal: &super::scene::Vec3D) -> super::scene::Vec3D {
        let dot = dir.dot(normal);
        *dir - (*normal * (2.0 * dot))
    }

    /// Calcula dirección de refracción (Snell's law)
    fn refract(
        dir: &super::scene::Vec3D,
        normal: &super::scene::Vec3D,
        eta: f32,
    ) -> Option<super::scene::Vec3D> {
        let cos_theta = (-*dir).dot(normal);
        let sen2_theta = eta * eta * (1.0 - cos_theta * cos_theta);

        if sen2_theta > 1.0 {
            None // Total internal reflection
        } else {
            let cos_theta_2 = (1.0 - sen2_theta).sqrt();
            Some(*dir * eta + *normal * (eta * cos_theta - cos_theta_2))
        }
    }

    /// Obtiene el framebuffer
    pub fn get_framebuffer(&self) -> &FrameBuffer {
        &self.framebuffer
    }

    /// Exporta a PNG (devuelve raw bytes RGBA)
    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        self.framebuffer.to_rgba_bytes()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_reflect() {
        use crate::visual::{Renderer, Vec3D};
        let dir = Vec3D::new(0.0, -1.0, 0.0);
        let normal = Vec3D::new(0.0, 1.0, 0.0);
        let reflected = Renderer::reflect(&dir, &normal);
        assert!((reflected.y - 1.0).abs() < 0.01);
    }
}
