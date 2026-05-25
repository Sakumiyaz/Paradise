// paradigms/frontier.rs — Spike · Quantum · Neuromorphic · NeuralODE · HyperNet · Siamese · Multimodal
use crate::paradigms::autograd::{Linear, Var};
use crate::paradigms::ParadigmSignals;
use std::collections::HashMap;

fn sigm(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}
fn softm(xs: &[f32]) -> Vec<f32> {
    let max = xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let e: Vec<f32> = xs.iter().map(|x| (x - max).exp()).collect();
    let s = e.iter().sum::<f32>().max(1e-10);
    e.iter().map(|x| x / s).collect()
}
fn csim(a: &[f32], b: &[f32]) -> f32 {
    let (d, sa, sb) = a
        .iter()
        .zip(b)
        .fold((0.0, 0.0, 0.0), |(d, sa, sb), (&x, &y)| {
            (d + x * y, sa + x * x, sb + y * y)
        });
    d / (sa.sqrt() * sb.sqrt()).max(1e-8)
}
fn l2(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

// ═══ SPIKE: Izhikevich neuron network ═══
pub struct Spike;
impl Spike {
    fn izh(v: f32, u: f32, a: f32, b: f32, c: f32, d: f32, ii: f32) -> (f32, f32, bool) {
        let dv = 0.04 * v * v + 5.0 * v + 140.0 - u + ii;
        let du = a * (b * v - u);
        let (vn, un) = (v + dv * 0.5, u + du * 0.5);
        if vn >= 30.0 {
            (c, un + d, true)
        } else {
            (vn, un, false)
        }
    }

    pub fn spike_tick(
        sig: &mut ParadigmSignals,
        _nn: usize,
        ne: usize,
        econf: &[(usize, usize, f32)],
        corr: usize,
        cyc: u64,
    ) -> String {
        let n = 10;
        let prm: [(f32, f32, f32, f32); 10] = [
            (0.1, 0.2, -65.0, 2.0),
            (0.1, 0.2, -65.0, 2.0),
            (0.1, 0.2, -65.0, 2.0),
            (0.02, 0.2, -65.0, 8.0),
            (0.02, 0.2, -65.0, 8.0),
            (0.02, 0.2, -65.0, 8.0),
            (0.02, 0.2, -65.0, 8.0),
            (0.02, 0.2, -65.0, 8.0),
            (0.02, 0.25, -55.0, 4.0),
            (0.02, 0.25, -55.0, 4.0),
        ];
        let mut ns: Vec<(f32, f32)> = vec![(-65.0, -13.0); n];
        let mut ww: Vec<Vec<f32>> = vec![vec![0.0f32; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    ww[i][j] = 0.3 * ((i * 7 + j * 13) as f32 * 0.01).sin().abs();
                }
            }
        }
        let ecr = (ne as f32 / cyc.max(1) as f32).min(10.0);
        let crr = if ne > 0 { corr as f32 / ne as f32 } else { 0.0 };
        let mut st: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut spike_counts = Vec::new();
        for t in 0..60 {
            let mut inp = vec![0.0f32; n];
            for i in 0..n {
                inp[i] = ecr * ((i + 1) as f32 * 0.5).sin().abs()
                    + crr * ((i as f32 * 1.3).sin()).abs() * 3.0;
            }
            let mut active = 0u32;
            for i in 0..n {
                let (a, b, c, d) = prm[i];
                let (nv, nu, spk) = Self::izh(ns[i].0, ns[i].1, a, b, c, d, inp[i] * 12.0);
                ns[i] = (nv, nu);
                if spk {
                    active += 1;
                    st[i].push(t);
                }
            }
            spike_counts.push(active);
        }
        let bursts = (0..60)
            .filter(|&t| {
                (0..n)
                    .filter(|&i| st[i].iter().any(|&s| (s as isize - t as isize).abs() <= 1))
                    .count()
                    >= 5
            })
            .count();
        sig.sleep_recommendation = (bursts as f32 / 10.0).min(1.0);
        sig.explore_rate = (spike_counts.iter().filter(|&&c| c <= 1).count() as f32
            / spike_counts.len().max(1) as f32)
            .clamp(0.1, 0.9);
        for &(a, _b, conf) in econf.iter().take(15) {
            let pv = ns[a % n].0;
            sig.edge_scorer_examples
                .push((vec![conf, sigm((pv + 85.0) * 0.1)], conf));
        }
        sig.activations.push("spike".into());
        format!("[SPIKE] 10n bursts={}", bursts)
    }
}

// ═══ QUANTUM: 3-qubit circuit ═══
pub struct Quantum;
impl Quantum {
    fn ry(st: &mut [[f32; 2]; 3], q: usize, th: f32) {
        let (cc, ss) = (th.cos(), th.sin());
        let (a0, a1) = (st[q][0], st[q][1]);
        st[q][0] = a0 * cc - a1 * ss;
        st[q][1] = a0 * ss + a1 * cc;
    }
    fn cnot(st: &mut [[f32; 2]; 3], c: usize, t: usize) {
        if st[c][1] * st[c][1] > 0.3 {
            let tmp = st[t][0];
            st[t][0] = st[t][1];
            st[t][1] = tmp;
        }
    }
    fn circ(ang: &[f32; 3]) -> [f32; 3] {
        let mut st = [[1.0f32, 0.0f32]; 3];
        for q in 0..3 {
            Self::ry(&mut st, q, ang[q]);
        }
        for q in 0..2 {
            Self::cnot(&mut st, q, q + 1);
        }
        Self::cnot(&mut st, 2, 0);
        for q in 0..3 {
            Self::ry(&mut st, q, ang[q] * 0.5);
        }
        let mut p = [0.0; 3];
        for q in 0..3 {
            p[q] = st[q][1] * st[q][1];
        }
        p
    }
    fn loss(ang: &[f32; 3], pa: f32, eg: f32) -> f32 {
        let p = Self::circ(ang);
        -(sigm(p.iter().map(|&x| 2.0 * x - 1.0).sum::<f32>() / 3.0) * (pa * 2.0 + eg * 0.5).abs())
    }

    pub fn quantum_tick(sig: &mut ParadigmSignals, pred_acc: f32, edge_gr: f32) -> String {
        let pi = std::f32::consts::PI;
        let mut ang = [
            sig.cooc_boost * pi,
            sig.embed_confidence * pi,
            sig.random_pages * pi,
        ];
        let (b1, b2, eps, lr) = (0.9, 0.999, 1e-8, 0.03);
        let (mut m, mut vv) = ([0.0f32; 3], [0.0f32; 3]);
        for t in 1..=20 {
            for q in 0..3 {
                let g = {
                    let mut ap = ang;
                    ap[q] += std::f32::consts::FRAC_PI_2;
                    let mut am = ang;
                    am[q] -= std::f32::consts::FRAC_PI_2;
                    (Self::loss(&ap, pred_acc, edge_gr) - Self::loss(&am, pred_acc, edge_gr)) / 2.0
                };
                m[q] = b1 * m[q] + (1.0 - b1) * g;
                vv[q] = b2 * vv[q] + (1.0 - b2) * g * g;
                let mh = m[q] / (1.0 - b1.powi(t as i32));
                let vh = vv[q] / (1.0 - b2.powi(t as i32));
                ang[q] = (ang[q] - lr * mh / (vh.sqrt() + eps)).clamp(0.0, pi);
            }
        }
        sig.cooc_boost = (ang[0] / pi).clamp(0.01, 2.0);
        sig.embed_confidence = (ang[1] / pi).clamp(0.01, 2.0);
        sig.activations.push("quantum".into());
        format!(
            "[QUANTUM] 3q cost={:.3}",
            Self::loss(&ang, pred_acc, edge_gr)
        )
    }
}

// ═══ NEUROMORPHIC: LIF neuron network ═══
pub struct Neuromorphic;
impl Neuromorphic {
    fn lif(v: f32, ii: f32, tau: f32, th: f32) -> (f32, bool) {
        let vn = v + (-v + ii) / tau;
        if vn >= th {
            (0.0, true)
        } else {
            (vn, false)
        }
    }

    pub fn neuromorphic_tick(sig: &mut ParadigmSignals, te: usize, tn: usize) -> String {
        let (rows, cols) = (4, 8);
        let n = rows * cols;
        let tau = 10.0;
        let target = 5.0;
        let mut vv = vec![0.0f32; n];
        let mut th = vec![1.0f32; n];
        let mut sh: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut fr = vec![0.0f32; n];
        let ed = if tn > 0 && te > 0 {
            (te as f32 / (tn.max(1) as f32 * 5.0)).min(3.0)
        } else {
            0.1
        };
        for t in 0..80 {
            let mut inp = vec![0.0f32; n];
            for i in 0..n {
                let rv = ((i / cols) as f32 * 0.3).sin() * ((i % cols) as f32 * 0.4).cos();
                inp[i] = ed * (1.0 + 0.3 * rv) * 6.0;
            }
            let mut inhib = vec![0.0f32; n];
            for i in 0..n {
                let (ri, ci) = ((i / cols) as isize, (i % cols) as isize);
                for j in 0..n {
                    if i == j {
                        continue;
                    }
                    let (rj, cj) = ((j / cols) as isize, (j % cols) as isize);
                    let d = (ri - rj).abs().max((ci - cj).abs()) as usize;
                    if d <= 2 {
                        inhib[i] += vv[j].max(0.0) * (0.5 / (d + 1) as f32);
                    }
                }
            }
            for i in 0..n {
                let ii = (inp[i] - inhib[i]).max(0.0);
                let (nv, spk) = Self::lif(vv[i], ii, tau, th[i]);
                vv[i] = nv;
                if spk {
                    sh[i].push(t);
                }
            }
            if t > 0 && t % 20 == 0 {
                for i in 0..n {
                    fr[i] = sh[i].iter().filter(|&&s| s >= t - 20).count() as f32;
                    th[i] = (th[i] + 0.05 * (fr[i] - target)).clamp(0.5, 3.0);
                }
            }
        }
        let mf = fr.iter().sum::<f32>() / n as f32;
        let sync = fr.iter().map(|&r| (r - mf).abs()).sum::<f32>() / n as f32;
        sig.sleep_recommendation = (sync / 5.0).min(1.0);
        sig.explore_rate =
            (fr.iter().filter(|&&r| r < 2.0).count() as f32 / n as f32).clamp(0.05, 0.95);
        sig.activations.push("neuromorphic".into());
        format!("[NEUROMORPHIC] 32LIF sync={:.2}", sig.sleep_recommendation)
    }
}

// ═══ NEURAL ODE: RK4 integration ═══
pub struct NeuralODE;
impl NeuralODE {
    fn ode(y: &[f32; 3], rn: f32, re: f32, a: f32, b: f32, crr: f32, k: f32) -> [f32; 3] {
        let n = y[0].max(1.0);
        let e = y[1].max(0.0);
        let c = y[2].clamp(0.0, 1.0);
        [
            rn * n * (1.0 - n / k),
            re * n * (1.0 - e / (n * 8.0).max(1.0)),
            -a * c + b * crr * (1.0 - c),
        ]
    }
    fn rk4(y: &[f32; 3], dt: f32, rn: f32, re: f32, a: f32, b: f32, crr: f32, k: f32) -> [f32; 3] {
        let f = |yy: &[f32; 3]| Self::ode(yy, rn, re, a, b, crr, k);
        let k1 = f(y);
        let y2: [f32; 3] = [
            y[0] + dt * 0.5 * k1[0],
            y[1] + dt * 0.5 * k1[1],
            y[2] + dt * 0.5 * k1[2],
        ];
        let k2 = f(&y2);
        let y3: [f32; 3] = [
            y[0] + dt * 0.5 * k2[0],
            y[1] + dt * 0.5 * k2[1],
            y[2] + dt * 0.5 * k2[2],
        ];
        let k3 = f(&y3);
        let y4: [f32; 3] = [y[0] + dt * k3[0], y[1] + dt * k3[1], y[2] + dt * k3[2]];
        let k4 = f(&y4);
        let mut r = [0.0; 3];
        for i in 0..3 {
            r[i] = y[i] + dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
        r
    }

    pub fn od_tick(
        sig: &mut ParadigmSignals,
        nn: f32,
        ne: f32,
        crr: f32,
        _hist: &[(f32, f32, f32)],
    ) -> String {
        let (k, rn, re, a, b) = ((nn * 3.0).max(10.0), 0.05, 0.03, 0.1, 0.8);
        let mut y = [nn, ne, crr];
        let mut dt = 0.1;
        for _ in 0..30 {
            let y1 = Self::rk4(&y, dt, rn, re, a, b, crr, k);
            let yh = Self::rk4(&y, dt * 0.5, rn, re, a, b, crr, k);
            let yh2 = Self::rk4(&yh, dt * 0.5, rn, re, a, b, crr, k);
            let err =
                (y1[0] - yh2[0]).abs() + (y1[1] - yh2[1]).abs() * 0.01 + (y1[2] - yh2[2]).abs();
            if err > 1e-4 {
                dt = (dt * 0.5).max(0.02);
            } else if err < 1e-6 {
                dt = (dt * 2.0).min(0.8);
            }
            y = y1;
            y[0] = y[0].max(1.0);
            y[1] = y[1].max(0.0);
        }
        sig.oracle_examples.push((
            vec![y[0] / k, y[1] / (k * 8.0), y[2]],
            sigm(-(y[1] - ne) / (ne.max(1.0) * 30.0) + 1.0),
        ));
        sig.activations.push("neuralode".into());
        format!("[NEURALODE] n={:.0} e={:.0} dt={:.3}", y[0], y[1], dt)
    }
}

// ═══ HYPERNET: Generate model weights from graph state ═══
pub struct HyperNet;
impl HyperNet {
    pub fn hypernet_tick(
        sig: &mut ParadigmSignals,
        gs: &[f32; 10],
        es_acc: f32,
        cp_y: f32,
    ) -> String {
        let mut l1 = Linear::new(10, 16);
        let mut l2 = Linear::new(16, 4);
        let mut hes = Linear::new(4, 3);
        let mut hcp = Linear::new(4, 3);
        let lr = 0.01;
        for _ in 0..5 {
            let h1 = l1.forward(gs);
            let h2 = l2.forward(&h1);
            let esp = hes.forward(&h2);
            let cpp = hcp.forward(&h2);
            hes.backward(&h2, &esp, &[es_acc, es_acc * 0.5, 1.0 - es_acc * 0.3], lr);
            hcp.backward(&h2, &cpp, &[cp_y, cp_y * 0.7, 1.0 - cp_y], lr);
        }
        let h1 = l1.forward(gs);
        let h2 = l2.forward(&h1);
        let esw = hes.forward(&h2);
        let cpw = hcp.forward(&h2);
        sig.model_updates.insert("edge_scorer".into(), esw.clone());
        sig.model_updates.insert("crawl_picker".into(), cpw.clone());
        sig.activations.push("hypernet".into());
        format!(
            "[HYPERNET] ES={:.2}/{:.2}/{:.2} CP={:.2}/{:.2}/{:.2}",
            esw[0], esw[1], esw[2], cpw[0], cpw[1], cpw[2]
        )
    }
}

// ═══ SIAMESE: Contrastive learning on embeddings ═══
pub struct Siamese;
impl Siamese {
    fn init_embs(names: &[String], dim: usize) -> Vec<Var> {
        names
            .iter()
            .map(|nm| {
                let mut v = vec![0.0f32; dim];
                for (k, b) in nm.bytes().enumerate() {
                    v[k % dim] += (b as f32 * 0.01).sin();
                }
                let nr = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
                v.iter_mut().for_each(|x| *x /= nr);
                Var::new(&v)
            })
            .collect()
    }

    pub fn siamese_tick(
        sig: &mut ParadigmSignals,
        nn: usize,
        names: &[String],
        pos: &[(usize, usize)],
        neg: &[(usize, usize)],
        econf: &[(usize, usize, f32)],
    ) -> String {
        let dim = 8;
        let n = nn.min(50);
        let margin = 0.2;
        let lr = 0.05;
        let mut embs = Self::init_embs(&names[..n.min(names.len())], dim);
        let mut a2n: HashMap<usize, Vec<usize>> = HashMap::new();
        for &(a, nn_idx) in neg {
            if a < n && nn_idx < n {
                a2n.entry(a).or_default().push(nn_idx);
            }
        }
        let mut loss = 0.0f32;
        let mut trained = 0;
        for &(a, p) in pos {
            if a >= n || p >= n {
                continue;
            }
            if let Some(negs) = a2n.get(&a) {
                if negs.is_empty() {
                    continue;
                }
                let npool: Vec<Vec<f32>> =
                    negs.iter().map(|&nidx| embs[nidx].data.clone()).collect();
                let (mut hni, mut bs) = (0, f32::NEG_INFINITY);
                for (i, nv) in npool.iter().enumerate() {
                    let s = csim(&embs[a].data, nv);
                    if s > bs {
                        bs = s;
                        hni = i;
                    }
                }
                let hn = negs[hni];
                let (ad, pd, nd) = (
                    embs[a].data.clone(),
                    embs[p].data.clone(),
                    embs[hn].data.clone(),
                );
                let (dap, dan) = (l2(&ad, &pd), l2(&ad, &nd));
                let tri = (dap - dan + margin).max(0.0);
                loss += tri;
                trained += 1;
                if tri > 0.0 {
                    for i in 0..dim {
                        let ga = if dap > 1e-8 {
                            (ad[i] - pd[i]) / dap
                        } else {
                            0.0
                        } - if dan > 1e-8 {
                            (ad[i] - nd[i]) / dan
                        } else {
                            0.0
                        };
                        let gp = if dap > 1e-8 {
                            -(ad[i] - pd[i]) / dap
                        } else {
                            0.0
                        };
                        let gn = if dan > 1e-8 {
                            (ad[i] - nd[i]) / dan
                        } else {
                            0.0
                        };
                        embs[a].data[i] -= lr * ga;
                        embs[p].data[i] -= lr * gp;
                        embs[hn].data[i] -= lr * gn;
                    }
                }
            }
        }
        for &(a, b, _) in econf {
            if a < n && b < n {
                let d = l2(&embs[a].data, &embs[b].data);
                sig.edge_trust.insert((a, b), sigm((1.0 - d) * 4.0));
            }
        }
        for &(a, b, ac) in econf.iter().take(20) {
            if a < n && b < n {
                let d = l2(&embs[a].data, &embs[b].data);
                sig.edge_scorer_examples
                    .push((vec![d, sigm((1.0 - d) * 4.0), ac], ac));
            }
        }
        let al = if trained > 0 {
            loss / trained as f32
        } else {
            0.0
        };
        sig.activations.push("siamese".into());
        format!("[SIAMESE] pairs={} loss={:.4}", trained, al)
    }
}

// ═══ MULTIMODAL: 3-stream fusion ═══
pub struct Multimodal;
impl Multimodal {
    fn stream_emb(items: &[Vec<f32>], d_in: usize, d_out: usize) -> Vec<Vec<f32>> {
        let mut l = Linear::new(d_in, d_out);
        items.iter().map(|v| l.forward(v)).collect()
    }
    fn cross_attn(q: &[f32], k1: &[f32], k2: &[f32], v1: &[f32], v2: &[f32]) -> Vec<f32> {
        let s1 = sigm(csim(q, k1) * 2.0);
        let s2 = sigm(csim(q, k2) * 2.0);
        let sum = (s1 + s2).max(1e-8);
        q.iter()
            .enumerate()
            .map(|(i, &x)| x * 0.5 + v1[i] * s1 / sum * 0.25 + v2[i] * s2 / sum * 0.25)
            .collect()
    }

    pub fn multimodal_tick(
        sig: &mut ParadigmSignals,
        nn: usize,
        names: &[String],
        degs: &[usize],
        posts: &[(String, f32, f32, f32)],
        econf: &[(usize, usize, f32)],
    ) -> String {
        let n = nn.min(50);
        let di = 8;
        let ds = 4;
        let dout = 16;
        let max_d = *degs.iter().max().unwrap_or(&1).max(&1) as f32;
        let t_raw: Vec<Vec<f32>> = names[..n]
            .iter()
            .map(|nm| {
                let mut v = vec![0.0f32; di];
                for (k, b) in nm.bytes().enumerate() {
                    v[k % di] += b as f32 * 0.02;
                }
                let nr = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
                v.iter_mut().for_each(|x| *x /= nr);
                v
            })
            .collect();
        let text_embs = Self::stream_emb(&t_raw, di, dout);
        let s_raw: Vec<Vec<f32>> = degs[..n]
            .iter()
            .map(|&d| {
                vec![
                    d as f32 / max_d,
                    (d as f32).sqrt() / max_d.sqrt(),
                    (d as f32).ln_1p() / max_d.ln_1p(),
                    if d > 2 { 1.0 } else { 0.0 },
                ]
            })
            .collect();
        let struct_embs = Self::stream_emb(&s_raw, ds, dout);
        let mut c_raw: Vec<Vec<f32>> = vec![vec![0.5, 0.5, 0.5, 0.5]; n];
        for (i, post) in posts.iter().enumerate().take(n) {
            c_raw[i] = vec![post.1, post.2, post.3, (post.1 + post.2 + post.3) / 3.0];
        }
        let conf_embs = Self::stream_emb(&c_raw, ds, dout);
        let (mut at, mut as_, mut ac) = (vec![], vec![], vec![]);
        for i in 0..n {
            at.push(Self::cross_attn(
                &text_embs[i],
                &struct_embs[i],
                &conf_embs[i],
                &struct_embs[i],
                &conf_embs[i],
            ));
            as_.push(Self::cross_attn(
                &struct_embs[i],
                &text_embs[i],
                &conf_embs[i],
                &text_embs[i],
                &conf_embs[i],
            ));
            ac.push(Self::cross_attn(
                &conf_embs[i],
                &text_embs[i],
                &struct_embs[i],
                &text_embs[i],
                &struct_embs[i],
            ));
        }
        let mut wl = [0.0f32; 3];
        let flr = 0.01;
        for _ in 0..10 {
            if let Some(&(a, b, actual)) = econf.first() {
                if a < n && b < n {
                    let al = softm(&wl);
                    let fa: Vec<f32> = (0..dout)
                        .map(|i| at[a][i] * al[0] + as_[a][i] * al[1] + ac[a][i] * al[2])
                        .collect();
                    let fb: Vec<f32> = (0..dout)
                        .map(|i| at[b][i] * al[0] + as_[b][i] * al[1] + ac[b][i] * al[2])
                        .collect();
                    let pred = (csim(&fa, &fb) + 1.0) * 0.5;
                    let err = actual - pred;
                    for q in 0..3 {
                        wl[q] += flr * err * al[q] * (1.0 - al[q]);
                    }
                }
            }
        }
        let alpha = softm(&wl);
        let fused: Vec<Vec<f32>> = (0..n)
            .map(|i| {
                (0..dout)
                    .map(|d| at[i][d] * alpha[0] + as_[i][d] * alpha[1] + ac[i][d] * alpha[2])
                    .collect()
            })
            .collect();
        for &(a, b, _) in econf {
            if a < n && b < n {
                let c = (csim(&fused[a], &fused[b]) + 1.0) * 0.5;
                sig.edge_trust.insert((a, b), c);
            }
        }
        for i in 0..n.min(10) {
            let tv = text_embs[i].iter().map(|x| x * x).sum::<f32>() / dout as f32;
            let sv = struct_embs[i].iter().map(|x| x * x).sum::<f32>() / dout as f32;
            let cv = conf_embs[i].iter().map(|x| x * x).sum::<f32>() / dout as f32;
            let fv = fused[i].iter().map(|x| x * x).sum::<f32>() / dout as f32;
            sig.emotion_examples.push((
                vec![
                    tv,
                    sv,
                    cv,
                    1.0 - alpha.iter().map(|&a| a * a).sum::<f32>(),
                    fv,
                ],
                vec![
                    sigm(tv - sv),
                    sigm(cv * 3.0 + fv - 1.5),
                    sigm(alpha[2] * 3.0 - 1.5),
                ],
            ));
        }
        sig.activations.push("multimodal".into());
        format!(
            "[MULTIMODAL] α={:.2}/{:.2}/{:.2}",
            alpha[0], alpha[1], alpha[2]
        )
    }
}
