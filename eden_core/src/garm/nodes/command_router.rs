use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GarmCommand {
    Quit,
    Tick,
    Status,
    Save,
    Load,
    Auto(usize),
    Remember(String),
    Memory,
    MemoryEval,
    CognitiveEval,
    EmbodiedEval,
    NeuralEval,
    SymbolicEval,
    SelfImprovementEval,
    FrontierArchitectureEval,
    ParadigmArchitectureEval,
    IntegrationGovernanceEval,
    GlobalExecutiveWorkspaceEval,
    GewcOperationalBenchmark,
    CapabilityRealityEval,
    ArchitectureAdvantageEval,
    ParadiseWorldcellEval,
    ParadiseIntent(String),
    ParadisePlan(String),
    ParadiseApprove(String),
    ParadiseExecute(String),
    ParadiseAudit,
    PraxisNexusEval,
    LocusLayerEval,
    LocusIngest(String),
    LocusContext(String),
    LocusAudit,
    OperatorForgeEval,
    OperatorForgeSynthesize(String),
    OperatorForgeVerify,
    OperatorForgeAudit,
    ExternalEcosystemEval,
    SovereignCognitionEval,
    ArtifactApiEval,
    TrainingEvidenceEval,
    Megatron7bEvidenceEval,
    ModelRuntimeEval,
    ModelAdapterRuntimeEval,
    ModelCheckpointManifestEval,
    TrainingHarnessEval,
    ModelGovernanceEval,
    FirstModelPrepare,
    FirstModelReadinessEval,
    ElcpPrepare,
    ElcpObjectiveEval,
    ElcpAdmissionGateEval,
    ElcpTraceQualityEval,
    ElcpReplayEval,
    ElcpDatasetFreezeEval,
    ElcpMetricsBoardEval,
    Elcp4bReadinessContractEval,
    ElcpHardeningEval,
    ElcpReadinessEval,
    RuntimeStateApiEval,
    OperationalApiEval,
    OperationalRuntimeEval,
    OperationalTaskSubmit(String),
    OperationalTaskRun,
    OperationalTaskAudit,
    OperationalActionExecute(String),
    OperationalMemoryCommit(String),
    OperationalMemoryRollback(String),
    OperationalReplayRun,
    OperationalSmokeRun,
    OperationalScenarioRun,
    OperationalPermissionsAudit,
    OperationalPermissionsDiff,
    OperationalPermissionsHistory,
    OperationalPermissionsRestore,
    OperationalPermissionsSet(String),
    OperationalRecoveryAudit,
    OperationalRecoveryRun,
    OperationalDemoRun,
    RuntimeSpineEval,
    RuntimeSpineAudit,
    RuntimeSpineVerify,
    RuntimeSpineEnforce,
    RuntimeSpineRisk,
    RuntimeSpineBreakers,
    RuntimeSpineReplay,
    GewcLifecycleControl(String),
    Query(String),
    WhatIs(String),
    Why(String),
    TellMe(String),
    Greeting,
    SelfQuery,
    Thinking,
    Feeling,
    Phi,
    Observatory,
    History,
    Start,
    Stop,
    Evolve,
    Crawl(String),
    ConceptNet(String),
    Rebirth,
    Readiness,
    ReadinessBench,
    ReadinessProbe,
    ReadinessExternal,
    ReadinessExternalRun,
    ReadinessPackage,
    ActionEvidence,
    CapabilityRegistry,
    Evaluation,
    EvaluationRun,
    EvaluationAudit,
    Learning,
    LearningRecord(String),
    LearningConsolidate,
    LearningAudit,
    WorldModel,
    WorldObserve(String),
    WorldPredict(String),
    WorldVerify,
    WorldAudit,
    WorldEval,
    Benchmark,
    BenchmarkRun,
    BenchmarkAudit,
    PlanExecutor,
    PlanExecutorPlan(String),
    PlanExecutorRun,
    PlanExecutorAudit,
    Attention,
    AttentionAttend(String),
    AttentionClear,
    AttentionAudit,
    Uncertainty,
    UncertaintyRecord(String),
    UncertaintyResolve,
    UncertaintyAudit,
    Experiment,
    ExperimentPlan(String),
    ExperimentRun,
    ExperimentAudit,
    Provenance,
    ProvenanceRecord(String),
    ProvenanceVerify,
    ProvenanceAudit,
    Policy,
    PolicyEval(String),
    PolicyAudit,
    Maturity,
    MaturityAssess(String),
    MaturityAudit,
    OrganicRitual,
    Lengua(String),
    Reloj(String),
    Juez(String),
    Voz,
    VozTexto(String),
    HybridVoice,
    HybridVoicePlan(String),
    HybridVoiceSynth(String),
    HybridVoiceAudit,
    HrmText,
    HrmTextCorpus(String),
    HrmTextIngest(String),
    HrmTextSearch(String),
    HrmTextContext(String),
    HrmTextEval,
    HrmTextObjective(String),
    HrmTextPlan,
    HrmTextRun,
    HrmTextAudit,
    Intestino,
    Piel,
    Autotuning,
    Cag,
    CagExplain(String),
    CagGaps(String),
    CagActions,
    CagAudit,
    CagPlan(String),
    CagRun(String),
    Hrm(String),
    HrmRun(String),
    ModelRegister(String),
    ModelLoad(String),
    ModelEvaluate(String),
    ModelUnload(String),
    ModelAudit,
    ReadinessPlan,
    ReadinessRun,
    GarmAudit,
    GarmReport,
    GarmReportHistory,
    GarmExport,
    GarmImport,
    GarmVerifyExport,
    GarmArtifacts,
    Goals,
    GoalsPlan(String),
    GoalsRun,
    GoalsAudit,
    GarmBackup,
    GarmRestore,
    GarmCompact,
    Organs,
    OrgansAudit,
    OrgansPlan,
    OrgansRun,
    OrgansHealth,
    OrgansRepair,
    OrgansActions,
    OrgansFeedback(bool),
    Help,
    Migration,
    Unknown(String),
}

pub struct CommandRouterNode {
    id: usize,
    routed: u64,
    last_command: String,
    internal_fe: f32,
}

impl CommandRouterNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            routed: 0,
            last_command: String::new(),
            internal_fe: 1.0,
        }
    }

    pub fn parse(&mut self, raw: &str) -> GarmCommand {
        self.routed += 1;
        Self::parse_raw(raw, &mut self.last_command)
    }

    pub fn validate_surface(&mut self) -> String {
        let command = self.parse("organos audit");
        format!(
            "[ROUTER-AUTO] surface_valid={} routed={} last={}",
            matches!(command, GarmCommand::OrgansAudit),
            self.routed,
            self.last_command
        )
    }

    pub fn router_snapshot(&self) -> String {
        format!(
            "router:routed:{} last:{} internal_fe:{:.3}",
            self.routed, self.last_command, self.internal_fe
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "routed": self.routed,
            "last_command": self.last_command,
            "internal_fe": self.internal_fe,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.routed = snapshot.get("routed").and_then(|v| v.as_u64()).unwrap_or(0);
        self.last_command = snapshot
            .get("last_command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        self.internal_fe = snapshot
            .get("internal_fe")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        Ok(())
    }

    pub fn parse_raw(raw: &str, last_command: &mut String) -> GarmCommand {
        let trimmed = raw.trim();
        *last_command = trimmed.to_string();
        match trimmed {
            "quit" | "exit" | "adios" | "bye" | "salir" | "hasta luego" => GarmCommand::Quit,
            "hola" | "hello" | "hi" | "hey" | "que tal" | "buenos" => GarmCommand::Greeting,
            "quien eres" | "who are you" | "tu identidad" | "tu mismo" | "yourself" => {
                GarmCommand::SelfQuery
            }
            "que piensas"
            | "what are you thinking"
            | "como piensas"
            | "explicale"
            | "tu mente"
            | "explain yourself"
            | "your mind" => GarmCommand::Thinking,
            "como te sientes" | "como estas" | "how are you" | "how do you feel"
            | "que emocion" | "que sientes" => GarmCommand::Feeling,
            "phi" | "conciencia" | "consciousness" | "consciencia" | "medir" | "medicion"
            | "grafo consciente" | "conscious graph" => GarmCommand::Phi,
            "observatorio" | "dashboard" | "metricas" | "sistemas" | "estado global"
            | "panorama" | "ver todo" => GarmCommand::Observatory,
            "historial" | "mi historial" | "ver historial" | "registro" | "log"
            | "eventos evolutivos" | "log de eventos" | "registro evolutivo" => {
                GarmCommand::History
            }
            "iniciar" | "despierta" | "empieza" | "vivir" | "start" | "awake" | "corre" | "run" => {
                GarmCommand::Start
            }
            "detener" | "para" | "duerme" | "stop" | "pausa" | "halt" | "quieto" => {
                GarmCommand::Stop
            }
            "evoluciona" | "mejorate" | "evolve" | "improve" | "grow" | "self-improve"
            | "mutate" => GarmCommand::Evolve,
            "rebirth" | "renacer" | "renacimiento" | "meltrace" => GarmCommand::Rebirth,
            "readiness bench" | "readiness benchmark" | "readiness gates bench" => {
                GarmCommand::ReadinessBench
            }
            "readiness probe" | "readiness generate" | "readiness evidence" => {
                GarmCommand::ReadinessProbe
            }
            "readiness external" | "readiness phase6" | "external validation" => {
                GarmCommand::ReadinessExternal
            }
            "readiness external run" | "external validation run" | "run external validation" => {
                GarmCommand::ReadinessExternalRun
            }
            "readiness package" | "garm package" | "validation package" => {
                GarmCommand::ReadinessPackage
            }
            "action evidence" | "actions evidence" | "evidence actions" => {
                GarmCommand::ActionEvidence
            }
            "capabilities audit" | "capability registry" | "capabilities registry" => {
                GarmCommand::CapabilityRegistry
            }
            "cognitive eval" | "cognitive architecture" | "arquitectura cognitiva" => {
                GarmCommand::CognitiveEval
            }
            "embodied eval" | "embodied grounding" | "embodiment eval" | "grounding embodied" => {
                GarmCommand::EmbodiedEval
            }
            "neural eval" | "neural architecture" | "arquitectura neural" => {
                GarmCommand::NeuralEval
            }
            "symbolic eval" | "symbolic architecture" | "arquitectura simbolica" => {
                GarmCommand::SymbolicEval
            }
            "self improvement eval"
            | "self-improvement eval"
            | "self improving eval"
            | "self improvement architecture"
            | "self-improvement architecture"
            | "mejora propia eval"
            | "arquitectura de mejora propia" => GarmCommand::SelfImprovementEval,
            "frontier architecture eval"
            | "frontier eval"
            | "safety control eval"
            | "foundation model eval"
            | "multimodal model eval"
            | "llm agent eval"
            | "probabilistic programming eval"
            | "hierarchical rl eval"
            | "cognitive robotics eval"
            | "vla eval"
            | "sim-to-real eval"
            | "open ended evolution eval"
            | "developmental robotics eval"
            | "whole brain eval"
            | "neuromorphic eval"
            | "spiking eval" => GarmCommand::FrontierArchitectureEval,
            "paradigm architecture eval"
            | "paradigm eval"
            | "paradigms eval"
            | "paradigmas eval"
            | "neuro symbolic eval"
            | "neuro-symbolic eval"
            | "universal formal eval"
            | "active inference eval"
            | "ecological systemic eval"
            | "computational programmatic eval"
            | "affective motivational eval"
            | "human in the loop eval"
            | "emergence metrics eval" => GarmCommand::ParadigmArchitectureEval,
            "integration governance eval"
            | "integration eval"
            | "executive integration eval"
            | "governance eval"
            | "executive governance eval"
            | "integracion ejecutiva eval"
            | "gobernanza eval" => GarmCommand::IntegrationGovernanceEval,
            "global executive workspace eval"
            | "global executive core eval"
            | "executive workspace eval"
            | "workspace core eval"
            | "hybrid executive core eval"
            | "hec eval"
            | "gewc eval"
            | "nucleo ejecutivo eval"
            | "nucleo ejecutivo global eval" => GarmCommand::GlobalExecutiveWorkspaceEval,
            "gewc operational benchmark"
            | "gewc operational eval"
            | "global executive workspace benchmark"
            | "operational benchmark"
            | "operational eval"
            | "benchmark operacional gewc"
            | "validacion operacional gewc" => GarmCommand::GewcOperationalBenchmark,
            "capability reality eval"
            | "capability reality"
            | "reality eval"
            | "current capability eval"
            | "current capabilities eval"
            | "evaluacion realidad capacidades"
            | "evaluacion capacidades actuales"
            | "capacidades reales"
            | "evaluar capacidades actuales" => GarmCommand::CapabilityRealityEval,
            "architecture advantage eval"
            | "architecture advantage"
            | "competitive architecture eval"
            | "architectural advantage eval"
            | "superar frameworks eval"
            | "ventaja arquitectura eval"
            | "evaluacion ventaja arquitectura" => GarmCommand::ArchitectureAdvantageEval,
            "paradise"
            | "paradise eval"
            | "paradise worldcell"
            | "paradise worldcell eval"
            | "worldcell"
            | "worldcell eval"
            | "worldcell runtime"
            | "worldcell runtime eval"
            | "runtime worldcell"
            | "paraiso worldcell"
            | "paraiso eval" => GarmCommand::ParadiseWorldcellEval,
            "paradise plan" | "worldcell plan" | "paraiso plan" => {
                GarmCommand::ParadisePlan("latest".to_string())
            }
            "paradise approve" | "worldcell approve" | "paraiso approve" => {
                GarmCommand::ParadiseApprove("latest".to_string())
            }
            "paradise execute" | "worldcell execute" | "paraiso execute" => {
                GarmCommand::ParadiseExecute("latest".to_string())
            }
            "paradise sessions" | "paradise audit" | "worldcell sessions" | "worldcell audit"
            | "paraiso sessions" => GarmCommand::ParadiseAudit,
            "praxis nexus eval"
            | "eden praxis nexus"
            | "praxis eval"
            | "praxis nexus"
            | "nexo praxis eval"
            | "nexo praxico eval"
            | "sustrato praxico eval" => GarmCommand::PraxisNexusEval,
            "locus" | "eden locus" | "locus eval" | "locus layer" | "eden locus layer"
            | "capa locus" | "capa locus eval" => GarmCommand::LocusLayerEval,
            "locus audit" | "eden locus audit" | "auditoria locus" => GarmCommand::LocusAudit,
            "operator forge"
            | "operator forge eval"
            | "eden operator forge"
            | "forge eval"
            | "formal synthesis eval"
            | "forja operativa eval" => GarmCommand::OperatorForgeEval,
            "operator forge verify" | "forge verify" | "forja verifica" => {
                GarmCommand::OperatorForgeVerify
            }
            "operator forge audit" | "forge audit" | "auditoria forja" => {
                GarmCommand::OperatorForgeAudit
            }
            "external ecosystem eval"
            | "external ecosystem"
            | "eden external ecosystem"
            | "ecosystem eval"
            | "ecosistema externo eval"
            | "ecosistema externo"
            | "superar ecosistema opencog eval"
            | "superar ecosistema hyperon eval" => GarmCommand::ExternalEcosystemEval,
            "sovereign cognition eval"
            | "eden sovereign cognition"
            | "eden sovereignty eval"
            | "sovereign eval"
            | "hyperon surpass eval"
            | "opencog surpass eval"
            | "superar opencog eval"
            | "superar hyperon eval"
            | "soberania cognitiva eval" => GarmCommand::SovereignCognitionEval,
            "artifact api eval"
            | "artifact api"
            | "artifacts api"
            | "artifact catalog eval"
            | "api artifacts eval"
            | "artefactos api eval"
            | "artefactos ejecutables eval" => GarmCommand::ArtifactApiEval,
            "training evidence eval"
            | "training evidence"
            | "training smoke evidence"
            | "training capability evidence"
            | "capability training evidence"
            | "evidencia entrenamiento eval" => GarmCommand::TrainingEvidenceEval,
            "megatron 7b evidence eval"
            | "megatron 7b evidence"
            | "training megatron 7b evidence"
            | "training 7b evidence eval"
            | "7b training evidence eval"
            | "7b evidence eval"
            | "evidencia 7b eval" => GarmCommand::Megatron7bEvidenceEval,
            "model runtime eval"
            | "training runtime eval"
            | "model runtime"
            | "model adapter runtime"
            | "runtime modelos eval"
            | "modelo runtime eval" => GarmCommand::ModelRuntimeEval,
            "model adapter runtime eval"
            | "model adapter eval"
            | "adapter runtime eval"
            | "adaptador modelo eval" => GarmCommand::ModelAdapterRuntimeEval,
            "model checkpoint manifest eval"
            | "model checkpoint eval"
            | "checkpoint manifest eval"
            | "manifest checkpoint eval"
            | "manifiesto checkpoint eval" => GarmCommand::ModelCheckpointManifestEval,
            "training harness eval"
            | "training harness"
            | "harness eval"
            | "arnes entrenamiento eval" => GarmCommand::TrainingHarnessEval,
            "model governance eval"
            | "gewc model governance eval"
            | "model governance"
            | "gobernanza modelo eval" => GarmCommand::ModelGovernanceEval,
            "first model prepare"
            | "eden first model prepare"
            | "first model 4a"
            | "prepare first model"
            | "primer modelo preparar"
            | "preparar primer modelo" => GarmCommand::FirstModelPrepare,
            "first model readiness"
            | "first model readiness eval"
            | "eden first model readiness"
            | "primer modelo readiness"
            | "readiness primer modelo" => GarmCommand::FirstModelReadinessEval,
            "elcp prepare"
            | "eden elcp prepare"
            | "elcp 4a"
            | "latent cognitive prediction prepare"
            | "eden latent cognitive prediction"
            | "eden latent cognitive prediction prepare"
            | "preparar elcp" => GarmCommand::ElcpPrepare,
            "elcp objective"
            | "elcp objective eval"
            | "elcp objective spec"
            | "elcp spec"
            | "latent cognitive prediction objective" => GarmCommand::ElcpObjectiveEval,
            "elcp admission"
            | "elcp admission gate"
            | "elcp admission gate eval"
            | "eden elcp admission"
            | "latent cognitive prediction admission" => GarmCommand::ElcpAdmissionGateEval,
            "elcp trace quality"
            | "elcp trace quality gate"
            | "elcp trace quality eval"
            | "latent cognitive prediction trace quality" => GarmCommand::ElcpTraceQualityEval,
            "elcp replay" | "elcp replay eval" | "latent cognitive prediction replay" => {
                GarmCommand::ElcpReplayEval
            }
            "elcp dataset freeze"
            | "elcp freeze"
            | "elcp dataset freeze manifest"
            | "latent cognitive prediction dataset freeze" => GarmCommand::ElcpDatasetFreezeEval,
            "elcp metrics"
            | "elcp metrics board"
            | "elcp metrics eval"
            | "latent cognitive prediction metrics" => GarmCommand::ElcpMetricsBoardEval,
            "elcp 4b readiness"
            | "elcp 4b contract"
            | "elcp readiness contract"
            | "latent cognitive prediction 4b contract" => GarmCommand::Elcp4bReadinessContractEval,
            "elcp hardening"
            | "elcp evidence hardening"
            | "elcp data hardening"
            | "latent cognitive prediction hardening" => GarmCommand::ElcpHardeningEval,
            "elcp readiness"
            | "elcp readiness eval"
            | "eden elcp readiness"
            | "latent cognitive prediction readiness" => GarmCommand::ElcpReadinessEval,
            "model audit" | "model runtime audit" | "auditar modelos" => GarmCommand::ModelAudit,
            "runtime state api eval"
            | "runtime api eval"
            | "state api eval"
            | "state management api eval"
            | "api runtime state eval"
            | "estado runtime api eval"
            | "apis estado runtime eval" => GarmCommand::RuntimeStateApiEval,
            "operational api eval"
            | "operational api"
            | "control api eval"
            | "capability api eval"
            | "capabilities api eval"
            | "gewc api eval"
            | "validation api eval"
            | "actions contract api eval"
            | "api operacional eval" => GarmCommand::OperationalApiEval,
            "operational runtime eval"
            | "operational runtime phase"
            | "runtime operativo eval"
            | "fase runtime operativo"
            | "runtime real eval"
            | "hacer runtime real" => GarmCommand::OperationalRuntimeEval,
            "operational task run" | "task run" | "run operational task" | "runtime task run" => {
                GarmCommand::OperationalTaskRun
            }
            "operational task audit"
            | "task audit"
            | "runtime task audit"
            | "auditoria tarea operacional" => GarmCommand::OperationalTaskAudit,
            "operational replay run" | "replay run" | "runtime replay" | "replay runtime" => {
                GarmCommand::OperationalReplayRun
            }
            "operational smoke run" | "runtime smoke" | "smoke test" | "smoke operacional" => {
                GarmCommand::OperationalSmokeRun
            }
            "operational scenario run"
            | "operational e2e scenario"
            | "runtime scenario"
            | "escenario operacional" => GarmCommand::OperationalScenarioRun,
            "operational permissions audit"
            | "permissions audit"
            | "runtime permissions audit"
            | "auditoria permisos operacional" => GarmCommand::OperationalPermissionsAudit,
            "operational permissions diff"
            | "permissions diff"
            | "runtime permissions diff"
            | "diff permisos operacional" => GarmCommand::OperationalPermissionsDiff,
            "operational permissions history"
            | "permissions history"
            | "runtime permissions history"
            | "historial permisos operacional" => GarmCommand::OperationalPermissionsHistory,
            "operational permissions restore"
            | "permissions restore"
            | "runtime permissions restore"
            | "restaurar permisos operacional" => GarmCommand::OperationalPermissionsRestore,
            "operational recovery audit"
            | "recovery audit"
            | "runtime recovery audit"
            | "auditoria recovery operacional" => GarmCommand::OperationalRecoveryAudit,
            "operational recovery run"
            | "recovery run"
            | "runtime recovery run"
            | "recuperacion operacional" => GarmCommand::OperationalRecoveryRun,
            "operational demo run"
            | "operational demos run"
            | "demo suite"
            | "demos operativos" => GarmCommand::OperationalDemoRun,
            "runtime spine eval"
            | "runtime spine"
            | "eden runtime spine"
            | "kernel spine eval"
            | "spine eval"
            | "contratos internos eval"
            | "columna runtime eval" => GarmCommand::RuntimeSpineEval,
            "runtime spine audit"
            | "spine audit"
            | "eden runtime spine audit"
            | "auditoria spine runtime" => GarmCommand::RuntimeSpineAudit,
            "runtime spine verify"
            | "spine verify"
            | "eden runtime spine verify"
            | "verificar spine runtime" => GarmCommand::RuntimeSpineVerify,
            "runtime spine enforce"
            | "spine enforce"
            | "eden runtime spine enforce"
            | "enforcement spine runtime" => GarmCommand::RuntimeSpineEnforce,
            "runtime spine risk" | "spine risk" | "workflow risk" | "riesgo workflow runtime" => {
                GarmCommand::RuntimeSpineRisk
            }
            "runtime spine breakers"
            | "spine breakers"
            | "runtime circuit breakers"
            | "circuit breakers" => GarmCommand::RuntimeSpineBreakers,
            "runtime spine replay"
            | "spine replay"
            | "runtime replay reconstruction"
            | "reconstruir replay runtime" => GarmCommand::RuntimeSpineReplay,
            "readiness plan" | "plan readiness" | "readiness gates plan" => {
                GarmCommand::ReadinessPlan
            }
            "readiness run" | "run readiness" | "readiness gates run" => GarmCommand::ReadinessRun,
            "readiness" | "readiness gates" | "architecture readiness" => GarmCommand::Readiness,
            "eval" | "evaluation" | "evaluacion" | "evaluacion arquitectura" => {
                GarmCommand::Evaluation
            }
            "eval run" | "evaluation run" | "evaluacion run" | "evaluar arquitectura" => {
                GarmCommand::EvaluationRun
            }
            "eval audit" | "evaluation audit" | "evaluacion audit" => GarmCommand::EvaluationAudit,
            "learning" | "learn ledger" | "aprendizajes" | "ledger aprendizaje" => {
                GarmCommand::Learning
            }
            "learning consolidate" | "aprendizajes consolidar" | "consolidar aprendizajes" => {
                GarmCommand::LearningConsolidate
            }
            "learning audit" | "aprendizajes audit" | "auditoria aprendizajes" => {
                GarmCommand::LearningAudit
            }
            "world" | "world model" | "modelo mundo" | "modelo del mundo" => {
                GarmCommand::WorldModel
            }
            "world verify" | "world model verify" | "verificar mundo" => GarmCommand::WorldVerify,
            "world audit" | "world model audit" | "auditoria mundo" => GarmCommand::WorldAudit,
            "world eval" | "world model eval" | "world evaluation" => GarmCommand::WorldEval,
            "bench" | "benchmark" | "competence" | "competence benchmark" => GarmCommand::Benchmark,
            "bench run" | "benchmark run" | "competence run" | "medir competencia" => {
                GarmCommand::BenchmarkRun
            }
            "bench audit" | "benchmark audit" | "competence audit" | "auditoria competencia" => {
                GarmCommand::BenchmarkAudit
            }
            "exec" | "executor" | "plan executor" | "ejecutor" => GarmCommand::PlanExecutor,
            "exec run" | "executor run" | "plan executor run" | "ejecutar plan" => {
                GarmCommand::PlanExecutorRun
            }
            "exec audit" | "executor audit" | "plan executor audit" | "auditoria ejecucion" => {
                GarmCommand::PlanExecutorAudit
            }
            "attention" | "attn" | "working memory" | "memoria trabajo" | "foco" => {
                GarmCommand::Attention
            }
            "attention clear" | "attn clear" | "limpiar foco" | "foco clear" => {
                GarmCommand::AttentionClear
            }
            "attention audit" | "attn audit" | "auditoria foco" => GarmCommand::AttentionAudit,
            "uncertainty" | "uncertainties" | "riesgos" | "risk ledger" => GarmCommand::Uncertainty,
            "uncertainty resolve" | "risk resolve" | "resolver incertidumbre" => {
                GarmCommand::UncertaintyResolve
            }
            "uncertainty audit" | "risk audit" | "auditoria riesgos" => {
                GarmCommand::UncertaintyAudit
            }
            "experiment" | "experiments" | "experimento" | "experimentos" => {
                GarmCommand::Experiment
            }
            "experiment run" | "experimento run" | "run experiment" => GarmCommand::ExperimentRun,
            "experiment audit" | "experimento audit" | "auditoria experimentos" => {
                GarmCommand::ExperimentAudit
            }
            "provenance" | "evidence ledger" | "procedencia" => GarmCommand::Provenance,
            "provenance verify" | "evidence verify" | "verificar evidencia" => {
                GarmCommand::ProvenanceVerify
            }
            "provenance audit" | "evidence audit" | "auditoria evidencia" => {
                GarmCommand::ProvenanceAudit
            }
            "policy" | "policy guard" | "guard" | "politica" => GarmCommand::Policy,
            "policy audit" | "guard audit" | "auditoria politica" => GarmCommand::PolicyAudit,
            "maturity" | "capability maturity" | "madurez" => GarmCommand::Maturity,
            "maturity audit" | "madurez audit" | "auditoria madurez" => GarmCommand::MaturityAudit,
            "ritual" | "organismo" | "umbra" | "child-autons" | "vida interna" => {
                GarmCommand::OrganicRitual
            }
            "voz" | "habla" | "autodocumenta" => GarmCommand::Voz,
            "hybrid voice" | "voz hibrida" | "hybrid tts" => GarmCommand::HybridVoice,
            "hybrid voice audit" | "voz hibrida audit" => GarmCommand::HybridVoiceAudit,
            "hrm text" | "hrm-text" | "hrm pretraining" | "pretraining hrm" => GarmCommand::HrmText,
            "hrm text plan" | "hrm-text plan" | "hrm pretraining plan" => GarmCommand::HrmTextPlan,
            "hrm text run" | "hrm-text run" | "hrm pretraining run" => GarmCommand::HrmTextRun,
            "hrm text eval" | "hrm-text eval" | "rag eval" | "rag benchmark" => {
                GarmCommand::HrmTextEval
            }
            "hrm text audit" | "hrm-text audit" | "hrm pretraining audit" => {
                GarmCommand::HrmTextAudit
            }
            "intestino" | "compacta" | "compactar" => GarmCommand::Intestino,
            "piel" | "frontera" | "sensores" => GarmCommand::Piel,
            "autotuning" | "autoajuste" | "ajusta organos" => GarmCommand::Autotuning,
            "cag" | "contexto" | "cache" | "context cache" | "cache contexto" => GarmCommand::Cag,
            "cag actions" | "contexto acciones" | "cache actions" => GarmCommand::CagActions,
            "cag audit" | "contexto audit" | "cache audit" | "auditoria cag" => {
                GarmCommand::CagAudit
            }
            "estado avanzado" | "garm audit" | "audit garm" => GarmCommand::GarmAudit,
            "garm report" | "report garm" | "reporte garm" => GarmCommand::GarmReport,
            "garm report history" | "report history" | "historial reportes" => {
                GarmCommand::GarmReportHistory
            }
            "garm export" | "export garm" | "exportar garm" => GarmCommand::GarmExport,
            "garm import" | "import garm" | "importar garm" => GarmCommand::GarmImport,
            "garm verify export" | "verify export" | "verificar export" => {
                GarmCommand::GarmVerifyExport
            }
            "garm artifacts" | "artifacts" | "artefactos garm" => GarmCommand::GarmArtifacts,
            "goals" | "objetivos" | "goal scheduler" => GarmCommand::Goals,
            "goals run" | "objetivos run" | "objetivos ejecutar" => GarmCommand::GoalsRun,
            "goals audit" | "objetivos audit" | "auditoria objetivos" => GarmCommand::GoalsAudit,
            "garm backup" | "backup garm" => GarmCommand::GarmBackup,
            "garm restore" | "restore garm" => GarmCommand::GarmRestore,
            "garm compact" | "compact garm" => GarmCommand::GarmCompact,
            "organos" | "órganos" | "organs" | "organ registry" => GarmCommand::Organs,
            "organos audit" | "órganos audit" | "organs audit" | "auditoria organos" => {
                GarmCommand::OrgansAudit
            }
            "organos plan" | "órganos plan" | "organs plan" => GarmCommand::OrgansPlan,
            "organos run" | "órganos run" | "organs run" => GarmCommand::OrgansRun,
            "organos health" | "órganos health" | "organos salud" | "organs health" => {
                GarmCommand::OrgansHealth
            }
            "organos repair" | "órganos repair" | "organos reparar" | "organs repair" => {
                GarmCommand::OrgansRepair
            }
            "organos actions" | "órganos actions" | "organos acciones" | "organs actions" => {
                GarmCommand::OrgansActions
            }
            "organos feedback good" | "organos feedback positivo" | "organs feedback good" => {
                GarmCommand::OrgansFeedback(true)
            }
            "organos feedback bad" | "organos feedback negativo" | "organs feedback bad" => {
                GarmCommand::OrgansFeedback(false)
            }
            "tick" => GarmCommand::Tick,
            "estas" | "estado" | "status" | "que pasa" | "que haces" => GarmCommand::Status,
            "save" | "guarda" | "guardar" | "persiste" => GarmCommand::Save,
            "load" | "carga" | "recupera" | "restore" => GarmCommand::Load,
            "help" | "ayuda" | "commands" | "comandos" | "que puedes" => GarmCommand::Help,
            "migration" | "migracion" | "legacy" => GarmCommand::Migration,
            "memoria"
            | "memory"
            | "que sabes"
            | "que recuerdas"
            | "what do you know"
            | "what do you remember" => GarmCommand::Memory,
            "memory eval" | "memory evaluation" | "memoria eval" => GarmCommand::MemoryEval,
            _ => {
                if let Some(topic) = trimmed
                    .strip_prefix("paradise intent ")
                    .or_else(|| trimmed.strip_prefix("worldcell intent "))
                    .or_else(|| trimmed.strip_prefix("paraiso intent "))
                    .or_else(|| trimmed.strip_prefix("paradise ask "))
                    .or_else(|| trimmed.strip_prefix("paradise task "))
                {
                    return GarmCommand::ParadiseIntent(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("paradise plan ")
                    .or_else(|| trimmed.strip_prefix("worldcell plan "))
                    .or_else(|| trimmed.strip_prefix("paraiso plan "))
                {
                    return GarmCommand::ParadisePlan(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("paradise approve ")
                    .or_else(|| trimmed.strip_prefix("worldcell approve "))
                    .or_else(|| trimmed.strip_prefix("paraiso approve "))
                {
                    return GarmCommand::ParadiseApprove(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("paradise execute ")
                    .or_else(|| trimmed.strip_prefix("worldcell execute "))
                    .or_else(|| trimmed.strip_prefix("paraiso execute "))
                {
                    return GarmCommand::ParadiseExecute(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("locus ingest ")
                    .or_else(|| trimmed.strip_prefix("eden locus ingest "))
                    .or_else(|| trimmed.strip_prefix("locus record "))
                    .or_else(|| trimmed.strip_prefix("locus registra "))
                {
                    return GarmCommand::LocusIngest(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("locus context ")
                    .or_else(|| trimmed.strip_prefix("eden locus context "))
                    .or_else(|| trimmed.strip_prefix("locus packet "))
                    .or_else(|| trimmed.strip_prefix("contexto locus "))
                {
                    return GarmCommand::LocusContext(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("operator forge synth ")
                    .or_else(|| trimmed.strip_prefix("operator forge synthesize "))
                    .or_else(|| trimmed.strip_prefix("forge synth "))
                    .or_else(|| trimmed.strip_prefix("forja sintetiza "))
                    .or_else(|| trimmed.strip_prefix("formal synth "))
                {
                    return GarmCommand::OperatorForgeSynthesize(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("operational task submit ")
                    .or_else(|| trimmed.strip_prefix("task submit "))
                    .or_else(|| trimmed.strip_prefix("runtime task submit "))
                    .or_else(|| trimmed.strip_prefix("submit task "))
                    .or_else(|| trimmed.strip_prefix("tarea operacional "))
                {
                    return GarmCommand::OperationalTaskSubmit(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("operational action execute ")
                    .or_else(|| trimmed.strip_prefix("action execute "))
                    .or_else(|| trimmed.strip_prefix("runtime action execute "))
                    .or_else(|| trimmed.strip_prefix("execute action "))
                    .or_else(|| trimmed.strip_prefix("accion operacional "))
                {
                    return GarmCommand::OperationalActionExecute(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("operational memory commit ")
                    .or_else(|| trimmed.strip_prefix("memory commit "))
                    .or_else(|| trimmed.strip_prefix("memory tx "))
                    .or_else(|| trimmed.strip_prefix("runtime memory commit "))
                    .or_else(|| trimmed.strip_prefix("memoria commit "))
                {
                    return GarmCommand::OperationalMemoryCommit(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("operational memory rollback ")
                    .or_else(|| trimmed.strip_prefix("memory rollback "))
                    .or_else(|| trimmed.strip_prefix("rollback memory "))
                    .or_else(|| trimmed.strip_prefix("runtime memory rollback "))
                    .or_else(|| trimmed.strip_prefix("memoria rollback "))
                {
                    return GarmCommand::OperationalMemoryRollback(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("operational permissions set ")
                    .or_else(|| trimmed.strip_prefix("permissions set "))
                    .or_else(|| trimmed.strip_prefix("runtime permissions set "))
                    .or_else(|| trimmed.strip_prefix("set permission "))
                    .or_else(|| trimmed.strip_prefix("permiso operacional "))
                {
                    return GarmCommand::OperationalPermissionsSet(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("gewc lifecycle ")
                    .or_else(|| trimmed.strip_prefix("lifecycle "))
                    .or_else(|| trimmed.strip_prefix("module lifecycle "))
                    .or_else(|| trimmed.strip_prefix("control lifecycle "))
                    .or_else(|| trimmed.strip_prefix("ciclo modulo "))
                {
                    return GarmCommand::GewcLifecycleControl(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("que sabes de ")
                    .or_else(|| trimmed.strip_prefix("what do you know about "))
                    .or_else(|| trimmed.strip_prefix("busca "))
                    .or_else(|| trimmed.strip_prefix("search "))
                    .or_else(|| trimmed.strip_prefix("memoria "))
                {
                    return GarmCommand::Query(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("que es ")
                    .or_else(|| trimmed.strip_prefix("what is "))
                    .or_else(|| trimmed.strip_prefix("definicion de "))
                    .or_else(|| trimmed.strip_prefix("definition of "))
                    .or_else(|| trimmed.strip_prefix("explicame "))
                    .or_else(|| trimmed.strip_prefix("explain "))
                {
                    return GarmCommand::WhatIs(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("por que ")
                    .or_else(|| trimmed.strip_prefix("why "))
                    .or_else(|| trimmed.strip_prefix("causa de "))
                    .or_else(|| trimmed.strip_prefix("motivo de "))
                    .or_else(|| trimmed.strip_prefix("cual es la razon "))
                {
                    return GarmCommand::Why(topic.trim().to_string());
                }
                if let Some(topic) = trimmed
                    .strip_prefix("cuentame ")
                    .or_else(|| trimmed.strip_prefix("hablame de "))
                    .or_else(|| trimmed.strip_prefix("dime sobre "))
                    .or_else(|| trimmed.strip_prefix("tell me about "))
                    .or_else(|| trimmed.strip_prefix("que opinas de "))
                    .or_else(|| trimmed.strip_prefix("que piensas de "))
                {
                    return GarmCommand::TellMe(topic.trim().to_string());
                }
                if trimmed.contains("quien eres") || trimmed.contains("who are you") {
                    return GarmCommand::SelfQuery;
                }
                if trimmed.contains("que piensas")
                    || trimmed.contains("what are you thinking")
                    || trimmed.contains("tu mente")
                {
                    return GarmCommand::Thinking;
                }
                if trimmed.contains("como te sientes") || trimmed.contains("how do you feel") {
                    return GarmCommand::Feeling;
                }
                if trimmed.contains("observatorio")
                    || trimmed.contains("dashboard")
                    || trimmed.contains("estado global")
                    || trimmed.contains("ver todo")
                {
                    return GarmCommand::Observatory;
                }
                if trimmed.contains("historial")
                    || trimmed.contains("registro evolutivo")
                    || trimmed.contains("eventos evolutivos")
                {
                    return GarmCommand::History;
                }
                if trimmed.contains("despierta") || trimmed.contains("empieza") {
                    return GarmCommand::Start;
                }
                if trimmed.contains("duerme") || trimmed.contains("pausa") {
                    return GarmCommand::Stop;
                }
                if trimmed.contains("evoluciona")
                    || trimmed.contains("mejorate")
                    || trimmed.contains("self-improve")
                {
                    return GarmCommand::Evolve;
                }
                if let Some(fact) = trimmed
                    .strip_prefix("recuerda ")
                    .or_else(|| trimmed.strip_prefix("remember "))
                    .or_else(|| trimmed.strip_prefix("aprende "))
                    .or_else(|| trimmed.strip_prefix("aprendizaje "))
                    .or_else(|| trimmed.strip_prefix("recorder "))
                    .or_else(|| trimmed.strip_prefix("learn "))
                {
                    return GarmCommand::Remember(fact.trim().to_string());
                }
                if let Some(url) = trimmed
                    .strip_prefix("crawl ")
                    .or_else(|| trimmed.strip_prefix("web "))
                    .or_else(|| trimmed.strip_prefix("crawler "))
                    .or_else(|| trimmed.strip_prefix("buscar web "))
                {
                    return GarmCommand::Crawl(url.trim().to_string());
                }
                if let Some(path) = trimmed
                    .strip_prefix("conceptnet ")
                    .or_else(|| trimmed.strip_prefix("load conceptnet "))
                    .or_else(|| trimmed.strip_prefix("cargar conceptnet "))
                {
                    return GarmCommand::ConceptNet(path.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("world observe ")
                    .or_else(|| trimmed.strip_prefix("mundo observa "))
                    .or_else(|| trimmed.strip_prefix("observar mundo "))
                {
                    return GarmCommand::WorldObserve(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("world predict ")
                    .or_else(|| trimmed.strip_prefix("mundo predice "))
                    .or_else(|| trimmed.strip_prefix("predecir mundo "))
                {
                    return GarmCommand::WorldPredict(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("learning record ")
                    .or_else(|| trimmed.strip_prefix("aprendizaje record "))
                    .or_else(|| trimmed.strip_prefix("registrar aprendizaje "))
                {
                    return GarmCommand::LearningRecord(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("goals plan ")
                    .or_else(|| trimmed.strip_prefix("objetivos plan "))
                    .or_else(|| trimmed.strip_prefix("plan objetivo "))
                {
                    return GarmCommand::GoalsPlan(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("maturity assess ")
                    .or_else(|| trimmed.strip_prefix("maturity "))
                    .or_else(|| trimmed.strip_prefix("madurez "))
                {
                    return GarmCommand::MaturityAssess(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("policy eval ")
                    .or_else(|| trimmed.strip_prefix("policy "))
                    .or_else(|| trimmed.strip_prefix("guard "))
                    .or_else(|| trimmed.strip_prefix("politica "))
                {
                    return GarmCommand::PolicyEval(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("provenance record ")
                    .or_else(|| trimmed.strip_prefix("provenance "))
                    .or_else(|| trimmed.strip_prefix("evidence record "))
                    .or_else(|| trimmed.strip_prefix("procedencia "))
                {
                    return GarmCommand::ProvenanceRecord(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("experiment plan ")
                    .or_else(|| trimmed.strip_prefix("experiment "))
                    .or_else(|| trimmed.strip_prefix("experimento "))
                {
                    return GarmCommand::ExperimentPlan(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("uncertainty record ")
                    .or_else(|| trimmed.strip_prefix("uncertainty "))
                    .or_else(|| trimmed.strip_prefix("risk record "))
                    .or_else(|| trimmed.strip_prefix("riesgo "))
                {
                    return GarmCommand::UncertaintyRecord(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("attention ")
                    .or_else(|| trimmed.strip_prefix("attn "))
                    .or_else(|| trimmed.strip_prefix("focus "))
                    .or_else(|| trimmed.strip_prefix("foco "))
                {
                    return GarmCommand::AttentionAttend(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("exec plan ")
                    .or_else(|| trimmed.strip_prefix("executor plan "))
                    .or_else(|| trimmed.strip_prefix("plan executor "))
                    .or_else(|| trimmed.strip_prefix("plan ejecutar "))
                {
                    return GarmCommand::PlanExecutorPlan(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("cag explain ")
                    .or_else(|| trimmed.strip_prefix("contexto explain "))
                    .or_else(|| trimmed.strip_prefix("cache explain "))
                    .or_else(|| trimmed.strip_prefix("explica contexto "))
                {
                    return GarmCommand::CagExplain(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("cag gaps ")
                    .or_else(|| trimmed.strip_prefix("contexto gaps "))
                    .or_else(|| trimmed.strip_prefix("cache gaps "))
                    .or_else(|| trimmed.strip_prefix("brechas contexto "))
                {
                    return GarmCommand::CagGaps(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("cag plan ")
                    .or_else(|| trimmed.strip_prefix("contexto plan "))
                    .or_else(|| trimmed.strip_prefix("cache plan "))
                {
                    return GarmCommand::CagPlan(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("cag run ")
                    .or_else(|| trimmed.strip_prefix("contexto run "))
                    .or_else(|| trimmed.strip_prefix("cache run "))
                {
                    return GarmCommand::CagRun(query.trim().to_string());
                }
                if let Some(path) = trimmed
                    .strip_prefix("hrm text ingest ")
                    .or_else(|| trimmed.strip_prefix("hrm-text ingest "))
                    .or_else(|| trimmed.strip_prefix("hrm pretraining ingest "))
                {
                    return GarmCommand::HrmTextIngest(path.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("hrm text search ")
                    .or_else(|| trimmed.strip_prefix("hrm-text search "))
                    .or_else(|| trimmed.strip_prefix("hrm pretraining search "))
                {
                    return GarmCommand::HrmTextSearch(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("hrm text context ")
                    .or_else(|| trimmed.strip_prefix("hrm-text context "))
                    .or_else(|| trimmed.strip_prefix("rag context "))
                    .or_else(|| trimmed.strip_prefix("rag answer "))
                {
                    return GarmCommand::HrmTextContext(query.trim().to_string());
                }
                if let Some(path) = trimmed
                    .strip_prefix("hrm text corpus ")
                    .or_else(|| trimmed.strip_prefix("hrm-text corpus "))
                    .or_else(|| trimmed.strip_prefix("hrm pretraining corpus "))
                {
                    return GarmCommand::HrmTextCorpus(path.trim().to_string());
                }
                if let Some(objective) = trimmed
                    .strip_prefix("hrm text objective ")
                    .or_else(|| trimmed.strip_prefix("hrm-text objective "))
                    .or_else(|| trimmed.strip_prefix("hrm pretraining objective "))
                {
                    return GarmCommand::HrmTextObjective(objective.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("model register ")
                    .or_else(|| trimmed.strip_prefix("register model "))
                    .or_else(|| trimmed.strip_prefix("registrar modelo "))
                {
                    return GarmCommand::ModelRegister(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("model load ")
                    .or_else(|| trimmed.strip_prefix("load model "))
                    .or_else(|| trimmed.strip_prefix("cargar modelo "))
                {
                    return GarmCommand::ModelLoad(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("model evaluate ")
                    .or_else(|| trimmed.strip_prefix("model eval "))
                    .or_else(|| trimmed.strip_prefix("evaluate model "))
                    .or_else(|| trimmed.strip_prefix("evaluar modelo "))
                {
                    return GarmCommand::ModelEvaluate(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("model unload ")
                    .or_else(|| trimmed.strip_prefix("unload model "))
                    .or_else(|| trimmed.strip_prefix("descargar modelo "))
                {
                    return GarmCommand::ModelUnload(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("hrm run ")
                    .or_else(|| trimmed.strip_prefix("ejecuta hrm "))
                {
                    return GarmCommand::HrmRun(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("hrm ")
                    .or_else(|| trimmed.strip_prefix("razona jerarquico "))
                    .or_else(|| trimmed.strip_prefix("hierarchical reason "))
                {
                    return GarmCommand::Hrm(query.trim().to_string());
                }
                if let Some(text) = trimmed
                    .strip_prefix("hybrid voice synth ")
                    .or_else(|| trimmed.strip_prefix("hybrid tts "))
                    .or_else(|| trimmed.strip_prefix("voz hibrida sintetiza "))
                {
                    return GarmCommand::HybridVoiceSynth(text.trim().to_string());
                }
                if let Some(text) = trimmed
                    .strip_prefix("hybrid voice plan ")
                    .or_else(|| trimmed.strip_prefix("voz hibrida plan "))
                {
                    return GarmCommand::HybridVoicePlan(text.trim().to_string());
                }
                if let Some(text) = trimmed
                    .strip_prefix("voz texto ")
                    .or_else(|| trimmed.strip_prefix("tts "))
                    .or_else(|| trimmed.strip_prefix("habla texto "))
                {
                    return GarmCommand::VozTexto(text.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("lengua ")
                    .or_else(|| trimmed.strip_prefix("responde "))
                    .or_else(|| trimmed.strip_prefix("answer "))
                {
                    return GarmCommand::Lengua(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("reloj ")
                    .or_else(|| trimmed.strip_prefix("timeline "))
                    .or_else(|| trimmed.strip_prefix("cuando "))
                    .or_else(|| trimmed.strip_prefix("cadena temporal "))
                {
                    return GarmCommand::Reloj(query.trim().to_string());
                }
                if let Some(query) = trimmed
                    .strip_prefix("juez ")
                    .or_else(|| trimmed.strip_prefix("validar "))
                    .or_else(|| trimmed.strip_prefix("valida "))
                    .or_else(|| trimmed.strip_prefix("judge "))
                {
                    return GarmCommand::Juez(query.trim().to_string());
                }
                if let Some(n_str) = trimmed
                    .strip_prefix("auto ")
                    .and_then(|s| s.split_whitespace().next())
                {
                    if let Ok(n) = n_str.parse::<usize>() {
                        return GarmCommand::Auto(n);
                    }
                }
                GarmCommand::Unknown(trimmed.to_string())
            }
        }
    }
}

impl GARMNode for CommandRouterNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "command_router"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Fast
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.routed as f32, self.internal_fe]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.routed as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
        0.1
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

#[cfg(test)]
mod tests {
    use super::{CommandRouterNode, GarmCommand};

    fn parse(raw: &str) -> GarmCommand {
        let mut last = String::new();
        CommandRouterNode::parse_raw(raw, &mut last)
    }

    #[test]
    fn parses_legacy_memory_aliases() {
        assert_eq!(
            parse("aprende cielo azul"),
            GarmCommand::Remember("cielo azul".to_string())
        );
        assert_eq!(
            parse("aprendizaje cielo azul"),
            GarmCommand::Remember("cielo azul".to_string())
        );
        assert_eq!(
            parse("recorder cielo azul"),
            GarmCommand::Remember("cielo azul".to_string())
        );
        assert_eq!(
            parse("learn rust"),
            GarmCommand::Remember("rust".to_string())
        );
        assert_eq!(parse("que sabes"), GarmCommand::Memory);
        assert_eq!(parse("what do you know"), GarmCommand::Memory);
        assert_eq!(parse("memory eval"), GarmCommand::MemoryEval);
        assert_eq!(parse("cognitive eval"), GarmCommand::CognitiveEval);
        assert_eq!(parse("embodied eval"), GarmCommand::EmbodiedEval);
        assert_eq!(parse("neural eval"), GarmCommand::NeuralEval);
        assert_eq!(parse("symbolic eval"), GarmCommand::SymbolicEval);
        assert_eq!(
            parse("self improvement eval"),
            GarmCommand::SelfImprovementEval
        );
        assert_eq!(
            parse("frontier architecture eval"),
            GarmCommand::FrontierArchitectureEval
        );
        assert_eq!(parse("vla eval"), GarmCommand::FrontierArchitectureEval);
        assert_eq!(
            parse("paradigm architecture eval"),
            GarmCommand::ParadigmArchitectureEval
        );
        assert_eq!(
            parse("active inference eval"),
            GarmCommand::ParadigmArchitectureEval
        );
        assert_eq!(
            parse("integration governance eval"),
            GarmCommand::IntegrationGovernanceEval
        );
        assert_eq!(
            parse("global executive workspace eval"),
            GarmCommand::GlobalExecutiveWorkspaceEval
        );
        assert_eq!(
            parse("gewc operational benchmark"),
            GarmCommand::GewcOperationalBenchmark
        );
        assert_eq!(
            parse("capability reality eval"),
            GarmCommand::CapabilityRealityEval
        );
        assert_eq!(
            parse("architecture advantage eval"),
            GarmCommand::ArchitectureAdvantageEval
        );
        assert_eq!(
            parse("paradise worldcell eval"),
            GarmCommand::ParadiseWorldcellEval
        );
        assert_eq!(
            parse("paradise intent inspect runtime status"),
            GarmCommand::ParadiseIntent("inspect runtime status".to_string())
        );
        assert_eq!(
            parse("paradise plan"),
            GarmCommand::ParadisePlan("latest".to_string())
        );
        assert_eq!(
            parse("paradise approve"),
            GarmCommand::ParadiseApprove("latest".to_string())
        );
        assert_eq!(
            parse("paradise execute"),
            GarmCommand::ParadiseExecute("latest".to_string())
        );
        assert_eq!(parse("paradise sessions"), GarmCommand::ParadiseAudit);
        assert_eq!(parse("praxis nexus eval"), GarmCommand::PraxisNexusEval);
        assert_eq!(parse("locus eval"), GarmCommand::LocusLayerEval);
        assert_eq!(
            parse("locus ingest user notes :: durable context"),
            GarmCommand::LocusIngest("user notes :: durable context".to_string())
        );
        assert_eq!(
            parse("locus context durable context"),
            GarmCommand::LocusContext("durable context".to_string())
        );
        assert_eq!(parse("locus audit"), GarmCommand::LocusAudit);
        assert_eq!(parse("operator forge eval"), GarmCommand::OperatorForgeEval);
        assert_eq!(
            parse("operator forge synth causal model"),
            GarmCommand::OperatorForgeSynthesize("causal model".to_string())
        );
        assert_eq!(
            parse("operator forge verify"),
            GarmCommand::OperatorForgeVerify
        );
        assert_eq!(
            parse("operator forge audit"),
            GarmCommand::OperatorForgeAudit
        );
        assert_eq!(
            parse("external ecosystem eval"),
            GarmCommand::ExternalEcosystemEval
        );
        assert_eq!(
            parse("sovereign cognition eval"),
            GarmCommand::SovereignCognitionEval
        );
        assert_eq!(parse("artifact api eval"), GarmCommand::ArtifactApiEval);
        assert_eq!(
            parse("training evidence eval"),
            GarmCommand::TrainingEvidenceEval
        );
        assert_eq!(
            parse("megatron 7b evidence eval"),
            GarmCommand::Megatron7bEvidenceEval
        );
        assert_eq!(parse("model runtime eval"), GarmCommand::ModelRuntimeEval);
        assert_eq!(
            parse("model adapter runtime eval"),
            GarmCommand::ModelAdapterRuntimeEval
        );
        assert_eq!(
            parse("model checkpoint manifest eval"),
            GarmCommand::ModelCheckpointManifestEval
        );
        assert_eq!(
            parse("training harness eval"),
            GarmCommand::TrainingHarnessEval
        );
        assert_eq!(
            parse("model governance eval"),
            GarmCommand::ModelGovernanceEval
        );
        assert_eq!(parse("first model prepare"), GarmCommand::FirstModelPrepare);
        assert_eq!(
            parse("first model readiness"),
            GarmCommand::FirstModelReadinessEval
        );
        assert_eq!(parse("elcp prepare"), GarmCommand::ElcpPrepare);
        assert_eq!(parse("elcp objective eval"), GarmCommand::ElcpObjectiveEval);
        assert_eq!(
            parse("elcp admission gate"),
            GarmCommand::ElcpAdmissionGateEval
        );
        assert_eq!(parse("elcp hardening"), GarmCommand::ElcpHardeningEval);
        assert_eq!(
            parse("elcp trace quality gate"),
            GarmCommand::ElcpTraceQualityEval
        );
        assert_eq!(parse("elcp replay eval"), GarmCommand::ElcpReplayEval);
        assert_eq!(
            parse("elcp dataset freeze"),
            GarmCommand::ElcpDatasetFreezeEval
        );
        assert_eq!(
            parse("elcp metrics board"),
            GarmCommand::ElcpMetricsBoardEval
        );
        assert_eq!(
            parse("elcp 4b contract"),
            GarmCommand::Elcp4bReadinessContractEval
        );
        assert_eq!(parse("elcp readiness"), GarmCommand::ElcpReadinessEval);
        assert_eq!(
            parse("model register candidate-a"),
            GarmCommand::ModelRegister("candidate-a".to_string())
        );
        assert_eq!(
            parse("model load candidate-a"),
            GarmCommand::ModelLoad("candidate-a".to_string())
        );
        assert_eq!(
            parse("model evaluate candidate-a"),
            GarmCommand::ModelEvaluate("candidate-a".to_string())
        );
        assert_eq!(
            parse("model unload candidate-a"),
            GarmCommand::ModelUnload("candidate-a".to_string())
        );
        assert_eq!(parse("model audit"), GarmCommand::ModelAudit);
        assert_eq!(
            parse("runtime state api eval"),
            GarmCommand::RuntimeStateApiEval
        );
        assert_eq!(
            parse("operational api eval"),
            GarmCommand::OperationalApiEval
        );
        assert_eq!(
            parse("operational runtime eval"),
            GarmCommand::OperationalRuntimeEval
        );
        assert_eq!(
            parse("operational task submit validate state"),
            GarmCommand::OperationalTaskSubmit("validate state".to_string())
        );
        assert_eq!(parse("task run"), GarmCommand::OperationalTaskRun);
        assert_eq!(parse("task audit"), GarmCommand::OperationalTaskAudit);
        assert_eq!(
            parse("operational action execute status"),
            GarmCommand::OperationalActionExecute("status".to_string())
        );
        assert_eq!(
            parse("memory commit durable fact"),
            GarmCommand::OperationalMemoryCommit("durable fact".to_string())
        );
        assert_eq!(
            parse("memory rollback memtx-1"),
            GarmCommand::OperationalMemoryRollback("memtx-1".to_string())
        );
        assert_eq!(parse("replay run"), GarmCommand::OperationalReplayRun);
        assert_eq!(
            parse("operational smoke run"),
            GarmCommand::OperationalSmokeRun
        );
        assert_eq!(
            parse("operational scenario run"),
            GarmCommand::OperationalScenarioRun
        );
        assert_eq!(
            parse("operational permissions audit"),
            GarmCommand::OperationalPermissionsAudit
        );
        assert_eq!(
            parse("operational permissions diff"),
            GarmCommand::OperationalPermissionsDiff
        );
        assert_eq!(
            parse("operational permissions history"),
            GarmCommand::OperationalPermissionsHistory
        );
        assert_eq!(
            parse("operational permissions restore"),
            GarmCommand::OperationalPermissionsRestore
        );
        assert_eq!(
            parse("operational permissions set remote_network deny"),
            GarmCommand::OperationalPermissionsSet("remote_network deny".to_string())
        );
        assert_eq!(
            parse("operational recovery audit"),
            GarmCommand::OperationalRecoveryAudit
        );
        assert_eq!(
            parse("operational recovery run"),
            GarmCommand::OperationalRecoveryRun
        );
        assert_eq!(
            parse("operational demo run"),
            GarmCommand::OperationalDemoRun
        );
        assert_eq!(parse("runtime spine eval"), GarmCommand::RuntimeSpineEval);
        assert_eq!(parse("runtime spine audit"), GarmCommand::RuntimeSpineAudit);
        assert_eq!(
            parse("runtime spine verify"),
            GarmCommand::RuntimeSpineVerify
        );
        assert_eq!(
            parse("runtime spine enforce"),
            GarmCommand::RuntimeSpineEnforce
        );
        assert_eq!(parse("runtime spine risk"), GarmCommand::RuntimeSpineRisk);
        assert_eq!(
            parse("runtime spine breakers"),
            GarmCommand::RuntimeSpineBreakers
        );
        assert_eq!(
            parse("runtime spine replay"),
            GarmCommand::RuntimeSpineReplay
        );
        assert_eq!(
            parse("gewc lifecycle world_model pause"),
            GarmCommand::GewcLifecycleControl("world_model pause".to_string())
        );
        assert_eq!(
            parse("busca cielo"),
            GarmCommand::Query("cielo".to_string())
        );
        assert_eq!(parse("search rust"), GarmCommand::Query("rust".to_string()));
        assert_eq!(
            parse("memoria rust"),
            GarmCommand::Query("rust".to_string())
        );
    }

    #[test]
    fn parses_migration_command() {
        assert_eq!(parse("migration"), GarmCommand::Migration);
        assert_eq!(parse("migracion"), GarmCommand::Migration);
        assert_eq!(parse("legacy"), GarmCommand::Migration);
    }

    #[test]
    fn parses_legacy_dialogue_intents() {
        assert_eq!(parse("hola"), GarmCommand::Greeting);
        assert_eq!(parse("que tal"), GarmCommand::Greeting);
        assert_eq!(parse("bye"), GarmCommand::Quit);
        assert_eq!(parse("quien eres"), GarmCommand::SelfQuery);
        assert_eq!(parse("yourself"), GarmCommand::SelfQuery);
        assert_eq!(parse("que piensas"), GarmCommand::Thinking);
        assert_eq!(parse("explicale"), GarmCommand::Thinking);
        assert_eq!(parse("explain yourself"), GarmCommand::Thinking);
        assert_eq!(parse("como te sientes"), GarmCommand::Feeling);
        assert_eq!(parse("como estas"), GarmCommand::Feeling);
        assert_eq!(parse("phi"), GarmCommand::Phi);
        assert_eq!(parse("medir"), GarmCommand::Phi);
        assert_eq!(parse("medicion"), GarmCommand::Phi);
        assert_eq!(parse("que puedes"), GarmCommand::Help);
    }

    #[test]
    fn parses_legacy_persistence_and_status_aliases() {
        assert_eq!(parse("estas"), GarmCommand::Status);
        assert_eq!(parse("que haces"), GarmCommand::Status);
        assert_eq!(parse("guarda"), GarmCommand::Save);
        assert_eq!(parse("guardar"), GarmCommand::Save);
        assert_eq!(parse("persiste"), GarmCommand::Save);
        assert_eq!(parse("carga"), GarmCommand::Load);
        assert_eq!(parse("recupera"), GarmCommand::Load);
        assert_eq!(parse("restore"), GarmCommand::Load);
    }

    #[test]
    fn parses_legacy_observatory_intents() {
        assert_eq!(parse("observatorio"), GarmCommand::Observatory);
        assert_eq!(parse("dashboard"), GarmCommand::Observatory);
        assert_eq!(parse("estado global"), GarmCommand::Observatory);
        assert_eq!(parse("ver todo"), GarmCommand::Observatory);
    }

    #[test]
    fn parses_legacy_history_intents() {
        assert_eq!(parse("historial"), GarmCommand::History);
        assert_eq!(parse("mi historial"), GarmCommand::History);
        assert_eq!(parse("log"), GarmCommand::History);
        assert_eq!(parse("log de eventos"), GarmCommand::History);
        assert_eq!(parse("registro evolutivo"), GarmCommand::History);
    }

    #[test]
    fn parses_legacy_start_stop_intents() {
        assert_eq!(parse("start"), GarmCommand::Start);
        assert_eq!(parse("despierta"), GarmCommand::Start);
        assert_eq!(parse("stop"), GarmCommand::Stop);
        assert_eq!(parse("duerme"), GarmCommand::Stop);
    }

    #[test]
    fn parses_legacy_evolution_intents() {
        assert_eq!(parse("evoluciona"), GarmCommand::Evolve);
        assert_eq!(parse("mejorate"), GarmCommand::Evolve);
        assert_eq!(parse("evolve"), GarmCommand::Evolve);
        assert_eq!(parse("improve"), GarmCommand::Evolve);
        assert_eq!(parse("self-improve"), GarmCommand::Evolve);
        assert_eq!(parse("rebirth"), GarmCommand::Rebirth);
        assert_eq!(parse("renacimiento"), GarmCommand::Rebirth);
        assert_eq!(parse("readiness"), GarmCommand::Readiness);
        assert_eq!(parse("readiness bench"), GarmCommand::ReadinessBench);
        assert_eq!(parse("readiness probe"), GarmCommand::ReadinessProbe);
        assert_eq!(parse("readiness external"), GarmCommand::ReadinessExternal);
        assert_eq!(
            parse("readiness external run"),
            GarmCommand::ReadinessExternalRun
        );
        assert_eq!(parse("readiness package"), GarmCommand::ReadinessPackage);
        assert_eq!(parse("action evidence"), GarmCommand::ActionEvidence);
        assert_eq!(parse("capabilities audit"), GarmCommand::CapabilityRegistry);
        assert_eq!(parse("readiness plan"), GarmCommand::ReadinessPlan);
        assert_eq!(parse("readiness run"), GarmCommand::ReadinessRun);
        assert_eq!(parse("ritual"), GarmCommand::OrganicRitual);
        assert_eq!(parse("umbra"), GarmCommand::OrganicRitual);
    }

    #[test]
    fn parses_recovered_organ_commands() {
        assert_eq!(
            parse("lengua energia"),
            GarmCommand::Lengua("energia".to_string())
        );
        assert_eq!(
            parse("cuando memoria"),
            GarmCommand::Reloj("memoria".to_string())
        );
        assert_eq!(
            parse("validar bird can fly"),
            GarmCommand::Juez("bird can fly".to_string())
        );
        assert_eq!(parse("voz"), GarmCommand::Voz);
        assert_eq!(
            parse("tts hola eden"),
            GarmCommand::VozTexto("hola eden".to_string())
        );
        assert_eq!(parse("intestino"), GarmCommand::Intestino);
        assert_eq!(parse("piel"), GarmCommand::Piel);
        assert_eq!(parse("autotuning"), GarmCommand::Autotuning);
        assert_eq!(
            parse("hrm memoria"),
            GarmCommand::Hrm("memoria".to_string())
        );
        assert_eq!(
            parse("hrm run memoria"),
            GarmCommand::HrmRun("memoria".to_string())
        );
        assert_eq!(parse("garm audit"), GarmCommand::GarmAudit);
        assert_eq!(parse("garm report"), GarmCommand::GarmReport);
        assert_eq!(parse("garm report history"), GarmCommand::GarmReportHistory);
        assert_eq!(parse("garm export"), GarmCommand::GarmExport);
        assert_eq!(parse("garm import"), GarmCommand::GarmImport);
        assert_eq!(parse("garm verify export"), GarmCommand::GarmVerifyExport);
        assert_eq!(parse("garm artifacts"), GarmCommand::GarmArtifacts);
        assert_eq!(parse("garm backup"), GarmCommand::GarmBackup);
        assert_eq!(parse("garm restore"), GarmCommand::GarmRestore);
        assert_eq!(parse("garm compact"), GarmCommand::GarmCompact);
    }

    #[test]
    fn parses_goal_scheduler_commands() {
        assert_eq!(parse("goals"), GarmCommand::Goals);
        assert_eq!(
            parse("goals plan mejorar arquitectura"),
            GarmCommand::GoalsPlan("mejorar arquitectura".to_string())
        );
        assert_eq!(parse("goals run"), GarmCommand::GoalsRun);
        assert_eq!(parse("goals audit"), GarmCommand::GoalsAudit);
    }

    #[test]
    fn parses_evaluation_loop_commands() {
        assert_eq!(parse("eval"), GarmCommand::Evaluation);
        assert_eq!(parse("eval run"), GarmCommand::EvaluationRun);
        assert_eq!(parse("eval audit"), GarmCommand::EvaluationAudit);
        assert_eq!(parse("evaluar arquitectura"), GarmCommand::EvaluationRun);
    }

    #[test]
    fn parses_learning_ledger_commands() {
        assert_eq!(parse("learning"), GarmCommand::Learning);
        assert_eq!(
            parse("learning record eval mejora arquitectura"),
            GarmCommand::LearningRecord("eval mejora arquitectura".to_string())
        );
        assert_eq!(
            parse("learning consolidate"),
            GarmCommand::LearningConsolidate
        );
        assert_eq!(parse("learning audit"), GarmCommand::LearningAudit);
    }

    #[test]
    fn parses_world_model_commands() {
        assert_eq!(parse("world"), GarmCommand::WorldModel);
        assert_eq!(
            parse("world observe rain causes wet_ground"),
            GarmCommand::WorldObserve("rain causes wet_ground".to_string())
        );
        assert_eq!(
            parse("world predict rain"),
            GarmCommand::WorldPredict("rain".to_string())
        );
        assert_eq!(parse("world verify"), GarmCommand::WorldVerify);
        assert_eq!(parse("world audit"), GarmCommand::WorldAudit);
        assert_eq!(parse("world eval"), GarmCommand::WorldEval);
    }

    #[test]
    fn parses_competence_benchmark_commands() {
        assert_eq!(parse("bench"), GarmCommand::Benchmark);
        assert_eq!(parse("benchmark run"), GarmCommand::BenchmarkRun);
        assert_eq!(parse("competence audit"), GarmCommand::BenchmarkAudit);
    }

    #[test]
    fn parses_plan_executor_commands() {
        assert_eq!(parse("exec"), GarmCommand::PlanExecutor);
        assert_eq!(parse("exec run"), GarmCommand::PlanExecutorRun);
        assert_eq!(parse("executor audit"), GarmCommand::PlanExecutorAudit);
        assert_eq!(
            parse("exec plan mejorar benchmark"),
            GarmCommand::PlanExecutorPlan("mejorar benchmark".to_string())
        );
    }

    #[test]
    fn parses_attention_commands() {
        assert_eq!(parse("attention"), GarmCommand::Attention);
        assert_eq!(parse("attention clear"), GarmCommand::AttentionClear);
        assert_eq!(parse("attn audit"), GarmCommand::AttentionAudit);
        assert_eq!(
            parse("attention benchmark risk"),
            GarmCommand::AttentionAttend("benchmark risk".to_string())
        );
    }

    #[test]
    fn parses_uncertainty_commands() {
        assert_eq!(parse("uncertainty"), GarmCommand::Uncertainty);
        assert_eq!(
            parse("uncertainty resolve"),
            GarmCommand::UncertaintyResolve
        );
        assert_eq!(parse("risk audit"), GarmCommand::UncertaintyAudit);
        assert_eq!(
            parse("uncertainty record unknown risk"),
            GarmCommand::UncertaintyRecord("unknown risk".to_string())
        );
    }

    #[test]
    fn parses_experiment_commands() {
        assert_eq!(parse("experiment"), GarmCommand::Experiment);
        assert_eq!(parse("experiment run"), GarmCommand::ExperimentRun);
        assert_eq!(parse("experiment audit"), GarmCommand::ExperimentAudit);
        assert_eq!(
            parse("experiment plan benchmark improves evaluation"),
            GarmCommand::ExperimentPlan("benchmark improves evaluation".to_string())
        );
    }

    #[test]
    fn parses_provenance_commands() {
        assert_eq!(parse("provenance"), GarmCommand::Provenance);
        assert_eq!(parse("provenance verify"), GarmCommand::ProvenanceVerify);
        assert_eq!(parse("evidence audit"), GarmCommand::ProvenanceAudit);
        assert_eq!(
            parse("provenance record test passed"),
            GarmCommand::ProvenanceRecord("test passed".to_string())
        );
    }

    #[test]
    fn parses_policy_commands() {
        assert_eq!(parse("policy"), GarmCommand::Policy);
        assert_eq!(parse("policy audit"), GarmCommand::PolicyAudit);
        assert_eq!(
            parse("policy eval local benchmark"),
            GarmCommand::PolicyEval("local benchmark".to_string())
        );
    }

    #[test]
    fn parses_hybrid_voice_commands() {
        assert_eq!(parse("hybrid voice"), GarmCommand::HybridVoice);
        assert_eq!(parse("hybrid voice audit"), GarmCommand::HybridVoiceAudit);
        assert_eq!(
            parse("hybrid voice plan hola"),
            GarmCommand::HybridVoicePlan("hola".to_string())
        );
        assert_eq!(
            parse("hybrid voice synth hola"),
            GarmCommand::HybridVoiceSynth("hola".to_string())
        );
    }

    #[test]
    fn parses_maturity_commands() {
        assert_eq!(parse("maturity"), GarmCommand::Maturity);
        assert_eq!(parse("maturity audit"), GarmCommand::MaturityAudit);
        assert_eq!(
            parse("maturity assess policy guard"),
            GarmCommand::MaturityAssess("policy guard".to_string())
        );
    }

    #[test]
    fn parses_safe_crawler_command() {
        assert_eq!(
            parse("crawl https://example.com"),
            GarmCommand::Crawl("https://example.com".to_string())
        );
        assert_eq!(
            parse("buscar web https://example.com"),
            GarmCommand::Crawl("https://example.com".to_string())
        );
        assert_eq!(
            parse("conceptnet /tmp/conceptnet.tsv"),
            GarmCommand::ConceptNet("/tmp/conceptnet.tsv".to_string())
        );
    }

    #[test]
    fn parses_legacy_reasoning_intents() {
        assert_eq!(
            parse("que es rust"),
            GarmCommand::WhatIs("rust".to_string())
        );
        assert_eq!(
            parse("why energia"),
            GarmCommand::Why("energia".to_string())
        );
        assert_eq!(
            parse("cual es la razon energia"),
            GarmCommand::Why("energia".to_string())
        );
        assert_eq!(
            parse("tell me about memoria"),
            GarmCommand::TellMe("memoria".to_string())
        );
    }

    #[test]
    fn parses_hrm_text_pretraining_commands() {
        assert_eq!(parse("hrm text"), GarmCommand::HrmText);
        assert_eq!(parse("hrm text plan"), GarmCommand::HrmTextPlan);
        assert_eq!(parse("hrm text run"), GarmCommand::HrmTextRun);
        assert_eq!(parse("hrm text eval"), GarmCommand::HrmTextEval);
        assert_eq!(parse("hrm text audit"), GarmCommand::HrmTextAudit);
        assert_eq!(
            parse("hrm text corpus /tmp/corpus.txt"),
            GarmCommand::HrmTextCorpus("/tmp/corpus.txt".to_string())
        );
        assert_eq!(
            parse("hrm text ingest /tmp/corpus"),
            GarmCommand::HrmTextIngest("/tmp/corpus".to_string())
        );
        assert_eq!(
            parse("hrm text search local evidence"),
            GarmCommand::HrmTextSearch("local evidence".to_string())
        );
        assert_eq!(
            parse("rag answer local evidence"),
            GarmCommand::HrmTextContext("local evidence".to_string())
        );
        assert_eq!(
            parse("hrm text objective text to plan priors"),
            GarmCommand::HrmTextObjective("text to plan priors".to_string())
        );
    }

    #[test]
    fn covers_all_legacy_simple_nlp_keywords() {
        let exact_cases = [
            ("hola", GarmCommand::Greeting),
            ("hello", GarmCommand::Greeting),
            ("hi", GarmCommand::Greeting),
            ("hey", GarmCommand::Greeting),
            ("que tal", GarmCommand::Greeting),
            ("buenos", GarmCommand::Greeting),
            ("estas", GarmCommand::Status),
            ("estado", GarmCommand::Status),
            ("status", GarmCommand::Status),
            ("que pasa", GarmCommand::Status),
            ("que haces", GarmCommand::Status),
            ("phi", GarmCommand::Phi),
            ("conciencia", GarmCommand::Phi),
            ("consciousness", GarmCommand::Phi),
            ("consciencia", GarmCommand::Phi),
            ("medir", GarmCommand::Phi),
            ("medicion", GarmCommand::Phi),
            ("ayuda", GarmCommand::Help),
            ("help", GarmCommand::Help),
            ("comandos", GarmCommand::Help),
            ("commands", GarmCommand::Help),
            ("que puedes", GarmCommand::Help),
            ("adios", GarmCommand::Quit),
            ("bye", GarmCommand::Quit),
            ("salir", GarmCommand::Quit),
            ("exit", GarmCommand::Quit),
            ("quit", GarmCommand::Quit),
            ("hasta luego", GarmCommand::Quit),
            ("memoria", GarmCommand::Memory),
            ("que sabes", GarmCommand::Memory),
            ("que recuerdas", GarmCommand::Memory),
            ("what do you know", GarmCommand::Memory),
            ("what do you remember", GarmCommand::Memory),
            ("evoluciona", GarmCommand::Evolve),
            ("mejorate", GarmCommand::Evolve),
            ("evolve", GarmCommand::Evolve),
            ("improve", GarmCommand::Evolve),
            ("grow", GarmCommand::Evolve),
            ("self-improve", GarmCommand::Evolve),
            ("mutate", GarmCommand::Evolve),
            ("quien eres", GarmCommand::SelfQuery),
            ("who are you", GarmCommand::SelfQuery),
            ("tu mismo", GarmCommand::SelfQuery),
            ("yourself", GarmCommand::SelfQuery),
            ("tu identidad", GarmCommand::SelfQuery),
            ("guarda", GarmCommand::Save),
            ("save", GarmCommand::Save),
            ("guardar", GarmCommand::Save),
            ("persiste", GarmCommand::Save),
            ("carga", GarmCommand::Load),
            ("load", GarmCommand::Load),
            ("recupera", GarmCommand::Load),
            ("restore", GarmCommand::Load),
            ("mi historial", GarmCommand::History),
            ("ver historial", GarmCommand::History),
            ("eventos evolutivos", GarmCommand::History),
            ("log de eventos", GarmCommand::History),
            ("registro evolutivo", GarmCommand::History),
            ("que piensas", GarmCommand::Thinking),
            ("what are you thinking", GarmCommand::Thinking),
            ("como piensas", GarmCommand::Thinking),
            ("explicale", GarmCommand::Thinking),
            ("explain yourself", GarmCommand::Thinking),
            ("tu mente", GarmCommand::Thinking),
            ("your mind", GarmCommand::Thinking),
            ("observatorio", GarmCommand::Observatory),
            ("dashboard", GarmCommand::Observatory),
            ("metricas", GarmCommand::Observatory),
            ("sistemas", GarmCommand::Observatory),
            ("estado global", GarmCommand::Observatory),
            ("panorama", GarmCommand::Observatory),
            ("ver todo", GarmCommand::Observatory),
            ("iniciar", GarmCommand::Start),
            ("despierta", GarmCommand::Start),
            ("empieza", GarmCommand::Start),
            ("vivir", GarmCommand::Start),
            ("start", GarmCommand::Start),
            ("awake", GarmCommand::Start),
            ("corre", GarmCommand::Start),
            ("run", GarmCommand::Start),
            ("detener", GarmCommand::Stop),
            ("para", GarmCommand::Stop),
            ("duerme", GarmCommand::Stop),
            ("stop", GarmCommand::Stop),
            ("pausa", GarmCommand::Stop),
            ("halt", GarmCommand::Stop),
            ("quieto", GarmCommand::Stop),
            ("como te sientes", GarmCommand::Feeling),
            ("como estas", GarmCommand::Feeling),
            ("how are you", GarmCommand::Feeling),
            ("how do you feel", GarmCommand::Feeling),
            ("que emocion", GarmCommand::Feeling),
            ("que sientes", GarmCommand::Feeling),
            ("voz", GarmCommand::Voz),
            ("hrm text", GarmCommand::HrmText),
            ("hrm text plan", GarmCommand::HrmTextPlan),
            ("hrm text run", GarmCommand::HrmTextRun),
            ("hrm text audit", GarmCommand::HrmTextAudit),
            ("intestino", GarmCommand::Intestino),
            ("piel", GarmCommand::Piel),
            ("autotuning", GarmCommand::Autotuning),
            ("cag", GarmCommand::Cag),
            ("contexto", GarmCommand::Cag),
            ("cache", GarmCommand::Cag),
            ("cag actions", GarmCommand::CagActions),
            ("cag audit", GarmCommand::CagAudit),
            ("organos", GarmCommand::Organs),
            ("organs", GarmCommand::Organs),
            ("organos audit", GarmCommand::OrgansAudit),
            ("organos plan", GarmCommand::OrgansPlan),
            ("organos run", GarmCommand::OrgansRun),
            ("organos health", GarmCommand::OrgansHealth),
            ("organos reparar", GarmCommand::OrgansRepair),
            ("organos actions", GarmCommand::OrgansActions),
            ("organos feedback good", GarmCommand::OrgansFeedback(true)),
            ("organos feedback bad", GarmCommand::OrgansFeedback(false)),
        ];
        for (raw, expected) in exact_cases {
            assert_eq!(parse(raw), expected, "legacy keyword '{}'", raw);
        }

        let prefixed_cases = [
            ("aprende tema", GarmCommand::Remember("tema".to_string())),
            (
                "aprendizaje tema",
                GarmCommand::Remember("tema".to_string()),
            ),
            ("recuerda tema", GarmCommand::Remember("tema".to_string())),
            ("recorder tema", GarmCommand::Remember("tema".to_string())),
            ("learn topic", GarmCommand::Remember("topic".to_string())),
            ("remember topic", GarmCommand::Remember("topic".to_string())),
            ("que es tema", GarmCommand::WhatIs("tema".to_string())),
            ("what is topic", GarmCommand::WhatIs("topic".to_string())),
            (
                "definicion de tema",
                GarmCommand::WhatIs("tema".to_string()),
            ),
            (
                "definition of topic",
                GarmCommand::WhatIs("topic".to_string()),
            ),
            ("explicame tema", GarmCommand::WhatIs("tema".to_string())),
            ("explain topic", GarmCommand::WhatIs("topic".to_string())),
            ("por que tema", GarmCommand::Why("tema".to_string())),
            ("why topic", GarmCommand::Why("topic".to_string())),
            (
                "cual es la razon tema",
                GarmCommand::Why("tema".to_string()),
            ),
            ("causa de tema", GarmCommand::Why("tema".to_string())),
            ("motivo de tema", GarmCommand::Why("tema".to_string())),
            ("cuentame tema", GarmCommand::TellMe("tema".to_string())),
            ("hablame de tema", GarmCommand::TellMe("tema".to_string())),
            ("dime sobre tema", GarmCommand::TellMe("tema".to_string())),
            (
                "tell me about topic",
                GarmCommand::TellMe("topic".to_string()),
            ),
            (
                "que opinas de tema",
                GarmCommand::TellMe("tema".to_string()),
            ),
            (
                "que piensas de tema",
                GarmCommand::TellMe("tema".to_string()),
            ),
            (
                "cag explain tema",
                GarmCommand::CagExplain("tema".to_string()),
            ),
            (
                "explica contexto tema",
                GarmCommand::CagExplain("tema".to_string()),
            ),
            ("cag gaps tema", GarmCommand::CagGaps("tema".to_string())),
            (
                "brechas contexto tema",
                GarmCommand::CagGaps("tema".to_string()),
            ),
            ("cag plan tema", GarmCommand::CagPlan("tema".to_string())),
            ("cag run tema", GarmCommand::CagRun("tema".to_string())),
        ];
        for (raw, expected) in prefixed_cases {
            assert_eq!(parse(raw), expected, "legacy prefixed keyword '{}'", raw);
        }
    }
}
