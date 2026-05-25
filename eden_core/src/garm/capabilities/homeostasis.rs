// EDEN GARM — Homeostasis (multi-objective metabolism)
// 5 recursos: energy, glucose, security, curiosity, integrity.
// Tradeoffs reales: gastar curiosidad gana knowledge pero consume energy.
// El sistema busca equilibrio homeostático en un espacio 5D.

#[derive(Clone, Debug)]
pub struct Homeostasis {
    pub energy: f32,
    pub glucose: f32,
    pub security: f32,
    pub curiosity: f32,
    pub integrity: f32,
    pub targets: [f32; 5],
    pub n_depleted: u64,
    pub n_balanced: u64,
    pub total_actions: u64,
}

impl Homeostasis {
    pub fn new() -> Self {
        Homeostasis {
            energy: 80.0,
            glucose: 80.0,
            security: 80.0,
            curiosity: 80.0,
            integrity: 80.0,
            targets: [80.0, 80.0, 80.0, 80.0, 80.0],
            n_depleted: 0,
            n_balanced: 0,
            total_actions: 0,
        }
    }

    /// Tick: natural decay + regeneration toward baseline.
    pub fn tick(&mut self) {
        // Drift toward targets
        let drift = 0.05;
        self.energy += (self.targets[0] - self.energy) * drift;
        self.glucose += (self.targets[1] - self.glucose) * drift;
        self.security += (self.targets[2] - self.security) * drift;
        self.curiosity += (self.targets[3] - self.curiosity) * drift;
        self.integrity += (self.targets[4] - self.integrity) * drift;
        // Clamp
        self.energy = self.energy.clamp(0.0, 100.0);
        self.glucose = self.glucose.clamp(0.0, 100.0);
        self.security = self.security.clamp(0.0, 100.0);
        self.curiosity = self.curiosity.clamp(0.0, 100.0);
        self.integrity = self.integrity.clamp(0.0, 100.0);
    }

    /// Spend resource. Returns false if depleted.
    pub fn spend(&mut self, resource: &str, amount: f32) -> bool {
        let val = match resource {
            "energy" => &mut self.energy,
            "glucose" => &mut self.glucose,
            "security" => &mut self.security,
            "curiosity" => &mut self.curiosity,
            "integrity" => &mut self.integrity,
            _ => return false,
        };
        if *val < amount {
            self.n_depleted += 1;
            false
        } else {
            *val -= amount;
            self.total_actions += 1;
            true
        }
    }

    /// Replenish resource (e.g., from tool success, learning, etc.)
    pub fn replenish(&mut self, resource: &str, amount: f32) {
        let val = match resource {
            "energy" => &mut self.energy,
            "glucose" => &mut self.glucose,
            "security" => &mut self.security,
            "curiosity" => &mut self.curiosity,
            "integrity" => &mut self.integrity,
            _ => return,
        };
        *val = (*val + amount).min(100.0);
    }

    /// Homeostatic imbalance: distance from target space.
    pub fn imbalance(&self) -> f32 {
        let d0 = (self.energy - self.targets[0]).abs();
        let d1 = (self.glucose - self.targets[1]).abs();
        let d2 = (self.security - self.targets[2]).abs();
        let d3 = (self.curiosity - self.targets[3]).abs();
        let d4 = (self.integrity - self.targets[4]).abs();
        (d0 + d1 + d2 + d3 + d4) / 5.0
    }

    /// Most depleted resource (what the system "needs" most).
    pub fn most_needed(&self) -> (&str, f32) {
        let deficits = [
            ("energy", self.targets[0] - self.energy),
            ("glucose", self.targets[1] - self.glucose),
            ("security", self.targets[2] - self.security),
            ("curiosity", self.targets[3] - self.curiosity),
            ("integrity", self.targets[4] - self.integrity),
        ];
        deficits
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
            .unwrap_or(("energy", 0.0))
    }

    pub fn status(&self) -> String {
        let (need, deficit) = self.most_needed();
        format!(
            "Homeo | E={:.1} G={:.1} S={:.1} C={:.1} I={:.1} | need={}({:.1}) | imbalance={:.1}",
            self.energy,
            self.glucose,
            self.security,
            self.curiosity,
            self.integrity,
            need,
            deficit,
            self.imbalance()
        )
    }
}
