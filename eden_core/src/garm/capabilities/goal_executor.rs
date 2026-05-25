// EDEN GARM GoalExecutor — Closes the cognitive loop: planned goals trigger real capability invocations.
// 100% Rust puro, 0 LLM, 0 red.
//
// Flow each tick:
//   1. Pull top goal from goal_stack (incomplete, not failed)
//   2. Match goal.label against capability keyword bags
//   3. Best-matching capability is invoked
//   4. Compare state before/after to estimate progress
//   5. complete_goal if delta exceeds threshold, else accumulate progress
//
// Without an executor, plan goal X just stuffs labels into a stack with no consequence.
// With an executor, goals drive behavior — perceiving, learning, exploring — that's the loop.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActionTarget {
    Exploration,   // hippocampus retrieve + temporal plan rare intent
    Metacognition, // self-model training
    Physics,       // physics.simulate_future
    Causality,     // run causal inference / derive transitive
    Memory,        // morphogenesis prune + evolution
    Semantics,     // semantics.compute_embeddings
    Perception,    // vision + world_model
    Goal,          // autonomous action
    NoMatch,
}

#[derive(Clone, Debug)]
pub struct ExecutionResult {
    pub goal_id: u64,
    pub goal_label: String,
    pub action_target: ActionTarget,
    pub match_score: f32,
    pub progress_delta: f32,
    pub completed: bool,
    pub failed: bool,
    pub note: String,
}

#[derive(Clone, Debug)]
pub struct GoalExecutor {
    /// keyword -> capability target
    pub capability_keywords: HashMap<ActionTarget, Vec<String>>,
    /// Counters
    pub n_executions: u64,
    pub n_completions: u64,
    pub n_failures: u64,
    pub n_no_match: u64,
    /// History of last N executions for inspection
    pub history: std::collections::VecDeque<ExecutionResult>,
    pub max_history: usize,
    /// Per-target counts
    pub target_counts: HashMap<String, u64>,
    /// Min match score required to invoke a capability
    pub min_match_score: f32,
    /// Progress delta threshold for considering goal complete
    pub completion_threshold: f32,
}

impl GoalExecutor {
    pub fn new() -> Self {
        let mut capability_keywords: HashMap<ActionTarget, Vec<String>> = HashMap::new();

        capability_keywords.insert(
            ActionTarget::Exploration,
            vec![
                "explorar",
                "explore",
                "explorando",
                "explores",
                "buscar",
                "busca",
                "buscan",
                "search",
                "searches",
                "searching",
                "descubrir",
                "discovers",
                "descubre",
                "descubren",
                "curiosidad",
                "curiosity",
                "novedad",
                "novelty",
                "rare",
                "raro",
                "desconocido",
                "unknown",
                "investigar",
                "investigates",
                "investigando",
                "migra",
                "migrate",
                "migrates",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        capability_keywords.insert(
            ActionTarget::Metacognition,
            vec![
                "aprender",
                "aprende",
                "aprendi",
                "learn",
                "learns",
                "learning",
                "entrenar",
                "entrena",
                "train",
                "trains",
                "training",
                "predecir",
                "predice",
                "predict",
                "predicts",
                "prediction",
                "modelo",
                "model",
                "models",
                "error",
                "equivocar",
                "mistake",
                "olvidar",
                "forgets",
                "remembers",
                "recordar",
                "introspeccion",
                "introspect",
                "reflexion",
                "feedback",
                "retroalimentacion",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        capability_keywords.insert(
            ActionTarget::Physics,
            vec![
                "calienta",
                "calentar",
                "heat",
                "heats",
                "heated",
                "caliente",
                "hot",
                "absorbe",
                "absorbs",
                "absorber",
                "absorption",
                "energia",
                "energy",
                "cae",
                "caer",
                "falls",
                "fall",
                "falling",
                "fell",
                "gravedad",
                "gravity",
                "masa",
                "mass",
                "peso",
                "weight",
                "pesado",
                "heavy",
                "ligero",
                "light",
                "momento",
                "momentum",
                "simulacion",
                "simulate",
                "simulation",
                "fuerza",
                "force",
                "empuja",
                "pushes",
                "atrae",
                "attracts",
                "rebota",
                "bounces",
                "desliza",
                "slides",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        capability_keywords.insert(
            ActionTarget::Causality,
            vec![
                "porque",
                "because",
                "causa",
                "cause",
                "causes",
                "caused",
                "razon",
                "reason",
                "entonces",
                "therefore",
                "then",
                "implica",
                "implies",
                "deriva",
                "derives",
                "infer",
                "inferir",
                "inferencia",
                "inference",
                "consequencia",
                "consequence",
                "efecto",
                "effect",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        capability_keywords.insert(
            ActionTarget::Memory,
            vec![
                "consolidar",
                "consolidate",
                "consolidates",
                "evolucion",
                "evolve",
                "evolves",
                "evolved",
                "recordar",
                "remember",
                "memoria",
                "memory",
                "memories",
                "almacenar",
                "store",
                "stores",
                "stored",
                "podar",
                "prune",
                "forgets",
                "recuperar",
                "recover",
                "retrieves",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        capability_keywords.insert(
            ActionTarget::Semantics,
            vec![
                "lenguaje",
                "language",
                "palabra",
                "word",
                "words",
                "vocabulario",
                "vocabulary",
                "significado",
                "meaning",
                "semantica",
                "semantics",
                "embedding",
                "embeddings",
                "representacion",
                "representation",
                "similar",
                "cercano",
                "nearest",
                "close",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        capability_keywords.insert(
            ActionTarget::Perception,
            vec![
                "ver", "ve", "viendo", "see", "sees", "seeing", "look", "watching", "vision",
                "visual", "percibir", "perceive", "percibes", "objeto", "object", "objects",
                "blob", "blobs", "escena", "scene", "mundo", "world", "rastrear", "track",
                "tracks", "tracking",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        capability_keywords.insert(
            ActionTarget::Goal,
            vec![
                "actuar",
                "act",
                "accion",
                "action",
                "actions",
                "perform",
                "perform",
                "completar",
                "complete",
                "achieve",
                "achieves",
                "alcanzar",
                "reach",
                "reaches",
                "logro",
                "attain",
                "ejecutar",
                "execute",
                "executes",
                "run",
                "runs",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        GoalExecutor {
            capability_keywords,
            n_executions: 0,
            n_completions: 0,
            n_failures: 0,
            n_no_match: 0,
            history: std::collections::VecDeque::with_capacity(100),
            max_history: 100,
            target_counts: HashMap::new(),
            min_match_score: 0.15,
            completion_threshold: 0.05,
        }
    }

    /// Score a goal label against each capability's keyword bag.
    /// Returns the best target and its score in [0, 1].
    pub fn match_goal_to_capability(&self, label: &str) -> (ActionTarget, f32) {
        let label_lower = label.to_lowercase();
        let words: std::collections::HashSet<String> = label_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        let total_words = words.len().max(1) as f32;

        let mut best_target = ActionTarget::NoMatch;
        let mut best_score = 0.0f32;

        for (target, keywords) in &self.capability_keywords {
            let mut hits = 0;
            for kw in keywords {
                if words.contains(kw) {
                    hits += 1;
                }
            }
            // Score = matches / sqrt(label_words). Favors short specific labels but penalizes long irrelevant ones.
            let score = (hits as f32) / total_words.sqrt();
            if score > best_score {
                best_score = score;
                best_target = target.clone();
            }
        }
        (best_target, best_score.min(1.0))
    }

    /// Record an execution in history and counters.
    pub fn record(&mut self, result: ExecutionResult) {
        self.n_executions += 1;
        if result.completed {
            self.n_completions += 1;
        }
        if result.failed {
            self.n_failures += 1;
        }
        let key = format!("{:?}", result.action_target);
        *self.target_counts.entry(key).or_insert(0) += 1;
        self.history.push_back(result);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    pub fn record_no_match(&mut self, goal_id: u64, goal_label: String, score: f32) {
        self.n_no_match += 1;
        self.record(ExecutionResult {
            goal_id,
            goal_label,
            action_target: ActionTarget::NoMatch,
            match_score: score,
            progress_delta: 0.0,
            completed: false,
            failed: false,
            note: "no capability matched above threshold".to_string(),
        });
    }

    pub fn status(&self) -> String {
        let mut targets: Vec<(String, u64)> = self
            .target_counts
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        targets.sort_by(|a, b| b.1.cmp(&a.1));
        let top: Vec<String> = targets
            .into_iter()
            .take(5)
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        format!(
            "GoalExecutor | exec={} | complete={} | fail={} | no_match={} | top: {}",
            self.n_executions,
            self.n_completions,
            self.n_failures,
            self.n_no_match,
            top.join(", ")
        )
    }

    pub fn report_recent(&self, max: usize) -> String {
        if self.history.is_empty() {
            return "GoalExecutor sin historial".to_string();
        }
        let mut out = format!("Ultimas {} ejecuciones:\n", max.min(self.history.len()));
        for r in self.history.iter().rev().take(max) {
            let status_marker = if r.completed {
                "OK"
            } else if r.failed {
                "X"
            } else if r.action_target == ActionTarget::NoMatch {
                "?"
            } else {
                ".."
            };
            out.push_str(&format!(
                "  [{}] gid={} '{}' -> {:?} | match={:.2} | delta={:.3} | {}\n",
                status_marker,
                r.goal_id,
                r.goal_label,
                r.action_target,
                r.match_score,
                r.progress_delta,
                r.note,
            ));
        }
        out
    }
}
