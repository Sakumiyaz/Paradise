use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelType {
    IsA,
    Causes,
    HasProperty,
    Opposes,
}

#[derive(Clone, Debug)]
pub struct KnowledgeEdge {
    pub target: u32,
    pub rel_type: RelType,
    pub confidence: f32,
    pub created_cycle: u64,
    pub valid_until: Option<u64>,
    pub temporal_weight: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GraphRegulationOutcome {
    pub expired: usize,
    pub pruned: usize,
    pub compacted: usize,
    pub renal_cleaned_sources: usize,
    pub edge_count_after: usize,
}

pub struct LegacyKnowledgeGraphNode {
    id: usize,
    node_ids: HashMap<String, u32>,
    node_names: Vec<String>,
    adjacency: Vec<Vec<KnowledgeEdge>>,
    edge_sources: HashMap<(u32, u32), HashSet<String>>,
    source_stats: HashMap<String, (u32, u32)>,
    ttl_adjustments: HashMap<String, i64>,
    current_cycle: u64,
    facts_accepted: u64,
    regulation_runs: u64,
    internal_fe: f32,
}

impl LegacyKnowledgeGraphNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            node_ids: HashMap::new(),
            node_names: Vec::new(),
            adjacency: Vec::new(),
            edge_sources: HashMap::new(),
            source_stats: HashMap::new(),
            ttl_adjustments: HashMap::new(),
            current_cycle: 0,
            facts_accepted: 0,
            regulation_runs: 0,
            internal_fe: 1.0,
        }
    }

    pub fn node_count(&self) -> usize {
        self.node_names.len()
    }

    pub fn edge_count(&self) -> usize {
        self.adjacency.iter().map(Vec::len).sum()
    }

    pub fn set_cycle(&mut self, cycle: u64) {
        self.current_cycle = cycle;
    }

    pub fn source_ttl(&self, source: &str) -> u64 {
        let s = source.to_lowercase();
        let base = if s.contains("wikidata") || s.contains("scholar") || s.contains("pubmed") {
            2000
        } else if s.contains("wikipedia") || s.contains("arxiv") {
            1000
        } else if s.contains("redis") || s.contains("cache") {
            500
        } else if s.contains("local") || s.contains("kb") {
            u64::MAX
        } else if s.contains("neuro") || s.contains("infer") || s.contains("graph_internal") {
            300
        } else if s.contains("cooc") {
            200
        } else {
            800
        };
        if base == u64::MAX {
            return base;
        }
        let adj = self.ttl_adjustments.get(&s).copied().unwrap_or(0);
        (base as i64 + adj).clamp(100, 10000) as u64
    }

    pub fn add_fact_from(&mut self, fact: &str, source: &str) -> bool {
        let lower = fact.to_lowercase();
        let parsed = if let Some(pos) = lower.find(" no es ") {
            Some((&fact[..pos], &fact[pos + 6..], RelType::Opposes))
        } else if let Some(pos) = lower.find(" is not ") {
            Some((&fact[..pos], &fact[pos + 8..], RelType::Opposes))
        } else if let Some(pos) = lower.find(" es ") {
            Some((&fact[..pos], &fact[pos + 4..], RelType::IsA))
        } else if let Some(pos) = lower.find(" is a ") {
            Some((&fact[..pos], &fact[pos + 6..], RelType::IsA))
        } else if let Some(pos) = lower.find(" is an ") {
            Some((&fact[..pos], &fact[pos + 7..], RelType::IsA))
        } else if let Some(pos) = lower.find(" is ") {
            Some((&fact[..pos], &fact[pos + 4..], RelType::IsA))
        } else if let Some(pos) = lower.find(" are ") {
            Some((&fact[..pos], &fact[pos + 5..], RelType::IsA))
        } else if let Some(pos) = lower.find(" causa ") {
            Some((&fact[..pos], &fact[pos + 7..], RelType::Causes))
        } else if let Some(pos) = lower.find(" provoca ") {
            Some((&fact[..pos], &fact[pos + 9..], RelType::Causes))
        } else if let Some(pos) = lower.find(" causes ") {
            Some((&fact[..pos], &fact[pos + 8..], RelType::Causes))
        } else if let Some(pos) = lower.find(" causes of ") {
            Some((&fact[..pos], &fact[pos + 11..], RelType::Causes))
        } else if let Some(pos) = lower.find(" tiene ") {
            Some((&fact[..pos], &fact[pos + 7..], RelType::HasProperty))
        } else if let Some(pos) = lower.find(" has ") {
            Some((&fact[..pos], &fact[pos + 5..], RelType::HasProperty))
        } else if let Some(pos) = lower.find(" can ") {
            Some((&fact[..pos], &fact[pos + 5..], RelType::HasProperty))
        } else if let Some(pos) = lower.find(" used for ") {
            Some((&fact[..pos], &fact[pos + 10..], RelType::HasProperty))
        } else {
            None
        };
        let Some((subject, object, rel_type)) = parsed else {
            return false;
        };
        let subject = Self::clean_name(subject);
        let object = Self::clean_name(object);
        if subject.is_empty() || object.is_empty() || subject == object {
            return false;
        }

        let sid = self.get_or_create_id(&subject);
        let tid = self.get_or_create_id(&object);
        let exists = self.adjacency[sid as usize]
            .iter()
            .position(|e| e.target == tid);
        self.record_source(sid, tid, source, rel_type != RelType::Opposes);
        let confidence = self.bayesian_confidence(
            sid,
            tid,
            if rel_type == RelType::Opposes {
                0.3
            } else {
                0.7
            },
        );
        let ttl = self.source_ttl(source);
        let valid_until = if ttl == u64::MAX {
            None
        } else {
            Some(self.current_cycle + ttl)
        };

        if let Some(pos) = exists {
            let edge = &mut self.adjacency[sid as usize][pos];
            edge.confidence = confidence;
            edge.valid_until = valid_until;
            edge.rel_type = rel_type;
        } else {
            self.adjacency[sid as usize].push(KnowledgeEdge {
                target: tid,
                rel_type,
                confidence,
                created_cycle: self.current_cycle,
                valid_until,
                temporal_weight: 1.0,
            });
        }
        self.facts_accepted += 1;
        self.internal_fe = (self.internal_fe + 0.05).min(5.0);
        true
    }

    pub fn temporal_query(&self, from: &str, to: &str, at_cycle: u64) -> Option<f32> {
        let sid = *self.node_ids.get(&Self::clean_name(from))? as usize;
        let tid = *self.node_ids.get(&Self::clean_name(to))?;
        self.adjacency.get(sid)?.iter().find_map(|edge| {
            if edge.target != tid {
                return None;
            }
            if edge.valid_until.is_some_and(|expires| at_cycle > expires) {
                return None;
            }
            let age = at_cycle.saturating_sub(edge.created_cycle);
            let decay = (-(age as f32) / 1000.0).exp().max(0.3);
            Some(edge.confidence * edge.temporal_weight * decay)
        })
    }

    pub fn expire_edges(&mut self, current_cycle: u64) -> usize {
        let mut removed = 0usize;
        for edges in &mut self.adjacency {
            let before = edges.len();
            edges.retain(|edge| {
                !edge
                    .valid_until
                    .is_some_and(|expires| current_cycle > expires)
            });
            removed += before - edges.len();
        }
        if removed > 0 {
            self.edge_sources.retain(|(from, to), _| {
                self.adjacency
                    .get(*from as usize)
                    .is_some_and(|edges| edges.iter().any(|edge| edge.target == *to))
            });
        }
        removed
    }

    pub fn regulate_capacity(
        &mut self,
        current_cycle: u64,
        soft_cap_edges: usize,
        hard_cap_edges: usize,
        min_confidence: f32,
    ) -> GraphRegulationOutcome {
        self.regulation_runs += 1;
        let expired = self.expire_edges(current_cycle);
        let mut total = self.edge_count();
        let mut pruned = 0usize;
        if total > soft_cap_edges {
            let threshold = if total > hard_cap_edges {
                min_confidence.max(0.55)
            } else {
                min_confidence
            };
            for edges in &mut self.adjacency {
                let before = edges.len();
                edges.retain(|edge| edge.confidence >= threshold || edge.temporal_weight >= 0.85);
                pruned += before - edges.len();
            }
            total = self.edge_count();
        }

        let mut compacted = 0usize;
        if total > hard_cap_edges {
            let target = soft_cap_edges.max(hard_cap_edges.saturating_mul(4) / 5);
            for edges in &mut self.adjacency {
                if total <= target {
                    break;
                }
                edges.sort_by(|a, b| {
                    let ascore = a.confidence * a.temporal_weight;
                    let bscore = b.confidence * b.temporal_weight;
                    bscore
                        .partial_cmp(&ascore)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let keep = edges.len().saturating_mul(3) / 4;
                if keep < edges.len() {
                    let removed = edges.len() - keep;
                    edges.truncate(keep);
                    compacted += removed;
                    total = total.saturating_sub(removed);
                }
            }
        }

        let sources_before = self.edge_sources.len();
        if expired + pruned + compacted > 0 {
            self.edge_sources.retain(|(from, to), _| {
                self.adjacency
                    .get(*from as usize)
                    .is_some_and(|edges| edges.iter().any(|edge| edge.target == *to))
            });
        }
        let renal_cleaned_sources = sources_before.saturating_sub(self.edge_sources.len());
        GraphRegulationOutcome {
            expired,
            pruned,
            compacted,
            renal_cleaned_sources,
            edge_count_after: self.edge_count(),
        }
    }

    pub fn hybrid_retrieve(&self, query: &str, top_k: usize) -> Vec<(String, f32)> {
        let query_terms = Self::terms(query);
        if query_terms.is_empty() {
            return Vec::new();
        }
        let mut scores: HashMap<u32, f32> = HashMap::new();
        for (idx, name) in self.node_names.iter().enumerate() {
            let lexical = Self::overlap_score(&query_terms, &Self::terms(name));
            if lexical > 0.0 {
                scores.insert(
                    idx as u32,
                    lexical
                        + if name.to_lowercase().contains(&query.to_lowercase()) {
                            0.35
                        } else {
                            0.0
                        },
                );
            }
        }
        let direct: Vec<(u32, f32)> = scores.iter().map(|(id, score)| (*id, *score)).collect();
        for (id, score) in direct {
            if let Some(edges) = self.adjacency.get(id as usize) {
                for edge in edges {
                    let expanded = score * 0.55 * edge.confidence * edge.temporal_weight;
                    scores
                        .entry(edge.target)
                        .and_modify(|s| *s = (*s).max(expanded))
                        .or_insert(expanded);
                }
            }
        }
        let mut ranked: Vec<(u32, f32)> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(top_k.max(1));
        ranked
            .into_iter()
            .filter_map(|(id, score)| {
                self.node_names
                    .get(id as usize)
                    .map(|name| (name.clone(), score))
            })
            .collect()
    }

    pub fn explain_paths(&self, query: &str, max_depth: usize) -> Vec<String> {
        let clean = Self::clean_name(query);
        let Some(&start) = self.node_ids.get(&clean) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut stack = vec![(start, Vec::<String>::new(), 0usize)];
        let mut visited = HashSet::new();
        while let Some((node, path, depth)) = stack.pop() {
            if depth >= max_depth || !visited.insert((node, depth)) {
                continue;
            }
            if let Some(edges) = self.adjacency.get(node as usize) {
                for edge in edges.iter().take(8) {
                    let from = self
                        .node_names
                        .get(node as usize)
                        .cloned()
                        .unwrap_or_else(|| "?".to_string());
                    let to = self
                        .node_names
                        .get(edge.target as usize)
                        .cloned()
                        .unwrap_or_else(|| "?".to_string());
                    let step = format!(
                        "{} -{}-> {} ({:.0}%)",
                        from,
                        rel_type_name(edge.rel_type),
                        to,
                        edge.confidence * 100.0
                    );
                    let mut next_path = path.clone();
                    next_path.push(step);
                    out.push(next_path.join(" | "));
                    stack.push((edge.target, next_path, depth + 1));
                    if out.len() >= 12 {
                        return out;
                    }
                }
            }
        }
        out
    }

    pub fn generate_hypotheses(&self, fact: &str) -> Vec<String> {
        let mut hypotheses = Vec::new();
        let lower = fact.to_lowercase();
        let subject = [
            " no es ",
            " is not ",
            " es ",
            " is a ",
            " is an ",
            " is ",
            " are ",
            " causa ",
            " provoca ",
            " causes ",
            " tiene ",
            " has ",
            " can ",
            " used for ",
        ]
        .iter()
        .find_map(|sep| lower.find(sep).map(|pos| &lower[..pos]))
        .unwrap_or("")
        .trim();
        if subject.is_empty() {
            return hypotheses;
        }
        for path in self.explain_paths(subject, 2).into_iter().take(3) {
            hypotheses.push(format!(
                "Hipotesis KG: si {}, entonces {}",
                fact.trim(),
                path
            ));
        }
        hypotheses
    }

    pub fn informe(&self) -> String {
        format!(
            "[KNOWLEDGE-GRAPH] nodes={} edges={} sources={} ttl_profiles={} facts_accepted={} regulation_runs={}",
            self.node_count(),
            self.edge_count(),
            self.source_stats.len(),
            self.ttl_adjustments.len(),
            self.facts_accepted,
            self.regulation_runs,
        )
    }

    pub fn autonomy_snapshot(&self) -> String {
        format!(
            "kg:nodes:{} edges:{} sources:{} facts_accepted:{} regulation_runs:{}",
            self.node_count(),
            self.edge_count(),
            self.source_stats.len(),
            self.facts_accepted,
            self.regulation_runs
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let edges: Vec<_> = self
            .adjacency
            .iter()
            .enumerate()
            .flat_map(|(from, list)| {
                list.iter().map(move |edge| {
                    serde_json::json!({
                        "from": from,
                        "target": edge.target,
                        "rel_type": rel_type_name(edge.rel_type),
                        "confidence": edge.confidence,
                        "created_cycle": edge.created_cycle,
                        "valid_until": edge.valid_until,
                        "temporal_weight": edge.temporal_weight,
                    })
                })
            })
            .collect();
        let sources: Vec<_> = self
            .edge_sources
            .iter()
            .map(|((from, to), values)| {
                serde_json::json!({
                    "from": from,
                    "to": to,
                    "sources": values.iter().cloned().collect::<Vec<_>>(),
                })
            })
            .collect();
        let source_stats: Vec<_> = self
            .source_stats
            .iter()
            .map(|(source, (hits, misses))| {
                serde_json::json!({
                    "source": source,
                    "hits": hits,
                    "misses": misses,
                })
            })
            .collect();
        let snapshot = serde_json::json!({
            "node_names": self.node_names,
            "edges": edges,
            "edge_sources": sources,
            "source_stats": source_stats,
            "ttl_adjustments": self.ttl_adjustments,
            "current_cycle": self.current_cycle,
            "facts_accepted": self.facts_accepted,
            "regulation_runs": self.regulation_runs,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.node_names = snapshot
            .get("node_names")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        self.node_ids.clear();
        for (id, name) in self.node_names.iter().enumerate() {
            self.node_ids.insert(name.clone(), id as u32);
        }
        self.adjacency = vec![Vec::new(); self.node_names.len()];
        for edge in snapshot
            .get("edges")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let from = edge.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            if from >= self.adjacency.len() {
                continue;
            }
            self.adjacency[from].push(KnowledgeEdge {
                target: edge.get("target").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                rel_type: parse_rel_type(
                    edge.get("rel_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("IsA"),
                ),
                confidence: edge
                    .get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.7) as f32,
                created_cycle: edge
                    .get("created_cycle")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                valid_until: edge.get("valid_until").and_then(|v| v.as_u64()),
                temporal_weight: edge
                    .get("temporal_weight")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0) as f32,
            });
        }
        self.edge_sources.clear();
        for item in snapshot
            .get("edge_sources")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let from = item.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let to = item.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let sources = item
                .get("sources")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();
            self.edge_sources.insert((from, to), sources);
        }
        self.source_stats.clear();
        for item in snapshot
            .get("source_stats")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            if let Some(source) = item.get("source").and_then(|v| v.as_str()) {
                self.source_stats.insert(
                    source.to_string(),
                    (
                        item.get("hits").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        item.get("misses").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    ),
                );
            }
        }
        self.ttl_adjustments =
            serde_json::from_value(snapshot.get("ttl_adjustments").cloned().unwrap_or_default())
                .unwrap_or_default();
        self.current_cycle = snapshot
            .get("current_cycle")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.facts_accepted = snapshot
            .get("facts_accepted")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.regulation_runs = snapshot
            .get("regulation_runs")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        Ok(())
    }

    fn get_or_create_id(&mut self, name: &str) -> u32 {
        if let Some(id) = self.node_ids.get(name) {
            return *id;
        }
        let id = self.node_names.len() as u32;
        self.node_ids.insert(name.to_string(), id);
        self.node_names.push(name.to_string());
        self.adjacency.push(Vec::new());
        id
    }

    fn record_source(&mut self, from: u32, to: u32, source: &str, positive: bool) {
        self.edge_sources
            .entry((from, to))
            .or_default()
            .insert(source.to_string());
        let stats = self
            .source_stats
            .entry(source.to_string())
            .or_insert((0, 0));
        if positive {
            stats.0 += 1;
        } else {
            stats.1 += 1;
        }
    }

    fn bayesian_confidence(&self, from: u32, to: u32, base: f32) -> f32 {
        let Some(sources) = self.edge_sources.get(&(from, to)) else {
            return base;
        };
        let mut belief = 0.5f32;
        for source in sources {
            let (hits, misses) = self.source_stats.get(source).copied().unwrap_or((1, 1));
            let accuracy = hits as f32 / (hits + misses).max(1) as f32;
            let likelihood = 0.5 + accuracy * 0.4;
            belief =
                (likelihood * belief) / (likelihood * belief + (1.0 - likelihood) * (1.0 - belief));
        }
        belief.clamp(base * 0.5, 0.99)
    }

    fn clean_name(name: &str) -> String {
        let lowered = name.to_lowercase();
        let trimmed = lowered
            .trim()
            .trim_matches(|c: char| c == '.' || c == ',' || c == ';' || c == ':');
        for prefix in ["la ", "el ", "los ", "las ", "un ", "una "] {
            if let Some(stripped) = trimmed.strip_prefix(prefix) {
                return stripped.trim().to_string();
            }
        }
        trimmed.to_string()
    }

    fn terms(text: &str) -> HashSet<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|term| {
                term.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|term| {
                term.len() >= 2
                    && ![
                        "de", "del", "la", "el", "un", "una", "the", "and", "is", "of",
                    ]
                    .contains(&term.as_str())
            })
            .collect()
    }

    fn overlap_score(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }
        let overlap = a.intersection(b).count() as f32;
        overlap / a.len().max(b.len()) as f32
    }
}

fn rel_type_name(rel_type: RelType) -> &'static str {
    match rel_type {
        RelType::IsA => "IsA",
        RelType::Causes => "Causes",
        RelType::HasProperty => "HasProperty",
        RelType::Opposes => "Opposes",
    }
}

fn parse_rel_type(value: &str) -> RelType {
    match value {
        "Causes" => RelType::Causes,
        "HasProperty" => RelType::HasProperty,
        "Opposes" => RelType::Opposes,
        _ => RelType::IsA,
    }
}

impl GARMNode for LegacyKnowledgeGraphNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_knowledge_graph"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe + self.edge_count() as f32 * 0.0001
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.node_count() as f32,
            self.edge_count() as f32,
            self.internal_fe,
        ]
    }

    fn act(&mut self, ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        self.current_cycle = ctx.tick;
        let removed = self.expire_edges(ctx.tick);
        let err = prediction_error.iter().map(|v| v.abs()).sum::<f32>();
        self.internal_fe = (self.internal_fe + err * 0.02 - removed as f32 * 0.001).clamp(0.2, 5.0);
        NodeAction::Output(vec![
            self.node_count() as f32,
            self.edge_count() as f32,
            removed as f32,
        ])
    }

    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.997;
        0.5
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        30.0
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
    use super::LegacyKnowledgeGraphNode;

    #[test]
    fn learns_relations_source_ttl_temporal_query_and_retrieval() {
        let mut graph = LegacyKnowledgeGraphNode::new(3);
        graph.set_cycle(10);
        assert!(graph.add_fact_from("La curiosidad es motor cognitivo", "local_kb"));
        assert!(graph.add_fact_from("curiosidad causa exploracion", "wikipedia"));
        assert!(graph.add_fact_from("bird can fly", "conceptnet"));
        assert!(graph.add_fact_from("fire causes heat", "conceptnet"));
        assert_eq!(graph.edge_count(), 4);
        assert!(graph
            .temporal_query("curiosidad", "motor cognitivo", 20)
            .is_some());
        assert!(graph
            .temporal_query("curiosidad", "exploracion", 20)
            .is_some());
        assert!(graph.temporal_query("bird", "fly", 20).is_some());
        assert!(graph.temporal_query("fire", "heat", 20).is_some());
        let hits = graph.hybrid_retrieve("curiosidad", 3);
        assert!(hits.iter().any(|(name, _)| name.contains("curiosidad")));
        assert!(!graph.explain_paths("curiosidad", 3).is_empty());
        assert!(!graph
            .generate_hypotheses("curiosidad causa exploracion")
            .is_empty());
        assert!(graph.informe().contains("KNOWLEDGE-GRAPH"));
    }

    #[test]
    fn expires_non_local_edges() {
        let mut graph = LegacyKnowledgeGraphNode::new(4);
        graph.set_cycle(1);
        assert!(graph.add_fact_from("a es b", "cooc"));
        assert_eq!(graph.expire_edges(500), 1);
        assert_eq!(graph.edge_count(), 0);
    }
}
