// EDEN GARM Multi-Agent — Message protocol for cognitive ensembles across processes
// Future: multiple EDEN instances can share predictions via serialized messages.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EdenMessage {
    pub sender_id: String,
    pub tick: u64,
    pub intent: String,
    pub confidence: f32,
    pub embedding: Vec<f32>,
    pub predicted_success: f32,
}

pub struct MultiAgentMesh {
    pub inbox: Vec<EdenMessage>,
    pub peers: HashMap<String, Vec<f32>>, // peer_id -> last embedding
    pub max_inbox: usize,
}

impl MultiAgentMesh {
    pub fn new() -> Self {
        MultiAgentMesh {
            inbox: Vec::with_capacity(100),
            peers: HashMap::new(),
            max_inbox: 100,
        }
    }

    pub fn send(&mut self, msg: EdenMessage) {
        self.inbox.push(msg);
        if self.inbox.len() > self.max_inbox {
            self.inbox.remove(0);
        }
    }

    pub fn receive(&mut self) -> Vec<EdenMessage> {
        let msgs = self.inbox.clone();
        self.inbox.clear();
        for msg in &msgs {
            self.peers
                .insert(msg.sender_id.clone(), msg.embedding.clone());
        }
        msgs
    }

    pub fn peer_consensus(&self, embedding: &[f32]) -> f32 {
        if self.peers.is_empty() {
            return 0.5;
        }
        let mut sim_sum = 0.0f32;
        let mut count = 0usize;
        for peer_emb in self.peers.values() {
            if peer_emb.len() == embedding.len() {
                let dot: f32 = peer_emb
                    .iter()
                    .zip(embedding.iter())
                    .map(|(a, b)| a * b)
                    .sum();
                let norm_a: f32 = peer_emb.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                let cos = if norm_a * norm_b > 0.0 {
                    dot / (norm_a * norm_b)
                } else {
                    0.0
                };
                sim_sum += cos;
                count += 1;
            }
        }
        if count == 0 {
            0.5
        } else {
            (sim_sum / count as f32).clamp(0.0, 1.0)
        }
    }

    pub fn status(&self) -> String {
        format!(
            "Mesh | peers: {} | inbox: {}",
            self.peers.len(),
            self.inbox.len()
        )
    }
}
