// EDEN GARM LanguageGen — Generacion textual desde el grafo causal + bigrams.
// 100% Rust puro, 0 LLM, 0 red.
//
// Cierra el ciclo de comunicacion: EDEN puede leer y razonar; ahora HABLA
// desde su conocimiento. Estrategias:
//   - explain_chain(concept): genera narracion de cadena causal hacia atras
//   - describe_concept(id): produce descripcion combinando relaciones del concepto
//   - generate_completion(prompt, max_words): completa una oracion usando bigrams
//   - answer_why(query): explicacion en lenguaje natural

use crate::eden_garm::capabilities::inference::InferenceEngine;
use crate::eden_garm::capabilities::morphogenesis::ConceptSpace;
use crate::eden_garm::capabilities::semantics::DistributionalSemantics;

#[derive(Clone, Debug)]
pub struct LanguageGenerator {
    /// Connectives in es/en for narrative generation
    pub causal_connectives: Vec<String>,
    pub temporal_connectives: Vec<String>,
    pub generation_count: u64,
}

impl LanguageGenerator {
    pub fn new() -> Self {
        LanguageGenerator {
            causal_connectives: vec![
                "porque".into(),
                "because".into(),
                "ya que".into(),
                "since".into(),
            ],
            temporal_connectives: vec!["luego".into(), "then".into(), "y".into(), "and".into()],
            generation_count: 0,
        }
    }

    /// Generate a narrative chain explaining why a concept exists,
    /// walking back through `causes` edges.
    pub fn explain_chain(&mut self, space: &ConceptSpace, query: &str, max_depth: usize) -> String {
        self.generation_count += 1;
        let matches = InferenceEngine::find_concepts_by_label(space, query);
        if matches.is_empty() {
            return format!("No se que es '{}'.", query);
        }
        let target = matches[0];
        let target_label = space
            .concepts
            .get(&target)
            .map(|c| c.label.clone())
            .unwrap_or_default();
        let mut narrative = format!("'{}'", target_label);
        let mut current = target;
        let mut visited = std::collections::HashSet::new();
        visited.insert(current);
        for _ in 0..max_depth {
            let causes = InferenceEngine::find_predecessors(space, current, "causes");
            if causes.is_empty() {
                break;
            }
            // Pick first novel cause
            let next = match causes.iter().find(|c| !visited.contains(c)) {
                Some(c) => *c,
                None => break,
            };
            let cause_label = space
                .concepts
                .get(&next)
                .map(|c| c.label.clone())
                .unwrap_or_default();
            narrative = format!("{} porque {}", narrative, cause_label);
            visited.insert(next);
            current = next;
        }
        format!("{}.", narrative)
    }

    /// Describe a concept by combining its outgoing and incoming relations.
    pub fn describe_concept(&mut self, space: &ConceptSpace, id: u64) -> String {
        self.generation_count += 1;
        let c = match space.concepts.get(&id) {
            Some(c) => c,
            None => return format!("(concepto {} no existe)", id),
        };
        let causes_in = InferenceEngine::find_predecessors(space, id, "causes");
        let causes_out: Vec<u64> = c.relations.get("causes").cloned().unwrap_or_default();
        let mut sentences: Vec<String> = vec![format!("'{}'", c.label)];
        if !causes_in.is_empty() {
            let causes_labels: Vec<String> = causes_in
                .iter()
                .take(3)
                .filter_map(|cid| space.concepts.get(cid).map(|cc| cc.label.clone()))
                .collect();
            if !causes_labels.is_empty() {
                sentences.push(format!("ocurre porque {}", causes_labels.join(" y ")));
            }
        }
        if !causes_out.is_empty() {
            let effects_labels: Vec<String> = causes_out
                .iter()
                .take(3)
                .filter_map(|cid| space.concepts.get(cid).map(|cc| cc.label.clone()))
                .collect();
            if !effects_labels.is_empty() {
                sentences.push(format!("y a su vez causa {}", effects_labels.join(" y ")));
            }
        }
        if sentences.len() == 1 {
            sentences.push("es un concepto aislado en mi conocimiento".to_string());
        }
        format!("{}.", sentences.join(" "))
    }

    /// Continue a sentence using bigram statistics from semantics.
    /// Uses greedy max-likelihood next-word selection.
    pub fn generate_completion(
        &mut self,
        sem: &DistributionalSemantics,
        prompt: &str,
        max_words: usize,
    ) -> String {
        self.generation_count += 1;
        let mut tokens: Vec<String> = prompt
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(String::from)
            .collect();
        if tokens.is_empty() {
            return prompt.to_string();
        }
        // Iteratively extend
        let mut last = tokens.last().cloned().unwrap_or_default();
        for _ in 0..max_words {
            let last_idx = match sem.vocab.get(&last) {
                Some(&i) => i,
                None => break,
            };
            // Find argmax of bigrams[(last_idx, ?)]
            let mut best_next: Option<usize> = None;
            let mut best_count = 0.0f32;
            for ((a, b), &count) in &sem.bigrams {
                if *a == last_idx && count > best_count {
                    best_count = count;
                    best_next = Some(*b);
                }
            }
            let next_word = match best_next {
                Some(idx) => match sem.index_to_word.get(idx) {
                    Some(w) => w.clone(),
                    None => break,
                },
                None => break,
            };
            // Avoid loops
            if tokens.iter().rev().take(3).any(|t| t == &next_word) {
                break;
            }
            tokens.push(next_word.clone());
            last = next_word;
        }
        tokens.join(" ")
    }

    /// Generate a natural-language answer to "why X" using the causal graph.
    pub fn answer_why(&mut self, space: &ConceptSpace, query: &str) -> String {
        let chain = self.explain_chain(space, query, 4);
        let matches = InferenceEngine::find_concepts_by_label(space, query);
        if matches.is_empty() {
            return chain;
        }
        let target = matches[0];
        let downstream = InferenceEngine::transitive_effects(space, target, 3);
        let mut answer = chain;
        if !downstream.is_empty() {
            let effects: Vec<String> = downstream
                .iter()
                .take(3)
                .map(|(_, _, l)| l.clone())
                .collect();
            answer.push_str(&format!(
                " Como consecuencia, esto lleva a: {}.",
                effects.join(", ")
            ));
        }
        answer
    }

    /// Beam search: keep top-k candidates at each step, using transformer logits.
    pub fn beam_search(
        &mut self,
        transformer: &mut crate::eden_garm::capabilities::transformer::EdenTransformer,
        sem: &DistributionalSemantics,
        prompt: &str,
        beam_width: usize,
        max_len: usize,
        temperature: f32,
    ) -> String {
        self.generation_count += 1;
        let init_tokens: Vec<String> = prompt
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(String::from)
            .collect();
        if init_tokens.is_empty() {
            return prompt.to_string();
        }
        let mut beams: Vec<(Vec<usize>, f32)> = vec![(
            init_tokens
                .iter()
                .filter_map(|t| sem.vocab.get(t).copied())
                .collect(),
            0.0f32,
        )];
        if beams[0].0.is_empty() {
            return prompt.to_string();
        }
        for _ in 0..max_len {
            let mut candidates: Vec<(Vec<usize>, f32)> = Vec::new();
            for (seq, score) in beams.iter() {
                if seq.len() >= transformer.max_seq_len {
                    continue;
                }
                let logits = transformer.predict_next(seq);
                let mut probs: Vec<(usize, f32)> = logits
                    .iter()
                    .enumerate()
                    .map(|(i, &l)| {
                        let p = (l / temperature).exp();
                        (i, p)
                    })
                    .collect();
                probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let sum: f32 = probs.iter().map(|(_, p)| p).sum();
                let denom = sum.max(1e-8);
                for (_idx, (tok, raw_p)) in probs.iter().take(beam_width).enumerate() {
                    let p = raw_p / denom;
                    let mut new_seq = seq.clone();
                    new_seq.push(*tok);
                    let new_score = score + p.ln();
                    candidates.push((new_seq, new_score));
                }
            }
            candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            beams = candidates.into_iter().take(beam_width).collect();
        }
        // Return best beam as text
        if let Some((best_seq, _)) = beams.first() {
            let words: Vec<String> = best_seq
                .iter()
                .filter_map(|&i| sem.index_to_word.get(i).cloned())
                .collect();
            words.join(" ")
        } else {
            prompt.to_string()
        }
    }

    /// Temperature sampling: sample next word from softmax distribution.
    pub fn temperature_sample(
        &mut self,
        transformer: &mut crate::eden_garm::capabilities::transformer::EdenTransformer,
        sem: &DistributionalSemantics,
        prompt: &str,
        max_len: usize,
        temperature: f32,
    ) -> String {
        self.generation_count += 1;
        let mut tokens: Vec<usize> = prompt
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .filter_map(|t| sem.vocab.get(t).copied())
            .collect();
        if tokens.is_empty() {
            return prompt.to_string();
        }
        for _ in 0..max_len {
            if tokens.len() >= transformer.max_seq_len {
                break;
            }
            let logits = transformer.predict_next(&tokens);
            let mut probs: Vec<f32> = logits.iter().map(|&l| (l / temperature).exp()).collect();
            let sum: f32 = probs.iter().sum();
            let denom = sum.max(1e-8);
            for p in probs.iter_mut() {
                *p /= denom;
            }
            // Sample
            let mut cumsum = 0.0f32;
            let mut seed: u64 = self.generation_count.wrapping_add(tokens.len() as u64);
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let r = (seed % 1000) as f32 / 1000.0;
            let mut chosen = 0usize;
            for (i, p) in probs.iter().enumerate() {
                cumsum += p;
                if cumsum >= r {
                    chosen = i;
                    break;
                }
            }
            tokens.push(chosen);
            // Stop at unknown or repeated patterns
            if chosen == 0
                || tokens.len() > 3
                    && tokens[tokens.len() - 2..] == tokens[tokens.len() - 4..tokens.len() - 2]
            {
                break;
            }
        }
        let words: Vec<String> = tokens
            .iter()
            .filter_map(|&i| sem.index_to_word.get(i).cloned())
            .collect();
        words.join(" ")
    }

    pub fn status(&self) -> String {
        format!("LanguageGen | generations={}", self.generation_count)
    }
}
