//! GARM FEP Engine — Free Energy Principle orchestrator
//!
//! No hay tick() maestro. Este motor solo calcula energia libre
//! y decide qué nodos tienen derecho a predecir/actuar.

use crate::eden_garm::node::GARMNode;

/// Motor FEP: distribuye energia y resuelve sorpresa.
pub struct FEPEngine {
    pub temperature: f32,
    pub energy_pool: f32,
    pub max_energy: f32,
}

impl FEPEngine {
    pub fn new() -> Self {
        FEPEngine {
            temperature: 1.0,
            energy_pool: 5000.0,
            max_energy: 5000.0,
        }
    }

    /// Regenera energia del pool global.
    pub fn regenerate(&mut self, amount: f32) {
        self.energy_pool = (self.energy_pool + amount).min(self.max_energy);
    }

    /// Selecciona nodos que merecen activarse (free energy > umbral).
    /// Los ordena por sorpresa descendente (mayor incertidumbre primero).
    pub fn select_active_nodes(&self, nodes: &mut [Box<dyn GARMNode>]) -> Vec<usize> {
        let mut candidates: Vec<(usize, f32)> = nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.is_alive())
            .map(|(i, n)| (i, n.free_energy()))
            .filter(|(_, fe)| *fe > 0.1) // umbral minimo de activacion
            .collect();
        // Ordenar por free energy descendente
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        candidates.into_iter().map(|(i, _)| i).collect()
    }

    /// Distribuye energia proporcional a la free energy del nodo.
    pub fn allocate_energy(&self, node_fe: f32, total_fe: f32) -> f32 {
        if total_fe <= 0.0 {
            return 0.0;
        }
        let ratio = node_fe / total_fe;
        let allocation = self.energy_pool * ratio * 0.1; // 10% del pool max por nodo
        allocation.min(50.0) // cap por nodo
    }
}
