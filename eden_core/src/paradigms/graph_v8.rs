// paradigms/graph_v8.rs — EDEN v8: petgraph + local spectral embeddings
// Proof of concept: GraphV8 wraps petgraph::DiGraph con O(1) edge lookup

use petgraph::algo::dijkstra;
use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use std::collections::HashMap;

pub struct GraphV8 {
    pub graph: DiGraph<String, EdgeV8>,
    pub node_map: HashMap<String, NodeIndex>,
    pub reverse: HashMap<NodeIndex, Vec<NodeIndex>>,
}
#[derive(Clone)]
pub struct EdgeV8 {
    pub rel: RelType,
    pub conf: f32,
    pub cycle: u64,
}
#[derive(Clone, PartialEq)]
pub enum RelType {
    IsA,
    Causes,
    HasProperty,
    PartOf,
    Opposes,
    Unknown,
}

impl GraphV8 {
    pub fn new() -> Self {
        GraphV8 {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            reverse: HashMap::new(),
        }
    }
    pub fn node(&mut self, name: &str) -> NodeIndex {
        let c = name.trim().to_lowercase();
        *self
            .node_map
            .entry(c.clone())
            .or_insert_with(|| self.graph.add_node(c))
    }
    pub fn edge(&mut self, a: NodeIndex, b: NodeIndex, r: RelType, c: f32) -> EdgeIndex {
        if let Some(e) = self.graph.find_edge(a, b) {
            if let Some(w) = self.graph.edge_weight_mut(e) {
                w.conf = (w.conf + 0.1).min(0.99);
            }
            return e;
        }
        let e = self.graph.add_edge(
            a,
            b,
            EdgeV8 {
                rel: r,
                conf: c,
                cycle: 0,
            },
        );
        self.reverse.entry(b).or_default().push(a);
        e
    }
    pub fn out(&self, n: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors(n).collect()
    }
    pub fn into(&self, n: NodeIndex) -> Vec<NodeIndex> {
        self.reverse.get(&n).cloned().unwrap_or_default()
    }
    pub fn len(&self) -> usize {
        self.graph.edge_count()
    }
    pub fn path(&self, a: NodeIndex, b: NodeIndex) -> Option<f64> {
        dijkstra(&self.graph, a, Some(b), |e| e.weight().conf as f64)
            .get(&b)
            .copied()
    }

    /// Local deterministic spectral embeddings for top-K nodes.
    pub fn svd_embeddings(&self, top_k: usize) -> Vec<(NodeIndex, Vec<f32>)> {
        let n = self.graph.node_count();
        if n < 10 {
            return vec![];
        }
        // Select top_k nodes by degree
        let mut degrees: Vec<(NodeIndex, usize)> = self
            .graph
            .node_indices()
            .map(|ni| (ni, self.graph.neighbors(ni).count()))
            .collect();
        degrees.sort_by(|a, b| b.1.cmp(&a.1));
        let top: Vec<NodeIndex> = degrees
            .iter()
            .take(top_k.min(n))
            .map(|(ni, _)| *ni)
            .collect();
        let k = top.len();
        // Build adjacency sub-matrix
        let mut adj = vec![vec![0.0f32; k]; k];
        for (i, &ni) in top.iter().enumerate() {
            for (j, &nj) in top.iter().enumerate() {
                if let Some(ei) = self.graph.find_edge(ni, nj) {
                    adj[i][j] = self.graph.edge_weight(ei).map(|e| e.conf).unwrap_or(0.0);
                }
            }
        }
        let dim = 32usize.min(k);
        let components = local_spectral_components(&adj, dim);
        let mut embs = Vec::new();
        for (i, &ni) in top.iter().enumerate() {
            let mut emb = vec![0.0f32; dim];
            for j in 0..dim.min(components.len()) {
                emb[j] = components[j][i];
            }
            embs.push((ni, emb));
        }
        embs
    }
}

fn local_spectral_components(adj: &[Vec<f32>], dim: usize) -> Vec<Vec<f32>> {
    let n = adj.len();
    let mut components: Vec<Vec<f32>> = Vec::new();
    for c in 0..dim {
        let mut v: Vec<f32> = (0..n)
            .map(|i| (((i + 1) * (c + 3)) as f32 * 0.618_034).sin())
            .collect();
        normalize(&mut v);
        for prev in &components {
            orthogonalize(&mut v, prev);
        }
        normalize(&mut v);
        for _ in 0..12 {
            let mut next = vec![0.0f32; n];
            for i in 0..n {
                for j in 0..n {
                    next[i] += adj[i][j] * v[j] + adj[j][i] * v[j] * 0.3;
                }
            }
            for prev in &components {
                orthogonalize(&mut next, prev);
            }
            normalize(&mut next);
            v = next;
        }
        components.push(v);
    }
    components
}

fn orthogonalize(v: &mut [f32], against: &[f32]) {
    let dot: f32 = v.iter().zip(against).map(|(a, b)| a * b).sum();
    for (value, base) in v.iter_mut().zip(against) {
        *value -= dot * base;
    }
}

fn normalize(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
    for value in v {
        *value /= norm;
    }
}
