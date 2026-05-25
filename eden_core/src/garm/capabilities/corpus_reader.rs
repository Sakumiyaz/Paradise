// EDEN GARM CorpusReader — Autonomous text ingestion. 100% Rust puro, 0 LLM, 0 red.
//
// Lee archivos de texto locales en background, segmenta en oraciones, y alimenta
// los primitivos de generalizacion ya existentes de EDEN:
//
//   text -> tokenize       (nlp.rs)
//        -> syntax.parse   -> SVO + pares causales
//        -> scene_parser   -> agente/accion/paciente/contexto
//        -> semantics      -> co-ocurrencia + embeddings online
//        -> morphogenesis  -> clusters de conceptos
//        -> causality      -> modelo causal estructural
//        -> hippocampus    -> memoria episodica densa
//
// Sin pre-trained vectors. Sin transformers. Sin internet. Solo std::fs.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::BufRead;

#[derive(Clone, Debug)]
pub struct CorpusSource {
    pub path: String,
    pub sentences: VecDeque<String>,
    pub total_sentences: usize,
    pub sentences_consumed: usize,
}

#[derive(Clone, Debug)]
pub struct CorpusReader {
    pub sources: Vec<CorpusSource>,
    pub current_source: usize,
    pub sentences_per_tick: usize,
    pub total_processed: u64,
    pub total_concepts_added: u64,
    pub total_causal_pairs: u64,
    pub total_svos: u64,
    pub paused: bool,
    pub min_sentence_len: usize,
    pub max_sentence_len: usize,
    /// Maps normalized phrase text to its concept id, so identical phrases reuse the same concept
    /// regardless of geometric clustering. This makes the causal graph reflect actual sentence structure.
    pub phrase_to_concept: HashMap<String, u64>,
    /// Performance: total time spent processing (microseconds)
    pub total_processing_us: u64,
    /// Throughput: sentences/sec computed periodically
    pub last_throughput: f32,
}

impl CorpusReader {
    pub fn new() -> Self {
        CorpusReader {
            sources: Vec::new(),
            current_source: 0,
            sentences_per_tick: 2,
            total_processed: 0,
            total_concepts_added: 0,
            total_causal_pairs: 0,
            total_svos: 0,
            paused: false,
            min_sentence_len: 3,
            max_sentence_len: 200,
            phrase_to_concept: HashMap::new(),
            total_processing_us: 0,
            last_throughput: 0.0,
        }
    }

    /// Normalize a phrase for deduplication: lowercase, trim, collapse whitespace.
    pub fn normalize_phrase(text: &str) -> String {
        text.to_lowercase()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Streaming variant: reads file line by line via BufReader without loading the
    /// whole file into memory. Each line becomes a sentence (or split further by punctuation).
    /// Suitable for large corpora (books, Wikipedia dumps).
    pub fn load_file_streaming(&mut self, path: &str) -> Result<usize, String> {
        let f = std::fs::File::open(path).map_err(|e| format!("open {}: {}", path, e))?;
        let reader = std::io::BufReader::new(f);
        let mut sentences: VecDeque<String> = VecDeque::new();
        let mut buffer = String::new();
        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue,
            };
            // Accumulate lines until we hit a sentence boundary
            buffer.push_str(&line);
            buffer.push(' ');
            // Split on sentence boundaries
            let parts = Self::segment_sentences(&buffer);
            // If last part doesn't end in . ! ? then keep it as carry-over
            let last_terminal = buffer.trim_end().ends_with('.')
                || buffer.trim_end().ends_with('!')
                || buffer.trim_end().ends_with('?');
            if parts.len() > 1 || (parts.len() == 1 && last_terminal) {
                let n_complete = if last_terminal {
                    parts.len()
                } else {
                    parts.len().saturating_sub(1)
                };
                for s in parts.iter().take(n_complete) {
                    let words = s.split_whitespace().count();
                    if words >= self.min_sentence_len && words <= self.max_sentence_len {
                        sentences.push_back(s.clone());
                    }
                }
                buffer = if last_terminal {
                    String::new()
                } else {
                    parts.last().cloned().unwrap_or_default()
                };
            }
        }
        // Final flush
        let leftover = buffer.trim().to_string();
        if !leftover.is_empty() {
            let words = leftover.split_whitespace().count();
            if words >= self.min_sentence_len && words <= self.max_sentence_len {
                sentences.push_back(leftover);
            }
        }
        let kept = sentences.len();
        self.sources.push(CorpusSource {
            path: path.to_string(),
            sentences,
            total_sentences: kept,
            sentences_consumed: 0,
        });
        Ok(kept)
    }

    /// Carga un archivo de texto local y lo segmenta en oraciones.
    pub fn load_file(&mut self, path: &str) -> Result<usize, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("read {}: {}", path, e))?;
        let sentences = Self::segment_sentences(&content);
        let filtered: VecDeque<String> = sentences
            .into_iter()
            .filter(|s| {
                let words = s.split_whitespace().count();
                words >= self.min_sentence_len && words <= self.max_sentence_len
            })
            .collect();
        let kept = filtered.len();
        self.sources.push(CorpusSource {
            path: path.to_string(),
            sentences: filtered,
            total_sentences: kept,
            sentences_consumed: 0,
        });
        Ok(kept)
    }

    /// Carga todos los .txt y .md de un directorio (no recursivo).
    pub fn load_directory(&mut self, dir: &str) -> Result<usize, String> {
        let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
        let mut total = 0usize;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    if ext == "txt" || ext == "md" {
                        if let Some(s) = p.to_str() {
                            if let Ok(n) = self.load_file(s) {
                                total += n;
                            }
                        }
                    }
                }
            }
        }
        Ok(total)
    }

    /// Segmentacion de oraciones por reglas: corta en . ! ? \n cuando seguido de espacio/EOF.
    pub fn segment_sentences(text: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = text.chars().collect();
        for (i, ch) in chars.iter().enumerate() {
            current.push(*ch);
            if *ch == '.' || *ch == '!' || *ch == '?' || *ch == '\n' {
                let next_is_boundary = chars.get(i + 1).map(|c| c.is_whitespace()).unwrap_or(true);
                if next_is_boundary {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() && trimmed.split_whitespace().count() >= 1 {
                        out.push(trimmed);
                    }
                    current.clear();
                }
            }
        }
        let last = current.trim().to_string();
        if !last.is_empty() {
            out.push(last);
        }
        out
    }

    /// Saca la siguiente oracion de la fuente activa. Rota cuando una se agota.
    pub fn next_sentence(&mut self) -> Option<String> {
        if self.sources.is_empty() || self.paused {
            return None;
        }
        let n = self.sources.len();
        for _ in 0..n {
            let idx = self.current_source % n;
            if !self.sources[idx].sentences.is_empty() {
                let s = self.sources[idx].sentences.pop_front();
                if s.is_some() {
                    self.sources[idx].sentences_consumed += 1;
                }
                return s;
            }
            self.current_source += 1;
        }
        None
    }

    pub fn remaining(&self) -> usize {
        self.sources.iter().map(|s| s.sentences.len()).sum()
    }

    pub fn total_loaded(&self) -> usize {
        self.sources.iter().map(|s| s.total_sentences).sum()
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Update the rolling throughput estimate based on accumulated processing time.
    pub fn update_throughput(&mut self) {
        if self.total_processing_us > 0 && self.total_processed > 0 {
            self.last_throughput =
                (self.total_processed as f32) * 1_000_000.0 / (self.total_processing_us as f32);
        }
    }

    pub fn status(&self) -> String {
        format!(
            "CorpusReader | sources={} | remaining={} | processed={} | concepts+={} | causal+={} | svos+={} | throughput={:.1}/s | paused={}",
            self.sources.len(),
            self.remaining(),
            self.total_processed,
            self.total_concepts_added,
            self.total_causal_pairs,
            self.total_svos,
            self.last_throughput,
            self.paused,
        )
    }
}
