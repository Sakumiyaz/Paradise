// EDEN GARM SceneVector — Compositional Role-Filler Semantics
// Parses sentences into structured roles (agent/action/patient/context)
// to distinguish "gato come pizza" from "pizza come gato".

use crate::eden_garm::capabilities::semantics::DistributionalSemantics;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct SceneVector {
    pub agent: Vec<f32>,
    pub action: Vec<f32>,
    pub patient: Vec<f32>,
    pub context: Vec<f32>,
    pub embedding_dim: usize,
}

pub struct SceneParser {
    pub verbs: HashSet<String>,
    pub causal_connectors: HashSet<String>,
    pub temporal_markers: HashSet<String>,
}

impl SceneParser {
    pub fn new() -> Self {
        let mut verbs = HashSet::new();
        for v in &[
            "come", "es", "tiene", "hace", "va", "da", "quiere", "puede", "debe", "es", "run",
            "eat", "has", "make", "go", "give", "want", "can", "must", "is", "are", "do", "did",
            "saw", "took", "put", "told", "asked",
        ] {
            verbs.insert(v.to_string());
        }
        let mut causal_connectors = HashSet::new();
        for c in &[
            "porque",
            "ya_que",
            "dado_que",
            "por_tanto",
            "asi_que",
            "si",
            "entonces",
            "debido_a",
            "because",
            "since",
            "given",
            "therefore",
            "so",
            "if",
            "then",
            "due_to",
        ] {
            causal_connectors.insert(c.to_string());
        }
        let mut temporal_markers = HashSet::new();
        for t in &[
            "antes", "despues", "mientras", "cuando", "ahora", "luego", "primero", "tick", "time",
            "then", "before", "after", "while", "when", "now", "later", "first",
        ] {
            temporal_markers.insert(t.to_string());
        }
        SceneParser {
            verbs,
            causal_connectors,
            temporal_markers,
        }
    }

    fn stem_list(word: &str) -> String {
        let w = word.to_lowercase().replace(" ", "_");
        w
    }

    fn is_verb(&self, word: &str) -> bool {
        let s = Self::stem_list(word);
        self.verbs.contains(&s)
            || word.ends_with("ar")
            || word.ends_with("er")
            || word.ends_with("ir")
    }

    fn is_nounish(&self, word: &str) -> bool {
        let w = word.to_lowercase();
        !self.is_verb(&w)
            && !self.causal_connectors.contains(&w)
            && !self.temporal_markers.contains(&w)
            && w.len() > 2
    }

    fn is_connector(&self, word: &str) -> bool {
        let s = Self::stem_list(word);
        self.causal_connectors.contains(&s) || self.temporal_markers.contains(&s)
    }

    fn average_embeddings(words: &[String], sem: &DistributionalSemantics, dim: usize) -> Vec<f32> {
        let mut sum = vec![0.0f32; dim];
        let mut count = 0usize;
        for w in words {
            if let Some(emb) = sem.embedding(w) {
                for i in 0..dim.min(emb.len()) {
                    sum[i] += emb[i];
                }
                count += 1;
            }
        }
        if count > 0 {
            for i in 0..dim {
                sum[i] /= count as f32;
            }
        }
        sum
    }

    pub fn parse(
        &self,
        tokens: &[String],
        sem: &DistributionalSemantics,
        dim: usize,
    ) -> SceneVector {
        let mut agent_words = Vec::new();
        let mut action_words = Vec::new();
        let mut patient_words = Vec::new();
        let mut context_words = Vec::new();

        // Find main verb position
        let mut verb_idx = tokens.len();
        for (i, t) in tokens.iter().enumerate() {
            if self.is_verb(t) {
                verb_idx = i;
                action_words.push(t.clone());
                break;
            }
        }

        if verb_idx >= tokens.len() {
            // No verb found: treat all as context
            context_words = tokens.to_vec();
        } else {
            // Before verb: agent candidates (nouns, skip connectors)
            for t in &tokens[..verb_idx] {
                if self.is_connector(t) {
                    context_words.push(t.clone());
                } else if self.is_nounish(t) {
                    agent_words.push(t.clone());
                } else {
                    context_words.push(t.clone());
                }
            }
            // After verb: patient candidates
            for t in &tokens[verb_idx + 1..] {
                if self.is_connector(t) {
                    context_words.push(t.clone());
                } else if self.is_nounish(t) {
                    patient_words.push(t.clone());
                } else {
                    context_words.push(t.clone());
                }
            }
        }

        SceneVector {
            agent: Self::average_embeddings(&agent_words, sem, dim),
            action: Self::average_embeddings(&action_words, sem, dim),
            patient: Self::average_embeddings(&patient_words, sem, dim),
            context: Self::average_embeddings(&context_words, sem, dim),
            embedding_dim: dim,
        }
    }

    /// Extract causal pairs from a sentence: "gato come porque tiene hambre" -> ("tiene hambre", "gato come")
    pub fn extract_causal(&self, tokens: &[String]) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        for (i, t) in tokens.iter().enumerate() {
            let s = Self::stem_list(t);
            if self.causal_connectors.contains(&s) {
                let before = tokens[..i].join(" ");
                let after = tokens[i + 1..].join(" ");
                if s == "porque" || s == "because" || s == "ya_que" || s == "since" {
                    // cause after connector: "come porque tiene hambre" -> cause = tiene hambre, effect = come
                    pairs.push((after.clone(), before.clone()));
                } else if s == "asi_que" || s == "therefore" || s == "so" || s == "entonces" {
                    // effect after connector: "tiene hambre asi que come" -> cause = tiene hambre, effect = come
                    pairs.push((before.clone(), after.clone()));
                } else if s == "si" || s == "if" {
                    // conditional: "si X entonces Y"
                    if let Some(then_idx) = tokens[i + 1..].iter().position(|x| {
                        Self::stem_list(x) == "entonces" || Self::stem_list(x) == "then"
                    }) {
                        let cond = tokens[i + 1..i + 1 + then_idx].join(" ");
                        let cons = tokens[i + 1 + then_idx + 1..].join(" ");
                        pairs.push((cond, cons));
                    }
                }
            }
        }
        pairs
    }
}

impl SceneVector {
    pub fn structural_similarity(&self, other: &SceneVector) -> f32 {
        let dim = self.embedding_dim.min(other.embedding_dim);
        let agent_sim = cosine_sim(
            &self.agent[..dim.min(self.agent.len())],
            &other.agent[..dim.min(other.agent.len())],
        );
        let action_sim = cosine_sim(
            &self.action[..dim.min(self.action.len())],
            &other.action[..dim.min(other.action.len())],
        );
        let patient_sim = cosine_sim(
            &self.patient[..dim.min(self.patient.len())],
            &other.patient[..dim.min(other.patient.len())],
        );
        let context_sim = cosine_sim(
            &self.context[..dim.min(self.context.len())],
            &other.context[..dim.min(other.context.len())],
        );
        // Cross-similarity detects inverse structures
        let cross_agent_patient = cosine_sim(
            &self.agent[..dim.min(self.agent.len())],
            &other.patient[..dim.min(other.patient.len())],
        );
        let cross_patient_agent = cosine_sim(
            &self.patient[..dim.min(self.patient.len())],
            &other.agent[..dim.min(other.agent.len())],
        );
        let cross = (cross_agent_patient + cross_patient_agent) / 2.0;
        // High structural similarity = aligned roles, low cross
        ((agent_sim + action_sim + patient_sim + context_sim) / 4.0) - cross * 0.3
    }

    pub fn flatten(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.embedding_dim * 4);
        out.extend_from_slice(&self.agent);
        out.extend_from_slice(&self.action);
        out.extend_from_slice(&self.patient);
        out.extend_from_slice(&self.context);
        out
    }

    pub fn status(&self) -> String {
        format!("SceneVector | dim={} | agent_mag={:.3} | action_mag={:.3} | patient_mag={:.3} | context_mag={:.3}",
            self.embedding_dim,
            self.agent.iter().map(|x| x*x).sum::<f32>().sqrt(),
            self.action.iter().map(|x| x*x).sum::<f32>().sqrt(),
            self.patient.iter().map(|x| x*x).sum::<f32>().sqrt(),
            self.context.iter().map(|x| x*x).sum::<f32>().sqrt(),
        )
    }
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let dot: f32 = a[..len]
        .iter()
        .zip(b[..len].iter())
        .map(|(x, y)| x * y)
        .sum();
    let ma: f32 = a[..len].iter().map(|x| x * x).sum::<f32>().sqrt();
    let mb: f32 = b[..len].iter().map(|x| x * x).sum::<f32>().sqrt();
    if ma > 1e-8 && mb > 1e-8 {
        dot / (ma * mb)
    } else {
        0.0
    }
}
