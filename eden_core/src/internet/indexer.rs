//! # Indexer - Search index for crawled content
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};

/// Documento indexado
#[derive(Debug, Clone)]
pub struct IndexedDoc {
    pub id: String,
    pub url: String,
    pub title: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub indexed_at: u64,
}

impl IndexedDoc {
    pub fn new(id: String, url: String, title: String, content: String) -> Self {
        let keywords = Self::extract_keywords(&content);
        Self {
            id,
            url,
            title,
            content,
            keywords,
            indexed_at: current_time_ms(),
        }
    }

    fn extract_keywords(content: &str) -> Vec<String> {
        let mut freq = HashMap::new();

        for word in content.split_whitespace() {
            let word_lower = word.to_lowercase();
            if word_lower.len() > 3 && !Self::is_stopword(&word_lower) {
                *freq.entry(word_lower).or_insert(0) += 1;
            }
        }

        // Top 20 keywords
        let mut pairs: Vec<_> = freq.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.into_iter().take(20).map(|(k, _)| k).collect()
    }

    fn is_stopword(word: &str) -> bool {
        matches!(
            word,
            "the"
                | "and"
                | "for"
                | "with"
                | "this"
                | "that"
                | "from"
                | "have"
                | "has"
                | "was"
                | "were"
        )
    }
}

/// Índice invertido
struct InvertedIndex {
    term_to_docs: HashMap<String, HashSet<String>>,
    doc_term_counts: HashMap<String, HashMap<String, usize>>,
}

impl InvertedIndex {
    fn new() -> Self {
        Self {
            term_to_docs: HashMap::new(),
            doc_term_counts: HashMap::new(),
        }
    }

    fn add_doc(&mut self, doc: &IndexedDoc) {
        for keyword in &doc.keywords {
            self.term_to_docs
                .entry(keyword.clone())
                .or_insert_with(HashSet::new)
                .insert(doc.id.clone());

            let count = doc
                .content
                .to_lowercase()
                .split_whitespace()
                .filter(|w| *w == *keyword)
                .count();

            self.doc_term_counts
                .entry(doc.id.clone())
                .or_insert_with(HashMap::new)
                .insert(keyword.clone(), count);
        }
    }
}

/// Indexador de contenido
pub struct Indexer {
    index: InvertedIndex,
    documents: HashMap<String, IndexedDoc>,
    doc_url_map: HashMap<String, String>, // url -> id
}

impl Indexer {
    pub fn new() -> Self {
        Self {
            index: InvertedIndex::new(),
            documents: HashMap::new(),
            doc_url_map: HashMap::new(),
        }
    }

    /// Indexa un documento
    pub fn index(&mut self, url: &str, title: &str, content: &str) -> String {
        // Generate ID
        let id = format!("doc_{}", self.documents.len());

        let doc = IndexedDoc::new(
            id.clone(),
            url.to_string(),
            title.to_string(),
            content.to_string(),
        );

        // Add to index
        self.index.add_doc(&doc);
        self.documents.insert(id.clone(), doc);
        self.doc_url_map.insert(url.to_string(), id.clone());

        id
    }

    /// Busca documentos
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_terms: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        if query_terms.is_empty() {
            return Vec::new();
        }

        // Collect matching docs and scores
        let mut doc_scores: HashMap<String, f32> = HashMap::new();

        for term in &query_terms {
            if let Some(doc_ids) = self.index.term_to_docs.get(term) {
                for doc_id in doc_ids {
                    let score = self.calculate_score(doc_id, term);
                    *doc_scores.entry(doc_id.clone()).or_insert(0.0) += score;
                }
            }
        }

        // Sort by score
        let mut scored_docs: Vec<_> = doc_scores.into_iter().collect();
        scored_docs.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // Build results
        let mut results = Vec::new();
        for (doc_id, score) in scored_docs {
            if results.len() >= limit {
                break;
            }

            if let Some(doc) = self.documents.get(&doc_id) {
                results.push(SearchResult {
                    url: doc.url.clone(),
                    title: doc.title.clone(),
                    snippet: self.create_snippet(&doc.content, &query_terms),
                    score,
                });
            }
        }

        results
    }

    fn calculate_score(&self, doc_id: &str, term: &str) -> f32 {
        let mut score = 1.0;

        // TF (term frequency)
        if let Some(term_counts) = self.index.doc_term_counts.get(doc_id) {
            if let Some(&count) = term_counts.get(term) {
                score *= 1.0 + (count as f32).log10();
            }
        }

        // IDF (inverse document frequency)
        let doc_count = self.documents.len() as f32;
        let term_doc_count = self
            .index
            .term_to_docs
            .get(term)
            .map(|s| s.len())
            .unwrap_or(0) as f32;
        if term_doc_count > 0.0 {
            score *= (doc_count / term_doc_count).log10() + 1.0;
        }

        score
    }

    fn create_snippet(&self, content: &str, terms: &[String]) -> String {
        let content_lower = content.to_lowercase();
        let mut best_pos = 0;
        let mut best_count = 0;

        for term in terms {
            if let Some(pos) = content_lower.find(term) {
                // Count terms near this position
                let window = &content_lower[pos..(pos + 200).min(content.len())];
                let count = terms.iter().filter(|t| window.contains(*t)).count();
                if count > best_count {
                    best_count = count;
                    best_pos = pos;
                }
            }
        }

        let start = best_pos.saturating_sub(50);
        let end = (best_pos + 150).min(content.len());
        let mut snippet = content[start..end].to_string();

        if start > 0 {
            snippet = format!("...{}", snippet);
        }
        if end < content.len() {
            snippet = format!("{}...", snippet);
        }

        snippet
    }

    /// Obtiene estadísticas del índice
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            doc_count: self.documents.len(),
            term_count: self.index.term_to_docs.len(),
        }
    }
}

impl Default for Indexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Resultado de búsqueda
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
}

/// Estadísticas del índice
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub doc_count: usize,
    pub term_count: usize,
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_and_search() {
        let mut indexer = Indexer::new();
        indexer.index(
            "http://example.com",
            "Example",
            "This is an example document about programming",
        );
        indexer.index("http://test.com", "Test", "Another test document");

        let results = indexer.search("example", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].url, "http://example.com");
    }
}
