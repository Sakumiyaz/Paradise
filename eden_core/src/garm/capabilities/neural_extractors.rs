// EDEN GARM NeuralExtractors — Classifiers entrenados que aprenden a detectar
// causalidad, dominio fisico, etc., reemplazando las heuristicas keyword-based.
// 100% Rust puro, 0 LLM, 0 red.
//
// Idea: cada extractor es un perceptron lineal que toma sentence_embedding
// como input y predice una etiqueta. Se entrena online con ejemplos generados
// del propio corpus (donde la heuristica acertaba) para luego generalizar.

#[derive(Clone, Debug)]
pub struct LinearClassifier {
    pub name: String,
    pub weights: Vec<f32>,
    pub bias: f32,
    pub n_inputs: usize,
    pub lr: f32,
    pub n_train: u64,
    pub n_correct: u64,
    pub last_loss: f32,
}

impl LinearClassifier {
    pub fn new(name: &str, n_inputs: usize, lr: f32) -> Self {
        // small random init
        let mut weights = vec![0.0f32; n_inputs];
        let mut seed: u64 = 1234567;
        for w in weights.iter_mut() {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let r = (seed as i64 % 1000) as f32 / 1000.0 - 0.5;
            *w = r * 0.1;
        }
        LinearClassifier {
            name: name.to_string(),
            weights,
            bias: 0.0,
            n_inputs,
            lr,
            n_train: 0,
            n_correct: 0,
            last_loss: 0.0,
        }
    }

    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Predict probability of positive class.
    pub fn predict(&self, x: &[f32]) -> f32 {
        let n = self.n_inputs.min(x.len());
        let mut z = self.bias;
        for i in 0..n {
            z += self.weights[i] * x[i];
        }
        Self::sigmoid(z)
    }

    /// Train on one labeled example. label in {0.0, 1.0}.
    pub fn train(&mut self, x: &[f32], label: f32) -> f32 {
        let n = self.n_inputs.min(x.len());
        let p = self.predict(x);
        let err = p - label; // gradient of binary cross-entropy after sigmoid
        let loss = -(label * (p + 1e-8).ln() + (1.0 - label) * (1.0 - p + 1e-8).ln());
        for i in 0..n {
            self.weights[i] -= self.lr * err * x[i];
        }
        self.bias -= self.lr * err;
        self.n_train += 1;
        if (p > 0.5 && label > 0.5) || (p <= 0.5 && label <= 0.5) {
            self.n_correct += 1;
        }
        self.last_loss = loss;
        loss
    }

    pub fn accuracy(&self) -> f32 {
        if self.n_train == 0 {
            0.0
        } else {
            self.n_correct as f32 / self.n_train as f32
        }
    }
}

#[derive(Clone, Debug)]
pub struct NeuralExtractors {
    pub causal_classifier: LinearClassifier,
    pub physical_classifier: LinearClassifier,
    pub n_predictions: u64,
}

impl NeuralExtractors {
    pub fn new(embed_dim: usize) -> Self {
        NeuralExtractors {
            causal_classifier: LinearClassifier::new("causal", embed_dim, 0.05),
            physical_classifier: LinearClassifier::new("physical", embed_dim, 0.05),
            n_predictions: 0,
        }
    }

    /// Predict if a sentence describes a causal relationship.
    pub fn predict_causal(&mut self, sentence_emb: &[f32]) -> f32 {
        self.n_predictions += 1;
        self.causal_classifier.predict(sentence_emb)
    }

    /// Predict if a sentence describes physical/material content.
    pub fn predict_physical(&mut self, sentence_emb: &[f32]) -> f32 {
        self.n_predictions += 1;
        self.physical_classifier.predict(sentence_emb)
    }

    /// Train both classifiers using heuristic labels:
    /// - causal: label=1 if sentence contains 'porque'/'because'/'si'
    /// - physical: label=1 if sentence contains physical keywords
    /// Returns (causal_loss, physical_loss).
    pub fn train_from_heuristics(&mut self, sentence: &str, sentence_emb: &[f32]) -> (f32, f32) {
        let lower = sentence.to_lowercase();
        let causal_words = [
            "porque", "because", "ya que", "si ", "if ", "asi que", "entonces", "then ",
        ];
        let causal_label: f32 = if causal_words.iter().any(|w| lower.contains(w)) {
            1.0
        } else {
            0.0
        };
        let physical_words = [
            "gravedad",
            "masa",
            "peso",
            "calor",
            "temperatura",
            "fuerza",
            "agua",
            "fuego",
            "tierra",
            "aire",
            "cae",
            "calienta",
            "gravity",
            "mass",
            "weight",
            "heat",
            "temperature",
            "force",
            "water",
            "fire",
            "earth",
            "air",
            "fall",
            "burn",
        ];
        let physical_label: f32 = if physical_words.iter().any(|w| lower.contains(w)) {
            1.0
        } else {
            0.0
        };
        let l1 = self.causal_classifier.train(sentence_emb, causal_label);
        let l2 = self.physical_classifier.train(sentence_emb, physical_label);
        (l1, l2)
    }

    pub fn status(&self) -> String {
        format!(
            "NeuralExtractors | preds={} | causal: train={} acc={:.3} loss={:.3} | physical: train={} acc={:.3} loss={:.3}",
            self.n_predictions,
            self.causal_classifier.n_train, self.causal_classifier.accuracy(),
            self.causal_classifier.last_loss,
            self.physical_classifier.n_train, self.physical_classifier.accuracy(),
            self.physical_classifier.last_loss,
        )
    }
}
