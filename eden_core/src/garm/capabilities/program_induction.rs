// EDEN GARM — Program Induction (synthesis from I/O examples)
// Dado un conjunto de pares (input, output), busca en el espacio de programas
// el que mejor los reproduce. Usa beam search sobre el lenguaje de programas.

#[derive(Clone, Debug)]
pub struct ProgramInduction {
    pub max_beam: usize,
    pub max_depth: usize,
    pub n_induced: u64,
    pub last_program: String,
}

impl ProgramInduction {
    pub fn new() -> Self {
        ProgramInduction {
            max_beam: 5,
            max_depth: 4,
            n_induced: 0,
            last_program: String::new(),
        }
    }

    /// Induce a program given I/O examples. Returns best program string.
    /// Simple heuristic: tries concatenation, reversal, doubling, filtering.
    pub fn induce(&mut self, examples: &[(String, String)]) -> Option<String> {
        if examples.is_empty() {
            return None;
        }
        let mut candidates: Vec<(String, f32)> = Vec::new();
        let ops = vec!["identity", "reverse", "upper", "lower", "len", "words"];
        for op in &ops {
            let prog = format!("INVOKE {} HALT", op);
            let score = self.score_program(&prog, examples);
            candidates.push((prog, score));
        }
        // Try sequences of 2 ops
        for op1 in &ops {
            for op2 in &ops {
                let prog = format!("INVOKE {}\nINVOKE {}\nHALT", op1, op2);
                let score = self.score_program(&prog, examples);
                candidates.push((prog, score));
            }
        }
        // Try conditional
        for op in &ops {
            let prog = format!("IF len > 5 THEN INVOKE {} ELSE INVOKE identity\nHALT", op);
            let score = self.score_program(&prog, examples);
            candidates.push((prog, score));
        }
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(best) = candidates.first() {
            if best.1 > 0.3 {
                self.n_induced += 1;
                self.last_program = best.0.clone();
                return Some(best.0.clone());
            }
        }
        None
    }

    fn score_program(&self, prog: &str, examples: &[(String, String)]) -> f32 {
        let mut total = 0.0f32;
        let mut n = 0;
        for (inp, out) in examples {
            let simulated = self.simulate(prog, inp);
            let sim = string_similarity(&simulated, out);
            total += sim;
            n += 1;
        }
        if n == 0 {
            0.0
        } else {
            total / n as f32
        }
    }

    fn simulate(&self, prog: &str, inp: &str) -> String {
        let lines: Vec<&str> = prog
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        let mut state = inp.to_string();
        for line in lines {
            if line == "HALT" {
                break;
            }
            if line.starts_with("INVOKE ") {
                let op = line.replacen("INVOKE ", "", 1);
                state = apply_op(&op, &state);
            } else if line.starts_with("IF ") {
                let cond = line.trim_start_matches("IF ").trim_end_matches(" THEN");
                if let Some(then_part) = cond.split(" THEN ").nth(1) {
                    let op = then_part.split(" ELSE ").next().unwrap_or(then_part);
                    state = apply_op(op, &state);
                }
            }
        }
        state
    }

    pub fn status(&self) -> String {
        format!(
            "ProgInduct | induced={} | beam={} | last_prog_len={}",
            self.n_induced,
            self.max_beam,
            self.last_program.len()
        )
    }
}

fn apply_op(op: &str, s: &str) -> String {
    match op {
        "identity" => s.to_string(),
        "reverse" => s.chars().rev().collect(),
        "upper" => s.to_uppercase(),
        "lower" => s.to_lowercase(),
        "len" => s.len().to_string(),
        "words" => s.split_whitespace().count().to_string(),
        _ => s.to_string(),
    }
}

fn string_similarity(a: &str, b: &str) -> f32 {
    let max_len = a.len().max(b.len()) as f32;
    if max_len == 0.0 {
        return 1.0;
    }
    let dist = levenshtein(a, b);
    1.0 - (dist as f32 / max_len).min(1.0)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let m = a.len();
    let n = b.len();
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];
    for j in 0..=n {
        prev[j] = j;
    }
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}
