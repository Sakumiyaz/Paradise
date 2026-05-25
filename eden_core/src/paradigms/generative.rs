// paradigms/generative.rs — Diffusion · Adversarial · RAG
use super::autograd::Linear;
use super::ParadigmSignals;
use std::collections::HashSet;

pub fn prng() -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::Instant::now().elapsed().as_nanos().hash(&mut h);
    (h.finish() as f32) / (u64::MAX as f32)
}
pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x.clamp(-15.0, 15.0)).exp())
}
fn cosim(a: &[f32], b: &[f32]) -> f32 {
    let (mut d, mut na, mut nb) = (0.0, 0.0, 0.0);
    for i in 0..a.len().min(b.len()) {
        d += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    d / (na.sqrt() * nb.sqrt()).max(1e-8)
}

// ═══ Diffusion: DDPM with Linear::new(1,1), 10 timesteps ═══
pub struct Diffusion {
    pub ts: usize,
    pub ab: Vec<f32>,
    pub model: Linear,
    pub loss_h: Vec<f32>,
}
impl Diffusion {
    pub fn new(ts: usize) -> Self {
        let s0 = 0.008;
        let tf = ts as f32;
        let mut ab = Vec::with_capacity(ts + 1);
        for i in 0..=ts {
            let frac = (i as f32 / tf + s0) / (1.0 + s0);
            ab.push(
                (frac * std::f32::consts::FRAC_PI_2).cos().powi(2)
                    / (s0 / (1.0 + s0) * std::f32::consts::FRAC_PI_2)
                        .cos()
                        .powi(2),
            );
        }
        Diffusion {
            ts,
            ab,
            model: Linear::new(1, 1),
            loss_h: vec![],
        }
    }

    pub fn abar(&self, t: usize) -> f32 {
        self.ab[t.min(self.ts)].clamp(0.0, 1.0)
    }

    pub fn forward(&self, x0: &[f32], t: usize) -> (Vec<f32>, Vec<f32>) {
        let ab = self.abar(t);
        let noise: Vec<f32> = (0..x0.len())
            .map(|i| {
                (i.wrapping_mul(2654435761)
                    .wrapping_add(t.wrapping_mul(1597334677)) as f32
                    * 0.001)
                    .sin()
            })
            .collect();
        let xt: Vec<f32> = x0
            .iter()
            .zip(&noise)
            .map(|(&x, &e)| ab.sqrt() * x + (1.0 - ab).sqrt() * e)
            .collect();
        (xt, noise)
    }

    pub fn train_step(&mut self, x0: &[f32], t: usize, lr: f32) -> f32 {
        if x0.is_empty() || t >= self.ts {
            return 0.0;
        }
        let (xt, noise) = self.forward(x0, t);
        let mut loss = 0.0f32;
        let n = xt.len();
        for i in 0..n {
            let inp = &[xt[i]];
            let pred = self.model.forward(inp)[0];
            let err = noise[i] - pred;
            loss += err * err;
        }
        loss /= n.max(1) as f32;
        for i in 0..n {
            let inp = &[xt[i]];
            let p = self.model.forward(inp);
            self.model.backward(inp, &p, &[noise[i]], lr * 0.01);
        }
        self.loss_h.push(loss);
        loss
    }

    pub fn train_batch(
        &mut self,
        seqs: &[Vec<f32>],
        lr: f32,
        signals: &mut ParadigmSignals,
    ) -> f32 {
        if seqs.is_empty() {
            return 0.0;
        }
        let mut tl = 0.0;
        for seq in seqs.iter() {
            let t = ((seq.len() * 7 + 3) % self.ts.max(1)) as usize;
            tl += self.train_step(seq, t, lr);
        }
        let avg = tl / seqs.len().max(1) as f32;
        for seq in seqs.iter().take(20) {
            let t = (prng() * self.ts as f32) as usize;
            let (xt, _) = self.forward(seq, t);
            signals.edge_scorer_examples.push((
                xt.iter().take(8).cloned().collect(),
                seq.iter().sum::<f32>() / seq.len().max(1) as f32,
            ));
        }
        signals.activations.push("diffusion".into());
        avg
    }

    pub fn reverse(&mut self, noisy: &[f32], steps: usize) -> Vec<f32> {
        let mut x = noisy.to_vec();
        for si in (1..steps).rev() {
            let t = (si as f32 / steps as f32 * (self.ts - 1) as f32) as usize;
            let ab = self.abar(t);
            for i in 0..x.len() {
                let pred = self.model.forward(&[x[i]])[0];
                x[i] = (x[i] - (1.0 - ab).sqrt().max(1e-8) * pred) / ab.sqrt().max(1e-8);
            }
        }
        x.iter().map(|&v| v.clamp(0.0, 1.0)).collect()
    }

    pub fn predict_edges(&mut self, edges: &[(usize, usize, f32)], signals: &mut ParadigmSignals) {
        if edges.is_empty() {
            return;
        }
        let seq: Vec<f32> = edges.iter().map(|&(_, _, c)| c).collect();
        let dn = self.reverse(&seq, self.ts);
        for (i, &(a, b, _)) in edges.iter().enumerate() {
            if let Some(&dc) = dn.get(i) {
                let old = signals.edge_trust.get(&(a, b)).copied().unwrap_or(0.5);
                signals
                    .edge_trust
                    .insert((a, b), old * 0.5 + dc.clamp(0.0, 1.0) * 0.5);
            }
        }
    }
}

// ═══ Adversarial: generator/discriminator with Linear::new(1,1) each ═══
pub struct Adversarial {
    pub gen: Linear,
    pub disc: Linear,
    pub gl: f32,
    pub dl: f32,
}
impl Adversarial {
    pub fn new() -> Self {
        Adversarial {
            gen: Linear::new(1, 1),
            disc: Linear::new(1, 1),
            gl: 0.0,
            dl: 0.0,
        }
    }

    pub fn train_critic(&mut self, real: &[f32], fake: &[f32], lr: f32) -> f32 {
        if real.is_empty() || fake.is_empty() {
            return 0.0;
        }
        let dr = self.disc.forward(real)[0];
        let df = self.disc.forward(fake)[0];
        let loss = df - dr + 0.01 * (dr.powi(2) + df.powi(2));
        self.dl = loss;
        self.disc.backward(real, &[dr], &[1.0], lr * 0.1);
        self.disc.backward(fake, &[df], &[0.0], lr * 0.1);
        loss
    }

    pub fn train_gen(&mut self, z: &[f32], lr: f32) -> f32 {
        let fake = self.gen.forward(z);
        let df_vec = self.disc.forward(&fake);
        let df = df_vec[0];
        self.gl = -df;
        self.gen.backward(z, &fake, &vec![1.0], lr * 0.1);
        -df
    }

    pub fn train_epoch(
        &mut self,
        edges: &[(usize, usize, f32)],
        lr: f32,
        signals: &mut ParadigmSignals,
    ) -> (f32, f32) {
        if edges.is_empty() {
            return (0.0, 0.0);
        }
        let (mut cd, mut cg) = (0.0, 0.0);
        let b = edges.len().min(20);
        for i in 0..b {
            let &(a, bb, conf) = &edges[i % edges.len()];
            let real = &[conf];
            let z = &[(a.wrapping_add(bb) as f32 * 0.01).sin().abs()];
            cd += self.train_critic(real, z, lr * 0.2);
            cg += self.train_gen(z, lr);
        }
        let n = b as f32;
        cd /= n;
        cg /= n;
        for &(a, b_, conf) in edges.iter().take(30) {
            let sc = sigmoid(self.disc.forward(&[conf])[0].clamp(-5.0, 5.0));
            let old = signals.edge_trust.get(&(a, b_)).copied().unwrap_or(0.5);
            signals.edge_trust.insert((a, b_), old * 0.7 + sc * 0.3);
        }
        for _ in 0..10 {
            let z = prng();
            let fc = self.gen.forward(&[z])[0];
            if self.disc.forward(&[fc])[0] > 0.4 {
                signals
                    .novel_edges
                    .push(((z * 100.0) as usize, (z * 200.0) as usize, fc));
            }
        }
        signals.activations.push("adversarial".into());
        (cd, cg)
    }
}

// ═══ RAG: Cosine similarity retrieval on embeddings ═══
pub struct RAG {
    pub idx: Vec<Vec<f32>>,
    pub ids: Vec<usize>,
}
impl RAG {
    pub fn new() -> Self {
        RAG {
            idx: vec![],
            ids: vec![],
        }
    }

    pub fn build(&mut self, signals: &ParadigmSignals, top_k: usize) {
        self.idx.clear();
        self.ids.clear();
        for (i, emb) in signals.node_embeddings.iter().enumerate().take(top_k) {
            self.idx.push(emb.clone());
            self.ids.push(i);
        }
    }

    pub fn build_raw(&mut self, embs: &[Vec<f32>], node_ids: &[usize]) {
        self.idx = embs.to_vec();
        self.ids = node_ids.to_vec();
    }

    pub fn retrieve(&self, query: &[f32], top_k: usize) -> Vec<(usize, f32)> {
        if self.idx.is_empty() {
            return vec![];
        }
        let mut cand: Vec<(usize, f32)> = self
            .ids
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, cosim(query, &self.idx[i])))
            .collect();
        cand.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        cand.truncate(top_k);
        cand
    }

    pub fn embed_query(
        &self,
        node: usize,
        adj: &[Vec<(usize, f32)>],
        signals: &ParadigmSignals,
    ) -> Option<Vec<f32>> {
        let pos = self.ids.iter().position(|&id| id == node)?;
        let q_emb = &self.idx[pos];
        let dim = q_emb.len();
        if node >= adj.len() || adj[node].is_empty() {
            return Some(q_emb.clone());
        }
        let trust = signals
            .source_scores
            .values()
            .fold(0.5f32, |m, &v| m.max(v));
        let mut wtd: Vec<(Vec<f32>, f32)> = adj[node]
            .iter()
            .filter_map(|&(nb, conf)| {
                let npos = self.ids.iter().position(|&id| id == nb)?;
                Some((self.idx[npos].clone(), conf * trust))
            })
            .collect();
        wtd.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        wtd.truncate(3);
        let mut avg = vec![0.0f32; dim];
        let mut tw = 0.0f32;
        for (emb, w) in &wtd {
            for d in 0..dim {
                avg[d] += emb[d] * w;
            }
            tw += w;
        }
        if tw < 1e-6 {
            return Some(q_emb.clone());
        }
        for d in 0..dim {
            avg[d] = q_emb[d] * 0.3 + (avg[d] / tw) * 0.7;
        }
        Some(avg)
    }

    pub fn predict_links(
        &self,
        node: usize,
        adj: &[Vec<(usize, f32)>],
        signals: &mut ParadigmSignals,
        top_k: usize,
    ) {
        let query = match self.embed_query(node, adj, signals) {
            Some(q) => q,
            None => return,
        };
        let existing: HashSet<usize> = if node < adj.len() {
            adj[node].iter().map(|&(n, _)| n).collect()
        } else {
            HashSet::new()
        };
        for &(target, sim) in self.retrieve(&query, top_k * 2).iter() {
            if target != node && !existing.contains(&target) && sim > 0.3 {
                signals.novel_edges.push((node, target, sim * 0.7));
            }
        }
        let cn = [
            "physics", "bio", "cs", "phil", "math", "chem", "psych", "econ", "hist", "ling",
        ];
        for &(target, sim) in self.retrieve(&query, 10).iter() {
            signals
                .crawl_recommendations
                .push((format!("concept_{}", cn[target % cn.len()]), sim));
        }
        signals.activations.push("rag".into());
    }
    /// Multi-hop: 2-hop indirect connections via neighbor-of-neighbor with Bayesian confidence
    pub fn predict_links_2hop(
        &self,
        node: usize,
        adj: &[Vec<(usize, f32)>],
        signals: &mut ParadigmSignals,
    ) {
        if node >= adj.len() {
            return;
        }
        let q = match self.embed_query(node, adj, signals) {
            Some(q) => q,
            None => return,
        };
        let direct: HashSet<usize> = adj[node].iter().map(|&(n, _)| n).collect();
        let mut candidates: Vec<(usize, f32)> = Vec::new();
        // 2-hop: for each neighbor, check its neighbors
        for &(n1, w1) in &adj[node] {
            if n1 < adj.len() {
                for &(n2, w2) in &adj[n1] {
                    if n2 != node && !direct.contains(&n2) {
                        let conf = (w1 * w2).min(1.0);
                        let ret_sim = self
                            .retrieve(&q, 3)
                            .iter()
                            .find(|(t, _)| *t == n2)
                            .map(|(_, s)| *s)
                            .unwrap_or(0.0);
                        let sim: f32 =
                            q.iter().map(|&x| x * ret_sim).sum::<f32>() / q.len().max(1) as f32;
                        candidates.push((n2, conf * 0.5 + sim));
                        if candidates.len() > 20 {
                            candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                            candidates.truncate(10);
                        }
                    }
                }
            }
        }
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for (n2, score) in candidates.iter().take(5) {
            if *score > 0.25 {
                signals.novel_edges.push((node, *n2, *score));
            }
        }
        signals.activations.push("rag_2hop".into());
    }
}
