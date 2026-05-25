// EDEN GARM Causality — Structural Causal Model (SCM) for explicit causal reasoning
// Observes actions and outcomes, builds a directed graph, and supports do-calculus interventions.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CausalNode {
    pub id: String,
    pub value: f32, // last observed value
    pub history: Vec<f32>,
    pub max_history: usize,
}

impl CausalNode {
    pub fn new(id: &str) -> Self {
        CausalNode {
            id: id.to_string(),
            value: 0.5,
            history: Vec::with_capacity(100),
            max_history: 100,
        }
    }

    pub fn observe(&mut self, v: f32) {
        self.value = v;
        self.history.push(v);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    pub fn mean(&self) -> f32 {
        if self.history.is_empty() {
            return 0.5;
        }
        self.history.iter().sum::<f32>() / self.history.len() as f32
    }
}

#[derive(Clone, Debug)]
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    pub weight: f32, // estimated causal effect strength
    pub observations: u32,
}

pub struct StructuralCausalModel {
    pub nodes: HashMap<String, CausalNode>,
    pub edges: Vec<CausalEdge>,
    pub alpha: f32, // learning rate for edge weights
}

impl StructuralCausalModel {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        for name in [
            "signal_sent",
            "process_alive",
            "memory_free",
            "load_avg",
            "prediction_error",
            "action_success",
        ] {
            nodes.insert(name.to_string(), CausalNode::new(name));
        }
        StructuralCausalModel {
            nodes,
            edges: Vec::new(),
            alpha: 0.1,
        }
    }

    pub fn node_mut(&mut self, id: &str) -> Option<&mut CausalNode> {
        self.nodes.get_mut(id)
    }

    pub fn observe_pair(
        &mut self,
        cause_id: &str,
        effect_id: &str,
        cause_val: f32,
        effect_val: f32,
    ) {
        // Update or create edge
        let mut found = false;
        for edge in &mut self.edges {
            if edge.from == cause_id && edge.to == effect_id {
                // Hebbian-like weight update: if cause and effect co-occur, strengthen
                let delta = self.alpha * (cause_val * effect_val - edge.weight);
                edge.weight = (edge.weight + delta).clamp(-1.0, 1.0);
                edge.observations += 1;
                found = true;
                break;
            }
        }
        if !found {
            self.edges.push(CausalEdge {
                from: cause_id.to_string(),
                to: effect_id.to_string(),
                weight: cause_val * effect_val,
                observations: 1,
            });
        }
        // Update node histories
        if let Some(n) = self.nodes.get_mut(cause_id) {
            n.observe(cause_val);
        }
        if let Some(n) = self.nodes.get_mut(effect_id) {
            n.observe(effect_val);
        }
    }

    /// Pearl's do-calculus: intervene on a node, breaking incoming edges, then propagate.
    /// Returns predicted values for all downstream nodes.
    pub fn do_intervention(&self, node_id: &str, value: f32) -> HashMap<String, f32> {
        let mut state: HashMap<String, f32> = self
            .nodes
            .iter()
            .map(|(k, v)| (k.clone(), v.value))
            .collect();
        state.insert(node_id.to_string(), value);
        // Propagate downstream in topological-ish order (multiple passes for stability)
        for _pass in 0..5 {
            for edge in &self.edges {
                let cause_val = *state.get(&edge.from).unwrap_or(&0.5);
                let current_effect = *state.get(&edge.to).unwrap_or(&0.5);
                // Only propagate if we didn't intervene directly on the effect node
                if edge.to != node_id {
                    let new_effect = current_effect + edge.weight * cause_val;
                    state.insert(edge.to.clone(), new_effect.clamp(0.0, 1.0));
                }
            }
        }
        state
    }

    /// Counterfactual: "What would have happened if I had done X?"
    /// 1. Abduce: infer state from observation (already in node.values)
    /// 2. Act: apply do(X)
    /// 3. Predict: propagate
    pub fn counterfactual(
        &self,
        intervention_node: &str,
        intervention_value: f32,
    ) -> HashMap<String, f32> {
        self.do_intervention(intervention_node, intervention_value)
    }

    /// Find the best intervention to maximize a target node.
    pub fn best_intervention(
        &self,
        intervention_candidates: &[(String, f32)],
        target_node: &str,
    ) -> Option<(String, f32)> {
        let mut best = None;
        let mut best_target_val = -1.0f32;
        for (node, val) in intervention_candidates {
            let predicted = self.do_intervention(node, *val);
            if let Some(&target_val) = predicted.get(target_node) {
                if target_val > best_target_val {
                    best_target_val = target_val;
                    best = Some((node.clone(), *val));
                }
            }
        }
        best
    }

    /// Explain why target changed by listing active causal paths from source.
    pub fn explain_effect(&self, source: &str, target: &str) -> Vec<String> {
        let mut explanations = Vec::new();
        for edge in &self.edges {
            if edge.from == source && edge.to == target && edge.weight.abs() > 0.05 {
                let dir = if edge.weight > 0.0 {
                    "increases"
                } else {
                    "decreases"
                };
                explanations.push(format!(
                    "{} {} {} (w={:.3})",
                    source, dir, target, edge.weight
                ));
            }
        }
        explanations
    }

    pub fn status(&self) -> String {
        format!(
            "Causality | nodes: {} | edges: {} | total_obs: {}",
            self.nodes.len(),
            self.edges.len(),
            self.edges.iter().map(|e| e.observations).sum::<u32>()
        )
    }
}
