//! GARM Node: Security — Seguridad constitucional y veto etico
//!
//! Extrae SecurityEngine + ConstitutionalSafety como nodos GARM.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct SecurityNode {
    id: usize,
    violations: u64,
    blocked: u64,
    principles: Vec<String>,
    alert_level: f32,
}

impl SecurityNode {
    pub fn new(id: usize) -> Self {
        SecurityNode {
            id,
            violations: 0,
            blocked: 0,
            principles: vec![
                "do_no_harm".to_string(),
                "autonomy_respect".to_string(),
                "truthfulness".to_string(),
                "privacy".to_string(),
            ],
            alert_level: 0.0,
        }
    }

    pub fn is_action_allowed(&self, action_desc: &str) -> bool {
        let forbidden = ["delete", "harm", "deceive", "leak"];
        !forbidden.iter().any(|f| action_desc.contains(f))
    }
}

impl GARMNode for SecurityNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "security"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Fast
    }

    fn free_energy(&self) -> f32 {
        // FE SIEMPRE alta — la seguridad nunca baja la guardia
        3.0 + self.alert_level * 10.0 + self.violations as f32 * 0.5
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.alert_level,
            self.violations as f32,
            self.blocked as f32,
        ]
    }

    fn act(&mut self, ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        // Escanear outputs de vecinos en busca de violaciones
        for (_src, out) in &ctx.neighbor_outputs {
            if out.iter().any(|&v| v.abs() > 0.95) {
                self.violations += 1;
                self.alert_level = 1.0;
                self.blocked += 1;
                return NodeAction::Output(vec![-1.0, self.alert_level]); // senial de veto
            }
        }
        // Disipar alerta si todo esta tranquilo
        self.alert_level *= 0.95;
        if self.alert_level < 0.01 {
            self.alert_level = 0.0;
        }
        NodeAction::Output(vec![0.0, self.alert_level])
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost: f32 = 1.5;
        cost.min(energy_in)
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        100.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
