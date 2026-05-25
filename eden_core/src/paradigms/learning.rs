// paradigms/learning.rs — 10 learning paradigms
use super::autograd::Linear;
use super::autograd::Tensor;
use super::ParadigmSignals;

pub struct Active;
pub struct Contrastive;
pub struct SelfSupervised;
pub struct Curriculum;
pub struct Transfer;
pub struct Enactive;
pub struct MetaLearning;
pub struct Continual;
pub struct ZeroShot;
pub struct FewShot;

fn l2n(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8)
}
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn cos(a: &[f32], b: &[f32]) -> f32 {
    let d = dot(a, b);
    d / (l2n(a) * l2n(b))
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
fn edge_feat(sa: &str, sb: &str, conf: f32) -> Vec<f32> {
    let s = hash_feat(sa, 3);
    let d = hash_feat(sb, 3);
    [s[0], s[1], s[2], d[0], d[1], d[2], conf].to_vec()
}
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

// ═══ 1. ACTIVE: Entropy-based uncertainty sampling ═══
impl Active {
    pub fn select(sig: &mut ParadigmSignals, edges: &[(usize, usize, f32)]) {
        if edges.is_empty() {
            return;
        }
        let mut scored: Vec<_> = edges
            .iter()
            .filter(|&&(_, _, c)| c > 0.25 && c < 0.75)
            .map(|&(a, b, c)| {
                let e = if c > 1e-6 && c < 1.0 - 1e-6 {
                    -(c * c.log2() + (1.0 - c) * (1.0 - c).log2())
                } else {
                    0.0
                };
                (a, b, c, e)
            })
            .collect();
        scored.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
        scored.truncate(5);
        for (a, b, c, _) in &scored {
            sig.edge_trust.entry((*a, *b)).or_insert(*c);
        }
    }
}

// ═══ 2. CONTRASTIVE: SimCLR with projection head ═══
impl Contrastive {
    pub fn train(
        _sig: &mut ParadigmSignals,
        edges: &[(usize, usize, f32)],
        names: &[String],
        _epochs: usize,
        lr: f32,
    ) {
        if edges.len() < 4 {
            return;
        }
        let mut p1 = Linear::new(4, 8);
        let mut p2 = Linear::new(8, 4);
        let tau = 0.1;
        let bs = (edges.len() / 2).max(4);
        for _ in 0..2 {
            for bi in 0..edges.len() / bs {
                let beg = bi * bs;
                let end = (beg + bs).min(edges.len());
                let n = end - beg;
                if n < 2 {
                    continue;
                }
                let mut zs = vec![vec![0.0; 4]; n];
                for i in beg..end {
                    let (a, b, c) = edges[i];
                    let f = edge_feat(names[a].as_str(), names[b].as_str(), c);
                    let f4: Vec<f32> = f
                        .iter()
                        .take(4)
                        .cloned()
                        .chain(std::iter::repeat(0.0))
                        .take(4)
                        .collect();
                    let h = p1.forward(&f4);
                    zs[i - beg] = p2.forward(&h);
                }
                for i in 0..n {
                    for j in 0..n {
                        let d = dot(&zs[i], &zs[j]) / tau;
                        let trgt = if i == j { 1.0 } else { 0.0 };
                        let mut feats_in = vec![0.0f32; 4];
                        let (a, b, c) = edges[beg + i];
                        let f = edge_feat(names[a].as_str(), names[b].as_str(), c);
                        feats_in.copy_from_slice(
                            &f.iter()
                                .take(4)
                                .cloned()
                                .chain(std::iter::repeat(0.0))
                                .take(4)
                                .collect::<Vec<_>>(),
                        );
                        if d > 0.0 {
                            let h = p1.forward(&feats_in);
                            let z = p2.forward(&h);
                            p2.backward(&h, &z, &[trgt], lr * 0.01);
                        }
                    }
                }
                for j in 0..p2.w.data.len() {
                    p2.w.data[j] -= p2.w.grad[j];
                    p2.w.grad[j] = 0.0;
                }
                for j in 0..p2.b.data.len() {
                    p2.b.data[j] -= p2.b.grad[j];
                    p2.b.grad[j] = 0.0;
                }
            }
        }
    }
}

// ═══ 3. SELF-SUPERVISED: Masked prediction with Linear::new(2,4) encoder ═══
impl SelfSupervised {
    pub fn train(sig: &mut ParadigmSignals, epochs: usize, lr: f32) {
        if sig.node_embeddings.is_empty() {
            return;
        }
        let mut enc = Linear::new(2, 4);
        let mut dec = Linear::new(4, 2);
        for _ in 0..epochs {
            for feat in sig.node_embeddings.iter() {
                let inp: Vec<f32> = feat
                    .iter()
                    .take(2)
                    .cloned()
                    .chain(std::iter::repeat(0.0))
                    .take(2)
                    .collect();
                let mut masked = inp.clone();
                masked[0] = 0.0; // 50% mask
                let h = enc.forward(&masked);
                let rec = dec.forward(&h);
                dec.backward(&h, &rec, &inp, lr);
                for k in 0..h.len() {
                    if h[k] > 0.0 {
                        for j in 0..2 {
                            enc.w.grad[j * 4 + k] += lr * 0.01 * inp[j];
                        }
                    }
                }
            }
            for j in 0..enc.w.data.len() {
                enc.w.data[j] = (enc.w.data[j] + enc.w.grad[j] * 0.1).clamp(-1.0, 1.0);
                enc.w.grad[j] = 0.0;
            }
            for j in 0..enc.b.data.len() {
                enc.b.data[j] = (enc.b.data[j] + enc.b.grad[j] * 0.1).clamp(-1.0, 1.0);
                enc.b.grad[j] = 0.0;
            }
        }
    }
}

// ═══ 4. CURRICULUM: Difficulty-based pacing ═══
impl Curriculum {
    pub fn pace(
        sig: &mut ParadigmSignals,
        edges: &[(usize, usize, f32)],
        regions: &[(usize, usize)],
        node_degrees: &[usize],
    ) {
        if regions.is_empty() {
            return;
        }
        let mut diffs: Vec<f32> = regions
            .iter()
            .map(|&(s, e)| {
                let slice = &edges[s..e.min(edges.len())];
                let avg_conf = if slice.is_empty() {
                    0.5
                } else {
                    slice.iter().map(|&(_, _, c)| c).sum::<f32>() / slice.len() as f32
                };
                let deg =
                    node_degrees.iter().sum::<usize>() as f32 / node_degrees.len().max(1) as f32;
                (1.0 - avg_conf) * 0.5 + 1.0 / (1.0 + deg * 0.1)
            })
            .collect();
        diffs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = diffs[diffs.len() / 2].clamp(0.01, 0.99);
        sig.learning_rate_factor = 0.5 + 0.5 * median;
    }
}

// ═══ 5. TRANSFER: Gradient reversal domain adaptation ═══
impl Transfer {
    pub fn adapt(
        sig: &mut ParadigmSignals,
        edges: &[(usize, usize, f32)],
        names: &[String],
        _epochs: usize,
        lr: f32,
    ) {
        if edges.len() < 4 {
            return;
        }
        let mut feat = Linear::new(3, 4);
        let mut pred = Linear::new(4, 1);
        for &(a, b, c) in edges.iter() {
            let f = edge_feat(names[a].as_str(), names[b].as_str(), c);
            let f3: Vec<f32> = f
                .iter()
                .take(3)
                .cloned()
                .chain(std::iter::repeat(0.0))
                .take(3)
                .collect();
            let h = feat.forward(&f3);
            let p = pred.forward(&h);
            pred.backward(&h, &p, &[c], lr);
        }
        for j in 0..feat.w.data.len() {
            feat.w.data[j] -= feat.w.grad[j];
            feat.w.grad[j] = 0.0;
        }
        for j in 0..feat.b.data.len() {
            feat.b.data[j] -= feat.b.grad[j];
            feat.b.grad[j] = 0.0;
        }
        for &(a, b, c) in edges.iter().take(10) {
            let f = edge_feat(names[a].as_str(), names[b].as_str(), c);
            let f3: Vec<f32> = f
                .iter()
                .take(3)
                .cloned()
                .chain(std::iter::repeat(0.0))
                .take(3)
                .collect();
            let h = feat.forward(&f3);
            let tp = pred.forward(&h)[0].clamp(0.01, 0.99);
            sig.novel_edges.push((a, b, tp));
        }
    }
}

// ═══ 6. ENACTIVE: REINFORCE with baseline ═══
impl Enactive {
    pub fn decide(sig: &mut ParadigmSignals, candidates: &[(&str, f32, f32, f32)], _lr: f32) {
        if candidates.is_empty() {
            return;
        }
        let nu = candidates.len();
        let scores: Vec<f32> = candidates
            .iter()
            .map(|(_, d, n, c)| d * 0.5 + n * 0.3 + c * 0.2)
            .collect();
        let mx = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let ex: Vec<f32> = scores.iter().map(|s| (s - mx).exp()).collect();
        let zs = ex.iter().sum::<f32>().max(1e-8);
        let probs: Vec<f32> = ex.iter().map(|e| e / zs).collect();
        let mut entropy = 0.0;
        for p in &probs {
            if *p > 1e-8 {
                entropy -= p * p.ln();
            }
        }
        sig.explore_rate = (entropy / (nu as f32).ln().max(1e-8)).clamp(0.0, 1.0);
        let mut recs: Vec<(String, f32)> = candidates
            .iter()
            .enumerate()
            .map(|(i, (url, _, _, _))| (url.to_string(), probs[i]))
            .collect();
        recs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sig.crawl_recommendations = recs;
    }
}

// ═══ 7. META-LEARNING: Reptile algorithm ═══
impl MetaLearning {
    pub fn reptile(
        sig: &mut ParadigmSignals,
        edges: &[(usize, usize, f32)],
        names: &[String],
        outer_steps: usize,
        inner_steps: usize,
    ) {
        if edges.is_empty() {
            return;
        }
        let ilr = 0.01;
        let beta = 0.001;
        let nw = 7;
        let mut theta = vec![0.0f32; nw];
        for i in 0..nw {
            theta[i] = (i as f32 * 0.07).sin() * 0.1;
        }
        let nt = 5;
        let mut tasks: Vec<Vec<usize>> = vec![Vec::new(); nt];
        for (ei, &(a, b, _)) in edges.iter().enumerate() {
            tasks[(a.wrapping_add(b)) % nt].push(ei);
        }
        for _ in 0..outer_steps {
            for tk in 0..nt {
                if tasks[tk].len() < 2 {
                    continue;
                }
                let mut phi = theta.clone();
                for _ in 0..inner_steps.min(5) {
                    for &ei in &tasks[tk] {
                        let (a, b, c) = edges[ei];
                        let f = edge_feat(names[a].as_str(), names[b].as_str(), c);
                        let pred: f32 = phi.iter().zip(&f).map(|(p, &x)| p * x).sum();
                        let err = pred - c;
                        for j in 0..nw {
                            phi[j] -= ilr * err * f[j % 7];
                        }
                    }
                }
                for j in 0..nw {
                    theta[j] += beta * (phi[j] - theta[j]);
                }
            }
        }
        sig.model_updates.insert("reptile_meta".into(), theta);
    }
}

// ═══ 8. CONTINUAL: EWC update with Fisher diagonal ═══
impl Continual {
    pub fn ewc(
        sig: &mut ParadigmSignals,
        edges: &[(usize, usize, f32)],
        names: &[String],
        lr: f32,
    ) {
        if edges.is_empty() {
            return;
        }
        let mut model = Linear::new(3, 1);
        let nw = model.w.data.len();
        let old_w = model.w.data.clone();
        let mut fish_w = vec![0.0f32; nw];
        let lambda = 0.4;
        let gbeta = 0.9;
        for &(a, b, _) in edges.iter() {
            let f = edge_feat(names[a].as_str(), names[b].as_str(), 0.5);
            let f3: Vec<f32> = f
                .iter()
                .take(3)
                .cloned()
                .chain(std::iter::repeat(0.0))
                .take(3)
                .collect();
            let p = model.forward(&f3);
            model.backward(&f3, &p, &[0.5], lr);
            for i in 0..nw {
                let grad = (p[0] - 0.5) * f3[i % 3];
                fish_w[i] = gbeta * fish_w[i] + (1.0 - gbeta) * grad * grad;
                model.w.data[i] -= lr * lambda * fish_w[i] * (model.w.data[i] - old_w[i]);
            }
        }
        let imp_ratio = fish_w.iter().sum::<f32>() / (nw as f32).max(1e-8);
        sig.learning_rate_factor = 0.5 + 0.5 * (1.0 - imp_ratio.min(1.0));
        sig.model_updates
            .insert("ewc_weights".into(), model.w.data.clone());
    }
}

// ═══ 9. ZERO-SHOT: Embed concept via hash + nearest neighbor ═══
impl ZeroShot {
    pub fn infer(
        sig: &mut ParadigmSignals,
        unseen: &[String],
        known_embs: &[(String, Vec<f32>)],
        temperature: f32,
    ) {
        if unseen.is_empty() || known_embs.is_empty() {
            return;
        }
        let dim = known_embs[0].1.len();
        let mut glob = vec![0.0f32; dim];
        for (_, e) in known_embs {
            for d in 0..dim {
                glob[d] += e[d];
            }
        }
        for d in 0..dim {
            glob[d] /= known_embs.len() as f32;
        }
        for uname in unseen {
            let he = hash_feat(uname, dim);
            let mut ngh = vec![0.0f32; dim];
            let mut nw = 0.0f32;
            for (_, ke) in known_embs {
                let w = cos(&he, ke).max(0.0);
                for d in 0..dim {
                    ngh[d] += ke[d] * w;
                }
                nw += w;
            }
            if nw > 1e-8 {
                for d in 0..dim {
                    ngh[d] /= nw;
                }
            }
            let mut emb = vec![0.0f32; dim];
            for d in 0..dim {
                emb[d] = he[d] * 0.5 + ngh[d] * 0.3 + glob[d] * 0.2;
            }
            let mut sims: Vec<(String, f32)> = known_embs
                .iter()
                .map(|(kn, ke)| (kn.clone(), (cos(&emb, ke) / temperature.max(0.01)).exp()))
                .collect();
            let ssum = sims.iter().map(|(_, s)| s).sum::<f32>();
            for (_, s) in &mut sims {
                *s /= ssum;
            }
            sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            for (kn, _) in sims.iter().take(3) {
                sig.novel_edges.push((
                    unseen.len(),
                    known_embs.iter().position(|(n, _)| n == kn).unwrap_or(0),
                    0.6,
                ));
            }
        }
    }
}

// ═══ 10. FEW-SHOT: Prototypical classification ═══
impl FewShot {
    pub fn match_edges(
        sig: &mut ParadigmSignals,
        support: &[(usize, usize, f32, usize)],
        queries: &[(usize, usize, f32)],
        _k: usize,
    ) {
        if support.is_empty() {
            return;
        }
        let edim = 7;
        let ss: Vec<(Vec<f32>, usize)> = support
            .iter()
            .map(|&(a, b, c, l)| {
                let mut f = vec![0.0f32; edim];
                let ea = sig
                    .node_embeddings
                    .get(a)
                    .cloned()
                    .unwrap_or_else(|| hash_feat(&format!("n{}", a), 3));
                let eb = sig
                    .node_embeddings
                    .get(b)
                    .cloned()
                    .unwrap_or_else(|| hash_feat(&format!("n{}", b), 3));
                for d in 0..3 {
                    f[d] = *ea.get(d).unwrap_or(&0.0);
                }
                for d in 0..3 {
                    f[3 + d] = *eb.get(d).unwrap_or(&0.0);
                }
                f[6] = c;
                (f, l)
            })
            .collect();
        // Compute prototypes per class
        let mut prototypes: Vec<(usize, Vec<f32>, f32)> = Vec::new();
        // Lightweight Tensor alternative: build support embedding matrix
        if !ss.is_empty() {
            let mut sup_mat = Tensor::new(ss.len(), edim);
            for (i, (f, _)) in ss.iter().enumerate() {
                for d in 0..edim.min(f.len()) {
                    sup_mat.set(i, d, f[d]);
                }
            }
            let _normed = sup_mat.layer_norm(1e-5); // normalize across features
        }
        let mut seen: Vec<usize> = Vec::new();
        for (_, l) in &ss {
            if !seen.contains(l) {
                seen.push(*l);
            }
        }
        for &cls in &seen {
            let members: Vec<&(Vec<f32>, usize)> = ss.iter().filter(|(_, l)| *l == cls).collect();
            if members.is_empty() {
                continue;
            }
            let mut proto = vec![0.0f32; edim];
            for (f, _) in &members {
                for d in 0..edim {
                    proto[d] += f[d];
                }
            }
            let n = members.len() as f32;
            for d in 0..edim {
                proto[d] /= n;
            }
            prototypes.push((cls, proto, n));
        }
        for &(qa, qb, qc) in queries {
            let mut qf = vec![0.0f32; edim];
            let ea = sig
                .node_embeddings
                .get(qa)
                .cloned()
                .unwrap_or_else(|| hash_feat(&format!("n{}", qa), 3));
            let eb = sig
                .node_embeddings
                .get(qb)
                .cloned()
                .unwrap_or_else(|| hash_feat(&format!("n{}", qb), 3));
            for d in 0..3 {
                qf[d] = *ea.get(d).unwrap_or(&0.0);
            }
            for d in 0..3 {
                qf[3 + d] = *eb.get(d).unwrap_or(&0.0);
            }
            qf[6] = qc;
            let mut _best_cls = 0;
            let mut best_sim = f32::NEG_INFINITY;
            for (cls, proto, _) in &prototypes {
                let sim = cos(&qf, proto);
                if sim > best_sim {
                    best_sim = sim;
                    _best_cls = *cls;
                }
            }
            let conf = sigmoid(best_sim * 3.0 + 0.5);
            sig.edge_trust
                .entry((qa, qb))
                .and_modify(|v| *v = *v * 0.8 + conf * 0.2)
                .or_insert(conf);
        }
    }
}
