// EDEN GARM — Next-Gen Autonomous Engine
// Integrates: NLP + Tool Calling + MCP Client + Computer Use + Sandbox + Vision + Voice + Security + Observability
// + Neural Real + Morphogenesis + Evolution + Self-Model
// + Temporal + Motivation
// ZERO LLM. ZERO external APIs. 100% Rust pure.

pub mod active_inference;
pub mod agent_mesh;
pub mod analogy;
pub mod architecture_model;
pub mod auto_debug;
pub mod autonomy;
pub mod autonomy_econ;
pub mod benchmark;
pub mod big_transformer;
pub mod bptt_engine;
pub mod bus_predictor;
pub mod causal_model;
pub mod causality;
pub mod circadian;
pub mod composition;
pub mod compositional;
pub mod computer;
pub mod constitutional_safety;
pub mod continual_learning;
pub mod continuous;
pub mod corpus_massive;
pub mod corpus_reader;
pub mod counterfactual;
pub mod critic;
pub mod curriculum;
pub mod dnc_memory;
pub mod economic_agent;
pub mod embodiment;
pub mod emotional_modulation;
pub mod epistemic;
pub mod evidence;
pub mod evolution;
pub mod experience_buffer;
pub mod exploration;
pub mod gate;
pub mod gen_metrics;
pub mod goal_executor;
pub mod gridworld;
pub mod grounding;
pub mod hierarchical_attention;
pub mod hippocampus;
pub mod homeostasis;
pub mod inference;
pub mod intention_hierarchy;
pub mod internal_language;
pub mod language_gen;
pub mod logic_reasoning;
pub mod mcp_client;
pub mod mdl_pruner;
pub mod memory_clustering;
pub mod meta_evolution;
pub mod meta_learning;
pub mod metabolism;
pub mod metacognition;
pub mod moe;
pub mod mood;
pub mod morphogenesis;
pub mod motivation;
pub mod multi_agent;
pub mod multimodal;
pub mod neural;
pub mod neural_extractors;
pub mod nlp;
pub mod observability;
pub mod perception;
pub mod phenomenology;
pub mod physics;
pub mod planner;
pub mod plugin;
pub mod predictive_loop;
pub mod program;
pub mod program_induction;
pub mod recurrent_state;
pub mod reward_oracle;
pub mod sandbox;
pub mod scene_vector;
pub mod security;
pub mod self_awareness;
pub mod self_improvement;
pub mod self_model;
pub mod self_modification;
pub mod semantics;
pub mod social_complex;
pub mod society;
pub mod surprise;
pub mod swarm;
pub mod syntax;
pub mod synthetic_vision;
pub mod temporal;
pub mod temporal_hierarchy;
pub mod theory_of_mind;
pub mod tools;
pub mod transformer;
pub mod unified_bus;
pub mod unified_hub;
pub mod vision;
pub mod voice;
pub mod working_memory;
pub mod world3d;
pub mod world_model;
pub mod world_model_nn;

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum GarmCapability {
    NaturalLanguage,
    ToolCalling,
    McpClient,
    ComputerUse,
    Sandbox,
    Vision,
    Voice,
    Security,
    Observability,
    Neural,
    Morphogenesis,
    Evolution,
    SelfModel,
    Temporal,
    Motivation,
    Mood,
    TheoryOfMind,
    Swarm,
    Metacognition,
    InternalLanguage,
    MultiAgent,
    Causality,
    MetaEvolution,
    UnifiedPerception,
    IntentionHierarchy,
    SocietyOfMind,
    Semantics,
    WorldModel,
    PluginSystem,
    Hippocampus,
    SceneVector,
    Physics,
    Syntax,
    PlannerSystem,
    CorpusReader,
    Grounding,
    GoalExecution,
    Autonomy,
    Analogy,
    SelfAwareness,
    SelfImprovement,
    Composition,
    ContinuousMode,
    SyntheticVision,
    PredictiveLoop,
    LanguageGen,
    Counterfactual,
    Curriculum,
    MemoryClustering,
    GridWorld,
    AgentMesh,
    Compositional,
    NeuralExtractors,
    Transformer,
    BigTransformer,
    Multimodal,
    World3D,
    SocialComplex,
    AutonomyEcon,
}

#[derive(Clone, Debug)]
pub struct GarmState {
    pub capabilities: Vec<GarmCapability>,
    pub tick_count: u64,
    pub idle_ticks: u64,
    pub last_input: String,
    pub last_intent: nlp::Intent,
    pub last_intent_confidence: f32,
    pub last_concept_id: u64,
    pub is_under_attack: bool,
}

pub struct GarmCapabilityState {
    pub state: GarmState,
    pub nlp: nlp::IntentClassifier,
    pub tool_registry: tools::ToolRegistry,
    pub mcp_client: Option<mcp_client::McpClient>,
    pub computer: computer::ComputerUseEngine,
    pub sandbox: sandbox::SandboxEngine,
    pub vision: vision::VisionEngine,
    pub voice: voice::VoiceEngine,
    pub security: security::SecurityEngine,
    pub observability: observability::ObservabilityEngine,
    pub neural: neural::OnlineNetwork,
    pub morphogenesis: morphogenesis::ConceptSpace,
    pub evolution: evolution::Population,
    pub self_model: self_model::SelfModel,
    pub temporal: temporal::EventLog,
    pub motivation: motivation::MotivationEngine,
    pub mood: mood::MoodState,
    pub theory_of_mind: theory_of_mind::UserModel,
    pub swarm: swarm::CognitiveEnsemble,
    pub metacognition: metacognition::Metacognition,
    pub internal_language: internal_language::InternalLanguage,
    pub multi_agent: multi_agent::MultiAgentMesh,
    pub causality: causality::StructuralCausalModel,
    pub meta_vm: Option<meta_evolution::MetaVM>,
    pub auto_debug: auto_debug::AutoDebugger,
    pub unified_bus: unified_bus::UnifiedBus,
    pub perception: perception::UnifiedPerception,
    pub goal_stack: intention_hierarchy::GoalStack,
    pub workspace: society::GlobalWorkspace,
    pub semantics: semantics::DistributionalSemantics,
    pub world_model: world_model::WorldModel,
    pub plugins: plugin::PluginRegistry,
    pub hippocampus: hippocampus::Hippocampus,
    pub scene_parser: scene_vector::SceneParser,
    pub physics: physics::PhysicsEngine,
    pub syntax: syntax::SyntaxParser,
    pub planner: planner::Planner,
    pub corpus_reader: corpus_reader::CorpusReader,
    pub grounding: grounding::GroundingEngine,
    pub goal_executor: goal_executor::GoalExecutor,
    pub autonomy: autonomy::AutonomyEngine,
    pub analogy: analogy::AnalogyEngine,
    pub self_awareness: self_awareness::SelfAwareness,
    pub self_improvement: self_improvement::SelfImprovement,
    pub composition: composition::CompositionEngine,
    pub continuous_mode: continuous::ContinuousMode,
    pub synthetic_vision: synthetic_vision::SyntheticVision,
    pub predictive_loop: predictive_loop::PredictiveLoop,
    pub language_gen: language_gen::LanguageGenerator,
    pub counterfactual: counterfactual::CounterfactualEngine,
    pub curriculum: curriculum::CurriculumEngine,
    pub memory_clustering: memory_clustering::MemoryClustering,
    pub gridworld: gridworld::GridWorld,
    pub agent_mesh: agent_mesh::AgentMesh,
    pub compositional: compositional::Compositional,
    pub neural_extractors: neural_extractors::NeuralExtractors,
    pub transformer: transformer::EdenTransformer,
    pub big_transformer: big_transformer::BigTransformer,
    pub multimodal: multimodal::MultimodalEncoder,
    pub world3d: world3d::World3D,
    pub social_complex: social_complex::SocialComplex,
    pub autonomy_econ: autonomy_econ::AutonomyEcon,
    pub active_program: Option<program::Program>,
    pub experience_buffer: experience_buffer::ExperienceBuffer,
    pub architecture_model: architecture_model::ArchitectureModel,
    pub bus_predictor: bus_predictor::BusPredictor,
    pub exploration: exploration::ExplorationEngine,
    pub attention_stack: Vec<String>,
    pub current_topic: String,
    pub gate: gate::VetoGate,
    pub metabolism: metabolism::Metabolism,
    pub evidence: evidence::EvidenceAccumulator,
    pub surprise: surprise::SurpriseDetector,
    pub epistemic: epistemic::EpistemicConfidence,
    pub circadian: circadian::Circadian,
    pub critic: critic::Critic,
    pub working_memory: working_memory::WorkingMemory,
    pub world_model_nn: world_model_nn::NeuralWorldModel,
    pub moe: moe::MoELayer,
    pub hierarchical_attention: hierarchical_attention::HierarchicalAttention,
    pub program_induction: program_induction::ProgramInduction,
    pub causal_model: causal_model::CausalModel,
    pub continual_learning: continual_learning::ContinualLearning,
    pub homeostasis: homeostasis::Homeostasis,
    pub meta_learning: meta_learning::MetaLearning,
    pub mdl_pruner: mdl_pruner::MDLPruner,
    pub emotional_modulation: emotional_modulation::EmotionalModulation,
    pub recurrent_state: recurrent_state::RecurrentState,
    pub dnc: dnc_memory::DNC,
    pub active_inference: active_inference::ActiveInference,
    pub embodiment: embodiment::Body,
    pub temporal_hierarchy: temporal_hierarchy::TemporalHierarchy,
    pub self_modification: self_modification::SelfModification,
    pub unified_hub: unified_hub::UnifiedHub,
    pub logic_reasoning: logic_reasoning::LogicReasoning,
    pub constitutional_safety: constitutional_safety::ConstitutionalSafety,
    pub phenomenology: phenomenology::Phenomenology,
    pub economic_agent: economic_agent::EconomicAgent,
    pub reward_oracle: reward_oracle::RewardOracle,
    pub bptt: bptt_engine::BPTTEngine,
    pub corpus_massive: corpus_massive::CorpusMassive,
    pub benchmark: benchmark::BenchmarkSuite,
    pub gen_metrics: gen_metrics::GenMetrics,
    pub action_log: Vec<String>,
    pub current_winner: String,
}

const STATE_DIM: usize = 8;
const ACTION_DIM: usize = 8;

impl GarmCapabilityState {
    /// Fast initialization for benchmarking/testing.
    /// Skips expensive IO: no process scan, no corpus auto-load.
    pub fn new_fast() -> Self {
        const OUTCOME_DIM: usize = 6;
        let neural = neural::OnlineNetwork::new(STATE_DIM + ACTION_DIM, 12, OUTCOME_DIM, 0.05);
        let weights_size = neural.n_weights();
        let program_size = 9usize;
        let genome_size = weights_size + program_size;
        let mut engine = GarmCapabilityState {
            state: GarmState {
                capabilities: vec![
                    GarmCapability::NaturalLanguage,
                    GarmCapability::ToolCalling,
                    GarmCapability::McpClient,
                    GarmCapability::ComputerUse,
                    GarmCapability::Sandbox,
                    GarmCapability::Vision,
                    GarmCapability::Voice,
                    GarmCapability::Security,
                    GarmCapability::Observability,
                    GarmCapability::Neural,
                    GarmCapability::Morphogenesis,
                    GarmCapability::Evolution,
                    GarmCapability::SelfModel,
                    GarmCapability::Temporal,
                    GarmCapability::Motivation,
                    GarmCapability::Mood,
                    GarmCapability::TheoryOfMind,
                    GarmCapability::Swarm,
                    GarmCapability::Metacognition,
                    GarmCapability::InternalLanguage,
                    GarmCapability::MultiAgent,
                    GarmCapability::Causality,
                    GarmCapability::MetaEvolution,
                    GarmCapability::UnifiedPerception,
                    GarmCapability::IntentionHierarchy,
                    GarmCapability::SocietyOfMind,
                    GarmCapability::Semantics,
                    GarmCapability::WorldModel,
                    GarmCapability::PluginSystem,
                    GarmCapability::Hippocampus,
                    GarmCapability::SceneVector,
                    GarmCapability::Physics,
                    GarmCapability::Syntax,
                    GarmCapability::PlannerSystem,
                    GarmCapability::CorpusReader,
                    GarmCapability::Grounding,
                    GarmCapability::GoalExecution,
                    GarmCapability::Autonomy,
                    GarmCapability::Analogy,
                    GarmCapability::SelfAwareness,
                    GarmCapability::SelfImprovement,
                    GarmCapability::Composition,
                    GarmCapability::ContinuousMode,
                    GarmCapability::SyntheticVision,
                    GarmCapability::PredictiveLoop,
                    GarmCapability::LanguageGen,
                    GarmCapability::Counterfactual,
                    GarmCapability::Curriculum,
                    GarmCapability::MemoryClustering,
                    GarmCapability::GridWorld,
                    GarmCapability::AgentMesh,
                    GarmCapability::Compositional,
                    GarmCapability::NeuralExtractors,
                    GarmCapability::Transformer,
                    GarmCapability::BigTransformer,
                    GarmCapability::Multimodal,
                    GarmCapability::World3D,
                    GarmCapability::SocialComplex,
                    GarmCapability::AutonomyEcon,
                ],
                tick_count: 0,
                idle_ticks: 0,
                last_input: String::new(),
                last_intent: nlp::Intent::Unknown,
                last_intent_confidence: 0.0,
                last_concept_id: 0,
                is_under_attack: false,
            },
            nlp: nlp::IntentClassifier::new(),
            tool_registry: tools::make_builtin_registry(),
            mcp_client: None,
            computer: computer::ComputerUseEngine::new(),
            sandbox: sandbox::SandboxEngine::new(),
            vision: vision::VisionEngine::new(),
            voice: voice::VoiceEngine::new(),
            security: security::SecurityEngine::new(),
            observability: observability::ObservabilityEngine::new(),
            neural,
            morphogenesis: morphogenesis::ConceptSpace::new(),
            evolution: evolution::Population::new(genome_size, 12),
            self_model: self_model::SelfModel::new(STATE_DIM, ACTION_DIM),
            temporal: temporal::EventLog::new(),
            motivation: motivation::MotivationEngine::new(),
            mood: mood::MoodState::new(),
            theory_of_mind: theory_of_mind::UserModel::new(),
            swarm: swarm::CognitiveEnsemble::new(STATE_DIM + ACTION_DIM, OUTCOME_DIM),
            metacognition: metacognition::Metacognition::new(),
            internal_language: internal_language::InternalLanguage::new(),
            multi_agent: multi_agent::MultiAgentMesh::new(),
            causality: causality::StructuralCausalModel::new(),
            meta_vm: None,
            auto_debug: auto_debug::AutoDebugger::new(),
            unified_bus: {
                let mut bus = unified_bus::UnifiedBus::new();
                for name in &[
                    "big_transformer",
                    "planner",
                    "morphogenesis",
                    "auto_debug",
                    "world_model",
                    "motivation",
                    "social_complex",
                    "autonomy_econ",
                ] {
                    bus.register(name);
                }
                bus
            },
            perception: perception::UnifiedPerception::new(12, 64, 13, 32),
            goal_stack: intention_hierarchy::GoalStack::new(),
            workspace: society::GlobalWorkspace::new(),
            semantics: semantics::DistributionalSemantics::new(32, 5000),
            world_model: world_model::WorldModel::new(),
            plugins: plugin::PluginRegistry::new(),
            hippocampus: hippocampus::Hippocampus::new(500),
            scene_parser: scene_vector::SceneParser::new(),
            physics: physics::PhysicsEngine::new(),
            syntax: syntax::SyntaxParser::new(),
            planner: planner::Planner::new(),
            corpus_reader: corpus_reader::CorpusReader::new(),
            grounding: grounding::GroundingEngine::new(),
            goal_executor: goal_executor::GoalExecutor::new(),
            autonomy: autonomy::AutonomyEngine::new(),
            analogy: analogy::AnalogyEngine::new(),
            self_awareness: self_awareness::SelfAwareness::new(),
            self_improvement: self_improvement::SelfImprovement::new(),
            composition: composition::CompositionEngine::new(),
            continuous_mode: continuous::ContinuousMode::new(),
            synthetic_vision: synthetic_vision::SyntheticVision::new(64, 48),
            predictive_loop: predictive_loop::PredictiveLoop::new(),
            language_gen: language_gen::LanguageGenerator::new(),
            counterfactual: counterfactual::CounterfactualEngine::new(),
            curriculum: curriculum::CurriculumEngine::new(),
            memory_clustering: memory_clustering::MemoryClustering::new(),
            gridworld: gridworld::GridWorld::new(8, 8),
            agent_mesh: agent_mesh::AgentMesh::new("eden-main"),
            compositional: compositional::Compositional::new(),
            neural_extractors: neural_extractors::NeuralExtractors::new(32),
            transformer: transformer::EdenTransformer::new(64, 4, 2, 128, 32),
            big_transformer: big_transformer::BigTransformer::new_large(),
            multimodal: multimodal::MultimodalEncoder::new(256),
            world3d: world3d::World3D::new(),
            social_complex: social_complex::SocialComplex::new(),
            autonomy_econ: autonomy_econ::AutonomyEcon::new(),
            active_program: None,
            experience_buffer: experience_buffer::ExperienceBuffer::new(256),
            architecture_model: architecture_model::ArchitectureModel::new(),
            bus_predictor: bus_predictor::BusPredictor::new(unified_bus::BUS_DIM, 10),
            exploration: exploration::ExplorationEngine::new(),
            attention_stack: Vec::new(),
            current_topic: "init".to_string(),
            gate: gate::VetoGate::new(),
            metabolism: metabolism::Metabolism::new(),
            evidence: evidence::EvidenceAccumulator::new(),
            surprise: surprise::SurpriseDetector::new(),
            epistemic: epistemic::EpistemicConfidence::new(),
            circadian: circadian::Circadian::new(),
            critic: critic::Critic::new(64),
            working_memory: working_memory::WorkingMemory::new(16, 64),
            world_model_nn: world_model_nn::NeuralWorldModel::new(64, 8),
            moe: moe::MoELayer::new(8, 2, 512, 256, 512),
            hierarchical_attention: hierarchical_attention::HierarchicalAttention::new(
                64, 64, 64, 64,
            ),
            program_induction: program_induction::ProgramInduction::new(),
            causal_model: causal_model::CausalModel::new(),
            continual_learning: continual_learning::ContinualLearning::new(10000),
            homeostasis: homeostasis::Homeostasis::new(),
            meta_learning: meta_learning::MetaLearning::new(64, 64),
            mdl_pruner: mdl_pruner::MDLPruner::new(),
            emotional_modulation: emotional_modulation::EmotionalModulation::new(),
            recurrent_state: recurrent_state::RecurrentState::new(128),
            dnc: dnc_memory::DNC::new(128, 64, 4),
            active_inference: active_inference::ActiveInference::new(64),
            embodiment: embodiment::Body::new(),
            temporal_hierarchy: temporal_hierarchy::TemporalHierarchy::new(),
            self_modification: self_modification::SelfModification::new(),
            unified_hub: {
                let mut hub = unified_hub::UnifiedHub::new(64);
                hub.register_domain("vision", 64);
                hub.register_domain("language", 64);
                hub.register_domain("action", 8);
                hub.register_domain("physics", 64);
                hub
            },
            logic_reasoning: logic_reasoning::LogicReasoning::new(),
            constitutional_safety: constitutional_safety::ConstitutionalSafety::new(),
            phenomenology: phenomenology::Phenomenology::new(64),
            economic_agent: economic_agent::EconomicAgent::new(),
            reward_oracle: reward_oracle::RewardOracle::new(),
            bptt: bptt_engine::BPTTEngine::new(10, 512),
            corpus_massive: corpus_massive::CorpusMassive::new(),
            benchmark: benchmark::BenchmarkSuite::new(),
            gen_metrics: gen_metrics::GenMetrics::new(),
            action_log: Vec::new(),
            current_winner: String::new(),
        };
        engine.corpus_reader.sentences_per_tick = 5;
        // Pre-seed semantics with hybrid corpus vocabulary so big_transformer has tokens from tick 0
        let cm_words = engine.corpus_massive.all_words();
        engine.semantics.ensure_raw_words(&cm_words);
        engine.big_transformer.build_embeddings_from_semantics(
            &engine.semantics.index_to_word,
            &engine.semantics.embeddings,
        );
        // Fast init skips heavy pre-bootstrap; full bootstrap only in new()
        engine
    }

    pub fn new() -> Self {
        const OUTCOME_DIM: usize = 6;
        let neural = neural::OnlineNetwork::new(STATE_DIM + ACTION_DIM, 12, OUTCOME_DIM, 0.05);
        let weights_size = neural.n_weights();
        let program_size = 9usize; // 3 ops × 3 genes each
        let genome_size = weights_size + program_size;
        let mut engine = GarmCapabilityState {
            state: GarmState {
                capabilities: vec![
                    GarmCapability::NaturalLanguage,
                    GarmCapability::ToolCalling,
                    GarmCapability::McpClient,
                    GarmCapability::ComputerUse,
                    GarmCapability::Sandbox,
                    GarmCapability::Vision,
                    GarmCapability::Voice,
                    GarmCapability::Security,
                    GarmCapability::Observability,
                    GarmCapability::Neural,
                    GarmCapability::Morphogenesis,
                    GarmCapability::Evolution,
                    GarmCapability::SelfModel,
                    GarmCapability::Temporal,
                    GarmCapability::Motivation,
                    GarmCapability::Mood,
                    GarmCapability::TheoryOfMind,
                    GarmCapability::Swarm,
                    GarmCapability::Metacognition,
                    GarmCapability::InternalLanguage,
                    GarmCapability::MultiAgent,
                    GarmCapability::Causality,
                    GarmCapability::MetaEvolution,
                    GarmCapability::UnifiedPerception,
                    GarmCapability::IntentionHierarchy,
                    GarmCapability::SocietyOfMind,
                    GarmCapability::Semantics,
                    GarmCapability::WorldModel,
                    GarmCapability::PluginSystem,
                    GarmCapability::Hippocampus,
                    GarmCapability::SceneVector,
                    GarmCapability::Physics,
                    GarmCapability::Syntax,
                    GarmCapability::PlannerSystem,
                    GarmCapability::CorpusReader,
                    GarmCapability::Grounding,
                    GarmCapability::GoalExecution,
                    GarmCapability::Autonomy,
                    GarmCapability::Analogy,
                    GarmCapability::SelfAwareness,
                    GarmCapability::SelfImprovement,
                    GarmCapability::Composition,
                    GarmCapability::ContinuousMode,
                    GarmCapability::SyntheticVision,
                    GarmCapability::PredictiveLoop,
                    GarmCapability::LanguageGen,
                    GarmCapability::Counterfactual,
                    GarmCapability::Curriculum,
                    GarmCapability::MemoryClustering,
                    GarmCapability::GridWorld,
                    GarmCapability::AgentMesh,
                    GarmCapability::Compositional,
                    GarmCapability::NeuralExtractors,
                    GarmCapability::Transformer,
                    GarmCapability::BigTransformer,
                    GarmCapability::Multimodal,
                    GarmCapability::World3D,
                    GarmCapability::SocialComplex,
                    GarmCapability::AutonomyEcon,
                ],
                tick_count: 0,
                idle_ticks: 0,
                last_input: String::new(),
                last_intent: nlp::Intent::Unknown,
                last_intent_confidence: 0.0,
                last_concept_id: 0,
                is_under_attack: false,
            },
            nlp: nlp::IntentClassifier::new(),
            tool_registry: tools::make_builtin_registry(),
            mcp_client: None,
            computer: computer::ComputerUseEngine::new(),
            sandbox: sandbox::SandboxEngine::new(),
            vision: vision::VisionEngine::new(),
            voice: voice::VoiceEngine::new(),
            security: security::SecurityEngine::new(),
            observability: observability::ObservabilityEngine::new(),
            neural,
            morphogenesis: morphogenesis::ConceptSpace::new(),
            evolution: evolution::Population::new(genome_size, 12),
            self_model: self_model::SelfModel::new(STATE_DIM, ACTION_DIM),
            temporal: temporal::EventLog::new(),
            motivation: motivation::MotivationEngine::new(),
            mood: mood::MoodState::new(),
            theory_of_mind: theory_of_mind::UserModel::new(),
            swarm: swarm::CognitiveEnsemble::new(STATE_DIM + ACTION_DIM, OUTCOME_DIM),
            metacognition: metacognition::Metacognition::new(),
            internal_language: internal_language::InternalLanguage::new(),
            multi_agent: multi_agent::MultiAgentMesh::new(),
            causality: causality::StructuralCausalModel::new(),
            meta_vm: None,
            auto_debug: auto_debug::AutoDebugger::new(),
            unified_bus: {
                let mut bus = unified_bus::UnifiedBus::new();
                for name in &[
                    "big_transformer",
                    "planner",
                    "morphogenesis",
                    "auto_debug",
                    "world_model",
                    "motivation",
                    "social_complex",
                    "autonomy_econ",
                ] {
                    bus.register(name);
                }
                bus
            },
            perception: perception::UnifiedPerception::new(12, 64, 13, 32),
            goal_stack: intention_hierarchy::GoalStack::new(),
            workspace: society::GlobalWorkspace::new(),
            semantics: semantics::DistributionalSemantics::new(32, 5000),
            world_model: world_model::WorldModel::new(),
            plugins: plugin::PluginRegistry::new(),
            hippocampus: hippocampus::Hippocampus::new(500),
            scene_parser: scene_vector::SceneParser::new(),
            physics: physics::PhysicsEngine::new(),
            syntax: syntax::SyntaxParser::new(),
            planner: planner::Planner::new(),
            corpus_reader: corpus_reader::CorpusReader::new(),
            grounding: grounding::GroundingEngine::new(),
            goal_executor: goal_executor::GoalExecutor::new(),
            autonomy: autonomy::AutonomyEngine::new(),
            analogy: analogy::AnalogyEngine::new(),
            self_awareness: self_awareness::SelfAwareness::new(),
            self_improvement: self_improvement::SelfImprovement::new(),
            composition: composition::CompositionEngine::new(),
            continuous_mode: continuous::ContinuousMode::new(),
            synthetic_vision: synthetic_vision::SyntheticVision::new(64, 48),
            predictive_loop: predictive_loop::PredictiveLoop::new(),
            language_gen: language_gen::LanguageGenerator::new(),
            counterfactual: counterfactual::CounterfactualEngine::new(),
            curriculum: curriculum::CurriculumEngine::new(),
            memory_clustering: memory_clustering::MemoryClustering::new(),
            gridworld: gridworld::GridWorld::new(8, 8),
            agent_mesh: agent_mesh::AgentMesh::new("eden-main"),
            compositional: compositional::Compositional::new(),
            neural_extractors: neural_extractors::NeuralExtractors::new(32),
            transformer: transformer::EdenTransformer::new(64, 4, 2, 128, 32),
            big_transformer: big_transformer::BigTransformer::new_large(),
            multimodal: multimodal::MultimodalEncoder::new(256),
            world3d: world3d::World3D::new(),
            social_complex: social_complex::SocialComplex::new(),
            autonomy_econ: autonomy_econ::AutonomyEcon::new(),
            active_program: None,
            experience_buffer: experience_buffer::ExperienceBuffer::new(256),
            architecture_model: architecture_model::ArchitectureModel::new(),
            bus_predictor: bus_predictor::BusPredictor::new(unified_bus::BUS_DIM, 10),
            exploration: exploration::ExplorationEngine::new(),
            attention_stack: Vec::new(),
            current_topic: "init".to_string(),
            gate: gate::VetoGate::new(),
            metabolism: metabolism::Metabolism::new(),
            evidence: evidence::EvidenceAccumulator::new(),
            surprise: surprise::SurpriseDetector::new(),
            epistemic: epistemic::EpistemicConfidence::new(),
            circadian: circadian::Circadian::new(),
            critic: critic::Critic::new(64),
            working_memory: working_memory::WorkingMemory::new(16, 64),
            world_model_nn: world_model_nn::NeuralWorldModel::new(64, 8),
            moe: moe::MoELayer::new(8, 2, 512, 256, 512),
            hierarchical_attention: hierarchical_attention::HierarchicalAttention::new(
                64, 64, 64, 64,
            ),
            program_induction: program_induction::ProgramInduction::new(),
            causal_model: causal_model::CausalModel::new(),
            continual_learning: continual_learning::ContinualLearning::new(10000),
            homeostasis: homeostasis::Homeostasis::new(),
            meta_learning: meta_learning::MetaLearning::new(64, 64),
            mdl_pruner: mdl_pruner::MDLPruner::new(),
            emotional_modulation: emotional_modulation::EmotionalModulation::new(),
            recurrent_state: recurrent_state::RecurrentState::new(128),
            dnc: dnc_memory::DNC::new(128, 64, 4),
            active_inference: active_inference::ActiveInference::new(64),
            embodiment: embodiment::Body::new(),
            temporal_hierarchy: temporal_hierarchy::TemporalHierarchy::new(),
            self_modification: self_modification::SelfModification::new(),
            unified_hub: {
                let mut hub = unified_hub::UnifiedHub::new(64);
                hub.register_domain("vision", 64);
                hub.register_domain("language", 64);
                hub.register_domain("action", 8);
                hub.register_domain("physics", 64);
                hub
            },
            logic_reasoning: logic_reasoning::LogicReasoning::new(),
            constitutional_safety: constitutional_safety::ConstitutionalSafety::new(),
            phenomenology: phenomenology::Phenomenology::new(64),
            economic_agent: economic_agent::EconomicAgent::new(),
            reward_oracle: reward_oracle::RewardOracle::new(),
            bptt: bptt_engine::BPTTEngine::new(10, 512),
            corpus_massive: corpus_massive::CorpusMassive::new(),
            benchmark: benchmark::BenchmarkSuite::new(),
            gen_metrics: gen_metrics::GenMetrics::new(),
            action_log: Vec::new(),
            current_winner: String::new(),
        };
        engine.computer.scan_processes_deep();
        // Load massive corpus for big_transformer training
        let corpus_paths = [
            "/home/ubuntu/eden_core/corpus/procedural_100k.txt",
            "/home/ubuntu/eden_core/corpus/procedural_5k.txt",
            "/home/ubuntu/eden_core/corpus/eden_large.txt",
        ];
        for path in &corpus_paths {
            if std::fs::metadata(path).is_ok() {
                if let Ok(n) = engine.corpus_reader.load_file_streaming(path) {
                    println!("[INIT] Loaded corpus: {} | {} sentences", path, n);
                }
            }
        }
        engine.corpus_reader.sentences_per_tick = 5;
        // Pre-seed semantics with hybrid corpus vocabulary so big_transformer has tokens from tick 0
        let cm_words = engine.corpus_massive.all_words();
        engine.semantics.ensure_raw_words(&cm_words);
        engine.big_transformer.build_embeddings_from_semantics(
            &engine.semantics.index_to_word,
            &engine.semantics.embeddings,
        );
        // Attempt to resume from previous saved state
        let state_path = crate::eden_garm::state_paths::capability_state_path();
        if std::fs::metadata(&state_path).is_ok() {
            if let Ok(_) = engine.load_state(&state_path) {
                println!(
                    "[INIT] Resumed from saved state at tick={} | vocab={} | train_steps={}",
                    engine.state.tick_count,
                    engine.big_transformer.vocab_size,
                    engine.big_transformer.n_train_steps
                );
            } else {
                println!(
                    "[INIT] Failed to load state from {}, starting fresh",
                    state_path
                );
                engine.pre_bootstrap(1000);
                println!(
                    "[INIT] Pre-bootstrap complete | vocab={} | train_steps={}",
                    engine.big_transformer.vocab_size, engine.big_transformer.n_train_steps
                );
            }
        } else {
            // Full pre-bootstrap: 1000 state->action pairs before first tick
            engine.pre_bootstrap(1000);
            println!(
                "[INIT] Pre-bootstrap complete | vocab={} | train_steps={}",
                engine.big_transformer.vocab_size, engine.big_transformer.n_train_steps
            );
        }
        engine
    }

    pub fn encode_state(&self, input_len: usize, n_entities: usize) -> Vec<f32> {
        let (l1, _, _) = self.computer.load_average();
        let top_mem = self
            .computer
            .top_processes(1)
            .first()
            .map(|p| p.mem_kb as f32 / 1_000_000.0)
            .unwrap_or(0.0);
        vec![
            (input_len as f32).clamp(0.0, 1000.0) / 1000.0,
            (self.state.tick_count as f32).clamp(0.0, 100_000.0) / 100_000.0,
            self.state.last_intent_confidence,
            if self.state.is_under_attack { 1.0 } else { 0.0 },
            (n_entities as f32).clamp(0.0, 50.0) / 50.0,
            (self.computer.state.processes.len() as f32).clamp(0.0, 1000.0) / 1000.0,
            l1.clamp(0.0, 20.0) / 20.0,
            top_mem.clamp(0.0, 10.0) / 10.0,
        ]
    }

    fn encode_action(intent: &nlp::Intent) -> Vec<f32> {
        match intent {
            nlp::Intent::ToolCall => vec![1.0, 0.0, 0.0, 0.0, 0.0],
            nlp::Intent::ExecuteCode => vec![0.0, 1.0, 0.0, 0.0, 0.0],
            nlp::Intent::BrowseUrl => vec![0.0, 0.0, 1.0, 0.0, 0.0],
            nlp::Intent::ComputerUse => vec![0.0, 0.0, 0.0, 1.0, 0.0],
            _ => vec![0.0, 0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn process_input(&mut self, input: &str) -> Vec<String> {
        let start = std::time::Instant::now();
        let mut actions = Vec::new();
        self.state.tick_count += 1;
        self.state.idle_ticks = 0;
        self.state.last_input = input.to_string();

        // 1. Security scan
        let scan = self.security.scan(input);
        if scan.score > self.security.block_threshold {
            self.state.is_under_attack = true;
            actions.push(format!(
                "[SECURITY] BLOCKED | score={:.2} | threats: {}",
                scan.score,
                scan.threats.len()
            ));
            return actions;
        }
        self.state.is_under_attack = false;

        // 2. NLP: classify intent
        let (intent, conf) = self.nlp.classify(input);
        self.state.last_intent = intent.clone();
        self.state.last_intent_confidence = conf;
        actions.push(format!(
            "[NLP] Intent: {:?} | confidence: {:.2}",
            intent, conf
        ));

        // 3. Extract entities with attentional focus
        let mut entities = nlp::extract_entities(input);
        if self.motivation.dominant_drive() == "efficiency" && entities.len() > 3 {
            entities.truncate(3);
        }
        if !entities.is_empty() {
            actions.push(format!(
                "[NLP] Entities: {} | focus={}",
                entities.len(),
                self.motivation.dominant_drive()
            ));
        }

        // 3a. Syntax + SceneVector: structural parsing of input
        let tokens = nlp::tokenize(input);
        let scene =
            self.scene_parser
                .parse(&tokens, &self.semantics, self.semantics.embed_dim.max(1));
        let causal_pairs = self.scene_parser.extract_causal(&tokens);
        if !causal_pairs.is_empty() {
            for (cause, effect) in &causal_pairs {
                actions.push(format!("[SYNTAX] Causal: '{}' -> '{}'", cause, effect));
                // Add to causal model if both map to concepts
                let cause_emb = self.semantics.sentence_embedding(cause);
                let effect_emb = self.semantics.sentence_embedding(effect);
                let cause_id = self
                    .morphogenesis
                    .classify(&cause_emb)
                    .map(|(id, _)| id)
                    .unwrap_or(0);
                let effect_id = self
                    .morphogenesis
                    .classify(&effect_emb)
                    .map(|(id, _)| id)
                    .unwrap_or(0);
                if cause_id != 0 && effect_id != 0 {
                    self.causality
                        .observe_pair("syntax_cause", "syntax_effect", 1.0, 1.0);
                    self.morphogenesis
                        .add_relation(cause_id, "causes", effect_id);
                }
            }
        }
        let dep_parse = self.syntax.parse(&tokens);
        if let Some((subj, verb, obj)) = self.syntax.extract_svo(&dep_parse) {
            actions.push(format!("[SYNTAX] SVO: {} | {} | {}", subj, verb, obj));
        }
        actions.push(format!("[SCENE] {}", scene.status()));

        // 3c. Neural extractors: predict causal/physical on user input
        let input_emb = self.semantics.sentence_embedding(input);
        if !input_emb.is_empty() {
            let pc = self.neural_extractors.predict_causal(&input_emb);
            let pp = self.neural_extractors.predict_physical(&input_emb);
            actions.push(format!("[NEURAL_EX] causal={:.2} physical={:.2}", pc, pp));
        }

        // 3d. Language generation: respond to explain/why/describe queries
        let lower_input = input.to_lowercase();
        if lower_input.contains("explain")
            || lower_input.contains("why")
            || lower_input.contains("porque")
            || lower_input.contains("describe")
        {
            let query = lower_input
                .replace("explain", "")
                .replace("why", "")
                .replace("porque", "")
                .replace("describe", "")
                .trim()
                .to_string();
            if !query.is_empty() {
                let ans = self.language_gen.answer_why(&self.morphogenesis, &query);
                actions.push(format!("[LANG_GEN] {}", ans));
            }
        }

        // 3b. Grounding Fase 1B + World Model Fase 2: vision capture every 10 ticks
        if self.state.tick_count % 10 == 0 {
            let screen = self.computer.capture_screen(320, 240);
            let img = vision::ImageBuffer {
                width: 320,
                height: 240,
                pixels: screen,
            };
            let vresult = self.vision.analyze(&img);
            // Track objects across frames (object permanence + physics)
            let wm_actions =
                self.world_model
                    .track_frame(&vresult.blobs, 320, 240, self.state.tick_count);
            actions.extend(wm_actions);
            // If occlusions detected, add symbolic relations to morphogenesis
            for obj in self.world_model.objects.values() {
                if let Some(occluder) = obj.occluded_by {
                    // Map object IDs to concept IDs via morphogenesis
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
            // Grounding: bind text entities to dominant visible blob
            if !vresult.blobs.is_empty() {
                let largest = vresult
                    .blobs
                    .iter()
                    .max_by_key(|b| b.area)
                    .cloned()
                    .unwrap();
                let feat = perception::UnifiedPerception::extract_blob_features(&largest, 320, 240);
                for ent in &entities {
                    self.perception.bind_vision_text(&ent.text, &feat);
                    // Also propagate label to world model
                    let wm_label = self.world_model.assign_labels(&ent.text, &feat);
                    actions.extend(wm_label);
                    actions.push(format!("[GROUNDING] Bound '{}' -> blob {}x{}@({},{}) | feat=[{:.2},{:.2},{:.2},{:.2},{:.2}]",
                        ent.text, largest.w, largest.h, largest.x, largest.y, feat[0], feat[1], feat[2], feat[3], feat[4]));
                }
                if entities.is_empty() {
                    if let Some((label, sim)) = self.perception.recognize_vision(&feat) {
                        actions.push(format!(
                            "[GROUNDING] Vision-only recognized '{}' (sim={:.2})",
                            label, sim
                        ));
                    }
                }
            }
        }

        // 3b. Distributional semantics: observe text and build own embeddings online
        self.semantics.observe(input);
        self.semantics.tick_since_compute += 1;
        if self.semantics.tick_since_compute >= self.semantics.compute_every {
            self.semantics.compute_embeddings();
            actions.push(format!(
                "[SEMANTICS] Computed embeddings | {}",
                self.semantics.status()
            ));
        }
        let semantic_emb = scene.flatten();
        if self.semantics.vocab_size > 10 {
            let target = entities
                .first()
                .map(|e| e.text.clone())
                .unwrap_or_else(|| input.to_string());
            let nearest = self.semantics.nearest(&target, 3);
            if !nearest.is_empty() {
                let nn_str = nearest
                    .iter()
                    .map(|(w, s)| format!("{}:{:.2}", w, s))
                    .collect::<Vec<_>>()
                    .join(", ");
                actions.push(format!("[SEMANTICS] Nearest to '{}' -> {}", target, nn_str));
            }
        }

        // 4. Encode state + action for neural/self-model
        let state_vec = self.encode_state(input.len(), entities.len());
        let action_vec = Self::encode_action(&intent);

        // 5. Self-model: predict outcome before acting
        let predicted_outcome = self.self_model.predict_outcome(&state_vec, &action_vec);
        let predicted_next = self.self_model.predict_next_state(&state_vec, &action_vec);
        let unc = self.self_model.uncertainty();
        actions.push(format!(
            "[SELF] Predicted success={:.2} ±{:.3} | next_state_diff={:.3}",
            predicted_outcome[0],
            unc.get(0).copied().unwrap_or(1.0),
            state_vec
                .iter()
                .zip(predicted_next.iter())
                .map(|(a, b)| (a - b).abs())
                .sum::<f32>()
                / STATE_DIM as f32
        ));

        // 6. Unified multimodal perception: fuse swarm + semantic embeddings into unified space
        let swarm_emb = self.swarm.get_hidden();
        let unified_emb = self.perception.fuse_text_only(&swarm_emb);
        // Combine perceptual embedding with own distributional semantic embedding
        let combined_emb: Vec<f32> = unified_emb
            .iter()
            .chain(semantic_emb.iter())
            .map(|&x| x)
            .collect();
        self.workspace.set_perception_input(input.len());
        self.workspace
            .set_memory_load(self.morphogenesis.n_concepts() + self.temporal.events.len());
        self.workspace
            .set_goal_state(self.goal_stack.stack.len(), self.motivation.discomfort);
        self.workspace.set_meta_state(
            self.metacognition.self_model_error_ema,
            self.motivation.drives.curiosity,
        );
        self.workspace.set_novelty(conf);
        self.workspace.set_exploration_state(
            self.motivation.drives.curiosity,
            self.morphogenesis.n_concepts(),
        );
        self.workspace.set_social_state(
            self.multi_agent.peers.len(),
            self.state
                .tick_count
                .saturating_sub(self.temporal.events.last().map(|e| e.tick).unwrap_or(0)),
        );
        let novel_combos = self.morphogenesis.relation_count();
        let inferred_rels = if let Some(ref vm) = self.meta_vm {
            vm.n_inferred()
        } else {
            0
        };
        self.workspace
            .set_creativity_state(novel_combos, inferred_rels);
        // Fase 6: Workspace modulates processing depth in reactive mode
        let broadcast = self.workspace.tick(self.state.tick_count);
        if let Some(ref b) = broadcast {
            actions.push(format!(
                "[WORKSPACE] Winner={} | confidence={:.2} | serial attention active",
                b.agent_name, b.confidence
            ));
        } else {
            actions.push(format!(
                "[WORKSPACE] No winner | below threshold={:.2}",
                self.workspace.global_threshold
            ));
        }
        actions.push(format!(
            "[PERCEPTION] {} | semantic_dim={}",
            self.perception.status(),
            semantic_emb.len()
        ));

        // 7. Swarm ensemble embedding -> morphogenesis (using combined perceptual + semantic embedding)
        let embedding = combined_emb;
        let tension = if unc.get(0).copied().unwrap_or(0.0) > 0.5 {
            2.0 * (1.0 - conf)
        } else {
            1.0 - conf
        };
        // Meta-evolution: if a VM is loaded, compute evolved alpha for centroid update
        if let Some(ref mut vm) = self.meta_vm {
            let closest = self.morphogenesis.classify(&embedding);
            let (dist, count) = closest
                .map(|(id, d)| {
                    let c = self
                        .morphogenesis
                        .concepts
                        .get(&id)
                        .map(|c| c.count)
                        .unwrap_or(0);
                    (d, c)
                })
                .unwrap_or((1.0, 0));
            let alpha = vm.run(dist, count);
            self.morphogenesis.evolved_alpha = Some(alpha);
            actions.push(format!(
                "[METAEVO] Centroid alpha={:.3} | dist={:.3} | count={}",
                alpha, dist, count
            ));
        }
        let (concept_id, is_new) =
            self.morphogenesis
                .add_sample(&embedding, input, self.state.tick_count, tension);
        if is_new {
            actions.push(format!(
                "[MORPHO] New concept born: id={} (tension={:.2})",
                concept_id,
                self.morphogenesis.tension()
            ));
        }
        if self.state.last_concept_id != 0 {
            let predicted_next = self
                .morphogenesis
                .predict_next_concept(self.state.last_concept_id)
                .unwrap_or(0);
            actions.push(format!(
                "[MORPHO] Anticipated next concept: id={}",
                predicted_next
            ));
        }
        // Record temporal transition + symbolic relation between concepts
        if concept_id != 0 && concept_id != self.state.last_concept_id {
            if self.state.last_concept_id != 0 {
                self.morphogenesis
                    .record_transition(self.state.last_concept_id, concept_id);
                self.morphogenesis
                    .add_relation(self.state.last_concept_id, "leads_to", concept_id);
            }
            self.state.last_concept_id = concept_id;
        }
        // Meta-evolution graph reasoning: evolved program discovers new relations
        if let Some(ref mut vm) = self.meta_vm {
            if concept_id != 0 {
                vm.run_graph(&mut self.morphogenesis, concept_id);
                if vm.n_inferred() > 0 {
                    actions.push(format!(
                        "[METAEVO] Graph inference discovered {} relation(s) via evolved program",
                        vm.n_inferred()
                    ));
                }
            }
        }
        // If new concept has parent, add symbolic "is_a" relation + inherit properties
        if is_new {
            let (parent_id_opt, label) = self
                .morphogenesis
                .concepts
                .get(&concept_id)
                .map(|c| (c.parent_id, c.label.clone()))
                .unwrap_or((None, String::new()));
            if let Some(pid) = parent_id_opt {
                self.morphogenesis.add_relation(concept_id, "is_a", pid);
                let inherited = self.morphogenesis.propagate_property(pid, "es_comestible");
                if inherited > 0 {
                    actions.push(format!(
                        "[ABSTRACTION] Concept {} inherited {} properties from parent {}",
                        concept_id, inherited, pid
                    ));
                }
            }
            // Set default properties based on label patterns
            if label.contains("planta") || label.contains("fruta") {
                self.morphogenesis
                    .set_property(concept_id, "es_comestible", "true");
            }
            if label.contains("piedra") || label.contains("metal") {
                self.morphogenesis
                    .set_property(concept_id, "es_duro", "true");
            }
        }
        // Movimiento E: periodic property inference check
        if self.state.tick_count % 50 == 0 {
            let dist = self.morphogenesis.abstraction_distribution();
            actions.push(format!(
                "[ABSTRACTION] Levels: perception={} | object={} | category={} | abstract={}",
                dist[0], dist[1], dist[2], dist[3]
            ));
        }
        // Internal language: encode this perception as a thought
        let activated_concepts = if concept_id != 0 {
            vec![concept_id]
        } else {
            vec![]
        };
        self.internal_language
            .encode_perception(activated_concepts, self.state.tick_count);

        // 7. Auto-dispatch tools for common question patterns (zero LLM)
        let input_lower = input.to_lowercase();
        if input_lower.starts_with("cuanto es")
            || input_lower.starts_with("how much is")
            || input_lower.starts_with("calculate")
        {
            let expr = input
                .replace("cuanto es", "")
                .replace("how much is", "")
                .replace("calculate", "")
                .trim()
                .to_string();
            if !expr.is_empty() {
                let call = tools::ToolCall {
                    tool_name: "calculator".into(),
                    args: {
                        let mut h = HashMap::new();
                        h.insert("expression".into(), expr.clone());
                        h
                    },
                };
                let result = self.tool_registry.execute(&call);
                actions.push(format!(
                    "[AUTO-TOOL] calculator | {} = {}",
                    expr, result.output
                ));
            }
        } else if input_lower.starts_with("que sabes de")
            || input_lower.starts_with("what do you know about")
            || input_lower.starts_with("search")
        {
            let query = input
                .replace("que sabes de", "")
                .replace("what do you know about", "")
                .replace("search", "")
                .trim()
                .to_string();
            if !query.is_empty() {
                let call = tools::ToolCall {
                    tool_name: "search_corpus".into(),
                    args: {
                        let mut h = HashMap::new();
                        h.insert("query".into(), query.clone());
                        h
                    },
                };
                let result = self.tool_registry.execute(&call);
                actions.push(format!(
                    "[AUTO-TOOL] search_corpus | {} matches found | preview: {}",
                    query,
                    if result.output.len() > 100 {
                        &result.output[..100]
                    } else {
                        &result.output
                    }
                ));
            }
        } else if input_lower.starts_with("eval") || input_lower.starts_with("evaluate") {
            let code = input
                .replace("eval", "")
                .replace("evaluate", "")
                .trim()
                .to_string();
            if !code.is_empty() {
                let call = tools::ToolCall {
                    tool_name: "eval".into(),
                    args: {
                        let mut h = HashMap::new();
                        h.insert("code".into(), code.clone());
                        h
                    },
                };
                let result = self.tool_registry.execute(&call);
                actions.push(format!("[AUTO-TOOL] eval | result = {}", result.output));
            }
        } else if input_lower.starts_with("think")
            || input_lower.starts_with("razona")
            || input_lower.starts_with("reason")
        {
            // Chain-of-thought on demand
            let query = input
                .replace("think", "")
                .replace("razona", "")
                .replace("reason", "")
                .trim()
                .to_string();
            let prompt_text = if query.is_empty() {
                format!(
                    "tick {} mood {}",
                    self.state.tick_count,
                    self.mood.dominant_quadrant()
                )
            } else {
                query
            };
            let prompt_tokens: Vec<usize> = prompt_text
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| !w.is_empty())
                .filter_map(|w| self.semantics.vocab.get(w).copied())
                .collect();
            if !prompt_tokens.is_empty() && self.big_transformer.vocab_size > 0 {
                let reasoning = self.big_transformer.generate(&prompt_tokens, 10, 0.8, 5);
                let reasoning_words: Vec<String> = reasoning
                    .iter()
                    .filter_map(|&t| self.semantics.index_to_word.get(t).cloned())
                    .collect();
                let answer = self.big_transformer.generate(&prompt_tokens, 5, 0.6, 3);
                let answer_words: Vec<String> = answer
                    .iter()
                    .filter_map(|&t| self.semantics.index_to_word.get(t).cloned())
                    .collect();
                actions.push(format!(
                    "[COT] reasoning: {} | answer: {}",
                    reasoning_words.join(" "),
                    answer_words.join(" ")
                ));
            } else {
                actions.push("[COT] insufficient vocabulary for reasoning".to_string());
            }
        }

        // 7b. Dispatch by intent
        let mut success = 0.0f32;
        let mut n_out = 0.0f32;
        match intent {
            nlp::Intent::ToolCall => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    let tool_name = parts[1];
                    let mut args = HashMap::new();
                    for part in &parts[2..] {
                        if let Some(eq) = part.find('=') {
                            args.insert(part[..eq].to_string(), part[eq + 1..].to_string());
                        }
                    }
                    let call = tools::ToolCall {
                        tool_name: tool_name.to_string(),
                        args,
                    };
                    let result = self.tool_registry.execute(&call);
                    success = if result.success { 1.0 } else { 0.0 };
                    n_out = actions.len() as f32;
                    actions.push(format!(
                        "[TOOL] {} | success={} | output={}",
                        tool_name, result.success, result.output
                    ));
                }
            }
            nlp::Intent::ExecuteCode => {
                let code = input
                    .replace("ejecuta codigo", "")
                    .replace("run code", "")
                    .trim()
                    .to_string();
                if !code.is_empty() {
                    let result = self.sandbox.execute(&code, "sh");
                    success = if result.exit_code == 0 { 1.0 } else { 0.0 };
                    actions.push(format!(
                        "[SANDBOX] exit={} | stdout_len={} | stderr_len={}",
                        result.exit_code,
                        result.stdout.len(),
                        result.stderr.len()
                    ));
                }
            }
            nlp::Intent::BrowseUrl => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    let url = parts[1];
                    let fetch = tools::ToolCall {
                        tool_name: "web_fetcher".into(),
                        args: {
                            let mut h = HashMap::new();
                            h.insert("url".into(), url.into());
                            h
                        },
                    };
                    let result = self.tool_registry.execute(&fetch);
                    success = if result.success { 1.0 } else { 0.0 };
                    actions.push(format!("[BROWSE] {} | {}", url, result.output));
                }
            }
            nlp::Intent::ComputerUse => {
                self.computer.scan_processes_deep();
                let top = self.computer.top_processes(3);
                for p in top {
                    actions.push(format!(
                        "[COMPUTER] PID {} | {} | CPU {:.1}% | Mem {}KB",
                        p.pid, p.name, p.cpu_percent, p.mem_kb
                    ));
                }
                success = 1.0;
            }
            _ => {}
        }

        // 8. Measure actual outcome and train predictors
        let actual_outcome = vec![
            success,
            (actions.len() as f32).clamp(0.0, 20.0) / 20.0,
            (n_out / 10.0).clamp(0.0, 1.0),
            (self.computer.state.processes.len() as f32).clamp(0.0, 1000.0) / 1000.0,
            self.motivation.discomfort,
            (self.mood.valence + 1.0) / 2.0,
        ];
        let actual_next = self.encode_state(input.len(), entities.len());
        let err = self.self_model.train(&actual_outcome[..3], &actual_next);
        // Train swarm ensemble with same signal (competitive learning)
        let mut sa = state_vec.clone();
        sa.extend(&action_vec);
        let swarm_err = self.swarm.train(&sa, &actual_outcome);
        self.metacognition
            .observe(swarm_err, err, self.morphogenesis.n_concepts());
        self.workspace
            .set_meta_state(err.clamp(0.0, 1.0), self.motivation.drives.curiosity);
        // Symbolic causal tagging: if error is high, mark last concept transition as risky
        if err > 1.0 && self.state.last_concept_id != 0 && concept_id != 0 {
            self.morphogenesis
                .add_relation(self.state.last_concept_id, "causes_risk", concept_id);
        }
        // Causal learning: observe action → outcome pairs
        let proc_load = self.computer.state.processes.len() as f32 / 1000.0;
        self.causality
            .observe_pair("action_success", "load_avg", success, 1.0 - proc_load);
        self.causality
            .observe_pair("action_success", "memory_free", success, proc_load);
        self.causality.observe_pair(
            "prediction_error",
            "action_success",
            1.0 - err.clamp(0.0, 1.0),
            success,
        );
        actions.push(format!(
            "[SELF] Prediction error: {:.4} | mean_err: {:.4}",
            err,
            self.self_model.mean_error()
        ));
        actions.push(format!("[CAUSAL] {}", self.causality.status()));

        // 9. Temporal logging (rich episodic memory)
        let intent_label = format!("{:?}", intent);
        let now_sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let concepts_activated = if concept_id != 0 {
            vec![concept_id, self.state.last_concept_id]
        } else {
            vec![]
        };
        self.temporal.log(
            self.state.tick_count,
            now_sec,
            &intent_label,
            self.state.last_concept_id,
            input,
            self.mood.valence,
            self.mood.arousal,
            err,
            actions.clone(),
            concepts_activated,
        );

        // 10. Motivation update + domain learning progress tracking + autonomous curriculum
        let elapsed_ms = start.elapsed().as_millis() as f32;
        self.motivation.update(err, actions.len(), elapsed_ms);
        // Observe per-domain errors for learning progress tracking
        self.motivation
            .observe_domain("prediction", err.clamp(0.0, 1.0));
        let vision_err = if self.state.tick_count % 10 == 0 {
            0.3
        } else {
            0.0
        }; // proxy from world model
        self.motivation.observe_domain("vision", vision_err);
        let sem_err = 1.0 - (self.semantics.vocab_size as f32 / 100.0).clamp(0.0, 1.0); // proxy: more vocab = lower err
        self.motivation.observe_domain("semantics", sem_err);
        let world_err = self.world_model.predictor.pred_err_ema.clamp(0.0, 1.0);
        self.motivation.observe_domain("world", world_err);
        // Auto-switch focus domain if stagnating
        if let Some(old_focus) = self.motivation.maybe_switch_focus() {
            let new_focus = self.motivation.current_focus.clone();
            actions.push(format!(
                "[MOTIVE] SWITCH focus | {} -> {} | learning progress stagnated",
                old_focus, new_focus
            ));
        }
        // Best learning opportunity
        let (opp_domain, opp_err) = self.motivation.best_learning_opportunity();
        if opp_err > 0.5 && self.motivation.ticks_since_switch > 10 {
            actions.push(format!("[MOTIVE] Learning opportunity | domain={} | error={:.2} | consider synthetic training", opp_domain, opp_err));
        }
        // If discomfort is high, push a hierarchical goal
        if self.motivation.discomfort > 0.6 {
            let dominant = self.motivation.dominant_drive();
            let goal_id = self.goal_stack.push_drive_goal(
                dominant,
                self.motivation.discomfort,
                self.state.tick_count,
            );
            actions.push(format!(
                "[GOALS] Pushed drive goal | id={} | drive={} | depth={}",
                goal_id,
                dominant,
                self.goal_stack.stack.len()
            ));
        }
        actions.push(format!("[MOTIVE] {}", self.motivation.status()));

        // 11. Mood (affective state) — modulates thresholds
        self.mood.update(self.motivation.discomfort, err);
        self.morphogenesis.tension_threshold = 0.5 * self.mood.tension_multiplier();
        self.morphogenesis.creation_threshold = 0.35 * self.mood.tension_multiplier();
        actions.push(format!("[MOOD] {}", self.mood.status()));

        // 12. Theory of Mind (model user)
        let (sentiment, _) = nlp::enhanced_sentiment(input);
        self.theory_of_mind.observe(&intent_label, sentiment);
        actions.push(format!("[ToM] {}", self.theory_of_mind.status()));
        if let Some(pred) = self.theory_of_mind.predict_next_intent() {
            actions.push(format!("[ToM] Predicted user next intent: {}", pred));
        }

        // 13. Observability
        self.observability
            .record_metric("GARM_input_length", input.len() as f64, HashMap::new());
        self.observability
            .record_metric("GARM_intent_confidence", conf as f64, HashMap::new());
        self.observability
            .record_metric("GARM_prediction_error", err as f64, HashMap::new());
        self.observability.record_histogram(
            "GARM_latency_ms",
            elapsed_ms as f64,
            &[1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0],
        );

        // 14. Hippocampus: store episodic memory of this tick
        let actions_summary = if actions.len() > 5 {
            format!("{} actions", actions.len())
        } else {
            actions.join("; ")
        };
        self.hippocampus.store(
            self.state.tick_count,
            &semantic_emb,
            concept_id,
            self.mood.valence,
            self.mood.arousal,
            &actions_summary,
            &input[..input.len().min(50)],
        );
        // Retrieve similar past episodes for context
        let retrieved = self.hippocampus.retrieve(&semantic_emb);
        if !retrieved.is_empty() {
            let ctx = retrieved
                .iter()
                .map(|(ep, sim)| format!("tick{}:{:.2}", ep.tick, sim))
                .collect::<Vec<_>>()
                .join(", ");
            actions.push(format!("[HIPPO] Retrieved similar episodes | {}", ctx));
        }

        actions
    }

    pub fn status_summary(&self) -> String {
        format!(
            "garm | Caps: {} | Ticks: {} | Idle: {} | NLP: {} | Tools: {} | Security: {} | Observability: {} | Neural: {} | Morpho: {} | Rel: {} | Evo: {} | Self: {} | Temporal: {} | Motive: {} | Mood: {} | ToM: {} | Swarm: {} | Meta: {} | Lang: {} | Mesh: {} | Causal: {} | MetaEvo: {} | Perception: {} | Goals: {} | Society: {} | Semantics: {} | World: {} | Focus: {} | Plugins: {} | Hippocampus: {} | Physics: {} | Syntax: {} | Planner: {} | Transformer: {} | BigTF: {} | 3D: {} | Social: {} | Econ: {} | Debug: {} | Bus: {} | Arch: {} | BusPred: {} | Explore: {} | Gate: {} | Metab: {} | Evid: {} | Surprise: {} | Epist: {} | Circad: {} | Critic: {} | WM: {} | WMNN: {} | MoE: {} | Hier: {} | ProgInd: {} | CausalM: {} | EWC: {} | Homeo: {} | MetaL: {} | MDL: {} | EmoMod: {} | Recur: {} | DNC: {} | FEP: {} | Body: {} | TempHier: {} | SelfMod: {} | Hub: {} | Logic: {} | Safe: {} | Phenom: {} | EconAgent: {} | Oracle: {} | BPTT: {} | CorpusM: {}",
            self.state.capabilities.len(),
            self.state.tick_count,
            self.state.idle_ticks,
            self.nlp.templates.len(),
            self.tool_registry.tools.len(),
            self.security.history.len(),
            self.observability.metrics.len(),
            self.neural.n_weights(),
            self.morphogenesis.n_concepts(),
            self.morphogenesis.relation_count(),
            self.evolution.generation,
            self.self_model.prediction_count,
            self.temporal.events.len(),
            self.motivation.dominant_drive(),
            self.mood.dominant_quadrant(),
            self.theory_of_mind.preferred_action,
            self.swarm.specialists[self.swarm.best_idx].name,
            self.metacognition.exploration_mode(),
            self.internal_language.thoughts.len(),
            self.multi_agent.peers.len(),
            self.causality.edges.len(),
            self.meta_vm.as_ref().map(|vm| vm.program.len()).unwrap_or(0),
            self.perception.status(),
            self.goal_stack.stack.len(),
            self.workspace.status(),
            self.semantics.vocab_size,
            self.world_model.objects.len(),
            self.motivation.current_focus.clone(),
            self.plugins.plugins.len(),
            self.hippocampus.n_episodes(),
            self.physics.objects.len(),
            self.syntax.status(),
            self.planner.status(),
            self.transformer.status(),
            self.big_transformer.status(),
            self.world3d.status(),
            self.social_complex.status(),
            self.autonomy_econ.status(),
            self.auto_debug.status(),
            self.unified_bus.status(),
            self.architecture_model.status(),
            self.bus_predictor.status(),
            self.exploration.status(),
            self.gate.status(),
            self.metabolism.status(),
            self.evidence.status(),
            self.surprise.status(),
            self.epistemic.status(),
            self.circadian.status(),
            self.critic.status(),
            self.working_memory.status(),
            self.world_model_nn.status(),
            self.moe.status(),
            self.hierarchical_attention.status(),
            self.program_induction.status(),
            self.causal_model.status(),
            self.continual_learning.status(),
            self.homeostasis.status(),
            self.meta_learning.status(),
            self.mdl_pruner.status(),
            self.emotional_modulation.status(),
            self.recurrent_state.status(),
            self.dnc.status(),
            self.active_inference.status(),
            self.embodiment.status(),
            self.temporal_hierarchy.status(),
            self.self_modification.status(),
            self.unified_hub.status(),
            self.logic_reasoning.status(),
            self.constitutional_safety.status(),
            self.phenomenology.status(),
            self.economic_agent.status(),
            self.reward_oracle.status(),
            self.bptt.status(),
            self.corpus_massive.status(),
        )
    }

    /// Cross-modal training: process a paired (image, caption) sample.
    /// EDEN ve la imagen y lee el caption simultaneamente. Las relaciones
    /// blob<->concept se aprenden por co-ocurrencia temporal.
    pub fn process_synthetic_pair(
        &mut self,
        scene: &synthetic_vision::SyntheticScene,
    ) -> Vec<String> {
        let mut actions = Vec::new();
        self.state.tick_count += 1;

        // 1. Vision: analyze the synthetic image
        let v_result = self.vision.analyze(&scene.image);
        actions.push(format!(
            "[VISION] blobs={} edges={} brightness={:.1}",
            v_result.blobs.len(),
            v_result.edge_count,
            v_result.avg_brightness,
        ));

        // 2. Track blobs in world model (gives them ids and persistence)
        let wm_actions = self.world_model.track_frame(
            &v_result.blobs,
            scene.image.width,
            scene.image.height,
            self.state.tick_count,
        );
        for a in wm_actions {
            actions.push(a);
        }

        // 3. Process the caption text through the syntax/scene pipeline
        let tokens = nlp::tokenize(&scene.caption);
        if tokens.len() < 2 {
            return actions;
        }
        self.semantics.observe(&scene.caption);
        let dim = self.semantics.embed_dim.max(1);
        let scene_vec = self.scene_parser.parse(&tokens, &self.semantics, dim);

        // 4. Create a concept for the caption (dedup by phrase)
        let key = corpus_reader::CorpusReader::normalize_phrase(&scene.caption);
        let cap_concept_id = if let Some(&id) = self.corpus_reader.phrase_to_concept.get(&key) {
            id
        } else {
            let flat = scene_vec.flatten();
            let id = self.morphogenesis.next_id;
            self.morphogenesis.next_id += 1;
            let label = if scene.caption.len() > 60 {
                scene.caption[..60].to_string()
            } else {
                scene.caption.clone()
            };
            self.morphogenesis.concepts.insert(
                id,
                morphogenesis::Concept {
                    id,
                    centroid: flat,
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
            self.corpus_reader.phrase_to_concept.insert(key, id);
            id
        };

        // 5. Cross-modal binding: when caption mentions specific shape/motion words,
        //    bind that text concept to the world_model objects detected in the image.
        // Multi-blob: concept_to_visuals contiene TODOS los blobs asociados al concepto,
        // mientras concept_to_visual mantiene el primero (backward compat).
        let mut bindings_made = 0usize;
        for obj in self.world_model.objects.values() {
            if obj.last_seen_tick + 2 >= self.state.tick_count {
                self.grounding
                    .concept_to_visual
                    .entry(cap_concept_id)
                    .or_insert(obj.id);
                let inserted = self
                    .grounding
                    .concept_to_visuals
                    .entry(cap_concept_id)
                    .or_insert_with(std::collections::HashSet::new)
                    .insert(obj.id);
                if inserted {
                    self.grounding.n_text_visual_bindings += 1;
                    bindings_made += 1;
                }
            }
        }

        // 6. Also classify scene-related concepts as physical (motion/falls/etc.)
        self.grounding
            .classify_concept_as_physical(cap_concept_id, &scene.caption);

        // 7. Episodic memory: store this multi-modal episode
        let flat_emb = scene_vec.flatten();
        if !flat_emb.is_empty() {
            self.hippocampus.store(
                self.state.tick_count,
                &flat_emb,
                cap_concept_id,
                self.mood.valence,
                self.mood.arousal,
                "synthetic_vision",
                if scene.caption.len() > 80 {
                    &scene.caption[..80]
                } else {
                    &scene.caption
                },
            );
        }

        // 8. Word-level vision attribution: associate caption keywords with detected blobs
        // (rough heuristic: each visible blob "co-occurs" with each meaningful caption word)
        let meaningful: Vec<String> = tokens
            .iter()
            .filter(|t| {
                t.len() >= 4 && !["the", "a", "an", "of", "on", "is", "are"].contains(&t.as_str())
            })
            .cloned()
            .collect();
        if !meaningful.is_empty() && !v_result.blobs.is_empty() {
            actions.push(format!(
                "[CROSS-MODAL] caption_concept={} | shape={:?} | motion={} | blobs={} | bindings+={} | words={:?}",
                cap_concept_id, scene.shape, scene.motion_label, v_result.blobs.len(),
                bindings_made,
                meaningful.iter().take(3).collect::<Vec<_>>(),
            ));
        }

        // 9. CIRCADIAN rhythm: advance global phase (vigilance/consolidation/repair)
        let phase_changed = self.circadian.tick();
        if phase_changed {
            actions.push(format!("[CIRCADIAN] phase={:?}", self.circadian.phase));
        }

        // 10. METABOLISM: energy tick and hibernation check
        self.metabolism.tick();
        if self.metabolism.hibernating {
            actions.push(format!(
                "[METABOLISM] hibernating | {}",
                self.metabolism.status()
            ));
            return actions; // skip rest if hibernating
        }

        // 11. EVIDENCE ACCUMULATOR: try to resolve any competing interpretations
        if let Some(winner) = self.evidence.tick() {
            actions.push(format!(
                "[EVIDENCE] winner='{}' evidence={:.2} from={}",
                winner.label, winner.evidence, winner.source
            ));
        }

        // 12. SURPRISE DETECTION: if bus prediction was bad, trigger learning boost
        let bus_err = self.bus_predictor.pred_error_ema;
        if self.surprise.observe(self.state.tick_count, bus_err) {
            actions.push(format!(
                "[SURPRISE] detected! err={:.3} | boost_lr={:.1}x",
                bus_err,
                self.surprise.lr_multiplier()
            ));
        }

        // 13. EPISTEMIC CONFIDENCE: report confidence from active modules
        self.epistemic.report(
            "planner",
            self.planner.n_simulations.max(1) as f32 / 100.0,
            1.0 / (self.planner.n_simulations.max(1) as f32),
        );
        self.epistemic.report(
            "big_transformer",
            self.big_transformer.n_train_steps.max(1) as f32 / 1000.0,
            0.5,
        );
        self.epistemic.report(
            "autonomy_econ",
            self.autonomy_econ.goals.len() as f32 / 10.0,
            0.3,
        );
        if let Some(top) = self.epistemic.ranked_sources().first() {
            actions.push(format!("[EPISTEMIC] top='{}' conf={:.2}", top.0, top.1));
        }

        // 14. GATE / VETO: if a risky action is proposed, gate it before execution
        if let Some(last_action) = actions.last() {
            let proposal = gate::ActionProposal {
                source: "tick_cycle".to_string(),
                action_label: last_action.clone(),
                confidence: 0.5 + self.epistemic.get("planner") * 0.5,
                cost_estimate: self.metabolism.cost_per_tool * 0.1,
            };
            if !self.gate.evaluate(&proposal) {
                actions.push(format!("[GATE] VETOED '{}'", proposal.action_label));
            }
        }

        actions
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let caps: Vec<String> = self
            .state
            .capabilities
            .iter()
            .map(|c| format!("{:?}", c))
            .collect();
        let neural_w: Vec<f32> = self.neural.weights();
        let best_swarm_idx = self.swarm.best_idx;
        let swarm_w: Vec<f32> = self.swarm.specialists[best_swarm_idx].net.weights();
        let concepts: Vec<serde_json::Value> = self
            .morphogenesis
            .concepts
            .values()
            .map(|c| {
                serde_json::json!({
                    "id": c.id, "label": c.label, "count": c.count, "birth_tick": c.birth_tick,
                    "parent_id": c.parent_id, "children": c.children,
                })
            })
            .collect();
        let events: Vec<serde_json::Value> = self.temporal.events.iter().map(|e| serde_json::json!({
            "tick": e.tick, "timestamp_sec": e.timestamp_sec, "intent_label": e.intent_label,
            "concept_id": e.concept_id, "prediction_error": e.prediction_error,
        })).collect();
        let genomes: Vec<serde_json::Value> = self.evolution.individuals.iter().map(|ind| serde_json::json!({
            "genome": ind.genome, "fitness": ind.fitness, "generation": ind.generation, "species_id": ind.species_id,
        })).collect();
        let mut snapshot = serde_json::json!({
            "version": 4,
            "tick_count": self.state.tick_count,
            "idle_ticks": self.state.idle_ticks,
            "capabilities": caps,
            "security_scans": self.security.history.len(),
            "obs_metrics": self.observability.metrics.len(),
            "neural_weights": neural_w,
            "swarm_weights": swarm_w,
            "morpho_concepts": concepts,
            "temporal_events": events,
            "evo_generation": self.evolution.generation,
            "evo_genomes": genomes,
            "self_predictions": self.self_model.prediction_count,
            "motivation_discomfort": self.motivation.discomfort,
            "mood_quadrant": self.mood.dominant_quadrant(),
            "tom_preferred": self.theory_of_mind.preferred_action,
            "tom_history": self.theory_of_mind.intent_history,
            "swarm_best": self.swarm.specialists[best_swarm_idx].name,
            "meta_swarm_err": self.metacognition.swarm_error_ema,
            "meta_self_err": self.metacognition.self_model_error_ema,
            "thoughts_count": self.internal_language.thoughts.len(),
            "mesh_peers": self.multi_agent.peers.len(),
            "causal_edges": self.causality.edges.len(),
            "metaevo_ops": self.meta_vm.as_ref().map(|vm| vm.program.len()).unwrap_or(0),
            "perception_weights": self.perception.fusion_weights,
            "goals": self.goal_stack.goals.values().map(|g| serde_json::json!({
                "id": g.id, "label": g.label, "priority": g.priority,
                "parent_id": g.parent_id, "sub_goals": g.sub_goals,
                "deadline_tick": g.deadline_tick, "progress": g.progress,
                "completed": g.completed, "failed": g.failed,
            })).collect::<Vec<_>>(),
            "goal_stack": self.goal_stack.stack.clone(),
            "goal_next_id": self.goal_stack.next_id,
            "workspace_history": self.workspace.history.iter().map(|b| serde_json::json!({
                "content": b.content, "confidence": b.confidence,
                "agent_name": b.agent_name, "tick": b.tick,
            })).collect::<Vec<_>>(),
            "semantics_vocab": self.semantics.index_to_word.clone(),
            "semantics_counts": self.semantics.word_counts.clone(),
            "semantics_cooc": self.semantics.cooc.iter().map(|(&(i, j), &v)| serde_json::json!({"i": i, "j": j, "v": v})).collect::<Vec<_>>(),
            "semantics_total_windows": self.semantics.total_windows,
            "semantics_tick_since_compute": self.semantics.tick_since_compute,
            "semantics_embeddings": self.semantics.embeddings.clone(),
            "world_next_id": self.world_model.next_id,
            "world_pred_err": self.world_model.predictor.pred_err_ema,
            "plugin_generated_count": self.plugins.generated_count,
            "planner_horizon": self.planner.horizon,
            "planner_sims": self.planner.n_simulations,
            "planner_last_tick": self.planner.last_plan_tick,
        });
        // Append big_transformer state outside json! macro
        if let Some(obj) = snapshot.as_object_mut() {
            obj.insert(
                "bigtf_vocab_size".to_string(),
                serde_json::json!(self.big_transformer.vocab_size),
            );
            obj.insert(
                "bigtf_train_steps".to_string(),
                serde_json::json!(self.big_transformer.n_train_steps),
            );
            obj.insert(
                "bigtf_output_bias".to_string(),
                serde_json::json!(self.big_transformer.output_bias.clone()),
            );
            obj.insert(
                "bigtf_output_proj".to_string(),
                serde_json::json!(self.big_transformer.output_proj.clone()),
            );
            obj.insert(
                "bigtf_token_embedding".to_string(),
                serde_json::json!(self.big_transformer.token_embedding.clone()),
            );
            obj.insert(
                "bigtf_adapter_dim".to_string(),
                serde_json::json!(self.big_transformer.adapter_dim),
            );
            if let Some(ref adapter) = self.big_transformer.adapter {
                obj.insert(
                    "bigtf_adapter_down".to_string(),
                    serde_json::json!(adapter.down.clone()),
                );
                obj.insert(
                    "bigtf_adapter_up".to_string(),
                    serde_json::json!(adapter.up.clone()),
                );
                obj.insert(
                    "bigtf_adapter_bias_down".to_string(),
                    serde_json::json!(adapter.bias_down.clone()),
                );
                obj.insert(
                    "bigtf_adapter_bias_up".to_string(),
                    serde_json::json!(adapter.bias_up.clone()),
                );
            }
            // Serialize LoRA matrices for all layers
            let lora_a_w2: Vec<Vec<Vec<f32>>> = self
                .big_transformer
                .layers
                .iter()
                .map(|l| l.lora_a_w2.clone())
                .collect();
            let lora_b_w2: Vec<Vec<Vec<f32>>> = self
                .big_transformer
                .layers
                .iter()
                .map(|l| l.lora_b_w2.clone())
                .collect();
            let lora_a_o: Vec<Vec<Vec<f32>>> = self
                .big_transformer
                .layers
                .iter()
                .map(|l| l.lora_a_o.clone())
                .collect();
            let lora_b_o: Vec<Vec<Vec<f32>>> = self
                .big_transformer
                .layers
                .iter()
                .map(|l| l.lora_b_o.clone())
                .collect();
            obj.insert("bigtf_lora_a_w2".to_string(), serde_json::json!(lora_a_w2));
            obj.insert("bigtf_lora_b_w2".to_string(), serde_json::json!(lora_b_w2));
            obj.insert("bigtf_lora_a_o".to_string(), serde_json::json!(lora_a_o));
            obj.insert("bigtf_lora_b_o".to_string(), serde_json::json!(lora_b_o));
            obj.insert(
                "autodebug_checks".to_string(),
                serde_json::json!(self.auto_debug.n_checks),
            );
            obj.insert(
                "autodebug_errors".to_string(),
                serde_json::json!(self
                    .auto_debug
                    .diagnostics
                    .iter()
                    .filter(|d| d.level == "error")
                    .count()),
            );
            obj.insert(
                "autodebug_warnings".to_string(),
                serde_json::json!(self
                    .auto_debug
                    .diagnostics
                    .iter()
                    .filter(|d| d.level == "warning")
                    .count()),
            );
            // Agujeros 6-11
            obj.insert(
                "gate_threshold".to_string(),
                serde_json::json!(self.gate.threshold),
            );
            obj.insert(
                "gate_n_vetoed".to_string(),
                serde_json::json!(self.gate.n_vetoed),
            );
            obj.insert(
                "gate_n_allowed".to_string(),
                serde_json::json!(self.gate.n_allowed),
            );
            obj.insert(
                "metabolism_energy".to_string(),
                serde_json::json!(self.metabolism.energy),
            );
            obj.insert(
                "metabolism_total_spent".to_string(),
                serde_json::json!(self.metabolism.total_spent),
            );
            obj.insert(
                "metabolism_n_hibernations".to_string(),
                serde_json::json!(self.metabolism.n_hibernations),
            );
            obj.insert(
                "evidence_decisions".to_string(),
                serde_json::json!(self.evidence.n_decisions),
            );
            obj.insert(
                "surprise_count".to_string(),
                serde_json::json!(self.surprise.n_surprises),
            );
            obj.insert(
                "surprise_ema".to_string(),
                serde_json::json!(self.surprise.surprise_ema),
            );
            obj.insert(
                "epistemic_uncertainty".to_string(),
                serde_json::json!(self.epistemic.global_uncertainty),
            );
            obj.insert(
                "circadian_phase".to_string(),
                serde_json::json!(format!("{:?}", self.circadian.phase)),
            );
            obj.insert(
                "circadian_cycles".to_string(),
                serde_json::json!(self.circadian.n_cycles),
            );
            // Nuevos modulos arquitectura
            obj.insert(
                "critic_n_updates".to_string(),
                serde_json::json!(self.critic.n_updates),
            );
            obj.insert(
                "critic_td_ema".to_string(),
                serde_json::json!(self.critic.td_error_ema),
            );
            obj.insert(
                "wm_reads".to_string(),
                serde_json::json!(self.working_memory.n_reads),
            );
            obj.insert(
                "wm_writes".to_string(),
                serde_json::json!(self.working_memory.n_writes),
            );
            obj.insert(
                "wmnn_n_train".to_string(),
                serde_json::json!(self.world_model_nn.n_train),
            );
            obj.insert(
                "wmnn_mse".to_string(),
                serde_json::json!(self.world_model_nn.pred_err_ema),
            );
            obj.insert(
                "moe_n_calls".to_string(),
                serde_json::json!(self.moe.n_calls),
            );
            obj.insert(
                "hier_passes".to_string(),
                serde_json::json!(self.hierarchical_attention.n_passes),
            );
            obj.insert(
                "prog_induced".to_string(),
                serde_json::json!(self.program_induction.n_induced),
            );
            obj.insert(
                "causal_interv".to_string(),
                serde_json::json!(self.causal_model.n_interventions),
            );
            obj.insert(
                "causal_cf".to_string(),
                serde_json::json!(self.causal_model.n_counterfactuals),
            );
            obj.insert(
                "ewc_tasks".to_string(),
                serde_json::json!(self.continual_learning.n_tasks),
            );
            obj.insert(
                "homeo_imbalance".to_string(),
                serde_json::json!(self.homeostasis.imbalance()),
            );
            obj.insert(
                "meta_tasks".to_string(),
                serde_json::json!(self.meta_learning.n_tasks),
            );
            obj.insert(
                "meta_loss".to_string(),
                serde_json::json!(self.meta_learning.last_meta_loss),
            );
            obj.insert(
                "mdl_pruned".to_string(),
                serde_json::json!(self.mdl_pruner.n_pruned),
            );
            obj.insert(
                "emotion_valence".to_string(),
                serde_json::json!(self.emotional_modulation.valence),
            );
            obj.insert(
                "emotion_arousal".to_string(),
                serde_json::json!(self.emotional_modulation.arousal),
            );
            obj.insert(
                "recurrent_steps".to_string(),
                serde_json::json!(self.recurrent_state.n_steps),
            );
            // Agujeros 14-21
            obj.insert(
                "dnc_accesses".to_string(),
                serde_json::json!(self.dnc.n_accesses),
            );
            obj.insert(
                "fep_steps".to_string(),
                serde_json::json!(self.active_inference.n_steps),
            );
            obj.insert(
                "fep_fe".to_string(),
                serde_json::json!(self.active_inference.last_free_energy),
            );
            obj.insert(
                "body_steps".to_string(),
                serde_json::json!(self.embodiment.n_steps),
            );
            obj.insert(
                "temp_hier_frames".to_string(),
                serde_json::json!(self.temporal_hierarchy.frames.len()),
            );
            obj.insert(
                "selmod_applied".to_string(),
                serde_json::json!(self.self_modification.applied),
            );
            obj.insert(
                "hub_projections".to_string(),
                serde_json::json!(self.unified_hub.n_projections),
            );
            obj.insert(
                "logic_inferences".to_string(),
                serde_json::json!(self.logic_reasoning.n_inferences),
            );
            obj.insert(
                "safety_violations".to_string(),
                serde_json::json!(self.constitutional_safety.n_violations),
            );
            obj.insert(
                "safety_blocked".to_string(),
                serde_json::json!(self.constitutional_safety.n_blocked),
            );
            // Agujeros 22-27 (cierre de arquitectura perfecta)
            obj.insert(
                "phenom_phi_max".to_string(),
                serde_json::json!(self.phenomenology.phi_max),
            );
            obj.insert(
                "econ_net_worth".to_string(),
                serde_json::json!(self.economic_agent.net_worth()),
            );
            obj.insert(
                "reward_ema".to_string(),
                serde_json::json!(self.reward_oracle.reward_ema),
            );
            obj.insert(
                "bptt_updates".to_string(),
                serde_json::json!(self.bptt.n_updates),
            );
            obj.insert(
                "corpus_massive_gen".to_string(),
                serde_json::json!(self.corpus_massive.generated_count),
            );
            obj.insert(
                "gen_n_gen".to_string(),
                serde_json::json!(self.gen_metrics.n_generations),
            );
            obj.insert(
                "gen_n_parsed".to_string(),
                serde_json::json!(self.gen_metrics.n_parsed),
            );
            obj.insert(
                "gen_n_exec".to_string(),
                serde_json::json!(self.gen_metrics.n_executed),
            );
            obj.insert(
                "gen_reward_ema".to_string(),
                serde_json::json!(self.gen_metrics.reward_ema),
            );
            obj.insert(
                "gen_total_reward".to_string(),
                serde_json::json!(self.gen_metrics.total_reward),
            );
            obj.insert(
                "gen_unique_actions".to_string(),
                serde_json::json!(self
                    .gen_metrics
                    .unique_actions
                    .iter()
                    .cloned()
                    .collect::<Vec<String>>()),
            );
            obj.insert(
                "gen_success_buffer".to_string(),
                serde_json::json!(self.gen_metrics.success_buffer.clone()),
            );
        }
        // Append hippocampus episodes outside json! to avoid recursion limit
        let hippo_eps: Vec<serde_json::Value> = self
            .hippocampus
            .episodes
            .iter()
            .map(|ep| {
                serde_json::json!({
                    "tick": ep.tick, "concept_id": ep.concept_id,
                    "mood_valence": ep.mood_valence, "mood_arousal": ep.mood_arousal,
                    "actions_summary": ep.actions_summary, "input_snippet": ep.input_snippet,
                    "embedding": ep.embedding.clone(),
                })
            })
            .collect();
        if let Some(obj) = snapshot.as_object_mut() {
            obj.insert(
                "hippo_episodes".to_string(),
                serde_json::Value::Array(hippo_eps),
            );
        }
        // Append world_objects outside the main json! macro to avoid recursion limit
        let world_objs: Vec<serde_json::Value> = self
            .world_model
            .objects
            .values()
            .map(|o| {
                serde_json::json!({
                    "id": o.id, "birth_tick": o.birth_tick, "last_seen_tick": o.last_seen_tick,
                    "cx": o.cx, "cy": o.cy, "area": o.area,
                    "vx": o.vx, "vy": o.vy, "visible": o.visible,
                    "occluded_by": o.occluded_by, "miss_count": o.miss_count,
                    "label": o.label, "feature_sig": o.feature_sig,
                })
            })
            .collect();
        if let Some(obj) = snapshot.as_object_mut() {
            obj.insert(
                "world_objects".to_string(),
                serde_json::Value::Array(world_objs),
            );
        }
        // Append physics objects
        let phys_objs: Vec<serde_json::Value> = self.physics.objects.values().map(|o| serde_json::json!({
            "id": o.id, "label": o.label, "position_x": o.position.x, "position_y": o.position.y,
            "velocity_x": o.velocity.x, "velocity_y": o.velocity.y,
            "mass": o.mass, "rigidity": o.rigidity, "size_x": o.size.x, "size_y": o.size.y,
            "visible": o.visible, "supported_by": o.supported_by,
        })).collect();
        if let Some(obj) = snapshot.as_object_mut() {
            obj.insert(
                "physics_objects".to_string(),
                serde_json::Value::Array(phys_objs),
            );
        }
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("save GARM capability state failed: {}", e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("load GARM capability state failed: {}", e))?;
        let data: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|e| format!("parse GARM capability state failed: {}", e))?;
        if let Some(v) = data.get("tick_count").and_then(|v| v.as_u64()) {
            self.state.tick_count = v;
        }
        if let Some(v) = data.get("idle_ticks").and_then(|v| v.as_u64()) {
            self.state.idle_ticks = v;
        }
        if let Some(arr) = data.get("neural_weights").and_then(|v| v.as_array()) {
            let w: Vec<f32> = arr
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect();
            if !w.is_empty() {
                self.neural.set_weights(&w);
            }
        }
        if let Some(arr) = data.get("swarm_weights").and_then(|v| v.as_array()) {
            let w: Vec<f32> = arr
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect();
            if !w.is_empty() {
                self.swarm.specialists[self.swarm.best_idx]
                    .net
                    .set_weights(&w);
            }
        }
        if let Some(arr) = data.get("tom_history").and_then(|v| v.as_array()) {
            self.theory_of_mind.intent_history = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            self.theory_of_mind.intent_freq.clear();
            for intent in &self.theory_of_mind.intent_history {
                *self
                    .theory_of_mind
                    .intent_freq
                    .entry(intent.clone())
                    .or_insert(0) += 1;
            }
        }
        // Restore perception weights
        if let Some(arr) = data.get("perception_weights").and_then(|v| v.as_array()) {
            let w: Vec<f32> = arr
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect();
            if w.len() == self.perception.fusion_weights.len() {
                self.perception.fusion_weights = w;
            }
        }
        // Restore goals
        if let Some(arr) = data.get("goals").and_then(|v| v.as_array()) {
            self.goal_stack.goals.clear();
            for g in arr {
                let id = g.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                if id == 0 {
                    continue;
                }
                let label = g
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let priority = g
                    .get("priority")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.5);
                let parent_id = g.get("parent_id").and_then(|v| v.as_u64());
                let sub_goals: Vec<u64> = g
                    .get("sub_goals")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|x| x.as_u64()).collect())
                    .unwrap_or_default();
                let deadline_tick = g.get("deadline_tick").and_then(|v| v.as_u64());
                let progress = g
                    .get("progress")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let completed = g
                    .get("completed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let failed = g.get("failed").and_then(|v| v.as_bool()).unwrap_or(false);
                self.goal_stack.goals.insert(
                    id,
                    intention_hierarchy::Goal {
                        id,
                        label,
                        priority,
                        parent_id,
                        sub_goals,
                        deadline_tick,
                        progress,
                        completed,
                        failed,
                    },
                );
            }
        }
        if let Some(arr) = data.get("goal_stack").and_then(|v| v.as_array()) {
            self.goal_stack.stack = arr.iter().filter_map(|v| v.as_u64()).collect();
        }
        if let Some(v) = data.get("goal_next_id").and_then(|v| v.as_u64()) {
            self.goal_stack.next_id = v;
        } else {
            // fallback: ensure next_id is safe above all restored goals
            let max_id = self.goal_stack.goals.keys().copied().max().unwrap_or(0);
            self.goal_stack.next_id = max_id + 1;
        }
        // Restore workspace broadcast history (agents rebuilt by default)
        if let Some(arr) = data.get("workspace_history").and_then(|v| v.as_array()) {
            self.workspace.history = arr
                .iter()
                .filter_map(|b| {
                    let content = b.get("content")?.as_str()?.to_string();
                    let confidence = b.get("confidence")?.as_f64()? as f32;
                    let agent_name = b.get("agent_name")?.as_str()?.to_string();
                    let tick = b.get("tick")?.as_u64()?;
                    Some(society::Broadcast {
                        content,
                        confidence,
                        agent_name,
                        tick,
                    })
                })
                .collect();
        }
        // Restore semantics
        if let Some(arr) = data.get("semantics_vocab").and_then(|v| v.as_array()) {
            self.semantics.index_to_word = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            self.semantics.vocab.clear();
            for (i, w) in self.semantics.index_to_word.iter().enumerate() {
                self.semantics.vocab.insert(w.clone(), i);
            }
            self.semantics.vocab_size = self.semantics.index_to_word.len();
        }
        if let Some(arr) = data.get("semantics_counts").and_then(|v| v.as_array()) {
            self.semantics.word_counts = arr
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect();
        }
        if let Some(arr) = data.get("semantics_cooc").and_then(|v| v.as_array()) {
            self.semantics.cooc.clear();
            for entry in arr {
                let i = entry.get("i").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let j = entry.get("j").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let v = entry.get("v").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                self.semantics.cooc.insert((i, j), v);
            }
        }
        if let Some(v) = data.get("semantics_total_windows").and_then(|v| v.as_u64()) {
            self.semantics.total_windows = v;
        }
        if let Some(v) = data
            .get("semantics_tick_since_compute")
            .and_then(|v| v.as_u64())
        {
            self.semantics.tick_since_compute = v;
        }
        if let Some(arr) = data.get("semantics_embeddings").and_then(|v| v.as_array()) {
            self.semantics.embeddings = arr
                .iter()
                .filter_map(|inner| {
                    inner.as_array().map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect::<Vec<f32>>()
                    })
                })
                .collect();
        }
        // Restore world model
        if let Some(arr) = data.get("world_objects").and_then(|v| v.as_array()) {
            self.world_model.objects.clear();
            for o in arr {
                let id = o.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                if id == 0 {
                    continue;
                }
                let birth_tick = o.get("birth_tick").and_then(|v| v.as_u64()).unwrap_or(0);
                let last_seen_tick = o
                    .get("last_seen_tick")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let cx = o
                    .get("cx")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let cy = o
                    .get("cy")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let area = o
                    .get("area")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let vx = o
                    .get("vx")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let vy = o
                    .get("vy")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let visible = o.get("visible").and_then(|v| v.as_bool()).unwrap_or(false);
                let occluded_by = o.get("occluded_by").and_then(|v| v.as_u64());
                let miss_count = o.get("miss_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let label = o
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let feature_sig: Vec<f32> = o
                    .get("feature_sig")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect()
                    })
                    .unwrap_or_default();
                self.world_model.objects.insert(
                    id,
                    world_model::TrackedObject {
                        id,
                        birth_tick,
                        last_seen_tick,
                        cx,
                        cy,
                        area,
                        vx,
                        vy,
                        visible,
                        occluded_by,
                        miss_count,
                        label,
                        feature_sig,
                    },
                );
            }
        }
        if let Some(v) = data.get("world_next_id").and_then(|v| v.as_u64()) {
            self.world_model.next_id = v;
        } else {
            let max_id = self.world_model.objects.keys().copied().max().unwrap_or(0);
            self.world_model.next_id = max_id + 1;
        }
        if let Some(v) = data.get("world_pred_err").and_then(|v| v.as_f64()) {
            self.world_model.predictor.pred_err_ema = v as f32;
        }
        if let Some(v) = data.get("plugin_generated_count").and_then(|v| v.as_u64()) {
            self.plugins.generated_count = v as u32;
        }
        // Restore hippocampus episodes
        if let Some(arr) = data.get("hippo_episodes").and_then(|v| v.as_array()) {
            self.hippocampus.episodes.clear();
            for ep in arr {
                let tick = ep.get("tick").and_then(|v| v.as_u64()).unwrap_or(0);
                let concept_id = ep.get("concept_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let mood_valence = ep
                    .get("mood_valence")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let mood_arousal = ep
                    .get("mood_arousal")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let actions_summary = ep
                    .get("actions_summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let input_snippet = ep
                    .get("input_snippet")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let embedding: Vec<f32> = ep
                    .get("embedding")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect()
                    })
                    .unwrap_or_default();
                self.hippocampus.episodes.push_back(hippocampus::Episode {
                    tick,
                    concept_id,
                    mood_valence,
                    mood_arousal,
                    actions_summary,
                    input_snippet,
                    embedding,
                });
            }
        }
        // Restore planner state
        if let Some(v) = data.get("planner_horizon").and_then(|v| v.as_u64()) {
            self.planner.horizon = v as usize;
        }
        if let Some(v) = data.get("planner_sims").and_then(|v| v.as_u64()) {
            self.planner.n_simulations = v as usize;
        }
        if let Some(v) = data.get("planner_last_tick").and_then(|v| v.as_u64()) {
            self.planner.last_plan_tick = v;
        }
        // Restore physics objects
        if let Some(arr) = data.get("physics_objects").and_then(|v| v.as_array()) {
            self.physics.objects.clear();
            for o in arr {
                let id = o.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                if id == 0 {
                    continue;
                }
                let label = o
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let px = o
                    .get("position_x")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let py = o
                    .get("position_y")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let vx = o
                    .get("velocity_x")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let vy = o
                    .get("velocity_y")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.0);
                let mass = o
                    .get("mass")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(1.0);
                let rigidity = o
                    .get("rigidity")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.8);
                let sx = o
                    .get("size_x")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.05);
                let sy = o
                    .get("size_y")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(0.05);
                let visible = o.get("visible").and_then(|v| v.as_bool()).unwrap_or(false);
                let supported_by = o.get("supported_by").and_then(|v| v.as_u64());
                self.physics.objects.insert(
                    id,
                    physics::PhysicalObject {
                        id,
                        label,
                        position: physics::Vec2::new(px, py),
                        velocity: physics::Vec2::new(vx, vy),
                        mass,
                        rigidity,
                        size: physics::Vec2::new(sx, sy),
                        visible,
                        contains: Vec::new(),
                        supported_by,
                        mass_observations: vec![mass],
                    },
                );
            }
            let max_id = self.physics.objects.keys().copied().max().unwrap_or(0);
            self.physics.next_id = max_id + 1;
        }
        // Restore big transformer
        if let Some(v) = data.get("bigtf_vocab_size").and_then(|v| v.as_u64()) {
            self.big_transformer.vocab_size = v as usize;
        }
        if let Some(v) = data.get("bigtf_train_steps").and_then(|v| v.as_u64()) {
            self.big_transformer.n_train_steps = v;
        }
        if let Some(arr) = data.get("bigtf_output_bias").and_then(|v| v.as_array()) {
            self.big_transformer.output_bias = arr
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect();
        }
        if let Some(arr) = data.get("bigtf_output_proj").and_then(|v| v.as_array()) {
            self.big_transformer.output_proj = arr
                .iter()
                .filter_map(|inner| {
                    inner.as_array().map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect::<Vec<f32>>()
                    })
                })
                .collect();
        }
        if let Some(arr) = data.get("bigtf_token_embedding").and_then(|v| v.as_array()) {
            self.big_transformer.token_embedding = arr
                .iter()
                .filter_map(|inner| {
                    inner.as_array().map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect::<Vec<f32>>()
                    })
                })
                .collect();
        }
        if let Some(dim) = data.get("bigtf_adapter_dim").and_then(|v| v.as_u64()) {
            let adapter_dim = dim as usize;
            let mut adapter =
                big_transformer::Adapter::new(self.big_transformer.d_model, adapter_dim);
            if let Some(arr) = data.get("bigtf_adapter_down").and_then(|v| v.as_array()) {
                adapter.down = arr
                    .iter()
                    .filter_map(|inner| {
                        inner.as_array().map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_f64().map(|f| f as f32))
                                .collect::<Vec<f32>>()
                        })
                    })
                    .collect();
            }
            if let Some(arr) = data.get("bigtf_adapter_up").and_then(|v| v.as_array()) {
                adapter.up = arr
                    .iter()
                    .filter_map(|inner| {
                        inner.as_array().map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_f64().map(|f| f as f32))
                                .collect::<Vec<f32>>()
                        })
                    })
                    .collect();
            }
            if let Some(arr) = data
                .get("bigtf_adapter_bias_down")
                .and_then(|v| v.as_array())
            {
                adapter.bias_down = arr
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();
            }
            if let Some(arr) = data.get("bigtf_adapter_bias_up").and_then(|v| v.as_array()) {
                adapter.bias_up = arr
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();
            }
            self.big_transformer.adapter = Some(adapter);
            self.big_transformer.adapter_dim = adapter_dim;
        }
        // Restore LoRA matrices per layer
        if let Some(arr) = data.get("bigtf_lora_a_w2").and_then(|v| v.as_array()) {
            for (li, layer_arr) in arr.iter().enumerate() {
                if li >= self.big_transformer.layers.len() {
                    break;
                }
                if let Some(inner) = layer_arr.as_array() {
                    self.big_transformer.layers[li].lora_a_w2 = inner
                        .iter()
                        .filter_map(|row| {
                            row.as_array().map(|a| {
                                a.iter()
                                    .filter_map(|x| x.as_f64().map(|f| f as f32))
                                    .collect::<Vec<f32>>()
                            })
                        })
                        .collect();
                }
            }
        }
        if let Some(arr) = data.get("bigtf_lora_b_w2").and_then(|v| v.as_array()) {
            for (li, layer_arr) in arr.iter().enumerate() {
                if li >= self.big_transformer.layers.len() {
                    break;
                }
                if let Some(inner) = layer_arr.as_array() {
                    self.big_transformer.layers[li].lora_b_w2 = inner
                        .iter()
                        .filter_map(|row| {
                            row.as_array().map(|a| {
                                a.iter()
                                    .filter_map(|x| x.as_f64().map(|f| f as f32))
                                    .collect::<Vec<f32>>()
                            })
                        })
                        .collect();
                }
            }
        }
        if let Some(arr) = data.get("bigtf_lora_a_o").and_then(|v| v.as_array()) {
            for (li, layer_arr) in arr.iter().enumerate() {
                if li >= self.big_transformer.layers.len() {
                    break;
                }
                if let Some(inner) = layer_arr.as_array() {
                    self.big_transformer.layers[li].lora_a_o = inner
                        .iter()
                        .filter_map(|row| {
                            row.as_array().map(|a| {
                                a.iter()
                                    .filter_map(|x| x.as_f64().map(|f| f as f32))
                                    .collect::<Vec<f32>>()
                            })
                        })
                        .collect();
                }
            }
        }
        if let Some(arr) = data.get("bigtf_lora_b_o").and_then(|v| v.as_array()) {
            for (li, layer_arr) in arr.iter().enumerate() {
                if li >= self.big_transformer.layers.len() {
                    break;
                }
                if let Some(inner) = layer_arr.as_array() {
                    self.big_transformer.layers[li].lora_b_o = inner
                        .iter()
                        .filter_map(|row| {
                            row.as_array().map(|a| {
                                a.iter()
                                    .filter_map(|x| x.as_f64().map(|f| f as f32))
                                    .collect::<Vec<f32>>()
                            })
                        })
                        .collect();
                }
            }
        }
        // Validate LoRA dimensions after load (fix corrupted states)
        for layer in &mut self.big_transformer.layers {
            layer.validate_lora_dimensions();
        }
        // Restore auto-debug
        if let Some(v) = data.get("autodebug_checks").and_then(|v| v.as_u64()) {
            self.auto_debug.n_checks = v;
        }
        // Restore agujeros 6-11
        if let Some(v) = data.get("gate_threshold").and_then(|v| v.as_f64()) {
            self.gate.threshold = v as f32;
        }
        if let Some(v) = data.get("gate_n_vetoed").and_then(|v| v.as_u64()) {
            self.gate.n_vetoed = v;
        }
        if let Some(v) = data.get("gate_n_allowed").and_then(|v| v.as_u64()) {
            self.gate.n_allowed = v;
        }
        if let Some(v) = data.get("metabolism_energy").and_then(|v| v.as_f64()) {
            self.metabolism.energy = v as f32;
        }
        if let Some(v) = data.get("metabolism_total_spent").and_then(|v| v.as_f64()) {
            self.metabolism.total_spent = v as f32;
        }
        if let Some(v) = data
            .get("metabolism_n_hibernations")
            .and_then(|v| v.as_u64())
        {
            self.metabolism.n_hibernations = v;
        }
        if let Some(v) = data.get("evidence_decisions").and_then(|v| v.as_u64()) {
            self.evidence.n_decisions = v;
        }
        if let Some(v) = data.get("surprise_count").and_then(|v| v.as_u64()) {
            self.surprise.n_surprises = v;
        }
        if let Some(v) = data.get("surprise_ema").and_then(|v| v.as_f64()) {
            self.surprise.surprise_ema = v as f32;
        }
        if let Some(v) = data.get("epistemic_uncertainty").and_then(|v| v.as_f64()) {
            self.epistemic.global_uncertainty = v as f32;
        }
        if let Some(v) = data.get("circadian_cycles").and_then(|v| v.as_u64()) {
            self.circadian.n_cycles = v;
        }
        // Restore nuevos modulos arquitectura
        if let Some(v) = data.get("critic_n_updates").and_then(|v| v.as_u64()) {
            self.critic.n_updates = v;
        }
        if let Some(v) = data.get("critic_td_ema").and_then(|v| v.as_f64()) {
            self.critic.td_error_ema = v as f32;
        }
        if let Some(v) = data.get("wm_reads").and_then(|v| v.as_u64()) {
            self.working_memory.n_reads = v;
        }
        if let Some(v) = data.get("wm_writes").and_then(|v| v.as_u64()) {
            self.working_memory.n_writes = v;
        }
        if let Some(v) = data.get("wmnn_n_train").and_then(|v| v.as_u64()) {
            self.world_model_nn.n_train = v;
        }
        if let Some(v) = data.get("wmnn_mse").and_then(|v| v.as_f64()) {
            self.world_model_nn.pred_err_ema = v as f32;
        }
        if let Some(v) = data.get("moe_n_calls").and_then(|v| v.as_u64()) {
            self.moe.n_calls = v;
        }
        if let Some(v) = data.get("hier_passes").and_then(|v| v.as_u64()) {
            self.hierarchical_attention.n_passes = v;
        }
        if let Some(v) = data.get("prog_induced").and_then(|v| v.as_u64()) {
            self.program_induction.n_induced = v;
        }
        if let Some(v) = data.get("causal_interv").and_then(|v| v.as_u64()) {
            self.causal_model.n_interventions = v;
        }
        if let Some(v) = data.get("causal_cf").and_then(|v| v.as_u64()) {
            self.causal_model.n_counterfactuals = v;
        }
        if let Some(v) = data.get("ewc_tasks").and_then(|v| v.as_u64()) {
            self.continual_learning.n_tasks = v;
        }
        if let Some(v) = data.get("meta_tasks").and_then(|v| v.as_u64()) {
            self.meta_learning.n_tasks = v;
        }
        if let Some(v) = data.get("meta_loss").and_then(|v| v.as_f64()) {
            self.meta_learning.last_meta_loss = v as f32;
        }
        if let Some(v) = data.get("mdl_pruned").and_then(|v| v.as_u64()) {
            self.mdl_pruner.n_pruned = v;
        }
        if let Some(v) = data.get("emotion_valence").and_then(|v| v.as_f64()) {
            self.emotional_modulation.valence = v as f32;
        }
        if let Some(v) = data.get("emotion_arousal").and_then(|v| v.as_f64()) {
            self.emotional_modulation.arousal = v as f32;
        }
        if let Some(v) = data.get("recurrent_steps").and_then(|v| v.as_u64()) {
            self.recurrent_state.n_steps = v;
        }
        // Restore agujeros 14-21
        if let Some(v) = data.get("dnc_accesses").and_then(|v| v.as_u64()) {
            self.dnc.n_accesses = v;
        }
        if let Some(v) = data.get("fep_steps").and_then(|v| v.as_u64()) {
            self.active_inference.n_steps = v;
        }
        if let Some(v) = data.get("fep_fe").and_then(|v| v.as_f64()) {
            self.active_inference.last_free_energy = v as f32;
        }
        if let Some(v) = data.get("body_steps").and_then(|v| v.as_u64()) {
            self.embodiment.n_steps = v;
        }
        if let Some(v) = data.get("selmod_applied").and_then(|v| v.as_u64()) {
            self.self_modification.applied = v;
        }
        if let Some(v) = data.get("hub_projections").and_then(|v| v.as_u64()) {
            self.unified_hub.n_projections = v;
        }
        if let Some(v) = data.get("logic_inferences").and_then(|v| v.as_u64()) {
            self.logic_reasoning.n_inferences = v;
        }
        if let Some(v) = data.get("safety_violations").and_then(|v| v.as_u64()) {
            self.constitutional_safety.n_violations = v;
        }
        if let Some(v) = data.get("safety_blocked").and_then(|v| v.as_u64()) {
            self.constitutional_safety.n_blocked = v;
        }
        // Restore agujeros 22-27
        if let Some(v) = data.get("phenom_phi_max").and_then(|v| v.as_f64()) {
            self.phenomenology.phi_max = v as f32;
        }
        if let Some(v) = data.get("reward_ema").and_then(|v| v.as_f64()) {
            self.reward_oracle.reward_ema = v as f32;
        }
        if let Some(v) = data.get("bptt_updates").and_then(|v| v.as_u64()) {
            self.bptt.n_updates = v;
        }
        if let Some(v) = data.get("corpus_massive_gen").and_then(|v| v.as_u64()) {
            self.corpus_massive.generated_count = v;
        }
        if let Some(v) = data.get("gen_n_gen").and_then(|v| v.as_u64()) {
            self.gen_metrics.n_generations = v;
        }
        if let Some(v) = data.get("gen_n_parsed").and_then(|v| v.as_u64()) {
            self.gen_metrics.n_parsed = v;
        }
        if let Some(v) = data.get("gen_n_exec").and_then(|v| v.as_u64()) {
            self.gen_metrics.n_executed = v;
        }
        if let Some(v) = data.get("gen_reward_ema").and_then(|v| v.as_f64()) {
            self.gen_metrics.reward_ema = v as f32;
        }
        if let Some(v) = data.get("gen_total_reward").and_then(|v| v.as_f64()) {
            self.gen_metrics.total_reward = v as f32;
        }
        if let Some(arr) = data.get("gen_unique_actions").and_then(|v| v.as_array()) {
            self.gen_metrics.unique_actions = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
        if let Some(arr) = data.get("gen_success_buffer").and_then(|v| v.as_array()) {
            self.gen_metrics.success_buffer = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            // Ensure buffer doesn't exceed max after load
            while self.gen_metrics.success_buffer.len() > self.gen_metrics.max_buffer {
                self.gen_metrics.success_buffer.remove(0);
            }
        }
        Ok(())
    }

    /// Movimiento B: execute a single instruction generated by the transformer.
    /// Returns (log_message, additional_jump_offset).
    pub fn execute_instruction(&mut self, instr: &program::Instruction) -> (String, usize) {
        match instr {
            program::Instruction::Halt => ("HALT".to_string(), 0),
            program::Instruction::Invoke(cap) => (format!("INVOKE {:?}", cap), 0),
            program::Instruction::ToolCall { name, arg } => {
                let call = tools::ToolCall {
                    tool_name: name.clone(),
                    args: {
                        let mut h = std::collections::HashMap::new();
                        h.insert("expression".to_string(), arg.clone());
                        h
                    },
                };
                let res = self.tool_registry.execute(&call);
                (
                    format!(
                        "TOOLCALL {} | success={} | out={}",
                        name, res.success, res.output
                    ),
                    0,
                )
            }
            program::Instruction::SetGoal { label, priority } => {
                let id =
                    self.autonomy_econ
                        .push_goal(label, *priority, self.state.tick_count + 100);
                (
                    format!("SETGOAL {} priority={:.2} id={}", label, priority, id),
                    0,
                )
            }
            program::Instruction::TrainTf { n_sentences } => {
                let mut total_loss = 0.0f32;
                let mut n = 0u64;
                for _ in 0..*n_sentences {
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
                                n += 1;
                                self.benchmark.report_train_loss(loss);
                            }
                        }
                    }
                }
                let log = if n > 0 {
                    format!(
                        "TRAIN_TF {} sentences | avg_loss={:.3}",
                        n,
                        total_loss / n as f32
                    )
                } else {
                    "TRAIN_TF 0 sentences".to_string()
                };
                (log, 0)
            }
            program::Instruction::ReadBus { slot } => {
                let log = if let Some(vec) = self.unified_bus.read(slot) {
                    let activity = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
                    format!("READ_BUS {} | activity={:.2}", slot, activity)
                } else {
                    format!("READ_BUS {} | not found", slot)
                };
                (log, 0)
            }
            program::Instruction::WriteBus { slot, value } => {
                self.unified_bus.project(
                    slot,
                    &[value.parse::<f32>().unwrap_or(0.0)],
                    self.state.tick_count,
                );
                (format!("WRITE_BUS {} = {}", slot, value), 0)
            }
            program::Instruction::Wait { ticks } => (format!("WAIT {}", ticks), 0),
            program::Instruction::If {
                slot,
                threshold,
                skip,
            } => {
                if let Some(vec) = self.unified_bus.read(slot) {
                    let activity = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
                    if activity > *threshold {
                        (
                            format!(
                                "IF {} activity={:.2} > {:.2} | JUMP {}",
                                slot, activity, threshold, skip
                            ),
                            *skip,
                        )
                    } else {
                        (
                            format!(
                                "IF {} activity={:.2} <= {:.2} | NOJUMP",
                                slot, activity, threshold
                            ),
                            0,
                        )
                    }
                } else {
                    (format!("IF {} | slot not found", slot), 0)
                }
            }
            program::Instruction::Log { message } => (format!("LOG: {}", message), 0),
        }
    }

    /// Helper: flatten all bus slots into a single state vector for experience buffer.
    pub fn slots_to_vector(&self) -> Vec<f32> {
        let mut out = Vec::new();
        for slot in &self.unified_bus.slots {
            out.extend_from_slice(&slot.vector);
        }
        out
    }

    /// Pre-bootstrap the big_transformer with N synthetic state->action pairs.
    /// This gives the transformer a strong prior on the prompt->instruction pattern
    /// before the autonomous cycle begins. Critical for generative controller success.
    pub fn pre_bootstrap(&mut self, n_pairs: usize) {
        if self.big_transformer.vocab_size == 0 || n_pairs == 0 {
            return;
        }
        let moods = vec!["calm", "happy", "sad", "angry", "anxious", "focused"];
        let goals = vec![
            "explore", "learn", "build", "optimize", "defend", "create", "none",
        ];
        let actions_pool = vec![
            "SETGOAL explore 0.8",
            "SETGOAL learn 0.7",
            "SETGOAL build 0.6",
            "INVOKE planner",
            "INVOKE morphogenesis",
            "INVOKE metacognition",
            "TOOLCALL calculator explore",
            "TOOLCALL search mission",
            "TRAIN_TF 3",
            "TRAIN_TF 5",
            "WAIT 2",
            "WAIT 4",
            "READ_BUS motivation",
            "READ_BUS prediction",
            "WRITE_BUS motivation 1.5",
            "WRITE_BUS curiosity 0.9",
            "LOG mission started",
            "LOG status ok",
        ];
        let mut total_loss = 0.0f32;
        let mut trained = 0u64;
        for i in 0..n_pairs {
            let tick = (i % 1000 + 1) as u64;
            let energy = 20 + (i % 81); // 20..100
            let mood = moods[i % moods.len()];
            let goal = goals[i % goals.len()];
            let action = actions_pool[i % actions_pool.len()];
            let sentence = format!(
                "tick {} energy {} mood {} goal {} action {}",
                tick, energy, mood, goal, action
            );
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
                    trained += 1;
                }
            }
        }
        if trained > 0 {
            println!(
                "[PRE_BOOTSTRAP] {} pairs | {} trained | avg_loss={:.3}",
                n_pairs,
                trained,
                total_loss / trained as f32
            );
        }
    }
}
