// paradigms/parser.rs — Parser entrenable con AutoGrad (ARM64 nativo)

use super::autograd::Linear;

/// Parser neuronal: reemplaza 150 patrones hardcoded por pesos aprendidos
pub struct NeuralParser {
    /// Linear: vocab[100] → relation_scores[6] (es, causa, tiene, parte, opone, ninguna)
    pub model: Linear,
    /// Vocabulario aprendido: las 100 palabras más frecuentes
    pub vocab: Vec<String>,
    /// Training samples acumulados
    pub samples: Vec<(Vec<f32>, usize)>,
}

impl NeuralParser {
    pub fn new() -> Self {
        NeuralParser {
            model: Linear::new(200, 6), // 100 unigrams + 100 bigrams → 6 relations
            vocab: Vec::new(),
            samples: Vec::new(),
        }
    }

    /// Construir features desde texto: unigrams + bigrams → vector de 200 floats
    fn featurize(&self, text: &str) -> Vec<f32> {
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().filter(|w| w.len() > 1).collect();
        let mut features = vec![0.0f32; 200];

        // Unigrams: first 100 slots
        for (pos, &word) in words.iter().enumerate() {
            let idx = self
                .vocab
                .iter()
                .position(|v| v == word)
                .unwrap_or(pos % 100);
            if idx < 100 {
                features[idx] += 1.0;
            }
        }

        // Bigrams: second 100 slots
        for pair in words.windows(2) {
            let bigram = format!("{}_{}", pair[0], pair[1]);
            let idx = 100 + (bigram.bytes().map(|b| b as usize).sum::<usize>() % 100);
            features[idx] += 1.0;
        }

        // Normalize
        let max = features.iter().cloned().fold(0.0f32, f32::max).max(1.0);
        for f in &mut features {
            *f /= max;
        }
        features
    }

    /// Predecir relación en un snippet
    pub fn predict(&mut self, text: &str) -> Option<(String, String, String, f32)> {
        if text.len() < 10 {
            return None;
        }
        let features = self.featurize(text);
        let scores = self.model.forward(&features);

        // Find best relation
        let mut best_idx = 5; // default: ninguna
        let mut best_score = 0.0f32;
        for i in 0..6 {
            if scores[i] > best_score {
                best_score = scores[i];
                best_idx = i;
            }
        }
        if best_idx == 5 || best_score < 0.3 {
            return None;
        }

        let rel = match best_idx {
            0 => "es",
            1 => "causa",
            2 => "tiene",
            3 => "es parte de",
            4 => "se opone a",
            _ => return None,
        };

        // Extract subject/object heuristics (uses existing verb patterns as baseline)
        let lower = text.to_lowercase();
        let (subj, obj) = match rel {
            "es" => {
                if let Some(pos) = lower.find(" es ") {
                    (text[..pos].to_string(), text[pos + 4..].to_string())
                } else {
                    return None;
                }
            }
            "causa" => {
                if let Some(pos) = lower.find("causa ") {
                    (text[..pos.min(40)].to_string(), text[pos + 6..].to_string())
                } else {
                    return None;
                }
            }
            "tiene" => {
                if let Some(pos) = lower.find(" tiene ") {
                    (text[..pos].to_string(), text[pos + 7..].to_string())
                } else {
                    return None;
                }
            }
            _ => {
                return None;
            }
        };

        let subj_trim = subj.trim().to_string();
        let obj_trim = obj.trim().to_string();
        if subj_trim.len() < 2 || obj_trim.len() < 2 {
            return None;
        }

        Some((subj_trim, rel.to_string(), obj_trim, best_score))
    }

    /// Entrenar con un ejemplo etiquetado (sujeto, relación, objeto)
    pub fn train(&mut self, text: &str, relation_idx: usize, lr: f32) -> f32 {
        let features = self.featurize(text);
        let scores = self.model.forward(&features);
        let mut target = vec![0.0f32; 6];
        target[relation_idx] = 1.0;
        self.model.backward(&features, &scores, &target, lr)
    }

    /// Expandir vocabulario con nuevas palabras
    pub fn expand_vocab(&mut self, text: &str) {
        for word in text.split_whitespace() {
            let clean = word
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_lowercase();
            if clean.len() > 2 && !self.vocab.contains(&clean) && self.vocab.len() < 200 {
                self.vocab.push(clean);
            }
        }
    }

    /// Auto-entrenamiento: edges corroborados son ground truth
    pub fn auto_train(&mut self, edges: &[(String, String, String)], lr: f32) -> f32 {
        let mut loss = 0.0;
        for (subj, rel, obj) in edges.iter().take(10) {
            let text = format!("{} {} {}", subj, rel, obj);
            self.expand_vocab(&text);
            let rel_idx = match rel.as_str() {
                "es" => 0,
                "causa" => 1,
                "tiene" => 2,
                "es parte de" => 3,
                "se opone a" => 4,
                _ => 5,
            };
            if rel_idx < 5 {
                loss += self.train(&text, rel_idx, lr);
            }
        }
        loss
    }
}
