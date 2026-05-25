use crate::eden_garm;
use crate::eden_garm::nodes::api_server::{ApiRuntimeMetrics, CommandResponseBus, QueuedCommand};
use std::sync::{Arc, Mutex};

pub struct BuiltGarmGraph {
    pub graph: eden_garm::HyperGraph,
    pub shared_engine: Arc<Mutex<eden_garm::capabilities::GarmCapabilityState>>,
    pub command_queue: Arc<Mutex<Vec<QueuedCommand>>>,
    pub command_response_bus: Arc<CommandResponseBus>,
    pub meta_id: usize,
    pub legacy_memory_id: usize,
    pub legacy_reason_id: usize,
    pub legacy_dialogue_id: usize,
    pub observatory_id: usize,
    pub legacy_history_id: usize,
    pub legacy_evolution_id: usize,
    pub legacy_cognition_id: usize,
    pub campo_tension_id: usize,
    pub legacy_knowledge_graph_id: usize,
    pub legacy_autoconsumo_id: usize,
    pub legacy_venado_id: usize,
    pub legacy_paradigm_hub_id: usize,
    pub legacy_ecosystem_id: usize,
    pub legacy_rebirth_meltrace_id: usize,
    pub legacy_crawler_id: usize,
    pub help_id: usize,
    pub readiness_id: usize,
    pub organic_lifecycle_id: usize,
    pub conscious_graph_regulator_id: usize,
    pub context_augmentation_id: usize,
    pub hrm_reasoner_id: usize,
    pub voice_synthesizer_id: usize,
    pub n_caps: usize,
}

pub struct GraphBuilder;

impl GraphBuilder {
    pub fn build(
        api_port: u16,
        daemon_config: eden_garm::nodes::daemon::DaemonConfig,
        runtime_metrics: Arc<ApiRuntimeMetrics>,
    ) -> BuiltGarmGraph {
        let mut graph = eden_garm::HyperGraph::new();
        let shared_engine: Arc<Mutex<eden_garm::capabilities::GarmCapabilityState>> = Arc::new(
            Mutex::new(eden_garm::capabilities::GarmCapabilityState::new_fast()),
        );
        let command_queue: Arc<Mutex<Vec<QueuedCommand>>> = Arc::new(Mutex::new(Vec::new()));
        let command_response_bus = Arc::new(CommandResponseBus::new());
        let mut next_id = 0usize;
        let mut cap_ids: Vec<(usize, eden_garm::nodes::capability::CapabilityId)> = Vec::new();

        for cap in Self::capabilities() {
            let id = next_id;
            next_id += 1;
            graph.add_node(Box::new(eden_garm::nodes::capability::CapabilityNode::new(
                id,
                cap,
                Arc::clone(&shared_engine),
            )));
            cap_ids.push((id, cap));
        }

        let coord_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::coordinator::CoordinatorNode::new(
                coord_id,
                Arc::clone(&shared_engine),
            ),
        ));

        let human_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::human_interface::HumanInterfaceNode::new(human_id, true),
        ));

        let meta_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::meta_architect::MetaArchitectNode::new(meta_id),
        ));

        let fast_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::fast_reflexes::FastReflexesNode::new(fast_id),
        ));

        let bench_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(eden_garm::nodes::benchmark::BenchmarkNode::new(
            bench_id,
        )));

        let command_router_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::command_router::CommandRouterNode::new(command_router_id),
        ));

        let persistence_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::persistence::PersistenceNode::new(persistence_id),
        ));

        let telemetry_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(eden_garm::nodes::telemetry::TelemetryNode::new(
            telemetry_id,
        )));

        let api_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(eden_garm::nodes::api_server::ApiServerNode::new(
            api_id,
            Arc::clone(&shared_engine),
            Arc::clone(&command_queue),
            Arc::clone(&command_response_bus),
            Arc::clone(&runtime_metrics),
            api_port,
        )));

        let daemon_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(eden_garm::nodes::daemon::DaemonNode::new(
            daemon_id,
            daemon_config,
        )));

        let legacy_memory_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_memory::LegacyMemoryNode::new(legacy_memory_id),
        ));

        let legacy_reason_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_reason::LegacyReasonNode::new(legacy_reason_id),
        ));

        let legacy_dialogue_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_dialogue::LegacyDialogueNode::new(legacy_dialogue_id),
        ));

        let observatory_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::observatory::ObservatoryNode::new(observatory_id),
        ));

        let legacy_history_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_history::LegacyHistoryNode::new(legacy_history_id),
        ));

        let legacy_evolution_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_evolution::LegacyEvolutionNode::new(legacy_evolution_id),
        ));

        let legacy_cognition_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_cognition::LegacyCognitionNode::new(legacy_cognition_id),
        ));

        let campo_tension_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::campo_tension::CampoTensionNode::new(campo_tension_id),
        ));

        let legacy_knowledge_graph_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode::new(
                legacy_knowledge_graph_id,
            ),
        ));

        let legacy_autoconsumo_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_runtime_extensions::AutoconsumoNode::new(
                legacy_autoconsumo_id,
            ),
        ));

        let legacy_venado_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_runtime_extensions::VenadoCompatibilityNode::new(
                legacy_venado_id,
            ),
        ));

        let legacy_paradigm_hub_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_runtime_extensions::ParadigmHubNode::new(
                legacy_paradigm_hub_id,
            ),
        ));

        let legacy_ecosystem_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_runtime_extensions::EcoSystemNode::new(legacy_ecosystem_id),
        ));

        let legacy_rebirth_meltrace_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_runtime_extensions::RebirthMeltraceNode::new(
                legacy_rebirth_meltrace_id,
            ),
        ));

        let legacy_crawler_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::legacy_runtime_extensions::LegacyCrawlerNode::new(legacy_crawler_id),
        ));

        let help_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(eden_garm::nodes::help::HelpNode::new(help_id)));

        let readiness_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(eden_garm::nodes::readiness::ReadinessNode::new(
            readiness_id,
        )));

        let organic_lifecycle_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::organic_lifecycle::OrganicLifecycleNode::new(organic_lifecycle_id),
        ));

        let conscious_graph_regulator_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::conscious_graph_regulator::ConsciousGraphRegulatorNode::new(
                conscious_graph_regulator_id,
            ),
        ));

        let context_augmentation_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::context_augmentation::ContextAugmentationNode::new(
                context_augmentation_id,
            ),
        ));

        let hrm_reasoner_id = next_id;
        next_id += 1;
        graph.add_node(Box::new(
            eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode::new(
                hrm_reasoner_id,
            ),
        ));

        let voice_synthesizer_id = next_id;
        graph.add_node(Box::new(
            eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode::new(voice_synthesizer_id),
        ));

        let n_caps = cap_ids.len();
        for &(cid, _) in &cap_ids {
            graph.add_edge(cid, bench_id, 0.05);
            graph.add_edge(meta_id, cid, 0.1);
        }
        for &(cid, cap) in &cap_ids {
            if !matches!(
                cap,
                eden_garm::nodes::capability::CapabilityId::Security
                    | eden_garm::nodes::capability::CapabilityId::Gate
                    | eden_garm::nodes::capability::CapabilityId::Surprise
            ) {
                graph.add_edge(human_id, cid, 0.1);
            }
        }
        for &(cid, cap) in &cap_ids {
            if matches!(
                cap,
                eden_garm::nodes::capability::CapabilityId::Security
                    | eden_garm::nodes::capability::CapabilityId::Gate
                    | eden_garm::nodes::capability::CapabilityId::Surprise
            ) {
                graph.add_edge(fast_id, cid, 0.3);
            }
        }
        for &(cid, _) in &cap_ids {
            graph.add_edge(cid, coord_id, 0.1);
            graph.add_edge(coord_id, cid, 0.05);
        }
        graph.add_edge(human_id, command_router_id, 0.5);
        graph.add_edge(command_router_id, persistence_id, 0.3);
        graph.add_edge(command_router_id, telemetry_id, 0.3);
        graph.add_edge(command_router_id, api_id, 0.2);
        graph.add_edge(command_router_id, daemon_id, 0.2);
        graph.add_edge(command_router_id, legacy_memory_id, 0.3);
        graph.add_edge(command_router_id, legacy_reason_id, 0.3);
        graph.add_edge(command_router_id, legacy_dialogue_id, 0.3);
        graph.add_edge(command_router_id, observatory_id, 0.3);
        graph.add_edge(command_router_id, legacy_history_id, 0.3);
        graph.add_edge(command_router_id, legacy_evolution_id, 0.3);
        graph.add_edge(command_router_id, legacy_cognition_id, 0.3);
        graph.add_edge(legacy_memory_id, legacy_knowledge_graph_id, 0.4);
        graph.add_edge(legacy_knowledge_graph_id, legacy_reason_id, 0.4);
        graph.add_edge(legacy_knowledge_graph_id, legacy_cognition_id, 0.2);
        graph.add_edge(legacy_cognition_id, campo_tension_id, 0.4);
        graph.add_edge(campo_tension_id, legacy_evolution_id, 0.4);
        graph.add_edge(campo_tension_id, meta_id, 0.2);
        graph.add_edge(legacy_autoconsumo_id, meta_id, 0.3);
        graph.add_edge(legacy_venado_id, persistence_id, 0.3);
        graph.add_edge(legacy_paradigm_hub_id, legacy_knowledge_graph_id, 0.2);
        graph.add_edge(legacy_ecosystem_id, legacy_cognition_id, 0.2);
        graph.add_edge(legacy_rebirth_meltrace_id, legacy_evolution_id, 0.2);
        graph.add_edge(legacy_crawler_id, legacy_memory_id, 0.2);
        graph.add_edge(command_router_id, help_id, 0.3);
        graph.add_edge(legacy_memory_id, readiness_id, 0.2);
        graph.add_edge(legacy_knowledge_graph_id, readiness_id, 0.2);
        graph.add_edge(legacy_cognition_id, readiness_id, 0.2);
        graph.add_edge(readiness_id, meta_id, 0.2);
        graph.add_edge(legacy_memory_id, organic_lifecycle_id, 0.3);
        graph.add_edge(legacy_knowledge_graph_id, organic_lifecycle_id, 0.3);
        graph.add_edge(campo_tension_id, organic_lifecycle_id, 0.3);
        graph.add_edge(organic_lifecycle_id, legacy_rebirth_meltrace_id, 0.2);
        graph.add_edge(organic_lifecycle_id, legacy_cognition_id, 0.2);
        graph.add_edge(legacy_knowledge_graph_id, conscious_graph_regulator_id, 0.5);
        graph.add_edge(legacy_memory_id, conscious_graph_regulator_id, 0.2);
        graph.add_edge(organic_lifecycle_id, conscious_graph_regulator_id, 0.3);
        graph.add_edge(conscious_graph_regulator_id, organic_lifecycle_id, 0.3);
        graph.add_edge(conscious_graph_regulator_id, readiness_id, 0.3);
        graph.add_edge(conscious_graph_regulator_id, meta_id, 0.2);
        graph.add_edge(legacy_memory_id, context_augmentation_id, 0.3);
        graph.add_edge(legacy_knowledge_graph_id, context_augmentation_id, 0.4);
        graph.add_edge(legacy_history_id, context_augmentation_id, 0.2);
        graph.add_edge(context_augmentation_id, legacy_reason_id, 0.2);
        graph.add_edge(context_augmentation_id, legacy_dialogue_id, 0.2);
        graph.add_edge(legacy_memory_id, hrm_reasoner_id, 0.3);
        graph.add_edge(legacy_knowledge_graph_id, hrm_reasoner_id, 0.4);
        graph.add_edge(legacy_history_id, hrm_reasoner_id, 0.2);
        graph.add_edge(context_augmentation_id, hrm_reasoner_id, 0.4);
        graph.add_edge(hrm_reasoner_id, legacy_reason_id, 0.2);
        graph.add_edge(hrm_reasoner_id, legacy_knowledge_graph_id, 0.2);
        graph.add_edge(command_router_id, voice_synthesizer_id, 0.3);
        graph.add_edge(legacy_history_id, voice_synthesizer_id, 0.2);
        graph.add_edge(voice_synthesizer_id, legacy_history_id, 0.2);
        graph.add_edge(voice_synthesizer_id, legacy_knowledge_graph_id, 0.2);

        BuiltGarmGraph {
            graph,
            shared_engine,
            command_queue,
            command_response_bus,
            meta_id,
            legacy_memory_id,
            legacy_reason_id,
            legacy_dialogue_id,
            observatory_id,
            legacy_history_id,
            legacy_evolution_id,
            legacy_cognition_id,
            campo_tension_id,
            legacy_knowledge_graph_id,
            legacy_autoconsumo_id,
            legacy_venado_id,
            legacy_paradigm_hub_id,
            legacy_ecosystem_id,
            legacy_rebirth_meltrace_id,
            legacy_crawler_id,
            help_id,
            readiness_id,
            organic_lifecycle_id,
            conscious_graph_regulator_id,
            context_augmentation_id,
            hrm_reasoner_id,
            voice_synthesizer_id,
            n_caps,
        }
    }

    fn capabilities() -> Vec<eden_garm::nodes::capability::CapabilityId> {
        use eden_garm::nodes::capability::CapabilityId::*;
        vec![
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
        ]
    }
}
