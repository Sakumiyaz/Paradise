// EDEN GARM Perception — Unified Multimodal Fusion
// Combines text embeddings, vision features, and voice features into a single
// cross-modal embedding space. All modalities project to the same dimensional space.
// + Fase 1B: Cross-modal grounding between vision blobs and text labels.

#[derive(Clone, Debug)]
pub struct CrossModalBinding {
    pub label: String,
    pub vision_proto: Vec<f32>, // EMA of vision blob features
    pub count: u32,
}

pub struct UnifiedPerception {
    pub text_dim: usize,
    pub vision_dim: usize,
    pub voice_dim: usize,
    pub unified_dim: usize,
    pub fusion_weights: Vec<f32>, // simple linear projection: [unified_dim × (text_dim + vision_dim + voice_dim)]
    pub bindings: Vec<CrossModalBinding>,
    pub binding_alpha: f32, // EMA learning rate for prototypes
}

impl UnifiedPerception {
    pub fn new(text_dim: usize, vision_dim: usize, voice_dim: usize, unified_dim: usize) -> Self {
        let n_inputs = text_dim + vision_dim + voice_dim;
        let mut weights = Vec::with_capacity(unified_dim * n_inputs);
        for i in 0..(unified_dim * n_inputs) {
            weights.push((i as f32 * 0.01).sin() * 0.1); // small random-ish init
        }
        UnifiedPerception {
            text_dim,
            vision_dim,
            voice_dim,
            unified_dim,
            fusion_weights: weights,
            bindings: Vec::new(),
            binding_alpha: 0.3,
        }
    }

    /// Fuse text, vision, and voice into a unified embedding.
    /// Any modality can be None (zero-padded).
    pub fn fuse(
        &self,
        text: Option<&[f32]>,
        vision: Option<&[f32]>,
        voice: Option<&[f32]>,
    ) -> Vec<f32> {
        let mut concat = vec![0.0f32; self.text_dim + self.vision_dim + self.voice_dim];
        if let Some(t) = text {
            for i in 0..t.len().min(self.text_dim) {
                concat[i] = t[i];
            }
        }
        if let Some(v) = vision {
            for i in 0..v.len().min(self.vision_dim) {
                concat[self.text_dim + i] = v[i];
            }
        }
        if let Some(vc) = voice {
            for i in 0..vc.len().min(self.voice_dim) {
                concat[self.text_dim + self.vision_dim + i] = vc[i];
            }
        }
        let mut out = vec![0.0f32; self.unified_dim];
        for j in 0..self.unified_dim {
            let mut sum = 0.0f32;
            for k in 0..concat.len() {
                sum += self.fusion_weights[j * concat.len() + k] * concat[k];
            }
            out[j] = sum.tanh(); // bounded activation
        }
        out
    }

    /// When only text is available (the normal case in REPL), just project text through.
    pub fn fuse_text_only(&self, text: &[f32]) -> Vec<f32> {
        self.fuse(Some(text), None, None)
    }

    // ─── Fase 1B: Cross-modal grounding ───

    /// Extract a compact feature vector from a vision blob.
    /// Features: aspect_ratio, fill_ratio, normalized_centroid_x, normalized_centroid_y, normalized_area
    pub fn extract_blob_features(blob: &super::vision::Blob, img_w: u32, img_h: u32) -> Vec<f32> {
        let w = blob.w.max(1) as f32;
        let h = blob.h.max(1) as f32;
        let aspect = w / h;
        let area = blob.area as f32;
        let bbox_area = w * h;
        let fill = area / bbox_area.max(1.0);
        let cx = blob.centroid_x / img_w.max(1) as f32;
        let cy = blob.centroid_y / img_h.max(1) as f32;
        let area_norm = area / (img_w as f32 * img_h as f32).max(1.0);
        vec![aspect, fill, cx, cy, area_norm]
    }

    /// Bind a text label to a vision feature via exponential moving average.
    pub fn bind_vision_text(&mut self, text_label: &str, vision_feat: &[f32]) {
        if let Some(b) = self.bindings.iter_mut().find(|b| b.label == text_label) {
            for (i, v) in vision_feat.iter().enumerate() {
                if i < b.vision_proto.len() {
                    b.vision_proto[i] =
                        self.binding_alpha * v + (1.0 - self.binding_alpha) * b.vision_proto[i];
                }
            }
            b.count += 1;
        } else {
            self.bindings.push(CrossModalBinding {
                label: text_label.to_string(),
                vision_proto: vision_feat.to_vec(),
                count: 1,
            });
        }
    }

    /// Given vision features, find the closest bound text label by cosine similarity.
    pub fn recognize_vision(&self, vision_feat: &[f32]) -> Option<(String, f32)> {
        let mut best = None;
        let mut best_sim = -1.0f32;
        for b in &self.bindings {
            if b.vision_proto.len() != vision_feat.len() {
                continue;
            }
            let sim = cosine_similarity(&b.vision_proto, vision_feat);
            if sim > best_sim {
                best_sim = sim;
                best = Some((b.label.clone(), sim));
            }
        }
        best
    }

    /// Given a text label, return the associated vision prototype if known.
    pub fn activate_from_text(&self, text_label: &str) -> Option<&[f32]> {
        self.bindings
            .iter()
            .find(|b| b.label == text_label)
            .map(|b| b.vision_proto.as_slice())
    }

    pub fn status(&self) -> String {
        format!(
            "Perception | unified={} | text={} | vision={} | voice={} | bindings={}",
            self.unified_dim,
            self.text_dim,
            self.vision_dim,
            self.voice_dim,
            self.bindings.len()
        )
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
    let ma = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mb = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if ma > 1e-8 && mb > 1e-8 {
        dot / (ma * mb)
    } else {
        0.0
    }
}
