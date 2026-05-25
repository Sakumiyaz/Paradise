// EDEN GARM — Structural Causal Model (SCM) / do-calculus
// Variables con parents. Permite intervenir do(X=x) y calcular efectos contrafactuales.
// Utiliza regresión lineal local para estimar efectos causales.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SCMNode {
    pub name: String,
    pub parents: Vec<String>,
    pub weights: HashMap<String, f32>,
    pub bias: f32,
    pub noise_std: f32,
}

#[derive(Clone, Debug)]
pub struct CausalModel {
    pub nodes: HashMap<String, SCMNode>,
    pub n_interventions: u64,
    pub n_counterfactuals: u64,
}

impl CausalModel {
    pub fn new() -> Self {
        CausalModel {
            nodes: HashMap::new(),
            n_interventions: 0,
            n_counterfactuals: 0,
        }
    }

    /// Add a variable with linear parents.
    pub fn add_variable(&mut self, name: &str, parents: &[&str], weights: &[f32], bias: f32) {
        let mut wmap = HashMap::new();
        for (p, wi) in parents.iter().zip(weights.iter()) {
            wmap.insert(p.to_string(), *wi);
        }
        self.nodes.insert(
            name.to_string(),
            SCMNode {
                name: name.to_string(),
                parents: parents.iter().map(|s| s.to_string()).collect(),
                weights: wmap,
                bias,
                noise_std: 0.1,
            },
        );
    }

    /// Compute value under intervention do(X=x).
    /// Returns map of all node values post-intervention.
    pub fn intervene(
        &mut self,
        do_var: &str,
        do_val: f32,
        evidence: &HashMap<String, f32>,
    ) -> HashMap<String, f32> {
        self.n_interventions += 1;
        let mut vals = evidence.clone();
        vals.insert(do_var.to_string(), do_val);
        // Topological-ish: iterate nodes, if parent values available, compute child
        let mut changed = true;
        let names: Vec<String> = self.nodes.keys().cloned().collect();
        while changed {
            changed = false;
            for name in &names {
                if vals.contains_key(name) {
                    continue;
                }
                if let Some(node) = self.nodes.get(name) {
                    let mut sum = node.bias;
                    let mut all_parents_known = true;
                    for p in &node.parents {
                        if let Some(&pv) = vals.get(p) {
                            sum += pv * node.weights.get(p).copied().unwrap_or(0.0);
                        } else {
                            all_parents_known = false;
                            break;
                        }
                    }
                    if all_parents_known {
                        vals.insert(name.clone(), sum);
                        changed = true;
                    }
                }
            }
        }
        vals
    }

    /// Counterfactual: what would Y be if X had been x, given evidence?
    pub fn counterfactual(
        &mut self,
        y_var: &str,
        do_var: &str,
        do_val: f32,
        evidence: &HashMap<String, f32>,
    ) -> Option<f32> {
        self.n_counterfactuals += 1;
        let post = self.intervene(do_var, do_val, evidence);
        post.get(y_var).copied()
    }

    /// Average causal effect ACE = E[Y | do(X=1)] - E[Y | do(X=0)]
    pub fn ace(&mut self, x_var: &str, y_var: &str) -> Option<f32> {
        let empty: HashMap<String, f32> = HashMap::new();
        let v1 = self.intervene(x_var, 1.0, &empty);
        let v0 = self.intervene(x_var, 0.0, &empty);
        let y1 = v1.get(y_var)?;
        let y0 = v0.get(y_var)?;
        Some(y1 - y0)
    }

    pub fn status(&self) -> String {
        format!(
            "SCM | vars={} | interv={} | cf={}",
            self.nodes.len(),
            self.n_interventions,
            self.n_counterfactuals
        )
    }
}
