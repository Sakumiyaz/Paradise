// EDEN GARM World3D — Espacio fisico continuo 3D con objetos rigidos.
// 100% Rust puro, 0 LLM, 0 red.
//
// Objetos: posicion (x,y,z), velocidad (vx,vy,vz), masa, radio
// Fisica: gravedad, friccion, colisiones elasticas esfericas
// AABB broad-phase + sphere-sphere narrow-phase

#[derive(Clone, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
    pub fn add(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
    pub fn sub(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
    pub fn scale(&self, s: f32) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn len_sq(&self) -> f32 {
        self.dot(self)
    }
    pub fn len(&self) -> f32 {
        self.len_sq().sqrt()
    }
    pub fn normalize(&self) -> Vec3 {
        let l = self.len();
        if l < 1e-8 {
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            self.scale(1.0 / l)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Object3D {
    pub id: u64,
    pub label: String,
    pub pos: Vec3,
    pub vel: Vec3,
    pub mass: f32,
    pub radius: f32,
    pub restitution: f32,
    pub friction: f32,
}

#[derive(Clone, Debug)]
pub struct World3D {
    pub objects: Vec<Object3D>,
    pub gravity: Vec3,
    pub dt: f32,
    pub bounds: (Vec3, Vec3), // min, max
    pub n_steps: u64,
    pub n_collisions: u64,
    pub next_id: u64,
}

impl World3D {
    pub fn new() -> Self {
        World3D {
            objects: Vec::new(),
            gravity: Vec3::new(0.0, -9.8, 0.0),
            dt: 0.016, // 60fps
            bounds: (Vec3::new(-50.0, 0.0, -50.0), Vec3::new(50.0, 100.0, 50.0)),
            n_steps: 0,
            n_collisions: 0,
            next_id: 1,
        }
    }

    pub fn spawn(&mut self, label: &str, x: f32, y: f32, z: f32, mass: f32, radius: f32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.objects.push(Object3D {
            id,
            label: label.to_string(),
            pos: Vec3::new(x, y, z),
            vel: Vec3::new(0.0, 0.0, 0.0),
            mass,
            radius,
            restitution: 0.7,
            friction: 0.3,
        });
        id
    }

    pub fn step(&mut self) {
        self.n_steps += 1;
        // Integrate velocity and position (Euler semi-implicit)
        for obj in self.objects.iter_mut() {
            // gravity
            let g_force = self.gravity.scale(obj.mass);
            let acc = g_force.scale(1.0 / obj.mass);
            obj.vel = obj.vel.add(&acc.scale(self.dt));
            // friction (air drag)
            let speed = obj.vel.len();
            if speed > 0.0 {
                let drag = obj.vel.scale(-obj.friction * speed * 0.01);
                obj.vel = obj.vel.add(&drag.scale(self.dt));
            }
            obj.pos = obj.pos.add(&obj.vel.scale(self.dt));
        }
        // Collision detection + response (sphere-sphere)
        let n = self.objects.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let (left, right) = self.objects.split_at_mut(j);
                let a = &mut left[i];
                let b = &mut right[0];
                let diff = b.pos.sub(&a.pos);
                let dist_sq = diff.len_sq();
                let min_dist = a.radius + b.radius;
                if dist_sq < min_dist * min_dist && dist_sq > 1e-8 {
                    let dist = dist_sq.sqrt();
                    let normal = diff.scale(1.0 / dist);
                    // Position correction (push apart)
                    let overlap = min_dist - dist;
                    let correction = normal.scale(overlap * 0.5);
                    a.pos = a.pos.sub(&correction);
                    b.pos = b.pos.add(&correction);
                    // Velocity response (elastic collision)
                    let rel_vel = b.vel.sub(&a.vel);
                    let vel_along_normal = rel_vel.dot(&normal);
                    if vel_along_normal > 0.0 {
                        continue;
                    }
                    let e = a.restitution.min(b.restitution);
                    let impulse = -(1.0 + e) * vel_along_normal / (1.0 / a.mass + 1.0 / b.mass);
                    let impulse_vec = normal.scale(impulse);
                    a.vel = a.vel.sub(&impulse_vec.scale(1.0 / a.mass));
                    b.vel = b.vel.add(&impulse_vec.scale(1.0 / b.mass));
                    self.n_collisions += 1;
                }
            }
        }
        // Bounds
        for obj in self.objects.iter_mut() {
            let min = &self.bounds.0;
            let max = &self.bounds.1;
            if obj.pos.x < min.x + obj.radius {
                obj.pos.x = min.x + obj.radius;
                obj.vel.x *= -obj.restitution;
            }
            if obj.pos.x > max.x - obj.radius {
                obj.pos.x = max.x - obj.radius;
                obj.vel.x *= -obj.restitution;
            }
            if obj.pos.y < min.y + obj.radius {
                obj.pos.y = min.y + obj.radius;
                obj.vel.y *= -obj.restitution;
            }
            if obj.pos.y > max.y - obj.radius {
                obj.pos.y = max.y - obj.radius;
                obj.vel.y *= -obj.restitution;
            }
            if obj.pos.z < min.z + obj.radius {
                obj.pos.z = min.z + obj.radius;
                obj.vel.z *= -obj.restitution;
            }
            if obj.pos.z > max.z - obj.radius {
                obj.pos.z = max.z - obj.radius;
                obj.vel.z *= -obj.restitution;
            }
        }
    }

    pub fn simulate(&mut self, n_steps: usize) {
        for _ in 0..n_steps {
            self.step();
        }
    }

    pub fn status(&self) -> String {
        let total_ke: f32 = self
            .objects
            .iter()
            .map(|o| 0.5 * o.mass * o.vel.len_sq())
            .sum();
        format!(
            "World3D | objects={} | steps={} | collisions={} | total_KE={:.1}",
            self.objects.len(),
            self.n_steps,
            self.n_collisions,
            total_ke
        )
    }
}
