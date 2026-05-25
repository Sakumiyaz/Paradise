// EDEN GARM Grounding — High-fidelity world model representations.
// Closes the gap between symbolic causal knowledge (from text) and the perceptual
// systems (physics, world_model). 100% Rust puro, 0 LLM, 0 red.
//
// Three capabilities:
//   1. Extract physical claims from text (gravity, mass, attraction, fall, heat, pressure)
//   2. Apply those claims to the physics engine (gravity vector, learned masses, etc.)
//   3. Cross-modal binding: link text concepts to visual blobs when label matches
//
// The grounding module is what makes EDEN's text knowledge "real" - it doesn't just
// know that "gravity attracts objects", it updates its physics simulator accordingly.

use crate::eden_garm::capabilities::morphogenesis::ConceptSpace;
use crate::eden_garm::capabilities::physics::PhysicsEngine;
use crate::eden_garm::capabilities::world_model::WorldModel;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum PhysicalClaim {
    /// X attracts Y (gravitational/electromagnetic-like)
    Attracts { source: String, target: String },
    /// X has mass M (or is heavy/light)
    HasMass { entity: String, magnitude: f32 },
    /// X falls (under gravity)
    Falls { entity: String },
    /// X is hot/cold (temperature claim)
    Temperature { entity: String, value: f32 },
    /// X pushes/exerts force on Y
    Pushes {
        source: String,
        target: String,
        magnitude: f32,
    },
    /// X contains Y (containment relation)
    Contains { container: String, contents: String },
    /// X is supported by Y
    SupportedBy { entity: String, support: String },
    /// X causes Y to accelerate
    Accelerates { entity: String },
    /// X has property P (generic physical property)
    HasProperty { entity: String, property: String },
}

#[derive(Clone, Debug)]
pub struct PhysicalFact {
    pub claim: PhysicalClaim,
    pub source_sentence: String,
    pub tick_learned: u64,
    pub confidence: f32,
}

#[derive(Clone, Debug)]
pub struct GroundingEngine {
    /// Words that signal a physical claim
    pub physical_keywords: HashSet<String>,
    /// Verbs indicating attraction / gravity-like force
    pub attraction_verbs: HashSet<String>,
    /// Verbs indicating falling / downward motion
    pub fall_verbs: HashSet<String>,
    /// Mass-related words
    pub mass_words: HashSet<String>,
    /// Temperature-related words
    pub temp_words: HashSet<String>,
    /// All facts learned from text
    pub facts: Vec<PhysicalFact>,
    /// Mapping concept_id -> physics_object_id (text -> physics binding)
    pub concept_to_physics: HashMap<u64, u64>,
    /// Mapping concept_id -> world_model_object_id (text -> visual binding, single-blob)
    pub concept_to_visual: HashMap<u64, u64>,
    /// Multi-blob bindings: concept_id -> all visual ids associated with it
    pub concept_to_visuals: HashMap<u64, std::collections::HashSet<u64>>,
    /// Concept ids classified as physical (have physical_keyword in label)
    pub physical_concepts: HashSet<u64>,
    /// Counters
    pub n_facts_extracted: u64,
    pub n_physics_updates: u64,
    pub n_text_visual_bindings: u64,
    pub n_text_physics_bindings: u64,
    /// Learned magnitude of gravity from text co-occurrences
    pub gravity_evidence_count: u32,
}

impl GroundingEngine {
    pub fn new() -> Self {
        let physical_keywords: HashSet<String> = [
            // ES
            "gravedad",
            "masa",
            "peso",
            "fuerza",
            "aceleracion",
            "velocidad",
            "momento",
            "energia",
            "calor",
            "temperatura",
            "presion",
            "caida",
            "cae",
            "cae",
            "empuja",
            "atrae",
            "atrayendo",
            "atraccion",
            "contiene",
            "contenedor",
            "soporta",
            "soporte",
            "sobre",
            "encima",
            "debajo",
            "pesado",
            "ligero",
            "duro",
            "blando",
            "liquido",
            "solido",
            "gas",
            "fluye",
            "flota",
            "hunde",
            "quema",
            "frio",
            "caliente",
            "tibio",
            // EN
            "gravity",
            "mass",
            "weight",
            "force",
            "acceleration",
            "velocity",
            "momentum",
            "energy",
            "heat",
            "temperature",
            "pressure",
            "fall",
            "falls",
            "push",
            "pushes",
            "attract",
            "attracts",
            "attraction",
            "contain",
            "contains",
            "support",
            "supports",
            "on",
            "above",
            "below",
            "heavy",
            "light",
            "hard",
            "soft",
            "liquid",
            "solid",
            "gas",
            "flow",
            "flows",
            "float",
            "sink",
            "burn",
            "burns",
            "cold",
            "hot",
            "warm",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let attraction_verbs: HashSet<String> = [
            "atrae",
            "atraen",
            "atrayendo",
            "jala",
            "tira",
            "attract",
            "attracts",
            "attracted",
            "pulls",
            "pulled",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let fall_verbs: HashSet<String> = [
            "cae", "caen", "cayo", "caera", "cayendo", "fall", "falls", "fell", "falling", "drops",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let mass_words: HashSet<String> = [
            "masa", "peso", "pesa", "pesado", "ligero", "heavy", "mass", "weight", "weighs",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let temp_words: HashSet<String> = [
            "calor",
            "caliente",
            "frio",
            "tibio",
            "temperatura",
            "heat",
            "hot",
            "cold",
            "warm",
            "temperature",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        GroundingEngine {
            physical_keywords,
            attraction_verbs,
            fall_verbs,
            mass_words,
            temp_words,
            facts: Vec::new(),
            concept_to_physics: HashMap::new(),
            concept_to_visual: HashMap::new(),
            concept_to_visuals: HashMap::new(),
            physical_concepts: HashSet::new(),
            n_facts_extracted: 0,
            n_physics_updates: 0,
            n_text_visual_bindings: 0,
            n_text_physics_bindings: 0,
            gravity_evidence_count: 0,
        }
    }

    /// Returns true if a sentence contains any physical keyword.
    pub fn is_physical(&self, sentence: &str) -> bool {
        let lower = sentence.to_lowercase();
        for kw in &self.physical_keywords {
            // Whole-word match using word boundaries
            for w in lower.split(|c: char| !c.is_alphanumeric()) {
                if w == kw {
                    return true;
                }
            }
        }
        false
    }

    /// Tokenize words from a sentence (lowercase alphanumeric only).
    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(|w| w.to_string())
            .collect()
    }

    /// Extract physical claims from one sentence using keyword + structural patterns.
    /// This is heuristic but catches the most common forms.
    pub fn extract_physical_facts(&self, sentence: &str, tick: u64) -> Vec<PhysicalFact> {
        let mut out = Vec::new();
        let tokens = Self::tokenize(sentence);
        if tokens.len() < 2 {
            return out;
        }

        // Pattern 1: ATTRACTION - "X attracts Y" / "X atrae Y"
        for (i, tok) in tokens.iter().enumerate() {
            if self.attraction_verbs.contains(tok) {
                let source = tokens[..i]
                    .iter()
                    .filter(|w| !w.is_empty() && w.len() > 2)
                    .last()
                    .cloned()
                    .unwrap_or_default();
                let target = tokens.get(i + 1).cloned().unwrap_or_default();
                if !source.is_empty() && !target.is_empty() {
                    out.push(PhysicalFact {
                        claim: PhysicalClaim::Attracts {
                            source: source.clone(),
                            target: target.clone(),
                        },
                        source_sentence: sentence.to_string(),
                        tick_learned: tick,
                        confidence: 0.7,
                    });
                }
            }
        }

        // Pattern 2: FALL - "X falls" / "X cae"
        for (i, tok) in tokens.iter().enumerate() {
            if self.fall_verbs.contains(tok) {
                let entity = tokens[..i]
                    .iter()
                    .filter(|w| !w.is_empty() && w.len() > 2)
                    .last()
                    .cloned()
                    .unwrap_or_default();
                if !entity.is_empty() {
                    out.push(PhysicalFact {
                        claim: PhysicalClaim::Falls { entity },
                        source_sentence: sentence.to_string(),
                        tick_learned: tick,
                        confidence: 0.6,
                    });
                }
            }
        }

        // Pattern 3: MASS - "X tiene masa" / "X has mass" / "X is heavy"
        for (i, tok) in tokens.iter().enumerate() {
            if self.mass_words.contains(tok) {
                let entity = tokens[..i]
                    .iter()
                    .filter(|w| !w.is_empty() && w.len() > 2)
                    .last()
                    .cloned()
                    .unwrap_or_default();
                let magnitude = if tok == "heavy" || tok == "pesado" {
                    2.0
                } else if tok == "light" || tok == "ligero" {
                    0.5
                } else {
                    1.0
                };
                if !entity.is_empty() {
                    out.push(PhysicalFact {
                        claim: PhysicalClaim::HasMass { entity, magnitude },
                        source_sentence: sentence.to_string(),
                        tick_learned: tick,
                        confidence: 0.5,
                    });
                }
            }
        }

        // Pattern 4: TEMPERATURE - "X is hot/cold" / "X esta caliente/frio"
        for (i, tok) in tokens.iter().enumerate() {
            if self.temp_words.contains(tok) && tok != "temperatura" && tok != "temperature" {
                let entity = tokens[..i]
                    .iter()
                    .filter(|w| !w.is_empty() && w.len() > 2)
                    .last()
                    .cloned()
                    .unwrap_or_default();
                let value = if tok == "hot" || tok == "caliente" {
                    0.8
                } else if tok == "cold" || tok == "frio" {
                    0.2
                } else {
                    0.5
                };
                if !entity.is_empty() {
                    out.push(PhysicalFact {
                        claim: PhysicalClaim::Temperature { entity, value },
                        source_sentence: sentence.to_string(),
                        tick_learned: tick,
                        confidence: 0.5,
                    });
                }
            }
        }

        // Pattern 5: CONTAINMENT - "X contiene Y" / "X contains Y"
        for (i, tok) in tokens.iter().enumerate() {
            if tok == "contiene" || tok == "contienen" || tok == "contains" || tok == "contain" {
                let container = tokens[..i]
                    .iter()
                    .filter(|w| !w.is_empty() && w.len() > 2)
                    .last()
                    .cloned()
                    .unwrap_or_default();
                let contents = tokens
                    .get(i + 1)
                    .filter(|w| w.len() > 2)
                    .cloned()
                    .or_else(|| tokens.get(i + 2).filter(|w| w.len() > 2).cloned())
                    .unwrap_or_default();
                if !container.is_empty() && !contents.is_empty() {
                    out.push(PhysicalFact {
                        claim: PhysicalClaim::Contains {
                            container,
                            contents,
                        },
                        source_sentence: sentence.to_string(),
                        tick_learned: tick,
                        confidence: 0.6,
                    });
                }
            }
        }

        // Pattern 6: SUPPORT - "X soporta Y" / "Y is on X" / "Y esta sobre X"
        for (i, tok) in tokens.iter().enumerate() {
            if tok == "sobre" || tok == "encima" || tok == "on" {
                let entity = tokens[..i]
                    .iter()
                    .filter(|w| !w.is_empty() && w.len() > 2)
                    .last()
                    .cloned()
                    .unwrap_or_default();
                let support = tokens
                    .get(i + 1)
                    .filter(|w| w.len() > 2)
                    .cloned()
                    .unwrap_or_default();
                if !entity.is_empty() && !support.is_empty() {
                    out.push(PhysicalFact {
                        claim: PhysicalClaim::SupportedBy { entity, support },
                        source_sentence: sentence.to_string(),
                        tick_learned: tick,
                        confidence: 0.5,
                    });
                }
            }
        }

        out
    }

    /// Apply a list of physical facts to the physics engine.
    /// Returns the number of physics state updates performed.
    pub fn apply_facts_to_physics(
        &mut self,
        facts: &[PhysicalFact],
        physics: &mut PhysicsEngine,
    ) -> usize {
        let mut updates = 0usize;
        for fact in facts {
            self.facts.push(fact.clone());
            self.n_facts_extracted += 1;
            match &fact.claim {
                PhysicalClaim::Attracts { source, target: _ } => {
                    // If "gravity" or "gravedad" is the source, increase gravity magnitude evidence
                    if source.contains("gravedad") || source.contains("gravity") {
                        self.gravity_evidence_count += 1;
                        // Slightly strengthen gravity downward each time text confirms it
                        physics.gravity.y = (physics.gravity.y + 0.005 * fact.confidence).min(0.3);
                        self.n_physics_updates += 1;
                        updates += 1;
                    }
                }
                PhysicalClaim::Falls { entity: _ } => {
                    // Reinforce gravity: things that fall are evidence of downward gravity
                    self.gravity_evidence_count += 1;
                    physics.gravity.y = (physics.gravity.y + 0.002 * fact.confidence).min(0.3);
                    self.n_physics_updates += 1;
                    updates += 1;
                }
                PhysicalClaim::HasMass { entity, magnitude } => {
                    // If a physics object's label matches the entity, set its mass
                    let mut updated = false;
                    for obj in physics.objects.values_mut() {
                        if let Some(label) = &obj.label {
                            if label.to_lowercase().contains(entity) {
                                obj.mass_observations.push(*magnitude);
                                if obj.mass_observations.len() > 20 {
                                    obj.mass_observations.remove(0);
                                }
                                obj.mass = obj.mass_observations.iter().sum::<f32>()
                                    / obj.mass_observations.len() as f32;
                                updated = true;
                            }
                        }
                    }
                    if updated {
                        self.n_physics_updates += 1;
                        updates += 1;
                    }
                }
                PhysicalClaim::Temperature { .. } => {
                    // No direct physics field for temperature yet, but could inform future expansion
                }
                PhysicalClaim::Pushes { .. } => {}
                PhysicalClaim::Contains { .. } => {}
                PhysicalClaim::SupportedBy { entity, support } => {
                    // Try to match physics objects and set supported_by
                    let mut entity_id = None;
                    let mut support_id = None;
                    for obj in physics.objects.values() {
                        if let Some(label) = &obj.label {
                            let l = label.to_lowercase();
                            if l.contains(entity) {
                                entity_id = Some(obj.id);
                            }
                            if l.contains(support) {
                                support_id = Some(obj.id);
                            }
                        }
                    }
                    if let (Some(e), Some(s)) = (entity_id, support_id) {
                        if let Some(obj) = physics.objects.get_mut(&e) {
                            obj.supported_by = Some(s);
                            self.n_physics_updates += 1;
                            updates += 1;
                        }
                    }
                }
                PhysicalClaim::Accelerates { .. } => {}
                PhysicalClaim::HasProperty { .. } => {}
            }
        }
        updates
    }

    /// Mark a concept as physical if its label contains a physical keyword.
    pub fn classify_concept_as_physical(&mut self, concept_id: u64, label: &str) -> bool {
        let lower = label.to_lowercase();
        for w in lower.split(|c: char| !c.is_alphanumeric()) {
            if self.physical_keywords.contains(w) {
                self.physical_concepts.insert(concept_id);
                return true;
            }
        }
        false
    }

    /// Cross-modal binding: when a TrackedObject in world_model has a label that
    /// matches a known concept label, create a binding between them.
    pub fn bind_text_to_visual(&mut self, world: &WorldModel, space: &ConceptSpace) -> usize {
        let mut new_bindings = 0usize;
        for obj in world.objects.values() {
            if let Some(visual_label) = &obj.label {
                let vl = visual_label.to_lowercase();
                for c in space.concepts.values() {
                    if self.concept_to_visual.contains_key(&c.id) {
                        continue;
                    }
                    let cl = c.label.to_lowercase();
                    // Match if visual label is a substring of concept label OR vice versa
                    if cl.contains(&vl) || vl.contains(&cl) {
                        self.concept_to_visual.insert(c.id, obj.id);
                        self.n_text_visual_bindings += 1;
                        new_bindings += 1;
                        break;
                    }
                }
            }
        }
        new_bindings
    }

    /// Bind text concepts to physics objects via label matching.
    pub fn bind_text_to_physics(&mut self, physics: &PhysicsEngine, space: &ConceptSpace) -> usize {
        let mut new_bindings = 0usize;
        for obj in physics.objects.values() {
            if let Some(physics_label) = &obj.label {
                let pl = physics_label.to_lowercase();
                for c in space.concepts.values() {
                    if self.concept_to_physics.contains_key(&c.id) {
                        continue;
                    }
                    let cl = c.label.to_lowercase();
                    if cl.contains(&pl) || pl.contains(&cl) {
                        self.concept_to_physics.insert(c.id, obj.id);
                        self.n_text_physics_bindings += 1;
                        new_bindings += 1;
                        break;
                    }
                }
            }
        }
        new_bindings
    }

    pub fn status(&self) -> String {
        format!(
            "Grounding | facts={} | physics_updates={} | gravity_evidence={} | physical_concepts={} | bindings: text->visual={}, text->physics={}",
            self.facts.len(),
            self.n_physics_updates,
            self.gravity_evidence_count,
            self.physical_concepts.len(),
            self.n_text_visual_bindings,
            self.n_text_physics_bindings,
        )
    }

    /// Detailed report of learned physical facts.
    pub fn report_facts(&self, max: usize) -> String {
        let mut out = format!("Grounding facts learned: {} total\n", self.facts.len());
        for fact in self.facts.iter().take(max) {
            let claim_str = match &fact.claim {
                PhysicalClaim::Attracts { source, target } => {
                    format!("ATTRACTS: {} -> {}", source, target)
                }
                PhysicalClaim::Falls { entity } => format!("FALLS: {}", entity),
                PhysicalClaim::HasMass { entity, magnitude } => {
                    format!("MASS: {} = {:.2}", entity, magnitude)
                }
                PhysicalClaim::Temperature { entity, value } => {
                    format!("TEMP: {} = {:.2}", entity, value)
                }
                PhysicalClaim::Pushes {
                    source,
                    target,
                    magnitude,
                } => format!("PUSHES: {} -> {} ({:.2})", source, target, magnitude),
                PhysicalClaim::Contains {
                    container,
                    contents,
                } => format!("CONTAINS: {} <- {}", container, contents),
                PhysicalClaim::SupportedBy { entity, support } => {
                    format!("SUPPORTED: {} on {}", entity, support)
                }
                PhysicalClaim::Accelerates { entity } => format!("ACCEL: {}", entity),
                PhysicalClaim::HasProperty { entity, property } => {
                    format!("PROP: {} = {}", entity, property)
                }
            };
            out.push_str(&format!(
                "  [t={}] {} (conf={:.2})\n",
                fact.tick_learned, claim_str, fact.confidence
            ));
        }
        out
    }
}
