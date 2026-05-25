// EDEN GARM Curriculum — Auto-aprendizaje dirigido por gaps de dominio.
// 100% Rust puro, 0 LLM, 0 red.
//
// Conecta self_awareness (que detecta dominios debiles) con corpus_reader
// (que ingiere texto). EDEN selecciona que corpus cargar siguiente segun
// que dominio le falta mas densidad de conocimiento.

use crate::eden_garm::capabilities::self_awareness::SelfAwareness;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct CurriculumEngine {
    /// Mapping Domain -> archivo a cargar para ese dominio
    pub domain_files: HashMap<String, String>,
    /// Path donde se busca por defecto
    pub corpus_dir: String,
    pub n_loads: u64,
    pub last_loaded_domain: String,
}

impl CurriculumEngine {
    pub fn new() -> Self {
        let mut domain_files = HashMap::new();
        // Convencion: domain_files["Cognitive"] = "cognitive.txt"
        domain_files.insert("Physical".to_string(), "physical.txt".to_string());
        domain_files.insert("Biological".to_string(), "biological.txt".to_string());
        domain_files.insert("Cognitive".to_string(), "cognitive.txt".to_string());
        domain_files.insert("Computational".to_string(), "computational.txt".to_string());
        domain_files.insert("Social".to_string(), "social.txt".to_string());
        CurriculumEngine {
            domain_files,
            corpus_dir: "corpus/domains".to_string(),
            n_loads: 0,
            last_loaded_domain: String::new(),
        }
    }

    /// Identify weakest domain from self_awareness stats.
    /// Returns (domain_name, density) or None if no stats yet.
    pub fn weakest_domain(awareness: &SelfAwareness) -> Option<(String, f32)> {
        awareness
            .last_domain_stats
            .iter()
            .filter(|d| d.n_concepts >= 5) // domain must have at least some content
            .min_by(|a, b| {
                a.density
                    .partial_cmp(&b.density)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|d| (format!("{:?}", d.domain), d.density))
    }

    /// Identify the file to load for the weakest domain.
    pub fn next_corpus_file(&self, awareness: &SelfAwareness) -> Option<(String, String, f32)> {
        let (domain, density) = Self::weakest_domain(awareness)?;
        let filename = self.domain_files.get(&domain)?.clone();
        let full_path = format!("{}/{}", self.corpus_dir, filename);
        if Path::new(&full_path).exists() {
            Some((domain, full_path, density))
        } else {
            None
        }
    }

    pub fn record_load(&mut self, domain: &str) {
        self.n_loads += 1;
        self.last_loaded_domain = domain.to_string();
    }

    pub fn status(&self) -> String {
        format!(
            "Curriculum | n_loads={} | corpus_dir='{}' | mappings={} | last='{}'",
            self.n_loads,
            self.corpus_dir,
            self.domain_files.len(),
            self.last_loaded_domain,
        )
    }
}
