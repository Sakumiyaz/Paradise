// EDEN GARM ArchitectureModel — Grafo explícito de la arquitectura propia.
// El sistema tiene un modelo de sí mismo: nodos son módulos, arcos son dependencias.
// Permite razonar sobre cuellos de botella, errores, y cambios estructurales.
//
// Ejemplo de razonamiento:
//   "big_transformer es lento porque CPU → cuantizar a INT8 → modificar forward"

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ArchNode {
    pub name: String,
    pub module_type: String, // "transformer", "bus", "tool", "memory", etc.
    pub params_count: usize,
    pub train_speed: f32, // tokens/sec or ops/sec
    pub error_rate: f32,  // 0..1
    pub memory_mb: f32,
    pub status: String, // "active", "slow", "error", "idle"
    pub custom_attrs: HashMap<String, f32>,
}

#[derive(Clone, Debug)]
pub struct ArchEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String, // "feeds", "controls", "depends_on", "monitors"
    pub weight: f32,
}

#[derive(Clone, Debug)]
pub struct ArchitectureModel {
    pub nodes: HashMap<String, ArchNode>,
    pub edges: Vec<ArchEdge>,
    pub self_model_accuracy: f32, // how well this model predicts actual behavior
    pub n_updates: u64,
}

impl ArchitectureModel {
    pub fn new() -> Self {
        let mut model = ArchitectureModel {
            nodes: HashMap::new(),
            edges: Vec::new(),
            self_model_accuracy: 0.5,
            n_updates: 0,
        };
        // Seed with known architecture
        model.add_node("big_transformer", "transformer", 38_000_000, 0.0);
        model.add_node("transformer", "transformer", 100_000, 0.0);
        model.add_node("unified_bus", "bus", 0, 0.0);
        model.add_node("morphogenesis", "memory", 0, 0.0);
        model.add_node("planner", "controller", 0, 0.0);
        model.add_node("auto_debug", "monitor", 0, 0.0);
        model.add_node("world_model", "memory", 0, 0.0);
        model.add_node("motivation", "controller", 0, 0.0);
        model.add_node("social_complex", "social", 0, 0.0);
        model.add_node("autonomy_econ", "controller", 0, 0.0);
        model.add_node("experience_buffer", "memory", 0, 0.0);
        model.add_node("corpus_reader", "io", 0, 0.0);
        model.add_node("tool_registry", "tool", 0, 0.0);
        model.add_node("self_model", "predictor", 0, 0.0);

        model.add_edge("big_transformer", "unified_bus", "feeds", 0.8);
        model.add_edge("unified_bus", "planner", "feeds", 0.7);
        model.add_edge("unified_bus", "motivation", "feeds", 0.6);
        model.add_edge("planner", "autonomy_econ", "controls", 0.9);
        model.add_edge("auto_debug", "big_transformer", "monitors", 0.5);
        model.add_edge("auto_debug", "self_model", "monitors", 0.4);
        model.add_edge("corpus_reader", "big_transformer", "feeds", 0.9);
        model.add_edge("tool_registry", "big_transformer", "depends_on", 0.3);
        model.add_edge("experience_buffer", "big_transformer", "feeds", 0.8);
        model.add_edge("morphogenesis", "big_transformer", "feeds", 0.6);
        model.add_edge("world_model", "planner", "feeds", 0.7);
        model.add_edge("social_complex", "unified_bus", "feeds", 0.5);
        model
    }

    pub fn add_node(&mut self, name: &str, module_type: &str, params: usize, train_speed: f32) {
        self.nodes.insert(
            name.to_string(),
            ArchNode {
                name: name.to_string(),
                module_type: module_type.to_string(),
                params_count: params,
                train_speed,
                error_rate: 0.0,
                memory_mb: 0.0,
                status: "active".to_string(),
                custom_attrs: HashMap::new(),
            },
        );
    }

    pub fn add_edge(&mut self, from: &str, to: &str, edge_type: &str, weight: f32) {
        self.edges.push(ArchEdge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type: edge_type.to_string(),
            weight,
        });
    }

    pub fn update_node(&mut self, name: &str, f: impl FnOnce(&mut ArchNode)) {
        if let Some(node) = self.nodes.get_mut(name) {
            f(node);
        }
    }

    /// Update from actual runtime observations.
    pub fn observe(&mut self, tick: u64, module_name: &str, metric_name: &str, value: f32) {
        self.n_updates = tick;
        if let Some(node) = self.nodes.get_mut(module_name) {
            node.custom_attrs.insert(metric_name.to_string(), value);
            match metric_name {
                "error_rate" => node.error_rate = value,
                "memory_mb" => node.memory_mb = value,
                "train_speed" => node.train_speed = value,
                "status" => node.status = if value > 0.5 { "error" } else { "active" }.to_string(),
                _ => {}
            }
        }
    }

    /// Find bottleneck nodes (high params, low speed, or high error rate).
    pub fn find_bottlenecks(&self) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = self
            .nodes
            .iter()
            .map(|(name, node)| {
                let score = (node.params_count as f32 / 1e6) * 0.3
                    + (1.0 / (node.train_speed + 1.0)) * 0.3
                    + node.error_rate * 0.4;
                (name.clone(), score)
            })
            .filter(|(_, s)| *s > 0.1)
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }

    /// Find nodes that depend on a given node (downstream).
    pub fn downstream(&self, name: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter(|e| e.from == name)
            .map(|e| e.to.clone())
            .collect()
    }

    /// Find nodes that a given node depends on (upstream).
    pub fn upstream(&self, name: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter(|e| e.to == name)
            .map(|e| e.from.clone())
            .collect()
    }

    /// Simple reasoning: if node A has high error, what nodes does it affect?
    pub fn infer_impact(&self, node_name: &str) -> Vec<(String, f32)> {
        let mut impacts = Vec::new();
        if let Some(node) = self.nodes.get(node_name) {
            let error = node.error_rate;
            for edge in &self.edges {
                if edge.from == node_name {
                    impacts.push((edge.to.clone(), edge.weight * error));
                }
            }
        }
        impacts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        impacts
    }

    /// Suggest a structural change based on bottleneck analysis.
    pub fn suggest_change(&self) -> Option<String> {
        let bottlenecks = self.find_bottlenecks();
        for (name, score) in bottlenecks.iter().take(3) {
            if let Some(node) = self.nodes.get(name) {
                if node.module_type == "transformer" && node.params_count > 10_000_000 {
                    return Some(format!("{} is bottleneck (score={:.2}). Suggest: quantize to INT8 or reduce adapter_dim", name, score));
                }
                if node.error_rate > 0.2 {
                    return Some(format!(
                        "{} has high error rate ({:.2}). Suggest: check inputs from {:?}",
                        name,
                        node.error_rate,
                        self.upstream(name)
                    ));
                }
                if node.train_speed < 1.0 && node.params_count > 0 {
                    return Some(format!("{} is slow (speed={:.1}). Suggest: gradient checkpointing or smaller batch", name, node.train_speed));
                }
            }
        }
        None
    }

    pub fn status(&self) -> String {
        let active = self.nodes.values().filter(|n| n.status == "active").count();
        let errors = self.nodes.values().filter(|n| n.error_rate > 0.0).count();
        format!("ArchModel | nodes={} | edges={} | active={} | with_errors={} | updates={} | accuracy={:.2}",
            self.nodes.len(), self.edges.len(), active, errors, self.n_updates, self.self_model_accuracy)
    }
}
