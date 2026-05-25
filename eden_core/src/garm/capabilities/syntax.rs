// EDEN GARM Syntax — Heuristic dependency parser
// Extracts structural relations and causal templates from sentences.
// No LLM, no transformer. Rules + word embeddings + observed statistics.

use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub enum Dependency {
    Root,
    Nsubj, // nominal subject
    Dobj,  // direct object
    Iobj,  // indirect object
    Nmod,  // nominal modifier
    Advcl, // adverbial clause (causal, conditional, temporal)
    Xcomp, // clausal complement
    Cop,   // copula
    Aux,   // auxiliary
    Punct,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct DependencyParse {
    pub tokens: Vec<String>,
    pub heads: Vec<usize>,
    pub deps: Vec<Dependency>,
    pub root: usize,
}

pub struct SyntaxParser {
    pub verbs: HashSet<String>,
    pub auxiliaries: HashSet<String>,
    pub copulas: HashSet<String>,
    pub prepositions: HashSet<String>,
}

impl SyntaxParser {
    pub fn new() -> Self {
        let mut verbs = HashSet::new();
        for v in &[
            "come", "es", "tiene", "hace", "va", "da", "quiere", "puede", "debe", "ser", "estar",
            "run", "eat", "has", "make", "go", "give", "want", "can", "must", "is", "are", "do",
            "did", "saw", "took", "put", "told", "asked", "be", "been", "being", "have", "had",
            "will", "would", "should", "may", "might", "shall", "could", "did", "does",
        ] {
            verbs.insert(v.to_string());
        }
        let mut auxiliaries = HashSet::new();
        for a in &[
            "ha", "he", "habia", "have", "has", "had", "will", "would", "shall", "should", "may",
            "might", "can", "could", "must", "do", "does", "did", "be", "been", "being",
        ] {
            auxiliaries.insert(a.to_string());
        }
        let mut copulas = HashSet::new();
        for c in &[
            "es", "son", "estar", "estoy", "estas", "esta", "estamos", "estan", "is", "are", "was",
            "were", "been", "be", "being", "am",
        ] {
            copulas.insert(c.to_string());
        }
        let mut prepositions = HashSet::new();
        for p in &[
            "de", "del", "al", "en", "con", "por", "para", "sin", "sobre", "entre", "hasta",
            "desde", "durante", "antes", "despues", "of", "in", "for", "on", "with", "at", "by",
            "from", "into", "through", "during", "before", "after", "above", "below", "between",
            "under",
        ] {
            prepositions.insert(p.to_string());
        }
        SyntaxParser {
            verbs,
            auxiliaries,
            copulas,
            prepositions,
        }
    }

    fn is_verb(&self, w: &str) -> bool {
        self.verbs.contains(&w.to_lowercase())
            || w.ends_with("ar")
            || w.ends_with("er")
            || w.ends_with("ir")
    }

    fn is_aux(&self, w: &str) -> bool {
        self.auxiliaries.contains(&w.to_lowercase())
    }
    fn is_cop(&self, w: &str) -> bool {
        self.copulas.contains(&w.to_lowercase())
    }
    fn is_prep(&self, w: &str) -> bool {
        self.prepositions.contains(&w.to_lowercase())
    }
    fn is_punct(&self, w: &str) -> bool {
        w == "," || w == "." || w == "!" || w == "?" || w == ";" || w == ":"
    }

    pub fn parse(&self, tokens: &[String]) -> DependencyParse {
        let n = tokens.len();
        let mut heads = vec![0usize; n];
        let mut deps = vec![Dependency::Unknown; n];
        let mut root = 0usize;

        // 1. Find root (first main verb, skip aux/cop at start)
        for (i, t) in tokens.iter().enumerate() {
            let w = t.to_lowercase();
            if self.is_verb(&w) && !self.is_aux(&w) && !self.is_cop(&w) {
                root = i;
                heads[i] = i; // root is its own head
                deps[i] = Dependency::Root;
                break;
            }
        }

        // 2. If no main verb found, try copula or auxiliary
        if deps[root] != Dependency::Root {
            for (i, t) in tokens.iter().enumerate() {
                let w = t.to_lowercase();
                if self.is_cop(&w) || self.is_verb(&w) {
                    root = i;
                    heads[i] = i;
                    deps[i] = Dependency::Root;
                    break;
                }
            }
        }

        // 3. Attach everything before root as nsubj (heuristic)
        for i in 0..root {
            if self.is_punct(&tokens[i]) {
                deps[i] = Dependency::Punct;
                heads[i] = root;
                continue;
            }
            if self.is_prep(&tokens[i]) {
                deps[i] = Dependency::Nmod;
                heads[i] = root;
                continue;
            }
            heads[i] = root;
            deps[i] = Dependency::Nsubj;
        }

        // 4. Attach everything after root as dobj / nmod
        let mut last_noun = root;
        for i in (root + 1)..n {
            if self.is_punct(&tokens[i]) {
                deps[i] = Dependency::Punct;
                heads[i] = root;
                continue;
            }
            if self.is_verb(&tokens[i].to_lowercase()) && !self.is_aux(&tokens[i].to_lowercase()) {
                // Secondary verb -> xcomp or advcl
                heads[i] = root;
                deps[i] = Dependency::Xcomp;
                continue;
            }
            if self.is_prep(&tokens[i]) {
                deps[i] = Dependency::Nmod;
                heads[i] = root;
                last_noun = i;
                continue;
            }
            heads[i] = root;
            if last_noun == root {
                deps[i] = Dependency::Dobj;
            } else {
                deps[i] = Dependency::Nmod;
            }
            last_noun = i;
        }

        // 5. Refine: connect auxiliaries to root
        for i in 0..n {
            if i == root {
                continue;
            }
            if self.is_aux(&tokens[i].to_lowercase())
                && heads[i] == root
                && deps[i] != Dependency::Punct
            {
                deps[i] = Dependency::Aux;
            }
            if self.is_cop(&tokens[i].to_lowercase())
                && heads[i] == root
                && deps[i] != Dependency::Punct
            {
                deps[i] = Dependency::Cop;
            }
        }

        DependencyParse {
            tokens: tokens.to_vec(),
            heads,
            deps,
            root,
        }
    }

    /// Extract causal templates from parsed sentence
    pub fn extract_causal(&self, parse: &DependencyParse) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        let causal = [
            "porque",
            "ya_que",
            "dado_que",
            "por_tanto",
            "asi_que",
            "si",
            "entonces",
            "debido_a",
            "because",
            "since",
            "given",
            "therefore",
            "so",
            "if",
            "then",
            "due_to",
        ];
        for (i, t) in parse.tokens.iter().enumerate() {
            let w = t.to_lowercase().replace("_", " ");
            if causal.contains(&w.as_str()) {
                let before = parse.tokens[..i].join(" ");
                let after = parse.tokens[i + 1..].join(" ");
                match w.as_str() {
                    "porque" | "because" | "since" | "ya_que" | "dado_que" | "debido_a" => {
                        pairs.push((after.clone(), before.clone()));
                    }
                    "asi_que" | "therefore" | "so" | "por_tanto" => {
                        pairs.push((before.clone(), after.clone()));
                    }
                    "si" | "if" => {
                        if let Some(then_pos) = parse.tokens[i + 1..].iter().position(|x| {
                            x.to_lowercase() == "entonces" || x.to_lowercase() == "then"
                        }) {
                            let cond = parse.tokens[i + 1..i + 1 + then_pos].join(" ");
                            let cons = parse.tokens[i + 1 + then_pos + 1..].join(" ");
                            pairs.push((cond, cons));
                        } else {
                            pairs.push((before.clone(), after.clone()));
                        }
                    }
                    _ => {}
                }
            }
        }
        pairs
    }

    /// Extract causal templates and split compound causes/effects on coordination words.
    /// E.g., "X y Y porque Z" -> [(Z, X), (Z, Y)]
    /// E.g., "X porque A y B" -> [(A, X), (B, X)]
    pub fn extract_causal_compound(&self, parse: &DependencyParse) -> Vec<(String, String)> {
        let basic = self.extract_causal(parse);
        let mut expanded = Vec::new();
        for (cause, effect) in basic {
            let causes = Self::split_compound(&cause);
            let effects = Self::split_compound(&effect);
            for c in &causes {
                for e in &effects {
                    if !c.is_empty() && !e.is_empty() {
                        expanded.push((c.clone(), e.clone()));
                    }
                }
            }
        }
        expanded
    }

    /// Split a phrase on coordination markers (y, and, o, or, ni, nor, tambien, also).
    /// Returns at least the original if no split possible.
    fn split_compound(phrase: &str) -> Vec<String> {
        let lower = phrase.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        let mut chunks: Vec<Vec<&str>> = vec![Vec::new()];
        for w in &words {
            let connector = matches!(
                *w,
                "y" | "and" | "o" | "or" | "ni" | "nor" | "tambien" | "also"
            );
            if connector {
                if !chunks.last().map(|c| c.is_empty()).unwrap_or(true) {
                    chunks.push(Vec::new());
                }
            } else {
                chunks.last_mut().unwrap().push(w);
            }
        }
        let parts: Vec<String> = chunks
            .into_iter()
            .filter(|c| !c.is_empty())
            .map(|c| c.join(" ").trim().to_string())
            .filter(|s| s.split_whitespace().count() >= 1)
            .collect();
        if parts.is_empty() {
            vec![phrase.to_string()]
        } else {
            parts
        }
    }

    /// Extract subject-verb-object as a flat action string
    pub fn extract_svo(&self, parse: &DependencyParse) -> Option<(String, String, String)> {
        let subject_tokens: Vec<String> = parse
            .deps
            .iter()
            .enumerate()
            .filter(|(_, d)| **d == Dependency::Nsubj)
            .map(|(i, _)| parse.tokens[i].clone())
            .collect();
        let verb = parse.tokens.get(parse.root).cloned().unwrap_or_default();
        let object_tokens: Vec<String> = parse
            .deps
            .iter()
            .enumerate()
            .filter(|(_, d)| **d == Dependency::Dobj)
            .map(|(i, _)| parse.tokens[i].clone())
            .collect();
        if subject_tokens.is_empty() && object_tokens.is_empty() {
            return None;
        }
        Some((subject_tokens.join(" "), verb, object_tokens.join(" ")))
    }

    pub fn status(&self) -> String {
        format!(
            "Syntax | rules: verb={} | aux={} | cop={} | prep={}",
            self.verbs.len(),
            self.auxiliaries.len(),
            self.copulas.len(),
            self.prepositions.len()
        )
    }
}
