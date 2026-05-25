// paradigms/reasoning.rs — Causal · Bayesian · Logic · NeuroSymbolic · ProgramSynthesis
use super::ParadigmSignals;
use std::collections::{HashMap, HashSet};

pub struct Causal;
pub struct Bayesian;
pub struct Logic;
pub struct NeuroSymbolic;
pub struct ProgramSynthesis;

pub const ISA: u8 = 0;
pub const CAUSES: u8 = 1;
pub const PART_OF: u8 = 3;
pub const OPPOSES: u8 = 4;

fn fast_rand() -> f32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static S: AtomicU32 = AtomicU32::new(12345);
    let x = S
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
            let mut z = v;
            z ^= z << 13;
            z ^= z >> 17;
            z ^= z << 5;
            Some(z)
        })
        .unwrap_or(12345);
    x as f32 / u32::MAX as f32
}
fn gamma_ln(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let pi = std::f32::consts::PI;
    if x < 0.5 {
        return (pi / ((pi * x).sin())).ln() - gamma_ln(1.0 - x);
    }
    let c = [
        76.18009172947146,
        -86.50532032941677,
        24.01409824083091,
        -1.231739572450155,
        0.1208650973866179e-2,
        -0.5395239384953e-5,
    ];
    let z = x - 1.0;
    let mut ser = 1.000000000190015;
    for i in 0..6 {
        ser += c[i] / (z + (i + 1) as f32);
    }
    let mut tmp = z + 5.5;
    tmp -= (z + 0.5) * tmp.ln();
    (2.5066282746310005_f32 * ser / z).ln() - tmp
}
fn l2(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum()
}
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let (mut d, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for i in 0..a.len() {
        d += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na * nb > 1e-8 {
        d / (na.sqrt() * nb.sqrt())
    } else {
        0.0
    }
}

// ═══ CAUSAL ═══
impl Causal {
    pub fn conditional_prob(cause: u32, effect: u32, adj: &[Vec<(u32, u8, f32)>]) -> f32 {
        let out = match adj.get(cause as usize) {
            Some(v) => v,
            None => return 0.0,
        };
        out.iter()
            .filter(|(t, r, _)| *t == effect && *r == CAUSES)
            .count() as f32
            / out.len().max(1) as f32
    }

    pub fn backdoor_set(cause: u32, effect: u32, adj: &[Vec<(u32, u8, f32)>]) -> Vec<u32> {
        let n = adj.len();
        let mut inc: Vec<Vec<u32>> = vec![vec![]; n];
        for s in 0..n {
            for (t, _, _) in &adj[s] {
                if (*t as usize) < n {
                    inc[*t as usize].push(s as u32);
                }
            }
        }
        fn ancestors(node: u32, inc: &[Vec<u32>], n: usize) -> HashSet<u32> {
            let mut a = HashSet::new();
            let mut vis = vec![false; n];
            let mut st = vec![node];
            vis[node as usize] = true;
            while let Some(x) = st.pop() {
                for &p in &inc[x as usize] {
                    if !vis[p as usize] {
                        vis[p as usize] = true;
                        a.insert(p);
                        st.push(p);
                    }
                }
            }
            a
        }
        let ax = ancestors(cause, &inc, n);
        let ay = ancestors(effect, &inc, n);
        ax.intersection(&ay).cloned().collect()
    }

    pub fn intervene(
        cause: u32,
        effect: u32,
        adj: &[Vec<(u32, u8, f32)>],
        _in: &[Vec<u32>],
        _nc: usize,
    ) -> f32 {
        let zs = Self::backdoor_set(cause, effect, adj);
        if zs.is_empty() {
            return Self::conditional_prob(cause, effect, adj);
        }
        let direct = Self::conditional_prob(cause, effect, adj);
        let total_e = adj.iter().flat_map(|v| v.iter()).count().max(1) as f32;
        let (mut sp, mut ws) = (0.0f32, 0.0f32);
        for &z in &zs {
            let zo = match adj.get(z as usize) {
                Some(v) => v,
                None => continue,
            };
            let zd = zo.len().max(1) as f32;
            let ps =
                (zo.iter().filter(|(t, _, _)| *t == cause).count().max(1) as f32 / zd).min(1.0);
            let w = (zd / total_e) / ps.max(0.01);
            let zy = zo.iter().filter(|(t, _, _)| *t == effect).count() as f32 / zd;
            sp += (direct * (1.0 - zy * 0.5) + zy * 0.5) * w;
            ws += w;
        }
        if ws > 0.0 {
            (sp / ws).min(1.0)
        } else {
            direct
        }
    }

    pub fn ate(adj: &[Vec<(u32, u8, f32)>]) -> f32 {
        let (mut tc, mut ts) = (0.0f32, 0u32);
        let (mut cc, mut cs) = (0.0f32, 0u32);
        for o in adj {
            for (_, r, c) in o {
                if *r == CAUSES {
                    tc += c;
                    ts += 1;
                } else {
                    cc += c;
                    cs += 1;
                }
            }
        }
        if ts == 0 || cs == 0 {
            0.0
        } else {
            (tc / ts as f32 - cc / cs as f32).abs()
        }
    }

    pub fn run(adj: &[Vec<(u32, u8, f32)>], signals: &mut ParadigmSignals) {
        let n = adj.len();
        if n < 2 {
            return;
        }
        let ate = Self::ate(adj);
        for (s, o) in adj.iter().enumerate() {
            for (t, r, c) in o {
                if *r == CAUSES && *c >= 0.3 {
                    signals
                        .edge_trust
                        .insert((s, *t as usize), (*c * (1.0 + ate * 0.3)).min(1.0));
                } else if *c >= 0.7 {
                    signals.edge_trust.insert((s, *t as usize), *c);
                }
            }
        }
        let total = adj.iter().flat_map(|v| v.iter()).count() as f32;
        signals.oracle_examples.push((
            vec![ate, n as f32 * 0.01, total * 0.001],
            if ate > 0.3 {
                0.85
            } else if ate > 0.1 {
                0.55
            } else {
                0.25
            },
        ));
    }
}

// ═══ BAYESIAN ═══
impl Bayesian {
    pub fn source_posterior(alpha: f32, beta: f32, corr: u32, total: u32) -> (f32, f32, f32, f32) {
        let a = alpha + corr as f32;
        let b = beta + (total.saturating_sub(corr)) as f32;
        (
            a / (a + b),
            (a * b) / ((a + b).powi(2) * (a + b + 1.0)),
            a,
            b,
        )
    }

    pub fn bayes_factor(hits: u32, misses: u32) -> f32 {
        let n = hits + misses;
        if n == 0 {
            return 1.0;
        }
        let (k, nk) = (hits as f32, (n - hits) as f32);
        let lb = |a: f32, b: f32| gamma_ln(a) + gamma_ln(b) - gamma_ln(a + b);
        (lb(k + 2.0, nk + 1.0) - lb(2.0, 1.0) - (lb(k + 1.0, nk + 1.0) - lb(1.0, 1.0))).exp()
    }

    pub fn update_all_sources(
        ss: &HashMap<String, (f32, u32, u32)>,
    ) -> Vec<(String, f32, f32, f32)> {
        ss.iter()
            .map(|(n, (_, h, m))| {
                let (mean, _, a, b) = Self::source_posterior(1.0, 1.0, *h, *h + *m);
                (n.clone(), mean, a, b)
            })
            .collect()
    }

    pub fn run(ss: &HashMap<String, (f32, u32, u32)>, signals: &mut ParadigmSignals) {
        let up = Self::update_all_sources(ss);
        let (mut st, mut ns) = (0.0f32, 0usize);
        for (n, mean, _, _) in &up {
            signals.source_scores.insert(n.clone(), *mean);
            let bf = Self::bayes_factor(
                ss.get(n).map(|(_, h, _)| *h).unwrap_or(0),
                ss.get(n).map(|(_, _, m)| *m).unwrap_or(0),
            );
            signals.source_bf.insert(n.clone(), bf);
            st += mean;
            ns += 1;
        }
        if ns > 0 {
            st /= ns as f32;
            signals.prune_threshold = (1.0 - st).max(0.05).min(0.95);
        }
    }
}

// ═══ LOGIC ═══
impl Logic {
    pub fn resolve_transitive(
        adj: &[Vec<(u32, u8, f32)>],
        node_count: usize,
    ) -> Vec<(u32, u32, u8, f32)> {
        let mut inf = Vec::new();
        let mut ex: HashSet<(u32, u32, u8)> = HashSet::new();
        for (sid, o) in adj.iter().enumerate() {
            for (tid, r, _) in o {
                ex.insert((sid as u32, *tid, *r));
            }
        }
        for &rel in &[ISA, PART_OF, CAUSES] {
            for a in 0..node_count as u32 {
                let ao = match adj.get(a as usize) {
                    Some(v) => v,
                    None => continue,
                };
                for (b, _r_ab, c_ab) in ao.iter().filter(|(_, r, _)| *r == rel) {
                    let bo = match adj.get(*b as usize) {
                        Some(v) => v,
                        None => continue,
                    };
                    for (c, _r_bc, c_bc) in bo.iter().filter(|(_, r, _)| *r == rel) {
                        if a == *c || ex.contains(&(a, *c, rel)) {
                            continue;
                        }
                        inf.push((a, *c, rel, c_ab.min(*c_bc)));
                        ex.insert((a, *c, rel));
                    }
                }
            }
        }
        inf
    }

    pub fn detect_contradictions(adj: &[Vec<(u32, u8, f32)>]) -> Vec<(u32, u32, String)> {
        let mut out = Vec::new();
        for sid in 0..adj.len() {
            let mut bt: HashMap<u32, Vec<u8>> = HashMap::new();
            for (t, r, _) in &adj[sid] {
                bt.entry(*t).or_default().push(*r);
            }
            for (t, rels) in &bt {
                let pos = rels.iter().any(|r| *r == ISA || *r == PART_OF);
                let neg = rels.contains(&OPPOSES);
                if pos && neg {
                    out.push((sid as u32, *t, format!("contradiction: {sid}->{t}")));
                }
            }
        }
        out
    }

    pub fn run(adj: &[Vec<(u32, u8, f32)>], signals: &mut ParadigmSignals) {
        for (s, t, _, c) in &Self::resolve_transitive(adj, adj.len()) {
            signals.novel_edges.push((*s as usize, *t as usize, *c));
        }
    }
}

// ═══ NEUROSYMBOLIC ═══
impl NeuroSymbolic {
    fn kpp_init(em: &[Vec<f32>], k: usize) -> Vec<Vec<f32>> {
        if em.is_empty() {
            return vec![];
        }
        let k = k.min(em.len());
        let mut ct = vec![em[0].clone()];
        let mut md = vec![f32::MAX; em.len()];
        for idx in 1..k {
            for (i, e) in em.iter().enumerate() {
                let d = l2(e, &ct[idx - 1]);
                if d < md[i] {
                    md[i] = d;
                }
            }
            let tot: f32 = md.iter().map(|d| d.powi(2)).sum();
            if tot <= 1e-10 {
                for (_i, e) in em.iter().enumerate() {
                    if !ct.iter().any(|c| l2(c, e) < 1e-6) {
                        ct.push(e.clone());
                        break;
                    }
                }
                continue;
            }
            let tgt = fast_rand() * tot;
            let mut cum = 0.0;
            for (i, d) in md.iter().enumerate() {
                cum += d.powi(2);
                if cum >= tgt {
                    ct.push(em[i].clone());
                    break;
                }
            }
            if ct.len() <= idx {
                ct.push(em[0].clone());
            }
        }
        ct
    }

    pub fn cluster_embeddings(em: &[Vec<f32>], k: usize, mi: usize) -> (Vec<usize>, Vec<Vec<f32>>) {
        if em.is_empty() || k == 0 {
            return (vec![], vec![]);
        }
        let dim = em[0].len();
        let mut ct = Self::kpp_init(em, k);
        let mut lb = vec![0usize; em.len()];
        for _ in 0..mi {
            let mut ch = false;
            for (i, e) in em.iter().enumerate() {
                let (mut bst, mut bd) = (0, f32::MAX);
                for (ci, c) in ct.iter().enumerate() {
                    let d = l2(e, c);
                    if d < bd {
                        bd = d;
                        bst = ci;
                    }
                }
                if lb[i] != bst {
                    ch = true;
                    lb[i] = bst;
                }
            }
            if !ch {
                break;
            }
            let mut cnt = vec![0usize; k];
            let mut sm = vec![vec![0.0f32; dim]; k];
            for (i, e) in em.iter().enumerate() {
                let cl = lb[i];
                cnt[cl] += 1;
                for d in 0..dim {
                    sm[cl][d] += e[d];
                }
            }
            for c in 0..k {
                if cnt[c] > 0 {
                    for d in 0..dim {
                        ct[c][d] = sm[c][d] / cnt[c] as f32;
                    }
                }
            }
        }
        (lb, ct)
    }

    pub fn extract_rules(
        adj: &[Vec<(u32, u8, f32)>],
        _nn: &[String],
        ca: &[usize],
        k: usize,
    ) -> Vec<String> {
        let rn = [
            "IsA",
            "Causes",
            "HasProperty",
            "PartOf",
            "Opposes",
            "Unknown",
        ];
        let mut rules = Vec::new();
        for c in 0..k {
            let mem: HashSet<usize> = ca
                .iter()
                .enumerate()
                .filter(|(_, &cl)| cl == c)
                .map(|(i, _)| i)
                .collect();
            if mem.len() < 2 {
                rules.push(format!("C{c}: <2 nodes"));
                continue;
            }
            let mut tc = [0usize; 6];
            let (mut tf, mut ne) = (0.0f32, 0usize);
            for &sid in &mem {
                if sid >= adj.len() {
                    continue;
                }
                for (tid, r, conf) in &adj[sid] {
                    if mem.contains(&(*tid as usize)) {
                        tc[*r as usize] += 1;
                        tf += conf;
                        ne += 1;
                    }
                }
            }
            let dom = (0..6).max_by_key(|&i| tc[i]).unwrap_or(5);
            let mu = if ne > 0 { tf / ne as f32 } else { 0.0 };
            rules.push(format!("C{c} [{}n]: {} mu={:.2}", mem.len(), rn[dom], mu));
        }
        rules
    }

    pub fn run(
        em: &[Vec<f32>],
        adj: &[Vec<(u32, u8, f32)>],
        nn: &[String],
        k: usize,
        sig: &mut ParadigmSignals,
    ) {
        if em.len() < 3 {
            return;
        }
        let k = k.min(em.len());
        let (lb, _ct) = Self::cluster_embeddings(em, k, 20);
        sig.inferred_rules = Self::extract_rules(adj, nn, &lb, k);
        for c in 0..k {
            let cnt = lb.iter().filter(|&&l| l == c).count();
            sig.crawl_recommendations
                .push((format!("cluster_{c}"), 1.0 / (1.0 + cnt as f32)));
        }
    }
}

// ═══ PROGRAM SYNTHESIS ═══
impl ProgramSynthesis {
    pub fn analyze_sources(ss: &HashMap<String, (f32, u32, u32)>) -> Vec<(String, f32)> {
        if ss.is_empty() {
            return vec![];
        }
        let cr = ss
            .values()
            .map(|(_, h, m)| {
                if *h + *m > 0 {
                    *h as f32 / (*h + *m) as f32
                } else {
                    0.0
                }
            })
            .sum::<f32>()
            / ss.len() as f32;
        let mut sc: Vec<_> = ss
            .iter()
            .map(|(n, (_, h, m))| {
                let t = *h + *m;
                let hr = if t > 0 { *h as f32 / t as f32 } else { 0.0 };
                (n.clone(), hr * (1.0 + t as f32).ln() * cr.max(0.1))
            })
            .collect();
        sc.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sc
    }

    pub fn synthesize_templates(ss: &HashMap<String, (f32, u32, u32)>) -> Vec<(String, f32)> {
        let ls = ["en", "es", "fr", "de"];
        let cs = [
            "Natural_language_processing",
            "Knowledge_representation",
            "Semantic_web",
            "Ontology_(information_science)",
            "Linked_data",
            "Artificial_intelligence",
            "Cognitive_science",
            "Language_acquisition",
        ];
        let ns = ss.len().max(1);
        let mut tm = Vec::new();
        for &l in &ls {
            for &c in &cs {
                let url = format!("https://{l}.wikipedia.org/wiki/{c}");
                let (mut sc, mut cnt) = (0.0f32, 0u32);
                for (sn, (_, h, m)) in ss {
                    let t = *h + *m;
                    if sn.contains(l) || sn.to_lowercase().contains(&c.to_lowercase()) {
                        if t > 0 {
                            sc += *h as f32 / t as f32;
                            cnt += 1;
                        }
                    }
                }
                let avg = if cnt > 0 { sc / cnt as f32 } else { 0.05 };
                tm.push((url, avg + 1.0 / ns as f32 * 0.3));
            }
        }
        tm.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        tm.truncate(40);
        tm
    }

    pub fn run(
        ss: &HashMap<String, (f32, u32, u32)>,
        _adj: &[Vec<(u32, u8, f32)>],
        _nn: &[String],
        sig: &mut ParadigmSignals,
    ) {
        let tm = Self::synthesize_templates(ss);
        let mut st: Vec<(String, f32)> = tm
            .into_iter()
            .map(|(url, base)| {
                let dim = 16usize;
                let mut te = vec![0.0f32; dim];
                for (k, b) in url.bytes().enumerate() {
                    te[k % dim] += (b as f32) * 0.002;
                }
                (url, base + cosine(&te, &vec![0.0; dim]) * 0.3)
            })
            .collect();
        st.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sig.synthesized_templates = st.iter().take(5).cloned().collect();
        for (url, sc) in st.iter().take(3) {
            sig.crawl_recommendations.push((url.clone(), *sc));
        }
    }
}
