// EDEN GARM Physics — Intuitive physics engine learned from observation
// Objects have mass, rigidity, support relationships. Learns from collisions.

use std::collections::HashMap;

#[derive(Clone, Debug, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
    pub fn zero() -> Self {
        Vec2 { x: 0.0, y: 0.0 }
    }
    pub fn add(&self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
    pub fn sub(&self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
    pub fn scale(&self, s: f32) -> Vec2 {
        Vec2 {
            x: self.x * s,
            y: self.y * s,
        }
    }
    pub fn mag(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn dot(&self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

#[derive(Clone, Debug)]
pub struct PhysicalObject {
    pub id: u64,
    pub label: Option<String>,
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub rigidity: f32, // 0.0 = liquid, 1.0 = solid
    pub size: Vec2,    // bounding box half-extents
    pub visible: bool,
    pub contains: Vec<u64>,
    pub supported_by: Option<u64>,
    pub mass_observations: Vec<f32>, // online estimate history
}

pub struct PhysicsEngine {
    pub objects: HashMap<u64, PhysicalObject>,
    pub gravity: Vec2,
    pub friction: f32,
    pub restitution: f32,
    pub collision_count: u64,
    pub next_id: u64,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        PhysicsEngine {
            objects: HashMap::new(),
            gravity: Vec2::new(0.0, 0.05),
            friction: 0.95,
            restitution: 0.7,
            collision_count: 0,
            next_id: 1,
        }
    }

    /// Create or update a physical object from vision blob + label
    pub fn register_object(
        &mut self,
        cx: f32,
        cy: f32,
        w: f32,
        h: f32,
        label: Option<String>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.objects.insert(
            id,
            PhysicalObject {
                id,
                label,
                position: Vec2::new(cx, cy),
                velocity: Vec2::zero(),
                mass: 1.0, // default, updated by collisions
                rigidity: 0.8,
                size: Vec2::new(w / 2.0, h / 2.0),
                visible: true,
                contains: Vec::new(),
                supported_by: None,
                mass_observations: vec![1.0],
            },
        );
        id
    }

    /// Update object position from tracking (world model)
    pub fn update_position(&mut self, id: u64, cx: f32, cy: f32) {
        if let Some(obj) = self.objects.get_mut(&id) {
            let new_pos = Vec2::new(cx, cy);
            obj.velocity = new_pos.sub(obj.position);
            obj.position = new_pos;
            obj.visible = true;
        }
    }

    /// Mark object as invisible
    pub fn mark_invisible(&mut self, id: u64) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.visible = false;
            obj.velocity = Vec2::zero();
        }
    }

    /// Infer support relationships: if A is directly above B and stopped, A is supported by B
    pub fn infer_supports(&mut self) {
        let ids: Vec<u64> = self.objects.keys().copied().collect();
        for &a_id in &ids {
            for &b_id in &ids {
                if a_id == b_id {
                    continue;
                }
                let (a_pos, a_vel, a_size, _a_mass) = {
                    let a = self.objects.get(&a_id).unwrap();
                    (a.position, a.velocity, a.size, a.mass)
                };
                let b = match self.objects.get(&b_id) {
                    Some(x) => x,
                    None => continue,
                };
                // A above B, close vertically, overlapping horizontally, nearly stopped
                let vertical_dist = b.position.y - a_pos.y;
                let horiz_overlap = (a_pos.x - b.position.x).abs() < (a_size.x + b.size.x);
                let stopped = a_vel.mag() < 0.01;
                if vertical_dist > 0.0
                    && vertical_dist < (a_size.y + b.size.y) * 1.5
                    && horiz_overlap
                    && stopped
                {
                    if let Some(a) = self.objects.get_mut(&a_id) {
                        a.supported_by = Some(b_id);
                    }
                }
            }
        }
    }

    /// Observe collision and update mass estimates via conservation of momentum
    pub fn observe_collision(
        &mut self,
        a_id: u64,
        b_id: u64,
        pre_a: Vec2,
        pre_b: Vec2,
        post_a: Vec2,
        post_b: Vec2,
    ) {
        let delta_va = post_a.sub(pre_a);
        let delta_vb = pre_b.sub(post_b);
        let da = delta_va.mag();
        let db = delta_vb.mag();
        if da > 1e-4 && db > 1e-4 {
            // m_a * da ≈ m_b * db => ratio = db / da
            let ratio = db / da;
            if let Some(a) = self.objects.get_mut(&a_id) {
                a.mass_observations.push(ratio);
                if a.mass_observations.len() > 20 {
                    a.mass_observations.remove(0);
                }
                a.mass = a.mass_observations.iter().sum::<f32>() / a.mass_observations.len() as f32;
            }
            if let Some(b) = self.objects.get_mut(&b_id) {
                b.mass_observations.push(1.0 / ratio);
                if b.mass_observations.len() > 20 {
                    b.mass_observations.remove(0);
                }
                b.mass = b.mass_observations.iter().sum::<f32>() / b.mass_observations.len() as f32;
            }
            self.collision_count += 1;
        }
    }

    /// Simulate N steps into the future (no side effects on real objects)
    pub fn simulate_future(&self, steps: usize) -> Vec<HashMap<u64, Vec2>> {
        let mut states: Vec<HashMap<u64, Vec2>> = Vec::new();
        let mut sim: HashMap<u64, PhysicalObject> =
            self.objects.iter().map(|(k, v)| (*k, v.clone())).collect();

        for _ in 0..steps {
            self.step_simulation(&mut sim);
            let snapshot: HashMap<u64, Vec2> =
                sim.iter().map(|(id, obj)| (*id, obj.position)).collect();
            states.push(snapshot);
        }
        states
    }

    fn step_simulation(&self, sim: &mut HashMap<u64, PhysicalObject>) {
        let ids: Vec<u64> = sim.keys().copied().collect();
        // Apply gravity + friction
        for &id in &ids {
            if let Some(obj) = sim.get_mut(&id) {
                if obj.supported_by.is_none() {
                    obj.velocity = obj.velocity.add(self.gravity);
                }
                obj.velocity = obj.velocity.scale(self.friction);
                obj.position = obj.position.add(obj.velocity);
            }
        }
        // Resolve overlaps (simple AABB)
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let a_id = ids[i];
                let b_id = ids[j];
                let overlap = {
                    let a = sim.get(&a_id).unwrap();
                    let b = sim.get(&b_id).unwrap();
                    let dx = (a.position.x - b.position.x).abs();
                    let dy = (a.position.y - b.position.y).abs();
                    let ox = dx - (a.size.x + b.size.x);
                    let oy = dy - (a.size.y + b.size.y);
                    (ox < 0.0 && oy < 0.0, ox, oy)
                };
                if overlap.0 {
                    let (a_pos, _a_vel, a_mass, a_rig) = {
                        let a = sim.get(&a_id).unwrap();
                        (a.position, a.velocity, a.mass, a.rigidity)
                    };
                    let (b_pos, _b_vel, b_mass, b_rig) = {
                        let b = sim.get(&b_id).unwrap();
                        (b.position, b.velocity, b.mass, b.rigidity)
                    };
                    // Simple elastic-ish response
                    let nx = if a_pos.x < b_pos.x { -1.0 } else { 1.0 };
                    let ny = if a_pos.y < b_pos.y { -1.0 } else { 1.0 };
                    let push = 0.01 * self.restitution;
                    if let Some(a) = sim.get_mut(&a_id) {
                        a.position.x += nx * push * (b_mass / (a_mass + b_mass));
                        a.position.y += ny * push * (b_mass / (a_mass + b_mass));
                        a.velocity = a.velocity.scale(1.0 - a_rig * 0.1);
                    }
                    if let Some(b) = sim.get_mut(&b_id) {
                        b.position.x -= nx * push * (a_mass / (a_mass + b_mass));
                        b.position.y -= ny * push * (a_mass / (a_mass + b_mass));
                        b.velocity = b.velocity.scale(1.0 - b_rig * 0.1);
                    }
                }
            }
        }
        // Floor boundary at y=1.0
        for &id in &ids {
            if let Some(obj) = sim.get_mut(&id) {
                if obj.position.y > 1.0 {
                    obj.position.y = 1.0;
                    obj.velocity.y = -obj.velocity.y * self.restitution;
                }
                if obj.position.y < 0.0 {
                    obj.position.y = 0.0;
                }
                if obj.position.x > 1.0 {
                    obj.position.x = 1.0;
                }
                if obj.position.x < 0.0 {
                    obj.position.x = 0.0;
                }
            }
        }
    }

    pub fn status(&self) -> String {
        let total_mass: f32 = self.objects.values().map(|o| o.mass).sum();
        format!(
            "Physics | objects={} | collisions={} | avg_mass={:.2} | gravity=({:.2},{:.2})",
            self.objects.len(),
            self.collision_count,
            total_mass / self.objects.len().max(1) as f32,
            self.gravity.x,
            self.gravity.y
        )
    }
}
