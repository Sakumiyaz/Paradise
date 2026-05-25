// paradigms/classic.rs — KNN · SVM(perceptron) · DecisionTree · HMM
use std::collections::HashMap;

pub struct KNN;
impl KNN {
    pub fn extract_features(ec: f32, corr: u32, stm: f32, age: u64, ndr: f32) -> Vec<f32> {
        vec![
            ec,
            (corr as f32 * 0.2).min(1.0),
            stm,
            (age as f32 * 0.01).min(1.0),
            ndr.clamp(0.0, 1.0),
        ]
    }

    pub fn classify_weighted(
        train_f: &[Vec<f32>],
        train_l: &[f32],
        qf: &[Vec<f32>],
        k: usize,
    ) -> Vec<f32> {
        let k = k.min(train_f.len().max(1));
        qf.iter()
            .map(|qi| {
                let mut ds: Vec<(f32, usize)> = train_f
                    .iter()
                    .enumerate()
                    .map(|(ti, tf)| {
                        let d2: f32 = tf.iter().zip(qi).map(|(a, b)| (a - b).powi(2)).sum::<f32>()
                            / tf.len().max(1) as f32;
                        (d2, ti)
                    })
                    .collect();
                ds.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                let (mut ws, mut wl) = (0.0f32, 0.0f32);
                for (rk, &(d2, ti)) in ds.iter().enumerate().take(k) {
                    let w = 1.0 / (d2 + 0.001) * (k - rk) as f32;
                    ws += w;
                    wl += w * train_l[ti];
                }
                if ws > 0.0 {
                    wl / ws
                } else {
                    0.5
                }
            })
            .collect()
    }

    pub fn score_edges(
        train_f: &[Vec<f32>],
        train_l: &[f32],
        qf: &[Vec<f32>],
        qp: &[(usize, usize)],
    ) -> (Vec<(usize, usize, f32)>, HashMap<(usize, usize), f32>) {
        let sc = Self::classify_weighted(train_f, train_l, qf, 7.min(train_f.len().max(1)));
        let mut tm = HashMap::new();
        let sd: Vec<_> = qp
            .iter()
            .zip(sc.iter())
            .map(|((a, b), s)| {
                tm.insert((*a, *b), *s);
                (*a, *b, *s)
            })
            .collect();
        (sd, tm)
    }

    pub fn rank_novel(
        train_f: &[Vec<f32>],
        train_l: &[f32],
        pairs: &[(usize, usize)],
        cf: &[Vec<f32>],
    ) -> (Vec<(usize, usize, f32)>, HashMap<(usize, usize), f32>) {
        let sc = Self::classify_weighted(train_f, train_l, cf, 7.min(train_f.len().max(1)));
        let mut nv: Vec<_> = pairs
            .iter()
            .zip(sc.iter())
            .map(|((a, b), s)| (*a, *b, *s))
            .collect();
        nv.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        nv.truncate(10);
        let mut tm = HashMap::new();
        for &(a, b, s) in &nv {
            tm.insert((a, b), s);
        }
        (nv, tm)
    }
}

// ═══ SVM: Perceptron-style gradient descent (no SMO) ═══
pub struct SVM;
impl SVM {
    pub fn extract_features(conf: f32, rel: u8, delta: u64, corr: u32) -> Vec<f32> {
        vec![
            conf,
            rel as f32 * 0.2,
            (delta as f32 * 0.001).min(1.0),
            (corr as f32 * 0.1).min(1.0),
        ]
    }

    fn dot(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }

    pub fn train(
        x: &[Vec<f32>],
        y: &[f32],
        _c: f32,
        _g: f32,
        mi: usize,
        _tol: f32,
    ) -> (Vec<f32>, f32, usize, f32) {
        let n = x.len();
        if n == 0 {
            return (vec![], 0.0, 0, 0.0);
        }
        let d = x[0].len();
        let mut w = vec![0.0f32; d];
        let mut b = 0.0f32;
        let lr = 0.01;
        for _ in 0..mi {
            let mut updated = false;
            for i in 0..n {
                let p = Self::dot(&w, &x[i]) + b;
                if y[i] * p <= 1.0 {
                    for j in 0..d {
                        w[j] += lr * y[i] * x[i][j];
                    }
                    b += lr * y[i];
                    updated = true;
                }
            }
            if !updated {
                break;
            }
        }
        let norm: f32 = w.iter().map(|v| v.powi(2)).sum::<f32>().sqrt();
        let margin = if norm > 1e-8 { 1.0 / norm } else { 0.0 };
        let alpha: Vec<f32> = w.iter().map(|&v| v.abs() / norm.max(1e-8)).collect();
        let svc = w.iter().filter(|&&v| v.abs() > 1e-6).count();
        (alpha, b, svc, margin)
    }

    pub fn predict(x: &[Vec<f32>], a: &[f32], y: &[f32], b: f32, _g: f32, q: &[f32]) -> f32 {
        let mut w = vec![0.0f32; a.len()];
        for i in 0..a.len().min(x.len()) {
            if a[i] > 1e-8 {
                for j in 0..x[i].len().min(w.len()) {
                    w[j] += a[i] * y[i] * x[i][j];
                }
            }
        }
        Self::dot(&w, q) + b
    }

    pub fn classify_edges(
        x: &[Vec<f32>],
        y: &[f32],
        c: f32,
        g: f32,
        mi: usize,
    ) -> (Vec<(Vec<f32>, f32)>, f32) {
        let (a, b, _sv, mg) = Self::train(x, y, c, g, mi, 1e-4);
        let pt = (1.0 - mg / 2.0).clamp(0.05, 0.95);
        let mut ex = Vec::new();
        for i in 0..x.len() {
            let p = Self::predict(x, &a, y, b, g, &x[i]);
            ex.push((x[i].clone(), 1.0 / (1.0 + (-p).exp())));
        }
        (ex, pt)
    }
}

// ═══ DecisionTree: CART with variance reduction ═══
pub enum TreeNode {
    Leaf(f32),
    Branch {
        f: usize,
        t: f32,
        l: Box<TreeNode>,
        r: Box<TreeNode>,
    },
}
pub struct DecisionTree;
impl DecisionTree {
    pub fn extract_features(cp: f32, gs: f32, fr: f32, ac: f32, nd: f32) -> Vec<f32> {
        vec![cp, gs, fr, ac, nd]
    }

    pub fn build(x: &[Vec<f32>], y: &[f32], depth: usize) -> Option<TreeNode> {
        let n = x.len();
        if n == 0 || x[0].is_empty() {
            return None;
        }
        let mean = y.iter().sum::<f32>() / n as f32;
        if depth >= 6 || n < 3 {
            return Some(TreeNode::Leaf(mean));
        }
        let var: f32 = y.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n as f32;
        if var < 1e-5 {
            return Some(TreeNode::Leaf(mean));
        }
        let (bf, bt, bg) = Self::best_split(x, y, var, n);
        if bg < 0.001 || bf >= x[0].len() {
            return Some(TreeNode::Leaf(mean));
        }
        let li: Vec<usize> = (0..n).filter(|&i| x[i][bf] <= bt).collect();
        let ri: Vec<usize> = (0..n).filter(|&i| x[i][bf] > bt).collect();
        if li.len() < 3 || ri.len() < 3 {
            return Some(TreeNode::Leaf(mean));
        }
        let (lx, ly): (Vec<_>, Vec<_>) = (
            li.iter().map(|&i| x[i].clone()).collect(),
            li.iter().map(|&i| y[i]).collect(),
        );
        let (rx, ry): (Vec<_>, Vec<_>) = (
            ri.iter().map(|&i| x[i].clone()).collect(),
            ri.iter().map(|&i| y[i]).collect(),
        );
        let l = Self::build(&lx, &ly, depth + 1).unwrap_or(TreeNode::Leaf(mean));
        let r = Self::build(&rx, &ry, depth + 1).unwrap_or(TreeNode::Leaf(mean));
        Some(TreeNode::Branch {
            f: bf,
            t: bt,
            l: Box::new(l),
            r: Box::new(r),
        })
    }

    fn best_split(x: &[Vec<f32>], y: &[f32], pvar: f32, n: usize) -> (usize, f32, f32) {
        let nf = x[0].len();
        let (mut bf, mut bt, mut bg) = (nf, 0.0, 0.0);
        for fi in 0..nf {
            let mut v: Vec<(f32, f32)> = x.iter().zip(y).map(|(r, &l)| (r[fi], l)).collect();
            v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let (mut sl, mut cl) = (0.0f32, 0usize);
            let (mut sr, mut cr): (f32, usize) = (y.iter().sum(), n);
            for i in 0..v.len() - 1 {
                sl += v[i].1;
                cl += 1;
                sr -= v[i].1;
                cr -= 1;
                if cl < 3 || cr < 3 || (v[i].0 - v[i + 1].0).abs() < 1e-8 {
                    continue;
                }
                let th = (v[i].0 + v[i + 1].0) * 0.5;
                let (ml, mr) = (sl / cl as f32, sr / cr as f32);
                let vl: f32 =
                    v[..=i].iter().map(|(_, lb)| (lb - ml).powi(2)).sum::<f32>() / cl as f32;
                let vr: f32 = v[i + 1..]
                    .iter()
                    .map(|(_, lb)| (lb - mr).powi(2))
                    .sum::<f32>()
                    / cr as f32;
                let gain = pvar - (cl as f32 * vl + cr as f32 * vr) / n as f32;
                if gain > bg {
                    bg = gain;
                    bf = fi;
                    bt = th;
                }
            }
        }
        (bf, bt, bg)
    }

    pub fn predict(node: &TreeNode, s: &[f32]) -> f32 {
        match node {
            TreeNode::Leaf(v) => *v,
            TreeNode::Branch { f, t, l, r } => {
                if s[*f] <= *t {
                    Self::predict(l, s)
                } else {
                    Self::predict(r, s)
                }
            }
        }
    }

    pub fn rank_categories(x: &[Vec<f32>], y: &[f32], names: &[String]) -> Vec<(String, f32)> {
        let t = match Self::build(x, y, 0) {
            Some(t) => t,
            None => return vec![],
        };
        let mut sc: Vec<_> = names
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if i < x.len() {
                    Some((n.clone(), Self::predict(&t, &x[i])))
                } else {
                    None
                }
            })
            .collect();
        sc.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sc.truncate(5);
        sc
    }
}

// ═══ HMM: Baum-Welch, 6-state, 4-dim emissions ═══
pub struct HMM;
pub const HMM_STATES: [&str; 6] = [
    "exploring",
    "learning",
    "consolidating",
    "stressed",
    "creative",
    "dormant",
];
impl HMM {
    fn emit(s: usize, o: &[f32], eb: &[Vec<f32>]) -> f32 {
        let d = o.len().min(eb[s].len() / 2);
        let mut ll = 0.0;
        for i in 0..d {
            let m = eb[s][i * 2];
            let v = eb[s][i * 2 + 1].max(0.01);
            ll += -0.5 * ((o[i] - m).powi(2) / v + (2.0 * std::f32::consts::PI * v).ln());
        }
        ll.exp()
    }

    fn fwd_bwd(
        obs: &[Vec<f32>],
        a: &[Vec<f32>],
        b: &[Vec<f32>],
        pi: &[f32],
    ) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
        let t = obs.len();
        let ns = a.len();
        if t == 0 {
            return (vec![], vec![]);
        }
        let (mut alf, mut bet) = (vec![vec![0.0; ns]; t], vec![vec![0.0; ns]; t]);
        for s in 0..ns {
            alf[0][s] = pi[s] * Self::emit(s, &obs[0], b);
        }
        for ti in 1..t {
            for s in 0..ns {
                let mut sm = 0.0;
                for ps in 0..ns {
                    sm += alf[ti - 1][ps] * a[ps][s];
                }
                alf[ti][s] = sm * Self::emit(s, &obs[ti], b);
            }
        }
        for s in 0..ns {
            bet[t - 1][s] = 1.0;
        }
        for ti in (0..t - 1).rev() {
            for s in 0..ns {
                let mut sm = 0.0;
                for ns2 in 0..ns {
                    sm += a[s][ns2] * Self::emit(ns2, &obs[ti + 1], b) * bet[ti + 1][ns2];
                }
                bet[ti][s] = sm;
            }
        }
        (alf, bet)
    }

    pub fn baum_welch(
        obs_sq: &[Vec<f32>],
        ns: usize,
        edim: usize,
        iters: usize,
    ) -> (Vec<Vec<f32>>, Vec<Vec<f32>>, Vec<f32>) {
        let t = obs_sq.len();
        if t < 2 {
            return Self::init(ns, edim);
        }
        let (mut a, mut b, mut pi) = Self::init(ns, edim);
        for _ in 0..iters.min(20) {
            let (alf, bet) = Self::fwd_bwd(obs_sq, &a, &b, &pi);
            let mut gam = vec![vec![0.0; ns]; t];
            for ti in 0..t {
                let nrm: f32 = (0..ns)
                    .map(|s| alf[ti][s] * bet[ti][s])
                    .sum::<f32>()
                    .max(1e-8);
                for s in 0..ns {
                    gam[ti][s] = alf[ti][s] * bet[ti][s] / nrm;
                }
            }
            let mut xi = vec![vec![vec![0.0; ns]; ns]; t - 1];
            for ti in 0..t - 1 {
                let nrm: f32 = (0..ns)
                    .map(|s| {
                        (0..ns)
                            .map(|ns2| {
                                alf[ti][s]
                                    * a[s][ns2]
                                    * Self::emit(ns2, &obs_sq[ti + 1], &b)
                                    * bet[ti + 1][ns2]
                            })
                            .sum::<f32>()
                    })
                    .sum::<f32>()
                    .max(1e-8);
                for s in 0..ns {
                    for ns2 in 0..ns {
                        xi[ti][s][ns2] = alf[ti][s]
                            * a[s][ns2]
                            * Self::emit(ns2, &obs_sq[ti + 1], &b)
                            * bet[ti + 1][ns2]
                            / nrm;
                    }
                }
            }
            for s in 0..ns {
                let dn: f32 = gam.iter().take(t - 1).map(|g| g[s]).sum::<f32>().max(1e-8);
                for ns2 in 0..ns {
                    a[s][ns2] = xi.iter().map(|r| r[s][ns2]).sum::<f32>() / dn;
                }
            }
            for s in 0..ns {
                let dn: f32 = gam.iter().map(|g| g[s]).sum::<f32>().max(1e-8);
                for d in 0..edim {
                    let mn: f32 = gam
                        .iter()
                        .enumerate()
                        .map(|(ti, g)| g[s] * obs_sq[ti][d])
                        .sum();
                    let vn: f32 = gam
                        .iter()
                        .enumerate()
                        .map(|(ti, g)| g[s] * (obs_sq[ti][d] - b[s][d * 2]).powi(2))
                        .sum();
                    b[s][d * 2] = mn / dn;
                    b[s][d * 2 + 1] = (vn / dn).max(0.01);
                }
            }
            pi = gam[0].clone();
        }
        (a, b, pi)
    }

    fn init(ns: usize, edim: usize) -> (Vec<Vec<f32>>, Vec<Vec<f32>>, Vec<f32>) {
        let a = vec![vec![1.0 / ns as f32; ns]; ns];
        let b: Vec<Vec<f32>> = (0..ns)
            .map(|s| {
                let mut r = vec![0.0f32; edim * 2];
                for d in 0..edim {
                    r[d * 2] = (s as f32 - ns as f32 / 2.0) * 0.2 + (d as f32 * 0.1);
                    r[d * 2 + 1] = 1.0;
                }
                r
            })
            .collect();
        let pi = vec![1.0 / ns as f32; ns];
        (a, b, pi)
    }

    pub fn viterbi(obs: &[Vec<f32>], a: &[Vec<f32>], b: &[Vec<f32>], pi: &[f32]) -> Vec<usize> {
        let t = obs.len();
        if t == 0 {
            return vec![];
        }
        let ns = a.len();
        let (mut del, mut psi) = (vec![vec![0.0; ns]; t], vec![vec![0usize; ns]; t]);
        for s in 0..ns {
            del[0][s] = (pi[s] * Self::emit(s, &obs[0], b)).max(1e-12).ln();
        }
        for ti in 1..t {
            for s in 0..ns {
                let (mut best, mut bp) = (f32::NEG_INFINITY, 0);
                for ps in 0..ns {
                    let v = del[ti - 1][ps] + a[ps][s].max(1e-12).ln();
                    if v > best {
                        best = v;
                        bp = ps;
                    }
                }
                del[ti][s] = best + Self::emit(s, &obs[ti], b).max(1e-12).ln();
                psi[ti][s] = bp;
            }
        }
        let mut last = 0;
        let mut best = f32::NEG_INFINITY;
        for s in 0..ns {
            if del[t - 1][s] > best {
                best = del[t - 1][s];
                last = s;
            }
        }
        let mut path = vec![last; t];
        for ti in (0..t - 1).rev() {
            path[ti] = psi[ti + 1][path[ti + 1]];
        }
        path
    }

    pub fn predict_state(
        obs: &[Vec<f32>],
        a: &[Vec<f32>],
        b: &[Vec<Vec<f32>>],
        pi: &[f32],
    ) -> (usize, f32) {
        let path = Self::viterbi(obs, a, &b[0], pi);
        let last = *path.last().unwrap_or(&0);
        let ns = a.len();
        let (mut bs, mut bp) = (0usize, 0.0f32);
        for ns2 in 0..ns {
            if a[last][ns2] > bp {
                bp = a[last][ns2];
                bs = ns2;
            }
        }
        (bs, a[last][5].max(a[last][2]).clamp(0.0, 1.0))
    }

    pub fn train_on_emotions(
        em_seq: &[Vec<f32>],
        iters: usize,
    ) -> (Vec<Vec<f32>>, Vec<Vec<f32>>, Vec<f32>) {
        let edim = if em_seq.is_empty() {
            4
        } else {
            em_seq[0].len().min(4)
        };
        let obs: Vec<Vec<f32>> = em_seq
            .iter()
            .map(|v| {
                let mut t = v.clone();
                t.resize(edim, 0.0);
                t.truncate(edim);
                t
            })
            .collect();
        Self::baum_welch(&obs, 6, edim.max(1), iters)
    }
}
