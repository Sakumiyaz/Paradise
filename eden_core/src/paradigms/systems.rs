// paradigms/systems.rs — Compression · Federated · Distillation · Cascade · Ensemble · AutoML
use super::autograd::Linear;
use super::ParadigmSignals;

pub struct Compression;
pub struct Federated;
pub struct Distillation;
pub struct Cascade;
pub struct Ensemble;
pub struct AutoML;

fn l2n(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8)
}
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn hash_feat(name: &str, dim: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; dim];
    for (i, b) in name.bytes().enumerate() {
        v[i % dim] += b as f32 * 0.01;
        v[(i.wrapping_mul(41).wrapping_add(17)) % dim] += b.wrapping_mul(13) as f32 * 0.007;
    }
    let n = l2n(&v);
    for x in &mut v {
        *x /= n;
    }
    v
}
fn edge_feat(sa: &str, sb: &str) -> Vec<f32> {
    let s = hash_feat(sa, 3);
    let d = hash_feat(sb, 3);
    [s[0], s[1], s[2], d[0], d[1], d[2], 0.5].to_vec()
}
fn cosim(a: &[f32], b: &[f32]) -> f32 {
    dot(a, b) / (l2n(a) * l2n(b))
}

// ═══ 1. COMPRESSION: Edge type encoding ═══
impl Compression {
    pub fn encode(sig: &mut ParadigmSignals, edges: &[(usize, usize, f32, u8)]) {
        if edges.is_empty() {
            sig.prune_threshold = 0.3;
            return;
        }
        let mut cnt = [0u32; 256];
        for &(_, _, _, t) in edges {
            cnt[t as usize] += 1;
        }
        let total = edges.len() as f32;
        let h: f32 = cnt
            .iter()
            .filter(|&&c| c > 0)
            .map(|&c| {
                let p = c as f32 / total;
                if p > 1e-8 {
                    -p * p.log2()
                } else {
                    0.0
                }
            })
            .sum::<f32>()
            / total.max(1.0);
        let ratio = h / 8.0;
        sig.prune_threshold = ((1.0 - ratio) * 0.8).clamp(0.1, 0.95);
        for &(a, b, c, _) in edges.iter().take(20) {
            sig.edge_trust.entry((a, b)).or_insert(c);
        }
        sig.activations
            .push(format!("COMP: {:.1}% entropy", ratio * 100.0));
    }
}

// ═══ 2. FEDERATED: Simple model averaging ═══
impl Federated {
    pub fn fedavg(
        sig: &mut ParadigmSignals,
        edges: &[(usize, usize, f32)],
        names: &[String],
        _epochs: usize,
        lr: f32,
    ) {
        if edges.len() < 6 {
            return;
        }
        let nc = 3; // number of clients
        let mut clients: Vec<Vec<usize>> = vec![Vec::new(); nc];
        for (ei, &(a, b, _)) in edges.iter().enumerate() {
            clients[(a.wrapping_add(b)) % nc].push(ei);
        }
        let mut cms: Vec<Linear> = (0..nc).map(|_| Linear::new(7, 1)).collect();
        // Local training
        for ci in 0..nc {
            let m = &mut cms[ci];
            for _ in 0..2 {
                for &ei in &clients[ci] {
                    let (a, b, c) = edges[ei];
                    let sa = names.get(a).map(|s| s.as_str()).unwrap_or("unk");
                    let sb = names.get(b).map(|s| s.as_str()).unwrap_or("unk");
                    let f = edge_feat(sa, sb);
                    let p = m.forward(&f);
                    m.backward(&f, &p, &[c], lr);
                }
            }
        }
        // Global aggregation
        let mut glob_w = vec![0.0f32; 7];
        let mut _glob_b = 0.0f32;
        let mut total_e = 0.0f32;
        for ci in 0..nc {
            let w = clients[ci].len() as f32;
            for j in 0..7 {
                glob_w[j] += cms[ci].w.data[j] * w;
            }
            _glob_b += cms[ci].b.data[0] * w;
            total_e += w;
        }
        if total_e > 0.0 {
            for j in 0..7 {
                glob_w[j] /= total_e;
            }
            _glob_b /= total_e;
        }
        // Diversity check
        let wmean = glob_w.iter().sum::<f32>() / 7.0;
        let wstd = (glob_w.iter().map(|&w| (w - wmean).powi(2)).sum::<f32>() / 7.0).sqrt();
        let diversity = (wstd / wmean.abs().max(1e-8)).min(1.0);
        sig.cooc_boost = (sig.cooc_boost * (1.0 + (1.0 - diversity) * 0.3)).clamp(0.02, 0.10);
        sig.model_updates.insert("fedavg_global".into(), glob_w);
    }
}

// ═══ 3. DISTILLATION: Teacher → student with soft labels ═══
impl Distillation {
    pub fn distill(
        sig: &mut ParadigmSignals,
        edges: &[(usize, usize, f32)],
        names: &[String],
        teacher_preds: &[Vec<f32>],
        epochs: usize,
        lr: f32,
    ) {
        if edges.is_empty() || teacher_preds.is_empty() {
            return;
        }
        let mut student = Linear::new(7, 1);
        for _ in 0..epochs {
            for (ei, &(a, b, ch)) in edges.iter().enumerate() {
                let sa = names.get(a).map(|s| s.as_str()).unwrap_or("unk");
                let sb = names.get(b).map(|s| s.as_str()).unwrap_or("unk");
                let f = edge_feat(sa, sb);
                let tlog: f32 = teacher_preds
                    .iter()
                    .map(|tp| tp.get(ei).copied().unwrap_or(0.5))
                    .sum::<f32>()
                    / teacher_preds.len().max(1) as f32;
                let th = tlog.clamp(0.01, 0.99);
                let p = student.forward(&f);
                // MSE loss: 0.7 * teacher_target + 0.3 * hard_label
                let combined = th * 0.7 + ch * 0.3;
                student.backward(&f, &p, &[combined], lr);
            }
        }
        sig.model_updates.insert(
            "distill_student".into(),
            [student.w.data.clone(), student.b.data.clone()].concat(),
        );
    }
}

// ═══ 4. CASCADE: Multi-stage edge filtering ═══
impl Cascade {
    pub fn filter(
        sig: &mut ParadigmSignals,
        edges: &[(usize, usize, f32)],
        sources: &[u32],
        embeddings: &[Vec<f32>],
        model_scores: &[Vec<f32>],
    ) {
        if edges.is_empty() {
            return;
        }
        let n = edges.len();
        // Stage 1: conf > 0.15
        let mut keep: Vec<bool> = edges.iter().map(|e| e.2 > 0.15).collect();
        // Stage 2: >=1 corroborated source
        for i in 0..n {
            if keep[i] && sources.get(i).copied().unwrap_or(0) < 1 {
                keep[i] = false;
            }
        }
        // Stage 3: cosine sim > 0.4
        for i in 0..n {
            if !keep[i] {
                continue;
            }
            let (a, b, _) = edges[i];
            if let (Some(ea), Some(eb)) = (embeddings.get(a), embeddings.get(b)) {
                if cosim(ea, eb) <= 0.4 {
                    keep[i] = false;
                }
            }
        }
        // Stage 4: >1 model predicts >0.5
        for i in 0..n {
            if !keep[i] {
                continue;
            }
            if model_scores
                .iter()
                .filter(|ms| ms.get(i).copied().unwrap_or(0.0) > 0.5)
                .count()
                < 2
            {
                keep[i] = false;
            }
        }
        // Write survivors
        for i in 0..n {
            if keep[i] {
                let (a, b, c) = edges[i];
                sig.edge_trust.insert((a, b), c);
            }
        }
        let reject_rate = 1.0 - keep.iter().filter(|&&k| k).count() as f32 / n.max(1) as f32;
        sig.prune_threshold = (0.2 + reject_rate * 0.6).clamp(0.15, 0.8);
    }
}

// ═══ 5. ENSEMBLE: Stacked meta-learner with inverse-error weighting ═══
impl Ensemble {
    pub fn stack(
        sig: &mut ParadigmSignals,
        edges: &[(usize, usize, f32)],
        base_preds: &[Vec<f32>],
        _names: &[String],
        _epochs: usize,
        _lr: f32,
    ) {
        if edges.is_empty() || base_preds.len() < 2 {
            return;
        }
        let nb = base_preds.len();
        let n = edges.len();
        // Inverse-error weighting
        let mut errs = vec![0.0f32; nb];
        for m in 0..nb {
            for (i, &(_, _, c)) in edges.iter().enumerate() {
                let pred = base_preds[m].get(i).copied().unwrap_or(0.5);
                errs[m] += (pred - c).abs();
            }
        }
        let total_err = errs.iter().sum::<f32>().max(1e-8);
        let weights: Vec<f32> = errs.iter().map(|e| 1.0 / (e / total_err + 0.1)).collect();
        let wsum = weights.iter().sum::<f32>();
        let norm_w: Vec<f32> = weights.iter().map(|w| w / wsum).collect();
        // Weighted ensemble prediction
        for i in 0..n {
            let (a, b, _) = edges[i];
            let ep: f32 = (0..nb)
                .map(|m| base_preds[m].get(i).copied().unwrap_or(0.5) * norm_w[m])
                .sum::<f32>()
                .clamp(0.01, 0.99);
            sig.edge_trust.insert((a, b), ep);
        }
        sig.model_updates.insert("stack_meta".into(), norm_w);
    }
}

// ═══ 6. AUTOML: Grid search over 3 parameters ═══
impl AutoML {
    pub fn optimize(
        sig: &mut ParadigmSignals,
        edges: &[(usize, usize, f32)],
        names: &[String],
        _iters: usize,
    ) {
        if edges.is_empty() {
            return;
        }
        // Grid over 3 parameters: learning_rate, cooc_boost, embed_confidence
        let lr_grid = [0.001, 0.005, 0.01, 0.05, 0.1];
        let cb_grid = [0.02, 0.04, 0.06, 0.08, 0.10];
        let ec_grid = [0.3, 0.35, 0.4, 0.45, 0.5, 0.55, 0.6];
        let train_n = (edges.len() as f32 * 0.7) as usize;
        let (mut best_lr, mut best_cb, mut best_ec, mut best_loss) = (0.01, 0.04, 0.5, f32::MAX);
        for &lr_try in &lr_grid {
            for &cb_try in &cb_grid {
                if cb_try <= lr_try {
                    continue;
                }
                for &ec_try in &ec_grid {
                    let mut model = Linear::new(7, 1);
                    for &(a, b, c) in edges.iter().take(train_n) {
                        let sa = names.get(a).map(|s| s.as_str()).unwrap_or("unk");
                        let sb = names.get(b).map(|s| s.as_str()).unwrap_or("unk");
                        let f = edge_feat(sa, sb);
                        let p = model.forward(&f);
                        model.backward(&f, &p, &[c], lr_try);
                    }
                    let mut loss = 0.0f32;
                    let mut cnt = 0;
                    for &(a, b, c) in edges.iter().skip(train_n) {
                        let sa = names.get(a).map(|s| s.as_str()).unwrap_or("unk");
                        let sb = names.get(b).map(|s| s.as_str()).unwrap_or("unk");
                        let f = edge_feat(sa, sb);
                        let p = model.forward(&f);
                        loss += (p[0] - c).abs();
                        cnt += 1;
                    }
                    if cnt > 0 {
                        loss /= cnt as f32;
                    }
                    if loss < best_loss {
                        best_loss = loss;
                        best_lr = lr_try;
                        best_cb = cb_try;
                        best_ec = ec_try;
                    }
                }
            }
        }
        sig.learning_rate_factor = best_lr;
        sig.cooc_boost = best_cb;
        sig.embed_confidence = best_ec;
        sig.activations.push(format!(
            "AUTOML: lr={:.4} cooc={:.3} ec={:.3}",
            best_lr, best_cb, best_ec
        ));
    }
}
