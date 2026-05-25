// EDEN GARM MemoryClustering — Clustering de episodios + consolidacion.
// 100% Rust puro, 0 LLM, 0 red.
//
// Hippocampus es ring buffer plano. Aqui:
//   - Cluster episodios similares por embedding (k-means online sencillo)
//   - Consolidacion: episodios frecuentemente recuperados se promueven a
//     conceptos en morphogenesis (memoria de largo plazo)
//   - Recuperacion contextual: dado un query con mood y semantica, encuentra
//     episodios mas relevantes

use crate::eden_garm::capabilities::hippocampus::{Episode, Hippocampus};
use crate::eden_garm::capabilities::morphogenesis::{Concept, ConceptSpace};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EpisodeCluster {
    pub centroid: Vec<f32>,
    pub member_ticks: Vec<u64>,
    pub label: String,
    pub access_count: u32,
}

#[derive(Clone, Debug)]
pub struct MemoryClustering {
    pub clusters: Vec<EpisodeCluster>,
    pub n_consolidations: u64,
    pub max_clusters: usize,
    pub similarity_threshold: f32,
}

impl MemoryClustering {
    pub fn new() -> Self {
        MemoryClustering {
            clusters: Vec::new(),
            n_consolidations: 0,
            max_clusters: 50,
            similarity_threshold: 0.7,
        }
    }

    fn cosine(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len().min(b.len());
        if len == 0 {
            return 0.0;
        }
        let mut dot = 0.0f32;
        let mut na = 0.0f32;
        let mut nb = 0.0f32;
        for i in 0..len {
            dot += a[i] * b[i];
            na += a[i] * a[i];
            nb += b[i] * b[i];
        }
        let denom = (na.sqrt() * nb.sqrt()).max(1e-8);
        dot / denom
    }

    /// Online clustering: assign episodes to nearest cluster or spawn new.
    pub fn cluster_all(&mut self, hippo: &Hippocampus) -> usize {
        self.clusters.clear();
        for episode in hippo.episodes.iter() {
            self.assign_or_spawn(episode);
        }
        self.clusters.len()
    }

    fn assign_or_spawn(&mut self, ep: &Episode) {
        if ep.embedding.is_empty() {
            return;
        }
        // Find best matching cluster
        let mut best_idx: Option<usize> = None;
        let mut best_sim = -1.0f32;
        for (i, cl) in self.clusters.iter().enumerate() {
            let sim = Self::cosine(&ep.embedding, &cl.centroid);
            if sim > best_sim {
                best_sim = sim;
                best_idx = Some(i);
            }
        }
        if let Some(idx) = best_idx {
            if best_sim >= self.similarity_threshold {
                let cl = &mut self.clusters[idx];
                let n = cl.member_ticks.len() as f32;
                for d in 0..cl.centroid.len().min(ep.embedding.len()) {
                    cl.centroid[d] = (cl.centroid[d] * n + ep.embedding[d]) / (n + 1.0);
                }
                cl.member_ticks.push(ep.tick);
                cl.access_count += 1;
                return;
            }
        }
        // New cluster
        if self.clusters.len() < self.max_clusters {
            self.clusters.push(EpisodeCluster {
                centroid: ep.embedding.clone(),
                member_ticks: vec![ep.tick],
                label: ep.input_snippet.chars().take(40).collect::<String>(),
                access_count: 1,
            });
        }
    }

    /// Promote large clusters into the concept space as long-term memory.
    /// Returns number of concepts added.
    pub fn consolidate(
        &mut self,
        space: &mut ConceptSpace,
        min_size: usize,
        current_tick: u64,
    ) -> usize {
        let mut added = 0usize;
        for cl in &self.clusters {
            if cl.member_ticks.len() >= min_size {
                let id = space.next_id;
                space.next_id += 1;
                let label = format!("episode_cluster:{}", cl.label);
                space.concepts.insert(
                    id,
                    Concept {
                        id,
                        centroid: cl.centroid.clone(),
                        label,
                        count: cl.member_ticks.len() as u32,
                        birth_tick: current_tick,
                        tension_accumulated: 0.0,
                        parent_id: None,
                        children: Vec::new(),
                        relations: HashMap::new(),
                        abstraction_level: 0,
                        properties: HashMap::new(),
                    },
                );
                added += 1;
                self.n_consolidations += 1;
            }
        }
        added
    }

    /// Contextual retrieval: find episodes near query embedding AND with mood near valence target.
    pub fn retrieve_contextual(
        &self,
        hippo: &Hippocampus,
        query: &[f32],
        target_valence: f32,
        k: usize,
    ) -> Vec<(u64, f32, String)> {
        let mut scored: Vec<(u64, f32, String)> = hippo
            .episodes
            .iter()
            .map(|ep| {
                let sem_sim = Self::cosine(&ep.embedding, query);
                let mood_dist = (ep.mood_valence - target_valence).abs();
                // Combined: semantic sim - mood distance
                let score = sem_sim - mood_dist * 0.3;
                (ep.tick, score, ep.input_snippet.clone())
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    pub fn report_clusters(&self, max: usize) -> String {
        if self.clusters.is_empty() {
            return "Sin clusters (ejecutar cluster primero)".to_string();
        }
        let mut sorted: Vec<&EpisodeCluster> = self.clusters.iter().collect();
        sorted.sort_by(|a, b| b.member_ticks.len().cmp(&a.member_ticks.len()));
        let mut out = format!("Clusters de episodios: {} totales\n", self.clusters.len());
        for cl in sorted.iter().take(max) {
            out.push_str(&format!(
                "  '{}' | {} miembros | accesos={}\n",
                cl.label,
                cl.member_ticks.len(),
                cl.access_count,
            ));
        }
        out
    }

    pub fn status(&self) -> String {
        format!(
            "MemoryClustering | clusters={} | consolidations={} | sim_thresh={:.2}",
            self.clusters.len(),
            self.n_consolidations,
            self.similarity_threshold,
        )
    }
}
