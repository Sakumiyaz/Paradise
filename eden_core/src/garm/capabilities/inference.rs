// EDEN GARM Inference — Query operations over the learned causal/relational graph.
// 100% Rust puro, 0 LLM. Lee morphogenesis.concepts y sus relations.
//
// Operations:
//   find_concepts_by_label   - fuzzy substring match over concept labels
//   find_causes_of           - "what causes X?"
//   find_effects_of          - "what does X cause?"
//   causal_chain             - shortest causal path from A to B
//   most_central             - hub concepts (high in/out causal degree)
//   summarize_concept        - human-readable summary of a concept
//
// All queries take or return concept IDs and human-readable strings.

use crate::eden_garm::capabilities::morphogenesis::ConceptSpace;
use std::collections::HashMap;

pub struct InferenceEngine;

impl InferenceEngine {
    /// Find concept IDs whose label contains the substring (case-insensitive).
    /// Sorted by (total causal degree desc, frequency desc) so concepts with relations rank first.
    pub fn find_concepts_by_label(space: &ConceptSpace, query: &str) -> Vec<u64> {
        let q = query.to_lowercase();
        let mut out: Vec<(u64, usize, u32)> = space
            .concepts
            .values()
            .filter(|c| c.label.to_lowercase().contains(&q))
            .map(|c| {
                let out_deg = c.relations.get("causes").map(|v| v.len()).unwrap_or(0);
                let in_deg = Self::find_predecessors(space, c.id, "causes").len();
                (c.id, out_deg + in_deg, c.count)
            })
            .collect();
        // Sort by total causal degree desc, then count desc
        out.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
        out.into_iter().map(|(id, _, _)| id).collect()
    }

    /// Find concepts that have a `rel_type` edge pointing TO `target_id`.
    /// I.e., "what causes target_id?" when rel_type = "causes".
    pub fn find_predecessors(space: &ConceptSpace, target_id: u64, rel_type: &str) -> Vec<u64> {
        let mut preds = Vec::new();
        for c in space.concepts.values() {
            if let Some(targets) = c.relations.get(rel_type) {
                if targets.contains(&target_id) {
                    preds.push(c.id);
                }
            }
        }
        preds
    }

    /// Find concepts that `source_id` has a `rel_type` edge TO.
    /// I.e., "what does source_id cause?" when rel_type = "causes".
    pub fn find_successors(space: &ConceptSpace, source_id: u64, rel_type: &str) -> Vec<u64> {
        space
            .concepts
            .get(&source_id)
            .and_then(|c| c.relations.get(rel_type))
            .cloned()
            .unwrap_or_default()
    }

    /// Highest-level query: "what causes X?" by label.
    /// Returns (cause_label, cause_id) pairs.
    pub fn find_causes_of(space: &ConceptSpace, query: &str) -> Vec<(String, u64)> {
        let matches = Self::find_concepts_by_label(space, query);
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for target in matches {
            for cause_id in Self::find_predecessors(space, target, "causes") {
                if seen.insert(cause_id) {
                    if let Some(c) = space.concepts.get(&cause_id) {
                        out.push((c.label.clone(), cause_id));
                    }
                }
            }
        }
        out
    }

    /// "What does X cause?" by label.
    pub fn find_effects_of(space: &ConceptSpace, query: &str) -> Vec<(String, u64)> {
        let matches = Self::find_concepts_by_label(space, query);
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for source in matches {
            for effect_id in Self::find_successors(space, source, "causes") {
                if seen.insert(effect_id) {
                    if let Some(c) = space.concepts.get(&effect_id) {
                        out.push((c.label.clone(), effect_id));
                    }
                }
            }
        }
        out
    }

    /// Shortest causal chain from concept matching `from_query` to one matching `to_query`.
    /// Returns the labels in path order.
    pub fn causal_chain(
        space: &ConceptSpace,
        from_query: &str,
        to_query: &str,
    ) -> Option<Vec<String>> {
        let from_ids = Self::find_concepts_by_label(space, from_query);
        let to_ids = Self::find_concepts_by_label(space, to_query);
        let to_set: std::collections::HashSet<u64> = to_ids.iter().copied().collect();
        // BFS from any from_id until we hit any to_id
        for &start in &from_ids {
            let mut queue = std::collections::VecDeque::new();
            let mut visited = std::collections::HashSet::new();
            let mut parent: HashMap<u64, u64> = HashMap::new();
            queue.push_back(start);
            visited.insert(start);
            while let Some(current) = queue.pop_front() {
                if to_set.contains(&current) && current != start {
                    // Reconstruct
                    let mut path = vec![current];
                    let mut node = current;
                    while let Some(&p) = parent.get(&node) {
                        path.push(p);
                        node = p;
                    }
                    path.reverse();
                    return Some(
                        path.into_iter()
                            .filter_map(|id| space.concepts.get(&id).map(|c| c.label.clone()))
                            .collect(),
                    );
                }
                if let Some(c) = space.concepts.get(&current) {
                    if let Some(targets) = c.relations.get("causes") {
                        for &t in targets {
                            if visited.insert(t) {
                                parent.insert(t, current);
                                queue.push_back(t);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Top N concepts by (in_degree + out_degree) in the causal graph.
    /// These are the "hub" concepts that connect the most ideas.
    pub fn most_central(space: &ConceptSpace, top_n: usize) -> Vec<(String, u64, usize, usize)> {
        let mut in_deg: HashMap<u64, usize> = HashMap::new();
        let mut out_deg: HashMap<u64, usize> = HashMap::new();
        for c in space.concepts.values() {
            if let Some(targets) = c.relations.get("causes") {
                *out_deg.entry(c.id).or_insert(0) += targets.len();
                for &t in targets {
                    *in_deg.entry(t).or_insert(0) += 1;
                }
            }
        }
        let mut scored: Vec<(u64, usize, usize)> = space
            .concepts
            .keys()
            .map(|&id| {
                (
                    id,
                    *in_deg.get(&id).unwrap_or(&0),
                    *out_deg.get(&id).unwrap_or(&0),
                )
            })
            .filter(|&(_, i, o)| i + o > 0)
            .collect();
        scored.sort_by(|a, b| (b.1 + b.2).cmp(&(a.1 + a.2)));
        scored
            .into_iter()
            .take(top_n)
            .filter_map(|(id, ind, outd)| {
                space
                    .concepts
                    .get(&id)
                    .map(|c| (c.label.clone(), id, ind, outd))
            })
            .collect()
    }

    /// Human-readable summary of a concept by id.
    pub fn summarize_concept(space: &ConceptSpace, id: u64) -> Option<String> {
        let c = space.concepts.get(&id)?;
        let n_rel: usize = c.relations.values().map(|v| v.len()).sum();
        let causes_in = Self::find_predecessors(space, id, "causes").len();
        let causes_out = c.relations.get("causes").map(|v| v.len()).unwrap_or(0);
        Some(format!(
            "Concept {} | label='{}' | seen={} times | birth_tick={} | total_relations={} | caused_by={} | causes={}",
            id, c.label, c.count, c.birth_tick, n_rel, causes_in, causes_out
        ))
    }

    /// Render a textual answer to a question like "why does X happen?".
    /// Walks back through the causal graph from X to root causes.
    pub fn explain_why(space: &ConceptSpace, query: &str, max_depth: usize) -> String {
        let matches = Self::find_concepts_by_label(space, query);
        if matches.is_empty() {
            return format!("No tengo conceptos que coincidan con '{}'", query);
        }
        let target = matches[0];
        let target_label = space
            .concepts
            .get(&target)
            .map(|c| c.label.clone())
            .unwrap_or_default();
        let mut lines = vec![format!("Para entender '{}' (id={}):", target_label, target)];
        let mut frontier = vec![(target, 0usize)];
        let mut visited = std::collections::HashSet::new();
        visited.insert(target);
        let mut found_any = false;
        while let Some((id, depth)) = frontier.pop() {
            if depth >= max_depth {
                continue;
            }
            let causes = Self::find_predecessors(space, id, "causes");
            for c in causes {
                if visited.insert(c) {
                    if let (Some(cc), Some(ic)) = (space.concepts.get(&c), space.concepts.get(&id))
                    {
                        let indent = "  ".repeat(depth + 1);
                        lines.push(format!("{}'{}' causa '{}'", indent, cc.label, ic.label));
                        found_any = true;
                        frontier.push((c, depth + 1));
                    }
                }
            }
        }
        if !found_any {
            lines.push(format!(
                "  (no he aprendido causas para '{}' aun)",
                target_label
            ));
        }
        lines.join("\n")
    }

    /// All concepts transitively upstream of `target_id` via "causes" edges (BFS).
    /// Returns a list of (concept_id, depth, label) sorted by depth ascending.
    pub fn transitive_causes(
        space: &ConceptSpace,
        target_id: u64,
        max_depth: usize,
    ) -> Vec<(u64, usize, String)> {
        let mut out = Vec::new();
        let mut visited = std::collections::HashSet::new();
        visited.insert(target_id);
        let mut frontier: std::collections::VecDeque<(u64, usize)> =
            std::collections::VecDeque::new();
        frontier.push_back((target_id, 0));
        while let Some((id, depth)) = frontier.pop_front() {
            if depth >= max_depth {
                continue;
            }
            for cause_id in Self::find_predecessors(space, id, "causes") {
                if visited.insert(cause_id) {
                    if let Some(c) = space.concepts.get(&cause_id) {
                        out.push((cause_id, depth + 1, c.label.clone()));
                    }
                    frontier.push_back((cause_id, depth + 1));
                }
            }
        }
        out
    }

    /// All concepts transitively downstream of `source_id` via "causes" edges (BFS).
    pub fn transitive_effects(
        space: &ConceptSpace,
        source_id: u64,
        max_depth: usize,
    ) -> Vec<(u64, usize, String)> {
        let mut out = Vec::new();
        let mut visited = std::collections::HashSet::new();
        visited.insert(source_id);
        let mut frontier: std::collections::VecDeque<(u64, usize)> =
            std::collections::VecDeque::new();
        frontier.push_back((source_id, 0));
        while let Some((id, depth)) = frontier.pop_front() {
            if depth >= max_depth {
                continue;
            }
            for effect_id in Self::find_successors(space, id, "causes") {
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

    /// Find concepts that are upstream causes of BOTH a_query and b_query.
    /// I.e., "what is the common cause of X and Y?".
    pub fn common_cause(
        space: &ConceptSpace,
        a_query: &str,
        b_query: &str,
        max_depth: usize,
    ) -> Vec<(String, u64, usize, usize)> {
        let a_ids = Self::find_concepts_by_label(space, a_query);
        let b_ids = Self::find_concepts_by_label(space, b_query);
        if a_ids.is_empty() || b_ids.is_empty() {
            return Vec::new();
        }
        let a = a_ids[0];
        let b = b_ids[0];
        let a_causes: std::collections::HashMap<u64, usize> =
            Self::transitive_causes(space, a, max_depth)
                .into_iter()
                .map(|(id, d, _)| (id, d))
                .collect();
        let b_causes: std::collections::HashMap<u64, usize> =
            Self::transitive_causes(space, b, max_depth)
                .into_iter()
                .map(|(id, d, _)| (id, d))
                .collect();
        let mut out: Vec<(String, u64, usize, usize)> = Vec::new();
        for (id, da) in &a_causes {
            if let Some(&db) = b_causes.get(id) {
                if let Some(c) = space.concepts.get(id) {
                    out.push((c.label.clone(), *id, *da, db));
                }
            }
        }
        // Sort by total depth ascending (closest common cause first)
        out.sort_by_key(|(_, _, da, db)| da + db);
        out
    }

    /// Detect groups of concepts that share at least `min_shared` significant keywords.
    /// Returns groups of concept IDs that form an abstract pattern.
    /// E.g., concepts containing "agua", "fluye", "cae" might form a "water flow" pattern.
    pub fn abstract_patterns(
        space: &ConceptSpace,
        min_shared: usize,
        min_keyword_len: usize,
    ) -> Vec<(String, Vec<(u64, String)>)> {
        // Collect significant keywords per concept (filter stopwords-like by length)
        let stopwords: std::collections::HashSet<&str> = [
            "el", "la", "los", "las", "un", "una", "unos", "unas", "de", "del", "al", "en", "con",
            "por", "para", "sin", "que", "es", "son", "no", "si", "y", "o", "u", "a", "se", "su",
            "sus", "lo", "le", "les", "mi", "tu", "te", "me", "the", "a", "an", "of", "in", "on",
            "at", "by", "for", "to", "from", "with", "without", "but", "and", "or", "is", "are",
            "was", "were", "be", "been", "being", "not", "no", "yes", "this", "that", "these",
            "those", "it", "its", "he", "she", "they", "them", "his", "her", "their", "my", "your",
            "our",
        ]
        .into_iter()
        .collect();
        let mut keyword_to_concepts: std::collections::HashMap<String, Vec<u64>> =
            std::collections::HashMap::new();
        for c in space.concepts.values() {
            let words: std::collections::HashSet<String> = c
                .label
                .to_lowercase()
                .split_whitespace()
                .filter(|w| w.len() >= min_keyword_len && !stopwords.contains(w))
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                .filter(|w| !w.is_empty())
                .collect();
            for w in words {
                keyword_to_concepts
                    .entry(w)
                    .or_insert_with(Vec::new)
                    .push(c.id);
            }
        }
        // Keep only keywords shared by >= min_shared concepts
        let mut patterns: Vec<(String, Vec<(u64, String)>)> = keyword_to_concepts
            .into_iter()
            .filter(|(_, ids)| ids.len() >= min_shared)
            .map(|(kw, ids)| {
                let labeled: Vec<(u64, String)> = ids
                    .iter()
                    .filter_map(|id| space.concepts.get(id).map(|c| (*id, c.label.clone())))
                    .collect();
                (kw, labeled)
            })
            .collect();
        // Sort by group size descending
        patterns.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        patterns
    }

    /// Auto-derive transitive causal relations as new "transitively_causes" edges.
    /// For each (A, B, C) with A causes B and B causes C, ensure A also has a transitively_causes edge to C.
    /// Returns the number of new edges added.
    pub fn derive_transitive_closure(space: &mut ConceptSpace, max_depth: usize) -> usize {
        let ids: Vec<u64> = space.concepts.keys().copied().collect();
        let mut to_add: Vec<(u64, u64)> = Vec::new();
        for &a in &ids {
            let downstream = Self::transitive_effects(space, a, max_depth);
            for (target_id, depth, _) in downstream {
                if depth > 1 {
                    // Only add if not already directly connected
                    let already = space
                        .concepts
                        .get(&a)
                        .and_then(|c| c.relations.get("transitively_causes"))
                        .map(|v| v.contains(&target_id))
                        .unwrap_or(false);
                    if !already {
                        to_add.push((a, target_id));
                    }
                }
            }
        }
        let n = to_add.len();
        for (from, to) in to_add {
            space.add_relation(from, "transitively_causes", to);
        }
        n
    }

    /// Backward chain from a goal: find sub-goals that, if achieved, would cause the goal.
    /// Returns a tree-shaped plan: list of (concept_id, depth, label, parent_id).
    /// The "leaves" (highest depth, no further upstream causes) are the actionable sub-goals.
    pub fn backward_chain_to_goal(
        space: &ConceptSpace,
        goal_query: &str,
        max_depth: usize,
    ) -> Vec<(u64, usize, String, Option<u64>)> {
        let matches = Self::find_concepts_by_label(space, goal_query);
        if matches.is_empty() {
            return Vec::new();
        }
        let goal = matches[0];
        let mut plan: Vec<(u64, usize, String, Option<u64>)> = Vec::new();
        if let Some(c) = space.concepts.get(&goal) {
            plan.push((goal, 0, c.label.clone(), None));
        }
        let mut visited = std::collections::HashSet::new();
        visited.insert(goal);
        let mut frontier: std::collections::VecDeque<(u64, usize)> =
            std::collections::VecDeque::new();
        frontier.push_back((goal, 0));
        while let Some((current, depth)) = frontier.pop_front() {
            if depth >= max_depth {
                continue;
            }
            for cause in Self::find_predecessors(space, current, "causes") {
                if visited.insert(cause) {
                    if let Some(c) = space.concepts.get(&cause) {
                        plan.push((cause, depth + 1, c.label.clone(), Some(current)));
                    }
                    frontier.push_back((cause, depth + 1));
                }
            }
        }
        plan
    }

    /// Forward predict: given a current state/concept, predict expected effects via causal chain.
    /// Returns a tree of (concept_id, depth, label, predicted_via).
    pub fn forward_predict(
        space: &ConceptSpace,
        state_query: &str,
        max_depth: usize,
    ) -> Vec<(u64, usize, String, Option<u64>)> {
        let matches = Self::find_concepts_by_label(space, state_query);
        if matches.is_empty() {
            return Vec::new();
        }
        let start = matches[0];
        let mut predictions: Vec<(u64, usize, String, Option<u64>)> = Vec::new();
        if let Some(c) = space.concepts.get(&start) {
            predictions.push((start, 0, c.label.clone(), None));
        }
        let mut visited = std::collections::HashSet::new();
        visited.insert(start);
        let mut frontier: std::collections::VecDeque<(u64, usize)> =
            std::collections::VecDeque::new();
        frontier.push_back((start, 0));
        while let Some((current, depth)) = frontier.pop_front() {
            if depth >= max_depth {
                continue;
            }
            for effect in Self::find_successors(space, current, "causes") {
                if visited.insert(effect) {
                    if let Some(c) = space.concepts.get(&effect) {
                        predictions.push((effect, depth + 1, c.label.clone(), Some(current)));
                    }
                    frontier.push_back((effect, depth + 1));
                }
            }
        }
        predictions
    }

    /// Render a backward-chain plan as a textual tree.
    pub fn render_plan(plan: &[(u64, usize, String, Option<u64>)]) -> String {
        if plan.is_empty() {
            return "(plan vacio)".to_string();
        }
        let goal = &plan[0];
        let mut out = format!("META: '{}' (id={})\n", goal.2, goal.0);
        // Sort by depth ascending so the tree reads top-down
        let mut sorted: Vec<&(u64, usize, String, Option<u64>)> = plan.iter().skip(1).collect();
        sorted.sort_by_key(|(_, d, _, _)| *d);
        for (id, depth, label, parent) in sorted {
            let indent = "  ".repeat(*depth);
            let arrow = if *depth == plan[0].1 + 1 {
                " <- causa directa de meta"
            } else {
                ""
            };
            let parent_str = parent
                .map(|p| format!(" (sub-meta de id={})", p))
                .unwrap_or_default();
            out.push_str(&format!(
                "{}[d={}] '{}' (id={}){}{}\n",
                indent, depth, label, id, parent_str, arrow
            ));
        }
        // Identify leaves (no further upstream causes)
        let parent_set: std::collections::HashSet<u64> =
            plan.iter().filter_map(|(_, _, _, p)| *p).collect();
        let leaves: Vec<&(u64, usize, String, Option<u64>)> = plan
            .iter()
            .filter(|(id, depth, _, _)| *depth > 0 && !parent_set.contains(id))
            .collect();
        if !leaves.is_empty() {
            out.push_str("\nSUB-METAS ACCIONABLES (ningun upstream conocido):\n");
            for (id, _, label, _) in leaves {
                out.push_str(&format!("  -> '{}' (id={})\n", label, id));
            }
        }
        out
    }
}
