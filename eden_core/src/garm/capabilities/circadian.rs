// EDEN GARM Circadian — Ritmos/fases globales (Agujero 11)
// El sistema alterna entre fases: vigilancia, consolidacion, reparacion.
// Cada fase modula que módulos están activos y a qué frecuencia.

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    Vigilance,     // Procesamiento sensorial, alta frecuencia
    Consolidation, // Memoria, clustering, baja frecuencia
    Repair,        // Auto-debug, self-improvement, prioridad absoluta
}

#[derive(Clone, Debug)]
pub struct Circadian {
    pub phase: Phase,
    pub tick_in_phase: u64,
    pub vig_ticks: u64,
    pub cons_ticks: u64,
    pub rep_ticks: u64,
    pub n_cycles: u64,
}

impl Circadian {
    pub fn new() -> Self {
        Circadian {
            phase: Phase::Vigilance,
            tick_in_phase: 0,
            vig_ticks: 30,
            cons_ticks: 20,
            rep_ticks: 10,
            n_cycles: 0,
        }
    }

    /// Tick the circadian rhythm. Returns true if phase changed.
    pub fn tick(&mut self) -> bool {
        self.tick_in_phase += 1;
        let max_ticks = match self.phase {
            Phase::Vigilance => self.vig_ticks,
            Phase::Consolidation => self.cons_ticks,
            Phase::Repair => self.rep_ticks,
        };
        if self.tick_in_phase >= max_ticks {
            self.advance_phase();
            true
        } else {
            false
        }
    }

    fn advance_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Vigilance => Phase::Consolidation,
            Phase::Consolidation => Phase::Repair,
            Phase::Repair => {
                self.n_cycles += 1;
                Phase::Vigilance
            }
        };
        self.tick_in_phase = 0;
    }

    /// Is a module allowed to run in current phase?
    pub fn module_allowed(&self, module: &str) -> bool {
        match self.phase {
            Phase::Vigilance => true,
            Phase::Consolidation => {
                matches!(
                    module,
                    "hippocampus"
                        | "memory_clustering"
                        | "morphogenesis"
                        | "semantics"
                        | "big_transformer"
                        | "transformer"
                )
            }
            Phase::Repair => {
                matches!(
                    module,
                    "auto_debug"
                        | "self_improvement"
                        | "self_awareness"
                        | "metacognition"
                        | "planner"
                )
            }
        }
    }

    /// Multiplier for tick frequency of a module.
    pub fn frequency_multiplier(&self, module: &str) -> f32 {
        match self.phase {
            Phase::Vigilance => 1.0,
            Phase::Consolidation => {
                if self.module_allowed(module) {
                    1.5
                } else {
                    0.0
                }
            }
            Phase::Repair => {
                if self.module_allowed(module) {
                    2.0
                } else {
                    0.1
                }
            }
        }
    }

    pub fn status(&self) -> String {
        let name = match self.phase {
            Phase::Vigilance => "vigilance",
            Phase::Consolidation => "consolidation",
            Phase::Repair => "repair",
        };
        format!(
            "Circadian | phase={} | tick_in={}/{} | cycles={}",
            name,
            self.tick_in_phase,
            match self.phase {
                Phase::Vigilance => self.vig_ticks,
                Phase::Consolidation => self.cons_ticks,
                Phase::Repair => self.rep_ticks,
            },
            self.n_cycles
        )
    }
}
