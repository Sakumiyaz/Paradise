// EDEN GARM Analogy — Razonamiento analógico para one-shot generalization.
// 100% Rust puro, 0 LLM, 0 red.
//
// Si EDEN conoce que X causes Y, y encuentra X' similar a X, infiere que
// probablemente X' también causa algo similar a Y. Es la definicion operativa
// de generalización: extender conocimiento de un caso a casos análogos.
//
// Métricas de similitud:
//   - Cosine similarity entre centroides (representación distribucional)
//   - Jaccard sobre keywords del label (representación simbólica)
//   - Score combinado = 0.6 * cosine + 0.4 * jaccard
//
// Las relaciones inferidas analógicamente se marcan con tipo
// 'analogously_causes' para distinguirlas de las observadas directamente
// del corpus.

use crate::eden_garm::capabilities::morphogenesis::ConceptSpace;

#[derive(Clone, Debug)]
pub struct AnalogyMatch {
    pub source_id: u64,
    pub target_id: u64,
    pub source_label: String,
    pub target_label: String,
    pub cosine: f32,
    pub jaccard: f32,
    pub combined_score: f32,
}

#[derive(Clone, Debug)]
pub struct AnalogicalInference {
    pub from_concept: u64,
    pub from_label: String,
    pub source_concept: u64,
    pub source_label: String,
    pub inferred_relation: String,
    pub target_concept: u64,
    pub target_label: String,
    pub confidence: f32,
}

#[derive(Clone, Debug)]
pub struct AnalogyEngine {
    pub min_combined_score: f32,
    pub n_inferences: u64,
    pub n_attempts: u64,
    pub history: Vec<AnalogicalInference>,
    pub max_history: usize,
}

impl AnalogyEngine {
    pub fn new() -> Self {
        AnalogyEngine {
            min_combined_score: 0.3,
            n_inferences: 0,
            n_attempts: 0,
            history: Vec::new(),
            max_history: 200,
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

    fn jaccard(a: &str, b: &str) -> f32 {
        let words_a: std::collections::HashSet<String> = a
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .map(String::from)
            .collect();
        let words_b: std::collections::HashSet<String> = b
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .map(String::from)
            .collect();
        if words_a.is_empty() && words_b.is_empty() {
            return 0.0;
        }
        let inter = words_a.intersection(&words_b).count() as f32;
        let union = words_a.union(&words_b).count() as f32;
        if union <= 0.0 {
            0.0
        } else {
            inter / union
        }
    }

    /// Find concepts most analogous to target_id, ranked by combined score.
    pub fn find_analogous_concepts(
        &self,
        space: &ConceptSpace,
        target_id: u64,
        top_k: usize,
    ) -> Vec<AnalogyMatch> {
        let target = match space.concepts.get(&target_id) {
            Some(c) => c,
            None => return Vec::new(),
        };
        let mut matches: Vec<AnalogyMatch> = Vec::new();
        for c in space.concepts.values() {
            if c.id == target_id {
                continue;
            }
            let has_relations: usize = c.relations.values().map(|v| v.len()).sum();
            if has_relations == 0 {
                continue;
            }
            let cos = Self::cosine(&target.centroid, &c.centroid);
            let jac = Self::jaccard(&target.label, &c.label);
            let combined = 0.6 * cos + 0.4 * jac;
            if combined >= self.min_combined_score {
                matches.push(AnalogyMatch {
                    source_id: c.id,
                    target_id,
                    source_label: c.label.clone(),
                    target_label: target.label.clone(),
                    cosine: cos,
                    jaccard: jac,
                    combined_score: combined,
                });
            }
        }
        matches.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches.truncate(top_k);
        matches
    }

    /// Transfer all "causes" relations from source to target as "analogously_causes".
    pub fn transfer_relations(
        &mut self,
        space: &mut ConceptSpace,
        target_id: u64,
        source_id: u64,
        confidence: f32,
    ) -> Vec<AnalogicalInference> {
        let mut inferences = Vec::new();
        let source_causes: Vec<u64> = match space.concepts.get(&source_id) {
            Some(c) => c.relations.get("causes").cloned().unwrap_or_default(),
            None => return inferences,
        };
        let source_label = space
            .concepts
            .get(&source_id)
            .map(|c| c.label.clone())
            .unwrap_or_default();
        let target_label = space
            .concepts
            .get(&target_id)
            .map(|c| c.label.clone())
            .unwrap_or_default();

        for cause_target_id in source_causes {
            // Skip self-loops: never create X -> analogously_causes -> X
            if cause_target_id == target_id {
                continue;
            }
            let already = space
                .concepts
                .get(&target_id)
                .map(|c| {
                    let direct = c
                        .relations
                        .get("causes")
                        .map(|v| v.contains(&cause_target_id))
                        .unwrap_or(false);
                    let analog = c
                        .relations
                        .get("analogously_causes")
                        .map(|v| v.contains(&cause_target_id))
                        .unwrap_or(false);
                    direct || analog
                })
                .unwrap_or(false);
            if already {
                continue;
            }

            space.add_relation(target_id, "analogously_causes", cause_target_id);
            let target_label_str = space
                .concepts
                .get(&cause_target_id)
                .map(|c| c.label.clone())
                .unwrap_or_default();
            let inf = AnalogicalInference {
                from_concept: target_id,
                from_label: target_label.clone(),
                source_concept: source_id,
                source_label: source_label.clone(),
                inferred_relation: "analogously_causes".to_string(),
                target_concept: cause_target_id,
                target_label: target_label_str,
                confidence,
            };
            self.history.push(inf.clone());
            inferences.push(inf);
            self.n_inferences += 1;
        }
        if self.history.len() > self.max_history {
            let drop = self.history.len() - self.max_history;
            self.history.drain(..drop);
        }
        inferences
    }

    /// Apply analogical reasoning to a single concept.
    pub fn infer_for_concept(
        &mut self,
        space: &mut ConceptSpace,
        target_id: u64,
    ) -> Vec<AnalogicalInference> {
        self.n_attempts += 1;
        let existing_causes: usize = space
            .concepts
            .get(&target_id)
            .map(|c| c.relations.get("causes").map(|v| v.len()).unwrap_or(0))
            .unwrap_or(0);
        if existing_causes > 0 {
            return Vec::new();
        }

        let candidates = self.find_analogous_concepts(space, target_id, 3);
        if candidates.is_empty() {
            return Vec::new();
        }

        let best = &candidates[0];
        let confidence = best.combined_score;
        self.transfer_relations(space, target_id, best.source_id, confidence)
    }

    /// Run analogical inference over concepts that lack causes.
    pub fn infer_all_gaps(&mut self, space: &mut ConceptSpace, max_concepts: usize) -> usize {
        let candidates: Vec<u64> = space
            .concepts
            .values()
            .filter(|c| {
                c.relations
                    .get("causes")
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
            })
            .filter(|c| {
                c.relations
                    .get("analogously_causes")
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
            })
            .map(|c| c.id)
            .take(max_concepts)
            .collect();
        let mut total = 0usize;
        for cid in candidates {
            let inferences = self.infer_for_concept(space, cid);
            total += inferences.len();
        }
        total
    }

    pub fn n_analogical_relations(space: &ConceptSpace) -> usize {
        space
            .concepts
            .values()
            .filter_map(|c| c.relations.get("analogously_causes"))
            .map(|v| v.len())
            .sum()
    }

    pub fn status(&self) -> String {
        format!(
            "Analogy | attempts={} | inferences={} | history_kept={}",
            self.n_attempts,
            self.n_inferences,
            self.history.len(),
        )
    }

    pub fn report(&self, max: usize) -> String {
        if self.history.is_empty() {
            return "Sin inferencias analogicas".to_string();
        }
        let mut out = format!("Inferencias analogicas: {} totales\n", self.history.len());
        for inf in self.history.iter().rev().take(max) {
            out.push_str(&format!(
                "  '{}' -[{}]-> '{}'  (analogo a '{}' | conf={:.2})\n",
                inf.from_label,
                inf.inferred_relation,
                inf.target_label,
                inf.source_label,
                inf.confidence,
            ));
        }
        out
    }
}
