// EDEN GARM SelfAwareness — Metacognicion estructurada.
// 100% Rust puro, 0 LLM, 0 red.
//
// Computa estadisticas estructuradas sobre el estado interno de EDEN para que
// el sistema pueda responder:
//   - Que se bien? (strengths)
//   - Que NO se? (gaps)
//   - Que deberia aprender ahora? (decide)
//
// Antes, las metricas estaban dispersas en cada modulo (n_concepts, target_counts,
// n_inferences, etc.). Aqui se consolidan en un modelo del self que EDEN consulta
// para tomar decisiones.

use crate::eden_garm::capabilities::goal_executor::GoalExecutor;
use crate::eden_garm::capabilities::morphogenesis::ConceptSpace;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Domain {
    Physical,
    Biological,
    Cognitive,
    Computational,
    Social,
    Other,
}

#[derive(Clone, Debug)]
pub struct DomainStats {
    pub domain: Domain,
    pub n_concepts: usize,
    pub n_relations: usize,
    pub density: f32, // relations / concepts
    pub avg_count: f32,
}

#[derive(Clone, Debug)]
pub struct CapabilityStats {
    pub name: String,
    pub n_executions: u64,
    pub n_completions: u64,
    pub success_rate: f32,
}

#[derive(Clone, Debug)]
pub struct SelfAwareness {
    pub domain_keywords: HashMap<Domain, Vec<String>>,
    pub last_introspection_tick: u64,
    pub last_domain_stats: Vec<DomainStats>,
    pub last_capability_stats: Vec<CapabilityStats>,
    pub last_decisions: Vec<String>,
    pub n_introspections: u64,
}

impl SelfAwareness {
    pub fn new() -> Self {
        let mut domain_keywords: HashMap<Domain, Vec<String>> = HashMap::new();

        domain_keywords.insert(
            Domain::Physical,
            vec![
                "gravedad",
                "masa",
                "peso",
                "fuerza",
                "aceleracion",
                "velocidad",
                "momento",
                "calor",
                "temperatura",
                "presion",
                "caida",
                "cae",
                "empuja",
                "atrae",
                "contiene",
                "soporta",
                "pesado",
                "ligero",
                "liquido",
                "solido",
                "agua",
                "fuego",
                "tierra",
                "aire",
                "piedra",
                "metal",
                "sol",
                "luna",
                "planeta",
                "gravity",
                "mass",
                "weight",
                "force",
                "heat",
                "temperature",
                "fall",
                "push",
                "attract",
                "contain",
                "heavy",
                "light",
                "liquid",
                "solid",
                "water",
                "fire",
                "earth",
                "air",
                "stone",
                "metal",
                "sun",
                "planet",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        domain_keywords.insert(
            Domain::Biological,
            vec![
                "celula",
                "organo",
                "tejido",
                "cuerpo",
                "sangre",
                "corazon",
                "pulmon",
                "cerebro",
                "neurona",
                "respira",
                "come",
                "duerme",
                "crece",
                "reproduce",
                "muere",
                "planta",
                "animal",
                "insecto",
                "mamifero",
                "gato",
                "perro",
                "ave",
                "pez",
                "ratificar",
                "arbol",
                "flor",
                "semilla",
                "fotosintesis",
                "oxigeno",
                "cell",
                "organ",
                "tissue",
                "body",
                "blood",
                "heart",
                "lung",
                "brain",
                "neuron",
                "breathe",
                "eat",
                "sleep",
                "grow",
                "reproduce",
                "die",
                "plant",
                "animal",
                "insect",
                "mammal",
                "cat",
                "dog",
                "bird",
                "fish",
                "tree",
                "flower",
                "seed",
                "photosynthesis",
                "oxygen",
                "predator",
                "prey",
                "herbivore",
                "carnivore",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        domain_keywords.insert(
            Domain::Cognitive,
            vec![
                "pensar",
                "piensa",
                "aprender",
                "aprende",
                "aprendi",
                "predecir",
                "predice",
                "planificar",
                "planifica",
                "razonar",
                "razona",
                "entender",
                "entiende",
                "recordar",
                "recuerda",
                "olvidar",
                "olvida",
                "modelo",
                "memoria",
                "conocimiento",
                "experiencia",
                "metas",
                "intencion",
                "creer",
                "creencia",
                "atencion",
                "conciencia",
                "percibir",
                "percepcion",
                "cognicion",
                "mente",
                "think",
                "thinks",
                "learn",
                "learns",
                "predict",
                "plan",
                "reason",
                "reasons",
                "understand",
                "remember",
                "forget",
                "model",
                "memory",
                "knowledge",
                "experience",
                "goal",
                "intent",
                "belief",
                "attention",
                "consciousness",
                "perceive",
                "perception",
                "cognition",
                "mind",
                "train",
                "feedback",
                "loss",
                "gradient",
                "weight",
                "backprop",
                "overfitting",
                "regularization",
                "layer",
                "attention",
                "recurrent",
                "hidden",
                "reinforcement",
                "reward",
                "exploration",
                "exploitation",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        domain_keywords.insert(
            Domain::Computational,
            vec![
                "computador",
                "procesador",
                "programa",
                "instruccion",
                "memoria",
                "disco",
                "red",
                "servidor",
                "cliente",
                "datos",
                "archivo",
                "sistema",
                "algoritmo",
                "cifrar",
                "compilar",
                "interpretar",
                "bug",
                "testing",
                "verificacion",
                "especificacion",
                "computer",
                "processor",
                "program",
                "instruction",
                "disk",
                "network",
                "server",
                "client",
                "data",
                "file",
                "system",
                "algorithm",
                "encrypt",
                "compile",
                "interpret",
                "bug",
                "testing",
                "verification",
                "specification",
                "cache",
                "compiler",
                "emulation",
                "packet",
                "bandwidth",
                "latency",
                "compression",
                "encryption",
                "decryption",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        domain_keywords.insert(
            Domain::Social,
            vec![
                "humano",
                "persona",
                "grupo",
                "equipo",
                "empresa",
                "comunidad",
                "familia",
                "amigos",
                "comunicar",
                "comunica",
                "cooperar",
                "colabora",
                "compartir",
                "ensear",
                "enseña",
                "aprendiz",
                "cultura",
                "sociedad",
                "lenguaje",
                "conversacion",
                "mensaje",
                "human",
                "person",
                "group",
                "team",
                "company",
                "community",
                "family",
                "friends",
                "communicate",
                "cooperate",
                "collaborate",
                "share",
                "teach",
                "apprentice",
                "student",
                "culture",
                "society",
                "language",
                "conversation",
                "message",
                "ideas",
                "share",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        SelfAwareness {
            domain_keywords,
            last_introspection_tick: 0,
            last_domain_stats: Vec::new(),
            last_capability_stats: Vec::new(),
            last_decisions: Vec::new(),
            n_introspections: 0,
        }
    }

    /// Classify a concept label into a domain by counting keyword hits.
    pub fn classify_domain(&self, label: &str) -> Domain {
        let words: HashSet<String> = label
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(String::from)
            .collect();

        let mut best = Domain::Other;
        let mut best_hits = 0;
        for (domain, keywords) in &self.domain_keywords {
            let hits = keywords.iter().filter(|kw| words.contains(*kw)).count();
            if hits > best_hits {
                best_hits = hits;
                best = domain.clone();
            }
        }
        best
    }

    /// Compute stats per domain over all concepts.
    pub fn compute_domain_stats(&mut self, space: &ConceptSpace) -> Vec<DomainStats> {
        let mut by_domain: HashMap<Domain, (usize, usize, u32)> = HashMap::new();
        for c in space.concepts.values() {
            let d = self.classify_domain(&c.label);
            let entry = by_domain.entry(d).or_insert((0, 0, 0));
            entry.0 += 1;
            let n_rel: usize = c.relations.values().map(|v| v.len()).sum();
            entry.1 += n_rel;
            entry.2 += c.count;
        }
        let mut out: Vec<DomainStats> = by_domain
            .into_iter()
            .map(|(d, (n_c, n_r, total_count))| {
                let density = if n_c == 0 {
                    0.0
                } else {
                    n_r as f32 / n_c as f32
                };
                let avg_count = if n_c == 0 {
                    0.0
                } else {
                    total_count as f32 / n_c as f32
                };
                DomainStats {
                    domain: d,
                    n_concepts: n_c,
                    n_relations: n_r,
                    density,
                    avg_count,
                }
            })
            .collect();
        out.sort_by(|a, b| b.n_concepts.cmp(&a.n_concepts));
        self.last_domain_stats = out.clone();
        out
    }

    /// Compute capability stats from goal_executor history.
    pub fn compute_capability_stats(&mut self, exec: &GoalExecutor) -> Vec<CapabilityStats> {
        // Count completions per target
        let mut completions: HashMap<String, u64> = HashMap::new();
        for h in &exec.history {
            if h.completed {
                let key = format!("{:?}", h.action_target);
                *completions.entry(key).or_insert(0) += 1;
            }
        }
        let mut out: Vec<CapabilityStats> = exec
            .target_counts
            .iter()
            .map(|(name, n_exec)| {
                let n_comp = *completions.get(name).unwrap_or(&0);
                let success_rate = if *n_exec == 0 {
                    0.0
                } else {
                    n_comp as f32 / *n_exec as f32
                };
                CapabilityStats {
                    name: name.clone(),
                    n_executions: *n_exec,
                    n_completions: n_comp,
                    success_rate,
                }
            })
            .collect();
        out.sort_by(|a, b| b.n_executions.cmp(&a.n_executions));
        self.last_capability_stats = out.clone();
        out
    }

    /// Full introspection: domain stats + capability stats + tick.
    pub fn introspect(&mut self, space: &ConceptSpace, exec: &GoalExecutor, tick: u64) {
        self.compute_domain_stats(space);
        self.compute_capability_stats(exec);
        self.last_introspection_tick = tick;
        self.n_introspections += 1;
    }

    /// Report strengths: domains with high density, capabilities with high success rate.
    pub fn report_strengths(&self) -> String {
        let mut out = String::from("=== STRENGTHS ===\n");
        out.push_str("\nDominios mas densos:\n");
        let mut by_density = self.last_domain_stats.clone();
        by_density.sort_by(|a, b| {
            b.density
                .partial_cmp(&a.density)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for d in by_density.iter().take(3) {
            out.push_str(&format!(
                "  {:?}: {} conceptos, {} relaciones, densidad={:.2}, avg_count={:.2}\n",
                d.domain, d.n_concepts, d.n_relations, d.density, d.avg_count,
            ));
        }
        out.push_str("\nCapacidades con mejor success rate:\n");
        let mut by_rate = self.last_capability_stats.clone();
        by_rate.sort_by(|a, b| {
            b.success_rate
                .partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for c in by_rate.iter().take(5) {
            if c.n_executions > 0 {
                out.push_str(&format!(
                    "  {}: {}/{} = {:.1}%\n",
                    c.name,
                    c.n_completions,
                    c.n_executions,
                    c.success_rate * 100.0,
                ));
            }
        }
        out
    }

    /// Report gaps: domains with low density, capabilities with low/no success.
    pub fn report_gaps(&self) -> String {
        let mut out = String::from("=== GAPS ===\n");
        out.push_str("\nDominios con baja densidad de conocimiento:\n");
        let mut sorted = self.last_domain_stats.clone();
        sorted.sort_by(|a, b| {
            a.density
                .partial_cmp(&b.density)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for d in sorted.iter().take(3) {
            if d.n_concepts > 0 {
                out.push_str(&format!(
                    "  {:?}: densidad={:.3} ({} conceptos, {} relaciones)\n",
                    d.domain, d.density, d.n_concepts, d.n_relations,
                ));
            }
        }
        out.push_str("\nCapacidades con bajo success rate:\n");
        let mut sorted = self.last_capability_stats.clone();
        sorted.sort_by(|a, b| {
            a.success_rate
                .partial_cmp(&b.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for c in sorted.iter().take(5) {
            if c.n_executions > 0 && c.success_rate < 0.5 {
                out.push_str(&format!(
                    "  {}: {}/{} = {:.1}%\n",
                    c.name,
                    c.n_completions,
                    c.n_executions,
                    c.success_rate * 100.0,
                ));
            }
        }
        out
    }

    /// Decide the next action based on current self-knowledge.
    pub fn decide_next_action(&mut self) -> String {
        let mut out = String::from("=== DECISION ===\n");
        self.last_decisions.clear();

        // Find weakest domain (lowest density, but at least one concept)
        let weakest = self
            .last_domain_stats
            .iter()
            .filter(|d| d.n_concepts > 0)
            .min_by(|a, b| {
                a.density
                    .partial_cmp(&b.density)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(d) = weakest {
            let decision = format!(
                "Recomiendo cargar mas corpus sobre dominio {:?} (densidad={:.3}, {} conceptos sin enlace medio)",
                d.domain, d.density, d.n_concepts
            );
            self.last_decisions.push(decision.clone());
            out.push_str(&format!("  - {}\n", decision));
        }

        // If any capability has 0 executions, recommend exercising it
        let unused: Vec<&CapabilityStats> = self
            .last_capability_stats
            .iter()
            .filter(|c| c.n_executions == 0)
            .collect();
        if !unused.is_empty() {
            let names: Vec<String> = unused.iter().map(|c| c.name.clone()).collect();
            let decision = format!(
                "Capacidades sin ejercitar: {:?}. Recomiendo generar goals que las activen.",
                names
            );
            self.last_decisions.push(decision.clone());
            out.push_str(&format!("  - {}\n", decision));
        }

        // If any capability has very low success rate, recommend reducing reliance
        let weak_caps: Vec<&CapabilityStats> = self
            .last_capability_stats
            .iter()
            .filter(|c| c.n_executions >= 3 && c.success_rate < 0.2)
            .collect();
        if !weak_caps.is_empty() {
            for c in &weak_caps {
                let decision = format!(
                    "{} tiene success rate {:.1}% sobre {} intentos. Considerar mejorar el extractor o reducir su uso.",
                    c.name, c.success_rate * 100.0, c.n_executions
                );
                self.last_decisions.push(decision.clone());
                out.push_str(&format!("  - {}\n", decision));
            }
        }

        // Total knowledge state
        let total_concepts: usize = self.last_domain_stats.iter().map(|d| d.n_concepts).sum();
        let total_relations: usize = self.last_domain_stats.iter().map(|d| d.n_relations).sum();
        let avg_density = if total_concepts == 0 {
            0.0
        } else {
            total_relations as f32 / total_concepts as f32
        };
        out.push_str(&format!(
            "\nEstado global: {} conceptos, {} relaciones, densidad media={:.2}\n",
            total_concepts, total_relations, avg_density,
        ));

        if self.last_decisions.is_empty() {
            self.last_decisions
                .push("No hay deficits claros. Continuar exploracion balanceada.".to_string());
            out.push_str("  - No hay deficits claros. Continuar exploracion balanceada.\n");
        }
        out
    }

    /// Full report: strengths + gaps + decision.
    pub fn full_report(&mut self) -> String {
        let s = self.report_strengths();
        let g = self.report_gaps();
        let d = self.decide_next_action();
        format!("{}\n{}\n{}", s, g, d)
    }

    pub fn status(&self) -> String {
        format!(
            "SelfAwareness | introspections={} | last_tick={} | domains={} | capabilities={}",
            self.n_introspections,
            self.last_introspection_tick,
            self.last_domain_stats.len(),
            self.last_capability_stats.len(),
        )
    }
}
