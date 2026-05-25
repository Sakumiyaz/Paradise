//! # Scene - 3D scene representation

#![allow(dead_code)]

use super::{Color, Material};
/// Vector 3D básico
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn dot(&self, other: &Vec3D) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3D) -> Vec3D {
        Vec3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(&self) -> Vec3D {
        let len = self.length();
        if len == 0.0 {
            Self::zero()
        } else {
            *self * (1.0 / len)
        }
    }
}

impl std::ops::Add for Vec3D {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Vec3D::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Sub for Vec3D {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Vec3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Sub<&Vec3D> for Vec3D {
    type Output = Self;
    fn sub(self, other: &Vec3D) -> Self {
        Vec3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Sub<Vec3D> for &Vec3D {
    type Output = Vec3D;
    fn sub(self, other: Vec3D) -> Vec3D {
        Vec3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Neg for Vec3D {
    type Output = Self;
    fn neg(self) -> Self {
        Vec3D::new(-self.x, -self.y, -self.z)
    }
}

impl std::ops::Mul<f32> for Vec3D {
    type Output = Self;
    fn mul(self, factor: f32) -> Self {
        Vec3D::new(self.x * factor, self.y * factor, self.z * factor)
    }
}

/// Rayo para ray tracing
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec3D,
    pub direction: Vec3D,
}

impl Ray {
    pub fn new(origin: Vec3D, direction: Vec3D) -> Self {
        Self {
            origin,
            direction: direction.normalized(),
        }
    }

    /// Punto en el rayo a distancia t
    pub fn at(&self, t: f32) -> Vec3D {
        self.origin + self.direction * t
    }
}

/// Información de intersección
#[derive(Debug, Clone)]
pub struct Hit<'a> {
    pub position: Vec3D,
    pub normal: Vec3D,
    pub distance: f32,
    pub material: Material,
    pub ray: &'a Ray,
}

/// Luz en la escena
#[derive(Debug, Clone)]
pub struct Light {
    pub position: Vec3D,
    pub intensity: f32,
    pub color: Color,
}

impl Light {
    pub fn new(x: f32, y: f32, z: f32, intensity: f32) -> Self {
        Self {
            position: Vec3D::new(x, y, z),
            intensity,
            color: Color::white(),
        }
    }
}

/// Cámara para renderizado
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3D,
    pub look_at: Vec3D,
    pub up: Vec3D,
    pub fov: f32, // Field of view en grados
}

impl Camera {
    pub fn new(position: Vec3D, look_at: Vec3D) -> Self {
        Self {
            position,
            look_at,
            up: Vec3D::new(0.0, 1.0, 0.0),
            fov: 60.0,
        }
    }

    /// Genera un rayo para coordenadas normalizadas (u, v) [0, 1]
    pub fn generate_ray(&self, u: f32, v: f32) -> Ray {
        let forward = (self.look_at - self.position).normalized();
        let right = forward.cross(&self.up).normalized();
        let up = right.cross(&forward);

        let aspect_ratio = 16.0 / 9.0;
        let half_fov = (self.fov / 2.0).to_radians();
        let fov_tan = half_fov.tan();

        let x = (2.0 * u - 1.0) * aspect_ratio * fov_tan;
        let y = (1.0 - 2.0 * v) * fov_tan;

        let direction = (forward + right * x + up * y).normalized();

        Ray::new(self.position, direction)
    }
}

/// Objeto en la escena
#[derive(Debug, Clone)]
pub enum SceneObject {
    Sphere {
        center: Vec3D,
        radius: f32,
        material: Material,
    },
    Plane {
        point: Vec3D,
        normal: Vec3D,
        material: Material,
    },
    Box {
        min: Vec3D,
        max: Vec3D,
        material: Material,
    },
}

impl SceneObject {
    /// Intersecciona con un rayo
    pub fn intersect<'a>(&self, ray: &'a Ray) -> Option<Hit<'a>> {
        match self {
            SceneObject::Sphere {
                center,
                radius,
                material,
            } => {
                let oc = ray.origin - *center;
                let a = ray.direction.dot(&ray.direction);
                let b = 2.0 * oc.dot(&ray.direction);
                let c = oc.dot(&oc) - radius * radius;

                let discriminant = b * b - 4.0 * a * c;

                if discriminant < 0.0 {
                    None
                } else {
                    let t = (-b - discriminant.sqrt()) / (2.0 * a);
                    if t > 0.001 {
                        let position = ray.at(t);
                        let normal = (position - *center).normalized();
                        Some(Hit {
                            position,
                            normal,
                            distance: t,
                            material: material.clone(),
                            ray,
                        })
                    } else {
                        None
                    }
                }
            }
            SceneObject::Plane {
                point,
                normal,
                material,
            } => {
                let denom = normal.dot(&ray.direction);
                if denom.abs() < 0.0001 {
                    None
                } else {
                    let t = (point - ray.origin).dot(normal) / denom;
                    if t > 0.001 {
                        Some(Hit {
                            position: ray.at(t),
                            normal: *normal,
                            distance: t,
                            material: material.clone(),
                            ray,
                        })
                    } else {
                        None
                    }
                }
            }
            SceneObject::Box { min, max, material } => {
                // Ray-box intersection usando slab method
                let txmin = (min.x - ray.origin.x) / ray.direction.x;
                let txmax = (max.x - ray.origin.x) / ray.direction.x;
                let tymin = (min.y - ray.origin.y) / ray.direction.y;
                let tymax = (max.y - ray.origin.y) / ray.direction.y;
                let tzmin = (min.z - ray.origin.z) / ray.direction.z;
                let tzmax = (max.z - ray.origin.z) / ray.direction.z;

                let tmin = txmin.max(tymin).max(tzmin);
                let tmax = txmax.min(tymax).min(tzmax);

                if tmax < tmin || tmax < 0.001 {
                    None
                } else {
                    let t = if tmin > 0.001 { tmin } else { tmax };
                    let position = ray.at(t);

                    // Determinar normal basada en qué cara fue golpeada
                    let eps = 0.001;
                    let normal = if (position.x - min.x).abs() < eps {
                        Vec3D::new(-1.0, 0.0, 0.0)
                    } else if (position.x - max.x).abs() < eps {
                        Vec3D::new(1.0, 0.0, 0.0)
                    } else if (position.y - min.y).abs() < eps {
                        Vec3D::new(0.0, -1.0, 0.0)
                    } else if (position.y - max.y).abs() < eps {
                        Vec3D::new(0.0, 1.0, 0.0)
                    } else if (position.z - min.z).abs() < eps {
                        Vec3D::new(0.0, 0.0, -1.0)
                    } else {
                        Vec3D::new(0.0, 0.0, 1.0)
                    };

                    Some(Hit {
                        position,
                        normal,
                        distance: t,
                        material: material.clone(),
                        ray,
                    })
                }
            }
        }
    }
}

/// Escena completa
#[derive(Debug, Clone)]
pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub lights: Vec<Light>,
    pub camera: Camera,
    pub background_color: Color,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            objects: Vec::new(),
            lights: Vec::new(),
            camera,
            background_color: Color::new(0.05, 0.05, 0.1),
        }
    }

    /// Agrega un objeto
    pub fn add(&mut self, object: SceneObject) {
        self.objects.push(object);
    }

    /// Agrega una luz
    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    /// Encuentra la intersección más cercana
    pub fn intersect<'a>(&self, ray: &'a Ray) -> Option<Hit<'a>> {
        let mut closest = None;
        let mut min_dist = f32::INFINITY;

        for object in &self.objects {
            if let Some(hit) = object.intersect(ray) {
                if hit.distance < min_dist {
                    min_dist = hit.distance;
                    closest = Some(hit);
                }
            }
        }

        closest
    }

    /// Crea escena simple de demo
    pub fn demo() -> Self {
        let camera = Camera::new(Vec3D::new(0.0, 2.0, 5.0), Vec3D::new(0.0, 0.0, 0.0));

        let mut scene = Scene::new(camera);

        // Esfera roja
        scene.add(SceneObject::Sphere {
            center: Vec3D::new(0.0, 0.0, 0.0),
            radius: 1.0,
            material: Material {
                color: Color::red(),
                reflectivity: 0.3,
                transparency: 0.0,
                refraction_index: 1.0,
                shininess: 32.0,
                specular: 0.5,
            },
        });

        // Plano verde
        scene.add(SceneObject::Plane {
            point: Vec3D::new(0.0, -1.0, 0.0),
            normal: Vec3D::new(0.0, 1.0, 0.0),
            material: Material {
                color: Color::new(0.2, 0.8, 0.2),
                reflectivity: 0.1,
                transparency: 0.0,
                refraction_index: 1.0,
                shininess: 8.0,
                specular: 0.2,
            },
        });

        // Luz
        scene.add_light(Light::new(5.0, 5.0, 5.0, 1.0));
        scene.add_light(Light::new(-5.0, 3.0, 2.0, 0.5));

        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_intersect() {
        let sphere = SceneObject::Sphere {
            center: Vec3D::new(0.0, 0.0, 0.0),
            radius: 1.0,
            material: Material::default(),
        };

        let ray = Ray::new(Vec3D::new(0.0, 0.0, 5.0), Vec3D::new(0.0, 0.0, -1.0));

        let hit = sphere.intersect(&ray);
        assert!(hit.is_some());
    }

    #[test]
    fn test_vec3d() {
        let a = Vec3D::new(1.0, 0.0, 0.0);
        let b = Vec3D::new(0.0, 1.0, 0.0);
        assert!((a.cross(&b).z - 1.0).abs() < 0.001);
    }
}
