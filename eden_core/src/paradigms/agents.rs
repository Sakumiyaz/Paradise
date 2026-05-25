// paradigms/agents.rs — RL(Q-table) · Evolutionary · Embodied · RLHF
use super::autograd::Linear;
use super::ParadigmSignals;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};

fn prng() -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::Instant::now().elapsed().as_nanos().hash(&mut h);
    (h.finish() as f32) / (u64::MAX as f32)
}

// ═══ RL: Q-table with TD(0), epsilon-greedy ═══
pub struct RL {
    pub q: HashMap<u64, [f32; 5]>,
    pub eps: f32,
    pub buf: VecDeque<([f32; 5], usize, f32, [f32; 5], bool)>,
    pub step: u64,
}
impl RL {
    pub fn new() -> Self {
        RL {
            q: HashMap::new(),
            eps: 0.1,
            buf: VecDeque::new(),
            step: 0,
        }
    }

    fn hash_state(s: &[f32; 5]) -> u64 {
        let mut h: u64 = 14695981039346656037;
        for &v in s.iter() {
            let bits = (v * 100.0).round() as i64 as u64;
            h = h.wrapping_mul(1099511628211).wrapping_add(bits);
        }
        h
    }

    fn get_q(&self, s: &[f32; 5]) -> [f32; 5] {
        let key = Self::hash_state(s);
        self.q.get(&key).copied().unwrap_or([0.0; 5])
    }

    pub fn build_state(v: f32, eg: f32, d: f32, cs: f32, gd: f32) -> [f32; 5] {
        [v, eg, d, cs, gd]
    }

    pub fn act(&mut self, state: [f32; 5], signals: &mut ParadigmSignals) -> usize {
        let a = if prng() < self.eps {
            (prng() * 5.0) as usize
        } else {
            let q = Self::get_q(self, &state);
            q.iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(4)
        };
        signals.explore_rate = self.eps;
        let ns = [
            "crawl_physics",
            "crawl_bio",
            "crawl_random",
            "parse_deep",
            "sleep",
        ];
        if a < 5 {
            let q = Self::get_q(self, &state);
            signals
                .crawl_recommendations
                .push((ns[a].into(), 1.0 / (1.0 + (-q[a]).exp())));
        }
        a.min(4)
    }

    pub fn store(&mut self, s: [f32; 5], a: usize, r: f32, ns: [f32; 5], done: bool) {
        if self.buf.len() >= 200 {
            self.buf.pop_front();
        }
        self.buf.push_back((s, a, r, ns, done));
    }

    pub fn train_batch(&mut self, gamma: f32, lr: f32, _tau: f32) -> (f32, usize, f32) {
        if self.buf.is_empty() {
            let dq = self.get_q(&[0.0; 5]);
            let (mut ba, mut bq) = (4, f32::NEG_INFINITY);
            for i in 0..5 {
                if dq[i] > bq {
                    bq = dq[i];
                    ba = i;
                }
            }
            return (0.0, ba, 0.5);
        }
        let mut loss = 0.0f32;
        let mut count = 0;
        for &(s, a, r, ns, done) in self.buf.iter() {
            let qs = self.get_q(&s);
            let qn = self.get_q(&ns);
            let max_qn = qn.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let td_target = r + gamma * max_qn * if done { 0.0 } else { 1.0 };
            let td_error = td_target - qs[a];
            loss += td_error * td_error;
            count += 1;
            let key = Self::hash_state(&s);
            let entry = self.q.entry(key).or_insert([0.0; 5]);
            entry[a] += lr * td_error;
        }
        self.eps = (self.eps * 0.995).max(0.01);
        self.step += 1;
        let dq = self.get_q(&[0.0; 5]);
        let (mut ba, mut bq) = (4, f32::NEG_INFINITY);
        for i in 0..5 {
            if dq[i] > bq {
                bq = dq[i];
                ba = i;
            }
        }
        (loss / count.max(1) as f32, ba, 1.0 / (1.0 + (-bq).exp()))
    }
}

// ═══ Evolutionary: GA with SBX, poly mutation, elitism ═══
pub struct Evolutionary;
impl Evolutionary {
    fn sbx(p1: &[f32], p2: &[f32], eta: f32) -> (Vec<f32>, Vec<f32>) {
        let n = p1.len();
        let (mut c1, mut c2) = (p1.to_vec(), p2.to_vec());
        for i in 0..n {
            let u = prng();
            let beta = if u < 0.5 {
                (2.0 * u).powf(1.0 / (eta + 1.0))
            } else {
                (1.0 / (2.0 * (1.0 - u))).powf(1.0 / (eta + 1.0))
            };
            c1[i] = (0.5 * ((1.0 + beta) * p1[i] + (1.0 - beta) * p2[i])).clamp(0.0, 1.0);
            c2[i] = (0.5 * ((1.0 - beta) * p1[i] + (1.0 + beta) * p2[i])).clamp(0.0, 1.0);
        }
        (c1, c2)
    }
    fn mutate(g: &mut [f32], rate: f32, eta: f32) {
        for x in g.iter_mut() {
            if prng() < rate {
                let r = prng();
                let d = if r < 0.5 {
                    (2.0 * r).powf(1.0 / (eta + 1.0)) - 1.0
                } else {
                    1.0 - (2.0 * (1.0 - r)).powf(1.0 / (eta + 1.0))
                };
                *x = (*x + d * 0.5).clamp(0.0, 1.0);
            }
        }
    }
    fn tourney(fit: &[f32], k: usize) -> usize {
        let n = fit.len();
        let mut b = 0;
        let mut bf = f32::NEG_INFINITY;
        for _ in 0..k {
            let i = (prng() * n as f32) as usize % n;
            if fit[i] > bf {
                bf = fit[i];
                b = i;
            }
        }
        b
    }

    pub fn evolve(signals: &mut ParadigmSignals, fitnesses: &[f32]) -> Vec<f32> {
        let (n, g, d) = (20usize, 20usize, 16usize);
        let mut pop: Vec<Vec<f32>> = (0..n)
            .map(|i| {
                (0..d)
                    .map(|j| ((i * 13 + j * 7 + 3) as f32 * 0.01).sin().abs())
                    .collect()
            })
            .collect();
        let mut fit = fitnesses.to_vec();
        fit.resize(n, 0.1);
        for _ in 0..g {
            let mut idx: Vec<usize> = (0..n).collect();
            idx.sort_by(|&a, &b| fit[b].partial_cmp(&fit[a]).unwrap());
            let mut next: Vec<Vec<f32>> = idx.iter().take(2).map(|&i| pop[i].clone()).collect();
            while next.len() < n {
                let p1 = Self::tourney(&fit, 5);
                let p2 = Self::tourney(&fit, 5);
                let (mut c1, mut c2) = Self::sbx(&pop[p1], &pop[p2], 15.0);
                Self::mutate(&mut c1, 0.1, 20.0);
                Self::mutate(&mut c2, 0.1, 20.0);
                next.push(c1);
                if next.len() < n {
                    next.push(c2);
                }
            }
            pop = next;
        }
        let mut bi = 0;
        let mut bf = f32::NEG_INFINITY;
        for i in 0..n {
            if fit[i] > bf {
                bf = fit[i];
                bi = i;
            }
        }
        let best = &pop[bi];
        let cn = ["physics", "bio", "cs", "phil", "math"];
        let ln = ["en", "es", "fr"];
        for ci in 0..5 {
            for li in 0..3 {
                let p = best[ci] * best[5 + li] * ((best[8] + 1.0) / 2.0).max(0.1);
                if p > 0.05 {
                    signals
                        .crawl_recommendations
                        .push((format!("{}_{}", cn[ci], ln[li]), p));
                }
            }
        }
        signals.activations.push("evolutionary".into());
        best.clone()
    }
}

// ═══ Embodied: A* on graph adjacency ═══
pub struct Embodied;
impl Embodied {
    pub fn find_bridges(
        adj: &[Vec<(usize, f32)>],
        signals: &mut ParadigmSignals,
    ) -> Vec<(usize, usize, f32)> {
        let n = adj.len();
        if n < 2 {
            return vec![];
        }
        let (mut src, mut dst, mut mxd) = (0, 1, 0.0);
        for i in 0..n.min(50) {
            for j in (i + 1)..n.min(50) {
                let d = (adj[i].len() as f32 - adj[j].len() as f32).abs();
                if d > mxd {
                    mxd = d;
                    src = i;
                    dst = j;
                }
            }
        }
        let path = Self::astar_graph(adj, src, dst);
        let mut br = vec![];
        for w in path.windows(2) {
            let (a, b) = (w[0], w[1]);
            if !adj[a].iter().any(|&(nb, _)| nb == b) {
                let sc = 0.5 + 0.5 / (1.0 + path.len() as f32);
                signals.novel_edges.push((a, b, sc));
                br.push((a, b, sc));
            }
        }
        if !br.is_empty() {
            signals.activations.push("embodied".into());
        }
        br
    }

    pub fn astar_graph(adj: &[Vec<(usize, f32)>], start: usize, goal: usize) -> Vec<usize> {
        if start == goal {
            return vec![start];
        }
        let n = adj.len();
        struct N(f32, usize);
        impl PartialEq for N {
            fn eq(&self, o: &Self) -> bool {
                self.0 == o.0 && self.1 == o.1
            }
        }
        impl Eq for N {}
        impl PartialOrd for N {
            fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
                o.0.partial_cmp(&self.0)
            }
        }
        impl Ord for N {
            fn cmp(&self, o: &Self) -> Ordering {
                self.partial_cmp(o).unwrap()
            }
        }
        let mut g = vec![f32::MAX; n];
        let mut c: Vec<Option<usize>> = vec![None; n];
        let mut d = vec![0; n];
        let mut o = BinaryHeap::new();
        g[start] = 0.0;
        o.push(N(0.0, start));
        while let Some(N(_, cur)) = o.pop() {
            if cur == goal {
                let mut p = vec![goal];
                let mut x = goal;
                while let Some(y) = c[x] {
                    p.push(y);
                    x = y;
                }
                p.reverse();
                return p;
            }
            if d[cur] >= 6 || cur >= n {
                continue;
            }
            for &(nb, conf) in &adj[cur] {
                if nb >= n {
                    continue;
                }
                let t = g[cur] + (1.0 - conf.clamp(0.0, 1.0));
                if t < g[nb] {
                    g[nb] = t;
                    c[nb] = Some(cur);
                    d[nb] = d[cur] + 1;
                    o.push(N(t, nb));
                }
            }
        }
        vec![]
    }
}

// ═══ RLHF: Simple preference model with Linear::new(2,1), BCE loss ═══
pub struct RLHF {
    pub model: Linear,
    pub loss_h: Vec<f32>,
}
impl RLHF {
    pub fn new() -> Self {
        RLHF {
            model: Linear::new(2, 1),
            loss_h: vec![],
        }
    }

    fn sig(x: f32) -> f32 {
        1.0 / (1.0 + (-x.clamp(-15.0, 15.0)).exp())
    }

    pub fn train_step(
        &mut self,
        hf: &[f32; 4],
        lf: &[f32; 4],
        lr: f32,
        _beta: f32,
        signals: &mut ParadigmSignals,
    ) -> f32 {
        let feat_high = &[hf[0], hf[1]];
        let feat_low = &[lf[0], lf[1]];
        let ph = self.model.forward(feat_high)[0];
        let pl = self.model.forward(feat_low)[0];
        let (ph_s, pl_s) = (Self::sig(ph).max(1e-7), Self::sig(pl).max(1e-7));
        let loss = -ph_s.ln() - (1.0 - pl_s).max(1e-7).ln();
        self.loss_h.push(loss);
        let pred_h = vec![ph];
        let pred_l = vec![pl];
        self.model.backward(feat_high, &pred_h, &[1.0], lr);
        self.model.backward(feat_low, &pred_l, &[0.0], lr);
        signals.activations.push("rlhf".into());
        signals
            .model_updates
            .insert("rlhf_pref".into(), vec![ph_s, pl_s, loss]);
        signals.source_scores.insert("pref_high".into(), ph_s);
        signals.source_scores.insert("pref_low".into(), 1.0 - pl_s);
        loss
    }

    pub fn train_on_sources(
        &mut self,
        signals: &mut ParadigmSignals,
        lr: f32,
        epochs: usize,
    ) -> f32 {
        if signals.source_scores.len() < 2 {
            return 0.0;
        }
        let mut sv: Vec<(String, f32)> = signals
            .source_scores
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        sv.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let (hn, hs) = (&sv[0].0, sv[0].1);
        let (ln, ls) = (&sv[sv.len() - 1].0, sv[sv.len() - 1].1);
        let feat = |n: &str, s: f32| -> [f32; 4] {
            let nf = n.bytes().fold(0.0, |a, b| a + b as f32 * 0.01);
            [
                s,
                (s - 0.5).abs(),
                (n.len() as f32 * 0.1).min(1.0),
                nf.sin().abs(),
            ]
        };
        let hf = feat(hn, hs);
        let lf = feat(ln, ls);
        let mut tl = 0.0;
        for _ in 0..epochs {
            tl += self.train_step(&hf, &lf, lr, 0.9, signals);
        }
        tl / epochs.max(1) as f32
    }

    /// External API: score a source name+trust pair (kept for API surface / backward compat)
    #[allow(dead_code)]
    pub fn score(&mut self, name: &str, trust: f32) -> f32 {
        let nf = name.bytes().fold(0.0, |a, b| a + b as f32 * 0.01);
        let feat = [trust, nf.sin().abs()];
        Self::sig(self.model.forward(&feat)[0])
    }
}
