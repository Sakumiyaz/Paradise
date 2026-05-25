// EDEN GARM — Embodiment (sensorimotor loop)
// Un cuerpo virtual simplificado con propiocepción (joint angles, forces)
// y actuación (movimiento en 2D, grasp). El grounding físico real.

#[derive(Clone, Debug)]
pub struct Body {
    pub x: f32,
    pub y: f32,
    pub theta: f32, // orientation
    pub vx: f32,
    pub vy: f32,
    pub force_left: f32,
    pub force_right: f32,
    pub sensors: Vec<f32>, // proximity, touch, balance
    pub n_steps: u64,
}

impl Body {
    pub fn new() -> Self {
        Body {
            x: 0.0,
            y: 0.0,
            theta: 0.0,
            vx: 0.0,
            vy: 0.0,
            force_left: 0.0,
            force_right: 0.0,
            sensors: vec![0.0f32; 6],
            n_steps: 0,
        }
    }

    /// Sense the world: proximity to objects, balance, tactile.
    pub fn sense(&mut self, objects: &[(f32, f32)]) {
        // Proximity sensors (front, left, right, back, top, bottom)
        for s in self.sensors.iter_mut() {
            *s = 0.0;
        }
        for (ox, oy) in objects {
            let dx = ox - self.x;
            let dy = oy - self.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 5.0 {
                let intensity = 1.0 - dist / 5.0;
                if dy > 0.5 {
                    self.sensors[0] += intensity;
                } // front
                if dx < -0.5 {
                    self.sensors[1] += intensity;
                } // left
                if dx > 0.5 {
                    self.sensors[2] += intensity;
                } // right
                if dy < -0.5 {
                    self.sensors[3] += intensity;
                } // back
            }
        }
        // Balance sensor
        self.sensors[4] = self.theta.sin(); // tilt
                                            // Speed sensor
        self.sensors[5] = (self.vx * self.vx + self.vy * self.vy).sqrt() / 2.0;
    }

    /// Act: apply motor forces and update physics.
    pub fn act(&mut self, left_motor: f32, right_motor: f32) {
        self.force_left = left_motor.clamp(-1.0, 1.0);
        self.force_right = right_motor.clamp(-1.0, 1.0);
        let forward_force = (self.force_left + self.force_right) * 0.1;
        let rot_force = (self.force_right - self.force_left) * 0.05;
        self.vx += forward_force * self.theta.cos();
        self.vy += forward_force * self.theta.sin();
        self.theta += rot_force;
        self.x += self.vx;
        self.y += self.vy;
        // Friction
        self.vx *= 0.9;
        self.vy *= 0.9;
        self.n_steps += 1;
    }

    pub fn proprioception(&self) -> Vec<f32> {
        vec![
            self.x,
            self.y,
            self.theta,
            self.vx,
            self.vy,
            self.force_left,
            self.force_right,
        ]
    }

    pub fn status(&self) -> String {
        format!(
            "Body | pos=({:.1},{:.1}) theta={:.1} | sensors={:?} | steps={}",
            self.x,
            self.y,
            self.theta,
            self.sensors
                .iter()
                .map(|v| format!("{:.1}", v))
                .collect::<Vec<_>>(),
            self.n_steps
        )
    }
}
