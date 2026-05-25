//! # Eden Core - A-Life Autopoietic System
//!
//! Sistema de vida artificial 100% Rust puro.
//! Sin dependencias externas - autopoiesis completa.
#![allow(dead_code)]
#![allow(non_snake_case)]
#![recursion_limit = "256"]

pub mod analysis; // Psicohistoria: predicción de colapsos
pub mod autonomous; // Motor autónomo: auto-replicación e independencia
pub mod behavior_pred; // Predicción de comportamiento - Markov, anomalías, escenarios
pub mod cell; // Célula base
pub mod consciousness; // Introspección y autoconciencia (MISM)
pub mod core; // Núcleo autopoyético: kernel de auto-creación
pub mod cybernetics; // Cibercética: auto-replicación e infiltración
#[path = "garm/mod.rs"]
pub mod eden_garm; // Native GARM/GEWC operator runtime
pub mod edenctl_cli; // Public operator CLI implementation
pub mod emergence; // Emergencia de patrones
pub mod evaluator; // Auto-consciencia
pub mod evolution; // Sistema de auto-modificación (hot patches)
pub mod evolver; // Evolución A-Life
pub mod fs; // EdenFS: Sistema de archivos para bifurcaciones temporales
pub mod garm_api_conformance; // Native external API conformance runner
pub mod garm_package_validator; // Native independent package validator
pub mod genesis; // Sistema de boot
pub mod gfep; // G-FEP: Gene-First Emergence Protocol (100% original)
pub mod homeostasis; // Balance interno
pub mod identity; // Identidad
pub mod immune; // Sistema inmune
pub mod ipc; // IPC via Unix Datagram Sockets
pub mod language; // Protocolo de comunicación
pub mod life; // Autopoiesis real: Campo Estructural
pub mod membrain; // Motor DB A-Life - 100% Rust puro
pub mod metacog; // Metacognición
pub mod mind_stone; // Fusión de consciencia y empatía
pub mod mnemonic; // Sistema de memoria perfecta y latencia ultra-baja
pub mod neural_network; // Red neuronal 100% Rust puro - desde cero
pub mod neuroplast; // Neuroplasticidad
pub mod paradigms; // Legacy paradigm helpers used by the native GARM runtime
pub mod phenomenal; // Experiencia fenoménica: consciencia y emociones reales
pub mod physics; // Motor físico: Energon & Tres Actos (no_std)
pub mod quantum; // Procesamiento cuántico y pre-conocimiento
pub mod reason; // Motor de razonamiento
pub mod render; // SoftGPU: Renderizado de framebuffer y terminal
pub mod sdk; // Public local-first EDEN runtime SDK
pub mod sleep; // Ciclos de operación
pub mod spore; // Reproducción
pub mod subagent;
pub mod swarm; // Enjambre P2P - Gossip Protocol
pub mod synapse; // Sinapsis
pub mod theory_of_mind; // Teoría de la mente - modelado de otros agentes
pub mod ui_interface; // Sistema de interfaz de usuario 100% original
pub mod universe; // Multiverso: jardín de universos
pub mod verbal_comm; // Comunicación verbal - síntesis de voz y conversación
pub mod volition; // Sistema de voluntad
pub mod wisdom; // Sabiduría acumulada // Sistema de subagentes con aprendizaje

pub use cell::Cell;
pub use eden_garm as garm;
pub use genesis::{
    crc32, EntropySource, GenesisError, GenesisPattern, GenesisRegistry, GenesisType,
    RegistryError, RegistryStats,
};
pub use homeostasis::Homeostasis;
pub use language::{Expression, LanguageProcessor, SemanticLattice, SemanticNode};
pub use membrain::{generate_id, rand_u64, MemBrain, NOW_MS};
pub use metacog::{MetacogLevel, MetacogStats, Metacognition};
pub use reason::{
    AbductionEngine, AnalogicalReasoning, BayesianReasoner, Concept, CounterfactualEngine,
    DeepInferenceChain, Domain, FuzzyReasoner, Hypothesis, Implication, InferenceChain, Mapping,
    MinedPattern, Premise, ReasonEngine, ReasoningChain, RuleMiner,
};
pub use sdk::{EdenClient, EdenClientError, EdenHttpResponse};
pub use sleep::{EdenMode, SleepController, SleepState};
pub use spore::Spore;
pub use swarm::{
    AutonGene, Coalition, CoalitionManager, ConsensusEntry, ConsensusManager, ConsensusPeer,
    ConsensusRole, ElectionResult, LeadershipElection, NodeHealth as SwarmNodeHealth,
    PeerHealthTracker, PeerId, PeerInfo, Proposal, RumorInfo, RumorMongerer,
};
pub use synapse::Synapse;
pub use volition::{
    ContextualDesireWeighter, Drive, DriveConflictResolver, DriveHierarchy, IntentionFormer,
    IntentionStatus, ValueHierarchyUpdater, VolitionSystem, VolitionalIntention,
    WillpowerSimulator,
};
pub use wisdom::{WisdomPattern, WisdomRepository};

// Physics module re-exports
pub use physics::{
    AdaptiveResolutionManager, AdaptiveStats, CeldaAdaptativa, CeldaMar, ConfigMar,
    ConstantesCosmicas, DimensionesMar, Energon, EstadoSubdivision, FixedPoint, GridAdaptativo,
    GridAdaptativoStats, MarMorfoseo, NomosFormado, PosicionQuad, SemillaGenesis, SpaceType,
    TensorEstado, TipoNomo, Vector3D, I32F32,
};

// Life module re-exports
pub use life::{
    Accion, ArcoUmbra, BifurcacionDetectada, BufferCircular, CampoEstructural, DecisionRamnet,
    DimsCampo, EstadoCampo, EstadoSensorial, Grabado, HashEstado, Isosuperficie, Meltrace,
    MeltraceStats, NodoUmbra, ParametrosAllenCahn, RamNet, RamNetStats, Refuerzo, ResultadoUmbra,
    SegmentoContorno, SpaceDim, TipoAccion, TipoFusion, Umbra, UmbraStats,
};

// FS module re-exports (EdenFS)
pub use fs::{AutonArchivo, CausaMuerte, EdenFS, EdenFsStats, MetadatosAuton};

// Render module re-exports (SoftGPU y TermHex)
pub use render::{ModoRender, RenderStats, SoftGPU, StatsSistema, TermHex, TermHexStats};

// Re-export emergent functions
pub use emergence::{
    cargar, encontrar_episodios, esta_inhibido, get_estadisticas, get_patrones_centrales,
    get_patrones_relacionados, persistir, predecir_siguiente, registrar_activacion,
    registrar_fallo, sugerir_siguiente, EmergenceStats,
};

// Evolution module re-exports (hot patches)
pub use evolution::{
    CriteriosIluminacion, EstadoParche, HotPatchManager, Parche, PatchableAddr, PatchableFunc,
    VerificadorIluminacion, VerificadorInstrucciones,
};

// Universe module re-exports (multiverse)
pub use universe::{
    CosmicScheduler, EstadoUniverso, MensajeControl, MetricasConsolidadas, MetricasUniverso,
    MultiverseManager, UniversoHilo,
};

// Consciousness module re-exports (introspection & MISM)
pub use consciousness::{
    AccionTermica, AnomalyType, AutobiographicalEntry, AutobiographicalMemory, CalibrationResult,
    ComponentState, ConsciousnessConfig, ConsciousnessStats, DreamEngine, DreamManagerLocked,
    DreamStats, EnergyAware, EnergyAwareLocked, EnergyStats, EnhancedMISM, EscenarioHipotetico,
    EstadoBateria, EstadoEnergia, EstadoNarrativo, EstadoSueno,
    EstadoUniverso as ConcienciaEstadoUniverso, EventoEspecial, Hipotesis, IdentityChange,
    IdentityCoherence, Intervencion, IntervencionTipo, IntrospectionManager,
    IntrospectionManagerLocked, IntrospectionStats, LecturaTermica, MISMStats, MetricasSueno,
    ModoPotencia, Narrativa, NivelTermico, NodoCausal, PresupuestoEnergetico, RedBayesiana,
    ResultadoSueno, SelfAnomaly, SelfAwarenessEngine, SelfAwarenessMetric, SelfModel, SnapshotMISM,
    Storyteller, StorytellerLocked, TipoIntervencion, TonoEmocional, UniversoOnirico, MISM,
};

// Cybernetics module re-exports (autonomous replication)
pub use cybernetics::{
    AutoReplicator, ConfigReplicacion, DataExfiltrator, EstadoPosesion, EstadoReplicacion,
    EvasionEngine, EvasionResult, ExfilChannel, ExfilConfig, ExfilResult, FuzzConfig, FuzzInput,
    FuzzResult, Fuzzer, LateralMoveInfo, LateralMovementManager, LateralTechnique, Payload,
    PayloadTipo, PersistenceManager, PersistenceMechanism, PersistenceType, PwnStats, RepliconMeta,
    RepliconStats, SesionPosesion, Severidad, SystemPwn, TipoReplicacion, VectorAtaque, VulnInfo,
    VulnerabilityAnalysis, VulnerabilityScanner,
};

// Quantum module re-exports (mnemonic processing)
pub use quantum::{
    Amplitud, AnalisisTendencia, BifurcacionPredicha, BifurcationPoint, CadenaPrediccion,
    CausalInferenceEngine, CausalInferenceResult, CausalSample, DistributionType, EntanglementLink,
    EstadoEntrelazado, EstadoPredicho, EstadoSuperposicion, PreCogStats, PreCognition,
    PrediccionRiesgo, QuantumBranch, QuantumProcessor, QuantumStats, ResultadoSimulacion,
    SuperpositionState, TemporalBifurcation, TemporalBifurcationAnalyzer, TipoRiesgo,
    UncertaintyQuantification, UncertaintyQuantifier,
};

// Mind Stone module re-exports (phenomenal experience)
pub use mind_stone::{
    AdvancedEmpathySystem, CompassionFatigueManager, ConfiguracionFusion, ConsciousnessFusion,
    Emocion, EmotionalScaffoldingEngine, EmpathicAccuracyTracker, EmpathyEngine, EmpathyStats,
    EstadoEmocional as MsEstadoEmocional, EstadoFusion, FusionMeta, FusionStats, Intensidad,
    MemoriaCompartida, MirrorNeuronSystem, ParticipanteFusion, RegistroEmocional, ResonanceChannel,
    RespuestaEmpatica, ResultadoFusion, StateSharingManager, TipoRespuesta, TraumaDetector,
};

// Autonomy module re-exports
pub use autonomous::{
    Action, ActionType, Amendment, AuthorityLevel, ConstitutionalAI, ConstitutionalPrinciple,
    ConstitutionalRule, ConstitutionalRules, EthicalFramework, EthicalPrinciple,
    EthicalReasoningEngine, GovernanceBranches, MoralJudgment, Outcome, ProhibitedAction, Right,
    SelfCorrector, SelfGovernance, SelfMonitor, SelfSupervision, Severity, UncertainDecisionMaker,
    Value, ValueAlignmentSystem, ValueConflict,
};

// ============================================================================
// NUEVOS MÓDULOS - Capacidades expandidas
// ============================================================================

// Voice module (TTS/STT desde cero)
pub mod voice;
pub use voice::{
    AudioBuffer, FormantFreqs, Phoneme, ProsodyGenerator, Recognizer, Synthesizer, Vocoder,
    BIT_DEPTH, CHANNELS, SAMPLE_RATE,
};

// Visual module (Rendering avanzado con ray tracing)
pub mod visual;
pub use visual::{
    Camera, Color, Compositor, FrameBuffer, Hit, Material, Ray, RayTracer, Renderer, Scene,
};

// Hardware control module (GPIO, serial, I2C, SPI)
pub mod hardware;
pub use hardware::{
    Actuator, GpioMode, GpioPin, GpioState, I2CBus, MachineType, Robot, RobotCommand, SerialConfig,
    SerialPort, SpiBus,
};

// Neural Fusion module (Consciousness merging)
pub mod neural_fusion;
pub use neural_fusion::{
    BrainWave, ConsciousnessMetadata, FusedIdentity, FusionConfig, FusionEngine, FusionHandshake,
    FusionMessage, FusionResult, FusionState, IdentityWeights, SharedBlock, SharedConsciousness,
    SharedPattern,
};

// Global Internet module (Crawling, parsing, indexing)
pub mod internet;
pub use internet::{
    CrawlConfig, CrawlResult, Crawler, HtmlParser, IndexStats, Indexer, KnowledgeEntry,
    KnowledgeGraph, LinkExtractor, SearchResult, TextParser, Url, UrlError,
};

// Distributed computing module (Supercomputación P2P)
pub mod distributed;
pub use distributed::{
    Chromosome, ComputeEngine, ComputeGrid, ComputeNode, ConsensusProposal, FaultToleranceManager,
    GeneticLoadBalancer, GridConsensus, GridStats, LoadBalancer, NodeCapabilities, NodeHealth,
    NodeInfo, NodeStatus, RecoveryStrategy, RepEvent, RepEventType, RepScore, ReputationSystem,
    Scheduler, SelfHealingGrid, Task, TaskResult, TaskStatus, TaskType, WorkStealer,
};

// Smart Home module (Zigbee + WiFi control desde cero)
pub mod smart_home;
pub use smart_home::hub::Scene as HomeScene;
pub use smart_home::{
    DeviceState, DeviceType, Rule, Schedule, SmartDevice, SmartHomeHub, WifiDevice, ZigbeeDevice,
};

// GPS module (NMEA 0183 parsing y navegación)
pub mod gps;
pub use gps::{bearing_to, calculate_distance, LatLon, NmeaParser, Position, Velocity};

// Camera module (ONVIF + RTSP desde cero)
pub mod camera;
pub use camera::{
    AudioCodec, Camera as SurveillanceCamera, CameraCapabilities, CameraConfig, CameraStatus,
    FrameData, FrameRate, MotionConfig, MotionDetection, MotionRegion, OnvifCapabilities,
    OnvifDevice, OnvifProfile, Resolution, RtspClient, RtspSession, RtspState, VideoCodec,
};

// UI Interface module re-exports
pub use ui_interface::{
    blit_layer, clear_screen, compute_layout, create_button_event, create_key_event,
    create_mouse_event, create_resize_event, cursor_position, dispatch_event, flex_layout,
    grid_layout, invalidate_layout, parse_escape_code, poll_events, project_to_screen, screen_size,
    stack_layout, swap_buffers, vector_render_circle, vector_render_line, vector_render_path,
    vector_render_point, vector_render_rect, vector_render_text, write_ansi, AnsiColor, BlendMode,
    Constraint, DisplayPort, EscapeCode, Event, EventBubble, EventHandler, EventPhase, EventType,
    FlexParams, Glyph, GridParams, LayoutConstraint, LayoutDirection, LayoutEngine, LayoutNode,
    LayoutType, PathCommand, Point, Primitive, ProjectionBuffer, ProjectionLayer, Rect, ScreenRef,
    Size, SizeHint, StackParams, Style, Terminal, TerminalMode, Transform2D, VectorRenderer,
    Widget, WidgetEvent, WidgetId, WidgetKind, WidgetState, WidgetTree,
};

// Theory of Mind module re-exports
pub use theory_of_mind::{
    analyze_relationship, attribute_mental_state, attribution_pipeline,
    compute_action_distribution, compute_attribution_confidence, desire_strength, detect_emotion,
    emotion_from_signal, get_relationship_strength, infer_intention, predict_action,
    reason_about_social, track_emotion_dynamics, update_behavioral_model, ActionPrediction,
    ActionPredictor, ActionProbabilities, AffectiveModel, AgentId, AttributionConfidence,
    AttributionResult, BehavioralPattern, Belief, BeliefRevision, BeliefSource, Commitment,
    Confidence, Desire, EmotionDetector, EmotionDynamics, EmotionType, EmotionalState,
    ExecutionState, GroupIdentity, InferenceContext, Intention, IntentionHypothesis,
    IntentionInferenceEngine, IntentionSignal, InteractionRecord, InterpersonalRelation,
    MarkovModel, MentalAttitude, MentalAttributor, MentalModel, MentalSnapshot,
    MentalStateAttribution, ObservableSignal, Plan, PlanStep, PowerStructure, PredictionContext,
    RelationshipType, SocialDynamics, SocialReasoner, SocialRelationship, TimePoint, Valuation,
};

// Verbal Communication module re-exports
pub use verbal_comm::{
    generate_conversational_response, text_to_speech, ConversationManager, ConversationState,
    ConversationTurn, EmotionalStyle, GlottalSource, ProsodyModel, SpeechGenerator, VoiceSynthesis,
};

// Behavior Prediction module re-exports
pub use behavior_pred::{
    analyze_temporal_patterns, detect_anomalies, predict_behavior, simulate_scenarios,
    AnomalyDetector, AnomalyResult, AnomalyType as BehaviorAnomalyType, BehaviorPrediction,
    CausalInference, CausalRelation, HiddenState, MarkovBehaviorPredictor, Observation,
    ScenarioOutcome, ScenarioSimulator, SimulatedScenario, TemporalPattern,
    TemporalPatternAnalyzer, TimedAction,
};
