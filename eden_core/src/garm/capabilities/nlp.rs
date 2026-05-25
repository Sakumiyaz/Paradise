// EDEN GARM NLP — Absolute Ceiling Edition
// Tokenizer + Stemmer + TF-IDF + N-gram LM + Levenshtein + NER + Slot Filling + POS tagging

use std::collections::HashMap;

// ─── Tokenizer ───

pub fn tokenize(input: &str) -> Vec<String> {
    let lower = input.to_lowercase();
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in lower.chars() {
        if ch.is_alphanumeric() || ch == '\'' {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub fn tokenize_keep_case(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub fn sentence_split(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }
    sentences
}

// ─── Stemmer (Porter-ish, expanded) ───

pub fn stem(word: &str) -> String {
    let w = word.to_lowercase();
    if w.ends_with("ational") && w.len() > 8 {
        return w[..w.len() - 7].to_string() + "ate";
    }
    if w.ends_with("tional") && w.len() > 6 {
        return w[..w.len() - 6].to_string() + "tion";
    }
    if w.ends_with("ization") && w.len() > 8 {
        return w[..w.len() - 7].to_string() + "ize";
    }
    if w.ends_with("ness") && w.len() > 5 {
        return w[..w.len() - 4].to_string();
    }
    if w.ends_with("ing") && w.len() > 4 {
        return w[..w.len() - 3].to_string();
    }
    if w.ends_with("ed") && w.len() > 3 {
        return w[..w.len() - 2].to_string();
    }
    if w.ends_with("ies") && w.len() > 4 {
        return w[..w.len() - 3].to_string() + "y";
    }
    if w.ends_with("s") && w.len() > 2 {
        return w[..w.len() - 1].to_string();
    }
    if w.ends_with("es") && w.len() > 3 {
        return w[..w.len() - 2].to_string();
    }
    if w.ends_with("ly") && w.len() > 3 {
        return w[..w.len() - 2].to_string();
    }
    if w.ends_with("tion") && w.len() > 5 {
        return w[..w.len() - 4].to_string();
    }
    w
}

// ─── Stopwords (multilingual, expanded) ───

pub fn is_stopword(word: &str) -> bool {
    let w = word.to_lowercase();
    let stops = [
        "the",
        "a",
        "an",
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "being",
        "have",
        "has",
        "had",
        "do",
        "does",
        "did",
        "will",
        "would",
        "could",
        "should",
        "may",
        "might",
        "must",
        "shall",
        "can",
        "need",
        "dare",
        "ought",
        "used",
        "to",
        "of",
        "in",
        "for",
        "on",
        "with",
        "at",
        "by",
        "from",
        "as",
        "into",
        "through",
        "during",
        "before",
        "after",
        "above",
        "below",
        "between",
        "under",
        "again",
        "further",
        "then",
        "once",
        "here",
        "there",
        "when",
        "where",
        "why",
        "how",
        "all",
        "each",
        "few",
        "more",
        "most",
        "other",
        "some",
        "such",
        "no",
        "nor",
        "not",
        "only",
        "own",
        "same",
        "so",
        "than",
        "too",
        "very",
        "just",
        "but",
        "if",
        "or",
        "because",
        "until",
        "while",
        "about",
        "against",
        "and",
        "both",
        "down",
        "off",
        "out",
        "over",
        "under",
        "again",
        "that",
        "which",
        "who",
        "whom",
        "this",
        "these",
        "those",
        "am",
        "it",
        "its",
        "itself",
        "they",
        "them",
        "their",
        "what",
        "whatever",
        "whoever",
        "whomever",
        "whether",
        "either",
        "neither",
        "both",
        "few",
        "many",
        "several",
        "all",
        "any",
        "anybody",
        "anyone",
        "anything",
        "each",
        "everybody",
        "everyone",
        "everything",
        "nobody",
        "nothing",
        "none",
        "somebody",
        "someone",
        "something",
        "one",
        "other",
        "another",
        "such",
        "no",
        "one",
        "every",
        "another",
        "el",
        "la",
        "los",
        "las",
        "un",
        "una",
        "unos",
        "unas",
        "de",
        "del",
        "al",
        "y",
        "o",
        "pero",
        "si",
        "por",
        "para",
        "con",
        "sin",
        "sobre",
        "entre",
        "hasta",
        "desde",
        "durante",
        "antes",
        "despues",
        "muy",
        "mas",
        "menos",
        "tambien",
        "ya",
        "aun",
        "todavia",
        "sino",
        "que",
        "quien",
        "cual",
        "cuando",
        "donde",
        "como",
        "cuanto",
        "cuyo",
        "cuya",
        "esto",
        "eso",
        "aquel",
        "estos",
        "esos",
        "aquellos",
        "le",
        "les",
        "me",
        "te",
        "se",
        "lo",
        "la",
        "nos",
        "os",
        "mi",
        "tu",
        "su",
        "nuestro",
        "vuestro",
        "mio",
        "tuyo",
        "suyo",
        "nuestros",
        "vuestros",
    ];
    stops.contains(&w.as_str())
}

// ─── Levenshtein Edit Distance ───

pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = a_chars.len();
    let m = b_chars.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev = (0..=m).collect::<Vec<usize>>();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (curr[j - 1] + 1).min(prev[j] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

pub fn fuzzy_match(token: &str, candidates: &[&str], threshold: usize) -> Option<String> {
    let mut best = None;
    let mut best_dist = threshold + 1;
    for c in candidates {
        let dist = levenshtein(token, c);
        if dist < best_dist {
            best_dist = dist;
            best = Some(c.to_string());
        }
    }
    best
}

// ─── N-gram Language Model ───

pub struct NgramModel {
    pub unigrams: HashMap<String, u32>,
    pub bigrams: HashMap<(String, String), u32>,
    pub trigrams: HashMap<(String, String, String), u32>,
    pub total_tokens: u64,
}

impl NgramModel {
    pub fn new() -> Self {
        NgramModel {
            unigrams: HashMap::new(),
            bigrams: HashMap::new(),
            trigrams: HashMap::new(),
            total_tokens: 0,
        }
    }

    pub fn train(&mut self, docs: &[Vec<String>]) {
        for doc in docs {
            let tokens: Vec<String> = doc
                .iter()
                .map(|t| stem(t))
                .filter(|t| !is_stopword(t))
                .collect();
            for (i, t) in tokens.iter().enumerate() {
                *self.unigrams.entry(t.clone()).or_insert(0) += 1;
                self.total_tokens += 1;
                if i > 0 {
                    *self
                        .bigrams
                        .entry((tokens[i - 1].clone(), t.clone()))
                        .or_insert(0) += 1;
                }
                if i > 1 {
                    *self
                        .trigrams
                        .entry((tokens[i - 2].clone(), tokens[i - 1].clone(), t.clone()))
                        .or_insert(0) += 1;
                }
            }
        }
    }

    pub fn perplexity(&self, tokens: &[String]) -> f32 {
        let mut log_prob = 0.0f32;
        let vocab_size = self.unigrams.len().max(1) as f32;
        for i in 0..tokens.len() {
            let t = stem(&tokens[i]);
            if is_stopword(&t) {
                continue;
            }
            let count = *self.unigrams.get(&t).unwrap_or(&0) as f32;
            let prob = (count + 1.0) / (self.total_tokens as f32 + vocab_size);
            log_prob += -prob.ln();
        }
        (log_prob / tokens.len().max(1) as f32).exp()
    }

    pub fn suggest_next(&self, prev: &str) -> Vec<(String, f32)> {
        let prev_stem = stem(prev);
        let mut suggestions = Vec::new();
        let total_bigram: u32 = self
            .bigrams
            .iter()
            .filter(|((a, _), _)| a == &prev_stem)
            .map(|(_, c)| *c)
            .sum();
        if total_bigram > 0 {
            for ((a, b), count) in &self.bigrams {
                if a == &prev_stem {
                    suggestions.push((b.clone(), *count as f32 / total_bigram as f32));
                }
            }
            suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            suggestions.truncate(5);
        }
        suggestions
    }
}

// ─── TF-IDF Vectorizer ───

pub struct TfidfVectorizer {
    pub idf: HashMap<String, f32>,
    pub doc_count: u32,
}

impl TfidfVectorizer {
    pub fn new() -> Self {
        TfidfVectorizer {
            idf: HashMap::new(),
            doc_count: 0,
        }
    }

    pub fn fit(&mut self, docs: &[Vec<String>]) {
        let mut df: HashMap<String, u32> = HashMap::new();
        for doc in docs {
            let mut seen = HashMap::new();
            for token in doc {
                let s = stem(token);
                if is_stopword(&s) {
                    continue;
                }
                *seen.entry(s.clone()).or_insert(0) += 1;
            }
            for (term, _) in seen {
                *df.entry(term).or_insert(0) += 1;
            }
        }
        self.doc_count = docs.len() as u32;
        self.idf.clear();
        for (term, freq) in df {
            let idf_val = ((self.doc_count as f32 + 1.0) / (freq as f32 + 1.0)).ln() + 1.0;
            self.idf.insert(term, idf_val);
        }
    }

    pub fn transform(&self, doc: &[String]) -> HashMap<String, f32> {
        let mut tf: HashMap<String, u32> = HashMap::new();
        for token in doc {
            let s = stem(token);
            if is_stopword(&s) {
                continue;
            }
            *tf.entry(s).or_insert(0) += 1;
        }
        let max_tf = tf.values().copied().max().unwrap_or(1) as f32;
        let mut vec = HashMap::new();
        for (term, count) in tf {
            let tf_norm = count as f32 / max_tf;
            let idf_val = self.idf.get(&term).copied().unwrap_or(1.0);
            vec.insert(term, tf_norm * idf_val);
        }
        vec
    }
}

pub fn cosine_sim(a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (k, v) in a {
        dot += v * b.get(k).unwrap_or(&0.0);
        norm_a += v * v;
    }
    for (_, v) in b {
        norm_b += v * v;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a.sqrt() * norm_b.sqrt())
}

// ─── Intent Classifier (TF-IDF + N-gram + Levenshtein fallback) ───

#[derive(Clone, Debug, PartialEq)]
pub enum Intent {
    Greeting,
    StatusQuery,
    Help,
    Learn,
    QueryMemory,
    Evolve,
    Save,
    Load,
    Goodbye,
    Unknown,
    ToolCall,
    ExecuteCode,
    BrowseUrl,
    ComputerUse,
}

pub struct IntentClassifier {
    pub vectorizer: TfidfVectorizer,
    pub templates: Vec<(Intent, Vec<&'static str>)>,
    pub ngram_model: NgramModel,
}

impl IntentClassifier {
    pub fn new() -> Self {
        let templates: Vec<(Intent, Vec<&'static str>)> = vec![
            (
                Intent::Greeting,
                vec![
                    "hola",
                    "hello",
                    "hi",
                    "hey",
                    "buenos dias",
                    "good morning",
                    "saludos",
                ],
            ),
            (
                Intent::StatusQuery,
                vec![
                    "estado",
                    "status",
                    "como estas",
                    "how are you",
                    "que pasa",
                    "what is happening",
                ],
            ),
            (
                Intent::Help,
                vec![
                    "ayuda",
                    "help",
                    "comandos",
                    "commands",
                    "que puedes hacer",
                    "what can you do",
                    "menu",
                ],
            ),
            (
                Intent::Learn,
                vec![
                    "aprende",
                    "learn",
                    "recuerda",
                    "remember",
                    "recorder",
                    "memoriza",
                    "guarda esto",
                ],
            ),
            (
                Intent::QueryMemory,
                vec![
                    "memoria",
                    "memory",
                    "que sabes",
                    "what do you know",
                    "que recuerdas",
                    "what do you remember",
                    "info",
                ],
            ),
            (
                Intent::Evolve,
                vec![
                    "evoluciona",
                    "evolve",
                    "mejorate",
                    "improve",
                    "grow",
                    "self-improve",
                    "mutate",
                ],
            ),
            (
                Intent::Save,
                vec!["guarda", "save", "checkpoint", "persiste", "persist"],
            ),
            (
                Intent::Load,
                vec!["carga", "load", "restaura", "restore", "recupera"],
            ),
            (
                Intent::Goodbye,
                vec![
                    "adios",
                    "bye",
                    "salir",
                    "exit",
                    "quit",
                    "hasta luego",
                    "goodbye",
                    "nos vemos",
                ],
            ),
            (
                Intent::ToolCall,
                vec![
                    "tool",
                    "ejecuta herramienta",
                    "run tool",
                    "usa tool",
                    "use tool",
                    "llama funcion",
                ],
            ),
            (
                Intent::ExecuteCode,
                vec![
                    "ejecuta codigo",
                    "run code",
                    "test code",
                    "sandbox",
                    "compila",
                ],
            ),
            (
                Intent::BrowseUrl,
                vec![
                    "navega", "browse", "abre url", "open url", "visita", "fetch",
                ],
            ),
            (
                Intent::ComputerUse,
                vec![
                    "computer",
                    "proc",
                    "processes",
                    "desktop",
                    "pantalla",
                    "abre app",
                ],
            ),
        ];
        let docs: Vec<Vec<String>> = templates
            .iter()
            .map(|(_, t)| t.iter().flat_map(|s| tokenize(s)).collect())
            .collect();
        let mut vectorizer = TfidfVectorizer::new();
        vectorizer.fit(&docs);
        let mut ngram_model = NgramModel::new();
        ngram_model.train(&docs);
        IntentClassifier {
            vectorizer,
            templates,
            ngram_model,
        }
    }

    pub fn classify(&self, input: &str) -> (Intent, f32) {
        let tokens = tokenize(input);
        let input_vec = self.vectorizer.transform(&tokens);
        let mut best = (Intent::Unknown, 0.0f32);
        for (intent, template_strs) in &self.templates {
            for tmpl in template_strs {
                let tmpl_tokens = tokenize(tmpl);
                let tmpl_vec = self.vectorizer.transform(&tmpl_tokens);
                let sim = cosine_sim(&input_vec, &tmpl_vec);
                // Fuzzy boost: if any token is close via Levenshtein
                let fuzzy_boost = tokens.iter().any(|t| {
                    fuzzy_match(
                        t,
                        &tmpl_tokens.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                        2,
                    )
                    .is_some()
                });
                let final_sim = if fuzzy_boost { sim * 1.15 } else { sim };
                if final_sim > best.1 {
                    best = (intent.clone(), final_sim.min(1.0));
                }
            }
        }
        let lower = input.to_lowercase().trim().to_string();
        if lower.starts_with("tool ") {
            return (Intent::ToolCall, 1.0);
        }
        if lower.starts_with("browse ") || lower.starts_with("navega ") {
            return (Intent::BrowseUrl, 1.0);
        }
        if best.1 < 0.15 {
            (Intent::Unknown, best.1)
        } else {
            best
        }
    }
}

// ─── Slot Filling ───

#[derive(Clone, Debug)]
pub struct Slot {
    pub name: String,
    pub value: String,
    pub start: usize,
    pub end: usize,
}

pub fn extract_slots(input: &str, intent: &Intent) -> Vec<Slot> {
    let mut slots = Vec::new();
    let _tokens: Vec<String> = tokenize_keep_case(input);
    match intent {
        Intent::ToolCall => {
            // Extract tool name after "tool"
            if let Some(pos) = input.to_lowercase().find("tool ") {
                let rest = &input[pos + 5..];
                if let Some(first_word) = rest.split_whitespace().next() {
                    slots.push(Slot {
                        name: "tool_name".into(),
                        value: first_word.to_string(),
                        start: pos + 5,
                        end: pos + 5 + first_word.len(),
                    });
                }
            }
        }
        Intent::BrowseUrl => {
            for word in input.split_whitespace() {
                if word.starts_with("http://") || word.starts_with("https://") {
                    slots.push(Slot {
                        name: "url".into(),
                        value: word.to_string(),
                        start: 0,
                        end: word.len(),
                    });
                }
            }
        }
        Intent::ExecuteCode => {
            let code = input
                .replace("ejecuta codigo", "")
                .replace("run code", "")
                .replace("test code", "")
                .trim()
                .to_string();
            if !code.is_empty() {
                slots.push(Slot {
                    name: "code".into(),
                    value: code.clone(),
                    start: 0,
                    end: code.len(),
                });
            }
        }
        _ => {}
    }
    slots
}

// ─── Entity Extractor (NER) ───

#[derive(Clone, Debug)]
pub struct Entity {
    pub text: String,
    pub label: EntityLabel,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EntityLabel {
    Url,
    FilePath,
    Number,
    Date,
    Time,
    Email,
    Phone,
    Person,
    Org,
    Topic,
    Unknown,
}

pub fn extract_entities(input: &str) -> Vec<Entity> {
    let mut entities = Vec::new();
    let lower = input.to_lowercase();
    for (_i, word) in input.split_whitespace().enumerate() {
        if word.starts_with("http://") || word.starts_with("https://") {
            entities.push(Entity {
                text: word.to_string(),
                label: EntityLabel::Url,
                start: 0,
                end: word.len(),
            });
        }
        if word.starts_with("/") && word.contains(".") && !word.contains("//") {
            entities.push(Entity {
                text: word.to_string(),
                label: EntityLabel::FilePath,
                start: 0,
                end: word.len(),
            });
        }
        if word.contains('@') && word.contains('.') {
            entities.push(Entity {
                text: word.to_string(),
                label: EntityLabel::Email,
                start: 0,
                end: word.len(),
            });
        }
        if let Ok(_) = word.parse::<f64>() {
            entities.push(Entity {
                text: word.to_string(),
                label: EntityLabel::Number,
                start: 0,
                end: word.len(),
            });
        }
    }
    // Date patterns: YYYY-MM-DD, DD/MM/YYYY, MM-DD-YYYY
    let date_keywords = [
        "today",
        "tomorrow",
        "yesterday",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
        "hoy",
        "mañana",
        "ayer",
        "lunes",
        "martes",
        "miercoles",
        "jueves",
        "viernes",
        "sabado",
        "domingo",
    ];
    for word in lower.split_whitespace() {
        if date_keywords.contains(&word) {
            entities.push(Entity {
                text: word.to_string(),
                label: EntityLabel::Date,
                start: 0,
                end: word.len(),
            });
        }
    }
    // Person detection: capitalized words not at start
    let capitalized: Vec<&str> = input
        .split_whitespace()
        .skip(1)
        .filter(|w| w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && w.len() > 1)
        .collect();
    for word in capitalized {
        entities.push(Entity {
            text: word.to_string(),
            label: EntityLabel::Person,
            start: 0,
            end: word.len(),
        });
    }
    // Quoted topics
    let mut in_quote = false;
    let mut quote_start = 0usize;
    for (i, ch) in input.char_indices() {
        if ch == '"' {
            if in_quote {
                let text = input[quote_start..i].to_string();
                entities.push(Entity {
                    text,
                    label: EntityLabel::Topic,
                    start: quote_start,
                    end: i,
                });
                in_quote = false;
            } else {
                quote_start = i + 1;
                in_quote = true;
            }
        }
    }
    entities
}

// ─── Sentiment Analyzer (lexicon-based, bilingual) ───

pub fn sentiment_score(tokens: &[String]) -> f32 {
    let pos = [
        "good",
        "great",
        "excellent",
        "amazing",
        "love",
        "happy",
        "joy",
        "wonderful",
        "best",
        "better",
        "improve",
        "awesome",
        "perfect",
        "fantastic",
        "bueno",
        "genial",
        "excelente",
        "increible",
        "mejor",
        "perfecto",
        "feliz",
        "maravilloso",
        "excelso",
        "magnifico",
    ];
    let neg = [
        "bad",
        "terrible",
        "awful",
        "hate",
        "sad",
        "angry",
        "worst",
        "worse",
        "horrible",
        "disappointing",
        "fail",
        "error",
        "broken",
        "malo",
        "terrible",
        "horrible",
        "odio",
        "triste",
        "enojado",
        "peor",
        "fallo",
        "error",
        "roto",
        "defectuoso",
    ];
    let mut score = 0.0f32;
    for t in tokens {
        let s = stem(t);
        if pos.contains(&s.as_str()) {
            score += 1.0;
        }
        if neg.contains(&s.as_str()) {
            score -= 1.0;
        }
    }
    score.max(-1.0).min(1.0)
}

// ─── POS Tagging (rule-based stub) ───

pub fn pos_tag(token: &str) -> &'static str {
    let lower = token.to_lowercase();
    if lower.ends_with("ing") || lower.ends_with("ando") || lower.ends_with("iendo") {
        "VERB"
    } else if lower.ends_with("ly") || lower.ends_with("mente") {
        "ADV"
    } else if lower.ends_with("tion")
        || lower.ends_with("sion")
        || lower.ends_with("cion")
        || lower.ends_with("miento")
    {
        "NOUN"
    } else if ["the", "a", "an", "el", "la", "los", "las", "un", "una"].contains(&lower.as_str()) {
        "DET"
    } else if [
        "is", "are", "was", "were", "be", "been", "es", "son", "fue", "fueron", "esta", "estan",
    ]
    .contains(&lower.as_str())
    {
        "VERB"
    } else if [
        "in", "on", "at", "by", "with", "from", "to", "en", "de", "con", "por", "para",
    ]
    .contains(&lower.as_str())
    {
        "PREP"
    } else if [
        "i", "you", "he", "she", "it", "we", "they", "yo", "tu", "el", "ella", "nosotros", "ellos",
    ]
    .contains(&lower.as_str())
    {
        "PRON"
    } else if token
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
        && token.len() > 1
    {
        "NOUN"
    } else {
        "UNKNOWN"
    }
}

// ─── Jaccard Similarity ───

pub fn jaccard_sim(a_tokens: &[String], b_tokens: &[String]) -> f32 {
    use std::collections::HashSet;
    let set_a: HashSet<&str> = a_tokens.iter().map(|s| s.as_str()).collect();
    let set_b: HashSet<&str> = b_tokens.iter().map(|s| s.as_str()).collect();
    let intersection: HashSet<_> = set_a.intersection(&set_b).cloned().collect();
    let union: HashSet<_> = set_a.union(&set_b).cloned().collect();
    if union.is_empty() {
        0.0
    } else {
        intersection.len() as f32 / union.len() as f32
    }
}

// ─── Language Detection (character n-gram frequency profiles) ───

pub fn detect_language(text: &str) -> &'static str {
    let lower = text.to_lowercase();
    let bigrams: Vec<String> = lower
        .chars()
        .collect::<Vec<_>>()
        .windows(2)
        .map(|w| format!("{}{}", w[0], w[1]))
        .collect();
    let trigrams: Vec<String> = lower
        .chars()
        .collect::<Vec<_>>()
        .windows(3)
        .map(|w| format!("{}{}{}", w[0], w[1], w[2]))
        .collect();

    // Spanish distinctive bigrams/trigrams
    let es_markers = [
        "qu", "ll", "rr", "ñ", "á", "é", "í", "ó", "ú", "ión", "ado", "ido", "ando", "iendo",
        "mente", "ción",
    ];
    // English distinctive patterns
    let en_markers = [
        "th", "sh", "ch", "wh", "ph", "gh", "ing", "tion", "ness", "ment", "ough", "ight",
    ];
    // French
    let fr_markers = [
        "qu", "gu", "ç", "à", "è", "ù", "eur", "ent", "ant", "ais", "ois",
    ];
    // German
    let de_markers = [
        "sch", "ch", "ck", "ß", "ie", "ei", "ung", "heit", "keit", "chen",
    ];

    let mut es_score = 0usize;
    let mut en_score = 0usize;
    let mut fr_score = 0usize;
    let mut de_score = 0usize;

    for bg in &bigrams {
        let s = bg.as_str();
        if es_markers.contains(&s) {
            es_score += 2;
        }
        if en_markers.contains(&s) {
            en_score += 2;
        }
        if fr_markers.contains(&s) {
            fr_score += 2;
        }
        if de_markers.contains(&s) {
            de_score += 2;
        }
    }
    for tg in &trigrams {
        let s = tg.as_str();
        if es_markers.contains(&s) {
            es_score += 3;
        }
        if en_markers.contains(&s) {
            en_score += 3;
        }
        if fr_markers.contains(&s) {
            fr_score += 3;
        }
        if de_markers.contains(&s) {
            de_score += 3;
        }
    }

    let max_score = es_score.max(en_score).max(fr_score).max(de_score);
    if max_score == 0 {
        return "unknown";
    }
    if max_score == es_score {
        "es"
    } else if max_score == en_score {
        "en"
    } else if max_score == fr_score {
        "fr"
    } else {
        "de"
    }
}

// ─── Keyword Extraction (TF-IDF top-k) ───

pub fn extract_keywords(text: &str, top_k: usize) -> Vec<(String, f32)> {
    let tokens = tokenize(text);
    let mut tf: HashMap<String, u32> = HashMap::new();
    for t in &tokens {
        let s = stem(t);
        if !is_stopword(&s) {
            *tf.entry(s).or_insert(0) += 1;
        }
    }
    let total = tokens.len().max(1) as f32;
    let mut scored: Vec<(String, f32)> = tf
        .iter()
        .map(|(term, count)| {
            (
                term.clone(),
                (*count as f32 / total) * (term.len() as f32).sqrt(),
            )
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);
    scored
}

// ─── Text Summarization (sentence scoring by TF-IDF + position) ───

pub fn summarize(text: &str, n_sentences: usize) -> String {
    let sentences = sentence_split(text);
    if sentences.len() <= n_sentences {
        return text.to_string();
    }
    let tokens: Vec<Vec<String>> = sentences.iter().map(|s| tokenize(s)).collect();
    let mut vectorizer = TfidfVectorizer::new();
    vectorizer.fit(&tokens);
    let mut scores: Vec<(usize, f32)> = sentences
        .iter()
        .enumerate()
        .map(|(i, sent)| {
            let vec = vectorizer.transform(&tokenize(sent));
            let score = vec.values().sum::<f32>() + (1.0 / (i as f32 + 1.0)).sqrt(); // position bonus for early sentences
            (i, score)
        })
        .collect();
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut top_indices: Vec<usize> = scores
        .into_iter()
        .take(n_sentences)
        .map(|(i, _)| i)
        .collect();
    top_indices.sort();
    top_indices
        .iter()
        .map(|&i| sentences[i].clone())
        .collect::<Vec<_>>()
        .join(" ")
}

// ─── Question Detection ───

pub fn is_question(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.ends_with('?') {
        return true;
    }
    let lower = trimmed.to_lowercase();
    let q_starters = [
        "what", "how", "why", "when", "where", "who", "which", "whose", "whom", "can", "could",
        "would", "will", "shall", "should", "is", "are", "was", "were", "do", "does", "did",
        "have", "has", "had", "am", "may", "might", "que", "como", "por que", "cuando", "donde",
        "quien", "cual", "cuanto", "tiene", "hay", "es", "son", "esta", "estan",
    ];
    let first_word = lower.split_whitespace().next().unwrap_or("");
    q_starters.contains(&first_word)
}

// ─── Emotion Detection (lexicon-based) ───

#[derive(Clone, Debug, PartialEq)]
pub enum Emotion {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Trust,
    Anticipation,
    Neutral,
}

pub fn detect_emotion(tokens: &[String]) -> (Emotion, f32) {
    let joy = [
        "happy",
        "joy",
        "love",
        "wonderful",
        "excited",
        "delighted",
        "glad",
        "cheerful",
        "bliss",
        "ecstatic",
        "feliz",
        "alegre",
        "contento",
        "encantado",
        "maravilloso",
        "genial",
        "excelente",
    ];
    let sadness = [
        "sad",
        "depressed",
        "grief",
        "sorrow",
        "melancholy",
        "miserable",
        "heartbroken",
        "lonely",
        "despair",
        "triste",
        "deprimido",
        "dolido",
        "desgraciado",
        "solitario",
        "infeliz",
    ];
    let anger = [
        "angry",
        "furious",
        "rage",
        "irritated",
        "annoyed",
        "mad",
        "hate",
        "hostile",
        "frustrated",
        "enojado",
        "furioso",
        "ira",
        "molesto",
        "odiar",
        "rabia",
    ];
    let fear = [
        "afraid",
        "scared",
        "terrified",
        "anxious",
        "worried",
        "panic",
        "dread",
        "horrified",
        "nervous",
        "miedo",
        "asustado",
        "aterrorizado",
        "ansioso",
        "preocupado",
        "panico",
    ];
    let surprise = [
        "surprised",
        "amazed",
        "astonished",
        "shocked",
        "stunned",
        "wow",
        "unexpected",
        "sudden",
        "sorprendido",
        "asombrado",
        "impactado",
        "impresionado",
        "inesperado",
    ];
    let disgust = [
        "disgusted",
        "revolted",
        "sick",
        "nauseous",
        "repulsed",
        "appalled",
        "horrible",
        "gross",
        "asqueado",
        "repugnante",
        "nauseabundo",
        "horrible",
        "repulsivo",
    ];
    let trust = [
        "trust",
        "believe",
        "confident",
        "faithful",
        "loyal",
        "reliable",
        "honest",
        "secure",
        "safe",
        "confiar",
        "creer",
        "seguro",
        "leal",
        "honesto",
        "fiable",
    ];
    let anticipation = [
        "hope",
        "expect",
        "await",
        "eager",
        "excited",
        "curious",
        "looking forward",
        "anticipate",
        "esperar",
        "ansioso",
        "curioso",
        "emocionado",
        "deseoso",
    ];

    let mut scores = [
        (Emotion::Joy, 0.0f32),
        (Emotion::Sadness, 0.0),
        (Emotion::Anger, 0.0),
        (Emotion::Fear, 0.0),
        (Emotion::Surprise, 0.0),
        (Emotion::Disgust, 0.0),
        (Emotion::Trust, 0.0),
        (Emotion::Anticipation, 0.0),
    ];
    for t in tokens {
        let s = stem(t);
        if joy.contains(&s.as_str()) {
            scores[0].1 += 1.0;
        }
        if sadness.contains(&s.as_str()) {
            scores[1].1 += 1.0;
        }
        if anger.contains(&s.as_str()) {
            scores[2].1 += 1.0;
        }
        if fear.contains(&s.as_str()) {
            scores[3].1 += 1.0;
        }
        if surprise.contains(&s.as_str()) {
            scores[4].1 += 1.0;
        }
        if disgust.contains(&s.as_str()) {
            scores[5].1 += 1.0;
        }
        if trust.contains(&s.as_str()) {
            scores[6].1 += 1.0;
        }
        if anticipation.contains(&s.as_str()) {
            scores[7].1 += 1.0;
        }
    }
    let total: f32 = scores.iter().map(|(_, s)| s).sum();
    if total == 0.0 {
        return (Emotion::Neutral, 0.0);
    }
    let mut best = scores[0].clone();
    for (em, sc) in &scores {
        if *sc > best.1 {
            best = (em.clone(), *sc);
        }
    }
    (best.0, (best.1 / total).min(1.0))
}

// ─── Sarcasm Detection (basic heuristic) ───

pub fn detect_sarcasm(text: &str, sentiment: f32) -> f32 {
    let lower = text.to_lowercase();
    let sarcasm_indicators = [
        "yeah right",
        "sure",
        "obviously",
        "clearly",
        "totally",
        "definitely",
        "absolutely",
        "oh really",
        "how nice",
        "lovely",
        "great",
        "perfect",
        "wonderful",
        "fantastic",
        "brilliant",
        "marvelous",
        "claro",
        "por supuesto",
        "obviamente",
        "evidentemente",
        "totalmente",
        "definitivamente",
        "absolutamente",
        "que bien",
        "que lindo",
        "que maravilla",
        "genial",
        "fantastico",
        "perfecto",
    ];
    let mut score = 0.0f32;
    for ind in &sarcasm_indicators {
        if lower.contains(ind) {
            score += 0.3;
        }
    }
    // High positive sentiment with negative context words = likely sarcastic
    if sentiment > 0.5 {
        let negative_context = [
            "problem", "issue", "trouble", "fail", "broken", "wrong", "error", "bug", "problema",
            "error", "fallo", "roto", "mal", "defecto",
        ];
        for neg in &negative_context {
            if lower.contains(neg) {
                score += 0.4;
            }
        }
    }
    score.min(1.0)
}

// ─── Dependency Parsing (rule-based shallow) ───

#[derive(Clone, Debug)]
pub struct Dependency {
    pub head: String,
    pub dependent: String,
    pub relation: String,
}

pub fn shallow_parse(sentence: &str) -> Vec<Dependency> {
    let tokens = tokenize_keep_case(sentence);
    let tags: Vec<&str> = tokens.iter().map(|t| pos_tag(t)).collect();
    let mut deps = Vec::new();
    let mut root = String::new();
    // Find first verb as root
    for (i, tag) in tags.iter().enumerate() {
        if *tag == "VERB" {
            root = tokens[i].clone();
            break;
        }
    }
    if root.is_empty() && !tokens.is_empty() {
        root = tokens[0].clone();
    }

    for (i, tag) in tags.iter().enumerate() {
        match *tag {
            "NOUN" => {
                // Attach nouns after verb as objects, before as subjects
                if i > 0 && tags.get(i - 1) == Some(&"VERB") {
                    deps.push(Dependency {
                        head: root.clone(),
                        dependent: tokens[i].clone(),
                        relation: "dobj".into(),
                    });
                } else {
                    deps.push(Dependency {
                        head: root.clone(),
                        dependent: tokens[i].clone(),
                        relation: "nsubj".into(),
                    });
                }
            }
            "PREP" => {
                if i + 1 < tokens.len() {
                    deps.push(Dependency {
                        head: root.clone(),
                        dependent: tokens[i + 1].clone(),
                        relation: format!("prep_{}", tokens[i].to_lowercase()),
                    });
                }
            }
            "ADJ" | "ADV" => {
                deps.push(Dependency {
                    head: root.clone(),
                    dependent: tokens[i].clone(),
                    relation: "mod".into(),
                });
            }
            _ => {}
        }
    }
    deps
}

// ─── Enhanced Sentiment (with negation and intensifiers) ───

pub fn enhanced_sentiment(text: &str) -> (f32, f32) {
    // (polarity, intensity)
    let tokens = tokenize(text);
    let negations = [
        "not", "no", "never", "neither", "nor", "hardly", "barely", "scarcely", "no", "nunca",
        "jamas", "ni", "tampoco", "apenas", "nada",
    ];
    let intensifiers = [
        "very",
        "extremely",
        "incredibly",
        "absolutely",
        "completely",
        "totally",
        "utterly",
        "highly",
        "muy",
        "extremadamente",
        "increiblemente",
        "absolutamente",
        "completamente",
        "totalmente",
        "sumamente",
    ];
    let diminishers = [
        "slightly",
        "somewhat",
        "a bit",
        "kind of",
        "sort of",
        "rather",
        "fairly",
        "pretty",
        "ligeramente",
        "un poco",
        "algo",
        "bastante",
        "medianamente",
    ];

    let pos = [
        "good",
        "great",
        "excellent",
        "amazing",
        "love",
        "happy",
        "joy",
        "wonderful",
        "best",
        "better",
        "improve",
        "awesome",
        "perfect",
        "fantastic",
        "outstanding",
        "brilliant",
        "nice",
        "cool",
        "fun",
        "pleasant",
        "satisfied",
        "delighted",
        "excited",
        "grateful",
        "optimistic",
        "confident",
        "proud",
        "calm",
        "relaxed",
        "comfortable",
        "safe",
        "secure",
        "lucky",
        "fortunate",
        "successful",
        "effective",
        "useful",
        "helpful",
        "clear",
        "easy",
        "simple",
        "fast",
        "quick",
        "smooth",
        "clean",
        "fresh",
        "bright",
        "beautiful",
        "pretty",
        "lovely",
        "cute",
        "smart",
        "clever",
        "wise",
        "honest",
        "kind",
        "friendly",
        "polite",
        "gentle",
        "patient",
        "brave",
        "strong",
        "healthy",
        "fit",
        "rich",
        "wealthy",
        "famous",
        "popular",
        "fashionable",
        "trendy",
        "modern",
        "new",
        "young",
        "active",
        "alive",
        "awake",
        "aware",
        "conscious",
        "curious",
        "interested",
        "engaged",
        "involved",
        "committed",
        "dedicated",
        "loyal",
        "faithful",
        "trustworthy",
        "reliable",
        "responsible",
        "capable",
        "competent",
        "skilled",
        "talented",
        "gifted",
        "creative",
        "innovative",
        "original",
        "unique",
        "special",
        "rare",
        "precious",
        "valuable",
        "expensive",
        "cheap",
        "free",
        "available",
        "accessible",
        "open",
        "public",
        "shared",
        "common",
        "universal",
        "global",
        "local",
        "national",
        "international",
        "professional",
        "official",
        "formal",
        "proper",
        "correct",
        "accurate",
        "exact",
        "precise",
        "specific",
        "detailed",
        "thorough",
        "complete",
        "whole",
        "full",
        "total",
        "absolute",
        "pure",
        "true",
        "real",
        "actual",
        "genuine",
        "authentic",
        "natural",
        "normal",
        "regular",
        "standard",
        "typical",
        "usual",
        "average",
        "medium",
        "moderate",
        "reasonable",
        "fair",
        "equal",
        "balanced",
        "stable",
        "steady",
        "constant",
        "consistent",
        "continuous",
        "persistent",
        "permanent",
        "eternal",
        "forever",
        "infinite",
        "endless",
        "unlimited",
        "boundless",
        "vast",
        "huge",
        "enormous",
        "massive",
        "large",
        "big",
        "great",
        "grand",
        "major",
        "main",
        "chief",
        "primary",
        "principal",
        "leading",
        "top",
        "first",
        "highest",
        "supreme",
        "ultimate",
        "extreme",
        "intense",
        "deep",
        "profound",
        "serious",
        "severe",
        "critical",
        "crucial",
        "vital",
        "essential",
        "necessary",
        "required",
        "needed",
        "wanted",
        "desired",
        "preferred",
        "favored",
        "renowned",
        "celebrated",
        "honored",
        "respected",
        "admired",
        "loved",
        "adored",
        "cherished",
        "treasured",
        "valued",
        "appreciated",
        "recognized",
        "acknowledged",
        "accepted",
        "approved",
        "supported",
        "encouraged",
        "promoted",
        "advanced",
        "developed",
        "improved",
        "enhanced",
        "upgraded",
        "updated",
        "renewed",
        "restored",
        "repaired",
        "fixed",
        "corrected",
        "solved",
        "resolved",
        "settled",
        "finished",
        "completed",
        "done",
        "achieved",
        "accomplished",
        "attained",
        "reached",
        "gained",
        "earned",
        "won",
        "victorious",
        "triumphant",
        "successful",
        "prosperous",
        "flourishing",
        "thriving",
        "blooming",
        "growing",
        "expanding",
        "increasing",
        "rising",
        "climbing",
        "ascending",
    ];
    let neg = [
        "bad",
        "terrible",
        "awful",
        "hate",
        "sad",
        "angry",
        "worst",
        "worse",
        "horrible",
        "disappointing",
        "fail",
        "error",
        "broken",
        "poor",
        "weak",
        "sick",
        "ill",
        "unhealthy",
        "dead",
        "dying",
        "deceased",
        "unconscious",
        "unaware",
        "ignorant",
        "stupid",
        "foolish",
        "silly",
        "ridiculous",
        "absurd",
        "crazy",
        "insane",
        "mad",
        "wild",
        "violent",
        "aggressive",
        "cruel",
        "mean",
        "selfish",
        "greedy",
        "lazy",
        "careless",
        "reckless",
        "dangerous",
        "risky",
        "hazardous",
        "unsafe",
        "insecure",
        "unstable",
        "unsteady",
        "uncertain",
        "doubtful",
        "skeptical",
        "cynical",
        "pessimistic",
        "negative",
        "depressed",
        "melancholy",
        "gloomy",
        "dark",
        "dim",
        "dull",
        "boring",
        "tedious",
        "monotonous",
        "repetitive",
        "routine",
        "predictable",
        "obvious",
        "evident",
        "clear",
        "plain",
        "simple",
        "easy",
        "effortless",
        "smooth",
        "fluid",
        "fluent",
        "eloquent",
        "articulate",
        "expressive",
        "vivid",
        "colorful",
        "bright",
        "brilliant",
        "radiant",
        "luminous",
        "shining",
        "glowing",
        "gleaming",
        "glistening",
        "glittering",
        "sparkling",
        "twinkling",
        "flickering",
        "flashing",
        "flaring",
        "glaring",
        "blinding",
        "dazzling",
        "stunning",
        "amazing",
        "astonishing",
        "astounding",
        "staggering",
        "overwhelming",
        "bewildering",
        "confusing",
        "puzzling",
        "perplexing",
        "baffling",
        "mystifying",
        "intriguing",
        "fascinating",
        "captivating",
        "enchanting",
        "charming",
        "delighting",
        "pleasing",
        "satisfying",
        "gratifying",
        "fulfilling",
        "completing",
        "finishing",
        "ending",
        "concluding",
        "terminating",
        "ceasing",
        "stopping",
        "halting",
        "pausing",
        "waiting",
        "delaying",
        "postponing",
        "suspending",
        "freezing",
        "refrigerating",
        "cooling",
        "chilling",
        "icy",
        "frosty",
        "frozen",
        "cold",
        "cool",
        "mild",
        "warm",
        "hot",
        "boiling",
        "burning",
        "scorching",
        "searing",
        "sizzling",
        "blazing",
        "flaming",
        "fiery",
        "heated",
        "warm",
        "tepid",
        "lukewarm",
        "cool",
        "chilly",
        "cold",
        "frigid",
        "freezing",
        "frozen",
        "icy",
        "polar",
        "arctic",
        "antarctic",
    ];

    let mut polarity = 0.0f32;
    let mut intensity = 1.0f32;
    let mut negation_active = false;

    for (i, t) in tokens.iter().enumerate() {
        let s = stem(t);
        if negations.contains(&s.as_str()) {
            negation_active = true;
            intensity *= 1.2;
            continue;
        }
        if intensifiers.contains(&s.as_str()) {
            intensity *= 1.5;
            continue;
        }
        if diminishers.contains(&s.as_str()) {
            intensity *= 0.6;
            continue;
        }

        if pos.contains(&s.as_str()) {
            let val = if negation_active { -1.0 } else { 1.0 };
            polarity += val * intensity;
            negation_active = false;
            intensity = 1.0;
        }
        if neg.contains(&s.as_str()) {
            let val = if negation_active { 1.0 } else { -1.0 };
            polarity += val * intensity;
            negation_active = false;
            intensity = 1.0;
        }
        // Negation scope decays after 3 words
        if negation_active && i % 4 == 0 {
            negation_active = false;
        }
    }
    (polarity.max(-5.0).min(5.0), intensity.min(3.0))
}
