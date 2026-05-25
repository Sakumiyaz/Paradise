// EDEN GARM Morphogenesis — Cognitive concept emergence via online clustering
// Concepts (Intents, Entities, Skills) are not hardcoded enums but emergent clusters
// in the neural embedding space. Tension = prediction error drives birth of new concepts.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Concept {
    pub id: u64,
    pub centroid: Vec<f32>,
    pub label: String,
    pub count: u32,
    pub birth_tick: u64,
    pub tension_accumulated: f32,
    pub parent_id: Option<u64>,
    pub children: Vec<u64>,
    // Neuro-symbolic: typed symbolic relations for compositional reasoning
    pub relations: HashMap<String, Vec<u64>>, // rel_type -> [target_concept_ids]
    // Movimiento E: abstraction hierarchy
    pub abstraction_level: u8, // 0=perception, 1=object, 2=category, 3=abstract
    pub properties: HashMap<String, String>, // property -> value (e.g. "es_comestible" -> "true")
}

pub struct ConceptSpace {
    pub concepts: HashMap<u64, Concept>,
    pub next_id: u64,
    pub min_samples_to_form: u32,
    pub creation_threshold: f32,
    pub tension_threshold: f32,
    pub transitions: HashMap<(u64, u64), u32>,
    pub evolved_alpha: Option<f32>, // set by meta-evolution before add_sample
}

impl ConceptSpace {
    pub fn new() -> Self {
        ConceptSpace {
            concepts: HashMap::new(),
            next_id: 1,
            min_samples_to_form: 2,
            creation_threshold: 0.35,
            tension_threshold: 0.5,
            transitions: HashMap::new(),
            evolved_alpha: None,
        }
    }

    fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len().min(b.len()).max(1);
        let sum_sq: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum();
        (sum_sq / len as f32).sqrt()
    }

    /// Classify an embedding. Returns the closest concept ID and distance.
    pub fn classify(&self, embedding: &[f32]) -> Option<(u64, f32)> {
        let mut best = None;
        let mut best_dist = f32::MAX;
        for (id, concept) in &self.concepts {
            let dist = Self::l2_distance(embedding, &concept.centroid);
            if dist < best_dist {
                best_dist = dist;
                best = Some((*id, dist));
            }
        }
        best
    }

    /// Add a sample. If it is close enough to an existing concept, updates centroid.
    /// If too far and tension is high enough, spawns a new concept.
    pub fn add_sample(
        &mut self,
        embedding: &[f32],
        raw_input: &str,
        tick: u64,
        tension: f32,
    ) -> (u64, bool) {
        let emb: Vec<f32> = embedding.to_vec();
        let closest = self.classify(&emb);

        if let Some((id, dist)) = closest {
            if dist < self.creation_threshold {
                // Assimilate into existing concept
                let concept = self.concepts.get_mut(&id).unwrap();
                let n = concept.count as f32;
                if let Some(alpha) = self.evolved_alpha {
                    // Meta-evolved update: use alpha to blend old centroid with new embedding
                    for i in 0..emb.len().min(concept.centroid.len()) {
                        let target = emb[i];
                        concept.centroid[i] = concept.centroid[i] * (1.0 - alpha) + target * alpha;
                    }
                } else {
                    // Default: simple moving average
                    for i in 0..emb.len().min(concept.centroid.len()) {
                        concept.centroid[i] = (concept.centroid[i] * n + emb[i]) / (n + 1.0);
                    }
                }
                concept.count += 1;
                concept.tension_accumulated += tension * 0.1;
                self.evolved_alpha = None; // consume after use
                return (id, false);
            }
        }

        // If tension is high, spawn new concept
        if tension > self.tension_threshold {
            let id = self.next_id;
            self.next_id += 1;
            let label = format!(
                "concept_{}_{}",
                id,
                &raw_input.split_whitespace().next().unwrap_or("unk")
            );

            let parent_id = closest
                .filter(|(_, dist)| *dist < self.creation_threshold * 0.5)
                .map(|(pid, _)| pid);

            if let Some(pid) = parent_id {
                if let Some(parent) = self.concepts.get_mut(&pid) {
                    parent.children.push(id);
                }
            }

            let abstraction_level = if parent_id.is_some() {
                let parent_level = parent_id
                    .and_then(|pid| self.concepts.get(&pid))
                    .map(|p| p.abstraction_level)
                    .unwrap_or(0);
                (parent_level + 1).min(3)
            } else {
                0
            };
            self.concepts.insert(
                id,
                Concept {
                    id,
                    centroid: emb.clone(),
                    label,
                    count: 1,
                    birth_tick: tick,
                    tension_accumulated: tension,
                    parent_id,
                    children: Vec::new(),
                    relations: HashMap::new(),
                    abstraction_level,
                    properties: HashMap::new(),
                },
            );
            return (id, true);
        }

        // No match, no spawn — return closest existing or 0
        (closest.map(|(id, _)| id).unwrap_or(0), false)
    }

    /// Remove concepts that are too weak or too old.
    pub fn prune(&mut self, min_count: u32, max_age: u64, tick: u64) {
        let to_remove: Vec<u64> = self
            .concepts
            .iter()
            .filter(|(_, c)| {
                c.count < min_count
                    || (tick.saturating_sub(c.birth_tick) > max_age && c.count < min_count * 2)
            })
            .map(|(id, _)| *id)
            .collect();

        for id in &to_remove {
            let (parent_id, children_ids) = self
                .concepts
                .get(id)
                .map(|c| (c.parent_id, c.children.clone()))
                .unwrap_or((None, Vec::new()));

            if let Some(pid) = parent_id {
                if let Some(parent) = self.concepts.get_mut(&pid) {
                    parent.children.retain(|&cid| cid != *id);
                }
            }

            for child_id in children_ids {
                if let Some(child) = self.concepts.get_mut(&child_id) {
                    child.parent_id = None;
                }
            }
        }

        for id in &to_remove {
            self.concepts.remove(id);
        }

        self.transitions
            .retain(|(from, to), _| !to_remove.contains(from) && !to_remove.contains(to));
    }

    /// Return IDs of all root concepts (no parent).
    pub fn get_roots(&self) -> Vec<u64> {
        self.concepts
            .values()
            .filter(|c| c.parent_id.is_none())
            .map(|c| c.id)
            .collect()
    }

    /// Return all concept IDs in the subtree rooted at `root_id` (DFS, includes root).
    pub fn get_subtree(&self, root_id: u64) -> Vec<u64> {
        let mut result = Vec::new();
        let mut stack = vec![root_id];
        while let Some(id) = stack.pop() {
            if let Some(concept) = self.concepts.get(&id) {
                result.push(id);
                for &child_id in &concept.children {
                    stack.push(child_id);
                }
            }
        }
        result
    }

    /// Record a temporal transition between two concepts.
    pub fn record_transition(&mut self, from_id: u64, to_id: u64) {
        *self.transitions.entry((from_id, to_id)).or_insert(0) += 1;
    }

    /// Predict the most frequent successor of a concept.
    pub fn predict_next_concept(&self, current_id: u64) -> Option<u64> {
        self.transitions
            .iter()
            .filter(|((from, _), _)| *from == current_id)
            .max_by_key(|(_, count)| *count)
            .map(|((_, to), _)| *to)
    }

    pub fn tension(&self) -> f32 {
        self.concepts
            .values()
            .map(|c| c.tension_accumulated)
            .sum::<f32>()
            / self.concepts.len().max(1) as f32
    }

    /// Prune the lowest-degree concepts when total exceeds `max_concepts`.
    /// Score = out_degree + count + (recency bonus). Removes lowest scorers first.
    /// Keeps relations consistent (removes any pointer to deleted concepts).
    /// Returns the number of concepts removed.
    pub fn prune_low_degree(&mut self, max_concepts: usize, tick: u64) -> usize {
        if self.concepts.len() <= max_concepts {
            return 0;
        }
        // Compute incoming degree for all concepts
        let mut in_deg: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
        for c in self.concepts.values() {
            for targets in c.relations.values() {
                for &t in targets {
                    *in_deg.entry(t).or_insert(0) += 1;
                }
            }
        }
        // Score each concept: lower score = more removable
        let mut scored: Vec<(u64, f32)> = self
            .concepts
            .values()
            .map(|c| {
                let out_deg: usize = c.relations.values().map(|v| v.len()).sum();
                let in_d = *in_deg.get(&c.id).unwrap_or(&0);
                let recency = 1.0 / (1.0 + tick.saturating_sub(c.birth_tick) as f32 * 0.001);
                let score = (out_deg + in_d) as f32 + (c.count as f32) * 0.1 + recency * 0.5;
                (c.id, score)
            })
            .collect();
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let n_to_remove = self.concepts.len() - max_concepts;
        let to_remove: Vec<u64> = scored
            .into_iter()
            .take(n_to_remove)
            .map(|(id, _)| id)
            .collect();
        let removed_set: std::collections::HashSet<u64> = to_remove.iter().copied().collect();
        // Remove the concepts
        for id in &to_remove {
            self.concepts.remove(id);
        }
        // Clean dangling references in remaining concepts
        for c in self.concepts.values_mut() {
            c.children.retain(|cid| !removed_set.contains(cid));
            if let Some(pid) = c.parent_id {
                if removed_set.contains(&pid) {
                    c.parent_id = None;
                }
            }
            for targets in c.relations.values_mut() {
                targets.retain(|t| !removed_set.contains(t));
            }
        }
        n_to_remove
    }

    pub fn n_concepts(&self) -> usize {
        self.concepts.len()
    }

    // ─── Symbolic relations and inference ───

    pub fn add_relation(&mut self, from_id: u64, rel_type: &str, to_id: u64) {
        if let Some(concept) = self.concepts.get_mut(&from_id) {
            concept
                .relations
                .entry(rel_type.to_string())
                .or_insert_with(Vec::new)
                .push(to_id);
        }
    }

    /// Infer all concepts reachable from `from_id` via transitive `rel_type`.
    pub fn infer_transitive(&self, from_id: u64, rel_type: &str) -> Vec<u64> {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![from_id];
        while let Some(id) = stack.pop() {
            if !visited.insert(id) {
                continue;
            }
            if let Some(concept) = self.concepts.get(&id) {
                if let Some(targets) = concept.relations.get(rel_type) {
                    for &t in targets {
                        stack.push(t);
                    }
                }
            }
        }
        visited.remove(&from_id);
        visited.into_iter().collect()
    }

    /// Find a path from `from_id` to `to_id` via `rel_type` (BFS shortest path).
    pub fn explain_path(&self, from_id: u64, to_id: u64, rel_type: &str) -> Option<Vec<u64>> {
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        let mut parent = std::collections::HashMap::new();
        queue.push_back(from_id);
        visited.insert(from_id);
        while let Some(current) = queue.pop_front() {
            if current == to_id {
                let mut path = vec![to_id];
                let mut node = to_id;
                while let Some(&p) = parent.get(&node) {
                    path.push(p);
                    node = p;
                }
                path.reverse();
                return Some(path);
            }
            if let Some(concept) = self.concepts.get(&current) {
                if let Some(targets) = concept.relations.get(rel_type) {
                    for &t in targets {
                        if visited.insert(t) {
                            parent.insert(t, current);
                            queue.push_back(t);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn relation_count(&self) -> usize {
        self.concepts
            .values()
            .map(|c| c.relations.values().map(|v| v.len()).sum::<usize>())
            .sum()
    }

    pub fn status(&self) -> String {
        format!(
            "Morphogenesis | concepts: {} | relations: {} | tension: {:.3}",
            self.concepts.len(),
            self.relation_count(),
            self.tension()
        )
    }

    // ─── Movimiento E: Abstraction Hierarchy ───

    /// Set a property on a concept. Properties can be inherited.
    pub fn set_property(&mut self, id: u64, key: &str, value: &str) {
        if let Some(c) = self.concepts.get_mut(&id) {
            c.properties.insert(key.to_string(), value.to_string());
        }
    }

    /// Get property, checking inheritance chain (bottom-up).
    /// If concept doesn't have property, checks parent recursively.
    pub fn get_property(&self, id: u64, key: &str) -> Option<String> {
        let mut current = Some(id);
        while let Some(cid) = current {
            if let Some(c) = self.concepts.get(&cid) {
                if let Some(val) = c.properties.get(key) {
                    return Some(val.clone());
                }
                current = c.parent_id;
            } else {
                break;
            }
        }
        None
    }

    /// Check if a concept has a property (own or inherited).
    pub fn has_property(&self, id: u64, key: &str) -> bool {
        self.get_property(id, key).is_some()
    }

    /// Infer property for all children of a concept.
    /// If "fruta" has "es_comestible=true", all children inherit it.
    pub fn propagate_property(&mut self, from_id: u64, key: &str) -> usize {
        let mut affected = 0usize;
        let value = match self.get_property(from_id, key) {
            Some(v) => v,
            None => return 0,
        };
        let children = self.get_subtree(from_id);
        for cid in children {
            if cid == from_id {
                continue;
            }
            if let Some(c) = self.concepts.get_mut(&cid) {
                if !c.properties.contains_key(key) {
                    c.properties.insert(key.to_string(), value.clone());
                    affected += 1;
                }
            }
        }
        affected
    }

    /// Get all concepts at a given abstraction level.
    pub fn concepts_at_level(&self, level: u8) -> Vec<u64> {
        self.concepts
            .values()
            .filter(|c| c.abstraction_level == level)
            .map(|c| c.id)
            .collect()
    }

    /// Compute abstraction distribution.
    pub fn abstraction_distribution(&self) -> [usize; 4] {
        let mut dist = [0usize; 4];
        for c in self.concepts.values() {
            let idx = (c.abstraction_level as usize).min(3);
            dist[idx] += 1;
        }
        dist
    }

    /// Infer category membership: if A has parent B, and B has property P,
    /// then A has property P (without explicit statement).
    pub fn infer_category_property(&self, id: u64, key: &str) -> Option<String> {
        self.get_property(id, key)
    }
}
