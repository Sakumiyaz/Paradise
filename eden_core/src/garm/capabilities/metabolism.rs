// EDEN GARM Metabolism — Presupuesto de energia computacional (Agujero 7)
// Cada operacion tiene un costo. Si energy < 0, el sistema hiberna (reduce ticks).

#[derive(Clone, Debug)]
pub struct Metabolism {
    pub energy: f32, // 0..100
    pub max_energy: f32,
    pub regen_per_tick: f32,
    pub cost_per_train: f32,
    pub cost_per_plan: f32,
    pub cost_per_tool: f32,
    pub cost_per_program: f32,
    pub hibernating: bool,
    pub n_hibernations: u64,
    pub total_spent: f32,
}

impl Metabolism {
    pub fn new() -> Self {
        Metabolism {
            energy: 100.0,
            max_energy: 100.0,
            regen_per_tick: 5.0,
            cost_per_train: 2.0,
            cost_per_plan: 3.0,
            cost_per_tool: 2.0,
            cost_per_program: 4.0,
            hibernating: false,
            n_hibernations: 0,
            total_spent: 0.0,
        }
    }

    pub fn tick(&mut self) {
        self.energy = (self.energy + self.regen_per_tick).min(self.max_energy);
        if self.energy > 20.0 {
            self.hibernating = false;
        }
    }

    pub fn spend(&mut self, amount: f32, _operation: &str) -> bool {
        if self.energy < amount {
            self.hibernating = true;
            self.n_hibernations += 1;
            false
        } else {
            self.energy -= amount;
            self.total_spent += amount;
            true
        }
    }

    pub fn can_afford(&self, amount: f32) -> bool {
        self.energy >= amount && !self.hibernating
    }

    pub fn status(&self) -> String {
        format!(
            "Metabolism | energy={:.1}/{:.0} | hibernating={} | hibernations={} | spent={:.1}",
            self.energy, self.max_energy, self.hibernating, self.n_hibernations, self.total_spent
        )
    }
}
