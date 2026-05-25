// EDEN GARM World Model — Fase 2: Object Permanence + Learned Physics
// Tracks visual objects across frames, learns their motion, and maintains
// permanence when they are occluded by other objects.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TrackedObject {
    pub id: u64,
    pub birth_tick: u64,
    pub last_seen_tick: u64,
    pub cx: f32,   // normalized centroid x (0..1)
    pub cy: f32,   // normalized centroid y (0..1)
    pub area: f32, // normalized area
    pub vx: f32,   // velocity x (EMA)
    pub vy: f32,   // velocity y (EMA)
    pub visible: bool,
    pub occluded_by: Option<u64>, // object ID that occludes this one
    pub miss_count: u32,          // consecutive frames missed
    pub label: Option<String>,    // from cross-modal grounding
    pub feature_sig: Vec<f32>,    // blob feature signature for re-identification
}

/// Tiny learned physics: EMA velocity + linear extrapolation.
/// Predicts next position from current position + smoothed velocity.
pub struct MotionPredictor {
    pub alpha: f32, // EMA factor for velocity
    pub pred_err_ema: f32,
}

impl MotionPredictor {
    pub fn new() -> Self {
        MotionPredictor {
            alpha: 0.3,
            pred_err_ema: 0.0,
        }
    }

    /// Predict next position given current position and velocity.
    pub fn predict(&self, cx: f32, cy: f32, vx: f32, vy: f32) -> (f32, f32) {
        ((cx + vx).clamp(0.0, 1.0), (cy + vy).clamp(0.0, 1.0))
    }

    /// Update velocity EMA given observed displacement.
    pub fn update_velocity(&self, vx: &mut f32, vy: &mut f32, dx: f32, dy: f32) {
        *vx = self.alpha * dx + (1.0 - self.alpha) * *vx;
        *vy = self.alpha * dy + (1.0 - self.alpha) * *vy;
    }

    /// Record prediction error to monitor physics model quality.
    pub fn observe_error(&mut self, pred_x: f32, pred_y: f32, actual_x: f32, actual_y: f32) {
        let err = ((pred_x - actual_x).powi(2) + (pred_y - actual_y).powi(2)).sqrt();
        self.pred_err_ema = 0.1 * err + 0.9 * self.pred_err_ema;
    }
}

pub struct WorldModel {
    pub objects: HashMap<u64, TrackedObject>,
    pub next_id: u64,
    pub predictor: MotionPredictor,
    pub association_radius: f32, // max normalized distance to match blob to tracked object
    pub max_miss: u32,           // frames before declaring object gone
    pub occlusion_threshold: f32, // if object vanishes while moving > this speed, it's occluded
}

impl WorldModel {
    pub fn new() -> Self {
        WorldModel {
            objects: HashMap::new(),
            next_id: 1,
            predictor: MotionPredictor::new(),
            association_radius: 0.15,
            max_miss: 5,
            occlusion_threshold: 0.02,
        }
    }

    /// Process a new frame of vision blobs. Associates blobs to existing
    /// tracked objects, updates or creates objects, handles occlusions.
    pub fn track_frame(
        &mut self,
        blobs: &[super::vision::Blob],
        img_w: u32,
        img_h: u32,
        tick: u64,
    ) -> Vec<String> {
        let mut actions = Vec::new();
        let mut matched_obj_ids: Vec<u64> = Vec::new();
        let mut matched_blob_indices: Vec<usize> = Vec::new();

        // Normalize blob centroids and areas
        let mut detections: Vec<(usize, f32, f32, f32, Vec<f32>)> = Vec::new();
        for (i, b) in blobs.iter().enumerate() {
            let cx = b.centroid_x / img_w.max(1) as f32;
            let cy = b.centroid_y / img_h.max(1) as f32;
            let area = b.area as f32 / (img_w as f32 * img_h as f32).max(1.0);
            let feat = super::perception::UnifiedPerception::extract_blob_features(b, img_w, img_h);
            detections.push((i, cx, cy, area, feat));
        }

        // Greedy association: for each detection, find closest visible object
        for (det_i, det_cx, det_cy, _det_area, _det_feat) in &detections {
            let mut best_id = None;
            let mut best_dist = self.association_radius;
            for (id, obj) in &self.objects {
                if !obj.visible {
                    continue;
                }
                let d = euclidean(*det_cx, *det_cy, obj.cx, obj.cy);
                // Also consider predicted position
                let (pred_x, pred_y) = self.predictor.predict(obj.cx, obj.cy, obj.vx, obj.vy);
                let d_pred = euclidean(*det_cx, *det_cy, pred_x, pred_y);
                let dist = d.min(d_pred);
                if dist < best_dist {
                    best_dist = dist;
                    best_id = Some(*id);
                }
            }
            if let Some(oid) = best_id {
                matched_obj_ids.push(oid);
                matched_blob_indices.push(*det_i);
            }
        }

        // Update matched objects
        for (&obj_id, &blob_idx) in matched_obj_ids.iter().zip(matched_blob_indices.iter()) {
            let (_, det_cx, det_cy, det_area, det_feat) = &detections[blob_idx];
            if let Some(obj) = self.objects.get_mut(&obj_id) {
                let dx = det_cx - obj.cx;
                let dy = det_cy - obj.cy;
                self.predictor
                    .observe_error(obj.cx + obj.vx, obj.cy + obj.vy, *det_cx, *det_cy);
                self.predictor
                    .update_velocity(&mut obj.vx, &mut obj.vy, dx, dy);
                obj.cx = *det_cx;
                obj.cy = *det_cy;
                obj.area = *det_area;
                obj.last_seen_tick = tick;
                obj.visible = true;
                obj.miss_count = 0;
                obj.occluded_by = None;
                obj.feature_sig = det_feat.clone();
            }
        }

        // Create new objects for unmatched detections
        for (det_i, det_cx, det_cy, det_area, det_feat) in &detections {
            if matched_blob_indices.contains(det_i) {
                continue;
            }
            let id = self.next_id;
            self.next_id += 1;
            self.objects.insert(
                id,
                TrackedObject {
                    id,
                    birth_tick: tick,
                    last_seen_tick: tick,
                    cx: *det_cx,
                    cy: *det_cy,
                    area: *det_area,
                    vx: 0.0,
                    vy: 0.0,
                    visible: true,
                    occluded_by: None,
                    miss_count: 0,
                    label: None,
                    feature_sig: det_feat.clone(),
                },
            );
            actions.push(format!(
                "[WORLD] New object {} born at ({:.2},{:.2}) | area={:.4}",
                id, det_cx, det_cy, det_area
            ));
        }

        // Update unmatched existing objects: they become invisible or occluded
        // First pass: collect occlusion decisions without mutable borrow
        let mut occlusion_decisions: Vec<(u64, u64, f32, f32, f32)> = Vec::new(); // (obj_id, occluder_id, pred_x, pred_y, speed)
        let all_obj_ids: Vec<u64> = self.objects.keys().copied().collect();
        for id in &all_obj_ids {
            if matched_obj_ids.contains(id) {
                continue;
            }
            if let Some(obj) = self.objects.get(id) {
                if obj.visible {
                    let (pred_x, pred_y) = self.predictor.predict(obj.cx, obj.cy, obj.vx, obj.vy);
                    let speed = (obj.vx.powi(2) + obj.vy.powi(2)).sqrt();
                    if speed > self.occlusion_threshold {
                        let mut closest = None;
                        let mut closest_dist = f32::MAX;
                        for (other_id, other) in &self.objects {
                            if *other_id == *id {
                                continue;
                            }
                            let d = euclidean(pred_x, pred_y, other.cx, other.cy);
                            if d < closest_dist {
                                closest_dist = d;
                                closest = Some(*other_id);
                            }
                        }
                        if let Some(occluder) = closest {
                            occlusion_decisions.push((*id, occluder, pred_x, pred_y, speed));
                        }
                    }
                }
            }
        }
        // Second pass: apply visibility and occlusion mutations
        for id in all_obj_ids {
            if matched_obj_ids.contains(&id) {
                continue;
            }
            if let Some(obj) = self.objects.get_mut(&id) {
                if obj.visible {
                    obj.visible = false;
                    obj.miss_count += 1;
                    if let Some(decision) = occlusion_decisions
                        .iter()
                        .find(|(oid, _, _, _, _)| *oid == id)
                    {
                        obj.occluded_by = Some(decision.1);
                        actions.push(format!("[WORLD] Object {} occluded by {} | predicted at ({:.2},{:.2}) | speed={:.3}",
                            id, decision.1, decision.2, decision.3, decision.4));
                    }
                } else {
                    obj.miss_count += 1;
                }
                if obj.miss_count > self.max_miss {
                    obj.vx = 0.0;
                    obj.vy = 0.0;
                }
            }
        }

        // Summary statistics
        let n_visible = self.objects.values().filter(|o| o.visible).count();
        let n_occluded = self
            .objects
            .values()
            .filter(|o| o.occluded_by.is_some())
            .count();
        if !actions.is_empty() || tick % 50 == 0 {
            actions.push(format!(
                "[WORLD] tracked={} | visible={} | occluded={} | pred_err={:.4}",
                self.objects.len(),
                n_visible,
                n_occluded,
                self.predictor.pred_err_ema
            ));
        }
        actions
    }

    /// After cross-modal grounding, assign labels to visible tracked objects
    /// whose features are near the bound prototype.
    pub fn assign_labels(&mut self, label: &str, proto: &[f32]) -> Vec<String> {
        let mut actions = Vec::new();
        let mut best_id = None;
        let mut best_sim = -1.0f32;
        for (id, obj) in &self.objects {
            if !obj.visible || obj.feature_sig.len() != proto.len() {
                continue;
            }
            let sim = cosine_sim(&obj.feature_sig, proto);
            if sim > best_sim {
                best_sim = sim;
                best_id = Some(*id);
            }
        }
        if let Some(id) = best_id {
            if let Some(obj) = self.objects.get_mut(&id) {
                obj.label = Some(label.to_string());
                actions.push(format!(
                    "[WORLD] Label '{}' assigned to object {} (sim={:.2})",
                    label, id, best_sim
                ));
            }
        }
        actions
    }

    pub fn status(&self) -> String {
        let n_visible = self.objects.values().filter(|o| o.visible).count();
        let n_occluded = self
            .objects
            .values()
            .filter(|o| o.occluded_by.is_some())
            .count();
        format!(
            "World | objects={} | visible={} | occluded={} | pred_err={:.4}",
            self.objects.len(),
            n_visible,
            n_occluded,
            self.predictor.pred_err_ema
        )
    }
}

fn euclidean(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
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
