// EDEN GARM Hippocampus — Long-term associative episodic memory
// Dense retrieval by semantic similarity. Each episode stores the full
// cognitive state at a tick. Retrieval returns past episodes similar
// to the current query embedding.

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Episode {
    pub tick: u64,
    pub embedding: Vec<f32>,
    pub concept_id: u64,
    pub mood_valence: f32,
    pub mood_arousal: f32,
    pub actions_summary: String,
    pub input_snippet: String,
}

pub struct Hippocampus {
    pub episodes: VecDeque<Episode>,
    pub max_capacity: usize,
    pub retrieval_k: usize,
    pub min_similarity: f32,
}

impl Hippocampus {
    pub fn new(capacity: usize) -> Self {
        Hippocampus {
            episodes: VecDeque::with_capacity(capacity),
            max_capacity: capacity,
            retrieval_k: 3,
            min_similarity: 0.5,
        }
    }

    /// Store an episode. If capacity exceeded, evict oldest.
    pub fn store(
        &mut self,
        tick: u64,
        embedding: &[f32],
        concept_id: u64,
        valence: f32,
        arousal: f32,
        actions_summary: &str,
        input_snippet: &str,
    ) {
        if self.episodes.len() >= self.max_capacity {
            self.episodes.pop_front();
        }
        self.episodes.push_back(Episode {
            tick,
            embedding: embedding.to_vec(),
            concept_id,
            mood_valence: valence,
            mood_arousal: arousal,
            actions_summary: actions_summary.to_string(),
            input_snippet: input_snippet.to_string(),
        });
    }

    /// Retrieve top-k episodes most similar to query embedding.
    pub fn retrieve(&self, query: &[f32]) -> Vec<(Episode, f32)> {
        let mut scored: Vec<(Episode, f32)> = self
            .episodes
            .iter()
            .map(|ep| {
                let sim = cosine_sim(query, &ep.embedding);
                (ep.clone(), sim)
            })
            .filter(|(_, sim)| *sim >= self.min_similarity)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(self.retrieval_k);
        scored
    }

    /// Retrieve episodes that occurred when mood was similar.
    pub fn retrieve_by_mood(&self, valence: f32, arousal: f32) -> Vec<(Episode, f32)> {
        let mut scored: Vec<(Episode, f32)> = self
            .episodes
            .iter()
            .map(|ep| {
                let mood_dist = ((ep.mood_valence - valence).powi(2)
                    + (ep.mood_arousal - arousal).powi(2))
                .sqrt();
                let sim = (-mood_dist).exp();
                (ep.clone(), sim)
            })
            .filter(|(_, sim)| *sim >= self.min_similarity)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(self.retrieval_k);
        scored
    }

    /// Retrieve episodes containing a specific concept.
    pub fn retrieve_by_concept(&self, concept_id: u64) -> Vec<&Episode> {
        self.episodes
            .iter()
            .filter(|ep| ep.concept_id == concept_id)
            .collect()
    }

    pub fn n_episodes(&self) -> usize {
        self.episodes.len()
    }

    pub fn status(&self) -> String {
        format!(
            "Hippocampus | episodes={}/{} | k={} | min_sim={:.2}",
            self.episodes.len(),
            self.max_capacity,
            self.retrieval_k,
            self.min_similarity
        )
    }
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let dot = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
    let ma = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mb = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if ma > 1e-8 && mb > 1e-8 {
        dot / (ma * mb)
    } else {
        0.0
    }
}
