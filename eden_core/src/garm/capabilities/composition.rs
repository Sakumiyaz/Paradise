// EDEN GARM Composition — Conjunctive causation y patrones compositivos.
// 100% Rust puro, 0 LLM, 0 red.
//
// La causalidad pairwise (X causes Y) ya esta implementada en morphogenesis.
// Aqui detectamos causalidad CONJUNTIVA: cuando varias causas convergen en
// un mismo efecto y juntas son necesarias para explicarlo.
//
// Conceptos:
//   - Cluster conjunctivo: set de >= 2 causas que comparten un mismo efecto
//     (estructura "fan-in" en el grafo causal)
//   - Recurring composition: el mismo set de causas explica multiples efectos
//   - Compositional query: "que se necesita para que pase X?" devuelve el
//     conjunto completo, no solo una causa

use crate::eden_garm::capabilities::morphogenesis::ConceptSpace;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct ConjunctiveCluster {
    pub effect_id: u64,
    pub effect_label: String,
    pub causes: Vec<(u64, String)>, // sorted by id
    pub n_causes: usize,
}

#[derive(Clone, Debug)]
pub struct RecurringComposition {
    /// Hash-key of the cause set (sorted ids joined)
    pub cause_set_key: String,
    pub causes: Vec<(u64, String)>,
    /// Effects that this set jointly causes
    pub effects: Vec<(u64, String)>,
    pub n_effects: usize,
}

#[derive(Clone, Debug)]
pub struct CompositionEngine {
    pub min_causes_for_cluster: usize,
    pub min_effects_for_recurring: usize,
    pub last_clusters: Vec<ConjunctiveCluster>,
    pub last_recurring: Vec<RecurringComposition>,
    pub n_analyses: u64,
}

impl CompositionEngine {
    pub fn new() -> Self {
        CompositionEngine {
            min_causes_for_cluster: 2,
            min_effects_for_recurring: 2,
            last_clusters: Vec::new(),
            last_recurring: Vec::new(),
            n_analyses: 0,
        }
    }

    /// Detect concepts that have >= min_causes_for_cluster predecessors via "causes".
    /// Each such concept defines a conjunctive cluster.
    pub fn detect_conjunctive_clusters(&mut self, space: &ConceptSpace) -> Vec<ConjunctiveCluster> {
        // Compute reverse adjacency: for each effect, list of unique causes
        let mut reverse: HashMap<u64, HashSet<u64>> = HashMap::new();
        for c in space.concepts.values() {
            if let Some(targets) = c.relations.get("causes") {
                for &t in targets {
                    reverse.entry(t).or_insert_with(HashSet::new).insert(c.id);
                }
            }
        }
        let mut clusters: Vec<ConjunctiveCluster> = reverse
            .into_iter()
            .filter(|(_, causes)| causes.len() >= self.min_causes_for_cluster)
            .map(|(eid, cause_set)| {
                let mut causes: Vec<u64> = cause_set.into_iter().collect();
                causes.sort();
                let effect_label = space
                    .concepts
                    .get(&eid)
                    .map(|c| c.label.clone())
                    .unwrap_or_default();
                let labelled_causes: Vec<(u64, String)> = causes
                    .iter()
                    .map(|cid| {
                        let lbl = space
                            .concepts
                            .get(cid)
                            .map(|c| c.label.clone())
                            .unwrap_or_default();
                        (*cid, lbl)
                    })
                    .collect();
                let n = labelled_causes.len();
                ConjunctiveCluster {
                    effect_id: eid,
                    effect_label,
                    causes: labelled_causes,
                    n_causes: n,
                }
            })
            .collect();
        clusters.sort_by(|a, b| b.n_causes.cmp(&a.n_causes));
        self.last_clusters = clusters.clone();
        self.n_analyses += 1;
        clusters
    }

    /// Detect recurring compositions: same set of causes that jointly causes
    /// multiple distinct effects. These are the "templates" of the causal graph.
    pub fn detect_recurring_compositions(
        &mut self,
        space: &ConceptSpace,
    ) -> Vec<RecurringComposition> {
        // For each concept, get its causes set (sorted ids).
        // Then group by causes-set and collect effects.
        let mut by_causes: HashMap<Vec<u64>, Vec<u64>> = HashMap::new();

        // Build reverse adjacency first (deduped)
        let mut reverse: HashMap<u64, HashSet<u64>> = HashMap::new();
        for c in space.concepts.values() {
            if let Some(targets) = c.relations.get("causes") {
                for &t in targets {
                    reverse.entry(t).or_insert_with(HashSet::new).insert(c.id);
                }
            }
        }

        for (effect_id, cause_set) in reverse {
            if cause_set.len() < self.min_causes_for_cluster {
                continue;
            }
            let mut causes: Vec<u64> = cause_set.into_iter().collect();
            causes.sort();
            by_causes
                .entry(causes)
                .or_insert_with(Vec::new)
                .push(effect_id);
        }

        let mut recurring: Vec<RecurringComposition> = by_causes
            .into_iter()
            .filter(|(_, effects)| effects.len() >= self.min_effects_for_recurring)
            .map(|(causes, effects)| {
                let cause_set_key = causes
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let labelled_causes: Vec<(u64, String)> = causes
                    .iter()
                    .map(|cid| {
                        let lbl = space
                            .concepts
                            .get(cid)
                            .map(|c| c.label.clone())
                            .unwrap_or_default();
                        (*cid, lbl)
                    })
                    .collect();
                let labelled_effects: Vec<(u64, String)> = effects
                    .iter()
                    .map(|eid| {
                        let lbl = space
                            .concepts
                            .get(eid)
                            .map(|c| c.label.clone())
                            .unwrap_or_default();
                        (*eid, lbl)
                    })
                    .collect();
                let n = labelled_effects.len();
                RecurringComposition {
                    cause_set_key,
                    causes: labelled_causes,
                    effects: labelled_effects,
                    n_effects: n,
                }
            })
            .collect();
        recurring.sort_by(|a, b| b.n_effects.cmp(&a.n_effects));
        self.last_recurring = recurring.clone();
        recurring
    }

    /// Get the conjunctive cause set for a specific concept (or single cause if not conjunctive).
    pub fn query_conjunctive_for(
        &self,
        space: &ConceptSpace,
        target_id: u64,
    ) -> Option<ConjunctiveCluster> {
        let mut cause_set: HashSet<u64> = HashSet::new();
        for c in space.concepts.values() {
            if let Some(targets) = c.relations.get("causes") {
                if targets.contains(&target_id) {
                    cause_set.insert(c.id);
                }
            }
        }
        if cause_set.is_empty() {
            return None;
        }
        let mut causes: Vec<u64> = cause_set.into_iter().collect();
        causes.sort();
        let effect_label = space
            .concepts
            .get(&target_id)
            .map(|c| c.label.clone())
            .unwrap_or_default();
        let labelled_causes: Vec<(u64, String)> = causes
            .iter()
            .map(|cid| {
                let lbl = space
                    .concepts
                    .get(cid)
                    .map(|c| c.label.clone())
                    .unwrap_or_default();
                (*cid, lbl)
            })
            .collect();
        let n = labelled_causes.len();
        Some(ConjunctiveCluster {
            effect_id: target_id,
            effect_label,
            causes: labelled_causes,
            n_causes: n,
        })
    }

    pub fn report_clusters(&self, max: usize) -> String {
        if self.last_clusters.is_empty() {
            return "Sin clusters conjuntivos detectados (ejecutar clusters primero)".to_string();
        }
        let mut out = format!(
            "Top {} clusters conjuntivos (efectos con >= {} causas):\n",
            max.min(self.last_clusters.len()),
            self.min_causes_for_cluster
        );
        for cl in self.last_clusters.iter().take(max) {
            out.push_str(&format!(
                "\n  EFECTO: '{}' (id={}) | {} causas convergentes:\n",
                cl.effect_label, cl.effect_id, cl.n_causes
            ));
            for (cid, lbl) in cl.causes.iter().take(8) {
                out.push_str(&format!("    + '{}' (id={})\n", lbl, cid));
            }
            if cl.causes.len() > 8 {
                out.push_str(&format!("    ... ({} mas)\n", cl.causes.len() - 8));
            }
        }
        out
    }

    pub fn report_recurring(&self, max: usize) -> String {
        if self.last_recurring.is_empty() {
            return "Sin composiciones recurrentes detectadas (ejecutar recurring primero)"
                .to_string();
        }
        let mut out = format!(
            "Top {} composiciones recurrentes (mismo set causa >= {} efectos):\n",
            max.min(self.last_recurring.len()),
            self.min_effects_for_recurring
        );
        for rc in self.last_recurring.iter().take(max) {
            out.push_str("\n  CAUSAS:\n");
            for (cid, lbl) in rc.causes.iter().take(5) {
                out.push_str(&format!("    + '{}' (id={})\n", lbl, cid));
            }
            out.push_str(&format!("  EFECTOS ({} distintos):\n", rc.n_effects));
            for (eid, lbl) in rc.effects.iter().take(5) {
                out.push_str(&format!("    -> '{}' (id={})\n", lbl, eid));
            }
        }
        out
    }

    pub fn status(&self) -> String {
        format!(
            "Composition | analyses={} | last_clusters={} | last_recurring={}",
            self.n_analyses,
            self.last_clusters.len(),
            self.last_recurring.len(),
        )
    }
}
