//! GARM Node: MetaArchitect — Evolucion arquitectonica del grafo
//!
//! Este nodo es el unico autorizado a:
//! - Spawnear nuevos nodos (si reduce la FE global)
//! - Matar nodos muertos o inutiles
//! - Reconfigurar aristas
//! - Auto-modificar su propia topologia interna

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct MetaArchitectNode {
    id: usize,
    internal_fe: f32,
    last_global_fe: f32,
    proposals_generated: u64,
    proposals_applied: u64,
    last_action: String,
}

impl MetaArchitectNode {
    pub fn new(id: usize) -> Self {
        MetaArchitectNode {
            id,
            internal_fe: 2.0,
            last_global_fe: 1000.0,
            proposals_generated: 0,
            proposals_applied: 0,
            last_action: "init".to_string(),
        }
    }

    pub fn last_action(&self) -> &str {
        &self.last_action
    }
    pub fn proposals_generated(&self) -> u64 {
        self.proposals_generated
    }
    pub fn proposals_applied(&self) -> u64 {
        self.proposals_applied
    }

    pub fn review_without_mutation(&mut self, global_fe: f32) -> String {
        self.proposals_generated += 1;
        self.last_global_fe = global_fe;
        self.last_action = "review_without_mutation".to_string();
        format!(
            "[META-AUTO] reviewed architecture global_fe={:.2} proposals={} applied={} guard=no_mutation",
            global_fe, self.proposals_generated, self.proposals_applied
        )
    }

    pub fn architecture_snapshot(&self) -> String {
        format!(
            "meta:generated:{} applied:{} last:{} global_fe:{:.3} internal_fe:{:.3}",
            self.proposals_generated,
            self.proposals_applied,
            self.last_action,
            self.last_global_fe,
            self.internal_fe
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "internal_fe": self.internal_fe,
            "last_global_fe": self.last_global_fe,
            "proposals_generated": self.proposals_generated,
            "proposals_applied": self.proposals_applied,
            "last_action": self.last_action,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.internal_fe = snapshot
            .get("internal_fe")
            .and_then(|v| v.as_f64())
            .unwrap_or(2.0) as f32;
        self.last_global_fe = snapshot
            .get("last_global_fe")
            .and_then(|v| v.as_f64())
            .unwrap_or(1000.0) as f32;
        self.proposals_generated = snapshot
            .get("proposals_generated")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.proposals_applied = snapshot
            .get("proposals_applied")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.last_action = snapshot
            .get("last_action")
            .and_then(|v| v.as_str())
            .unwrap_or("init")
            .to_string();
        Ok(())
    }
}

impl GARMNode for MetaArchitectNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "meta_architect"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Evolutionary
    }

    fn free_energy(&self) -> f32 {
        // El arquitecto sufre cuando el grafo global no mejora.
        // FE siempre >= 0 (principio FEP).
        let improvement = (self.last_global_fe - self.internal_fe).max(0.0);
        (self.internal_fe + improvement * 0.1 + 0.5).max(0.1)
    }

    fn predict(&mut self, ctx: &NodeContext) -> Vec<f32> {
        // Predice: el grafo mejorara o empeorara?
        let trend = if ctx.global_free_energy < self.last_global_fe {
            1.0
        } else {
            -1.0
        };
        vec![trend, ctx.global_free_energy, self.internal_fe]
    }

    fn act(&mut self, ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        let current_fe = ctx.global_free_energy;
        self.proposals_generated += 1;

        // Regla 1: si FE global sube por 10 ticks consecutivos, proponer matar el nodo mas costoso
        if current_fe > self.last_global_fe * 1.1 {
            self.last_action = "propose_kill_costliest".to_string();
            self.internal_fe += 0.2;
            // En version completa, esto emitiria NodeAction::KillNode(id)
            // pero por seguridad, en fase 1 solo logueamos
            return NodeAction::Output(vec![-2.0, current_fe, self.last_global_fe]);
        }

        // Regla 2: si FE global baja, reforzar aristas entre nodos cooperativos
        if current_fe < self.last_global_fe * 0.95 {
            self.last_action = "reinforce_edges".to_string();
            self.proposals_applied += 1;
            self.internal_fe *= 0.95;
        }

        // Regla 3: si hay mucha energia libre en el pool, proponer spawn de nodo especializado
        if ctx.ambient_energy > 30.0 && self.proposals_generated % 50 == 0 {
            self.last_action = "propose_spawn".to_string();
            return NodeAction::SpawnNode("benchmark".to_string(), 20);
        }

        self.last_global_fe = current_fe;

        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }

        NodeAction::Output(vec![
            current_fe,
            self.internal_fe,
            self.proposals_applied as f32,
        ])
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost = 5.0; // el arquitecto es caro (pensar en la arquitectura cuesta)
        if energy_in < cost {
            self.internal_fe += 0.05;
        }
        cost
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        1000.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
