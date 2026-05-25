//! GARM Node: Capability — Generic wrapper for any GarmCapabilityState capability
//!
//! This node fragments the monolithic GarmCapabilityState::tick() into individual
//! executable units. Each capability gets its own node, metabolismo,
//! y free energy en el grafo GARM.

use crate::eden_garm::capabilities::{GarmCapability, GarmCapabilityState};
use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Imports for moved tick() logic
use crate::eden_garm::capabilities::{
    compositional, continuous, corpus_reader, experience_buffer, exploration, goal_executor,
    gridworld, hippocampus, inference, meta_evolution, morphogenesis, nlp, predictive_loop,
    program, self_improvement, social_complex, tools, unified_bus, vision,
};

/// Identificador de cada capability de GarmCapabilityState.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityId {
    RecurrentState,
    Homeostasis,
    CorpusProcessing,
    BigTransformerTrain,
    BigTransformerGenerate,
    SemanticsObserve,
    SyntaxParse,
    SceneParser,
    Morphogenesis,
    Causality,
    Grounding,
    Physics,
    Hippocampus,
    Mood,
    Motivation,
    GoalStack,
    Planner,
    Security,
    Neural,
    TransformerSmall,
    BusPredictor,
    WorldModelNN,
    MoE,
    HierarchicalAttention,
    ContinualLearning,
    MetaLearning,
    EWC,
    MDLPruner,
    EmotionalModulation,
    DNC,
    ActiveInference,
    Body,
    TemporalHierarchy,
    SelfModification,
    LogicReasoning,
    ConstitutionalSafety,
    Phenomenology,
    EconomicAgent,
    RewardOracle,
    BPTT,
    CorpusMassive,
    GenController,
    SocialComplex,
    MultiAgent,
    Swarm,
    Metacognition,
    SelfAwareness,
    IntentionHierarchy,
    Exploration,
    Gate,
    Evidence,
    Surprise,
    Epistemic,
    Circadian,
    Critic,
    WorkingMemory,
    ProgramInduction,
    Counterfactual,
    Analogy,
    Composition,
    Autonomy,
    GoalExecutor,
    LanguageGen,
    SyntheticVision,
    PredictiveLoop,
    Curriculum,
    MemoryClustering,
    Gridworld,
    AgentMesh,
    Compositional,
    NeuralExtractors,
    World3D,
    PluginSystem,
    UnifiedPerception,
    UnifiedBus,
    ArchitectureModel,
    AutoDebug,
    OpenEndedness,
    Evolution,
    SelfModel,
    Temporal,
    TheoryOfMind,
    InternalLanguage,
    Perception,
    Sandbox,
    ComputerUse,
    ToolCalling,
    McpClient,
    Voice,
    Vision,
    NaturalLanguage,
}

/// Nodo genérico que envuelve una capability de GarmCapabilityState.
pub struct CapabilityNode {
    id: usize,
    cap: CapabilityId,
    engine: Arc<Mutex<GarmCapabilityState>>,
    internal_fe: f32,
    tick_interval: f32,
    tick_accumulator: f32,
    last_cost: f32,
    n_executions: u64,
}

impl CapabilityNode {
    pub fn new(id: usize, cap: CapabilityId, engine: Arc<Mutex<GarmCapabilityState>>) -> Self {
        CapabilityNode {
            id,
            cap,
            engine,
            internal_fe: 1.0,
            tick_interval: 1.0,
            tick_accumulator: 0.0,
            last_cost: 5.0,
            n_executions: 0,
        }
    }

    fn name_for_cap(cap: CapabilityId) -> &'static str {
        match cap {
            CapabilityId::RecurrentState => "recurrent_state",
            CapabilityId::Homeostasis => "homeostasis",
            CapabilityId::CorpusProcessing => "corpus_processing",
            CapabilityId::BigTransformerTrain => "big_transformer_train",
            CapabilityId::BigTransformerGenerate => "big_transformer_generate",
            CapabilityId::SemanticsObserve => "semantics_observe",
            CapabilityId::SyntaxParse => "syntax_parse",
            CapabilityId::SceneParser => "scene_parser",
            CapabilityId::Morphogenesis => "morphogenesis",
            CapabilityId::Causality => "causality",
            CapabilityId::Grounding => "grounding",
            CapabilityId::Physics => "physics",
            CapabilityId::Hippocampus => "hippocampus",
            CapabilityId::Mood => "mood",
            CapabilityId::Motivation => "motivation",
            CapabilityId::GoalStack => "goal_stack",
            CapabilityId::Planner => "planner",
            CapabilityId::Security => "security",
            CapabilityId::Neural => "neural",
            CapabilityId::TransformerSmall => "transformer_small",
            CapabilityId::BusPredictor => "bus_predictor",
            CapabilityId::WorldModelNN => "world_model_nn",
            CapabilityId::MoE => "moe",
            CapabilityId::HierarchicalAttention => "hierarchical_attention",
            CapabilityId::ContinualLearning => "continual_learning",
            CapabilityId::MetaLearning => "meta_learning",
            CapabilityId::EWC => "ewc",
            CapabilityId::MDLPruner => "mdl_pruner",
            CapabilityId::EmotionalModulation => "emotional_modulation",
            CapabilityId::DNC => "dnc",
            CapabilityId::ActiveInference => "active_inference",
            CapabilityId::Body => "body",
            CapabilityId::TemporalHierarchy => "temporal_hierarchy",
            CapabilityId::SelfModification => "self_modification",
            CapabilityId::LogicReasoning => "logic_reasoning",
            CapabilityId::ConstitutionalSafety => "constitutional_safety",
            CapabilityId::Phenomenology => "phenomenology",
            CapabilityId::EconomicAgent => "economic_agent",
            CapabilityId::RewardOracle => "reward_oracle",
            CapabilityId::BPTT => "bptt",
            CapabilityId::CorpusMassive => "corpus_massive",
            CapabilityId::GenController => "gen_controller",
            CapabilityId::SocialComplex => "social_complex",
            CapabilityId::MultiAgent => "multi_agent",
            CapabilityId::Swarm => "swarm",
            CapabilityId::Metacognition => "metacognition",
            CapabilityId::SelfAwareness => "self_awareness",
            CapabilityId::IntentionHierarchy => "intention_hierarchy",
            CapabilityId::Exploration => "exploration",
            CapabilityId::Gate => "gate",
            CapabilityId::Evidence => "evidence",
            CapabilityId::Surprise => "surprise",
            CapabilityId::Epistemic => "epistemic",
            CapabilityId::Circadian => "circadian",
            CapabilityId::Critic => "critic",
            CapabilityId::WorkingMemory => "working_memory",
            CapabilityId::ProgramInduction => "program_induction",
            CapabilityId::Counterfactual => "counterfactual",
            CapabilityId::Analogy => "analogy",
            CapabilityId::Composition => "composition",
            CapabilityId::Autonomy => "autonomy",
            CapabilityId::GoalExecutor => "goal_executor",
            CapabilityId::LanguageGen => "language_gen",
            CapabilityId::SyntheticVision => "synthetic_vision",
            CapabilityId::PredictiveLoop => "predictive_loop",
            CapabilityId::Curriculum => "curriculum",
            CapabilityId::MemoryClustering => "memory_clustering",
            CapabilityId::Gridworld => "gridworld",
            CapabilityId::AgentMesh => "agent_mesh",
            CapabilityId::Compositional => "compositional",
            CapabilityId::NeuralExtractors => "neural_extractors",
            CapabilityId::World3D => "world3d",
            CapabilityId::PluginSystem => "plugin_system",
            CapabilityId::UnifiedPerception => "unified_perception",
            CapabilityId::UnifiedBus => "unified_bus",
            CapabilityId::ArchitectureModel => "architecture_model",
            CapabilityId::AutoDebug => "auto_debug",
            CapabilityId::OpenEndedness => "open_endedness",
            CapabilityId::Evolution => "evolution",
            CapabilityId::SelfModel => "self_model",
            CapabilityId::Temporal => "temporal",
            CapabilityId::TheoryOfMind => "theory_of_mind",
            CapabilityId::InternalLanguage => "internal_language",
            CapabilityId::Perception => "perception",
            CapabilityId::Sandbox => "sandbox",
            CapabilityId::ComputerUse => "computer_use",
            CapabilityId::ToolCalling => "tool_calling",
            CapabilityId::McpClient => "mcp_client",
            CapabilityId::Voice => "voice",
            CapabilityId::Vision => "vision",
            CapabilityId::NaturalLanguage => "natural_language",
        }
    }

    fn execute(&mut self) -> f32 {
        let mut guard = self.engine.lock().unwrap();
        let cost = guard.dispatch_capability(self.cap);
        drop(guard);
        self.n_executions += 1;
        cost
    }
}

impl crate::eden_garm::capabilities::GarmCapabilityState {
    pub fn dispatch_capability(&mut self, cap: CapabilityId) -> f32 {
        let cost = match cap {
            CapabilityId::RecurrentState => {
                // === NUEVA ARQUITECTURA: Recurrent State persistente entre ticks ===
                self.recurrent_state.step(&self.slots_to_vector());

                // === NUEVA ARQUITECTURA: Recurrent State status ===
                if self.state.tick_count % 40 == 0 {
                    self.action_log
                        .push(format!("[RECURRENT] {}", self.recurrent_state.status()));
                }

                1.0
            }
            CapabilityId::Homeostasis => {
                // === NUEVA ARQUITECTURA: Homeostasis multi-objetivo ===
                self.homeostasis.tick();
                if self.homeostasis.imbalance() > 40.0 {
                    let (need, deficit) = self.homeostasis.most_needed();
                    self.action_log.push(format!(
                        "[HOMEOSTASIS] CRITICAL need={} deficit={:.1}",
                        need, deficit
                    ));
                }

                2.0
            }
            CapabilityId::CorpusProcessing => {
                // 0. CORPUS BACKGROUND PROCESSING: feed generalization primitives with text
                // Each self.state.tick_count consumes a few sentences and runs the full generalization pipeline.
                let corpus_start = std::time::Instant::now();
                let n_to_process = self.corpus_reader.sentences_per_tick;
                let mut corpus_concepts = 0u64;
                let mut corpus_causal = 0u64;
                let mut corpus_svos = 0u64;
                for _ in 0..n_to_process {
                    let sentence = match self.corpus_reader.next_sentence() {
                        Some(s) => s,
                        None => break,
                    };
                    // 1. Tokenize via existing nlp module
                    let tokens = nlp::tokenize(&sentence);
                    if tokens.len() < 2 {
                        continue;
                    }
                    // 2. Feed semantics co-occurrence model
                    self.semantics.observe(&sentence);
                    self.semantics.tick_since_compute += 1;
                    if self.semantics.tick_since_compute >= self.semantics.compute_every {
                        self.semantics.compute_embeddings();
                        self.semantics.tick_since_compute = 0;
                    }
                    // 2b. Neural extractors: train online from heuristic labels
                    let sent_emb = self.semantics.sentence_embedding(&sentence);
                    if !sent_emb.is_empty() {
                        self.neural_extractors
                            .train_from_heuristics(&sentence, &sent_emb);
                    }
                    // 2c. Transformer: build embeddings from semantics once, then train on each sentence
                    if self.transformer.vocab_size == 0 && !self.semantics.index_to_word.is_empty()
                    {
                        self.transformer.build_embeddings_from_semantics(
                            &self.semantics.index_to_word,
                            &self.semantics.embeddings,
                        );
                    }
                    if self.transformer.vocab_size > 0 {
                        let tok_indices: Vec<usize> = tokens
                            .iter()
                            .map(|t| {
                                self.semantics
                                    .vocab
                                    .get(t)
                                    .copied()
                                    .unwrap_or(self.semantics.vocab_size)
                            })
                            .collect();
                        if tok_indices.len() >= 2 {
                            let loss = self.transformer.train_on_sentence(&tok_indices);
                            if loss > 0.0 && corpus_start.elapsed().as_millis() % 100 == 0 {
                                // Only report occasionally to avoid spam
                            }
                        }
                    }
                    // 3. Syntax parse for SVO + causal templates
                    let dep = self.syntax.parse(&tokens);
                    if let Some((subj, verb, obj)) = self.syntax.extract_svo(&dep) {
                        if !subj.is_empty() && !verb.is_empty() && !obj.is_empty() {
                            corpus_svos += 1;
                        }
                    }
                    // 4. Causal pair extraction (both syntax and scene parser, deduplicated)
                    let mut all_causal = self.syntax.extract_causal_compound(&dep);
                    all_causal.extend(self.scene_parser.extract_causal(&tokens));
                    all_causal.sort();
                    all_causal.dedup();
                    let dim = self.semantics.embed_dim.max(1);
                    for (cause, effect) in &all_causal {
                        // Use scene_vector for richer structural embeddings (agent+action+patient+context)
                        let cause_tokens = nlp::tokenize(cause);
                        let effect_tokens = nlp::tokenize(effect);
                        let cause_scene =
                            self.scene_parser.parse(&cause_tokens, &self.semantics, dim);
                        let effect_scene =
                            self.scene_parser
                                .parse(&effect_tokens, &self.semantics, dim);
                        let cause_emb = cause_scene.flatten();
                        let effect_emb = effect_scene.flatten();
                        if cause_emb.is_empty() || effect_emb.is_empty() {
                            continue;
                        }

                        // Deduplicate by normalized phrase: identical phrases reuse the same concept_id
                        // instead of relying on geometric clustering (which collapses too aggressively).
                        let cause_key = corpus_reader::CorpusReader::normalize_phrase(cause);
                        let effect_key = corpus_reader::CorpusReader::normalize_phrase(effect);
                        let (cid, c_new) = if let Some(&id) =
                            self.corpus_reader.phrase_to_concept.get(&cause_key)
                        {
                            (id, false)
                        } else {
                            let id = self.morphogenesis.next_id;
                            self.morphogenesis.next_id += 1;
                            let label = if cause.len() > 60 {
                                cause[..60].to_string()
                            } else {
                                cause.clone()
                            };
                            self.morphogenesis.concepts.insert(
                                id,
                                morphogenesis::Concept {
                                    id,
                                    centroid: cause_emb.clone(),
                                    label,
                                    count: 1,
                                    birth_tick: self.state.tick_count,
                                    tension_accumulated: 0.0,
                                    parent_id: None,
                                    children: Vec::new(),
                                    relations: std::collections::HashMap::new(),
                                    abstraction_level: 0,
                                    properties: std::collections::HashMap::new(),
                                },
                            );
                            self.corpus_reader.phrase_to_concept.insert(cause_key, id);
                            (id, true)
                        };
                        let (eid, e_new) = if let Some(&id) =
                            self.corpus_reader.phrase_to_concept.get(&effect_key)
                        {
                            (id, false)
                        } else {
                            let id = self.morphogenesis.next_id;
                            self.morphogenesis.next_id += 1;
                            let label = if effect.len() > 60 {
                                effect[..60].to_string()
                            } else {
                                effect.clone()
                            };
                            self.morphogenesis.concepts.insert(
                                id,
                                morphogenesis::Concept {
                                    id,
                                    centroid: effect_emb.clone(),
                                    label,
                                    count: 1,
                                    birth_tick: self.state.tick_count,
                                    tension_accumulated: 0.0,
                                    parent_id: None,
                                    children: Vec::new(),
                                    relations: std::collections::HashMap::new(),
                                    abstraction_level: 0,
                                    properties: std::collections::HashMap::new(),
                                },
                            );
                            self.corpus_reader.phrase_to_concept.insert(effect_key, id);
                            (id, true)
                        };
                        if c_new {
                            corpus_concepts += 1;
                        }
                        if e_new {
                            corpus_concepts += 1;
                        }
                        // Mark physical concepts based on label keywords
                        if c_new {
                            if let Some(c) = self.morphogenesis.concepts.get(&cid) {
                                let label = c.label.clone();
                                self.grounding.classify_concept_as_physical(cid, &label);
                            }
                        }
                        if e_new {
                            if let Some(c) = self.morphogenesis.concepts.get(&eid) {
                                let label = c.label.clone();
                                self.grounding.classify_concept_as_physical(eid, &label);
                            }
                        }
                        if cid != 0 && eid != 0 && cid != eid {
                            self.morphogenesis.add_relation(cid, "causes", eid);
                            self.causality.observe_pair(
                                &format!("c{}", cid),
                                &format!("c{}", eid),
                                1.0,
                                1.0,
                            );
                            corpus_causal += 1;
                        }
                    }
                    // 5. Scene vector for compositional structure (already used above for cause/effect)
                    let scene = self.scene_parser.parse(&tokens, &self.semantics, dim);

                    // 5b. GROUNDING: extract physical claims and apply them to physics
                    if self.grounding.is_physical(&sentence) {
                        let pf = self
                            .grounding
                            .extract_physical_facts(&sentence, self.state.tick_count);
                        if !pf.is_empty() {
                            self.grounding
                                .apply_facts_to_physics(&pf, &mut self.physics);
                        }
                    }
                    // 6. Add the sentence as a concept candidate based on its scene flatten (dedup by phrase)
                    let flat = scene.flatten();
                    if !flat.is_empty() && flat.iter().any(|x| x.abs() > 1e-6) {
                        let sent_key = corpus_reader::CorpusReader::normalize_phrase(&sentence);
                        let (sid, is_new) = if let Some(&id) =
                            self.corpus_reader.phrase_to_concept.get(&sent_key)
                        {
                            (id, false)
                        } else {
                            let id = self.morphogenesis.next_id;
                            self.morphogenesis.next_id += 1;
                            let label = if sentence.len() > 60 {
                                sentence[..60].to_string()
                            } else {
                                sentence.clone()
                            };
                            self.morphogenesis.concepts.insert(
                                id,
                                morphogenesis::Concept {
                                    id,
                                    centroid: flat.clone(),
                                    label,
                                    count: 1,
                                    birth_tick: self.state.tick_count,
                                    tension_accumulated: 0.0,
                                    parent_id: None,
                                    children: Vec::new(),
                                    relations: std::collections::HashMap::new(),
                                    abstraction_level: 0,
                                    properties: std::collections::HashMap::new(),
                                },
                            );
                            self.corpus_reader.phrase_to_concept.insert(sent_key, id);
                            (id, true)
                        };
                        if is_new {
                            corpus_concepts += 1;
                        }
                        // 7. Episodic store
                        self.hippocampus.store(
                            self.state.tick_count,
                            &flat,
                            sid,
                            self.mood.valence,
                            self.mood.arousal,
                            "corpus_ingest",
                            if sentence.len() > 80 {
                                &sentence[..80]
                            } else {
                                &sentence
                            },
                        );
                    }
                    self.corpus_reader.total_processed += 1;
                }
                self.corpus_reader.total_concepts_added += corpus_concepts;
                self.corpus_reader.total_causal_pairs += corpus_causal;
                self.corpus_reader.total_svos += corpus_svos;
                // Auto-prune morphogenesis if it grows beyond a soft cap
                const MAX_CONCEPTS_SOFT_CAP: usize = 5000;
                if self.morphogenesis.n_concepts() > MAX_CONCEPTS_SOFT_CAP {
                    let removed = self
                        .morphogenesis
                        .prune_low_degree(MAX_CONCEPTS_SOFT_CAP, self.state.tick_count);
                    // Also clean phrase_to_concept for removed ids
                    self.corpus_reader
                        .phrase_to_concept
                        .retain(|_, id| self.morphogenesis.concepts.contains_key(id));
                    if removed > 0 {
                        self.action_log.push(format!(
                            "[CORPUS] auto-pruned {} low-degree concepts (now {})",
                            removed,
                            self.morphogenesis.n_concepts()
                        ));
                    }
                }
                // Track processing time for throughput metrics
                if n_to_process > 0 {
                    self.corpus_reader.total_processing_us +=
                        corpus_start.elapsed().as_micros() as u64;
                    self.corpus_reader.update_throughput();
                }
                if n_to_process > 0 && (corpus_concepts + corpus_causal + corpus_svos) > 0 {
                    self.action_log.push(format!(
                                "[CORPUS] consumed={} | new_concepts={} | causal+={} | svo+={} | remaining={}",
                                n_to_process, corpus_concepts, corpus_causal, corpus_svos, self.corpus_reader.remaining()
                            ));
                }

                4.0
            }
            CapabilityId::BigTransformerTrain => {
                // 6m. BigTransformer: build embeddings + train on corpus (like small transformer)
                if self.state.tick_count % 10 == 0 && !self.semantics.index_to_word.is_empty() {
                    if self.big_transformer.vocab_size == 0 {
                        self.big_transformer.build_embeddings_from_semantics(
                            &self.semantics.index_to_word,
                            &self.semantics.embeddings,
                        );
                    }
                    // Train on more sentences per self.state.tick_count for faster corpus consumption
                    let mut total_loss = 0.0f32;
                    let mut n_train = 0u64;
                    if !self
                        .metabolism
                        .spend(self.metabolism.cost_per_train, "big_transformer_train")
                    {
                        self.action_log
                            .push(format!("[METABOLISM] Skip train: energy too low"));
                    } else {
                        let n_to_train = if self.metabolism.energy > 50.0 { 4 } else { 2 };
                        // Prioritize successful generations from buffer (50% of training budget)
                        let n_from_buffer =
                            (n_to_train / 2).min(self.gen_metrics.success_buffer.len());
                        for _ in 0..n_from_buffer {
                            if let Some(seq) = self.gen_metrics.sample_success() {
                                let tokens: Vec<usize> = seq
                                    .to_lowercase()
                                    .split(|c: char| !c.is_alphanumeric())
                                    .filter(|w| !w.is_empty())
                                    .filter_map(|w| self.semantics.vocab.get(w).copied())
                                    .collect();
                                if tokens.len() >= 2 {
                                    let loss = self.big_transformer.train_on_sentence(&tokens);
                                    if loss > 0.0 {
                                        total_loss += loss;
                                        n_train += 1;
                                        self.benchmark.report_train_loss(loss);
                                    }
                                }
                            }
                        }
                        for _ in 0..n_to_train.saturating_sub(n_from_buffer) {
                            if let Some(sentence) = self.corpus_reader.next_sentence() {
                                let tokens: Vec<usize> = sentence
                                    .to_lowercase()
                                    .split(|c: char| !c.is_alphanumeric())
                                    .filter(|w| !w.is_empty())
                                    .filter_map(|w| self.semantics.vocab.get(w).copied())
                                    .collect();
                                if tokens.len() >= 2 {
                                    let loss = self.big_transformer.train_on_sentence(&tokens);
                                    if loss > 0.0 {
                                        total_loss += loss;
                                        n_train += 1;
                                        self.benchmark.report_train_loss(loss);
                                    }
                                }
                            }
                        }
                        if n_train > 0 {
                            self.action_log.push(format!("[BIG_TRANSFORMER] Trained {} sentences | avg_loss={:.3} | corpus_rem={}", n_train, total_loss / n_train as f32, self.corpus_reader.remaining()));
                        }
                    } // close metabolism spend block
                      // 6m2. PROMPT-AWARE TRAINING: teach transformer that "self.state.tick_count N energy E mood M goal G action -> INSTRUCTION"
                      // This is the critical curriculum that makes the generative controller produce parseable actions.
                    let prompt_aware_sentences: Vec<String> = {
                        let mood = self.mood.dominant_quadrant().to_lowercase();
                        let goal_label = self
                            .goal_stack
                            .stack
                            .last()
                            .and_then(|&id| self.goal_stack.goals.get(&id))
                            .map(|g| g.label.clone())
                            .unwrap_or_else(|| "none".to_string());
                        let energy = self.metabolism.energy.clamp(0.0, 100.0) as usize;
                        let actions_pool = vec![
                            format!("SETGOAL explore 0.{}", (self.state.tick_count % 9) + 1),
                            format!("SETGOAL learn 0.{}", ((self.state.tick_count + 1) % 9) + 1),
                            format!("INVOKE planner"),
                            format!("INVOKE morphogenesis"),
                            format!("TOOLCALL calculator explore"),
                            format!("TRAIN_TF {}", ((self.state.tick_count % 5) + 1)),
                            format!("WAIT {}", ((self.state.tick_count % 4) + 1)),
                            format!("READ_BUS motivation"),
                            format!(
                                "WRITE_BUS motivation 1.{}",
                                ((self.state.tick_count % 5) + 1)
                            ),
                            format!("LOG mission"),
                        ];
                        let prompt = format!(
                            "self.state.tick_count {} energy {} mood {} goal {} action",
                            self.state.tick_count, energy, mood, goal_label
                        );
                        let n_pairs = 2usize;
                        (0..n_pairs)
                            .map(|i| {
                                let instr = &actions_pool[i % actions_pool.len()];
                                format!("{} {}", prompt, instr)
                            })
                            .collect()
                    };
                    for sentence in &prompt_aware_sentences {
                        let tokens: Vec<usize> = sentence
                            .to_lowercase()
                            .split(|c: char| !c.is_alphanumeric())
                            .filter(|w| !w.is_empty())
                            .filter_map(|w| self.semantics.vocab.get(w).copied())
                            .collect();
                        if tokens.len() >= 2 {
                            let loss = self.big_transformer.train_on_sentence(&tokens);
                            if loss > 0.0 {
                                total_loss += loss;
                                n_train += 1;
                                self.benchmark.report_train_loss(loss);
                            }
                        }
                    }
                    if !prompt_aware_sentences.is_empty() {
                        self.action_log.push(format!(
                            "[PROMPT_AWARE] Trained {} state->action pairs | avg_loss={:.3}",
                            prompt_aware_sentences.len(),
                            total_loss / n_train.max(1) as f32
                        ));
                    }
                    // Hole 1: Full end-to-end backprop every 50 ticks (updates FF + LayerNorm weights)
                    if self.state.tick_count % 50 == 0 && n_train > 0 {
                        if let Some(sentence) = self.corpus_reader.next_sentence() {
                            let tokens: Vec<usize> = sentence
                                .to_lowercase()
                                .split(|c: char| !c.is_alphanumeric())
                                .filter(|w| !w.is_empty())
                                .filter_map(|w| self.semantics.vocab.get(w).copied())
                                .collect();
                            if tokens.len() >= 2 {
                                let full_loss = self.big_transformer.train_full(&tokens);
                                self.benchmark.report_train_loss(full_loss);
                                self.action_log.push(format!("[BIG_TRANSFORMER] FULL backprop | loss={:.3} | layers={} | params_updated=FF+LN+Adapter+Output", full_loss, self.big_transformer.n_layers));
                            }
                        }
                    }
                }

                8.0
            }
            CapabilityId::BigTransformerGenerate => {
                // 6n2. Chain-of-Thought: generate reasoning about current state
                if self.state.tick_count % 35 == 0 && self.big_transformer.vocab_size > 0 {
                    let prompt_text = format!(
                        "self.state.tick_count {} mood {} focus {} concepts {} ",
                        self.state.tick_count,
                        self.mood.dominant_quadrant(),
                        self.motivation.current_focus,
                        self.morphogenesis.n_concepts()
                    );
                    let prompt_tokens: Vec<usize> = prompt_text
                        .to_lowercase()
                        .split(|c: char| !c.is_alphanumeric())
                        .filter(|w| !w.is_empty())
                        .filter_map(|w| self.semantics.vocab.get(w).copied())
                        .collect();
                    if !prompt_tokens.is_empty() {
                        let reasoning = self.big_transformer.generate(&prompt_tokens, 12, 0.8, 5);
                        let reasoning_words: Vec<String> = reasoning
                            .iter()
                            .filter_map(|&t| self.semantics.index_to_word.get(t).cloned())
                            .collect();
                        if !reasoning_words.is_empty() {
                            self.action_log
                                .push(format!("[COT] reasoning: {}", reasoning_words.join(" ")));
                        }
                        // Force answer: predict next action recommendation
                        let answer = self.big_transformer.generate(&prompt_tokens, 6, 0.6, 3);
                        let answer_words: Vec<String> = answer
                            .iter()
                            .filter_map(|&t| self.semantics.index_to_word.get(t).cloned())
                            .collect();
                        if !answer_words.is_empty() {
                            self.action_log
                                .push(format!("[COT] answer: {}", answer_words.join(" ")));
                        }
                        // INTERCONNECTION: CoT -> morphogenesis (reasoning words become concepts)
                        for word in reasoning_words.iter().chain(answer_words.iter()) {
                            if let Some(emb) = self.semantics.embedding(word) {
                                let concept_id = self
                                    .morphogenesis
                                    .add_sample(&emb.to_vec(), word, self.state.tick_count, 0.3)
                                    .0;
                                if concept_id != 0 {
                                    self.morphogenesis.add_relation(
                                        concept_id,
                                        "reasoned_by",
                                        self.state.last_concept_id,
                                    );
                                }
                            }
                        }
                    }
                }

                10.0
            }
            CapabilityId::SemanticsObserve => 3.0,
            CapabilityId::SyntaxParse => 2.0,
            CapabilityId::SceneParser => 2.0,
            CapabilityId::Morphogenesis => 3.0,
            CapabilityId::Causality => {
                // === NUEVA ARQUITECTURA: Causal Model (do-calculus) ===
                if self.state.tick_count % 100 == 0 {
                    self.causal_model
                        .add_variable("motivation", &["stress"], &[0.7], 0.1);
                    self.causal_model.add_variable("stress", &[], &[], 0.5);
                    let mut evidence = std::collections::HashMap::new();
                    evidence.insert("stress".to_string(), self.motivation.discomfort);
                    let post = self.causal_model.intervene("motivation", 0.2, &evidence);
                    if let Some(&stress_val) = post.get("stress") {
                        self.action_log.push(format!(
                            "[CAUSAL_MODEL] do(motivation=0.2) -> stress={:.2}",
                            stress_val
                        ));
                    }
                }

                2.0
            }
            CapabilityId::Grounding => 2.0,
            CapabilityId::Physics => 2.0,
            CapabilityId::Hippocampus => 2.0,
            CapabilityId::Mood => {
                // 7. Passive updates (all subsystems observe, but only the winner acted)
                self.mood.update(self.motivation.discomfort, 0.0);

                1.0
            }
            CapabilityId::Motivation => 1.0,
            CapabilityId::GoalStack => 1.0,
            CapabilityId::Planner => {
                // 4. Planner: simulate futures and score action sequences
                if self.state.tick_count % 5 == 0
                    && self.metabolism.can_afford(self.metabolism.cost_per_plan)
                {
                    let plan = self.planner.plan(
                        self.state.tick_count,
                        &self.motivation,
                        &self.self_model,
                        &self.morphogenesis,
                        &self.causality,
                        &self.state.capabilities,
                    );
                    if let Some(p) = plan {
                        self.action_log.push(format!(
                            "[PLANNER] {} | score={:.3} | predicted_disc={:.2} | steps={:?}",
                            self.planner.status(),
                            p.score,
                            p.predicted_final_discomfort,
                            p.steps
                                .iter()
                                .map(|c| format!("{:?}", c))
                                .collect::<Vec<_>>()
                                .join(",")
                        ));
                    }
                }

                // 4b. Auto-dispatch tools if planner recommends actionable capability
                if let Some(ref best) = self.planner.best {
                    if let Some(first) = best.steps.first() {
                        match first {
                            GarmCapability::ComputerUse => {
                                if !self
                                    .metabolism
                                    .spend(self.metabolism.cost_per_tool, "tool_system_info")
                                {
                                }
                                let call = tools::ToolCall {
                                    tool_name: "system_info".into(),
                                    args: {
                                        let mut h = HashMap::new();
                                        h.insert("info_type".into(), "load".into());
                                        h
                                    },
                                };
                                let res = self.tool_registry.execute(&call);
                                if res.success {
                                    self.action_log
                                        .push(format!("[AUTO-TOOL] system_info -> {}", res.output));
                                }
                            }
                            GarmCapability::Semantics | GarmCapability::CorpusReader => {
                                if !self
                                    .metabolism
                                    .spend(self.metabolism.cost_per_tool, "tool_search_corpus")
                                {
                                }
                                let call = tools::ToolCall {
                                    tool_name: "search_corpus".into(),
                                    args: {
                                        let mut h = HashMap::new();
                                        h.insert("query".into(), "learn".into());
                                        h
                                    },
                                };
                                let res = self.tool_registry.execute(&call);
                                if res.success {
                                    self.action_log.push(format!(
                                        "[AUTO-TOOL] search_corpus -> {} chars",
                                        res.output.len()
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // 5. Planner observes actual outcome vs prediction
                self.planner
                    .observe_outcome(self.motivation.discomfort, self.self_model.mean_error());

                // 6n3. INTERCONNECTION: planner + CoT -> simulate plan with reasoning
                if self.state.tick_count % 20 == 0 && self.big_transformer.vocab_size > 0 {
                    if let Some(ref plan) = self.planner.best {
                        let plan_text = format!(
                            "plan score {:.2} discomfort {:.2}",
                            plan.score, plan.predicted_final_discomfort
                        );
                        let plan_tokens: Vec<usize> = plan_text
                            .to_lowercase()
                            .split(|c: char| !c.is_alphanumeric())
                            .filter(|w| !w.is_empty())
                            .filter_map(|w| self.semantics.vocab.get(w).copied())
                            .collect();
                        if !plan_tokens.is_empty() {
                            let plan_reasoning =
                                self.big_transformer.generate(&plan_tokens, 8, 0.8, 5);
                            let words: Vec<String> = plan_reasoning
                                .iter()
                                .filter_map(|&t| self.semantics.index_to_word.get(t).cloned())
                                .collect();
                            if !words.is_empty() {
                                self.action_log.push(format!(
                                    "[INTERCONN] planner->CoT | plan reasoning: {}",
                                    words.join(" ")
                                ));
                            }
                        }
                    }
                }

                3.0
            }
            CapabilityId::Security => 1.5,
            CapabilityId::Neural => 2.0,
            CapabilityId::TransformerSmall => {
                // 6i. E) Distillation from symbolic causal graph -> neural transformer
                if self.state.tick_count % 40 == 0 && self.transformer.vocab_size > 0 {
                    let relations: Vec<(u64, u64)> = self
                        .morphogenesis
                        .concepts
                        .iter()
                        .flat_map(|(cid, c)| {
                            c.relations
                                .get("causes")
                                .cloned()
                                .unwrap_or_default()
                                .iter()
                                .map(|&t| (*cid, t))
                                .collect::<Vec<_>>()
                        })
                        .collect();
                    if !relations.is_empty() {
                        let mut total_distill_loss = 0.0f32;
                        let mut n_distill = 0u64;
                        for (a_id, b_id) in relations.iter().take(20) {
                            if let (Some(ca), Some(cb)) = (
                                self.morphogenesis.concepts.get(a_id),
                                self.morphogenesis.concepts.get(b_id),
                            ) {
                                let cause_tokens: Vec<usize> = ca
                                    .label
                                    .to_lowercase()
                                    .split(|c: char| !c.is_alphanumeric())
                                    .filter(|w| !w.is_empty())
                                    .filter_map(|w| self.semantics.vocab.get(w).copied())
                                    .collect();
                                let effect_tokens: Vec<usize> = cb
                                    .label
                                    .to_lowercase()
                                    .split(|c: char| !c.is_alphanumeric())
                                    .filter(|w| !w.is_empty())
                                    .filter_map(|w| self.semantics.vocab.get(w).copied())
                                    .collect();
                                if !cause_tokens.is_empty() && !effect_tokens.is_empty() {
                                    let loss = self
                                        .transformer
                                        .distill_from_graph(&cause_tokens, &effect_tokens);
                                    if loss > 0.0 {
                                        total_distill_loss += loss;
                                        n_distill += 1;
                                    }
                                }
                            }
                        }
                        if n_distill > 0 {
                            self.action_log.push(format!(
                                "[DISTILL] Symbolic->Neural | {} pairs | avg_loss={:.3}",
                                n_distill,
                                total_distill_loss / n_distill as f32
                            ));
                        }
                    }
                }

                2.0
            }
            CapabilityId::BusPredictor => 2.0,
            CapabilityId::WorldModelNN => {
                // === NUEVA ARQUITECTURA: World Model NN ===
                let action_vec = vec![
                    self.motivation.drives.curiosity,
                    self.motivation.drives.efficiency,
                    self.motivation.drives.stability,
                    self.motivation.drives.competence,
                ];
                let wm_mse = self.world_model_nn.train_step(
                    &self.slots_to_vector(),
                    &action_vec,
                    &self.slots_to_vector(),
                );
                if self.state.tick_count % 40 == 0 {
                    self.action_log
                        .push(format!("[WORLD_MODEL_NN] train | mse={:.6}", wm_mse));
                }

                2.0
            }
            CapabilityId::MoE => {
                // === NUEVA ARQUITECTURA: MoE status ===
                if self.state.tick_count % 70 == 0 {
                    self.action_log.push(format!("[MOE] {}", self.moe.status()));
                }

                2.0
            }
            CapabilityId::HierarchicalAttention => {
                // === NUEVA ARQUITECTURA: Hierarchical Attention pass ===
                let tokens: Vec<Vec<f32>> =
                    self.semantics.embeddings.iter().cloned().take(4).collect();
                let objects: Vec<Vec<f32>> = self
                    .world_model
                    .objects
                    .values()
                    .map(|o| vec![o.cx, o.cy, o.area, o.vx, o.vy])
                    .take(4)
                    .collect();
                let episodes: Vec<Vec<f32>> = self
                    .hippocampus
                    .episodes
                    .iter()
                    .map(|e| e.embedding.clone())
                    .take(4)
                    .collect();
                self.hierarchical_attention
                    .full_pass(&tokens, &objects, &episodes);
                if self.state.tick_count % 60 == 0 {
                    self.action_log.push(format!(
                        "[HIERARCHICAL] {}",
                        self.hierarchical_attention.status()
                    ));
                }

                2.0
            }
            CapabilityId::ContinualLearning => {
                // === NUEVA ARQUITECTURA: Continual Learning (EWC) ===
                if self.state.tick_count % 200 == 0 && self.big_transformer.vocab_size > 0 {
                    // Proxy: consolidate current big_transformer output bias as "task params"
                    let params: Vec<f32> = self.big_transformer.output_bias.clone();
                    self.continual_learning.consolidate_task(&params);
                    let pen = self.continual_learning.penalty(&params);
                    self.action_log
                        .push(format!("[CONTINUAL] EWC consolidated | penalty={:.6}", pen));
                }

                2.0
            }
            CapabilityId::MetaLearning => {
                // === NUEVA ARQUITECTURA: Meta-Learning (MAML) ===
                if self.state.tick_count % 120 == 0 {
                    let task: Vec<(Vec<f32>, Vec<f32>)> = (0..4)
                        .map(|i| {
                            let x = vec![i as f32 * 0.1; 64];
                            let y = vec![i as f32 * 0.1; 64];
                            (x, y)
                        })
                        .collect();
                    self.meta_learning.meta_update(&task);
                    if self.state.tick_count % 120 == 0 {
                        self.action_log
                            .push(format!("[META_LEARN] {}", self.meta_learning.status()));
                    }
                }

                2.0
            }
            CapabilityId::EWC => 1.0,
            CapabilityId::MDLPruner => {
                // === NUEVA ARQUITECTURA: MDL Pruner ===
                if self.state.tick_count % 150 == 0 {
                    let concepts_info: Vec<(String, u32, u32, usize)> = self
                        .morphogenesis
                        .concepts
                        .values()
                        .take(10)
                        .map(|c| {
                            let n_corr = c.children.len() as u32;
                            (
                                c.label.clone(),
                                c.count,
                                n_corr,
                                self.morphogenesis.relation_count(),
                            )
                        })
                        .collect();
                    let to_prune = self.mdl_pruner.prune_batch(&concepts_info);
                    if !to_prune.is_empty() {
                        self.action_log.push(format!(
                            "[MDL_PRUNER] Would prune {} concepts",
                            to_prune.len()
                        ));
                    }
                }

                1.0
            }
            CapabilityId::EmotionalModulation => {
                // === NUEVA ARQUITECTURA: Emotional Modulation ===
                let reward_signal = if self.action_log.iter().any(|a| a.contains("success=true")) {
                    0.3
                } else {
                    -0.1
                };
                let novelty_signal = self.surprise.surprise_ema.min(1.0);
                let control_signal = if self.planner.best.is_some() {
                    0.5
                } else {
                    0.1
                };
                self.emotional_modulation
                    .update(reward_signal, novelty_signal, control_signal);

                1.0
            }
            CapabilityId::DNC => {
                // === NUEVA ARQUITECTURA: DNC Memory read/write ===
                let dnc_key = self.recurrent_state.readout(64);
                let _dnc_reads = self.dnc.read(&dnc_key, 1.0);
                self.dnc.write(&dnc_key, &dnc_key, &vec![0.1f32; 64], 0.5);
                if self.state.tick_count % 50 == 0 {
                    self.action_log.push(format!("[DNC] {}", self.dnc.status()));
                }

                // === NUEVA ARQUITECTURA: DNC wired to Transformer ===
                let dnc_reads = self.dnc.read(&self.slots_to_vector(), 1.0);
                let dnc_flat = dnc_reads
                    .iter()
                    .flat_map(|v| v.iter().copied())
                    .collect::<Vec<f32>>();
                // Inject DNC readout into transformer external_context
                for d in 0..self.big_transformer.d_model.min(dnc_flat.len()) {
                    self.big_transformer.external_context[d] = dnc_flat[d] * 0.1;
                }
                if self.state.tick_count % 50 == 0 {
                    self.action_log.push(format!(
                        "[DNC_WIRED] injected {} dims into transformer external_context",
                        dnc_flat.len().min(self.big_transformer.d_model)
                    ));
                }

                3.0
            }
            CapabilityId::ActiveInference => {
                // === NUEVA ARQUITECTURA: Active Inference (FEP) ===
                let obs = self.slots_to_vector();
                self.active_inference.perceive(&obs);
                let f = self.active_inference.free_energy(&obs);
                if self.state.tick_count % 60 == 0 {
                    self.action_log.push(format!(
                        "[FEP] FreeEnergy={:.3} | steps={}",
                        f, self.active_inference.n_steps
                    ));
                }

                2.0
            }
            CapabilityId::Body => {
                // === NUEVA ARQUITECTURA: Embodiment (sensorimotor) ===
                let objs: Vec<(f32, f32)> = self
                    .world_model
                    .objects
                    .values()
                    .map(|o| (o.cx, o.cy))
                    .collect();
                self.embodiment.sense(&objs);
                self.embodiment.act(
                    self.motivation.drives.curiosity * 2.0 - 1.0,
                    self.motivation.drives.efficiency * 2.0 - 1.0,
                );
                if self.state.tick_count % 40 == 0 {
                    self.action_log
                        .push(format!("[BODY] {}", self.embodiment.status()));
                }

                1.0
            }
            CapabilityId::TemporalHierarchy => {
                // === NUEVA ARQUITECTURA: Temporal Hierarchy ===
                let frame = self.embodiment.proprioception();
                self.temporal_hierarchy
                    .push_frame(&frame, self.state.tick_count);
                if self.state.tick_count % 70 == 0 {
                    self.action_log
                        .push(format!("[TEMP_HIER] {}", self.temporal_hierarchy.status()));
                }

                1.0
            }
            CapabilityId::SelfModification => {
                // === NUEVA ARQUITECTURA: Self-Modification ===
                if self.state.tick_count % 100 == 0 {
                    if let Some(prop) = self.self_modification.propose(
                        "planner",
                        "horizon",
                        self.planner.horizon as f32,
                        self.planner.n_simulations as f32 / 100.0,
                        self.state.tick_count,
                    ) {
                        if self.self_modification.apply(&prop) {
                            self.action_log.push(format!(
                                "[SELMOD] Applied: {} | {}",
                                prop.target, prop.reason
                            ));
                        }
                    }
                }

                1.0
            }
            CapabilityId::LogicReasoning => {
                // === NUEVA ARQUITECTURA: Logic Reasoning ===
                if self.state.tick_count % 120 == 0 {
                    self.logic_reasoning.add_fact(
                        "morpho_growing",
                        &["concepts"],
                        self.morphogenesis.n_concepts() > 10,
                    );
                    self.logic_reasoning.add_fact(
                        "errors_low",
                        &["build"],
                        self.auto_debug
                            .diagnostics
                            .iter()
                            .filter(|d| d.level == "error")
                            .count()
                            == 0,
                    );
                    self.logic_reasoning.add_rule(
                        &[("morpho_growing".to_string(), vec!["concepts".to_string()])],
                        ("system_healthy".to_string(), vec![]),
                    );
                    let inferred = self.logic_reasoning.infer();
                    if !inferred.is_empty() && self.state.tick_count % 120 == 0 {
                        self.action_log.push(format!(
                            "[LOGIC] Inferred {} new facts | last='{}'",
                            inferred.len(),
                            self.logic_reasoning.last_conclusion
                        ));
                    }
                }

                2.0
            }
            CapabilityId::ConstitutionalSafety => {
                // === NUEVA ARQUITECTURA: Constitutional Safety ===
                if let Some(last) = self.action_log.last() {
                    let (allowed, penalty) = self.constitutional_safety.evaluate(last);
                    if !allowed {
                        self.action_log.push(format!(
                            "[SAFETY] BLOCKED '{}' | penalty={:.2}",
                            last, penalty
                        ));
                    } else if penalty > 0.0 && self.state.tick_count % 50 == 0 {
                        self.action_log
                            .push(format!("[SAFETY] Penalty={:.2} for '{}'", penalty, last));
                    }
                }

                1.0
            }
            CapabilityId::Phenomenology => {
                // === NUEVA ARQUITECTURA: Phenomenology (Qualia proxy) ===
                self.phenomenology.update(&self.slots_to_vector());
                if self.state.tick_count % 60 == 0 {
                    self.action_log.push(format!(
                        "[PHENOM] {} | {}",
                        self.phenomenology.status(),
                        self.phenomenology.what_it_is_like()
                    ));
                }

                1.0
            }
            CapabilityId::EconomicAgent => {
                // 6l. AutonomyEcon: resource management self.state.tick_count
                if self.state.tick_count % 10 == 0 {
                    self.autonomy_econ.tick();
                    self.action_log
                        .push(format!("[AUTONOMY] {}", self.autonomy_econ.status()));
                    // INTERCONNECTION: if resources critically low, negotiate socially
                    let critical: Vec<String> = self
                        .autonomy_econ
                        .resources
                        .iter()
                        .filter(|(_, r)| r.fraction() < 0.15)
                        .map(|(name, _)| name.clone())
                        .collect();
                    for res_name in critical {
                        let peer = "peer-alpha";
                        if let Some(offer) =
                            self.social_complex
                                .negotiate("eden", peer, &res_name, "knowledge")
                        {
                            self.action_log.push(format!("[INTERCONN] autonomy_econ->social | negotiated {} from {} | util={:.2}", res_name, peer, offer.utility_for_receiver));
                        }
                    }
                }

                // === NUEVA ARQUITECTURA: Economic Agent (real resource economy) ===
                self.economic_agent.tick();
                // Produce from observable successes
                if self.action_log.iter().any(|a| a.contains("success=true")) {
                    self.economic_agent.produce("tool_success", 1.0);
                }
                if self
                    .auto_debug
                    .diagnostics
                    .iter()
                    .filter(|d| d.level == "error")
                    .count()
                    == 0
                {
                    self.economic_agent.produce("build_clean", 1.0);
                }
                if self.morphogenesis.n_concepts() > 0 {
                    self.economic_agent
                        .produce("discovery", self.morphogenesis.n_concepts() as f32 / 100.0);
                }
                // Consume for expensive operations
                if self.state.tick_count % 10 == 0 {
                    self.economic_agent.consume("train", 1.0);
                }
                if self.state.tick_count % 30 == 0 {
                    self.action_log
                        .push(format!("[ECON_AGENT] {}", self.economic_agent.status()));
                }

                2.0
            }
            CapabilityId::RewardOracle => {
                // === NUEVA ARQUITECTURA: Reward Oracle (external consistent reward) ===
                let prev_errors = 0usize; // we don't track prev in this scope; use 0 as baseline
                let curr_errors = self
                    .auto_debug
                    .diagnostics
                    .iter()
                    .filter(|d| d.level == "error")
                    .count();
                let curr_concepts = self.morphogenesis.n_concepts();
                let (reward, explanation) = self.reward_oracle.evaluate(
                    prev_errors,
                    curr_errors,
                    0,
                    curr_concepts,
                    0,
                    self.corpus_reader.total_processed,
                    self.action_log.iter().any(|a| a.contains("success=true")),
                    self.world_model_nn.pred_err_ema,
                );
                if self.state.tick_count % 30 == 0 {
                    self.action_log.push(format!(
                        "[REWARD_ORACLE] R={:.2} | adv={:.2} | {}",
                        reward,
                        self.reward_oracle.advantage(),
                        explanation
                    ));
                }

                1.0
            }
            CapabilityId::BPTT => {
                // === NUEVA ARQUITECTURA: BPTT (truncated backprop through time) ===
                let recurrent_before = self.big_transformer.recurrent_hidden.clone();
                // Record state for BPTT
                let bptt_tokens: Vec<usize> = vec![0; 4]; // placeholder tokens
                self.bptt.record(
                    &bptt_tokens,
                    &recurrent_before,
                    &self.big_transformer.recurrent_hidden,
                    self.state.tick_count,
                );
                self.bptt
                    .accumulate_and_apply(&mut self.big_transformer.recurrent_hidden);
                if self.state.tick_count % 40 == 0 {
                    self.action_log
                        .push(format!("[BPTT] {}", self.bptt.status()));
                }

                2.0
            }
            CapabilityId::CorpusMassive => {
                // === NUEVA ARQUITECTURA: Corpus Massive (10M+ streaming) ===
                if self.state.tick_count % 10 == 0
                    && self.big_transformer.vocab_size > 0
                    && self.metabolism.energy > 10.0
                {
                    let n_gen = if self.metabolism.energy > 60.0 { 4 } else { 2 };
                    let mut total_loss = 0.0f32;
                    let mut n = 0u64;
                    // Prioritize successful generations from buffer (50% of training budget)
                    let n_from_buffer = (n_gen / 2).min(self.gen_metrics.success_buffer.len());
                    for _ in 0..n_from_buffer {
                        if let Some(seq) = self.gen_metrics.sample_success() {
                            let tokens: Vec<usize> = seq
                                .to_lowercase()
                                .split(|c: char| !c.is_alphanumeric())
                                .filter(|w| !w.is_empty())
                                .filter_map(|w| self.semantics.vocab.get(w).copied())
                                .collect();
                            if tokens.len() >= 2 {
                                let loss = self.big_transformer.train_on_sentence(&tokens);
                                if loss > 0.0 {
                                    total_loss += loss;
                                    n += 1;
                                    self.benchmark.report_train_loss(loss);
                                }
                            }
                        }
                    }
                    let sentences = self
                        .corpus_massive
                        .generate_n(n_gen.saturating_sub(n_from_buffer));
                    for sentence in sentences {
                        let tokens: Vec<usize> = sentence
                            .to_lowercase()
                            .split(|c: char| !c.is_alphanumeric())
                            .filter(|w| !w.is_empty())
                            .filter_map(|w| self.semantics.vocab.get(w).copied())
                            .collect();
                        if tokens.len() >= 2 {
                            let loss = self.big_transformer.train_on_sentence(&tokens);
                            if loss > 0.0 {
                                total_loss += loss;
                                n += 1;
                                self.benchmark.report_train_loss(loss);
                            }
                        }
                    }
                    if n > 0 && self.state.tick_count % 50 == 0 {
                        self.action_log.push(format!(
                            "[CORPUS_MASSIVE] Trained {} proc sentences | avg_loss={:.3}",
                            n,
                            total_loss / n as f32
                        ));
                    }
                }

                3.0
            }
            CapabilityId::GenController => {
                // 6r. Transformer as Program Generator (Movimiento B)
                if self.state.tick_count % 25 == 0 && self.big_transformer.vocab_size > 0 {
                    if self.active_program.is_none() {
                        let prompt = format!(
                            "program for self.state.tick_count {} mood {} ",
                            self.state.tick_count,
                            self.mood.dominant_quadrant()
                        );
                        let prompt_tokens: Vec<usize> = prompt
                            .to_lowercase()
                            .split(|c: char| !c.is_alphanumeric())
                            .filter(|w| !w.is_empty())
                            .filter_map(|w| self.semantics.vocab.get(w).copied())
                            .collect();
                        if !prompt_tokens.is_empty() {
                            let generated =
                                self.big_transformer.generate(&prompt_tokens, 15, 0.7, 5);
                            let generated_words: Vec<String> = generated
                                .iter()
                                .filter_map(|&t| self.semantics.index_to_word.get(t).cloned())
                                .collect();
                            let source_text = generated_words.join(" ");
                            let prog = program::Program::from_text(&source_text);
                            if !prog.instructions.is_empty() {
                                self.action_log.push(format!(
                                    "[PROGRAM] generated {} instructions from transformer",
                                    prog.instructions.len()
                                ));
                                self.active_program = Some(prog);
                            }
                        }
                    }
                }
                // Execute one instruction per self.state.tick_count from active program
                if let Some(mut prog) = self.active_program.take() {
                    if let Some(instr) = prog.current().cloned() {
                        let pc = prog.pc;
                        let (log, jump) = self.execute_instruction(&instr);
                        self.action_log
                            .push(format!("[PROGRAM] pc={} | {}", pc, log));
                        if jump > 0 {
                            prog.jump(jump);
                        } else {
                            prog.step();
                        }
                        self.active_program = Some(prog);
                    } else {
                        // PROGRAM COMPLETED — compute reward and train with Meta-RL (Movimiento C)
                        self.action_log.push("[PROGRAM] completed".to_string());
                        let mut reward = 0.0f32;
                        // +1 if build is clean
                        let errors = self
                            .auto_debug
                            .diagnostics
                            .iter()
                            .filter(|d| d.level == "error")
                            .count();
                        if errors == 0 {
                            reward += 1.0;
                        }
                        // +0.5 if corpus is being consumed
                        if self.corpus_reader.total_processed > 0 {
                            reward += 0.5;
                        }
                        // +0.5 if morphogenesis grew
                        if self.morphogenesis.n_concepts() > 0 {
                            reward += 0.5;
                        }
                        // +0.2 per successful tool call in program (heuristic: check actions)
                        let tool_successes = self
                            .action_log
                            .iter()
                            .filter(|a| a.contains("TOOLCALL") && a.contains("success=true"))
                            .count();
                        reward += tool_successes as f32 * 0.2;
                        // -1 if program was trivial (0 or 1 instructions)
                        if prog.instructions.len() <= 1 {
                            reward -= 1.0;
                        }
                        // Clamp
                        reward = reward.clamp(-2.0, 3.0);
                        // RL finetune: reward all tokens of the program
                        if self.big_transformer.vocab_size > 0 && !prog.source_text.is_empty() {
                            let tokens: Vec<usize> = prog
                                .source_text
                                .to_lowercase()
                                .split(|c: char| !c.is_alphanumeric())
                                .filter(|w| !w.is_empty())
                                .filter_map(|w| self.semantics.vocab.get(w).copied())
                                .collect();
                            for i in 1..tokens.len() {
                                let context = &tokens[..i];
                                let predicted = tokens[i];
                                self.big_transformer
                                    .rl_finetune_step(context, predicted, reward);
                            }
                            self.action_log.push(format!(
                                "[META-RL] Program reward={:.2} | {} tokens reinforced",
                                reward,
                                tokens.len()
                            ));
                        }
                        // Save to experience buffer
                        let state_bus = self.slots_to_vector();
                        self.experience_buffer.add(experience_buffer::Experience {
                            state_bus,
                            program_text: prog.source_text.clone(),
                            reward,
                            next_state_bus: self.slots_to_vector(),
                            tick: self.state.tick_count,
                            n_tokens: prog.instructions.len(),
                        });
                    }
                }

                // === GENERATIVE CONTROLLER: BigTransformer generates executable program ===
                // Adaptive frequency: more frequent when self.gen_metrics.parse_rate() is low (need more training examples),
                // less frequent when converged.
                let gen_freq = if self.gen_metrics.parse_rate() < 0.05 {
                    10
                } else if self.gen_metrics.parse_rate() < 0.20 {
                    15
                } else {
                    20
                };
                if self.state.tick_count % gen_freq == 0
                    && self.state.tick_count > 10
                    && self.big_transformer.vocab_size > 0
                    && self.metabolism.energy > 20.0
                {
                    let mood = self.mood.dominant_quadrant().to_lowercase();
                    let _motive = self.motivation.dominant_drive();
                    let goal_label = self
                        .goal_stack
                        .stack
                        .last()
                        .and_then(|&id| self.goal_stack.goals.get(&id))
                        .map(|g| g.label.clone())
                        .unwrap_or_else(|| "none".to_string());
                    let energy = self.metabolism.energy.clamp(0.0, 100.0) as usize;
                    let prompt_text = format!(
                        "self.state.tick_count {} energy {} mood {} goal {} action",
                        self.state.tick_count, energy, mood, goal_label
                    );
                    let prompt_tokens: Vec<usize> = prompt_text
                        .to_lowercase()
                        .split(|c: char| !c.is_alphanumeric())
                        .filter(|w| !w.is_empty())
                        .filter_map(|w| self.semantics.vocab.get(w).copied())
                        .collect();
                    if !prompt_tokens.is_empty() {
                        // Adaptive temperature: cold (deterministic) when self.gen_metrics.parse_rate() is low, warmer as it improves
                        let temp = if self.gen_metrics.parse_rate() < 0.05 {
                            0.2
                        } else if self.gen_metrics.parse_rate() < 0.15 {
                            0.3
                        } else if self.gen_metrics.parse_rate() < 0.30 {
                            0.5
                        } else {
                            0.7
                        };
                        // Adaptive top_k: restrict to top 2 when learning, open to 5 when converged
                        let top_k = if self.gen_metrics.parse_rate() < 0.05 {
                            2
                        } else {
                            5
                        };
                        let generated =
                            self.big_transformer
                                .generate(&prompt_tokens, 5, temp, top_k);
                        let gen_text: String = generated
                            .iter()
                            .filter_map(|&t| self.semantics.index_to_word.get(t))
                            .cloned()
                            .collect::<Vec<String>>()
                            .join(" ");
                        if !gen_text.is_empty() {
                            let instrs = program::parse_program(&gen_text);
                            if !instrs.is_empty() {
                                let goals_before = self.goal_stack.stack.len();
                                let mut exec_logs = Vec::new();
                                for instr in &instrs {
                                    let (log, _) = self.execute_instruction(instr);
                                    exec_logs.push(log);
                                }
                                let goals_after = self.goal_stack.stack.len();
                                let mut reward = 0.0f32;
                                if goals_after > goals_before {
                                    reward += 5.0;
                                }
                                reward += exec_logs.len() as f32;
                                // RL finetune each generated token with this reward
                                let mut ctx = prompt_tokens.clone();
                                for &tok in &generated {
                                    self.big_transformer.rl_finetune_step(&ctx, tok, reward);
                                    ctx.push(tok);
                                }
                                let temp = if self.gen_metrics.parse_rate() < 0.05 {
                                    0.2
                                } else if self.gen_metrics.parse_rate() < 0.15 {
                                    0.3
                                } else if self.gen_metrics.parse_rate() < 0.30 {
                                    0.5
                                } else {
                                    0.7
                                };
                                self.gen_metrics.report_generation(
                                    &gen_text,
                                    true,
                                    instrs.len(),
                                    reward,
                                    Some(&prompt_text),
                                );
                                self.action_log.push(format!("[GEN_CTRL] temp={:.1} prompt='{}' | gen='{}' | instrs={} | reward={:.1} | exec={}",
                                            temp, prompt_text, gen_text, instrs.len(), reward, exec_logs.join("; ")));
                            } else {
                                let temp = if self.gen_metrics.parse_rate() < 0.05 {
                                    0.2
                                } else if self.gen_metrics.parse_rate() < 0.15 {
                                    0.3
                                } else if self.gen_metrics.parse_rate() < 0.30 {
                                    0.5
                                } else {
                                    0.7
                                };
                                self.gen_metrics.report_generation(
                                    &gen_text,
                                    false,
                                    0,
                                    0.0,
                                    Some(&prompt_text),
                                );
                                self.action_log.push(format!(
                                    "[GEN_CTRL] temp={:.1} prompt='{}' | gen='{}' | no_parse",
                                    temp, prompt_text, gen_text
                                ));
                            }
                        }
                    }
                }

                // === GEN_METRICS: report generative controller convergence every 50 ticks ===
                if self.state.tick_count % 50 == 0 && self.gen_metrics.n_generations > 0 {
                    self.action_log
                        .push(format!("[GEN_METRICS] {}", self.gen_metrics.status()));
                }

                0.0
            }
            CapabilityId::SocialComplex => {
                // 6k. SocialComplex: simulate interactions between "eden" and random peers
                if self.state.tick_count % 45 == 0 {
                    let peers = vec!["peer-alpha", "peer-beta", "peer-gamma"];
                    let p = peers[self.state.tick_count as usize % peers.len()];
                    let action = if self.state.tick_count % 90 == 0 {
                        social_complex::Action::Cooperate
                    } else {
                        social_complex::Action::Negotiate
                    };
                    self.social_complex.interact(
                        "eden",
                        p,
                        action.clone(),
                        social_complex::Action::Cooperate,
                    );
                    if let Some(offer) =
                        self.social_complex
                            .negotiate("eden", p, "knowledge", "compute")
                    {
                        self.action_log.push(format!(
                            "[SOCIAL] Negotiated with {} | give={} | receive={} | util={:.2}",
                            p, offer.give, offer.receive, offer.utility_for_receiver
                        ));
                    }
                    // INTERCONNECTION: social outcomes -> ToM beliefs + mood modulation
                    if let Some(rep) = self.social_complex.reputations.get(p) {
                        let trust = rep.trust_score;
                        self.theory_of_mind.observe(
                            if trust > 0.5 { "cooperate" } else { "defect" },
                            trust - 0.5,
                        );
                        let valence_mod = if trust > 0.5 { 0.05 } else { -0.05 };
                        self.mood.update(self.motivation.discomfort, valence_mod);
                    }
                    self.action_log
                        .push(format!("[SOCIAL] {}", self.social_complex.status()));
                }

                2.0
            }
            CapabilityId::MultiAgent => 2.0,
            CapabilityId::Swarm => 2.0,
            CapabilityId::Metacognition => 2.0,
            CapabilityId::SelfAwareness => 2.0,
            CapabilityId::IntentionHierarchy => 1.0,
            CapabilityId::Exploration => {
                // 6n5. Hole 5: Exploration by uncertainty
                if self.state.tick_count % 15 == 0 {
                    let entropy = if self.big_transformer.vocab_size > 0 {
                        let sample_logits = self.big_transformer.predict_next(&[0, 1, 2]);
                        let mut probs = sample_logits.clone();
                        let max = probs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                        let mut sum = 0.0f32;
                        for v in probs.iter_mut() {
                            *v = (*v - max).exp();
                            sum += *v;
                        }
                        let denom = sum.max(1e-8);
                        for v in probs.iter_mut() {
                            *v /= denom;
                        }
                        exploration::ExplorationEngine::entropy(&probs)
                    } else {
                        0.0
                    };
                    let min_count = self
                        .morphogenesis
                        .concepts
                        .values()
                        .map(|c| c.count)
                        .min()
                        .unwrap_or(0);
                    let bus_var = self
                        .unified_bus
                        .slots
                        .iter()
                        .map(|s| {
                            let mean = s.vector.iter().sum::<f32>() / s.vector.len().max(1) as f32;
                            s.vector.iter().map(|v| (v - mean).powi(2)).sum::<f32>()
                                / s.vector.len().max(1) as f32
                        })
                        .sum::<f32>()
                        / self.unified_bus.slots.len().max(1) as f32;
                    let strategy = self.exploration.choose_strategy(
                        entropy,
                        min_count,
                        bus_var,
                        self.bus_predictor.pred_error_ema,
                    );
                    match strategy {
                        exploration::ExplorationStrategy::Directed { reasons } => {
                            self.action_log.push(format!(
                                "[EXPLORATION] Directed: {} | entropy={:.2} | min_count={}",
                                reasons.join(","),
                                entropy,
                                min_count
                            ));
                            // When directed, increase corpus load or train more
                            self.corpus_reader.sentences_per_tick =
                                (self.corpus_reader.sentences_per_tick + 1).min(10);
                        }
                        exploration::ExplorationStrategy::RandomUniform => {
                            self.action_log.push(format!(
                                "[EXPLORATION] Random uniform | entropy={:.2}",
                                entropy
                            ));
                            self.corpus_reader.sentences_per_tick = self
                                .corpus_reader
                                .sentences_per_tick
                                .saturating_sub(1)
                                .max(2);
                        }
                    }
                }

                1.0
            }
            CapabilityId::Gate => 0.5,
            CapabilityId::Evidence => 1.0,
            CapabilityId::Surprise => 0.5,
            CapabilityId::Epistemic => 1.0,
            CapabilityId::Circadian => 0.5,
            CapabilityId::Critic => {
                // === NUEVA ARQUITECTURA: Critic (Actor-Critic RL) ===
                // Update critic with bus state transition and program reward proxy
                let reward_proxy = self.motivation.discomfort * -0.5 + 0.2; // lower discomfort = positive reward
                let td_err = self.critic.update(
                    &self.slots_to_vector(),
                    reward_proxy,
                    &self.slots_to_vector(),
                );
                if self.state.tick_count % 50 == 0 {
                    self.action_log.push(format!(
                        "[CRITIC] td_err={:.4} | V(s)={:.3}",
                        td_err, self.critic.last_v
                    ));
                }

                2.0
            }
            CapabilityId::WorkingMemory => {
                // === NUEVA ARQUITECTURA: Working Memory read/write ===
                let wm_query = self.recurrent_state.readout(self.working_memory.dim);
                let _wm_read = self.working_memory.read(&wm_query);
                self.working_memory.write(&wm_query);
                if self.state.tick_count % 40 == 0 {
                    self.action_log
                        .push(format!("[WORKING_MEMORY] {}", self.working_memory.status()));
                }

                2.0
            }
            CapabilityId::ProgramInduction => {
                // === NUEVA ARQUITECTURA: Program Induction ===
                if self.state.tick_count % 80 == 0 {
                    let examples: Vec<(String, String)> = vec![
                        ("hello".to_string(), "hello".to_string()),
                        ("world".to_string(), "world".to_string()),
                    ];
                    if let Some(prog) = self.program_induction.induce(&examples) {
                        self.action_log
                            .push(format!("[PROG_INDUCTION] induced program: {}", prog));
                    }
                }

                2.0
            }
            CapabilityId::Counterfactual => {
                // 6e. Counterfactual: intervene on last concept and observe eliminated effects
                if self.state.tick_count % 70 == 0 && self.state.last_concept_id != 0 {
                    if let Some(c) = self.morphogenesis.concepts.get(&self.state.last_concept_id) {
                        if let Some(res) =
                            self.counterfactual
                                .intervene(&self.morphogenesis, &c.label, 3)
                        {
                            self.action_log.push(format!("[COUNTERFACTUAL] If '{}' did not cause -> {} effects eliminated | {} remain", res.intervention, res.eliminated.len(), res.still_present.len()));
                        }
                    }
                }

                2.0
            }
            CapabilityId::Analogy => 2.0,
            CapabilityId::Composition => {
                // 6d. Composition: detect conjunctive causal clusters and recurring compositions
                if self.state.tick_count % 60 == 0 {
                    let clusters = self
                        .composition
                        .detect_conjunctive_clusters(&self.morphogenesis);
                    let recurring = self
                        .composition
                        .detect_recurring_compositions(&self.morphogenesis);
                    if !clusters.is_empty() || !recurring.is_empty() {
                        self.action_log.push(format!(
                            "[COMPOSITION] {} conjunctive clusters | {} recurring compositions",
                            clusters.len(),
                            recurring.len()
                        ));
                    }
                }

                2.0
            }
            CapabilityId::Autonomy => {
                // 6. Continuous mode: record this iteration's stats
                self.continuous_mode.record(continuous::IterationStats {
                    iter: self.continuous_mode.n_iterations_total,
                    tick: self.state.tick_count,
                    n_concepts: self.morphogenesis.n_concepts(),
                    n_relations: self.morphogenesis.relation_count(),
                    n_executions: self.goal_executor.n_executions,
                    n_completions: self.goal_executor.n_completions,
                    n_grounding_facts: self.grounding.facts.len(),
                    n_analogies: self.analogy.n_inferences,
                    remaining_corpus: self.corpus_reader.remaining(),
                    goals_pending: self.goal_stack.stack.len(),
                });

                // 6b. Autonomy: introspect knowledge gaps and generate goals periodically
                if self.state.tick_count % 50 == 0 {
                    let gaps = self.autonomy.introspect(
                        &self.morphogenesis,
                        &self.grounding,
                        self.state.tick_count,
                    );
                    let n_goals = self.autonomy.generate_goals(
                        &gaps,
                        &mut self.goal_stack,
                        self.state.tick_count,
                    );
                    if n_goals > 0 {
                        self.action_log.push(format!(
                            "[AUTONOMY] Introspected {} gaps | generated {} goals",
                            gaps.len(),
                            n_goals
                        ));
                    }
                    // Self-awareness: structured metacognition
                    self.self_awareness.introspect(
                        &self.morphogenesis,
                        &self.goal_executor,
                        self.state.tick_count,
                    );
                    // Self-improvement: audit and apply top proposal
                    let proposals = self.self_improvement.audit(
                        self.morphogenesis.n_concepts(),
                        self.morphogenesis.relation_count(),
                        self.goal_executor.n_executions,
                        self.goal_executor.n_completions,
                        self.goal_executor.n_no_match,
                        self.analogy.n_attempts,
                        self.analogy.n_inferences,
                        self.grounding.facts.len(),
                        self.corpus_reader.total_processed,
                    );
                    if let Some(prop) = proposals.first() {
                        let new_val = self.self_improvement.propose_value(
                            match prop.param {
                                self_improvement::TunableParam::MorphoCreationThreshold => {
                                    self.morphogenesis.creation_threshold
                                }
                                self_improvement::TunableParam::MorphoTensionThreshold => {
                                    self.morphogenesis.tension_threshold
                                }
                                self_improvement::TunableParam::AnalogyMinScore => {
                                    self.analogy.min_combined_score
                                }
                                self_improvement::TunableParam::ExecutorMinMatch => {
                                    self.goal_executor.min_match_score
                                }
                                self_improvement::TunableParam::ExecutorCompletionThreshold => {
                                    self.goal_executor.completion_threshold
                                }
                                self_improvement::TunableParam::CorpusSentencesPerTick => {
                                    self.corpus_reader.sentences_per_tick as f32
                                }
                                self_improvement::TunableParam::AutonomyMaxGoals => {
                                    self.autonomy.max_goals_per_run as f32
                                }
                            },
                            prop,
                        );
                        match prop.param {
                            self_improvement::TunableParam::MorphoCreationThreshold => {
                                self.morphogenesis.creation_threshold = new_val
                            }
                            self_improvement::TunableParam::MorphoTensionThreshold => {
                                self.morphogenesis.tension_threshold = new_val
                            }
                            self_improvement::TunableParam::AnalogyMinScore => {
                                self.analogy.min_combined_score = new_val
                            }
                            self_improvement::TunableParam::ExecutorMinMatch => {
                                self.goal_executor.min_match_score = new_val
                            }
                            self_improvement::TunableParam::ExecutorCompletionThreshold => {
                                self.goal_executor.completion_threshold = new_val
                            }
                            self_improvement::TunableParam::CorpusSentencesPerTick => {
                                self.corpus_reader.sentences_per_tick = new_val as usize
                            }
                            self_improvement::TunableParam::AutonomyMaxGoals => {
                                self.autonomy.max_goals_per_run = new_val as usize
                            }
                        };
                        self.self_improvement
                            .record(self_improvement::ParamAdjustment {
                                param: prop.param.clone(),
                                old_value: new_val, // placeholder; actual old would need snapshot
                                new_value: new_val,
                                reason: prop.reason.clone(),
                                tick: self.state.tick_count,
                                metric_before: 0.0,
                                metric_after: None,
                            });
                        self.action_log.push(format!(
                            "[SELF_IMPROVE] Applied {:?} -> {:.3} | reason: {}",
                            prop.param, new_val, prop.reason
                        ));
                    }
                }

                2.0
            }
            CapabilityId::GoalExecutor => {
                // 5b. GOAL EXECUTOR: pull top goal, match to capability, execute, mark progress
                // This is the closing of the cognitive loop: planned goals -> behavior.
                if let Some(top_id) = self.goal_stack.pop_top() {
                    // Don't re-execute already completed/failed goals at the top
                    let (label, completed, failed) = match self.goal_stack.goals.get(&top_id) {
                        Some(g) => (g.label.clone(), g.completed, g.failed),
                        None => ("".to_string(), false, false),
                    };
                    if !completed && !failed && !label.is_empty() {
                        // Snapshot pre-state for delta measurement
                        let pre_concepts = self.morphogenesis.n_concepts();
                        let pre_relations = self.morphogenesis.relation_count();
                        let pre_error = self.self_model.mean_error();
                        let pre_discomfort = self.motivation.discomfort;
                        let pre_grounding_facts = self.grounding.facts.len();
                        let pre_analogies = self.analogy.n_inferences;

                        let (target, score) = self.goal_executor.match_goal_to_capability(&label);

                        if score < self.goal_executor.min_match_score {
                            self.goal_executor
                                .record_no_match(top_id, label.clone(), score);
                            self.action_log.push(format!(
                                "[EXEC] gid={} '{}' -> NO_MATCH (score={:.2})",
                                top_id, label, score
                            ));
                            // Rotate to bottom so others get a chance
                            self.goal_stack.stack.retain(|&g| g != top_id);
                            self.goal_stack.stack.insert(0, top_id);
                        } else {
                            let mut note = String::new();
                            // PREDICTIVE LOOP: predict outcome BEFORE dispatch using self_model
                            let pre_state_vec = self.encode_state(self.state.last_input.len(), 0);
                            let action_vec = predictive_loop::encode_action_target(&target);
                            let predicted =
                                self.self_model.predict_outcome(&pre_state_vec, &action_vec);
                            let tick_pre = self.state.tick_count;
                            // Dispatch to subsystem based on matched capability
                            match target {
                                goal_executor::ActionTarget::Exploration => {
                                    // Retrieve a distant episode and plan revisit
                                    let dummy = vec![0.5f32; 32];
                                    let retrieved = self.hippocampus.retrieve(&dummy);
                                    if let Some((ep, sim)) = retrieved.first() {
                                        self.temporal.plan(
                                            self.state.tick_count + 5,
                                            &format!("revisit_concept_{}", ep.concept_id),
                                        );
                                        note = format!(
                                            "retrieved self.state.tick_count={} sim={:.2}",
                                            ep.tick, sim
                                        );
                                    } else {
                                        note = "no episodes retrieved".to_string();
                                    }
                                }
                                goal_executor::ActionTarget::Metacognition => {
                                    // Train self_model with current outcome (defensive: only if last_state populated)
                                    let outcome = vec![0.5, 0.5, self.motivation.discomfort];
                                    let next = self.encode_state(self.state.last_input.len(), 0);
                                    let err = self.self_model.train(&outcome, &next);
                                    self.metacognition.observe(
                                        0.0,
                                        err,
                                        self.morphogenesis.n_concepts(),
                                    );
                                    note = format!("trained, err={:.4}", err);
                                }
                                goal_executor::ActionTarget::Physics => {
                                    // Run a brief future simulation
                                    let future = self.physics.simulate_future(3);
                                    note = format!(
                                        "simulated {} steps over {} objects",
                                        future.len(),
                                        self.physics.objects.len()
                                    );
                                }
                                goal_executor::ActionTarget::Causality => {
                                    // Derive transitive closure
                                    let n = inference::InferenceEngine::derive_transitive_closure(
                                        &mut self.morphogenesis,
                                        3,
                                    );
                                    note = format!("derived {} transitive edges", n);
                                }
                                goal_executor::ActionTarget::Memory => {
                                    // Prune with high count threshold and run a small evolution step
                                    self.morphogenesis.prune(2, 1000, self.state.tick_count);
                                    note = format!(
                                        "pruned, concepts now={}",
                                        self.morphogenesis.n_concepts()
                                    );
                                }
                                goal_executor::ActionTarget::Semantics => {
                                    self.semantics.compute_embeddings();
                                    note = format!(
                                        "recomputed embeddings, vocab={}",
                                        self.semantics.vocab_size
                                    );
                                }
                                goal_executor::ActionTarget::Perception => {
                                    // Capture and analyze a frame
                                    let screen = self.computer.capture_screen(64, 48);
                                    let img = vision::ImageBuffer {
                                        width: 64,
                                        height: 48,
                                        pixels: screen,
                                    };
                                    let v = self.vision.analyze(&img);
                                    note = format!("vision blobs={}", v.blobs.len());
                                }
                                goal_executor::ActionTarget::Goal => {
                                    // Push a child goal to break this one down via causal trace
                                    let plan = inference::InferenceEngine::backward_chain_to_goal(
                                        &self.morphogenesis,
                                        &label,
                                        2,
                                    );
                                    note = format!(
                                        "decomposed into {} sub-steps",
                                        plan.len().saturating_sub(1)
                                    );
                                }
                                goal_executor::ActionTarget::NoMatch => {}
                            }

                            // Measure delta after execution
                            let post_concepts = self.morphogenesis.n_concepts();
                            let post_relations = self.morphogenesis.relation_count();
                            let post_error = self.self_model.mean_error();
                            let post_discomfort = self.motivation.discomfort;
                            let post_grounding_facts = self.grounding.facts.len();
                            let post_analogies = self.analogy.n_inferences;
                            let concept_delta = (post_concepts as f32 - pre_concepts as f32).abs()
                                / (pre_concepts as f32 + 1.0);
                            let relation_delta = (post_relations as f32 - pre_relations as f32)
                                .abs()
                                / (pre_relations as f32 + 1.0);
                            let error_delta = (pre_error - post_error).max(0.0);
                            let discomfort_delta = (pre_discomfort - post_discomfort).max(0.0);
                            let grounding_delta =
                                (post_grounding_facts as f32 - pre_grounding_facts as f32).abs()
                                    / (pre_grounding_facts as f32 + 1.0);
                            let analogy_delta = (post_analogies as f32 - pre_analogies as f32)
                                .abs()
                                / (pre_analogies as f32 + 1.0);
                            let progress_delta = concept_delta
                                + relation_delta * 1.5
                                + error_delta
                                + discomfort_delta
                                + grounding_delta * 0.5
                                + analogy_delta * 0.8;

                            let completed =
                                progress_delta >= self.goal_executor.completion_threshold;
                            if completed {
                                self.goal_stack.complete_goal(top_id, progress_delta);
                                // Remove completed goal from stack
                                self.goal_stack.stack.retain(|&g| g != top_id);
                            } else {
                                // Update partial progress
                                if let Some(g) = self.goal_stack.goals.get_mut(&top_id) {
                                    g.progress = (g.progress + progress_delta * 0.5).min(1.0);
                                }
                                // Rotate: move to bottom so other goals get attention
                                self.goal_stack.stack.retain(|&g| g != top_id);
                                self.goal_stack.stack.insert(0, top_id);
                            }

                            // PREDICTIVE LOOP: train self_model with REAL outcome
                            let actual_success: f32 = if completed { 1.0 } else { 0.0 };
                            let actual_duration: f32 = (self.state.tick_count - tick_pre) as f32;
                            let actual_growth: f32 = post_concepts as f32 - pre_concepts as f32;
                            let actual_outcome =
                                vec![actual_success, actual_duration, actual_growth];
                            let post_state_vec = self.encode_state(self.state.last_input.len(), 0);
                            let _train_err =
                                self.self_model.train(&actual_outcome, &post_state_vec);
                            self.predictive_loop.n_trainings += 1;
                            // Record prediction <-> actual
                            let pred_short: Vec<f32> = predicted.iter().take(3).cloned().collect();
                            self.predictive_loop
                                .record(predictive_loop::PredictionRecord {
                                    tick: tick_pre,
                                    goal_id: top_id,
                                    goal_label: label.clone(),
                                    action: target.clone(),
                                    predicted: pred_short,
                                    actual: actual_outcome.clone(),
                                    error_per_dim: vec![], // computed inside record
                                    brier_success: 0.0,    // computed inside record
                                });

                            // G) RL fine-tuning: use error_delta as reward for transformer
                            if self.transformer.vocab_size > 0 {
                                let rl_reward = error_delta * 2.0 - 0.5; // positive if error decreased significantly
                                let ctx_tokens: Vec<usize> = label
                                    .to_lowercase()
                                    .split(|c: char| !c.is_alphanumeric())
                                    .filter(|w| !w.is_empty())
                                    .filter_map(|w| self.semantics.vocab.get(w).copied())
                                    .collect();
                                if !ctx_tokens.is_empty() && rl_reward.abs() > 0.1 {
                                    // Pick a "predicted token" from the label (last word)
                                    let predicted_tok = ctx_tokens.last().copied().unwrap_or(0);
                                    self.transformer.rl_finetune_step(
                                        &ctx_tokens,
                                        predicted_tok,
                                        rl_reward,
                                    );
                                }
                            }

                            self.goal_executor.record(goal_executor::ExecutionResult {
                                goal_id: top_id,
                                goal_label: label.clone(),
                                action_target: target.clone(),
                                match_score: score,
                                progress_delta,
                                completed,
                                failed: false,
                                note: note.clone(),
                            });

                            self.action_log.push(format!(
                                "[EXEC] gid={} '{}' -> {:?} | match={:.2} | delta={:.3} | {} | {}",
                                top_id,
                                label,
                                target,
                                score,
                                progress_delta,
                                if completed {
                                    "COMPLETED"
                                } else {
                                    "in_progress"
                                },
                                note,
                            ));
                        }
                    } else {
                        // Already completed/failed: just pop without re-executing
                        self.goal_stack.stack.retain(|&g| g != top_id);
                    }
                }

                2.0
            }
            CapabilityId::LanguageGen => {
                // 6h. Language generation: explain a random concept from the space
                if self.state.tick_count % 55 == 0 {
                    let keys: Vec<u64> = self.morphogenesis.concepts.keys().copied().collect();
                    if !keys.is_empty() {
                        let cid = keys[self.state.tick_count as usize % keys.len()];
                        if let Some(c) = self.morphogenesis.concepts.get(&cid) {
                            let explanation =
                                self.language_gen
                                    .explain_chain(&self.morphogenesis, &c.label, 3);
                            self.action_log.push(format!(
                                "[LANGUAGE_GEN] explain '{}' -> {}",
                                c.label, explanation
                            ));
                        }
                    }
                }

                2.0
            }
            CapabilityId::SyntheticVision => 2.0,
            CapabilityId::PredictiveLoop => 2.0,
            CapabilityId::Curriculum => {
                // 6f. Curriculum: auto-load corpus for weakest domain
                if self.state.tick_count % 80 == 0 {
                    if let Some((domain, path, density)) =
                        self.curriculum.next_corpus_file(&self.self_awareness)
                    {
                        if self.corpus_reader.load_file(&path).is_ok() {
                            self.curriculum.record_load(&domain);
                            self.action_log.push(format!(
                                "[CURRICULUM] Auto-loaded '{}' for weak domain {} (density={:.3})",
                                path, domain, density
                            ));
                        }
                    }
                }

                // === META-CURRICULUM: adapt corpus bias based on generative controller parse rate ===
                self.corpus_massive.program_bias = if self.gen_metrics.parse_rate() < 0.05 {
                    0.8
                } else if self.gen_metrics.parse_rate() < 0.20 {
                    0.6
                } else {
                    0.3
                };

                1.0
            }
            CapabilityId::MemoryClustering => {
                // 6c. Memory clustering: cluster hippocampus episodes and consolidate to long-term memory
                if self.state.tick_count % 30 == 0 {
                    let n_cl = self.memory_clustering.cluster_all(&self.hippocampus);
                    let n_cons = self.memory_clustering.consolidate(
                        &mut self.morphogenesis,
                        3,
                        self.state.tick_count,
                    );
                    if n_cl > 0 || n_cons > 0 {
                        self.action_log.push(format!(
                            "[MEMCLUSTER] {} clusters | {} consolidated to concepts",
                            n_cl, n_cons
                        ));
                    }
                }

                // 6n4. Hole 2: Sleep / Memory Consolidation
                if self.state.tick_count % 120 == 0 {
                    let old_episodes: Vec<hippocampus::Episode> = self
                        .hippocampus
                        .episodes
                        .iter()
                        .filter(|ep| self.state.tick_count.saturating_sub(ep.tick) > 60)
                        .cloned()
                        .collect();
                    if !old_episodes.is_empty() {
                        let mut space = morphogenesis::ConceptSpace::new();
                        for ep in &old_episodes {
                            space.add_sample(&ep.embedding, &ep.actions_summary, ep.tick, 0.2);
                        }
                        let n_consolidated = self.memory_clustering.consolidate(
                            &mut self.morphogenesis,
                            3,
                            self.state.tick_count,
                        );
                        if n_consolidated > 0 {
                            self.action_log.push(format!(
                                "[SLEEP] Consolidated {} episode clusters into long-term concepts",
                                n_consolidated
                            ));
                        }
                        // Prune old hippocampus episodes
                        let before = self.hippocampus.episodes.len();
                        self.hippocampus
                            .episodes
                            .retain(|ep| self.state.tick_count.saturating_sub(ep.tick) <= 90);
                        let after = self.hippocampus.episodes.len();
                        if before > after {
                            self.action_log.push(format!(
                                "[SLEEP] Pruned {} old episodes | remaining={}",
                                before - after,
                                after
                            ));
                        }
                    }
                }

                2.0
            }
            CapabilityId::Gridworld => {
                // 6o. GridWorld: simple embodied policy (move toward goal)
                if self.state.tick_count % 25 == 0 {
                    let gx = self.gridworld.goal_x;
                    let gy = self.gridworld.goal_y;
                    let ax = self.gridworld.agent_x;
                    let ay = self.gridworld.agent_y;
                    let action = if ax < gx {
                        gridworld::GridAction::MoveRight
                    } else if ax > gx {
                        gridworld::GridAction::MoveLeft
                    } else if ay < gy {
                        gridworld::GridAction::MoveDown
                    } else {
                        gridworld::GridAction::MoveUp
                    };
                    let action_label = format!("{:?}", action);
                    let res = self.gridworld.step(action);
                    self.action_log.push(format!(
                        "[GRIDWORLD] {} -> success={} reward={:.2} | steps={} goals={}",
                        action_label,
                        res.success,
                        res.reward,
                        self.gridworld.n_steps,
                        self.gridworld.n_goal_reached
                    ));
                }

                1.0
            }
            CapabilityId::AgentMesh => 1.0,
            CapabilityId::Compositional => {
                // 6g. Compositional: parse last input into recursive tree
                if !self.state.last_input.is_empty() && self.state.tick_count % 40 == 0 {
                    let tree = self
                        .compositional
                        .parse_to_tree(&self.state.last_input, &self.semantics);
                    let atoms = compositional::Compositional::flatten_to_atoms(&tree);
                    if !atoms.is_empty() {
                        self.action_log.push(format!(
                            "[COMPOSITIONAL] Parsed input -> {} atoms",
                            atoms.len()
                        ));
                    }
                }

                2.0
            }
            CapabilityId::NeuralExtractors => 0.0,
            CapabilityId::World3D => {
                // 6j. World3D: simulate physics
                if self.state.tick_count % 20 == 0 {
                    if self.world3d.objects.is_empty() {
                        // Seed with some objects
                        self.world3d.spawn("sphere_a", 0.0, 50.0, 0.0, 1.0, 2.0);
                        self.world3d.spawn("sphere_b", 5.0, 60.0, 0.0, 2.0, 3.0);
                        self.world3d.spawn("sphere_c", -3.0, 40.0, 2.0, 0.5, 1.5);
                    }
                    self.world3d.simulate(10);
                    // INTERCONNECTION: world3d -> world_model (sync fast-moving objects)
                    for obj in self.world3d.objects.iter() {
                        let speed = obj.vel.len();
                        if speed > 0.5 {
                            let px = ((obj.pos.x + 10.0) as u32).saturating_mul(16);
                            let py = ((obj.pos.y + 10.0) as u32).saturating_mul(12);
                            let blobs = vec![vision::Blob {
                                x: px,
                                y: py,
                                w: 10,
                                h: 10,
                                area: (obj.mass * 10.0) as u32,
                                centroid_x: px as f32,
                                centroid_y: py as f32,
                                contour: vec![(px, py)],
                            }];
                            self.world_model
                                .track_frame(&blobs, 320, 240, self.state.tick_count);
                        }
                    }
                    self.action_log
                        .push(format!("[WORLD3D] {}", self.world3d.status()));
                }

                2.0
            }
            CapabilityId::PluginSystem => 1.0,
            CapabilityId::UnifiedPerception => {
                // 6n. Multimodal: fuse vision + text if vision was captured
                if self.state.tick_count % 10 == 0 {
                    // Simulate visual capture
                    let visual_feat = self.multimodal.extract_features(3, 12, 0.7, 2);
                    // Dummy text embeddings
                    let text_emb = vec![vec![0.1f32; 256]; 4];
                    let fused = self.multimodal.fuse(&text_emb, &visual_feat);
                    // INTERCONNECTION: multimodal -> big_transformer (generate pseudo-sentence from visual + train)
                    if !fused.is_empty() && self.big_transformer.vocab_size > 0 {
                        let pseudo_text = format!(
                            "visual scene with {} blobs motion {}",
                            visual_feat.blob_count,
                            if visual_feat.blob_count > 2 {
                                "active"
                            } else {
                                "static"
                            }
                        );
                        let tokens: Vec<usize> = pseudo_text
                            .to_lowercase()
                            .split(|c: char| !c.is_alphanumeric())
                            .filter(|w| !w.is_empty())
                            .filter_map(|w| self.semantics.vocab.get(w).copied())
                            .collect();
                        if tokens.len() >= 2 {
                            let loss = self.big_transformer.train_on_sentence(&tokens);
                            self.benchmark.report_train_loss(loss);
                            self.action_log.push(format!("[INTERCONN] multimodal->big_transformer | pseudo-text='{}' | loss={:.3}", pseudo_text, loss));
                        }
                    }
                    self.action_log
                        .push(format!("[MULTIMODAL] {}", self.multimodal.status()));
                }

                // === NUEVA ARQUITECTURA: Unified Hub (cross-domain transfer) ===
                let hub_vision = self.unified_hub.project("vision", &self.embodiment.sensors);
                let _hub_lang = self.unified_hub.project(
                    "language",
                    &self
                        .semantics
                        .embeddings
                        .get(0)
                        .cloned()
                        .unwrap_or_default(),
                );
                if let Some((domain, sim)) =
                    self.unified_hub.nearest_cross_domain("vision", &hub_vision)
                {
                    if self.state.tick_count % 80 == 0 {
                        self.action_log
                            .push(format!("[HUB] vision <-> {} | sim={:.3}", domain, sim));
                    }
                }

                2.0
            }
            CapabilityId::UnifiedBus => {
                // 2. Update workspace agent states with current system condition
                self.workspace
                    .set_perception_input(self.state.last_input.len());
                self.workspace
                    .set_goal_state(self.goal_stack.stack.len(), self.motivation.discomfort);
                self.workspace
                    .set_memory_load(self.morphogenesis.n_concepts() + self.temporal.events.len());
                self.workspace.set_meta_state(
                    self.metacognition.self_model_error_ema,
                    self.motivation.drives.curiosity,
                );
                self.workspace.set_novelty(0.05); // autonomous self.state.tick_count = low novelty by default

                // 3. WORKSPACE ORCHESTRATOR: winner-take-all with serial conscious processing
                // Hole 3: Attention stack - if topic is set, bias workspace toward relevant agents
                if !self.current_topic.is_empty() {
                    self.workspace.set_novelty(0.15);
                }
                let broadcast = self.workspace.tick(self.state.tick_count);
                self.current_winner = broadcast
                    .as_ref()
                    .map(|b| b.agent_name.clone())
                    .unwrap_or_else(|| "none".to_string());
                // Hole 3: Push topic to attention stack on interruption (auto-debug error)
                if self
                    .auto_debug
                    .diagnostics
                    .iter()
                    .filter(|d| d.level == "error")
                    .count()
                    > 0
                {
                    if !self.current_topic.is_empty() {
                        self.attention_stack.push(self.current_topic.clone());
                        self.action_log.push(format!(
                            "[ATTN_STACK] Pushed topic '{}' (interruption by auto-debug)",
                            self.current_topic
                        ));
                    }
                    self.current_topic = "fix_errors".to_string();
                }
                // Hole 3: Pop from stack when errors resolved
                if self
                    .auto_debug
                    .diagnostics
                    .iter()
                    .filter(|d| d.level == "error")
                    .count()
                    == 0
                    && !self.attention_stack.is_empty()
                {
                    if let Some(popped) = self.attention_stack.pop() {
                        self.current_topic = popped.clone();
                        self.action_log.push(format!(
                            "[ATTN_STACK] Restored topic '{}' | stack_depth={}",
                            popped,
                            self.attention_stack.len()
                        ));
                    }
                }
                if let Some(ref b) = broadcast {
                    self.action_log.push(format!("[WORKSPACE] Broadcast from {} | confidence={:.2} | self.state.tick_count={} | broadcasts={} | skipped={}",
                                b.agent_name, b.confidence, b.tick, self.workspace.n_broadcasts, self.workspace.n_skipped));
                } else {
                    self.action_log.push(format!(
                        "[WORKSPACE] No broadcast | threshold={:.2} | below ignition",
                        self.workspace.global_threshold
                    ));
                }

                // 5. Execute ONLY the winning subsystem (serial attention)
                match self.current_winner.as_str() {
                    "perception" => {
                        // Vision + world model + grounding + physics
                        if self.state.tick_count % 10 == 0 {
                            let screen = self.computer.capture_screen(320, 240);
                            let img = vision::ImageBuffer {
                                width: 320,
                                height: 240,
                                pixels: screen,
                            };
                            let vresult = self.vision.analyze(&img);
                            let wm_actions = self.world_model.track_frame(
                                &vresult.blobs,
                                320,
                                240,
                                self.state.tick_count,
                            );
                            self.action_log.extend(wm_actions);
                            self.action_log.push(format!(
                                "[GARM/PERCEPTION] Vision captured | blobs={}",
                                vresult.blobs.len()
                            ));
                            // Sync physics engine with tracked objects
                            for obj in self.world_model.objects.values() {
                                if obj.visible {
                                    if !self.physics.objects.contains_key(&obj.id) {
                                        self.physics.register_object(
                                            obj.cx,
                                            obj.cy,
                                            0.05,
                                            0.05,
                                            obj.label.clone(),
                                        );
                                    } else {
                                        self.physics.update_position(obj.id, obj.cx, obj.cy);
                                    }
                                } else {
                                    self.physics.mark_invisible(obj.id);
                                }
                                // If occlusions detected, add symbolic relations
                                if let Some(occluder) = obj.occluded_by {
                                    let cid_obj = self
                                        .morphogenesis
                                        .concepts
                                        .values()
                                        .find(|c| c.label == format!("obj_{}", obj.id))
                                        .map(|c| c.id);
                                    let cid_occ = self
                                        .morphogenesis
                                        .concepts
                                        .values()
                                        .find(|c| c.label == format!("obj_{}", occluder))
                                        .map(|c| c.id);
                                    if let (Some(a), Some(b)) = (cid_obj, cid_occ) {
                                        self.morphogenesis.add_relation(a, "occluded_by", b);
                                    }
                                }
                            }
                            // Infer physical supports
                            self.physics.infer_supports();
                            // Simulate future
                            let future = self.physics.simulate_future(5);
                            if !future.is_empty() {
                                let last = future.last().unwrap();
                                self.action_log.push(format!("[PHYSICS] Simulated {} steps | objects={} | final_positions={}", future.len(), last.len(), self.physics.status()));
                            }
                        }
                    }
                    "goal" => {
                        // Autonomous action + goal stack execution
                        let idle_thresh = (5.0 * self.mood.idle_threshold_multiplier()) as u64;
                        if self.state.idle_ticks >= idle_thresh
                            && self
                                .motivation
                                .should_act_autonomously(self.state.idle_ticks)
                        {
                            self.action_log.push(format!(
                                "[MOTIVE] Autonomous trigger | dominant={} | discomfort={:.2}",
                                self.motivation.dominant_drive(),
                                self.motivation.discomfort
                            ));
                            let interventions = vec![
                                ("action_success".to_string(), 0.2f32),
                                ("action_success".to_string(), 0.8f32),
                            ];
                            let best = self.causality.best_intervention(&interventions, "load_avg");
                            let chosen_action = best.map(|(node, val)| {
                                        let explanation = self.causality.explain_effect(&node, "load_avg");
                                        self.action_log.push(format!("[CAUSAL] Planning | best_intervention={}={:.2} | explanation: {:?}", node, val, explanation));
                                        if val > 0.5 { 2usize } else { 0usize }
                                    }).unwrap_or(0usize);
                            match chosen_action {
                                0 => {
                                    self.computer.scan_processes_deep();
                                    let top = self.computer.top_processes(1);
                                    if let Some(p) = top.first() {
                                        self.action_log.push(format!(
                                            "[GARM/AUTO-ACTION] Scanning system | top: {} (PID {})",
                                            p.name, p.pid
                                        ));
                                    }
                                }
                                2 => {
                                    let _ = self
                                        .computer
                                        .write_file("/tmp/eden_heartbeat.txt", "alive");
                                    self.action_log.push(format!("[GARM/AUTO-ACTION] Wrote heartbeat /tmp/eden_heartbeat.txt | causal_plan={}", chosen_action));
                                }
                                _ => {}
                            }
                            let obs = format!("self.state.tick_count={} | discomfort={:.2} | dominant={} | chosen_action={}\n",
                                        self.state.tick_count, self.motivation.discomfort, self.motivation.dominant_drive(), chosen_action);
                            let _ = self
                                .computer
                                .write_file("/tmp/eden_autonomous_log.txt", &obs);
                            self.state.idle_ticks = 0;
                        }
                        // Goal stack execution: handled by goal_executor at step 5b. Workspace 'goal'
                        // winner now only logs that goals are pending — executor will handle them.
                        if !self.goal_stack.stack.is_empty() {
                            self.action_log.push(format!(
                                "[GOALS] {} goals pending in stack (executor will handle)",
                                self.goal_stack.stack.len()
                            ));
                        }
                    }
                    "memory" => {
                        // Morphogenesis pruning + evolution (memory consolidation)
                        if self.state.tick_count % 100 == 0 {
                            self.morphogenesis.prune(3, 500, self.state.tick_count);
                        }
                        if self.state.tick_count % 50 == 0 {
                            let weights_size = self.neural.n_weights();
                            let fitness_fn = |genome: &[f32]| {
                                let mut net =
                                    crate::eden_garm::capabilities::neural::OnlineNetwork::new(
                                        8 + 8,
                                        12,
                                        6,
                                        0.05,
                                    );
                                let w = &genome[..genome.len().min(weights_size)];
                                net.set_weights(w);
                                let err = self.self_model.recent_error(20);
                                if err <= 0.0 {
                                    1.0
                                } else {
                                    (1.0 / (1.0 + err)).clamp(0.0, 1.0)
                                }
                            };
                            self.evolution.evaluate(fitness_fn);
                            self.evolution.evolve();
                            self.evolution.update_hall_of_fame();
                            let boost = self.metacognition.recommended_mutation_boost();
                            if boost > 0.0 {
                                self.evolution.mutation_strength =
                                    (self.evolution.mutation_strength + boost).clamp(0.01, 0.5);
                                self.action_log.push(format!(
                                    "[META] Boosted mutation_strength to {:.3} due to stagnation",
                                    self.evolution.mutation_strength
                                ));
                            }
                            self.action_log.push(format!(
                                "[EVOLVE] HoF size: {}",
                                self.evolution.hall_of_fame.len()
                            ));
                            if let Some(best) = self.evolution.best_genome() {
                                let best_weights: Vec<f32> = best[..weights_size].to_vec();
                                let program_tail = &best[weights_size..];
                                if !program_tail.is_empty() {
                                    self.meta_vm =
                                        Some(meta_evolution::MetaVM::from_genome(program_tail));
                                    let n_ops = self.meta_vm.as_ref().unwrap().program.len();
                                    self.action_log.push(format!(
                                        "[METAEVO] Decoded {} ops from genome tail",
                                        n_ops
                                    ));
                                }
                                let best_ind = self.evolution.individuals.first().unwrap();
                                let new_hidden = best_ind.get_hidden_size();
                                let new_lr = best_ind.get_lr() * self.mood.lr_multiplier();
                                self.neural =
                                    crate::eden_garm::capabilities::neural::OnlineNetwork::new(
                                        8 + 8,
                                        new_hidden,
                                        6,
                                        new_lr,
                                    );
                                self.neural.set_weights(&best_weights);
                                let best_swarm_idx = self.swarm.best_idx;
                                self.swarm.specialists[best_swarm_idx].net =
                                    crate::eden_garm::capabilities::neural::OnlineNetwork::new(
                                        8 + 8,
                                        new_hidden,
                                        6,
                                        new_lr,
                                    );
                                self.swarm.specialists[best_swarm_idx]
                                    .net
                                    .set_weights(&best_weights);
                                self.action_log.push(format!("[NEURAL] Rebuilt with hidden={} lr={:.4} from genome | swarm specialist[{}] synced", new_hidden, new_lr, best_swarm_idx));
                                self.action_log.push(format!("[EVOLVE] Gen {} | best_fit {:.4} | mean_fit {:.4} | species {} | Genome applied to neural",
                                            self.evolution.generation, self.evolution.best_fitness(), self.evolution.mean_fitness(), self.evolution.species_count()));
                                let best_fit = self.evolution.best_fitness();
                                if best_fit > 0.7 && !program_tail.is_empty() {
                                    match self.plugins.generate_and_load(program_tail) {
                                        Ok(name) => {
                                            self.action_log.push(format!("[PLUGIN] Generated and loaded '{}' from evolved genome | fit={:.3}", name, best_fit));
                                        }
                                        Err(e) => {
                                            self.action_log
                                                .push(format!("[PLUGIN] Generation failed: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                        // Directed exploration
                        if self.motivation.drives.curiosity > 0.7 {
                            if let Some((rare_intent, _)) = self
                                .theory_of_mind
                                .intent_freq
                                .iter()
                                .min_by_key(|(_, v)| *v)
                            {
                                self.temporal.plan(self.state.tick_count + 5, rare_intent);
                                self.action_log.push(format!(
                                    "[META] Exploration mode | planning rare intent: {}",
                                    rare_intent
                                ));
                            }
                        }
                        // Symbolic safety check
                        if self.state.last_concept_id != 0 {
                            let risks = self
                                .morphogenesis
                                .infer_transitive(self.state.last_concept_id, "causes_risk");
                            if !risks.is_empty() {
                                self.action_log.push(format!("[SYMBOLIC] WARNING | concept {} transitively causes {} risk(s)", self.state.last_concept_id, risks.len()));
                            }
                        }
                    }
                    "metacognition" => {
                        // Plugins + metacognition + self-model training
                        let last_input_ref = &self.state.last_input;
                        for p in &mut self.plugins.plugins {
                            let plugin_out = p.process(last_input_ref);
                            if !plugin_out.is_empty() && plugin_out != *last_input_ref {
                                self.action_log.push(format!(
                                    "[PLUGIN] {} processed input -> '{}'",
                                    p.name, plugin_out
                                ));
                            }
                        }
                        // Self-model training with recent experience
                        let actual_outcome = vec![
                            0.5, // success placeholder
                            0.5, // actions placeholder
                            0.0,
                            0.0,
                            self.motivation.discomfort,
                            (self.mood.valence + 1.0) / 2.0,
                        ];
                        let actual_next = self.encode_state(self.state.last_input.len(), 0);
                        let err = self.self_model.train(&actual_outcome[..3], &actual_next);
                        self.metacognition
                            .observe(0.0, err, self.morphogenesis.n_concepts());
                        self.action_log.push(format!(
                            "[SELF] Training self.state.tick_count | err={:.4}",
                            err
                        ));
                    }
                    "exploration" => {
                        // Hippocampus-based novelty search: retrieve distant episodes and plan exploration
                        let dummy_query = vec![0.5f32; 32];
                        let retrieved = self.hippocampus.retrieve(&dummy_query);
                        if let Some((ep, sim)) = retrieved.first() {
                            self.action_log.push(format!("[EXPLORATION] Retrieved distant episode self.state.tick_count={} | sim={:.2} | planning revisit", ep.tick, sim));
                            self.temporal.plan(
                                self.state.tick_count + 10,
                                &format!("revisit_concept_{}", ep.concept_id),
                            );
                        } else {
                            self.action_log.push(format!("[EXPLORATION] No distant episodes | initiating random concept search"));
                        }
                    }
                    "social" => {
                        // Multi-agent synchronization attempt
                        if !self.multi_agent.peers.is_empty() {
                            let msg = crate::eden_garm::capabilities::multi_agent::EdenMessage {
                                sender_id: "eden".to_string(),
                                tick: self.state.tick_count,
                                intent: format!("social_sync | mood={:.2}", self.mood.valence),
                                confidence: self.mood.valence.abs(),
                                embedding: self
                                    .semantics
                                    .sentence_embedding(&self.state.last_input),
                                predicted_success: 0.5,
                            };
                            let _ = msg; // placeholder: in future, msg would be sent to peers
                            self.action_log.push(format!(
                                "[SOCIAL] Broadcasting state to {} peers",
                                self.multi_agent.peers.len()
                            ));
                        } else {
                            self.action_log
                                .push(format!("[SOCIAL] No peers to synchronize with"));
                        }
                    }
                    "creativity" => {
                        // Combine two random concepts via symbolic relation
                        let concepts: Vec<u64> =
                            self.morphogenesis.concepts.keys().copied().collect();
                        if concepts.len() >= 2 {
                            let a = concepts[self.state.tick_count as usize % concepts.len()];
                            let b = concepts[(self.state.tick_count as usize + 1) % concepts.len()];
                            self.morphogenesis.add_relation(a, "similar_to", b);
                            self.action_log.push(format!("[CREATIVITY] Linked concept {} -> similar_to -> {} via random combination", a, b));
                        }
                    }
                    _ => {
                        // No winner → minimal processing, just passive updates
                        self.action_log.push(format!(
                            "[WORKSPACE] Serial attention idle | no subsystem active"
                        ));
                    }
                }

                // 6q. BRU: Unified Bus — all modules project to shared representation space
                {
                    // Project each key module into the bus
                    let mut bigtf_vec = vec![0.0f32; unified_bus::BUS_DIM];
                    bigtf_vec[0] = self.big_transformer.n_train_steps as f32 / 10000.0;
                    bigtf_vec[1] = self.big_transformer.vocab_size as f32 / 5000.0;
                    bigtf_vec[2] = self.big_transformer.adapter_dim as f32 / 128.0;
                    bigtf_vec[3] = self.big_transformer.total_params as f32 / 40_000_000.0;
                    self.unified_bus
                        .project("big_transformer", &bigtf_vec, self.state.tick_count);

                    let mut planner_vec = vec![0.0f32; unified_bus::BUS_DIM];
                    if let Some(ref plan) = self.planner.best {
                        planner_vec[0] = plan.score;
                        planner_vec[1] = plan.predicted_final_discomfort;
                        planner_vec[2] = plan.steps.len() as f32 / 10.0;
                    }
                    planner_vec[3] = self.planner.n_simulations as f32 / 20.0;
                    self.unified_bus
                        .project("planner", &planner_vec, self.state.tick_count);

                    let mut morpho_vec = vec![0.0f32; unified_bus::BUS_DIM];
                    morpho_vec[0] = self.morphogenesis.tension();
                    morpho_vec[1] = self.morphogenesis.n_concepts() as f32 / 1000.0;
                    morpho_vec[2] = self.morphogenesis.relation_count() as f32 / 1000.0;
                    self.unified_bus
                        .project("morphogenesis", &morpho_vec, self.state.tick_count);

                    let mut debug_vec = vec![0.0f32; unified_bus::BUS_DIM];
                    let err_count = self
                        .auto_debug
                        .diagnostics
                        .iter()
                        .filter(|d| d.level == "error")
                        .count();
                    debug_vec[0] = err_count as f32 / 10.0;
                    debug_vec[1] = self.auto_debug.n_patches_applied as f32 / 10.0;
                    debug_vec[2] = self.auto_debug.n_checks as f32 / 100.0;
                    self.unified_bus
                        .project("auto_debug", &debug_vec, self.state.tick_count);

                    let mut wm_vec = vec![0.0f32; unified_bus::BUS_DIM];
                    wm_vec[0] = self.world_model.objects.len() as f32 / 100.0;
                    wm_vec[1] = self.world_model.predictor.pred_err_ema;
                    self.unified_bus
                        .project("world_model", &wm_vec, self.state.tick_count);

                    let mut motive_vec = vec![0.0f32; unified_bus::BUS_DIM];
                    motive_vec[0] = self.motivation.discomfort;
                    motive_vec[1] = self.motivation.drives.curiosity;
                    motive_vec[2] = if self.motivation.dominant_drive() == "curiosity" {
                        1.0
                    } else {
                        0.0
                    };
                    self.unified_bus
                        .project("motivation", &motive_vec, self.state.tick_count);

                    let mut social_vec = vec![0.0f32; unified_bus::BUS_DIM];
                    let mean_trust: f32 = if self.social_complex.reputations.is_empty() {
                        0.5
                    } else {
                        self.social_complex
                            .reputations
                            .values()
                            .map(|r| r.trust_score)
                            .sum::<f32>()
                            / self.social_complex.reputations.len() as f32
                    };
                    social_vec[0] = mean_trust;
                    social_vec[1] = self.social_complex.n_interactions as f32 / 100.0;
                    self.unified_bus
                        .project("social_complex", &social_vec, self.state.tick_count);

                    let mut econ_vec = vec![0.0f32; unified_bus::BUS_DIM];
                    let mean_res: f32 = if self.autonomy_econ.resources.is_empty() {
                        0.5
                    } else {
                        self.autonomy_econ
                            .resources
                            .values()
                            .map(|r| r.fraction())
                            .sum::<f32>()
                            / self.autonomy_econ.resources.len() as f32
                    };
                    econ_vec[0] = mean_res;
                    econ_vec[1] = self.autonomy_econ.goals.len() as f32 / 20.0;
                    self.unified_bus
                        .project("autonomy_econ", &econ_vec, self.state.tick_count);

                    // Route attention across the bus
                    self.unified_bus.route(self.state.tick_count);

                    // 6q2. Architecture Model: self-observation of system structure
                    self.architecture_model.observe(
                        self.state.tick_count,
                        "big_transformer",
                        "train_speed",
                        self.big_transformer.n_train_steps as f32
                            / (self.state.tick_count as f32 + 1.0),
                    );
                    self.architecture_model.observe(
                        self.state.tick_count,
                        "big_transformer",
                        "error_rate",
                        self.auto_debug
                            .diagnostics
                            .iter()
                            .filter(|d| d.level == "error")
                            .count() as f32
                            / 10.0,
                    );
                    self.architecture_model.observe(
                        self.state.tick_count,
                        "morphogenesis",
                        "memory_mb",
                        self.morphogenesis.n_concepts() as f32 / 1000.0,
                    );
                    self.architecture_model.observe(
                        self.state.tick_count,
                        "planner",
                        "error_rate",
                        if self.planner.best.is_none() {
                            1.0
                        } else {
                            0.0
                        },
                    );
                    if let Some(suggestion) = self.architecture_model.suggest_change() {
                        self.action_log.push(format!("[ARCH_MODEL] {}", suggestion));
                    }
                    let bottlenecks = self.architecture_model.find_bottlenecks();
                    if !bottlenecks.is_empty() {
                        self.action_log.push(format!(
                            "[ARCH_MODEL] Bottleneck: {} score={:.2}",
                            bottlenecks[0].0, bottlenecks[0].1
                        ));
                    }

                    // Hole 4: Bus Predictor — train on observed transitions
                    if self.state.tick_count % 5 == 0
                        && self
                            .metabolism
                            .spend(self.metabolism.cost_per_plan * 0.5, "bus_predictor_train")
                    {
                        let bus_snapshot = self.slots_to_vector();
                        let instr_vec = self
                            .bus_predictor
                            .encode_instruction((self.state.tick_count % 10) as usize, 10);
                        // Predict BEFORE route (predict next state)
                        let _pred = self.bus_predictor.predict(&bus_snapshot, &instr_vec);
                        // After route, actual next state is current bus
                        let actual = self.slots_to_vector();
                        let mse = self
                            .bus_predictor
                            .train_step(&bus_snapshot, &instr_vec, &actual);
                        if self.state.tick_count % 20 == 0 {
                            self.action_log.push(format!(
                                "[BUS_PREDICTOR] train_step | mse={:.6} | ema={:.4}",
                                mse, self.bus_predictor.pred_error_ema
                            ));
                        }
                    }

                    // Modules LISTEN to the bus: react to cross-module signals
                    if let Some(planner_signal) = self.unified_bus.read("planner") {
                        // Big transformer listens to planner: if plan score is bad, increase training
                        if planner_signal[0] > 0.5 && self.big_transformer.vocab_size > 0 {
                            self.action_log.push("[BUS] planner->big_transformer | bad plan detected -> will train harder".to_string());
                        }
                    }
                    if let Some(motive_signal) = self.unified_bus.read("motivation") {
                        // Planner listens to motivation: if discomfort high, increase horizon
                        if motive_signal[0] > 0.6 {
                            self.planner.horizon = (self.planner.horizon + 1).min(6);
                            self.action_log.push(format!(
                                "[BUS] motivation->planner | high discomfort {:.2} -> horizon={}",
                                motive_signal[0], self.planner.horizon
                            ));
                        }
                    }
                    if let Some(debug_signal) = self.unified_bus.read("auto_debug") {
                        // Autonomy listens to auto-debug: if errors high, generate 'fix code' goal
                        if debug_signal[0] > 0.2 {
                            self.autonomy_econ.push_goal(
                                "fix build errors",
                                0.95,
                                self.state.tick_count + 50,
                            );
                            self.action_log.push("[BUS] auto_debug->autonomy_econ | generated goal: fix build errors".to_string());
                        }
                    }
                    if let Some(top) = self.unified_bus.top_active(3).first() {
                        self.action_log
                            .push(format!("[BUS] top active: {}={:.2}", top.0, top.1));
                    }
                }

                1.0
            }
            CapabilityId::ArchitectureModel => 2.0,
            CapabilityId::AutoDebug => {
                // 6p. AutoDebug: periodic cargo check + TODO scan + auto-patch
                if self.state.tick_count % 60 == 0
                    && self
                        .metabolism
                        .spend(self.metabolism.cost_per_tool, "auto_debug")
                {
                    let manifest_dir = env!("CARGO_MANIFEST_DIR");
                    let capability_dir =
                        std::path::Path::new(manifest_dir).join("src/garm/capabilities");
                    let capability_dir = capability_dir.to_string_lossy().to_string();

                    self.auto_debug.check(self.state.tick_count, manifest_dir);
                    self.auto_debug.enrich_fixes();
                    // Auto-modificacion: intenta aplicar patches para errores conocidos
                    let patches_applied = self.auto_debug.try_auto_patch(&capability_dir);
                    if patches_applied > 0 {
                        self.action_log.push(format!(
                            "[AUTOPATCH] {} patches applied automatically",
                            patches_applied
                        ));
                        for log in self.auto_debug.patch_log.iter().rev().take(patches_applied) {
                            self.action_log.push(format!("[AUTOPATCH] {}", log));
                        }
                        // Re-check after patching to verify fix
                        self.auto_debug.check(self.state.tick_count, manifest_dir);
                        let remaining_errors = self
                            .auto_debug
                            .diagnostics
                            .iter()
                            .filter(|d| d.level == "error")
                            .count();
                        self.action_log.push(format!(
                            "[AUTOPATCH] Re-check after patch | {} errors remaining",
                            remaining_errors
                        ));
                    }
                    let todos = self.auto_debug.scan_todos(&capability_dir);
                    let diag_summary = self.auto_debug.status();
                    self.action_log
                        .push(format!("[AUTODEBUG] {}", diag_summary));
                    if !todos.is_empty() {
                        self.action_log
                            .push(format!("[AUTODEBUG] {} TODOs/FIXMEs found", todos.len()));
                    }
                    for d in self.auto_debug.diagnostics.iter().take(3) {
                        self.action_log.push(format!(
                            "[AUTODEBUG] {} | {}:{} | {} | fix={:?}",
                            d.level, d.file, d.line, d.message, d.suggested_fix
                        ));
                    }
                    // INTERCONNECTION: auto-debug -> self_improvement
                    let error_count = self
                        .auto_debug
                        .diagnostics
                        .iter()
                        .filter(|d| d.level == "error")
                        .count();
                    if error_count > 0 {
                        let current_spt = self.corpus_reader.sentences_per_tick as f32;
                        let new_spt = (current_spt - 1.0).max(1.0);
                        self.corpus_reader.sentences_per_tick = new_spt as usize;
                        self.self_improvement
                            .record(self_improvement::ParamAdjustment {
                                param: self_improvement::TunableParam::CorpusSentencesPerTick,
                                old_value: current_spt,
                                new_value: new_spt,
                                reason: format!(
                                    "{} build errors detected by auto-debug; reducing corpus load",
                                    error_count
                                ),
                                tick: self.state.tick_count,
                                metric_before: error_count as f32,
                                metric_after: None,
                            });
                        self.action_log.push(format!("[INTERCONN] auto_debug->self_improve | {} errors | reduced corpus load spt {} -> {}", error_count, current_spt, new_spt));
                    }
                }

                2.0
            }
            CapabilityId::OpenEndedness => 2.0,
            CapabilityId::Evolution => 2.0,
            CapabilityId::SelfModel => 1.0,
            CapabilityId::Temporal => {
                // 1. Passive temporal plans (always run)
                let due = self.temporal.due_plans(self.state.tick_count);
                for plan in due {
                    self.action_log
                        .push(format!("[TEMPORAL] Executing planned intent: {}", plan));
                }

                1.0
            }
            CapabilityId::TheoryOfMind => 1.0,
            CapabilityId::InternalLanguage => 1.0,
            CapabilityId::Perception => 2.0,
            CapabilityId::Sandbox => 1.0,
            CapabilityId::ComputerUse => 1.0,
            CapabilityId::ToolCalling => 1.0,
            CapabilityId::McpClient => 1.0,
            CapabilityId::Voice => 1.0,
            CapabilityId::Vision => 2.0,
            CapabilityId::NaturalLanguage => 1.0,
        };
        cost
    }
}

impl GARMNode for CapabilityNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        Self::name_for_cap(self.cap)
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        let guard = self.engine.lock().unwrap();
        let energy_stress = (100.0 - guard.metabolism.energy) / 100.0;
        drop(guard);
        self.internal_fe + energy_stress * 0.3 + 0.5
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.internal_fe, self.n_executions as f32, self.last_cost]
    }

    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.1).min(10.0);
        }
        NodeAction::Output(vec![self.internal_fe, self.n_executions as f32])
    }

    fn update(&mut self, dt: f32, energy_in: f32) -> f32 {
        self.tick_accumulator += dt;
        let cost = if energy_in > 1.0 && self.tick_accumulator >= self.tick_interval {
            self.tick_accumulator -= self.tick_interval;
            self.execute()
        } else {
            0.0
        };
        self.last_cost = cost;
        self.internal_fe *= 0.99;
        cost
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        10.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
