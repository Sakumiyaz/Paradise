//! GARM HyperGraph — Grafo vivo de nodos autopoieticos
//!
//! No hay tick() maestro. El grafo pulsa: cada nodo se activa
//! cuando su energia libre supera el umbral, a su propia escala temporal.

use crate::eden_garm::fep::FEPEngine;
use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use std::collections::HashMap;

/// Mensaje en transito entre nodos.
#[derive(Clone, Debug)]
pub struct EdgeMessage {
    pub from: usize,
    pub to: usize,
    pub payload: Vec<f32>,
    pub tick: u64,
}

/// Grafo autopoietico multi-escala.
pub struct HyperGraph {
    pub nodes: Vec<Box<dyn GARMNode>>,
    pub adjacency: Vec<Vec<(usize, f32)>>, // (target_id, weight)
    pub reverse_adj: Vec<Vec<(usize, f32)>>, // (source_id, weight) para contexto
    pub fep: FEPEngine,
    pub global_tick: u64,
    pub messages: Vec<EdgeMessage>,
    pub born_nodes: Vec<Box<dyn GARMNode>>,
    pub dead_nodes: Vec<usize>,
    pub sensor_bus: Vec<f32>,
    pub output_log: Vec<String>,
}

impl HyperGraph {
    pub fn new() -> Self {
        HyperGraph {
            nodes: Vec::new(),
            adjacency: Vec::new(),
            reverse_adj: Vec::new(),
            fep: FEPEngine::new(),
            global_tick: 0,
            messages: Vec::new(),
            born_nodes: Vec::new(),
            dead_nodes: Vec::new(),
            sensor_bus: Vec::new(),
            output_log: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Box<dyn GARMNode>) {
        let id = node.id();
        // Asegurar que vectores son lo suficientemente largos
        while self.nodes.len() <= id {
            self.nodes.push(Box::new(NullNode::new(self.nodes.len())));
            self.adjacency.push(Vec::new());
            self.reverse_adj.push(Vec::new());
        }
        self.nodes[id] = node;
        self.adjacency[id] = Vec::new();
        self.reverse_adj[id] = Vec::new();
    }

    pub fn add_edge(&mut self, from: usize, to: usize, weight: f32) {
        if from < self.adjacency.len() && to < self.nodes.len() {
            self.adjacency[from].push((to, weight));
            self.reverse_adj[to].push((from, weight));
        }
    }

    /// Pulso del grafo: NO es un tick() maestro.
    /// Cada nodo se activa si su free energy > umbral.
    pub fn pulse(&mut self, dt: f32) {
        self.global_tick += 1;
        self.fep.regenerate(dt * 10.0); // regen continua

        // 1. Calcular free energy total
        let total_fe: f32 = self
            .nodes
            .iter()
            .filter(|n| n.is_alive())
            .map(|n| n.free_energy())
            .sum();

        // 2. Seleccionar nodos activos ordenados por sorpresa
        let active_ids = self.fep.select_active_nodes(&mut self.nodes);

        // 3. Para cada nodo activo, construir contexto y ejecutar predict/act
        let mut actions: Vec<(usize, NodeAction)> = Vec::with_capacity(active_ids.len());
        let mut outputs: HashMap<usize, Vec<f32>> = HashMap::with_capacity(active_ids.len());

        for &node_id in &active_ids {
            let node_fe = self.nodes[node_id].free_energy();
            let energy_allocation = self.fep.allocate_energy(node_fe, total_fe);

            // Consumir energia
            let _consumed = self.nodes[node_id].update(dt, energy_allocation);
            if !self.nodes[node_id].is_alive() {
                self.dead_nodes.push(node_id);
                continue;
            }

            // Construir contexto con outputs de vecinos y mensajes directos pendientes.
            let mut neighbor_outputs: Vec<(usize, Vec<f32>)> = self.reverse_adj[node_id]
                .iter()
                .filter_map(|(src_id, _weight)| {
                    outputs.get(src_id).map(|out| (*src_id, out.clone()))
                })
                .collect();
            neighbor_outputs.extend(
                self.messages
                    .iter()
                    .filter(|msg| msg.to == node_id && msg.tick < self.global_tick)
                    .map(|msg| (msg.from, msg.payload.clone())),
            );

            let ctx = NodeContext {
                tick: self.global_tick,
                global_free_energy: total_fe,
                neighbor_outputs,
                sensor_input: self.sensor_bus.clone(),
                ambient_energy: energy_allocation,
            };

            // Predict
            let prediction = self.nodes[node_id].predict(&ctx);
            // Simular error de prediccion (placeholder — en realidad viene de sensor)
            let prediction_error: Vec<f32> = prediction.iter().map(|_| 0.1).collect();

            // Act
            let action = self.nodes[node_id].act(&ctx, &prediction_error);
            actions.push((node_id, action.clone()));

            if let NodeAction::Output(ref out) = action {
                outputs.insert(node_id, out.clone());
            }
        }

        // 4. Procesar acciones (mensajes, requests de energia, spawn, kill)
        for (node_id, action) in actions {
            match action {
                NodeAction::None => {}
                NodeAction::Output(_) => {} // ya registrado arriba
                NodeAction::SendMessage(target, payload) => {
                    self.messages.push(EdgeMessage {
                        from: node_id,
                        to: target,
                        payload,
                        tick: self.global_tick,
                    });
                }
                NodeAction::RequestEnergy(amount) => {
                    self.fep.energy_pool -= amount.min(self.fep.energy_pool);
                }
                NodeAction::SpawnNode(ref node_type, suggested_id) => {
                    if self.fep.energy_pool >= 100.0 {
                        self.fep.energy_pool -= 100.0;
                        if let Some(node) = self.spawn_node_by_type(node_type, suggested_id) {
                            let id = node.id();
                            self.born_nodes.push(node);
                            self.output_log.push(format!(
                                "[GARM] Node {} spawn requested by {} (type={})",
                                id, node_id, node_type
                            ));
                        }
                    }
                }
                NodeAction::KillNode(target_id) => {
                    if target_id < self.nodes.len() && self.nodes[target_id].is_alive() {
                        self.dead_nodes.push(target_id);
                        self.output_log.push(format!(
                            "[GARM] Node {} kill requested by {}",
                            target_id, node_id
                        ));
                    }
                }
            }
        }

        // 5. Retener mensajes recientes con TTL de 10 ticks.
        let message_cutoff = self.global_tick.saturating_sub(10);
        self.messages.retain(|m| m.tick > message_cutoff);

        // 6. Limpiar nodos muertos
        self.dead_nodes.sort_unstable();
        self.dead_nodes.dedup();
        for dead in &self.dead_nodes {
            self.output_log.push(format!("[GARM] Node {} died", dead));
        }
        self.dead_nodes.clear();

        // 7. Incorporar nodos nacidos
        let new_nodes: Vec<Box<dyn GARMNode>> = self.born_nodes.drain(..).collect();
        for new_node in new_nodes {
            let id = new_node.id();
            self.add_node(new_node);
            self.output_log.push(format!("[GARM] Node {} born", id));
        }
    }

    /// Inyecta sensores externos (ej. input humano, datos de red).
    pub fn inject_sensor(&mut self, data: Vec<f32>) {
        self.sensor_bus = data;
    }

    pub fn inject_zero_sensor(&mut self, width: usize) {
        self.sensor_bus.resize(width, 0.0);
        self.sensor_bus.fill(0.0);
    }

    /// Crea un nuevo nodo por nombre de tipo.
    pub fn spawn_node_by_type(
        &self,
        node_type: &str,
        suggested_id: usize,
    ) -> Option<Box<dyn GARMNode>> {
        let id = suggested_id.max(self.nodes.len());
        match node_type {
            "fast_reflexes" => Some(Box::new(
                super::nodes::fast_reflexes::FastReflexesNode::new(id),
            )),
            "generative_controller" => Some(Box::new(
                super::nodes::generative_controller::GenerativeControllerNode::new(id),
            )),
            "benchmark" => Some(Box::new(super::nodes::benchmark::BenchmarkNode::new(id))),
            "corpus" => Some(Box::new(super::nodes::corpus::CorpusNode::new(id))),
            "planner" => Some(Box::new(super::nodes::planner::PlannerNode::new(id))),
            "memory" => Some(Box::new(super::nodes::memory::MemoryNode::new(id))),
            "security" => Some(Box::new(super::nodes::security::SecurityNode::new(id))),
            "cognitive" => Some(Box::new(super::nodes::cognitive::CognitiveNode::new(id))),
            "perception" => Some(Box::new(super::nodes::perception::PerceptionNode::new(id))),
            "social" => Some(Box::new(super::nodes::social::SocialNode::new(id))),
            "world" => Some(Box::new(super::nodes::world::WorldNode::new(id))),
            "evolution" => Some(Box::new(super::nodes::evolution::EvolutionNode::new(id))),
            "meta_architect" => Some(Box::new(
                super::nodes::meta_architect::MetaArchitectNode::new(id),
            )),
            "command_router" => Some(Box::new(
                super::nodes::command_router::CommandRouterNode::new(id),
            )),
            "persistence" => Some(Box::new(super::nodes::persistence::PersistenceNode::new(
                id,
            ))),
            "telemetry" => Some(Box::new(super::nodes::telemetry::TelemetryNode::new(id))),
            "legacy_memory" => Some(Box::new(
                super::nodes::legacy_memory::LegacyMemoryNode::new(id),
            )),
            "legacy_reason" => Some(Box::new(
                super::nodes::legacy_reason::LegacyReasonNode::new(id),
            )),
            "legacy_dialogue" => Some(Box::new(
                super::nodes::legacy_dialogue::LegacyDialogueNode::new(id),
            )),
            "observatory" => Some(Box::new(super::nodes::observatory::ObservatoryNode::new(
                id,
            ))),
            "legacy_history" => Some(Box::new(
                super::nodes::legacy_history::LegacyHistoryNode::new(id),
            )),
            "legacy_evolution" => Some(Box::new(
                super::nodes::legacy_evolution::LegacyEvolutionNode::new(id),
            )),
            "legacy_cognition" => Some(Box::new(
                super::nodes::legacy_cognition::LegacyCognitionNode::new(id),
            )),
            "campo_tension" => Some(Box::new(
                super::nodes::campo_tension::CampoTensionNode::new(id),
            )),
            "legacy_knowledge_graph" => Some(Box::new(
                super::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode::new(id),
            )),
            "legacy_autoconsumo" => Some(Box::new(
                super::nodes::legacy_runtime_extensions::AutoconsumoNode::new(id),
            )),
            "legacy_venado" => Some(Box::new(
                super::nodes::legacy_runtime_extensions::VenadoCompatibilityNode::new(id),
            )),
            "legacy_paradigm_hub" => Some(Box::new(
                super::nodes::legacy_runtime_extensions::ParadigmHubNode::new(id),
            )),
            "legacy_ecosystem" => Some(Box::new(
                super::nodes::legacy_runtime_extensions::EcoSystemNode::new(id),
            )),
            "legacy_rebirth_meltrace" => Some(Box::new(
                super::nodes::legacy_runtime_extensions::RebirthMeltraceNode::new(id),
            )),
            "legacy_crawler" => Some(Box::new(
                super::nodes::legacy_runtime_extensions::LegacyCrawlerNode::new(id),
            )),
            "help" => Some(Box::new(super::nodes::help::HelpNode::new(id))),
            _ => None,
        }
    }

    pub fn alive_node_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_alive()).count()
    }

    pub fn edge_count(&self) -> usize {
        self.adjacency.iter().map(|targets| targets.len()).sum()
    }

    /// Guarda el estado del hipergrafo en JSON (metadata + aristas + FEP).
    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let nodes: Vec<serde_json::Value> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.is_alive())
            .map(|(i, n)| {
                serde_json::json!({
                    "id": i,
                    "name": n.name(),
                    "scale": format!("{:?}", n.scale()),
                })
            })
            .collect();
        let edges: Vec<serde_json::Value> = self
            .adjacency
            .iter()
            .enumerate()
            .flat_map(|(from, targets)| {
                targets.iter().map(move |(to, weight)| {
                    serde_json::json!({"from": from, "to": to, "weight": weight})
                })
            })
            .collect();
        let snapshot = serde_json::json!({
            "nodes": nodes,
            "edges": edges,
            "fep": { "energy_pool": self.fep.energy_pool, "temperature": self.fep.temperature },
            "global_tick": self.global_tick,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    /// Carga estado del hipergrafo desde JSON.
    /// Si el runtime ya construyo la topologia, preserva nodos con dependencias vivas
    /// y restaura solo estado estructural seguro (FEP, tick, aristas).
    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;

        self.dead_nodes.clear();
        self.born_nodes.clear();
        self.messages.clear();
        self.output_log.clear();

        if let Some(fep_obj) = snapshot.get("fep").and_then(|v| v.as_object()) {
            if let Some(v) = fep_obj.get("energy_pool").and_then(|v| v.as_f64()) {
                self.fep.energy_pool = v as f32;
            }
            if let Some(v) = fep_obj.get("temperature").and_then(|v| v.as_f64()) {
                self.fep.temperature = v as f32;
            }
        }
        if let Some(v) = snapshot.get("global_tick").and_then(|v| v.as_u64()) {
            self.global_tick = v;
        }

        if self.nodes.is_empty() {
            if let Some(arr) = snapshot.get("nodes").and_then(|v| v.as_array()) {
                for node_val in arr {
                    let id = node_val.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let name = node_val
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    if let Some(node) = self.spawn_node_by_type(name, id) {
                        let actual_id = node.id();
                        self.add_node(node);
                        if actual_id != id {
                            self.output_log
                                .push(format!("[LOAD] Node {} remapped to {}", id, actual_id));
                        }
                    }
                }
            }
        }

        self.adjacency = vec![Vec::new(); self.nodes.len()];
        self.reverse_adj = vec![Vec::new(); self.nodes.len()];
        if let Some(arr) = snapshot.get("edges").and_then(|v| v.as_array()) {
            for edge_val in arr {
                let from = edge_val.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let to = edge_val.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let weight = edge_val
                    .get("weight")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
                if from < self.nodes.len() && to < self.nodes.len() {
                    self.add_edge(from, to, weight);
                }
            }
        }
        self.output_log.push(format!(
            "[LOAD] HyperGraph restored | nodes={} | edges={}",
            self.alive_node_count(),
            self.adjacency.iter().map(|v| v.len()).sum::<usize>()
        ));
        Ok(())
    }
}

/// Nodo nulo placeholder para rellenar indices.
pub struct NullNode {
    id: usize,
}

impl NullNode {
    pub fn new(id: usize) -> Self {
        NullNode { id }
    }
}

impl GARMNode for NullNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "null"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Fast
    }
    fn free_energy(&self) -> f32 {
        0.0
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![0.0]
    }
    fn act(&mut self, _ctx: &NodeContext, _err: &[f32]) -> NodeAction {
        NodeAction::None
    }
    fn update(&mut self, _dt: f32, _energy: f32) -> f32 {
        0.0
    }
    fn is_alive(&self) -> bool {
        false
    }
    fn spawn_cost(&self) -> f32 {
        0.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SenderNode {
        sent: bool,
    }

    impl GARMNode for SenderNode {
        fn id(&self) -> usize {
            0
        }
        fn name(&self) -> &str {
            "sender"
        }
        fn scale(&self) -> TemporalScale {
            TemporalScale::Fast
        }
        fn free_energy(&self) -> f32 {
            1.0
        }
        fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
            vec![0.0]
        }
        fn act(&mut self, _ctx: &NodeContext, _err: &[f32]) -> NodeAction {
            if self.sent {
                NodeAction::None
            } else {
                self.sent = true;
                NodeAction::SendMessage(1, vec![7.0, 9.0])
            }
        }
        fn update(&mut self, _dt: f32, _energy: f32) -> f32 {
            0.0
        }
        fn is_alive(&self) -> bool {
            true
        }
        fn spawn_cost(&self) -> f32 {
            0.0
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    struct ReceiverNode {
        seen_payloads: Vec<Vec<f32>>,
    }

    impl GARMNode for ReceiverNode {
        fn id(&self) -> usize {
            1
        }
        fn name(&self) -> &str {
            "receiver"
        }
        fn scale(&self) -> TemporalScale {
            TemporalScale::Fast
        }
        fn free_energy(&self) -> f32 {
            1.0
        }
        fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
            vec![0.0]
        }
        fn act(&mut self, ctx: &NodeContext, _err: &[f32]) -> NodeAction {
            for (_src, payload) in &ctx.neighbor_outputs {
                self.seen_payloads.push(payload.clone());
            }
            NodeAction::None
        }
        fn update(&mut self, _dt: f32, _energy: f32) -> f32 {
            0.0
        }
        fn is_alive(&self) -> bool {
            true
        }
        fn spawn_cost(&self) -> f32 {
            0.0
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn delivers_direct_messages_to_target_context() {
        let mut graph = HyperGraph::new();
        graph.add_node(Box::new(SenderNode { sent: false }));
        graph.add_node(Box::new(ReceiverNode {
            seen_payloads: Vec::new(),
        }));

        graph.pulse(0.1);
        graph.pulse(0.1);

        let receiver = graph.nodes[1]
            .as_any()
            .downcast_ref::<ReceiverNode>()
            .expect("receiver node should remain present");
        assert!(receiver
            .seen_payloads
            .iter()
            .any(|payload| payload == &[7.0, 9.0]));
    }
}
