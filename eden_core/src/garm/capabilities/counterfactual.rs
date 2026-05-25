// EDEN GARM Counterfactual — Razonamiento contrafactual.
// 100% Rust puro, 0 LLM, 0 red.
//
// Pearl-style do-calculus simplificado: "Si X NO causara Y, ¿qué cambiaría?"
// Implementacion:
//   1. Identificar el concepto a "desactivar" (intervention)
//   2. Hacer una copia del grafo causal con ese concepto sin sus efectos
//   3. Comparar transitive_effects originales vs intervened
//   4. Reportar diferencia: efectos que ya no ocurrirían

use crate::eden_garm::capabilities::inference::InferenceEngine;
use crate::eden_garm::capabilities::morphogenesis::ConceptSpace;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct CounterfactualResult {
    pub intervention: String,
    pub intervention_id: u64,
    pub original_downstream: Vec<(u64, String)>,
    pub intervened_downstream: Vec<(u64, String)>,
    pub eliminated: Vec<(u64, String)>,
    pub still_present: Vec<(u64, String)>,
}

#[derive(Clone, Debug)]
pub struct CounterfactualEngine {
    pub n_interventions: u64,
    pub history: Vec<CounterfactualResult>,
    pub max_history: usize,
}

impl CounterfactualEngine {
    pub fn new() -> Self {
        CounterfactualEngine {
            n_interventions: 0,
            history: Vec::new(),
            max_history: 50,
        }
    }

    /// Compute downstream effects of a concept normally vs after removing its outgoing edges.
    /// Returns full result including which effects would no longer occur.
    pub fn intervene(
        &mut self,
        space: &ConceptSpace,
        query: &str,
        max_depth: usize,
    ) -> Option<CounterfactualResult> {
        self.n_interventions += 1;
        let matches = InferenceEngine::find_concepts_by_label(space, query);
        if matches.is_empty() {
            return None;
        }
        let target = matches[0];
        let target_label = space
            .concepts
            .get(&target)
            .map(|c| c.label.clone())
            .unwrap_or_default();

        // Original downstream
        let original = InferenceEngine::transitive_effects(space, target, max_depth);
        let _original_set: HashSet<u64> = original.iter().map(|(id, _, _)| *id).collect();

        // Build a clone of the space with target's outgoing causes removed, plus
        // remove ALL paths that go through target (since intervention=disable target).
        // Use BFS on a modified adjacency.
        let intervened = self.compute_downstream_without(space, target, max_depth);
        let intervened_set: HashSet<u64> = intervened.iter().map(|(id, _, _)| *id).collect();

        // Effects that disappear under intervention
        let eliminated: Vec<(u64, String)> = original
            .iter()
            .filter(|(id, _, _)| !intervened_set.contains(id))
            .map(|(id, _, lbl)| (*id, lbl.clone()))
            .collect();
        let still_present: Vec<(u64, String)> = original
            .iter()
            .filter(|(id, _, _)| intervened_set.contains(id))
            .map(|(id, _, lbl)| (*id, lbl.clone()))
            .collect();

        let original_pairs: Vec<(u64, String)> =
            original.iter().map(|(id, _, l)| (*id, l.clone())).collect();
        let intervened_pairs: Vec<(u64, String)> = intervened
            .iter()
            .map(|(id, _, l)| (*id, l.clone()))
            .collect();

        let result = CounterfactualResult {
            intervention: target_label,
            intervention_id: target,
            original_downstream: original_pairs,
            intervened_downstream: intervened_pairs,
            eliminated,
            still_present,
        };
        self.history.push(result.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        Some(result)
    }

    /// BFS downstream from any concept that points TO `target`, but skipping `target` itself.
    /// This simulates the world where `target` does not exist as a causal node.
    fn compute_downstream_without(
        &self,
        space: &ConceptSpace,
        removed: u64,
        max_depth: usize,
    ) -> Vec<(u64, usize, String)> {
        // Find all "root causes" that originally caused target
        let predecessors = InferenceEngine::find_predecessors(space, removed, "causes");
        let mut out: Vec<(u64, usize, String)> = Vec::new();
        let mut visited: HashSet<u64> = HashSet::new();
        visited.insert(removed);
        let mut frontier: std::collections::VecDeque<(u64, usize)> =
            std::collections::VecDeque::new();
        // Start from each predecessor (they exist in the intervened world too)
        for p in predecessors {
            visited.insert(p);
            frontier.push_back((p, 0));
        }
        while let Some((id, depth)) = frontier.pop_front() {
            if depth >= max_depth {
                continue;
            }
            for effect_id in InferenceEngine::find_successors(space, id, "causes") {
                if effect_id == removed {
                    continue;
                } // skip the intervened node
                if visited.insert(effect_id) {
                    if let Some(c) = space.concepts.get(&effect_id) {
                        out.push((effect_id, depth + 1, c.label.clone()));
                    }
                    frontier.push_back((effect_id, depth + 1));
                }
            }
        }
        out
    }

    /// "What if X happened instead of Y?" - swap intervention.
    /// For each effect of Y, check if X has a similar effect (by label overlap).
    pub fn substitute(
        &mut self,
        space: &ConceptSpace,
        original_query: &str,
        alternative_query: &str,
    ) -> Option<(Vec<(u64, String)>, Vec<(u64, String)>)> {
        self.n_interventions += 1;
        let original = InferenceEngine::find_concepts_by_label(space, original_query);
        let alternative = InferenceEngine::find_concepts_by_label(space, alternative_query);
        if original.is_empty() || alternative.is_empty() {
            return None;
        }
        let orig_id = original[0];
        let alt_id = alternative[0];
        let orig_effects = InferenceEngine::find_successors(space, orig_id, "causes");
        let alt_effects = InferenceEngine::find_successors(space, alt_id, "causes");
        let orig_pairs: Vec<(u64, String)> = orig_effects
            .iter()
            .filter_map(|id| space.concepts.get(id).map(|c| (*id, c.label.clone())))
            .collect();
        let alt_pairs: Vec<(u64, String)> = alt_effects
            .iter()
            .filter_map(|id| space.concepts.get(id).map(|c| (*id, c.label.clone())))
            .collect();
        Some((orig_pairs, alt_pairs))
    }

    pub fn render_intervention(r: &CounterfactualResult) -> String {
        let mut out = format!(
            "Si '{}' (id={}) NO ocurriera:\n",
            r.intervention, r.intervention_id
        );
        out.push_str(&format!(
            "\n  Originalmente causaba {} efectos.\n",
            r.original_downstream.len()
        ));
        out.push_str(&format!(
            "  Bajo la intervencion, solo {} efectos persistirian.\n",
            r.intervened_downstream.len()
        ));
        if !r.eliminated.is_empty() {
            out.push_str(&format!(
                "\n  EFECTOS ELIMINADOS ({}):\n",
                r.eliminated.len()
            ));
            for (id, lbl) in r.eliminated.iter().take(10) {
                out.push_str(&format!("    - '{}' (id={})\n", lbl, id));
            }
        }
        if !r.still_present.is_empty() {
            out.push_str(&format!(
                "\n  EFECTOS QUE PERSISTEN (alcanzados por otras causas, {}):\n",
                r.still_present.len()
            ));
            for (id, lbl) in r.still_present.iter().take(5) {
                out.push_str(&format!("    + '{}' (id={})\n", lbl, id));
            }
        }
        out
    }

    pub fn status(&self) -> String {
        format!(
            "Counterfactual | interventions={} | history={}",
            self.n_interventions,
            self.history.len()
        )
    }
}
