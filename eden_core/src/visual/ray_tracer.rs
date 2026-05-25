//! # Ray Tracer - Core ray tracing implementation

#![allow(dead_code)]

use super::{Color, Hit, Ray, Scene};
/// RayTracer optimizado con varias técnicas
pub struct RayTracer {
    samples_per_pixel: u32,
    max_depth: u32,
}

impl RayTracer {
    pub fn new() -> Self {
        Self {
            samples_per_pixel: 4,
            max_depth: 5,
        }
    }

    /// Trace un rayo y retorna color
    pub fn trace(&self, scene: &Scene, ray: &Ray) -> Color {
        self.trace_recursive(scene, ray, 0)
    }

    fn trace_recursive(&self, scene: &Scene, ray: &Ray, depth: u32) -> Color {
        if depth >= self.max_depth {
            return scene.background_color;
        }

        match scene.intersect(ray) {
            Some(hit) => {
                // Color directo
                let direct_color = self.compute_direct(scene, &hit);

                // Reflexión
                let reflect_color = if hit.material.reflectivity > 0.0 {
                    let dir = RayTracer::reflect(&ray.direction, &hit.normal);
                    let reflect_ray = Ray {
                        origin: hit.position + hit.normal * 0.001,
                        direction: dir,
                        ..*ray
                    };
                    let reflected = self.trace_recursive(scene, &reflect_ray, depth + 1);
                    reflected * hit.material.reflectivity
                } else {
                    Color::black()
                };

                // Transmisión (refracción)
                let refract_color = if hit.material.transparency > 0.0 {
                    if let Some(dir) = RayTracer::refract(
                        &ray.direction,
                        &hit.normal,
                        1.0 / hit.material.refraction_index,
                    ) {
                        let refract_ray = Ray {
                            origin: hit.position - hit.normal * 0.001,
                            direction: dir,
                            ..*ray
                        };
                        self.trace_recursive(scene, &refract_ray, depth + 1)
                            * hit.material.transparency
                    } else {
                        Color::black()
                    }
                } else {
                    Color::black()
                };

                direct_color + reflect_color + refract_color
            }
            None => scene.background_color,
        }
    }

    fn compute_direct(&self, scene: &Scene, hit: &Hit) -> Color {
        let mut color = Color::black();

        for light in &scene.lights {
            let light_dir = (light.position - hit.position).normalized();
            let light_dist = (light.position - hit.position).length();

            // Shadow ray
            let shadow_ray = Ray {
                origin: hit.position + hit.normal * 0.001,
                direction: light_dir,
                ..*hit.ray
            };

            let shadowed = scene
                .intersect(&shadow_ray)
                .map(|h| h.distance < light_dist)
                .unwrap_or(false);

            if !shadowed {
                // Lambert
                let diff = hit.normal.dot(&light_dir).max(0.0);
                color = color + hit.material.color * diff * light.intensity;

                // Blinn-Phong
                let view = -hit.ray.direction;
                let half = (light_dir + view).normalized();
                let spec = hit.normal.dot(&half).max(0.0).powf(hit.material.shininess);
                color = color
                    + Color::new(1.0, 1.0, 1.0) * spec * hit.material.specular * light.intensity;
            }
        }

        color
    }

    fn reflect(dir: &super::scene::Vec3D, normal: &super::scene::Vec3D) -> super::scene::Vec3D {
        let dot = dir.dot(normal);
        *dir - *normal * (2.0 * dot)
    }

    fn refract(
        dir: &super::scene::Vec3D,
        normal: &super::scene::Vec3D,
        eta: f32,
    ) -> Option<super::scene::Vec3D> {
        let cos_i = (-*dir).dot(normal);
        let sin2_t = eta * eta * (1.0 - cos_i * cos_i);

        if sin2_t > 1.0 {
            None
        } else {
            let cos_t = (1.0 - sin2_t).sqrt();
            Some(*dir * eta + *normal * (eta * cos_i - cos_t))
        }
    }
}

impl Default for RayTracer {
    fn default() -> Self {
        Self::new()
    }
}
