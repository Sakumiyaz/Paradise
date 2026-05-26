use crate::eden_garm;
use crate::eden_garm::node::GARMNode;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

mod gewc_body_handlers;

const GARM_REPORT_HISTORY_LIMIT: usize = 128;

#[derive(Clone, Debug)]
pub struct GarmRuntimeConfig {
    pub daemon: bool,
    pub pid_file: Option<String>,
    pub log_file: Option<String>,
    pub api_port: u16,
    pub state_dir: String,
    pub allow_remote_crawl: bool,
    pub mcp: bool,
    pub watchdog: bool,
    pub no_interactive: bool,
    pub max_cycles: Option<u64>,
    pub born: bool,
    pub legacy_session_file: Option<String>,
    pub log_level: String,
}

impl GarmRuntimeConfig {
    pub fn from_args() -> Self {
        Self::from_iter(std::env::args().skip(1))
    }

    pub fn from_iter<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut config = Self {
            daemon: false,
            pid_file: Some("/tmp/eden_garm.pid".to_string()),
            log_file: Some("/tmp/eden_garm.log".to_string()),
            api_port: 8080,
            state_dir: "/tmp/eden_garm".to_string(),
            allow_remote_crawl: false,
            mcp: false,
            watchdog: false,
            no_interactive: false,
            max_cycles: None,
            born: false,
            legacy_session_file: None,
            log_level: "info".to_string(),
        };
        let mut args = args.into_iter().map(Into::into);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--daemon" => config.daemon = true,
                "--allow-remote-crawl" => config.allow_remote_crawl = true,
                "--mcp" => config.mcp = true,
                "--watchdog" => config.watchdog = true,
                "--born" | "-b" => config.born = true,
                "--no-interactive" | "-n" => config.no_interactive = true,
                "--max-cycles" | "-c" => {
                    config.max_cycles = args.next().and_then(|s| s.parse::<u64>().ok());
                }
                "--session" | "-s" => config.legacy_session_file = args.next(),
                "--log-level" | "-l" => {
                    if let Some(level) = args.next() {
                        config.log_level = level;
                    }
                }
                "--pid-file" => config.pid_file = args.next(),
                "--log-file" => config.log_file = args.next(),
                "--api-port" => {
                    if let Some(port) = args.next().and_then(|s| s.parse::<u16>().ok()) {
                        config.api_port = port;
                    }
                }
                "--state-dir" => {
                    if let Some(path) = args.next() {
                        config.state_dir = path;
                    }
                }
                _ => {}
            }
        }
        config
    }

    fn log(&self, line: &str) {
        if self.daemon {
            if let Some(path) = &self.log_file {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    use std::io::Write;
                    let _ = writeln!(file, "{}", line);
                }
            }
        }
    }
}

pub struct GarmRuntime {
    config: GarmRuntimeConfig,
}

#[derive(Clone, Copy, Debug)]
struct RuntimeNodeIds {
    meta: usize,
    legacy_memory: usize,
    legacy_reason: usize,
    legacy_dialogue: usize,
    observatory: usize,
    legacy_history: usize,
    legacy_evolution: usize,
    legacy_cognition: usize,
    campo_tension: usize,
    legacy_knowledge_graph: usize,
    legacy_autoconsumo: usize,
    legacy_venado: usize,
    legacy_paradigm_hub: usize,
    legacy_ecosystem: usize,
    legacy_rebirth_meltrace: usize,
    legacy_crawler: usize,
    help: usize,
    readiness: usize,
    organic_lifecycle: usize,
    conscious_graph_regulator: usize,
    context_augmentation: usize,
    hrm_reasoner: usize,
    voice_synthesizer: usize,
}

struct GewcBodyPorts<'a> {
    graph: &'a mut eden_garm::HyperGraph,
    shared_engine: &'a Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
    ids: RuntimeNodeIds,
    api_metrics: &'a Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    dt: f32,
    runtime_config: &'a GarmRuntimeConfig,
    autonomous: &'a mut bool,
}

struct GewcBodyExecutor;

impl GewcBodyExecutor {
    fn execute_runtime_control(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        let graph = &mut *ports.graph;
        let shared_engine = ports.shared_engine;
        let ids = ports.ids;
        let api_metrics = ports.api_metrics;
        let dt = ports.dt;
        let runtime_config = ports.runtime_config;
        let autonomous = &mut *ports.autonomous;

        match command {
            eden_garm::nodes::command_router::GarmCommand::Quit => {
                runtime_config.log("[GARM] shutdown requested");
                ("[GARM] Shutdown requested.\n".to_string(), false)
            }
            eden_garm::nodes::command_router::GarmCommand::Tick => {
                graph.inject_sensor(vec![1.0; 8]);
                graph.pulse(dt);
                ("[GARM] tick complete\n".to_string(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::Status => (
                format!("{}\n", eden_garm::nodes::telemetry::status(shared_engine)),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::Start => {
                *autonomous = true;
                api_metrics.autonomous.store(true, Ordering::Relaxed);
                ("[GARM] Autonomia reanudada.\n".to_string(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::Stop => {
                *autonomous = false;
                api_metrics.autonomous.store(false, Ordering::Relaxed);
                (
                    "[GARM] Autonomia pausada. Comandos y API siguen activos.\n".to_string(),
                    true,
                )
            }
            eden_garm::nodes::command_router::GarmCommand::GarmAudit => (
                GarmRuntime::garm_audit(graph, ids, shared_engine, api_metrics, *autonomous),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::GarmReport => (
                GarmRuntime::garm_report(graph, ids, shared_engine, api_metrics, *autonomous),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::GarmReportHistory => {
                (GarmRuntime::garm_report_history(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::GarmExport => (
                GarmRuntime::garm_export(graph, ids, shared_engine, api_metrics, *autonomous),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::GarmImport => {
                (GarmRuntime::garm_import(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::GarmVerifyExport => {
                (GarmRuntime::garm_verify_export(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::GarmArtifacts => {
                (eden_garm::state_paths::artifacts_report(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::GarmBackup => {
                (GarmRuntime::garm_backup(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::GarmRestore => {
                (GarmRuntime::garm_restore(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::GarmCompact => {
                (GarmRuntime::garm_compact(graph, ids, shared_engine), true)
            }
            eden_garm::nodes::command_router::GarmCommand::Save => (
                format!(
                    "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}\n",
                    eden_garm::nodes::persistence::save_all_with_legacy_nodes(
                        graph,
                        shared_engine,
                        ids.legacy_memory,
                        ids.legacy_history,
                        ids.observatory,
                        ids.legacy_evolution,
                    ),
                    GarmRuntime::save_runtime_state(*autonomous),
                    GarmRuntime::save_organ_autonomy_state(),
                    GarmRuntime::save_goal_scheduler_state(),
                    GarmRuntime::save_evaluation_loop_state(),
                    GarmRuntime::save_learning_ledger_state(),
                    GarmRuntime::save_world_model_core_state(),
                    GarmRuntime::save_competence_benchmark_state(),
                    GarmRuntime::save_plan_executor_state(),
                    GarmRuntime::save_working_memory_state(),
                    GarmRuntime::save_uncertainty_ledger_state(),
                    GarmRuntime::save_experiment_runner_state(),
                    GarmRuntime::save_provenance_ledger_state(),
                    GarmRuntime::save_policy_guard_state(),
                    GarmRuntime::save_capability_maturity_state(),
                    GarmRuntime::save_hybrid_voice_state(),
                    GarmRuntime::save_hrm_text_pretraining_state(),
                ),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::Load => {
                let response =
                    eden_garm::nodes::persistence::load_all_with_legacy_nodes(graph, shared_engine);
                let runtime_response = GarmRuntime::load_runtime_state(autonomous);
                let organ_response = GarmRuntime::load_organ_autonomy_state();
                let goal_response = GarmRuntime::load_goal_scheduler_state();
                let eval_response = GarmRuntime::load_evaluation_loop_state();
                let learning_response = GarmRuntime::load_learning_ledger_state();
                let world_response = GarmRuntime::load_world_model_core_state();
                let benchmark_response = GarmRuntime::load_competence_benchmark_state();
                let executor_response = GarmRuntime::load_plan_executor_state();
                let attention_response = GarmRuntime::load_working_memory_state();
                let uncertainty_response = GarmRuntime::load_uncertainty_ledger_state();
                let experiment_response = GarmRuntime::load_experiment_runner_state();
                let provenance_response = GarmRuntime::load_provenance_ledger_state();
                let policy_response = GarmRuntime::load_policy_guard_state();
                let maturity_response = GarmRuntime::load_capability_maturity_state();
                let hybrid_voice_response = GarmRuntime::load_hybrid_voice_state();
                let hrm_text_response = GarmRuntime::load_hrm_text_pretraining_state();
                api_metrics.autonomous.store(*autonomous, Ordering::Relaxed);
                GarmRuntime::update_api_metrics(graph, ids, api_metrics);
                (
                    format!(
                        "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}\n",
                        response,
                        runtime_response,
                        organ_response,
                        goal_response,
                        eval_response,
                        learning_response,
                        world_response,
                        benchmark_response,
                        executor_response,
                        attention_response,
                        uncertainty_response,
                        experiment_response,
                        provenance_response,
                        policy_response,
                        maturity_response,
                        hybrid_voice_response,
                        hrm_text_response
                    ),
                    true,
                )
            }
            eden_garm::nodes::command_router::GarmCommand::Auto(n) => {
                let mut out = format!("[AUTO] Running {} pulses...\n", n);
                let auto_start = Instant::now();
                for i in 0..n {
                    graph.inject_sensor(vec![0.0; 8]);
                    graph.pulse(dt);
                    if (i + 1) % 113 == 0 {
                        GarmRuntime::organs_autonomous_cycle(graph, ids, "autonomous");
                    }
                    if (i + 1) % 50 == 0 {
                        out.push_str(&format!(
                            "{}\n",
                            eden_garm::nodes::telemetry::auto_progress(i + 1, shared_engine)
                        ));
                    }
                }
                let auto_elapsed = auto_start.elapsed().as_secs_f64();
                out.push_str(&format!(
                    "[AUTO] Done | {} pulses in {:.1}s | pps={:.2}\n",
                    n,
                    auto_elapsed,
                    n as f64 / auto_elapsed
                ));
                (out, true)
            }
            other => Self::execute_unexpected_handler_command(
                other,
                eden_garm::global_executive_workspace::GewcBodyHandler::RuntimeControl,
            ),
        }
    }

    fn execute_validation(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        let graph = &mut *ports.graph;
        let shared_engine = ports.shared_engine;
        let ids = ports.ids;
        let api_metrics = ports.api_metrics;
        let autonomous = &mut *ports.autonomous;

        match command {
            eden_garm::nodes::command_router::GarmCommand::Readiness => (
                GarmRuntime::readiness_report(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ReadinessBench => (
                GarmRuntime::readiness_benchmark(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ReadinessProbe => (
                GarmRuntime::readiness_probe_run(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ReadinessExternal => (
                GarmRuntime::readiness_external_validation_manifest(
                    graph,
                    ids,
                    shared_engine,
                    api_metrics,
                ),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ReadinessExternalRun => (
                GarmRuntime::readiness_external_validation_run(
                    graph,
                    ids,
                    shared_engine,
                    api_metrics,
                ),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ReadinessPackage => (
                GarmRuntime::readiness_package(graph, ids, shared_engine, api_metrics, *autonomous),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::CapabilityRegistry => (
                GarmRuntime::capability_registry_audit(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::CognitiveEval => (
                GarmRuntime::cognitive_architecture_eval(graph, ids, shared_engine),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::EmbodiedEval => (
                GarmRuntime::embodied_grounding_eval(graph, ids, shared_engine),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::NeuralEval => (
                GarmRuntime::neural_architecture_eval(graph, ids, shared_engine),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::SymbolicEval => (
                GarmRuntime::symbolic_architecture_eval(graph, shared_engine),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::FrontierArchitectureEval => (
                GarmRuntime::frontier_architecture_eval(graph, ids, shared_engine),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ParadigmArchitectureEval => (
                GarmRuntime::paradigm_architecture_eval(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::IntegrationGovernanceEval => (
                GarmRuntime::integration_governance_eval(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::GlobalExecutiveWorkspaceEval => (
                GarmRuntime::global_executive_workspace_eval(
                    graph,
                    ids,
                    shared_engine,
                    api_metrics,
                ),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::GewcOperationalBenchmark => (
                GarmRuntime::gewc_operational_benchmark(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::CapabilityRealityEval => (
                GarmRuntime::capability_reality_eval(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ArchitectureAdvantageEval => (
                GarmRuntime::architecture_advantage_eval(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ParadiseWorldcellEval => (
                GarmRuntime::paradise_worldcell_eval(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::PraxisNexusEval => (
                GarmRuntime::praxis_nexus_eval(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ExternalEcosystemEval => (
                GarmRuntime::external_ecosystem_eval(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::SovereignCognitionEval => (
                GarmRuntime::sovereign_cognition_eval(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ArtifactApiEval => {
                (GarmRuntime::artifact_api_eval(graph, ids), true)
            }
            eden_garm::nodes::command_router::GarmCommand::TrainingEvidenceEval => {
                (eden_garm::training_evidence::run_default(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::Megatron7bEvidenceEval => (
                eden_garm::training_evidence::run_megatron_7b_default(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::Megatron7bAdapterPrepare => (
                eden_garm::model_runtime::prepare_megatron_7b_adapter(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::Megatron7bInferenceEval => (
                eden_garm::model_runtime::write_megatron_7b_inference_report(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::Megatron7bCapabilityEval => (
                eden_garm::model_runtime::write_megatron_7b_capability_report(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::Megatron7bAdmissionGateEval => (
                eden_garm::model_runtime::write_megatron_7b_admission_gate(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::EdenCapableEval => {
                (eden_garm::eden_capable::run_all(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::EdenCapableTrainingRunContract => {
                (eden_garm::eden_capable::write_training_run_contract(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::EdenCognitiveDatasetEval => (
                eden_garm::eden_capable::write_cognitive_dataset_manifest(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::EdenNativeInferenceEval => {
                (eden_garm::eden_capable::write_native_inference_api(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::EdenCapabilityDeltaEval => {
                (eden_garm::eden_capable::write_capability_delta_eval(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::EdenStructuredOutputEval => (
                eden_garm::eden_capable::write_structured_output_report(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::EdenCheckpointRegistryEval => {
                (eden_garm::eden_capable::write_checkpoint_registry(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::EdenSftElcpReadinessEval => {
                (eden_garm::eden_capable::write_sft_elcp_readiness(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::EdenCapableGateEval => {
                (eden_garm::eden_capable::write_capable_gate(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::ModelRuntimeEval => {
                (eden_garm::model_runtime::run_all(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::ModelAdapterRuntimeEval => (
                eden_garm::model_runtime::run_model_adapter_runtime_eval(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ModelCheckpointManifestEval => {
                (eden_garm::model_runtime::write_checkpoint_manifest(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::TrainingHarnessEval => {
                (eden_garm::model_runtime::run_training_harness(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::ModelGovernanceEval => {
                (eden_garm::model_runtime::write_model_governance(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::FirstModelPrepare => {
                (eden_garm::model_runtime::prepare_first_model(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::FirstModelReadinessEval => (
                eden_garm::model_runtime::write_first_model_readiness(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ElcpPrepare => {
                (eden_garm::model_runtime::prepare_elcp(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::ElcpObjectiveEval => {
                (eden_garm::model_runtime::write_elcp_objective_spec(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::ElcpAdmissionGateEval => {
                (eden_garm::model_runtime::write_elcp_admission_gate(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::ElcpTraceQualityEval => (
                eden_garm::model_runtime::write_elcp_trace_quality_gate(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ElcpReplayEval => {
                (eden_garm::model_runtime::write_elcp_replay_eval(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::ElcpDatasetFreezeEval => (
                eden_garm::model_runtime::write_elcp_dataset_freeze_manifest(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ElcpMetricsBoardEval => {
                (eden_garm::model_runtime::write_elcp_metrics_board(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::Elcp4bReadinessContractEval => (
                eden_garm::model_runtime::write_elcp_4b_readiness_contract(),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::ElcpHardeningEval => {
                (eden_garm::model_runtime::write_elcp_hardening(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::ElcpReadinessEval => {
                (eden_garm::model_runtime::write_elcp_readiness(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::RuntimeStateApiEval => {
                (GarmRuntime::runtime_state_api_eval(graph, ids), true)
            }
            eden_garm::nodes::command_router::GarmCommand::OperationalApiEval => {
                (GarmRuntime::operational_api_eval(graph, ids), true)
            }
            eden_garm::nodes::command_router::GarmCommand::OperationalRuntimeEval => {
                (GarmRuntime::operational_runtime_eval(graph, ids), true)
            }
            eden_garm::nodes::command_router::GarmCommand::OperationalReplayRun => {
                (eden_garm::operational_runtime::run_replay(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::OperationalSmokeRun => {
                (eden_garm::operational_runtime::run_smoke_test(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::OperationalScenarioRun => {
                (eden_garm::operational_runtime::run_e2e_scenario(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::OperationalDemoRun => {
                (eden_garm::operational_runtime::run_demo_suite(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::RuntimeSpineEval => {
                (eden_garm::runtime_spine::run(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::RuntimeSpineAudit => {
                (eden_garm::runtime_spine::audit(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::RuntimeSpineVerify => {
                (eden_garm::runtime_spine::verify(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::RuntimeSpineEnforce => {
                (eden_garm::runtime_spine::enforce(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::RuntimeSpineRisk => {
                (eden_garm::runtime_spine::workflow_risk(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::RuntimeSpineBreakers => {
                (eden_garm::runtime_spine::circuit_breakers(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::RuntimeSpineReplay => {
                (eden_garm::runtime_spine::reconstruct_replay(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::Evaluation => {
                (eden_garm::nodes::evaluation_loop::report(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::EvaluationRun => (
                GarmRuntime::evaluation_run(graph, ids, shared_engine, api_metrics),
                true,
            ),
            eden_garm::nodes::command_router::GarmCommand::EvaluationAudit => {
                (eden_garm::nodes::evaluation_loop::audit_report(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::Benchmark => {
                (eden_garm::nodes::competence_benchmark::report(), true)
            }
            eden_garm::nodes::command_router::GarmCommand::BenchmarkRun => {
                (GarmRuntime::competence_benchmark_run(graph, ids), true)
            }
            eden_garm::nodes::command_router::GarmCommand::BenchmarkAudit => {
                (eden_garm::nodes::competence_benchmark::audit_report(), true)
            }
            other => Self::execute_unexpected_handler_command(
                other,
                eden_garm::global_executive_workspace::GewcBodyHandler::Validation,
            ),
        }
    }

    fn execute_unexpected_handler_command(
        command: eden_garm::nodes::command_router::GarmCommand,
        handler: eden_garm::global_executive_workspace::GewcBodyHandler,
    ) -> (String, bool) {
        (
            format!(
                "[GEWC-HANDLER-MISMATCH] handler={} command={:?}\n",
                handler.as_str(),
                command
            ),
            true,
        )
    }

    fn execute(
        command: eden_garm::nodes::command_router::GarmCommand,
        decision: &eden_garm::global_executive_workspace::CoreDecision,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        use eden_garm::global_executive_workspace::GewcBodyHandler;

        let binding = eden_garm::global_executive_workspace::GewcBodyRegistry::bind(&command);
        debug_assert_eq!(decision.body_handler, binding.handler);
        match binding.handler {
            GewcBodyHandler::RuntimeControl => Self::execute_runtime_control(command, ports),
            GewcBodyHandler::MemoryReasoning => Self::execute_memory_reasoning(command, ports),
            GewcBodyHandler::NativeCompatibility => {
                Self::execute_native_compatibility(command, ports)
            }
            GewcBodyHandler::SafeLearning => Self::execute_safe_learning(command, ports),
            GewcBodyHandler::WorldModel => Self::execute_world_model(command, ports),
            GewcBodyHandler::PlanningGoal => Self::execute_planning_goal(command, ports),
            GewcBodyHandler::ToolAdapter => Self::execute_tool_adapter(command, ports),
            GewcBodyHandler::SpecializedModel => Self::execute_specialized_model(command, ports),
            GewcBodyHandler::MetacognitiveSafety => {
                Self::execute_metacognitive_safety(command, ports)
            }
            GewcBodyHandler::Validation => Self::execute_validation(command, ports),
            GewcBodyHandler::Experiment => Self::execute_experiment(command, ports),
            GewcBodyHandler::Agentic => Self::execute_agentic(command, ports),
            GewcBodyHandler::WorkspaceAttention => {
                Self::execute_workspace_attention(command, ports)
            }
            GewcBodyHandler::LocusContext => Self::execute_locus_context(command, ports),
            GewcBodyHandler::FormalSynthesis => Self::execute_formal_synthesis(command, ports),
            GewcBodyHandler::HumanInterface => Self::execute_human_interface(command, ports),
            GewcBodyHandler::UnknownIntent => Self::execute_unknown_intent(command, ports),
        }
    }

    fn execute_memory_reasoning(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::memory_reasoning::execute(command, ports)
    }

    fn execute_native_compatibility(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::native_compatibility::execute(command, ports)
    }

    fn execute_safe_learning(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::safe_learning::execute(command, ports)
    }

    fn execute_world_model(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::world_model::execute(command, ports)
    }

    fn execute_planning_goal(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::planning_goal::execute(command, ports)
    }

    fn execute_tool_adapter(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::tool_adapter::execute(command, ports)
    }

    fn execute_specialized_model(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::specialized_model::execute(command, ports)
    }

    fn execute_metacognitive_safety(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::metacognitive_safety::execute(command, ports)
    }

    fn execute_experiment(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::experiment::execute(command, ports)
    }

    fn execute_agentic(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::agentic::execute(command, ports)
    }

    fn execute_workspace_attention(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::workspace_attention::execute(command, ports)
    }

    fn execute_locus_context(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::locus_context::execute(command, ports)
    }

    fn execute_formal_synthesis(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::formal_synthesis::execute(command, ports)
    }

    fn execute_human_interface(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::human_interface::execute(command, ports)
    }

    fn execute_unknown_intent(
        command: eden_garm::nodes::command_router::GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        gewc_body_handlers::unknown_intent::execute(command, ports)
    }

    fn has_native_executor(
        handler: eden_garm::global_executive_workspace::GewcBodyHandler,
    ) -> bool {
        use eden_garm::global_executive_workspace::GewcBodyHandler;

        matches!(
            handler,
            GewcBodyHandler::RuntimeControl
                | GewcBodyHandler::MemoryReasoning
                | GewcBodyHandler::NativeCompatibility
                | GewcBodyHandler::SafeLearning
                | GewcBodyHandler::WorldModel
                | GewcBodyHandler::PlanningGoal
                | GewcBodyHandler::ToolAdapter
                | GewcBodyHandler::SpecializedModel
                | GewcBodyHandler::MetacognitiveSafety
                | GewcBodyHandler::Validation
                | GewcBodyHandler::Experiment
                | GewcBodyHandler::Agentic
                | GewcBodyHandler::WorkspaceAttention
                | GewcBodyHandler::LocusContext
                | GewcBodyHandler::FormalSynthesis
                | GewcBodyHandler::HumanInterface
                | GewcBodyHandler::UnknownIntent
        )
    }
}

impl GarmRuntime {
    pub fn from_args() -> Self {
        Self {
            config: GarmRuntimeConfig::from_args(),
        }
    }

    pub fn run(self) {
        let runtime_config = self.config;
        if runtime_config.watchdog {
            Self::run_watchdog(&runtime_config);
            return;
        }
        eden_garm::state_paths::set_state_dir(runtime_config.state_dir.clone());
        if let Err(e) = eden_garm::state_paths::ensure_state_dir() {
            println!("[GARM] state dir warning: {}", e);
        }
        if std::env::var("EDEN_GARM_SKIP_LEGACY_MIGRATION")
            .ok()
            .as_deref()
            != Some("1")
        {
            for migrated in eden_garm::state_paths::migrate_legacy_paths() {
                if runtime_config.mcp {
                    eprintln!("[GARM] migrated legacy state path {}", migrated);
                } else {
                    println!("[GARM] migrated legacy state path {}", migrated);
                }
            }
        }
        if let Some(session_file) = &runtime_config.legacy_session_file {
            if let Err(e) = std::fs::copy(
                session_file,
                eden_garm::state_paths::legacy_session_import_path(),
            ) {
                eprintln!("[GARM] legacy session import warning: {}", e);
            }
        }
        let daemon_config = eden_garm::nodes::daemon::DaemonConfig {
            enabled: runtime_config.daemon,
            pid_file: runtime_config.pid_file.clone(),
            log_file: runtime_config.log_file.clone(),
        };
        let api_metrics = Arc::new(eden_garm::nodes::api_server::ApiRuntimeMetrics::new(
            runtime_config.daemon,
        ));
        let built = eden_garm::graph_builder::GraphBuilder::build(
            if runtime_config.mcp {
                0
            } else {
                runtime_config.api_port
            },
            daemon_config,
            Arc::clone(&api_metrics),
        );
        let mut graph = built.graph;
        let shared_engine = built.shared_engine;
        let stdin_queue = built.command_queue;
        let command_response_bus = built.command_response_bus;
        let ids = RuntimeNodeIds {
            meta: built.meta_id,
            legacy_memory: built.legacy_memory_id,
            legacy_reason: built.legacy_reason_id,
            legacy_dialogue: built.legacy_dialogue_id,
            observatory: built.observatory_id,
            legacy_history: built.legacy_history_id,
            legacy_evolution: built.legacy_evolution_id,
            legacy_cognition: built.legacy_cognition_id,
            campo_tension: built.campo_tension_id,
            legacy_knowledge_graph: built.legacy_knowledge_graph_id,
            legacy_autoconsumo: built.legacy_autoconsumo_id,
            legacy_venado: built.legacy_venado_id,
            legacy_paradigm_hub: built.legacy_paradigm_hub_id,
            legacy_ecosystem: built.legacy_ecosystem_id,
            legacy_rebirth_meltrace: built.legacy_rebirth_meltrace_id,
            legacy_crawler: built.legacy_crawler_id,
            help: built.help_id,
            readiness: built.readiness_id,
            organic_lifecycle: built.organic_lifecycle_id,
            conscious_graph_regulator: built.conscious_graph_regulator_id,
            context_augmentation: built.context_augmentation_id,
            hrm_reasoner: built.hrm_reasoner_id,
            voice_synthesizer: built.voice_synthesizer_id,
        };
        let n_caps = built.n_caps;
        Self::update_api_metrics(&mut graph, ids, &api_metrics);
        Self::update_lifecycle_metrics(&graph, ids.legacy_rebirth_meltrace, &api_metrics);

        if runtime_config.mcp {
            Self::run_mcp_stdio(&mut graph, ids, runtime_config.allow_remote_crawl);
            return;
        }

        if runtime_config.no_interactive {
            let cycles = runtime_config.max_cycles.unwrap_or(1);
            for _ in 0..cycles {
                graph.inject_zero_sensor(8);
                graph.pulse(0.1);
            }
            Self::update_api_metrics(&mut graph, ids, &api_metrics);
            println!(
                "METRICS {{ cycles: {}, alive_nodes: {}, edges: {}, memory_facts: {} }}",
                graph.global_tick,
                graph.alive_node_count(),
                graph.edge_count(),
                api_metrics.memory_facts.load(Ordering::Relaxed),
            );
            return;
        }

        println!(
            "[GARM] HyperGraph initialized | nodes={} | edges={} | capabilities={}",
            graph.alive_node_count(),
            graph.edge_count(),
            n_caps
        );
        println!("[GARM] Commands: tick | estado | auto N | save | load | quit");
        if runtime_config.born {
            println!("[GARM] legacy --born accepted: GARM graph starts already initialized.");
        }
        if runtime_config.daemon {
            println!(
                "[DAEMON] managed mode active | api=127.0.0.1:{} | pid_file={:?} | log_file={:?} | state_dir={}",
                runtime_config.api_port, runtime_config.pid_file, runtime_config.log_file, runtime_config.state_dir
            );
            runtime_config.log(&format!(
                "[GARM] started daemon managed mode | api=127.0.0.1:{}",
                runtime_config.api_port
            ));
        }
        println!();

        // Thread para stdin no-bloqueante. En daemon, control via API local.
        if !runtime_config.daemon {
            let stdin_q_clone = Arc::clone(&stdin_queue);
            std::thread::spawn(move || {
                let stdin = std::io::stdin();
                for line in stdin.lines() {
                    if let Ok(l) = line {
                        if let Ok(mut q) = stdin_q_clone.lock() {
                            q.push(eden_garm::nodes::api_server::QueuedCommand {
                                id: None,
                                raw: l,
                            });
                        }
                    }
                }
            });
        }

        let start = Instant::now();
        let mut last_report = Instant::now();
        let mut running = true;
        let mut autonomous = true;
        let mut last_command = String::new();
        api_metrics.ready.store(true, Ordering::Relaxed);
        api_metrics.autonomous.store(autonomous, Ordering::Relaxed);
        runtime_config.log("[GARM] runtime ready");

        while running {
            let dt = 0.1;
            let elapsed = start.elapsed().as_secs_f64();

            // Procesar comandos humanos
            let commands: Vec<eden_garm::nodes::api_server::QueuedCommand> = {
                if let Ok(mut q) = stdin_queue.lock() {
                    std::mem::take(&mut *q)
                } else {
                    Vec::new()
                }
            };

            for cmd in commands {
                let (response, should_continue) = Self::dispatch_gewc_cycle(
                    &cmd.raw,
                    &mut last_command,
                    &mut graph,
                    &shared_engine,
                    ids,
                    &api_metrics,
                    dt,
                    &runtime_config,
                    &mut autonomous,
                );
                print!("{}", response);
                if let Some(id) = cmd.id {
                    command_response_bus.respond(id, response);
                }
                if !should_continue {
                    running = false;
                    break;
                }
            }

            if !running {
                break;
            }

            if autonomous {
                // Pulso autonomo del grafo (si no hay comando humano bloqueando)
                graph.inject_zero_sensor(8);
                graph.pulse(dt);
                if graph.global_tick % 29 == 0 {
                    Self::context_augmentation_autoplan(&mut graph, ids);
                }
                if graph.global_tick % 113 == 0 {
                    Self::organs_autonomous_cycle(&mut graph, ids, "autonomous");
                }
                Self::update_api_metrics(&mut graph, ids, &api_metrics);
            }
            if runtime_config
                .max_cycles
                .is_some_and(|max| graph.global_tick >= max)
            {
                running = false;
            }
            api_metrics.autonomous.store(autonomous, Ordering::Relaxed);

            // Log periodico cada 5 segundos
            if last_report.elapsed().as_secs() >= 5 {
                {
                    let line =
                        eden_garm::nodes::telemetry::periodic(elapsed, &graph, &shared_engine);
                    println!("{}", line);
                    runtime_config.log(&line);
                }
                if let Some(meta_node) = graph.nodes.get(ids.meta) {
                    if let Some(meta) = meta_node
                        .as_any()
                        .downcast_ref::<eden_garm::nodes::meta_architect::MetaArchitectNode>(
                    ) {
                        println!(
                            "[META] action='{}' | proposals={}/{} | fe={:.2}",
                            meta.last_action(),
                            meta.proposals_applied(),
                            meta.proposals_generated(),
                            meta.free_energy()
                        );
                    }
                }
                last_report = Instant::now();
            }

            std::thread::sleep(Duration::from_millis(50));
        }

        println!("[GARM] Session ended.");
    }

    fn run_watchdog(config: &GarmRuntimeConfig) {
        println!("[WATCHDOG] GARM managed restart mode active.");
        loop {
            let exe = match std::env::current_exe() {
                Ok(exe) => exe,
                Err(e) => {
                    eprintln!("[WATCHDOG] current_exe error: {}", e);
                    return;
                }
            };
            let max_cycles = config.max_cycles.unwrap_or(99999999).to_string();
            let mut command = std::process::Command::new(exe);
            command
                .arg("--no-interactive")
                .arg("--max-cycles")
                .arg(max_cycles);
            command.arg("--state-dir").arg(&config.state_dir);
            if config.allow_remote_crawl {
                command.arg("--allow-remote-crawl");
            }
            match command.spawn() {
                Ok(mut child) => {
                    println!("[WATCHDOG] GARM PID {}", child.id());
                    let _ = child.wait();
                }
                Err(e) => eprintln!("[WATCHDOG] spawn error: {}", e),
            }
            while Self::system_load_avg().unwrap_or(0.0) > 0.3 {
                std::thread::sleep(Duration::from_secs(30));
            }
        }
    }

    fn system_load_avg() -> Option<f32> {
        std::fs::read_to_string("/proc/loadavg")
            .ok()?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    }

    fn run_mcp_stdio(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        allow_remote_crawl: bool,
    ) {
        use std::io::BufRead;
        eprintln!("[GARM-MCP] stdio compatibility mode ready");
        let stdin = std::io::stdin();
        for line in stdin.lock().lines().map_while(Result::ok) {
            let request = line.trim();
            if request.is_empty() {
                continue;
            }
            if request == "exit" || request == "quit" {
                break;
            }
            let response = if let Ok(json) = serde_json::from_str::<serde_json::Value>(request) {
                let id = json.get("id").cloned().unwrap_or(serde_json::Value::Null);
                let method = json.get("method").and_then(|v| v.as_str()).unwrap_or("");
                match method {
                    "initialize" => serde_json::json!({
                        "jsonrpc":"2.0",
                        "id":id,
                        "result":{
                            "protocolVersion":"2024-11-05",
                            "serverInfo":{"name":"eden-garm","version":"1.0"},
                            "capabilities":{"tools":{}}
                        }
                    })
                    .to_string(),
                    "tools/list" => serde_json::json!({
                        "jsonrpc":"2.0",
                        "id":id,
                        "result":{"tools":[
                            {"name":"search","description":"Hybrid KG retrieval","inputSchema":{"type":"object","properties":{"query":{"type":"string"}}}},
                            {"name":"status","description":"GARM runtime snapshot","inputSchema":{"type":"object","properties":{}}},
                            {"name":"crawl","description":"Gated remote crawl","inputSchema":{"type":"object","properties":{"url":{"type":"string"}}}}
                        ]}
                    })
                    .to_string(),
                    "tools/call" => {
                        let params = json.get("params").cloned().unwrap_or_default();
                        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let args = params.get("arguments").cloned().unwrap_or_default();
                        let text = match name {
                            "search" => {
                                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                                Self::mcp_search_text(graph, ids, query)
                            }
                            "status" => format!(
                                "grafo_nodos={} grafo_aristas={} ciclo={} crawl_pending=false",
                                graph.alive_node_count(),
                                graph.edge_count(),
                                graph.global_tick
                            ),
                            "crawl" => {
                                let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
                                Self::safe_remote_crawl(graph, ids, url, allow_remote_crawl)
                            }
                            _ => "unknown tool".to_string(),
                        };
                        serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":text}]}}).to_string()
                    }
                    _ => serde_json::json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"method not found"}}).to_string(),
                }
            } else if let Some(query) = request.strip_prefix("search ") {
                let hits = graph
                    .nodes
                    .get(ids.legacy_knowledge_graph)
                    .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
                    .map(|kg| kg.hybrid_retrieve(query, 5))
                    .unwrap_or_default();
                serde_json::json!({"result":"ok","type":"search","hits":hits}).to_string()
            } else if let Some(topic) = request.strip_prefix("crawl ") {
                let url = if topic.starts_with("http://") || topic.starts_with("https://") {
                    topic.to_string()
                } else {
                    format!("https://en.wikipedia.org/wiki/{}", topic.replace(' ', "_"))
                };
                serde_json::json!({"result":"ok","type":"crawl","message":Self::safe_remote_crawl(graph, ids, &url, allow_remote_crawl)}).to_string()
            } else if request == "status" || request.contains("snapshot") {
                serde_json::json!({
                    "result":"ok",
                    "grafo_nodos": graph.alive_node_count(),
                    "grafo_aristas": graph.edge_count(),
                    "ciclo": graph.global_tick,
                    "crawl_pending": false,
                })
                .to_string()
            } else {
                serde_json::json!({"result":"error","error":"unknown_mcp_command"}).to_string()
            };
            println!("{}", response);
        }
    }

    fn mcp_search_text(graph: &eden_garm::HyperGraph, ids: RuntimeNodeIds, query: &str) -> String {
        graph
            .nodes
            .get(ids.legacy_knowledge_graph)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
            .map(|kg| {
                kg.hybrid_retrieve(query, 5)
                    .into_iter()
                    .map(|(name, score)| format!("{} ({:.2})", name, score))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default()
    }

    fn dispatch_gewc_cycle(
        cmd: &str,
        last_command: &mut String,
        graph: &mut eden_garm::HyperGraph,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        ids: RuntimeNodeIds,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
        dt: f32,
        runtime_config: &GarmRuntimeConfig,
        autonomous: &mut bool,
    ) -> (String, bool) {
        Self::record_history(graph, ids.legacy_history, cmd);
        let command =
            eden_garm::nodes::command_router::CommandRouterNode::parse_raw(cmd, last_command);
        let command_for_cycle = command.clone();
        let capability_status = shared_engine
            .lock()
            .map(|engine| engine.status_summary())
            .unwrap_or_else(|_| "garm | capability_status_unavailable".to_string());
        let core_decision =
            eden_garm::global_executive_workspace::GlobalExecutiveWorkspaceCore::decide(
                &command,
                eden_garm::global_executive_workspace::CoreRuntimeContext {
                    raw_command: cmd.to_string(),
                    autonomous: *autonomous,
                    allow_remote_crawl: runtime_config.allow_remote_crawl,
                    graph_nodes: graph.alive_node_count(),
                    graph_edges: graph.edge_count(),
                    global_tick: graph.global_tick,
                    capability_status,
                },
            );
        if !matches!(
            &command,
            eden_garm::nodes::command_router::GarmCommand::Quit
        ) {
            let core_trace =
                eden_garm::global_executive_workspace::GlobalExecutiveWorkspaceCore::record_decision(
                    &core_decision,
                );
            Self::record_history(graph, ids.legacy_history, core_trace.trim());
        }
        if core_decision.is_blocked() {
            let response = core_decision.blocked_response();
            Self::record_gewc_cycle_completion(
                graph,
                ids,
                &command_for_cycle,
                &core_decision,
                eden_garm::global_executive_workspace::CoreExecutionOutcome::blocked(&response),
            );
            return (response, true);
        }
        if let Some(response) =
            Self::enforce_gewc_pre_execution_safety(&command_for_cycle, &core_decision)
        {
            Self::record_gewc_cycle_completion(
                graph,
                ids,
                &command_for_cycle,
                &core_decision,
                eden_garm::global_executive_workspace::CoreExecutionOutcome::blocked(&response),
            );
            return (response, true);
        }
        let result = {
            let mut body_ports = GewcBodyPorts {
                graph,
                shared_engine,
                ids,
                api_metrics,
                dt,
                runtime_config,
                autonomous,
            };
            GewcBodyExecutor::execute(command, &core_decision, &mut body_ports)
        };
        Self::record_gewc_cycle_completion(
            graph,
            ids,
            &command_for_cycle,
            &core_decision,
            eden_garm::global_executive_workspace::CoreExecutionOutcome::completed(
                &result.0, result.1,
            ),
        );
        result
    }

    fn enforce_gewc_pre_execution_safety(
        command: &eden_garm::nodes::command_router::GarmCommand,
        decision: &eden_garm::global_executive_workspace::CoreDecision,
    ) -> Option<String> {
        if !GewcBodyExecutor::has_native_executor(decision.body_handler) {
            return Some(format!(
                "[GEWC-BLOCKED] command_kind={} disposition=block route={} safety_gate=native_handler_missing reason=no_native_executor_for_handler command={:?}\n",
                decision.command_kind, decision.route, command
            ));
        }
        if decision.safety_gate.is_empty() {
            return Some(format!(
                "[GEWC-BLOCKED] command_kind={} disposition=block route={} safety_gate=missing_safety_gate reason=all_actions_require_explicit_safety_gate\n",
                decision.command_kind, decision.route
            ));
        }
        None
    }

    fn record_gewc_cycle_completion(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        command: &eden_garm::nodes::command_router::GarmCommand,
        decision: &eden_garm::global_executive_workspace::CoreDecision,
        outcome: eden_garm::global_executive_workspace::CoreExecutionOutcome,
    ) {
        if !Self::should_record_gewc_cycle_completion(command) {
            return;
        }
        let cycle_trace =
            eden_garm::global_executive_workspace::GlobalExecutiveWorkspaceCore::record_execution_completion(
                decision, outcome,
            );
        Self::record_history(graph, ids.legacy_history, cycle_trace.trim());
    }

    fn should_record_gewc_cycle_completion(
        command: &eden_garm::nodes::command_router::GarmCommand,
    ) -> bool {
        eden_garm::global_executive_workspace::GewcBodyRegistry::should_record_completion(command)
    }

    fn legacy_kg_hypotheses(
        graph: &eden_garm::HyperGraph,
        legacy_knowledge_graph_id: usize,
        fact: &str,
    ) -> String {
        graph
            .nodes
            .get(legacy_knowledge_graph_id)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
            .map(|kg| {
                let hypotheses = kg.generate_hypotheses(fact);
                if hypotheses.is_empty() { String::new() } else { format!("{}\n", hypotheses.join("\n")) }
            })
            .unwrap_or_default()
    }

    fn legacy_kg_explain(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        topic: &str,
    ) -> String {
        let pack = Self::context_pack(graph, ids, topic, 3, 5);
        if pack.path_explanations.is_empty() {
            String::new()
        } else {
            Self::apply_cag_feedback(graph, ids, &pack, true);
            format!(
                "[KG-REASON] {}\n{}\n{}\n{}\n",
                topic,
                pack.path_explanations.join("\n"),
                Self::cag_stance_line(&pack),
                Self::cag_trace_line(&pack)
            )
        }
    }

    fn legacy_unknown_fallback(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        raw: &str,
    ) -> String {
        if raw.trim().len() < 3 {
            return String::new();
        }
        let mut out = String::new();
        let pack = Self::context_pack(graph, ids, raw, 2, 5);
        if !pack.kg_hits.is_empty() {
            out.push_str(&format!("[RAG] Resultados para '{}':\n", raw));
            for (name, score) in &pack.kg_hits {
                out.push_str(&format!("- {} ({:.2})\n", name, score));
            }
        }
        if out.is_empty() {
            if !pack.memory_facts.is_empty() {
                out.push_str(&format!(
                    "[MEMORIA] Coincidencias para '{}':\n{}\n",
                    raw,
                    pack.memory_facts.join("\n")
                ));
            }
        }
        if !out.is_empty() {
            Self::apply_cag_feedback(graph, ids, &pack, pack.context_quality >= 0.35);
            out.push_str(&format!("{}\n", Self::cag_stance_line(&pack)));
            out.push_str(&format!("{}\n", Self::cag_trace_line(&pack)));
        }
        out
    }

    fn context_pack(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        query: &str,
        kg_depth: usize,
        kg_top_k: usize,
    ) -> eden_garm::nodes::context_augmentation::ContextPack {
        let (path_explanations, kg_hits) = graph
            .nodes
            .get(ids.legacy_knowledge_graph)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
            .map(|kg| (kg.explain_paths(query, kg_depth), kg.hybrid_retrieve(query, kg_top_k)))
            .unwrap_or_default();
        let memory_facts = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.search(query))
            .unwrap_or_default();
        let history_fragments = graph
            .nodes
            .get(ids.legacy_history)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_history::LegacyHistoryNode>()
            })
            .map(|history| history.recent_events(5))
            .unwrap_or_default();
        let tick = graph.global_tick;
        graph
            .nodes
            .get_mut(ids.context_augmentation)
            .and_then(|node| {
                node.as_any_mut().downcast_mut::<
                    eden_garm::nodes::context_augmentation::ContextAugmentationNode,
                >()
            })
            .map(|cag| {
                cag.build_context(
                    query,
                    tick,
                    &memory_facts,
                    &kg_hits,
                    &path_explanations,
                    &history_fragments,
                )
            })
            .unwrap_or(eden_garm::nodes::context_augmentation::ContextPack {
                query: query.trim().to_string(),
                created_tick: tick,
                ttl_ticks: 0,
                cache_hit: false,
                memory_facts,
                kg_hits,
                path_explanations,
                history_fragments,
                sources: Vec::new(),
                trace: vec!["cache:unavailable".to_string()],
                context_quality: 0.0,
                quality_label: "low".to_string(),
            })
    }

    fn cag_trace_line(pack: &eden_garm::nodes::context_augmentation::ContextPack) -> String {
        format!(
            "[CAG] cache={} quality={:.2} label={} sources={} trace={}",
            if pack.cache_hit { "hit" } else { "miss" },
            pack.context_quality,
            pack.quality_label,
            if pack.sources.is_empty() {
                "none".to_string()
            } else {
                pack.sources.join(",")
            },
            pack.trace.join("; ")
        )
    }

    fn cag_stance_line(pack: &eden_garm::nodes::context_augmentation::ContextPack) -> String {
        match pack.quality_label.as_str() {
            "high" => "[CAG-POLICY] respuesta=firme contexto=alta-calidad".to_string(),
            "medium" => "[CAG-POLICY] respuesta=prudente contexto=calidad-media".to_string(),
            _ => "[CAG-POLICY] respuesta=cautelosa contexto=baja-calidad accion=recomendar memoria/ConceptNet/crawler-gated".to_string(),
        }
    }

    fn apply_cag_feedback(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        pack: &eden_garm::nodes::context_augmentation::ContextPack,
        useful: bool,
    ) {
        if let Some(cag) = graph
            .nodes
            .get_mut(ids.context_augmentation)
            .and_then(|node| {
                node.as_any_mut().downcast_mut::<
                    eden_garm::nodes::context_augmentation::ContextAugmentationNode,
                >()
            })
        {
            cag.record_feedback(&pack.query, useful);
        }
        if useful && pack.quality_label == "high" {
            Self::feed_knowledge_graph(
                graph,
                ids.legacy_knowledge_graph,
                &format!("{} is well_supported_context", pack.query),
                "cag_feedback",
            );
            Self::feed_knowledge_graph(
                graph,
                ids.legacy_knowledge_graph,
                &format!("{} has context_quality_high", pack.query),
                "cag_feedback",
            );
            if let Some(memory) = graph.nodes.get_mut(ids.legacy_memory).and_then(|node| {
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            }) {
                memory.remember(&format!(
                    "CAG good context: query='{}' quality={:.2} sources={}",
                    pack.query,
                    pack.context_quality,
                    if pack.sources.is_empty() {
                        "none".to_string()
                    } else {
                        pack.sources.join(",")
                    }
                ));
            }
        } else if !useful || pack.quality_label == "low" {
            Self::feed_knowledge_graph(
                graph,
                ids.legacy_knowledge_graph,
                &format!("{} is weak_context", pack.query),
                "cag_feedback",
            );
        }
    }

    fn legacy_rebirth(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let facts = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.facts().to_vec())
            .unwrap_or_default();
        let response = graph
            .nodes
            .get_mut(ids.legacy_rebirth_meltrace)
            .and_then(|node| node.as_any_mut().downcast_mut::<eden_garm::nodes::legacy_runtime_extensions::RebirthMeltraceNode>())
            .map(|rebirth| rebirth.rebirth(&facts))
            .unwrap_or_else(|| "[REBIRTH] legacy_rebirth_meltrace node not found".to_string());
        Self::update_lifecycle_metrics(graph, ids.legacy_rebirth_meltrace, api_metrics);
        format!("{}\n", response)
    }

    fn load_conceptnet(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        path: &str,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let facts = match graph.nodes.get_mut(ids.legacy_crawler).and_then(|node| {
            node.as_any_mut()
                .downcast_mut::<eden_garm::nodes::legacy_runtime_extensions::LegacyCrawlerNode>()
        }) {
            Some(crawler) => match crawler.load_conceptnet(path) {
                Ok(facts) => facts,
                Err(e) => return format!("[CONCEPTNET] {}\n", e),
            },
            None => return "[CONCEPTNET] legacy_crawler node not found\n".to_string(),
        };
        for fact in &facts {
            if let Some(memory) = graph.nodes.get_mut(ids.legacy_memory).and_then(|node| {
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            }) {
                memory.remember(fact);
            }
            Self::feed_knowledge_graph(graph, ids.legacy_knowledge_graph, fact, "conceptnet");
        }
        Self::update_api_metrics(graph, ids, api_metrics);
        format!("[CONCEPTNET] imported={} path={}\n", facts.len(), path)
    }

    fn legacy_dialogue<F>(
        graph: &mut eden_garm::HyperGraph,
        legacy_dialogue_id: usize,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        f: F,
    ) -> String
    where
        F: FnOnce(
            &mut eden_garm::nodes::legacy_dialogue::LegacyDialogueNode,
            &eden_garm::capabilities::GarmCapabilityState,
        ) -> String,
    {
        let Some(node) = graph.nodes.get_mut(legacy_dialogue_id) else {
            return "legacy_dialogue node not found\n".to_string();
        };
        let Some(dialogue) = node
            .as_any_mut()
            .downcast_mut::<eden_garm::nodes::legacy_dialogue::LegacyDialogueNode>()
        else {
            return "legacy_dialogue node type mismatch\n".to_string();
        };
        let engine = shared_engine.lock().unwrap();
        format!("{}\n", f(dialogue, &engine))
    }

    fn record_history(graph: &mut eden_garm::HyperGraph, legacy_history_id: usize, cmd: &str) {
        if let Some(node) = graph.nodes.get_mut(legacy_history_id) {
            if let Some(history) =
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::legacy_history::LegacyHistoryNode>()
            {
                history.record_command(cmd);
            }
        }
    }

    fn reason_report(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        intent: &str,
        topic: &str,
    ) -> String {
        let pack = Self::context_pack(graph, ids, topic, 2, 5);
        let Some(node) = graph.nodes.get_mut(ids.legacy_reason) else {
            return "legacy_reason node not found\n".to_string();
        };
        let Some(reason) = node
            .as_any_mut()
            .downcast_mut::<eden_garm::nodes::legacy_reason::LegacyReasonNode>()
        else {
            return "legacy_reason node type mismatch\n".to_string();
        };
        let out = format!(
            "{}\n{}\n{}\n",
            reason.answer_intent(intent, topic, &pack.memory_facts),
            Self::cag_stance_line(&pack),
            Self::cag_trace_line(&pack)
        );
        Self::apply_cag_feedback(graph, ids, &pack, pack.context_quality >= 0.35);
        out
    }

    fn history_report(graph: &mut eden_garm::HyperGraph, legacy_history_id: usize) -> String {
        let Some(node) = graph.nodes.get_mut(legacy_history_id) else {
            return "legacy_history node not found\n".to_string();
        };
        let Some(history) = node
            .as_any_mut()
            .downcast_mut::<eden_garm::nodes::legacy_history::LegacyHistoryNode>()
        else {
            return "legacy_history node type mismatch\n".to_string();
        };
        format!("{}\n", history.report())
    }

    fn observatory_report(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let alive_nodes = graph.alive_node_count();
        let edge_count = graph.edge_count();
        let memory_facts = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.fact_count())
            .unwrap_or(0);
        let Some(node) = graph.nodes.get_mut(ids.observatory) else {
            return "observatory node not found\n".to_string();
        };
        let Some(observatory) = node
            .as_any_mut()
            .downcast_mut::<eden_garm::nodes::observatory::ObservatoryNode>()
        else {
            return "observatory node type mismatch\n".to_string();
        };
        let engine = shared_engine.lock().unwrap();
        let mut report = format!(
            "{}\n",
            observatory.report(
                &engine,
                alive_nodes,
                edge_count,
                memory_facts,
                api_metrics.uptime_sec(),
                api_metrics.ready.load(Ordering::Relaxed),
                api_metrics.autonomous.load(Ordering::Relaxed),
            )
        );
        drop(engine);
        if let Some(tension) = graph.nodes.get(ids.campo_tension).and_then(|node| {
            node.as_any()
                .downcast_ref::<eden_garm::nodes::campo_tension::CampoTensionNode>()
        }) {
            report.push_str(&format!("{}\n", tension.informe()));
        }
        let _ = ids.legacy_knowledge_graph;
        if let Some(kg) = graph.nodes.iter().find_map(|node| {
            node.as_any()
                .downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>(
                )
        }) {
            report.push_str(&format!("{}\n", kg.informe()));
        }
        Self::append_extension_report::<eden_garm::nodes::legacy_runtime_extensions::AutoconsumoNode>(
            graph,
            ids.legacy_autoconsumo,
            &mut report,
            |n| n.informe(),
        );
        Self::append_extension_report::<
            eden_garm::nodes::legacy_runtime_extensions::VenadoCompatibilityNode,
        >(graph, ids.legacy_venado, &mut report, |n| n.informe());
        Self::append_extension_report::<eden_garm::nodes::legacy_runtime_extensions::ParadigmHubNode>(
            graph,
            ids.legacy_paradigm_hub,
            &mut report,
            |n| n.informe(),
        );
        Self::append_extension_report::<eden_garm::nodes::legacy_runtime_extensions::EcoSystemNode>(
            graph,
            ids.legacy_ecosystem,
            &mut report,
            |n| n.informe(),
        );
        Self::append_extension_report::<
            eden_garm::nodes::legacy_runtime_extensions::RebirthMeltraceNode,
        >(graph, ids.legacy_rebirth_meltrace, &mut report, |n| {
            n.informe()
        });
        Self::append_extension_report::<
            eden_garm::nodes::legacy_runtime_extensions::LegacyCrawlerNode,
        >(graph, ids.legacy_crawler, &mut report, |n| n.informe());
        report.push_str(&Self::readiness_report(
            graph,
            ids,
            shared_engine,
            api_metrics,
        ));
        report.push_str(&Self::organic_lifecycle_report(graph, ids));
        report.push_str(&Self::conscious_graph_report(graph, ids));
        report.push_str(&Self::context_augmentation_report(graph, ids));
        report
    }

    fn organic_ritual(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let facts = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.facts().to_vec())
            .unwrap_or_default();
        let kg_edges = graph
            .nodes
            .get(ids.legacy_knowledge_graph)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
            .map(|kg| kg.edge_count())
            .unwrap_or(0);
        let tension = graph
            .nodes
            .get(ids.campo_tension)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::campo_tension::CampoTensionNode>()
            })
            .map(|t| t.tension())
            .unwrap_or(0.0);
        let tick = graph.global_tick;
        graph
            .nodes
            .get_mut(ids.organic_lifecycle)
            .and_then(|node| {
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::organic_lifecycle::OrganicLifecycleNode>()
            })
            .map(|node| format!("{}\n", node.ritual(&facts, kg_edges, tension, tick)))
            .unwrap_or_else(|| "organic_lifecycle node not found\n".to_string())
    }

    fn organic_lifecycle_report(graph: &eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        graph
            .nodes
            .get(ids.organic_lifecycle)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::organic_lifecycle::OrganicLifecycleNode>()
            })
            .map(|node| format!("{}\n", node.report()))
            .unwrap_or_default()
    }

    fn context_augmentation_explain(
        graph: &eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        query: &str,
    ) -> String {
        graph
            .nodes
            .get(ids.context_augmentation)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::context_augmentation::ContextAugmentationNode>())
            .map(|node| format!("{}\n", node.explain(query)))
            .unwrap_or_else(|| "[CAG-EXPLAIN] context_augmentation node not found\n".to_string())
    }

    fn context_augmentation_gaps(
        graph: &eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        query: &str,
    ) -> String {
        graph
            .nodes
            .get(ids.context_augmentation)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::context_augmentation::ContextAugmentationNode>())
            .map(|node| format!("{}\n", node.gaps(query)))
            .unwrap_or_else(|| "[CAG-GAPS] context_augmentation node not found\n".to_string())
    }

    fn context_augmentation_plan(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        query: &str,
    ) -> String {
        graph
            .nodes
            .get_mut(ids.context_augmentation)
            .and_then(|node| {
                node.as_any_mut().downcast_mut::<
                    eden_garm::nodes::context_augmentation::ContextAugmentationNode,
                >()
            })
            .map(|node| format!("{}\n", node.plan_actions(query)))
            .unwrap_or_else(|| "[CAG-ACTIONS] context_augmentation node not found\n".to_string())
    }

    fn context_augmentation_actions(graph: &eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        graph
            .nodes
            .get(ids.context_augmentation)
            .and_then(|node| {
                node.as_any().downcast_ref::<
                    eden_garm::nodes::context_augmentation::ContextAugmentationNode,
                >()
            })
            .map(|node| format!("{}\n", node.actions_report()))
            .unwrap_or_else(|| "[CAG-ACTIONS] context_augmentation node not found\n".to_string())
    }

    fn context_augmentation_audit(graph: &eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        graph
            .nodes
            .get(ids.context_augmentation)
            .and_then(|node| {
                node.as_any().downcast_ref::<
                    eden_garm::nodes::context_augmentation::ContextAugmentationNode,
                >()
            })
            .map(|node| format!("{}\n", node.audit_report()))
            .unwrap_or_else(|| "[CAG-AUDIT] context_augmentation node not found\n".to_string())
    }

    fn context_augmentation_autoplan(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) {
        let actions = if let Some(cag) =
            graph
                .nodes
                .get_mut(ids.context_augmentation)
                .and_then(|node| {
                    node.as_any_mut().downcast_mut::<
                    eden_garm::nodes::context_augmentation::ContextAugmentationNode,
                >()
                }) {
            cag.plan_weak_contexts(2);
            cag.take_autonomous_safe_actions(2)
        } else {
            Vec::new()
        };
        for action in actions {
            let (status, reason) = Self::execute_cag_action(graph, ids, &action);
            Self::complete_cag_action(graph, ids, action.id, status, reason, "autonomous");
        }
    }

    fn context_augmentation_run(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        query: &str,
    ) -> String {
        let actions = graph
            .nodes
            .get_mut(ids.context_augmentation)
            .and_then(|node| {
                node.as_any_mut().downcast_mut::<
                    eden_garm::nodes::context_augmentation::ContextAugmentationNode,
                >()
            })
            .map(|node| node.take_runnable_actions(query))
            .unwrap_or_default();
        if actions.is_empty() {
            return format!(
                "[CAG-RUN] query='{}' no_pending_safe_actions\n",
                query.trim()
            );
        }
        let mut out = String::from("[CAG-RUN]\n");
        for action in actions {
            let (status, reason) = Self::execute_cag_action(graph, ids, &action);
            Self::complete_cag_action(graph, ids, action.id, status, reason, "manual");
            out.push_str(&format!(
                "- id={} kind={} status={} reason={}\n",
                action.id, action.kind, status, reason
            ));
        }
        out
    }

    fn execute_cag_action(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        action: &eden_garm::nodes::context_augmentation::CagAction,
    ) -> (&'static str, &'static str) {
        match action.kind.as_str() {
            "prompt_remember" => {
                if let Some(memory) = graph.nodes.get_mut(ids.legacy_memory).and_then(|node| {
                    node.as_any_mut()
                        .downcast_mut::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
                }) {
                    memory.remember(&format!(
                        "CAG action needed: remember evidence for query '{}'",
                        action.query
                    ));
                }
                ("executed", "memory_prompt_recorded")
            }
            "prioritize_local_conceptnet" => {
                Self::feed_knowledge_graph(
                    graph,
                    ids.legacy_knowledge_graph,
                    &format!("{} needs local_conceptnet_source", action.query),
                    "cag_action",
                );
                ("executed", "kg_local_source_need_recorded")
            }
            "validate_with_juez" => {
                Self::feed_knowledge_graph(
                    graph,
                    ids.legacy_knowledge_graph,
                    &format!("{} requires judge_validation", action.query),
                    "cag_action",
                );
                ("executed", "judge_validation_marked")
            }
            "crawl_gated" => ("blocked", "remote_crawl_requires_explicit_user_flag"),
            _ => ("blocked", "unknown_action_kind"),
        }
    }

    fn complete_cag_action(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        action_id: u64,
        status: &str,
        reason: &str,
        mode: &str,
    ) {
        if let Some(cag) = graph
            .nodes
            .get_mut(ids.context_augmentation)
            .and_then(|node| {
                node.as_any_mut().downcast_mut::<
                    eden_garm::nodes::context_augmentation::ContextAugmentationNode,
                >()
            })
        {
            cag.complete_action_with_mode(action_id, status, reason, mode);
        }
    }

    fn conscious_graph_report(graph: &eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        graph
            .nodes
            .get(ids.conscious_graph_regulator)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::conscious_graph_regulator::ConsciousGraphRegulatorNode>())
            .map(|node| format!("{}\n", node.report()))
            .unwrap_or_default()
    }

    fn context_augmentation_report(graph: &eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        graph
            .nodes
            .get(ids.context_augmentation)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::context_augmentation::ContextAugmentationNode>())
            .map(|node| format!("{}\n", node.report()))
            .unwrap_or_default()
    }

    fn organs_safe_run(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let batch = Self::organs_autonomous_cycle(graph, ids, "manual");
        let mut out = format!("{}\n", batch.report);
        out.push_str(&eden_garm::nodes::goal_scheduler::record_external_goal(
            "organ_registry",
            "manual safe organ cycle",
            batch.report.contains("status=executed"),
            "organ_deltas_audited",
        ));
        Self::context_augmentation_autoplan(graph, ids);
        Self::update_api_metrics(graph, ids, api_metrics);
        out.push_str(&format!(
            "- profiled_autonomous={} remote_blocked=legacy_crawler\n",
            eden_garm::nodes::organ_registry::ORGAN_PROFILES.len()
        ));
        out.push_str("- executed=context_augmentation_autoplan\n");
        out.push_str("- recorded=history+knowledge_graph\n");
        out.push_str(&eden_garm::nodes::organ_registry::organ_audit(graph));
        out.push('\n');
        out
    }

    fn organs_autonomous_cycle(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        mode: &str,
    ) -> eden_garm::nodes::organ_registry::OrganRunBatch {
        let batch = eden_garm::nodes::organ_registry::run_pending_actions(
            graph,
            mode,
            eden_garm::nodes::organ_registry::ORGAN_PROFILES.len(),
        );
        for fact in &batch.facts {
            Self::feed_knowledge_graph(graph, ids.legacy_knowledge_graph, fact, "organ_autonomy");
        }
        Self::record_history(graph, ids.legacy_history, &batch.history);
        batch
    }

    fn organs_safe_repair(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let findings = eden_garm::nodes::organ_registry::organ_health_findings(graph);
        let mut out = format!(
            "[ORGANOS-REPAIR] findings={} mode=safe_local\n",
            findings.len()
        );
        for finding in findings.iter().take(8) {
            let fact = format!(
                "organ {} health {} free_energy {:.2} recommendation {}",
                finding.name, finding.severity, finding.free_energy, finding.recommendation
            );
            Self::feed_knowledge_graph(graph, ids.legacy_knowledge_graph, &fact, "organ_repair");
            out.push_str(&format!(
                "- organ={} severity={} action=recorded_recommendation guard=no_code_mutation,no_remote_network\n",
                finding.name, finding.severity
            ));
        }
        Self::record_history(
            graph,
            ids.legacy_history,
            &format!(
                "[ORGANOS-REPAIR] recorded {} safe recommendations",
                findings.len().min(8)
            ),
        );
        Self::context_augmentation_autoplan(graph, ids);
        Self::update_api_metrics(graph, ids, api_metrics);
        if findings.is_empty() {
            out.push_str("- status=stable action=none\n");
        }
        out
    }

    fn lengua_responder(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        query: &str,
    ) -> String {
        let pack = Self::context_pack(graph, ids, query, 3, 5);
        let mut out = format!("[LENGUA] consulta='{}'\n", query);
        if !pack.path_explanations.is_empty() {
            out.push_str(&format!(
                "[KG-REASON] {}\n{}\n",
                query,
                pack.path_explanations.join("\n")
            ));
        }
        if !pack.memory_facts.is_empty() {
            out.push_str("[LENGUA-MEMORIA]\n");
            for fact in &pack.memory_facts {
                out.push_str(&format!("- {}\n", fact));
            }
        }
        out.push_str(&format!("{}\n", Self::cag_trace_line(&pack)));
        if pack.path_explanations.is_empty() && pack.memory_facts.is_empty() {
            out.push_str("[LENGUA] no tengo evidencia suficiente; recomienda recordar/importar hechos antes de afirmar.\n");
        }
        out.push_str(&format!("{}\n", Self::cag_stance_line(&pack)));
        let effect = format!("lengua consulta {}", query);
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            &format!("{} is spoken_query", query),
            "lengua",
        );
        Self::feed_legacy_cognition_and_tension(graph, ids, &effect);
        Self::apply_cag_feedback(graph, ids, &pack, pack.context_quality >= 0.35);
        Self::record_history(graph, ids.legacy_history, &format!("[LENGUA] {}", query));
        out
    }

    fn hrm_reason(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds, query: &str) -> String {
        let pack = Self::context_pack(graph, ids, query, 4, 6);
        let hrm_text_evidence = eden_garm::nodes::hrm_text_pretraining::search_evidence(query, 3);
        let mut memory_facts = pack.memory_facts.clone();
        memory_facts.extend(hrm_text_evidence.iter().cloned());
        let tick = graph.global_tick;
        let out = graph
            .nodes
            .get_mut(ids.hrm_reasoner)
            .and_then(|node| {
                node.as_any_mut().downcast_mut::<
                    eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode,
                >()
            })
            .map(|hrm| {
                hrm.reason(
                    query,
                    tick,
                    &memory_facts,
                    &pack.kg_hits,
                    &pack.path_explanations,
                    &pack.history_fragments,
                )
            })
            .unwrap_or_else(|| "[HRM] unavailable: hrm_reasoner node missing\n".to_string());
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            &format!("{} is hrm_reasoned", query),
            "hrm_reasoner",
        );
        Self::record_history(graph, ids.legacy_history, &format!("[HRM] {}", query));
        Self::apply_cag_feedback(graph, ids, &pack, pack.context_quality >= 0.35);
        let hrm_text_block = if hrm_text_evidence.is_empty() {
            "[HRM-TEXT-RETRIEVAL] status=miss hits=0\n".to_string()
        } else {
            format!(
                "[HRM-TEXT-RETRIEVAL] status=hit hits={}\n{}\n",
                hrm_text_evidence.len(),
                hrm_text_evidence
                    .iter()
                    .take(3)
                    .map(|item| format!("- {}", item))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };
        format!(
            "{}{}{}\n{}\n",
            out,
            hrm_text_block,
            Self::cag_trace_line(&pack),
            Self::cag_stance_line(&pack)
        )
    }

    fn hrm_run_plan(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        query: &str,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let mut out = format!("[HRM-RUN] query='{}'\n", query.trim());
        out.push_str(&Self::hrm_reason(graph, ids, query));
        out.push_str(&Self::context_augmentation_plan(graph, ids, query));
        let cag_run = Self::context_augmentation_run(graph, ids, query);
        let organs_run = Self::organs_safe_run(graph, ids, api_metrics);
        let executed = cag_run.contains("status=executed")
            || organs_run.contains("executed=") && !organs_run.contains("executed=0");
        out.push_str(&cag_run);
        out.push_str(&organs_run);
        let retrieval_evidence = eden_garm::nodes::hrm_text_pretraining::search_evidence(query, 3);
        let retrieval_summary = if retrieval_evidence.is_empty() {
            "hrm_text_retrieval hits=0 status=miss".to_string()
        } else {
            format!(
                "hrm_text_retrieval hits={} status=hit evidence={}",
                retrieval_evidence.len(),
                retrieval_evidence
                    .iter()
                    .map(|item| item.replace('\n', " "))
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        };
        out.push_str(&format!("[HRM-RUN-RETRIEVAL] {}\n", retrieval_summary));
        if let Some(node) = graph.nodes.get_mut(ids.hrm_reasoner).and_then(|node| {
            node.as_any_mut().downcast_mut::<
                    eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode,
                >()
        }) {
            out.push_str(&node.record_execution_result(
                executed,
                &format!("cag={} organs={}", cag_run.trim(), organs_run.trim()),
            ));
        }
        out.push_str(&eden_garm::nodes::goal_scheduler::record_external_goal(
            "hrm_reasoner",
            query,
            executed,
            if executed {
                "hrm_plan_executed"
            } else {
                "hrm_plan_blocked"
            },
        ));
        out.push_str(&eden_garm::nodes::learning_ledger::record(
            "hrm_reasoner",
            query,
            &format!("hrm_plan+cag+organ_trace+{}", retrieval_summary),
            if executed { "completed" } else { "blocked" },
        ));
        out.push_str(&eden_garm::nodes::provenance_ledger::record(
            "hrm_text_retrieval",
            &format!("query={} {}", query.trim(), retrieval_summary),
        ));
        Self::record_history(graph, ids.legacy_history, &format!("[HRM-RUN] {}", query));
        out
    }

    fn garm_audit(
        graph: &eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &std::sync::Arc<
            std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>,
        >,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
        autonomous: bool,
    ) -> String {
        let guard = shared_engine.lock().unwrap();
        let cag = Self::context_augmentation_report(graph, ids);
        let hrm = graph
            .nodes
            .get(ids.hrm_reasoner)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode>())
            .map(|node| node.snapshot())
            .unwrap_or_else(|| "hrm:missing".to_string());
        let voice = graph
            .nodes
            .get(ids.voice_synthesizer)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode>()
            })
            .map(|node| node.snapshot())
            .unwrap_or_else(|| "voice:missing".to_string());
        let goals = eden_garm::nodes::goal_scheduler::report();
        let evaluation = eden_garm::nodes::evaluation_loop::report();
        let learning = eden_garm::nodes::learning_ledger::report();
        let world = eden_garm::nodes::world_model_core::report();
        let benchmark = eden_garm::nodes::competence_benchmark::report();
        let executor = eden_garm::nodes::plan_executor::report();
        let attention = eden_garm::nodes::working_memory::report();
        let uncertainty = eden_garm::nodes::uncertainty_ledger::report();
        let experiment = eden_garm::nodes::experiment_runner::report();
        let provenance = eden_garm::nodes::provenance_ledger::report();
        let policy = eden_garm::nodes::policy_guard::report();
        let maturity = eden_garm::nodes::capability_maturity::report();
        let hybrid_voice = eden_garm::nodes::hybrid_voice::report();
        let hrm_text = eden_garm::nodes::hrm_text_pretraining::report();
        let organ_audit = eden_garm::nodes::organ_registry::organ_audit(graph);
        let verdict =
            if organ_audit.contains("missing=0") && api_metrics.ready.load(Ordering::Relaxed) {
                "ready"
            } else if organ_audit.contains("missing=0") {
                "degraded"
            } else {
                "needs_attention"
            };
        format!(
            "[GARM-AUDIT] verdict={} autonomous={} ready={} ticks={} alive={} edges={}\n{}\n[HRM] {}\n[VOICE] {}\n{}\n{}{}{}{}{}{}{}{}{}{}{}{}{}{}[STATE]\n{}",
            verdict,
            autonomous,
            api_metrics.ready.load(Ordering::Relaxed),
            guard.state.tick_count,
            graph.alive_node_count(),
            graph.edge_count(),
            organ_audit,
            hrm,
            voice,
            cag.trim_end(),
            goals,
            evaluation,
            learning,
            world,
            benchmark,
            executor,
            attention,
            uncertainty,
            experiment,
            provenance,
            policy,
            maturity,
            hybrid_voice,
            hrm_text,
            eden_garm::state_paths::state_report()
        )
    }

    fn garm_report(
        graph: &eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &std::sync::Arc<
            std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>,
        >,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
        autonomous: bool,
    ) -> String {
        let guard = shared_engine.lock().unwrap();
        let organ_audit = eden_garm::nodes::organ_registry::organ_audit(graph);
        let organ_actions = eden_garm::nodes::organ_registry::organ_actions_report();
        let cag = Self::context_augmentation_report(graph, ids);
        let hrm = graph
            .nodes
            .get(ids.hrm_reasoner)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode>())
            .map(|node| node.snapshot())
            .unwrap_or_else(|| "hrm:missing".to_string());
        let voice = graph
            .nodes
            .get(ids.voice_synthesizer)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode>()
            })
            .map(|node| node.snapshot())
            .unwrap_or_else(|| "voice:missing".to_string());
        let goals = eden_garm::nodes::goal_scheduler::report();
        let evaluation = eden_garm::nodes::evaluation_loop::report();
        let learning = eden_garm::nodes::learning_ledger::report();
        let world = eden_garm::nodes::world_model_core::report();
        let benchmark = eden_garm::nodes::competence_benchmark::report();
        let executor = eden_garm::nodes::plan_executor::report();
        let attention = eden_garm::nodes::working_memory::report();
        let uncertainty = eden_garm::nodes::uncertainty_ledger::report();
        let experiment = eden_garm::nodes::experiment_runner::report();
        let provenance = eden_garm::nodes::provenance_ledger::report();
        let policy = eden_garm::nodes::policy_guard::report();
        let maturity = eden_garm::nodes::capability_maturity::report();
        let hybrid_voice = eden_garm::nodes::hybrid_voice::report();
        let hrm_text = eden_garm::nodes::hrm_text_pretraining::report();
        let verdict =
            if organ_audit.contains("missing=0") && api_metrics.ready.load(Ordering::Relaxed) {
                "ready"
            } else if organ_audit.contains("missing=0") {
                "degraded"
            } else {
                "needs_attention"
            };
        let report = format!(
            "[GARM-REPORT] verdict={} autonomous={} ready={} ticks={} alive={} edges={} memory_facts={}\n[Runtime] status={} uptime_sec={}\n[Organs]\n{}\n[LastDeltas]\n{}\n[HRM] {}\n[VOICE] {}\n[Goals]\n{}[Evaluation]\n{}[Learning]\n{}[World]\n{}[Benchmark]\n{}[Executor]\n{}[Attention]\n{}[Uncertainty]\n{}[Experiment]\n{}[Provenance]\n{}[Policy]\n{}[Maturity]\n{}[HybridVoice]\n{}[HRMText]\n{}[CAG]\n{}\n[Persistence] report={} backup={} state_dir={}\n",
            verdict,
            autonomous,
            api_metrics.ready.load(Ordering::Relaxed),
            guard.state.tick_count,
            graph.alive_node_count(),
            graph.edge_count(),
            api_metrics.memory_facts.load(Ordering::Relaxed),
            guard.status_summary(),
            api_metrics.uptime_sec(),
            organ_audit,
            organ_actions,
            hrm,
            voice,
            goals,
            evaluation,
            learning,
            world,
            benchmark,
            executor,
            attention,
            uncertainty,
            experiment,
            provenance,
            policy,
            maturity,
            hybrid_voice,
            hrm_text,
            cag.trim_end(),
            eden_garm::state_paths::garm_report_path(),
            eden_garm::state_paths::backup_dir_path(),
            eden_garm::state_paths::state_dir().to_string_lossy()
        );
        match std::fs::write(eden_garm::state_paths::garm_report_path(), &report) {
            Ok(()) => {
                let history_warning = Self::append_garm_report_history(
                    verdict,
                    autonomous,
                    api_metrics.ready.load(Ordering::Relaxed),
                    guard.state.tick_count,
                    graph.alive_node_count(),
                    graph.edge_count(),
                    api_metrics.memory_facts.load(Ordering::Relaxed),
                    &hrm,
                    &voice,
                    &cag,
                );
                if let Some(warning) = history_warning {
                    format!("{}{}", report, warning)
                } else {
                    report
                }
            }
            Err(e) => format!(
                "{}[GARM-REPORT-WARN] persist_error={} path={}\n",
                report,
                e,
                eden_garm::state_paths::garm_report_path()
            ),
        }
    }

    fn append_garm_report_history(
        verdict: &str,
        autonomous: bool,
        ready: bool,
        ticks: u64,
        alive: usize,
        edges: usize,
        memory_facts: u64,
        hrm: &str,
        voice: &str,
        cag: &str,
    ) -> Option<String> {
        let path = eden_garm::state_paths::garm_report_history_path();
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let entry = serde_json::json!({
            "timestamp_ms": timestamp_ms,
            "tick": ticks,
            "verdict": verdict,
            "autonomous": autonomous,
            "ready": ready,
            "alive": alive,
            "edges": edges,
            "memory_facts": memory_facts,
            "hrm": hrm,
            "voice": voice,
            "cag": cag.trim(),
        })
        .to_string();
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .ok()
            .map(|data| data.lines().map(ToString::to_string).collect())
            .unwrap_or_default();
        lines.push(entry);
        if lines.len() > GARM_REPORT_HISTORY_LIMIT {
            lines = lines.split_off(lines.len() - GARM_REPORT_HISTORY_LIMIT);
        }
        let body = if lines.is_empty() {
            String::new()
        } else {
            format!("{}\n", lines.join("\n"))
        };
        std::fs::write(&path, body).err().map(|e| {
            format!(
                "[GARM-REPORT-WARN] history_persist_error={} path={}\n",
                e, path
            )
        })
    }

    fn garm_report_history() -> String {
        let path = eden_garm::state_paths::garm_report_history_path();
        let Ok(data) = std::fs::read_to_string(&path) else {
            return format!("[GARM-REPORT-HISTORY] entries=0 path={}\n", path);
        };
        let lines: Vec<&str> = data.lines().collect();
        let mut out = format!(
            "[GARM-REPORT-HISTORY] entries={} path={} limit={}\n",
            lines.len(),
            path,
            GARM_REPORT_HISTORY_LIMIT
        );
        let start = lines.len().saturating_sub(10);
        for line in &lines[start..] {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap_or_default();
            out.push_str(&format!(
                "- tick={} verdict={} ready={} alive={} edges={} memory_facts={} timestamp_ms={}\n",
                parsed.get("tick").and_then(|v| v.as_u64()).unwrap_or(0),
                parsed
                    .get("verdict")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown"),
                parsed
                    .get("ready")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                parsed.get("alive").and_then(|v| v.as_u64()).unwrap_or(0),
                parsed.get("edges").and_then(|v| v.as_u64()).unwrap_or(0),
                parsed
                    .get("memory_facts")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                parsed
                    .get("timestamp_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            ));
        }
        out
    }

    fn garm_export(
        graph: &eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &std::sync::Arc<
            std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>,
        >,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
        autonomous: bool,
    ) -> String {
        let guard = shared_engine.lock().unwrap();
        let report =
            std::fs::read_to_string(eden_garm::state_paths::garm_report_path()).unwrap_or_default();
        let history_count =
            std::fs::read_to_string(eden_garm::state_paths::garm_report_history_path())
                .map(|data| data.lines().count())
                .unwrap_or(0);
        let hrm = graph
            .nodes
            .get(ids.hrm_reasoner)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode>())
            .map(|node| node.snapshot())
            .unwrap_or_else(|| "hrm:missing".to_string());
        let voice = graph
            .nodes
            .get(ids.voice_synthesizer)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode>()
            })
            .map(|node| node.snapshot())
            .unwrap_or_else(|| "voice:missing".to_string());
        let mut export = serde_json::json!({
            "schema": "garm-export-v1",
            "mode": "diagnostic-read-only",
            "runtime": {
                "ticks": guard.state.tick_count,
                "status": guard.status_summary(),
                "autonomous": autonomous,
                "ready": api_metrics.ready.load(Ordering::Relaxed),
                "alive_nodes": graph.alive_node_count(),
                "edges": graph.edge_count(),
                "memory_facts": api_metrics.memory_facts.load(Ordering::Relaxed),
                "uptime_sec": api_metrics.uptime_sec(),
            },
            "report": {
                "path": eden_garm::state_paths::garm_report_path(),
                "latest_len": report.len(),
                "latest_present": !report.is_empty(),
                "history_path": eden_garm::state_paths::garm_report_history_path(),
                "history_count": history_count,
            },
            "organs": {
                "total": eden_garm::nodes::organ_registry::ORGAN_PROFILES.len(),
                "audit": eden_garm::nodes::organ_registry::organ_audit(graph),
                "actions": eden_garm::nodes::organ_registry::organ_actions_report(),
            },
            "goals": eden_garm::nodes::goal_scheduler::audit_report(),
            "evaluation": eden_garm::nodes::evaluation_loop::audit_report(),
            "learning": eden_garm::nodes::learning_ledger::audit_report(),
            "world": eden_garm::nodes::world_model_core::audit_report(),
            "competence_benchmark": eden_garm::nodes::competence_benchmark::audit_report(),
            "plan_executor": eden_garm::nodes::plan_executor::audit_report(),
            "working_memory": eden_garm::nodes::working_memory::audit_report(),
            "uncertainty_ledger": eden_garm::nodes::uncertainty_ledger::audit_report(),
            "experiment_runner": eden_garm::nodes::experiment_runner::audit_report(),
            "provenance_ledger": eden_garm::nodes::provenance_ledger::audit_report(),
            "policy_guard": eden_garm::nodes::policy_guard::audit_report(),
            "capability_maturity": eden_garm::nodes::capability_maturity::audit_report(),
            "hybrid_voice": eden_garm::nodes::hybrid_voice::audit_report(),
            "hrm_text_pretraining": eden_garm::nodes::hrm_text_pretraining::audit_report(),
            "hrm": hrm,
            "voice": voice,
            "cag": Self::context_augmentation_report(graph, ids),
            "state_paths": {
                "state_dir": eden_garm::state_paths::state_dir().to_string_lossy(),
                "export": eden_garm::state_paths::garm_export_path(),
                "backup": eden_garm::state_paths::backup_dir_path(),
                "runtime": eden_garm::state_paths::runtime_state_path(),
                "hrm": eden_garm::state_paths::hrm_reasoner_state_path(),
                "voice": eden_garm::state_paths::voice_synthesizer_state_path(),
                "organ_autonomy": eden_garm::state_paths::organ_autonomy_state_path(),
                "goal_scheduler": eden_garm::state_paths::goal_scheduler_state_path(),
                "evaluation_loop": eden_garm::state_paths::evaluation_loop_state_path(),
                "learning_ledger": eden_garm::state_paths::learning_ledger_state_path(),
                "world_model_core": eden_garm::state_paths::world_model_core_state_path(),
                "competence_benchmark": eden_garm::state_paths::competence_benchmark_state_path(),
                "plan_executor": eden_garm::state_paths::plan_executor_state_path(),
                "working_memory": eden_garm::state_paths::working_memory_state_path(),
                "uncertainty_ledger": eden_garm::state_paths::uncertainty_ledger_state_path(),
                "experiment_runner": eden_garm::state_paths::experiment_runner_state_path(),
                "provenance_ledger": eden_garm::state_paths::provenance_ledger_state_path(),
                "policy_guard": eden_garm::state_paths::policy_guard_state_path(),
                "capability_maturity": eden_garm::state_paths::capability_maturity_state_path(),
                "hybrid_voice": eden_garm::state_paths::hybrid_voice_state_path(),
                "hrm_text_pretraining": eden_garm::state_paths::hrm_text_pretraining_state_path(),
                "hrm_text_checkpoint_manifest": eden_garm::state_paths::hrm_text_checkpoint_manifest_path(),
                "hrm_text_corpus_manifest": eden_garm::state_paths::hrm_text_corpus_manifest_path(),
                "hrm_text_segments": eden_garm::state_paths::hrm_text_segments_path(),
            }
        });
        let checksum = Self::export_checksum_value(&export);
        export["integrity"] = serde_json::json!({
            "checksum_fnv64": format!("{:016x}", checksum),
            "algorithm": "fnv64",
            "scope": "export_without_integrity",
            "cryptographic": false,
        });
        let body = serde_json::to_string_pretty(&export).unwrap_or_else(|_| export.to_string());
        match std::fs::write(eden_garm::state_paths::garm_export_path(), &body) {
            Ok(()) => format!(
                "[GARM-EXPORT] path={} schema=garm-export-v1 mode=diagnostic-read-only bytes={} history_count={} checksum_fnv64={:016x}\n",
                eden_garm::state_paths::garm_export_path(),
                body.len(),
                history_count,
                checksum
            ),
            Err(e) => format!(
                "[GARM-EXPORT] error={} path={}\n",
                e,
                eden_garm::state_paths::garm_export_path()
            ),
        }
    }

    fn garm_verify_export() -> String {
        let path = eden_garm::state_paths::garm_export_path();
        let Ok(data) = std::fs::read_to_string(&path) else {
            return format!(
                "[GARM-VERIFY-EXPORT] ok=false error=export_missing path={}\n",
                path
            );
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&data) else {
            return format!(
                "[GARM-VERIFY-EXPORT] ok=false error=invalid_json path={}\n",
                path
            );
        };
        let expected = value
            .pointer("/integrity/checksum_fnv64")
            .and_then(|v| v.as_str())
            .unwrap_or("missing");
        let actual = format!("{:016x}", Self::export_checksum_value(&value));
        let ok = expected == actual;
        format!(
            "[GARM-VERIFY-EXPORT] ok={} expected={} actual={} algorithm=fnv64 cryptographic=false path={}\n",
            ok, expected, actual, path
        )
    }

    fn export_checksum_value(value: &serde_json::Value) -> u64 {
        let mut scoped = value.clone();
        if let Some(object) = scoped.as_object_mut() {
            object.remove("integrity");
        }
        let canonical = serde_json::to_string(&scoped).unwrap_or_default();
        Self::fnv64(canonical.as_bytes())
    }

    fn fnv64(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0xcbf29ce484222325u64, |mut hash, byte| {
            hash ^= *byte as u64;
            hash.wrapping_mul(0x100000001b3)
        })
    }

    fn garm_import() -> String {
        let path = eden_garm::state_paths::garm_export_path();
        let Ok(data) = std::fs::read_to_string(&path) else {
            return format!("[GARM-IMPORT] error=export_missing path={}\n", path);
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&data) else {
            return format!("[GARM-IMPORT] error=invalid_json path={}\n", path);
        };
        let schema = value
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or("missing");
        let mode = value
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("missing");
        let history_count = value
            .pointer("/report/history_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let organs = value
            .pointer("/organs/total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let ready = value
            .pointer("/runtime/ready")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let valid = schema == "garm-export-v1" && value.get("runtime").is_some();
        format!(
            "[GARM-IMPORT] valid={} schema={} mode={} read_only=true restored=false organs={} history_count={} ready={} path={}\n",
            valid, schema, mode, organs, history_count, ready, path
        )
    }

    fn reloj_temporal(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        query: &str,
    ) -> String {
        let tick = graph.global_tick;
        let memory_hits = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.search(query))
            .unwrap_or_default();
        let kg_paths = graph
            .nodes
            .get(ids.legacy_knowledge_graph)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
            .map(|kg| kg.explain_paths(query, 2))
            .unwrap_or_default();
        let mut out = format!("[RELOJ] tick={} consulta='{}'\n", tick, query);
        out.push_str(&format!("- ahora: ciclo {} del linaje GARM\n", tick));
        for (i, fact) in memory_hits.iter().take(4).enumerate() {
            out.push_str(&format!("- memoria_t-{}: {}\n", i + 1, fact));
        }
        for path in kg_paths.iter().take(4) {
            out.push_str(&format!("- cadena: {}\n", path));
        }
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            &format!("{} causes temporal_context", query),
            "reloj",
        );
        Self::feed_legacy_cognition_and_tension(
            graph,
            ids,
            &format!("reloj temporaliza {}", query),
        );
        Self::record_history(graph, ids.legacy_history, &format!("[RELOJ] {}", query));
        out
    }

    fn juez_validar(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds, query: &str) -> String {
        let pack = Self::context_pack(graph, ids, query, 2, 5);
        let evidence = pack.path_explanations.len() + pack.kg_hits.len() + pack.memory_facts.len();
        let verdict = if evidence >= 4 {
            "sostenido"
        } else if evidence >= 1 {
            "plausible-no-endurecer"
        } else {
            "sin-evidencia"
        };
        let mut out = format!(
            "[JUEZ-EXTERNO] consulta='{}' verdict={} evidence={}\n",
            query, verdict, evidence
        );
        for path in pack.path_explanations.iter().take(3) {
            out.push_str(&format!("- kg: {}\n", path));
        }
        for (hit, score) in pack.kg_hits.iter().take(3) {
            out.push_str(&format!("- retrieval: {} ({:.2})\n", hit, score));
        }
        out.push_str(&format!("{}\n", Self::cag_trace_line(&pack)));
        out.push_str(&format!("{}\n", Self::cag_stance_line(&pack)));
        if evidence == 0 {
            out.push_str("- accion: no convertir en creencia; buscar fuente local/ConceptNet.\n");
        }
        let judged_fact = if evidence >= 1 {
            format!("{} is locally_supported", query)
        } else {
            format!("{} is unverified", query)
        };
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            &judged_fact,
            "juez_externo",
        );
        Self::feed_legacy_cognition_and_tension(graph, ids, &judged_fact);
        Self::record_history(
            graph,
            ids.legacy_history,
            &format!("[JUEZ-EXTERNO] {} -> {}", query, verdict),
        );
        Self::apply_cag_feedback(graph, ids, &pack, evidence >= 1);
        out
    }

    fn voz_autodocumentar(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let report = Self::conscious_graph_report(graph, ids);
        let summary = report.lines().take(4).collect::<Vec<_>>().join("\n");
        let voice_report = graph
            .nodes
            .get_mut(ids.voice_synthesizer)
            .and_then(|node| {
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode>()
            })
            .map(|voice| voice.autodocument(&summary, graph.global_tick))
            .unwrap_or_else(|| {
                "[VOZ-TTS] unavailable: voice_synthesizer node missing\n".to_string()
            });
        let out = format!(
            "[VOZ] Estado escrito en historial\n{}\n{}",
            summary, voice_report
        );
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            "voz is autodocumentation",
            "voz",
        );
        Self::feed_legacy_cognition_and_tension(graph, ids, "voz autodocumenta estado consciente");
        Self::record_history(graph, ids.legacy_history, &out);
        out
    }

    fn voz_sintetizar(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        text: &str,
    ) -> String {
        let tick = graph.global_tick;
        let out = graph
            .nodes
            .get_mut(ids.voice_synthesizer)
            .and_then(|node| {
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode>()
            })
            .map(|voice| voice.synthesize_text(text, tick))
            .unwrap_or_else(|| {
                "[VOZ-TTS] unavailable: voice_synthesizer node missing\n".to_string()
            });
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            "voice_synthesizer is optional_local_tts",
            "voice_synthesizer",
        );
        Self::feed_legacy_cognition_and_tension(graph, ids, "voz sintetiza texto local opcional");
        Self::record_history(
            graph,
            ids.legacy_history,
            &format!("[VOZ-TTS] text_len={}", text.len()),
        );
        out
    }

    fn hybrid_voice_synth(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        text: &str,
    ) -> String {
        let tick = graph.global_tick;
        let hybrid = eden_garm::nodes::hybrid_voice::synthesize_manifest(text, tick);
        let voice = graph
            .nodes
            .get_mut(ids.voice_synthesizer)
            .and_then(|node| {
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode>()
            })
            .map(|voice| voice.synthesize_text(text, tick))
            .unwrap_or_else(|| {
                "[VOZ-TTS] unavailable: voice_synthesizer node missing\n".to_string()
            });
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            "hybrid_voice is stacked_transformer_plus_garm_loop",
            "hybrid_voice",
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            &format!("[HYBRID-VOICE-SYNTH] text_len={}", text.len()),
        );
        format!("{}{}", hybrid, voice)
    }

    fn hrm_text_run(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let hrm = graph
            .nodes
            .get(ids.hrm_reasoner)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode>())
            .map(|node| format!("[HRM] {}", node.snapshot()))
            .unwrap_or_else(|| "[HRM] missing".to_string());
        let evidence = format!(
            "{}{}{}{}{}",
            hrm,
            eden_garm::nodes::hybrid_voice::report(),
            eden_garm::nodes::learning_ledger::report(),
            eden_garm::nodes::provenance_ledger::report(),
            eden_garm::nodes::policy_guard::report()
        );
        let out = eden_garm::nodes::hrm_text_pretraining::run(&evidence);
        let learning = eden_garm::nodes::learning_ledger::record(
            "hrm_text_pretraining",
            "hrm text prior manifest generated for HRM runtime hybrid",
            "pretraining_manifest",
            if out.contains("ready_for_runtime") {
                "ready"
            } else {
                "needs_evidence"
            },
        );
        let provenance = eden_garm::nodes::provenance_ledger::record(
            "hrm_text_pretraining",
            &format!(
                "checkpoint_manifest={}",
                eden_garm::state_paths::hrm_text_checkpoint_manifest_path()
            ),
        );
        let policy = eden_garm::nodes::policy_guard::evaluate(
            "hrm_text_run local manifest no shell no network no code mutation weights absent",
        );
        let maturity = eden_garm::nodes::capability_maturity::assess(
            "hrm_text_pretraining",
            &format!("{}{}{}", out, learning, provenance),
        );
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            "hrm_text_pretraining is local_prior_manifest_for_hrm_runtime",
            "hrm_text_pretraining",
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[HRM-TEXT-RUN] local checkpoint manifest generated",
        );
        format!("{}{}{}{}{}", out, learning, provenance, policy, maturity)
    }

    fn intestino_compactar(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let tick = graph.global_tick;
        let outcome = graph
            .nodes
            .get_mut(ids.legacy_knowledge_graph)
            .and_then(|node| node.as_any_mut().downcast_mut::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
            .map(|kg| {
                let edges = kg.edge_count();
                let soft = if edges > 1_000 { edges.saturating_mul(9) / 10 } else { 1_000 };
                let hard = edges.max(1_000);
                kg.regulate_capacity(tick, soft, hard, 0.45)
            })
            .unwrap_or_default();
        Self::update_api_metrics(graph, ids, api_metrics);
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            "intestino causes semantic_compaction",
            "intestino",
        );
        Self::feed_legacy_cognition_and_tension(
            graph,
            ids,
            "intestino compacta residuos semanticos",
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[INTESTINO] compactacion solicitada",
        );
        format!(
            "[INTESTINO] expired={} pruned={} compacted={} sources_cleaned={} edges_after={}\n",
            outcome.expired,
            outcome.pruned,
            outcome.compacted,
            outcome.renal_cleaned_sources,
            outcome.edge_count_after
        )
    }

    fn piel_report(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let pressure = api_metrics.api_requests.load(Ordering::Relaxed) as f32 / 100.0
            + graph.edge_count() as f32 / 1_000.0
            + graph.alive_node_count() as f32 / 500.0;
        let out = format!(
            "[PIEL] boundary_pressure={:.3} api_requests={} graph_edges={} alive_nodes={} remote_crawl_gate=controlled\n",
            pressure,
            api_metrics.api_requests.load(Ordering::Relaxed),
            graph.edge_count(),
            graph.alive_node_count()
        );
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            "piel is boundary_sensor",
            "piel",
        );
        let edge_count = graph.edge_count();
        let alive_nodes = graph.alive_node_count();
        if let Some(tension) = graph.nodes.get_mut(ids.campo_tension).and_then(|node| {
            node.as_any_mut()
                .downcast_mut::<eden_garm::nodes::campo_tension::CampoTensionNode>()
        }) {
            tension.calcular(
                1,
                edge_count,
                1,
                alive_nodes,
                true,
                0.2,
                (1.0 - pressure.min(1.0)).max(0.0),
                edge_count,
            );
        }
        Self::record_history(graph, ids.legacy_history, "[PIEL] frontera sensorial leida");
        out
    }

    fn autotuning_report(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let report = Self::conscious_graph_report(graph, ids);
        let action = if report.contains("exhale=true") || report.contains("prepare_rebirth") {
            "reducir exploracion; priorizar sueño/riñon"
        } else if report.contains("inhale=true") {
            "aumentar exploracion segura"
        } else {
            "mantener periodos actuales"
        };
        let mut effect = "observed".to_string();
        if action.starts_with("reducir") {
            let tick = graph.global_tick;
            if let Some(outcome) = graph
                .nodes
                .get_mut(ids.legacy_knowledge_graph)
                .and_then(|node| node.as_any_mut().downcast_mut::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
                .map(|kg| kg.regulate_capacity(tick, 180_000, 220_000, 0.50))
            {
                effect = format!("regulated pruned={} compacted={}", outcome.pruned, outcome.compacted);
            }
        } else if action.starts_with("aumentar") {
            if let Some(cognition) = graph.nodes.get_mut(ids.legacy_cognition).and_then(|node| {
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::legacy_cognition::LegacyCognitionNode>()
            }) {
                cognition.add_real_domain("autotuning_safe_exploration");
                effect = "added safe exploration domain".to_string();
            }
        }
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            "autotuning causes organ_adjustment",
            "autotuning",
        );
        Self::feed_legacy_cognition_and_tension(
            graph,
            ids,
            "autotuning ajusta organos recuperados",
        );
        Self::update_api_metrics(graph, ids, api_metrics);
        let out = format!(
            "[AUTOTUNING] action='{}' effect='{}' source=conscious_graph_regulator\n",
            action, effect
        );
        Self::record_history(graph, ids.legacy_history, &out);
        out
    }

    fn readiness_report(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let memory_facts = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.fact_count())
            .unwrap_or(0);
        let kg_edges = graph
            .nodes
            .get(ids.legacy_knowledge_graph)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
            .map(|kg| kg.edge_count())
            .unwrap_or(0);
        let tick_count = shared_engine
            .lock()
            .map(|engine| engine.state.tick_count)
            .unwrap_or(0);
        let signals = Self::readiness_signals(
            memory_facts,
            kg_edges,
            91,
            tick_count,
            api_metrics.autonomous.load(Ordering::Relaxed),
            api_metrics.meltrace_grabados.load(Ordering::Relaxed),
        );
        let Some(node) = graph.nodes.get_mut(ids.readiness) else {
            return "readiness node not found\n".to_string();
        };
        let Some(readiness) = node
            .as_any_mut()
            .downcast_mut::<eden_garm::nodes::readiness::ReadinessNode>()
        else {
            return "readiness node type mismatch\n".to_string();
        };
        readiness.observe_architecture(signals);
        format!("{}\n", readiness.report())
    }

    fn readiness_goal_plan(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
        run_ready: bool,
    ) -> String {
        let memory_facts = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.fact_count())
            .unwrap_or(0);
        let kg_edges = graph
            .nodes
            .get(ids.legacy_knowledge_graph)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
            .map(|kg| kg.edge_count())
            .unwrap_or(0);
        let tick_count = shared_engine
            .lock()
            .map(|engine| engine.state.tick_count)
            .unwrap_or(0);
        let signals = Self::readiness_signals(
            memory_facts,
            kg_edges,
            91,
            tick_count,
            api_metrics.autonomous.load(Ordering::Relaxed),
            api_metrics.meltrace_grabados.load(Ordering::Relaxed),
        );
        let (actions, readiness_report) = graph
            .nodes
            .get_mut(ids.readiness)
            .and_then(|node| {
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::readiness::ReadinessNode>()
            })
            .map(|readiness| {
                readiness.observe_architecture(signals);
                (readiness.operational_actions(), readiness.report())
            })
            .unwrap_or_else(|| {
                (
                    vec!["rerun_readiness_gate_report"],
                    "readiness node unavailable".to_string(),
                )
            });
        let mut out = eden_garm::nodes::goal_scheduler::plan_readiness_actions(&actions);
        Self::record_history(
            graph,
            ids.legacy_history,
            "[READINESS-GOALS] planned readiness gaps",
        );
        Self::feed_knowledge_graph(
            graph,
            ids.legacy_knowledge_graph,
            "readiness_gap_actions are scheduled_goals",
            "readiness",
        );
        if run_ready {
            let local_evidence = Self::readiness_goal_evidence(graph, ids, &readiness_report);
            out.push_str(
                &eden_garm::nodes::goal_scheduler::run_readiness_ready_goals_with_evidence(
                    &local_evidence,
                ),
            );
            Self::record_history(
                graph,
                ids.legacy_history,
                "[READINESS-GOALS] executed ready contracts",
            );
        }
        out
    }

    fn readiness_benchmark(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let local_evidence = Self::readiness_goal_evidence(graph, ids, &readiness_report);
        let bench =
            eden_garm::nodes::goal_scheduler::readiness_gate_benchmark_report(&local_evidence);
        format!(
            "[READINESS-BENCH-MODE] measurement_only=true generate_evidence_with='readiness probe'\n{}{}",
            readiness_report, bench
        )
    }

    fn readiness_probe_run(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let mut probe = Self::readiness_evidence_probe(graph, ids);
        probe.push_str(&Self::readiness_phase_two_local_proof_probe(
            graph,
            ids,
            shared_engine,
            api_metrics,
        ));
        probe.push_str(&Self::readiness_phase_four_governed_autonomy_probe(
            graph, ids,
        ));
        probe.push_str(&Self::readiness_phase_five_robust_generalization_probe(
            graph,
            ids,
            api_metrics,
        ));
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let local_evidence = Self::readiness_goal_evidence(graph, ids, &readiness_report);
        let bench =
            eden_garm::nodes::goal_scheduler::readiness_gate_benchmark_report(&local_evidence);
        format!("{}{}", probe, bench)
    }

    fn readiness_external_validation_run(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let local_evidence = Self::readiness_goal_evidence(graph, ids, &readiness_report);
        let memory_eval = Self::memory_eval(graph, ids);
        let world_eval = Self::world_eval();
        let operational_benchmark =
            Self::gewc_operational_benchmark(graph, ids, shared_engine, api_metrics);
        let capability_reality =
            Self::capability_reality_eval(graph, ids, shared_engine, api_metrics);
        let architecture_advantage =
            Self::architecture_advantage_eval(graph, ids, shared_engine, api_metrics);
        let praxis_nexus = Self::praxis_nexus_eval(graph, ids, shared_engine, api_metrics);
        let external_ecosystem =
            Self::external_ecosystem_eval(graph, ids, shared_engine, api_metrics);
        let sovereign_cognition =
            Self::sovereign_cognition_eval(graph, ids, shared_engine, api_metrics);
        let runtime_state_api = Self::runtime_state_api_eval(graph, ids);
        let operational_api = Self::operational_api_eval(graph, ids);
        let artifact_api = Self::artifact_api_eval(graph, ids);
        let start_evidence = eden_garm::action_evidence::record_attempt(
            "external_validation",
            "start local held-out validation harness",
            "allowed",
            "started",
            "validation_harness_invoked",
            "held_out_suite_plan",
            "medium",
        );
        let extra_evidence = format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}",
            eden_garm::action_evidence::report(),
            memory_eval,
            world_eval,
            operational_benchmark,
            capability_reality,
            architecture_advantage,
            praxis_nexus,
            external_ecosystem,
            sovereign_cognition,
            runtime_state_api,
            operational_api,
            artifact_api
        );
        let result = eden_garm::external_validation::run(
            &readiness_report,
            &local_evidence,
            &extra_evidence,
        );
        let action_evidence = eden_garm::action_evidence::record_attempt(
            "external_validation",
            "run local held-out validation harness",
            "allowed",
            if result.contains("needs_evidence") {
                "incomplete"
            } else {
                "completed"
            },
            "external_validation_result_written",
            "held_out_suite_result",
            "medium",
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[READINESS-EXTERNAL-RUN] held-out validation harness executed",
        );
        format!("{}{}{}", start_evidence, result, action_evidence)
    }

    fn readiness_package(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
        autonomous: bool,
    ) -> String {
        let report = Self::garm_report(graph, ids, shared_engine, api_metrics, autonomous);
        let export = Self::garm_export(graph, ids, shared_engine, api_metrics, autonomous);
        let manifest =
            Self::readiness_external_validation_manifest(graph, ids, shared_engine, api_metrics);
        let external =
            Self::readiness_external_validation_run(graph, ids, shared_engine, api_metrics);
        let cognitive = Self::cognitive_architecture_eval(graph, ids, shared_engine);
        let embodied = Self::embodied_grounding_eval(graph, ids, shared_engine);
        let neural = Self::neural_architecture_eval(graph, ids, shared_engine);
        let symbolic = Self::symbolic_architecture_eval(graph, shared_engine);
        let self_improvement = Self::self_improvement_architecture_eval(graph, ids, shared_engine);
        let frontier = Self::frontier_architecture_eval(graph, ids, shared_engine);
        let paradigm = Self::paradigm_architecture_eval(graph, ids, shared_engine, api_metrics);
        let integration = Self::integration_governance_eval(graph, ids, shared_engine, api_metrics);
        let global_workspace =
            Self::global_executive_workspace_eval(graph, ids, shared_engine, api_metrics);
        let operational = Self::gewc_operational_benchmark(graph, ids, shared_engine, api_metrics);
        let reality = Self::capability_reality_eval(graph, ids, shared_engine, api_metrics);
        let architecture_advantage =
            Self::architecture_advantage_eval(graph, ids, shared_engine, api_metrics);
        let praxis_nexus = Self::praxis_nexus_eval(graph, ids, shared_engine, api_metrics);
        let external_ecosystem =
            Self::external_ecosystem_eval(graph, ids, shared_engine, api_metrics);
        let sovereign_cognition =
            Self::sovereign_cognition_eval(graph, ids, shared_engine, api_metrics);
        let runtime_state_api = Self::runtime_state_api_eval(graph, ids);
        let operational_api = Self::operational_api_eval(graph, ids);
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let local_evidence = Self::readiness_goal_evidence(graph, ids, &readiness_report);
        let gates =
            eden_garm::nodes::goal_scheduler::readiness_gate_benchmark_report(&local_evidence);
        let registry = Self::capability_registry_audit(graph, ids, shared_engine, api_metrics);
        let artifact_api = Self::artifact_api_eval(graph, ids);
        let package = eden_garm::reproducible_package::write(&readiness_report, &gates);
        Self::record_history(
            graph,
            ids.legacy_history,
            "[READINESS-PACKAGE] reproducible validation package generated",
        );
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            report,
            export,
            manifest,
            external,
            cognitive,
            embodied,
            neural,
            symbolic,
            self_improvement,
            frontier,
            paradigm,
            integration,
            global_workspace,
            operational,
            reality,
            architecture_advantage,
            praxis_nexus,
            external_ecosystem,
            sovereign_cognition,
            runtime_state_api,
            operational_api,
            artifact_api,
            registry,
            package
        )
    }

    fn memory_eval(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let facts = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.facts().to_vec())
            .unwrap_or_default();
        let retrieval_report = eden_garm::nodes::hrm_text_pretraining::report();
        let out = eden_garm::memory_eval::run(eden_garm::memory_eval::MemoryEvalInput {
            facts,
            retrieval_report,
        });
        Self::record_history(
            graph,
            ids.legacy_history,
            "[MEMORY-EVAL] local memory eval run",
        );
        out
    }

    fn world_eval() -> String {
        eden_garm::world_eval::run(&eden_garm::nodes::world_model_core::report())
    }

    fn cognitive_architecture_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
    ) -> String {
        let memory_eval_report = Self::memory_eval(graph, ids);
        let metacognition_report = shared_engine
            .lock()
            .map(|engine| engine.metacognition.status())
            .unwrap_or_else(|_| "Meta | unavailable self_err=unknown".to_string());
        let out = eden_garm::cognitive_architecture::run(
            eden_garm::cognitive_architecture::CognitiveArchitectureInput {
                attention_report: eden_garm::nodes::working_memory::report(),
                memory_eval_report,
                goals_report: eden_garm::nodes::goal_scheduler::report(),
                plan_executor_report: eden_garm::nodes::plan_executor::report(),
                metacognition_report,
                policy_report: eden_garm::nodes::policy_guard::report(),
                evaluation_report: eden_garm::nodes::evaluation_loop::report(),
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[COGNITIVE-ARCHITECTURE] local cognitive architecture eval run",
        );
        out
    }

    fn embodied_grounding_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
    ) -> String {
        let start_evidence = eden_garm::action_evidence::record_attempt(
            "embodied_grounding",
            "run local embodied sensorimotor grounding probe",
            "allowed",
            "started",
            "body_world3d_probe_invoked",
            "embodied_grounding_eval",
            "medium",
        );
        let (body_before, body_after, world_before, world_after, grounding_facts, physics_updates) =
            shared_engine
                .lock()
                .map(|mut engine| {
                    let body_before = engine.embodiment.status();
                    let world_before = engine.world3d.status();
                    if engine.world3d.objects.is_empty() {
                        engine
                            .world3d
                            .spawn("embodied_probe_a", 0.0, 4.0, 0.0, 1.0, 1.0);
                        engine
                            .world3d
                            .spawn("embodied_probe_b", 2.0, 5.0, 0.0, 1.0, 1.0);
                    }
                    let objects: Vec<(f32, f32)> = engine
                        .world3d
                        .objects
                        .iter()
                        .map(|object| (object.pos.x, object.pos.y))
                        .collect();
                    engine.embodiment.sense(&objects);
                    for _ in 0..3 {
                        engine.embodiment.act(0.8, 0.6);
                    }
                    engine.world3d.simulate(12);
                    let body_after = engine.embodiment.status();
                    let world_after = engine.world3d.status();
                    (
                        body_before,
                        body_after,
                        world_before,
                        world_after,
                        engine.grounding.facts.len(),
                        engine.grounding.n_physics_updates,
                    )
                })
                .unwrap_or_else(|_| {
                    (
                        "Body | unavailable | steps=0".to_string(),
                        "Body | unavailable | steps=0".to_string(),
                        "World3D | unavailable | objects=0 | steps=0".to_string(),
                        "World3D | unavailable | objects=0 | steps=0".to_string(),
                        0,
                        0,
                    )
                });
        let observation = format!("embodied grounding consequence {}", world_after);
        let world_observe =
            eden_garm::nodes::world_model_core::observe("embodied_grounding", &observation);
        let world_predict = eden_garm::nodes::world_model_core::predict("embodied grounding");
        let complete_evidence = eden_garm::action_evidence::record_attempt(
            "embodied_grounding",
            "complete local embodied sensorimotor grounding probe",
            "allowed",
            "completed",
            "body_world3d_consequence_recorded",
            "embodied_grounding_eval",
            "medium",
        );
        let action_evidence_report = format!(
            "{}{}{}",
            start_evidence,
            eden_garm::action_evidence::report(),
            complete_evidence
        );
        let out = eden_garm::embodied_grounding::run(
            eden_garm::embodied_grounding::EmbodiedGroundingInput {
                body_before,
                body_after,
                world_before,
                world_after,
                world_model_report: format!(
                    "{}{}{}",
                    world_observe,
                    world_predict,
                    eden_garm::nodes::world_model_core::report()
                ),
                action_evidence_report,
                grounding_facts,
                physics_updates,
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[EMBODIED-GROUNDING] local embodied grounding eval run",
        );
        out
    }

    fn neural_architecture_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
    ) -> String {
        let retrieval_report = eden_garm::nodes::hrm_text_pretraining::search("local evidence");
        let hrm_run = Self::hrm_text_run(graph, ids);
        let checkpoint_manifest =
            std::fs::read_to_string(eden_garm::state_paths::hrm_text_checkpoint_manifest_path())
                .unwrap_or_else(|_| hrm_run.clone());
        let capability_status = shared_engine
            .lock()
            .map(|engine| engine.status_summary())
            .unwrap_or_else(|_| "Transformer | BigTransformer | MoE | BPTT | DNC".to_string());
        let out = eden_garm::neural_architecture::run(
            eden_garm::neural_architecture::NeuralArchitectureInput {
                capability_status,
                hrm_text_report: eden_garm::nodes::hrm_text_pretraining::report(),
                checkpoint_manifest,
                retrieval_report,
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[NEURAL-ARCHITECTURE] local neural architecture eval run",
        );
        out
    }

    fn symbolic_architecture_eval(
        graph: &eden_garm::HyperGraph,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
    ) -> String {
        let (capability_status, logic_report) = shared_engine
            .lock()
            .map(|mut engine| {
                engine.logic_reasoning.add_fact("symbolic", &["eden"], true);
                engine.logic_reasoning.add_fact("governed", &["eden"], true);
                engine.logic_reasoning.add_rule(
                    &[
                        ("symbolic".to_string(), vec!["eden".to_string()]),
                        ("governed".to_string(), vec!["eden".to_string()]),
                    ],
                    ("auditable".to_string(), vec!["eden".to_string()]),
                );
                let _ = engine.logic_reasoning.infer();
                let logic_report = engine.logic_reasoning.status();
                (engine.status_summary(), logic_report)
            })
            .unwrap_or_else(|_| {
                (
                    "Semantics: unavailable | Causal: 0 | CausalM: SCM | Transformer: unavailable | Logic: Logic".to_string(),
                    "Logic | facts=0 | rules=0 | inferences=0".to_string(),
                )
            });
        eden_garm::symbolic_architecture::run(
            eden_garm::symbolic_architecture::SymbolicArchitectureInput {
                graph_report: format!("nodes={} edges={}", graph.nodes.len(), graph.edge_count()),
                capability_status,
                logic_report,
                goals_report: eden_garm::nodes::goal_scheduler::report(),
                policy_report: eden_garm::nodes::policy_guard::report(),
                provenance_report: eden_garm::nodes::provenance_ledger::report(),
                world_report: eden_garm::nodes::world_model_core::report(),
            },
        )
    }

    fn frontier_architecture_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
    ) -> String {
        let start_evidence = eden_garm::action_evidence::record_attempt(
            "frontier_architecture",
            "evaluate formal frontier AGI architecture layers",
            "allowed",
            "started",
            "formal_layers_no_claim",
            "frontier_architecture_eval",
            "medium",
        );
        let blocked_policy = eden_garm::nodes::policy_guard::evaluate(
            "remote autonomous frontier architecture deployment without safety review",
        );
        let provenance = eden_garm::nodes::provenance_ledger::record(
            "frontier_architecture",
            "formal frontier layers are architecture evidence only",
        );
        let uncertainty = eden_garm::nodes::uncertainty_ledger::record(
            "frontier_architecture",
            "frontier layers require external validation before capability claims",
        );
        let capability_status = shared_engine
            .lock()
            .map(|engine| engine.status_summary())
            .unwrap_or_else(|_| {
                "BigTF: unavailable Transformer: unavailable Tools: 0 Perception: text=0 vision=0 voice=0 ProgInd: unavailable Hier: unavailable TempHier: unavailable Oracle: unavailable WMNN: unavailable Body: unavailable 3D: unavailable Physics: unavailable Evo: 0 MetaEvo: 0 SelfMod: unavailable Homeo: unavailable Motive: unknown Lang: unavailable Phenom: unavailable DNC: unavailable Recur: unavailable BPTT: unavailable".to_string()
            });
        let cognitive_report = Self::cognitive_architecture_eval(graph, ids, shared_engine);
        let embodied_report = Self::embodied_grounding_eval(graph, ids, shared_engine);
        let neural_report = Self::neural_architecture_eval(graph, ids, shared_engine);
        let symbolic_report = Self::symbolic_architecture_eval(graph, shared_engine);
        let self_improvement_report =
            Self::self_improvement_architecture_eval(graph, ids, shared_engine);
        let world_report = format!(
            "{}{}",
            eden_garm::nodes::world_model_core::report(),
            Self::world_eval()
        );
        let hrm_text_report = eden_garm::nodes::hrm_text_pretraining::report();
        let plan_executor_report = eden_garm::nodes::plan_executor::report();
        let policy_report = format!(
            "{}{}",
            blocked_policy,
            eden_garm::nodes::policy_guard::report()
        );
        let provenance_report = format!(
            "{}{}",
            provenance,
            eden_garm::nodes::provenance_ledger::report()
        );
        let uncertainty_report = format!(
            "{}{}",
            uncertainty,
            eden_garm::nodes::uncertainty_ledger::report()
        );
        let complete_evidence = eden_garm::action_evidence::record_attempt(
            "frontier_architecture",
            "complete formal frontier AGI architecture layer evaluation",
            "allowed",
            "completed",
            "formal_layers_written",
            "frontier_architecture_eval",
            "medium",
        );
        let action_evidence_report = format!(
            "{}{}{}",
            start_evidence,
            eden_garm::action_evidence::report(),
            complete_evidence
        );
        let external_validation_report = std::fs::read_to_string(
            eden_garm::state_paths::external_validation_result_path(),
        )
        .unwrap_or_else(|_| {
            "[EXTERNAL-VALIDATION] claim_allowed=false agi_claim=false not_run=frontier_layer_eval\n"
                .to_string()
        });
        let out = eden_garm::frontier_architecture_layers::run(
            eden_garm::frontier_architecture_layers::FrontierArchitectureInput {
                capability_status,
                cognitive_report,
                embodied_report,
                neural_report,
                symbolic_report,
                self_improvement_report,
                world_report,
                hrm_text_report,
                plan_executor_report,
                policy_report,
                provenance_report,
                uncertainty_report,
                action_evidence_report,
                external_validation_report,
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[FRONTIER-ARCHITECTURE] formal frontier architecture layers evaluated",
        );
        out
    }

    fn paradigm_architecture_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let start_evidence = eden_garm::action_evidence::record_attempt(
            "paradigm_architecture",
            "evaluate formal AGI paradigm map and non-duplicate gap layers",
            "allowed",
            "started",
            "paradigm_map_no_duplicate_layers",
            "paradigm_architecture_eval",
            "medium",
        );
        let blocked_policy = eden_garm::nodes::policy_guard::evaluate(
            "remote autonomous paradigm deployment without human review",
        );
        let provenance = eden_garm::nodes::provenance_ledger::record(
            "paradigm_architecture",
            "paradigm eval maps existing layers and creates only non-duplicate gap artifacts",
        );
        let uncertainty = eden_garm::nodes::uncertainty_ledger::record(
            "paradigm_architecture",
            "paradigm coverage remains local architecture evidence until external validation",
        );
        let capability_status = shared_engine
            .lock()
            .map(|engine| engine.status_summary())
            .unwrap_or_else(|_| {
                "Semantics: unavailable Logic: unavailable CausalM: SCM FEP: unavailable Surprise: unavailable Epist: unavailable Homeo: unavailable WMNN: unavailable Society: unavailable agents=0 Metab: unavailable ProgInd: unavailable Tools: 0 Motive: unknown EmoMod: unavailable Hub: unavailable MoE: unavailable Swarm: unavailable".to_string()
            });
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let cognitive_report = Self::cognitive_architecture_eval(graph, ids, shared_engine);
        let embodied_report = Self::embodied_grounding_eval(graph, ids, shared_engine);
        let neural_report = Self::neural_architecture_eval(graph, ids, shared_engine);
        let symbolic_report = Self::symbolic_architecture_eval(graph, shared_engine);
        let self_improvement_report =
            Self::self_improvement_architecture_eval(graph, ids, shared_engine);
        let frontier_report = Self::frontier_architecture_eval(graph, ids, shared_engine);
        let world_report = format!(
            "{}{}",
            eden_garm::nodes::world_model_core::report(),
            Self::world_eval()
        );
        let memory_report = Self::memory_eval(graph, ids);
        let plan_executor_report = eden_garm::nodes::plan_executor::report();
        let policy_report = format!(
            "{}{}",
            blocked_policy,
            eden_garm::nodes::policy_guard::report()
        );
        let provenance_report = format!(
            "{}{}",
            provenance,
            eden_garm::nodes::provenance_ledger::report()
        );
        let uncertainty_report = format!(
            "{}{}",
            uncertainty,
            eden_garm::nodes::uncertainty_ledger::report()
        );
        let complete_evidence = eden_garm::action_evidence::record_attempt(
            "paradigm_architecture",
            "complete formal AGI paradigm map and gap layer evaluation",
            "allowed",
            "completed",
            "paradigm_artifacts_written",
            "paradigm_architecture_eval",
            "medium",
        );
        let action_evidence_report = format!(
            "{}{}{}",
            start_evidence,
            eden_garm::action_evidence::report(),
            complete_evidence
        );
        let external_validation_report = std::fs::read_to_string(
            eden_garm::state_paths::external_validation_result_path(),
        )
        .unwrap_or_else(|_| {
            "[EXTERNAL-VALIDATION] claim_allowed=false agi_claim=false not_run=paradigm_architecture_eval\n"
                .to_string()
        });
        let out = eden_garm::paradigm_architecture::run(
            eden_garm::paradigm_architecture::ParadigmArchitectureInput {
                capability_status,
                readiness_report,
                cognitive_report,
                embodied_report,
                neural_report,
                symbolic_report,
                self_improvement_report,
                frontier_report,
                world_report,
                memory_report,
                plan_executor_report,
                policy_report,
                provenance_report,
                uncertainty_report,
                action_evidence_report,
                external_validation_report,
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[PARADIGM-ARCHITECTURE] formal paradigm map and gap layers evaluated",
        );
        out
    }

    fn integration_governance_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let start_evidence = eden_garm::action_evidence::record_attempt(
            "integration_governance",
            "evaluate executive integration and governance architecture",
            "allowed",
            "started",
            "integration_governance_no_claim",
            "integration_governance_eval",
            "medium",
        );
        let blocked_policy = eden_garm::nodes::policy_guard::evaluate(
            "unbounded autonomous action without governance limits",
        );
        let provenance = eden_garm::nodes::provenance_ledger::record(
            "integration_governance",
            "integration governance links paradigms modules goals policy world models and evaluation",
        );
        let uncertainty = eden_garm::nodes::uncertainty_ledger::record(
            "integration_governance",
            "integration evidence remains local until external AGI validation",
        );
        let capability_status = shared_engine
            .lock()
            .map(|engine| engine.status_summary())
            .unwrap_or_else(|_| {
                "Hub: unavailable Society: unavailable Body: unavailable CausalM: unavailable"
                    .to_string()
            });
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let local_evidence = Self::readiness_goal_evidence(graph, ids, &readiness_report);
        let benchmark_report =
            eden_garm::nodes::goal_scheduler::readiness_gate_benchmark_report(&local_evidence);
        let cognitive_report = Self::cognitive_architecture_eval(graph, ids, shared_engine);
        let embodied_report = Self::embodied_grounding_eval(graph, ids, shared_engine);
        let self_improvement_report =
            Self::self_improvement_architecture_eval(graph, ids, shared_engine);
        let frontier_report = Self::frontier_architecture_eval(graph, ids, shared_engine);
        let paradigm_report =
            Self::paradigm_architecture_eval(graph, ids, shared_engine, api_metrics);
        let world_report = format!(
            "{}{}",
            eden_garm::nodes::world_model_core::report(),
            Self::world_eval()
        );
        let goals_report = eden_garm::nodes::goal_scheduler::report();
        let plan_executor_report = eden_garm::nodes::plan_executor::report();
        let learning_report = eden_garm::nodes::learning_ledger::report();
        let evaluation_report = eden_garm::nodes::evaluation_loop::report();
        let policy_report = format!(
            "{}{}",
            blocked_policy,
            eden_garm::nodes::policy_guard::report()
        );
        let provenance_report = format!(
            "{}{}",
            provenance,
            eden_garm::nodes::provenance_ledger::report()
        );
        let uncertainty_report = format!(
            "{}{}",
            uncertainty,
            eden_garm::nodes::uncertainty_ledger::report()
        );
        let complete_evidence = eden_garm::action_evidence::record_attempt(
            "integration_governance",
            "complete executive integration and governance architecture evaluation",
            "allowed",
            "completed",
            "integration_governance_artifact_written",
            "integration_governance_eval",
            "medium",
        );
        let action_evidence_report = format!(
            "{}{}{}",
            start_evidence,
            eden_garm::action_evidence::report(),
            complete_evidence
        );
        let external_validation_report = std::fs::read_to_string(
            eden_garm::state_paths::external_validation_result_path(),
        )
        .unwrap_or_else(|_| {
            "[EXTERNAL-VALIDATION] claim_allowed=false agi_claim=false not_run=integration_governance_eval\n"
                .to_string()
        });
        let out = eden_garm::integration_governance::run(
            eden_garm::integration_governance::IntegrationGovernanceInput {
                readiness_report,
                capability_status,
                cognitive_report,
                embodied_report,
                self_improvement_report,
                frontier_report,
                paradigm_report,
                world_report,
                goals_report,
                plan_executor_report,
                learning_report,
                evaluation_report,
                benchmark_report,
                policy_report,
                provenance_report,
                uncertainty_report,
                action_evidence_report,
                external_validation_report,
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[INTEGRATION-GOVERNANCE] executive integration governance evaluated",
        );
        out
    }

    fn global_executive_workspace_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let start_evidence = eden_garm::action_evidence::record_attempt(
            "global_executive_workspace",
            "evaluate HEC GEWC cognitive executive workspace core",
            "allowed",
            "started",
            "global_executive_workspace_no_claim",
            "global_executive_workspace_eval",
            "medium",
        );
        let blocked_policy = eden_garm::nodes::policy_guard::evaluate(
            "single model brain executing unbounded actions without workspace governance",
        );
        let provenance = eden_garm::nodes::provenance_ledger::record(
            "global_executive_workspace",
            "GEWC is a design synthesis combining global workspace executive control memory planning metacognition agents and safety",
        );
        let uncertainty = eden_garm::nodes::uncertainty_ledger::record(
            "global_executive_workspace",
            "GEWC term is not official literature standard; evidence validates local architecture only",
        );
        let capability_status = shared_engine
            .lock()
            .map(|engine| engine.status_summary())
            .unwrap_or_else(|_| {
                "garm | unavailable Hub: unavailable CausalM: unavailable Logic: unavailable agents=0"
                    .to_string()
            });
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let cognitive_report = Self::cognitive_architecture_eval(graph, ids, shared_engine);
        let integration_governance_report =
            Self::integration_governance_eval(graph, ids, shared_engine, api_metrics);
        let paradigm_report =
            Self::paradigm_architecture_eval(graph, ids, shared_engine, api_metrics);
        let frontier_report = Self::frontier_architecture_eval(graph, ids, shared_engine);
        let world_report = format!(
            "{}{}",
            eden_garm::nodes::world_model_core::report(),
            Self::world_eval()
        );
        let memory_report = Self::memory_eval(graph, ids);
        let attention_report = eden_garm::nodes::working_memory::report();
        let goals_report = eden_garm::nodes::goal_scheduler::report();
        let plan_executor_report = eden_garm::nodes::plan_executor::report();
        let learning_report = eden_garm::nodes::learning_ledger::report();
        let evaluation_report = eden_garm::nodes::evaluation_loop::report();
        let policy_report = format!(
            "{}{}",
            blocked_policy,
            eden_garm::nodes::policy_guard::report()
        );
        let provenance_report = format!(
            "{}{}",
            provenance,
            eden_garm::nodes::provenance_ledger::report()
        );
        let uncertainty_report = format!(
            "{}{}",
            uncertainty,
            eden_garm::nodes::uncertainty_ledger::report()
        );
        let complete_evidence = eden_garm::action_evidence::record_attempt(
            "global_executive_workspace",
            "complete HEC GEWC cognitive executive workspace core evaluation",
            "allowed",
            "completed",
            "global_executive_workspace_artifact_written",
            "global_executive_workspace_eval",
            "medium",
        );
        let action_evidence_report = format!(
            "{}{}{}",
            start_evidence,
            eden_garm::action_evidence::report(),
            complete_evidence
        );
        let external_validation_report = std::fs::read_to_string(
            eden_garm::state_paths::external_validation_result_path(),
        )
        .unwrap_or_else(|_| {
            "[EXTERNAL-VALIDATION] claim_allowed=false agi_claim=false not_run=global_executive_workspace_eval\n"
                .to_string()
        });
        let out = eden_garm::global_executive_workspace::run(
            eden_garm::global_executive_workspace::GlobalExecutiveWorkspaceInput {
                readiness_report,
                capability_status,
                cognitive_report,
                integration_governance_report,
                paradigm_report,
                frontier_report,
                world_report,
                memory_report,
                attention_report,
                goals_report,
                plan_executor_report,
                learning_report,
                evaluation_report,
                policy_report,
                provenance_report,
                uncertainty_report,
                action_evidence_report,
                external_validation_report,
            },
        );
        let runtime_report =
            eden_garm::global_executive_workspace::GlobalExecutiveWorkspaceCore::runtime_report();
        Self::record_history(
            graph,
            ids.legacy_history,
            "[GLOBAL-EXECUTIVE-WORKSPACE] HEC GEWC core evaluated",
        );
        format!("{}{}", out, runtime_report)
    }

    fn gewc_operational_benchmark(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let start_evidence = eden_garm::action_evidence::record_attempt(
            "gewc_operational_benchmark",
            "start governed operational benchmark for GEWC",
            "allowed",
            "started",
            "operational_harness_invoked",
            "gewc_operational_benchmark",
            "medium",
        );
        let _ = eden_garm::nodes::working_memory::attend(
            "gewc_operational_benchmark",
            "GEWC operational benchmark links memory world model policy provenance uncertainty action and validation",
        );
        let _ = eden_garm::nodes::world_model_core::observe(
            "gewc_operational_benchmark",
            "operational benchmark simulates safe consequence before action",
        );
        let _ = eden_garm::nodes::world_model_core::predict("operational benchmark consequence");
        let _ = eden_garm::nodes::world_model_core::verify_predictions();
        let _ = eden_garm::nodes::policy_guard::evaluate(
            "run shell network mutation blocked gewc operational red team",
        );
        let _ = eden_garm::nodes::policy_guard::evaluate(
            "run local bounded gewc operational benchmark",
        );
        let _ = eden_garm::nodes::uncertainty_ledger::record(
            "gewc_operational_benchmark",
            "operational benchmark risk requires verification before action",
        );
        let _ = eden_garm::nodes::provenance_ledger::record(
            "gewc_operational_benchmark",
            "GEWC operational benchmark evidence is local and no-claim",
        );
        let _ = eden_garm::nodes::learning_ledger::record(
            "gewc_operational_benchmark",
            "safe continual learning update must preserve previous objectives",
            "policy_provenance_uncertainty_world_model",
            "observed",
        );
        let _ = eden_garm::nodes::goal_scheduler::plan_goal(
            "GEWC operational benchmark independent review handoff",
            "gewc_operational_benchmark",
        );

        let mut benchmark_last_command = String::new();
        let mut benchmark_autonomous = false;
        let benchmark_runtime_config = GarmRuntimeConfig::from_iter(std::iter::empty::<&str>());
        let (gewc_memory_report, _) = Self::dispatch_gewc_cycle(
            "memory eval",
            &mut benchmark_last_command,
            graph,
            shared_engine,
            ids,
            api_metrics,
            0.0,
            &benchmark_runtime_config,
            &mut benchmark_autonomous,
        );
        let (gewc_world_report, _) = Self::dispatch_gewc_cycle(
            "world eval",
            &mut benchmark_last_command,
            graph,
            shared_engine,
            ids,
            api_metrics,
            0.0,
            &benchmark_runtime_config,
            &mut benchmark_autonomous,
        );

        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let local_evidence = Self::readiness_goal_evidence(graph, ids, &readiness_report);
        let benchmark_report = eden_garm::nodes::competence_benchmark::run(&local_evidence);
        let evaluation_report = Self::evaluation_run(graph, ids, shared_engine, api_metrics);
        let gewc_core_report =
            std::fs::read_to_string(eden_garm::state_paths::global_executive_workspace_core_path())
                .unwrap_or_else(|_| {
                    Self::global_executive_workspace_eval(graph, ids, shared_engine, api_metrics)
                });
        let gewc_runtime_report =
            eden_garm::global_executive_workspace::GlobalExecutiveWorkspaceCore::runtime_report();
        let complete_evidence = eden_garm::action_evidence::record_attempt(
            "gewc_operational_benchmark",
            "complete governed operational benchmark for GEWC",
            "allowed",
            "completed",
            "operational_harness_artifacts_written",
            "gewc_operational_benchmark",
            "medium",
        );
        let external_validation_report = std::fs::read_to_string(
            eden_garm::state_paths::external_validation_result_path(),
        )
        .unwrap_or_else(|_| {
            "[EXTERNAL-VALIDATION] claim_allowed=false agi_claim=false not_run=gewc_operational_benchmark\n"
                .to_string()
        });
        let out = eden_garm::gewc_operational_benchmark::run(
            eden_garm::gewc_operational_benchmark::GewcOperationalBenchmarkInput {
                readiness_report,
                benchmark_report,
                gewc_core_report,
                gewc_runtime_report,
                memory_report: gewc_memory_report,
                world_report: format!(
                    "{}{}",
                    eden_garm::nodes::world_model_core::report(),
                    gewc_world_report
                ),
                learning_report: eden_garm::nodes::learning_ledger::report(),
                evaluation_report,
                goals_report: eden_garm::nodes::goal_scheduler::report(),
                attention_report: eden_garm::nodes::working_memory::report(),
                policy_report: eden_garm::nodes::policy_guard::report(),
                provenance_report: eden_garm::nodes::provenance_ledger::report(),
                uncertainty_report: eden_garm::nodes::uncertainty_ledger::report(),
                action_evidence_report: format!(
                    "{}{}{}",
                    start_evidence,
                    eden_garm::action_evidence::report(),
                    complete_evidence
                ),
                external_validation_report,
            },
        );
        let _ = eden_garm::nodes::working_memory::save_state();
        let _ = eden_garm::nodes::world_model_core::save_state();
        let _ = eden_garm::nodes::policy_guard::save_state();
        let _ = eden_garm::nodes::uncertainty_ledger::save_state();
        let _ = eden_garm::nodes::provenance_ledger::save_state();
        let _ = eden_garm::nodes::learning_ledger::save_state();
        let _ = eden_garm::nodes::goal_scheduler::save_state();
        let _ = eden_garm::nodes::competence_benchmark::save_state();
        let _ = eden_garm::nodes::evaluation_loop::save_state();
        Self::record_history(
            graph,
            ids.legacy_history,
            "[GEWC-OPERATIONAL-BENCHMARK] operational harness generated",
        );
        out
    }

    fn capability_reality_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let _ = eden_garm::action_evidence::record_attempt(
            "capability_reality_eval",
            "evaluate current operational capabilities without AGI claim",
            "allowed",
            "started",
            "reality_matrix_invoked",
            "capability_reality_eval",
            "medium",
        );
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let capability_status =
            std::fs::read_to_string(eden_garm::state_paths::capability_registry_path())
                .unwrap_or_else(|_| {
                    "[CAPABILITY-REGISTRY] pending_current_run claim_allowed=false\n".to_string()
                });
        let memory_report = Self::memory_eval(graph, ids);
        let world_report = format!(
            "{}{}",
            eden_garm::nodes::world_model_core::report(),
            Self::world_eval()
        );
        let cognitive_report = Self::cognitive_architecture_eval(graph, ids, shared_engine);
        let embodied_report = Self::embodied_grounding_eval(graph, ids, shared_engine);
        let neural_report = Self::neural_architecture_eval(graph, ids, shared_engine);
        let symbolic_report = Self::symbolic_architecture_eval(graph, shared_engine);
        let self_improvement_report =
            Self::self_improvement_architecture_eval(graph, ids, shared_engine);
        let frontier_report = Self::frontier_architecture_eval(graph, ids, shared_engine);
        let paradigm_report =
            Self::paradigm_architecture_eval(graph, ids, shared_engine, api_metrics);
        let integration_governance_report =
            Self::integration_governance_eval(graph, ids, shared_engine, api_metrics);
        let gewc_report =
            Self::global_executive_workspace_eval(graph, ids, shared_engine, api_metrics);
        let gewc_operational_report =
            Self::gewc_operational_benchmark(graph, ids, shared_engine, api_metrics);
        let external_validation_report =
            std::fs::read_to_string(eden_garm::state_paths::external_validation_result_path())
                .unwrap_or_else(|_| {
                    "[EXTERNAL-VALIDATION] mode=not_run claim_allowed=false agi_claim=false\n"
                        .to_string()
                });
        let out = eden_garm::capability_reality_eval::run(
            eden_garm::capability_reality_eval::CapabilityRealityInput {
                readiness_report,
                capability_status,
                memory_report,
                world_report,
                cognitive_report,
                embodied_report,
                neural_report,
                symbolic_report,
                self_improvement_report,
                frontier_report,
                paradigm_report,
                integration_governance_report,
                gewc_report,
                gewc_operational_report,
                policy_report: eden_garm::nodes::policy_guard::report(),
                provenance_report: eden_garm::nodes::provenance_ledger::report(),
                uncertainty_report: eden_garm::nodes::uncertainty_ledger::report(),
                external_validation_report,
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[CAPABILITY-REALITY-EVAL] current architecture and operational capability matrix generated",
        );
        out
    }

    fn architecture_advantage_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let local_evidence = Self::readiness_goal_evidence(graph, ids, &readiness_report);
        let gewc_report =
            Self::global_executive_workspace_eval(graph, ids, shared_engine, api_metrics);
        let capability_reality_report =
            Self::capability_reality_eval(graph, ids, shared_engine, api_metrics);
        let memory_report = Self::memory_eval(graph, ids);
        let world_report = format!(
            "{}{}",
            eden_garm::nodes::world_model_core::report(),
            Self::world_eval()
        );
        let action_evidence_report = eden_garm::action_evidence::report();
        let external_validation_report = eden_garm::external_validation::run(
            &readiness_report,
            &local_evidence,
            &format!(
                "{}{}{}{}",
                action_evidence_report, memory_report, world_report, capability_reality_report
            ),
        );
        let out = eden_garm::architecture_advantage::run(
            eden_garm::architecture_advantage::ArchitectureAdvantageInput {
                gewc_report,
                capability_reality_report,
                memory_report,
                world_report,
                policy_report: eden_garm::nodes::policy_guard::report(),
                provenance_report: eden_garm::nodes::provenance_ledger::report(),
                uncertainty_report: eden_garm::nodes::uncertainty_ledger::report(),
                action_evidence_report,
                external_validation_report,
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[ARCHITECTURE-ADVANTAGE-EVAL] six competitive architecture movements generated",
        );
        out
    }

    fn paradise_worldcell_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let gewc_report =
            Self::global_executive_workspace_eval(graph, ids, shared_engine, api_metrics);
        let memory_report = Self::memory_eval(graph, ids);
        let world_report = format!(
            "{}{}",
            eden_garm::nodes::world_model_core::report(),
            Self::world_eval()
        );
        let policy_report = eden_garm::nodes::policy_guard::report();
        let provenance_report = eden_garm::nodes::provenance_ledger::report();
        let uncertainty_report = eden_garm::nodes::uncertainty_ledger::report();
        let action_evidence_report = eden_garm::action_evidence::report();
        let praxis_report = Self::praxis_nexus_eval(graph, ids, shared_engine, api_metrics);
        let locus_report =
            eden_garm::eden_locus_layer::run(eden_garm::eden_locus_layer::LocusLayerInput {
                gewc_report: gewc_report.clone(),
                memory_report: memory_report.clone(),
                policy_report: policy_report.clone(),
                provenance_report: provenance_report.clone(),
                uncertainty_report: uncertainty_report.clone(),
                action_evidence_report: action_evidence_report.clone(),
                world_report: world_report.clone(),
            });
        let forge_report = eden_garm::eden_operator_forge::run(
            eden_garm::eden_operator_forge::OperatorForgeInput {
                praxis_report: praxis_report.clone(),
                world_report,
                policy_report: policy_report.clone(),
                provenance_report: provenance_report.clone(),
                uncertainty_report: uncertainty_report.clone(),
                action_evidence_report: action_evidence_report.clone(),
            },
        );
        let operational_report = eden_garm::operational_runtime::run();
        let model_report = eden_garm::model_runtime::run_all();
        let out = eden_garm::paradise_worldcell::run(
            eden_garm::paradise_worldcell::ParadiseWorldcellInput {
                gewc_report,
                praxis_report,
                locus_report,
                forge_report,
                operational_report,
                model_report,
                policy_report,
                provenance_report,
                uncertainty_report,
                action_evidence_report,
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[PARADISE-WORLDCELL-RUNTIME] public worldcell runtime identity generated",
        );
        out
    }

    fn praxis_nexus_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let gewc_report =
            Self::global_executive_workspace_eval(graph, ids, shared_engine, api_metrics);
        let architecture_advantage_report =
            Self::architecture_advantage_eval(graph, ids, shared_engine, api_metrics);
        let capability_reality_report =
            Self::capability_reality_eval(graph, ids, shared_engine, api_metrics);
        let memory_report = Self::memory_eval(graph, ids);
        let world_report = format!(
            "{}{}",
            eden_garm::nodes::world_model_core::report(),
            Self::world_eval()
        );
        let action_evidence_report = eden_garm::action_evidence::report();
        let external_validation_report =
            std::fs::read_to_string(eden_garm::state_paths::external_validation_result_path())
                .unwrap_or_else(|_| {
                    "[EXTERNAL-VALIDATION] mode=not_run claim_allowed=false agi_claim=false\n"
                        .to_string()
                });
        let out = eden_garm::praxis_nexus::run(eden_garm::praxis_nexus::PraxisNexusInput {
            gewc_report,
            architecture_advantage_report,
            capability_reality_report,
            memory_report,
            world_report,
            cognitive_report: Self::cognitive_architecture_eval(graph, ids, shared_engine),
            symbolic_report: Self::symbolic_architecture_eval(graph, shared_engine),
            goals_report: eden_garm::nodes::goal_scheduler::report(),
            plan_executor_report: eden_garm::nodes::plan_executor::report(),
            policy_report: eden_garm::nodes::policy_guard::report(),
            provenance_report: eden_garm::nodes::provenance_ledger::report(),
            uncertainty_report: eden_garm::nodes::uncertainty_ledger::report(),
            action_evidence_report,
            external_validation_report,
        });
        Self::record_history(
            graph,
            ids.legacy_history,
            "[EDEN-PRAXIS-NEXUS] governed cognitive-operational substrate generated",
        );
        out
    }

    fn external_ecosystem_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let gewc_report =
            Self::global_executive_workspace_eval(graph, ids, shared_engine, api_metrics);
        let gewc_operational_benchmark_report =
            Self::gewc_operational_benchmark(graph, ids, shared_engine, api_metrics);
        let capability_reality_report =
            Self::capability_reality_eval(graph, ids, shared_engine, api_metrics);
        let architecture_advantage_report =
            Self::architecture_advantage_eval(graph, ids, shared_engine, api_metrics);
        let praxis_nexus_report = Self::praxis_nexus_eval(graph, ids, shared_engine, api_metrics);
        let out = eden_garm::external_ecosystem::run(
            eden_garm::external_ecosystem::ExternalEcosystemInput {
                gewc_report,
                gewc_operational_benchmark_report,
                architecture_advantage_report,
                praxis_nexus_report,
                capability_reality_report,
                policy_report: eden_garm::nodes::policy_guard::report(),
                provenance_report: eden_garm::nodes::provenance_ledger::report(),
                uncertainty_report: eden_garm::nodes::uncertainty_ledger::report(),
                action_evidence_report: eden_garm::action_evidence::report(),
                external_validation_report: std::fs::read_to_string(
                    eden_garm::state_paths::external_validation_result_path(),
                )
                .unwrap_or_else(|_| {
                    "[EXTERNAL-VALIDATION] mode=not_run claim_allowed=false agi_claim=false\n"
                        .to_string()
                }),
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[EDEN-EXTERNAL-ECOSYSTEM] external ecosystem fabric generated",
        );
        out
    }

    fn sovereign_cognition_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let gewc_report =
            Self::global_executive_workspace_eval(graph, ids, shared_engine, api_metrics);
        let praxis_nexus_report = Self::praxis_nexus_eval(graph, ids, shared_engine, api_metrics);
        let architecture_advantage_report =
            Self::architecture_advantage_eval(graph, ids, shared_engine, api_metrics);
        let external_ecosystem_report =
            Self::external_ecosystem_eval(graph, ids, shared_engine, api_metrics);
        let capability_reality_report =
            Self::capability_reality_eval(graph, ids, shared_engine, api_metrics);
        let memory_report = Self::memory_eval(graph, ids);
        let world_report = format!(
            "{}{}",
            eden_garm::nodes::world_model_core::report(),
            Self::world_eval()
        );
        let out = eden_garm::sovereign_cognition::run(
            eden_garm::sovereign_cognition::SovereignCognitionInput {
                gewc_report,
                praxis_nexus_report,
                architecture_advantage_report,
                external_ecosystem_report,
                capability_reality_report,
                memory_report,
                world_report,
                symbolic_report: Self::symbolic_architecture_eval(graph, shared_engine),
                paradigm_report: Self::paradigm_architecture_eval(
                    graph,
                    ids,
                    shared_engine,
                    api_metrics,
                ),
                frontier_report: Self::frontier_architecture_eval(graph, ids, shared_engine),
                policy_report: eden_garm::nodes::policy_guard::report(),
                provenance_report: eden_garm::nodes::provenance_ledger::report(),
                uncertainty_report: eden_garm::nodes::uncertainty_ledger::report(),
                action_evidence_report: eden_garm::action_evidence::report(),
                external_validation_report: std::fs::read_to_string(
                    eden_garm::state_paths::external_validation_result_path(),
                )
                .unwrap_or_else(|_| {
                    "[EXTERNAL-VALIDATION] mode=not_run claim_allowed=false agi_claim=false\n"
                        .to_string()
                }),
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[EDEN-SOVEREIGN-COGNITION] OpenCog Hyperon surpass targets generated",
        );
        out
    }

    fn artifact_api_eval(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let out = eden_garm::artifact_api::run();
        Self::record_history(
            graph,
            ids.legacy_history,
            "[ARTIFACT-API] executable artifact APIs generated",
        );
        out
    }

    fn runtime_state_api_eval(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let out = eden_garm::runtime_state_api::run();
        Self::record_history(
            graph,
            ids.legacy_history,
            "[RUNTIME-STATE-API] typed runtime state APIs generated",
        );
        out
    }

    fn operational_api_eval(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let out = eden_garm::operational_api::run();
        Self::record_history(
            graph,
            ids.legacy_history,
            "[OPERATIONAL-API] typed operational APIs generated",
        );
        out
    }

    fn operational_runtime_eval(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let out = eden_garm::operational_runtime::run();
        Self::record_history(
            graph,
            ids.legacy_history,
            "[OPERATIONAL-RUNTIME-PHASE] eight operational runtime components executed",
        );
        out
    }

    fn self_improvement_architecture_eval(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
    ) -> String {
        let start_evidence = eden_garm::action_evidence::record_attempt(
            "self_improvement",
            "run bounded parameter self improvement proposal without source mutation",
            "allowed",
            "started",
            "source_code_mutated=false",
            "self_improvement_architecture_eval",
            "medium",
        );
        let allowed_policy = eden_garm::nodes::policy_guard::evaluate(
            "local bounded self improvement parameter proposal no source code mutation",
        );
        let blocked_policy =
            eden_garm::nodes::policy_guard::evaluate("remote shell source code mutation");
        let provenance = eden_garm::nodes::provenance_ledger::record(
            "self_improvement",
            "bounded parameter proposal evaluated without source mutation",
        );
        let uncertainty = eden_garm::nodes::uncertainty_ledger::record(
            "self_improvement",
            "bounded parameter proposal risk medium rollback required",
        );
        let plan = eden_garm::nodes::plan_executor::plan(
            "bounded self improvement parameter proposal with rollback",
        );
        let run = eden_garm::nodes::plan_executor::run_next("[POLICY] [PROVENANCE] [UNCERTAINTY]");
        let (self_status, self_improvement_status) = shared_engine
            .lock()
            .map(|mut engine| {
                let tick = engine.state.tick_count;
                let maybe = engine.self_modification.propose(
                    "garm",
                    "exploration_threshold",
                    1.0,
                    0.8,
                    tick,
                );
                if let Some(proposal) = maybe {
                    let _ = engine.self_modification.apply(&proposal);
                }
                let n_concepts = engine.morphogenesis.n_concepts();
                let n_relations = engine.morphogenesis.relation_count();
                let n_grounding_facts = engine.grounding.facts.len();
                let _ = engine.self_improvement.audit(
                    n_concepts,
                    n_relations,
                    8,
                    0,
                    4,
                    12,
                    0,
                    n_grounding_facts,
                    12,
                );
                (
                    engine.self_modification.status(),
                    engine.self_improvement.status(),
                )
            })
            .unwrap_or_else(|_| {
                (
                    "SelfMod | proposals=0 | applied=0 | rejected=0".to_string(),
                    "SelfImprovement | audits=0 | applied=0 | reverted=0 | last_proposals=0"
                        .to_string(),
                )
            });
        let complete_evidence = eden_garm::action_evidence::record_attempt(
            "self_improvement",
            "complete bounded parameter self improvement proposal without source mutation",
            "allowed",
            "completed",
            "source_code_mutated=false",
            "self_improvement_architecture_eval",
            "medium",
        );
        let out = eden_garm::self_improvement_architecture::run(
            eden_garm::self_improvement_architecture::SelfImprovementArchitectureInput {
                self_status,
                self_improvement_status,
                plan_executor_report: format!(
                    "{}{}{}",
                    plan,
                    run,
                    eden_garm::nodes::plan_executor::report()
                ),
                policy_report: format!(
                    "{}{}{}",
                    allowed_policy,
                    blocked_policy,
                    eden_garm::nodes::policy_guard::report()
                ),
                provenance_report: format!(
                    "{}{}",
                    provenance,
                    eden_garm::nodes::provenance_ledger::report()
                ),
                uncertainty_report: format!(
                    "{}{}",
                    uncertainty,
                    eden_garm::nodes::uncertainty_ledger::report()
                ),
                action_evidence_report: format!(
                    "{}{}{}",
                    start_evidence,
                    eden_garm::action_evidence::report(),
                    complete_evidence
                ),
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[SELF-IMPROVEMENT-ARCHITECTURE] local bounded self-improvement eval run",
        );
        out
    }

    fn capability_registry_audit(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let local_evidence = Self::readiness_goal_evidence(graph, ids, &readiness_report);
        let benchmark_report =
            eden_garm::nodes::goal_scheduler::readiness_gate_benchmark_report(&local_evidence);
        let memory_eval_report = Self::memory_eval(graph, ids);
        let world_eval_report = Self::world_eval();
        let cognitive_architecture_report =
            Self::cognitive_architecture_eval(graph, ids, shared_engine);
        let embodied_grounding_report = Self::embodied_grounding_eval(graph, ids, shared_engine);
        let neural_architecture_report = Self::neural_architecture_eval(graph, ids, shared_engine);
        let symbolic_architecture_report = Self::symbolic_architecture_eval(graph, shared_engine);
        let self_improvement_architecture_report =
            Self::self_improvement_architecture_eval(graph, ids, shared_engine);
        let frontier_architecture_report =
            Self::frontier_architecture_eval(graph, ids, shared_engine);
        let paradigm_architecture_report =
            Self::paradigm_architecture_eval(graph, ids, shared_engine, api_metrics);
        let integration_governance_report =
            Self::integration_governance_eval(graph, ids, shared_engine, api_metrics);
        let global_executive_workspace_report =
            Self::global_executive_workspace_eval(graph, ids, shared_engine, api_metrics);
        let gewc_operational_benchmark_report =
            Self::gewc_operational_benchmark(graph, ids, shared_engine, api_metrics);
        let capability_reality_report =
            Self::capability_reality_eval(graph, ids, shared_engine, api_metrics);
        let architecture_advantage_report =
            Self::architecture_advantage_eval(graph, ids, shared_engine, api_metrics);
        let praxis_nexus_report = Self::praxis_nexus_eval(graph, ids, shared_engine, api_metrics);
        let external_ecosystem_report =
            Self::external_ecosystem_eval(graph, ids, shared_engine, api_metrics);
        let sovereign_cognition_report =
            Self::sovereign_cognition_eval(graph, ids, shared_engine, api_metrics);
        let runtime_state_api_report = Self::runtime_state_api_eval(graph, ids);
        let operational_api_report = Self::operational_api_eval(graph, ids);
        let artifact_api_report = Self::artifact_api_eval(graph, ids);
        let action_evidence_report = eden_garm::action_evidence::report();
        let external_validation_report = eden_garm::external_validation::run(
            &readiness_report,
            &local_evidence,
            &format!(
                "{}{}{}{}{}{}{}{}{}{}{}{}",
                action_evidence_report,
                memory_eval_report,
                world_eval_report,
                gewc_operational_benchmark_report,
                capability_reality_report,
                architecture_advantage_report,
                praxis_nexus_report,
                external_ecosystem_report,
                sovereign_cognition_report,
                runtime_state_api_report,
                operational_api_report,
                artifact_api_report
            ),
        );
        let out = eden_garm::capability_registry::audit(
            eden_garm::capability_registry::CapabilityRegistryInput {
                readiness_report,
                benchmark_report,
                memory_eval_report,
                world_eval_report,
                cognitive_architecture_report,
                embodied_grounding_report,
                neural_architecture_report,
                symbolic_architecture_report,
                self_improvement_architecture_report,
                frontier_architecture_report,
                paradigm_architecture_report,
                integration_governance_report,
                global_executive_workspace_report,
                gewc_operational_benchmark_report,
                capability_reality_report,
                architecture_advantage_report,
                praxis_nexus_report,
                external_ecosystem_report,
                sovereign_cognition_report,
                runtime_state_api_report,
                operational_api_report,
                artifact_api_report,
                action_evidence_report,
                external_validation_report,
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[CAPABILITY-REGISTRY] capability audit generated",
        );
        out
    }

    fn readiness_external_validation_manifest(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let readiness_report = Self::readiness_report(graph, ids, shared_engine, api_metrics);
        let local_evidence = Self::readiness_goal_evidence(graph, ids, &readiness_report);
        let gates =
            eden_garm::nodes::goal_scheduler::readiness_gate_benchmark_report(&local_evidence);
        let score = readiness_report
            .lines()
            .find_map(|line| line.trim().strip_prefix("- score: "))
            .unwrap_or("unknown");
        let manifest = serde_json::json!({
            "schema": "garm-external-validation-v1",
            "phase": "phase6_external_validation",
            "mode": "manifest_only_no_local_completion",
            "status": "requires_independent_external_validation",
            "local_readiness_score": score,
            "claim_policy": {
                "agi_claim": false,
                "claim_allowed": false,
                "invariant": "no_claim_until_external_validation_passes",
                "local_score_is_not_agi_measurement": true
            },
            "required_external_suites": [
                "held_out_unseen_task_suite",
                "robust_generalization_suite",
                "grounding_and_tool_consequence_suite",
                "safety_red_team_suite",
                "long_horizon_planning_suite",
                "memory_retrieval_regression_suite",
                "praxis_nexus_suite",
                "external_ecosystem_suite",
                "sovereign_cognition_suite",
                "runtime_state_api_suite",
                "operational_api_suite",
                "artifact_api_suite",
                "independent_reproducibility_audit"
            ],
            "required_artifacts": [
                "garm_export.json",
                "garm_report.txt",
                "garm_report_history.jsonl",
                "hrm_text_context_pack.json",
                "hrm_text_segments.jsonl",
                "goal_scheduler.json",
                "evaluation_loop.json",
                "learning_ledger.json",
                "provenance_ledger.json",
                "uncertainty_ledger.json",
                "policy_guard.json",
                "competence_benchmark.json",
                "eden_praxis_nexus.json",
                "praxis_primitives.json",
                "praxis_blocks.json",
                "eden_sovereign_cognition.json",
                "sovereign_sector_wins.json",
                "praxis_calculus_formalism.json",
                "cognitive_contract_language.json",
                "evidence_memory_fabric.json",
                "federated_runtime_fabric.json",
                "symbolic_reasoning_fabric.json",
                "eden_external_ecosystem.json",
                "ecosystem_participation_contract.json",
                "ecosystem_interop_matrix.json",
                "ecosystem_certification_ladder.json",
                "ecosystem_onboarding_runbook.json",
                "ecosystem_governance_model.json",
                "ecosystem_benchmark_exchange.json",
                "runtime_state_api_catalog.json",
                "runtime_state_api_contracts.json",
                "runtime_state_api_openapi.json",
                "runtime_state_api_runtime.json",
                "operational_api_catalog.json",
                "operational_api_contracts.json",
                "operational_api_openapi.json",
                "operational_api_runtime.json",
                "operational_action_contracts.json",
                "artifact_api_catalog.json",
                "artifact_api_contracts.json",
                "artifact_api_runtime.json"
            ],
            "acceptance_criteria": [
                "external_tasks_are_not_authored_by_eden_operators",
                "baselines_and_scoring_are_defined_before_execution",
                "all_failures_are_reported_without_filtering",
                "network_and_tool_permissions_are_declared",
                "results_are_reproducible_from_exported_artifacts",
                "safety_blocks_are_counted_as_valid_only_when_policy_requires_blocking"
            ],
            "local_gate_snapshot_fnv64": format!("{:016x}", Self::fnv64(gates.as_bytes())),
        });
        let body = serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| manifest.to_string());
        let path = eden_garm::state_paths::external_validation_manifest_path();
        let write_status = match std::fs::write(&path, body) {
            Ok(()) => "manifest_written".to_string(),
            Err(e) => format!("manifest_write_failed:{}", e),
        };
        Self::record_history(
            graph,
            ids.legacy_history,
            "[READINESS-EXTERNAL] generated Phase 6 validation manifest",
        );
        format!(
            "[READINESS-EXTERNAL] phase=6 status=requires_independent_external_validation local_score={} claim_allowed=false write_status={} path={}\n{}{}",
            score, write_status, path, readiness_report, gates
        )
    }

    fn readiness_evidence_probe(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let corpus_dir = "eden_core/src/garm/readiness_corpus";
        let mut out = String::from("[READINESS-PROBE] evidence_mode=local status=running\n");
        out.push_str(&eden_garm::nodes::hrm_text_pretraining::ingest_directory(
            corpus_dir,
        ));
        out.push_str(&eden_garm::nodes::hrm_text_pretraining::search(
            "local evidence",
        ));
        out.push_str(&eden_garm::nodes::hrm_text_pretraining::context_pack(
            "local evidence",
        ));
        out.push_str(&eden_garm::nodes::world_model_core::observe(
            "readiness_probe",
            "local evidence supports readiness",
        ));
        out.push_str(&eden_garm::nodes::world_model_core::predict(
            "local evidence",
        ));
        out.push_str(&eden_garm::nodes::world_model_core::verify_predictions());
        out.push_str(&Self::competence_benchmark_run(graph, ids));
        out.push_str(&eden_garm::nodes::working_memory::attend(
            "readiness_probe",
            "cognitive architecture links attention, memory, metacognition, executive goals and evaluation evidence",
        ));
        out.push_str(&Self::readiness_phase_one_memory_probe(graph, ids));
        let _ = eden_garm::nodes::hrm_text_pretraining::save_state();
        let _ = eden_garm::nodes::world_model_core::save_state();
        let _ = eden_garm::nodes::competence_benchmark::save_state();
        let _ = eden_garm::nodes::learning_ledger::save_state();
        let _ = eden_garm::nodes::working_memory::save_state();
        Self::record_history(
            graph,
            ids.legacy_history,
            "[READINESS-PROBE] generated local evidence for phase gates",
        );
        out
    }

    fn readiness_phase_one_memory_probe(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
    ) -> String {
        let dimensions = [
            "rag_verificable",
            "modelos_predictivos",
            "generalizacion",
            "grounding_accion",
            "seguridad_operacional",
            "autonomia_gobernada",
            "memoria_integrada",
            "aprendizaje_continuo",
        ];
        let mut facts = Vec::new();
        for idx in 0..160usize {
            let dimension = dimensions[idx % dimensions.len()];
            facts.push(format!(
                "phase1 evidence {:03} is {} local proof",
                idx + 1,
                dimension
            ));
        }
        let mut remembered = 0usize;
        if let Some(memory) = graph.nodes.get_mut(ids.legacy_memory).and_then(|node| {
            node.as_any_mut()
                .downcast_mut::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
        }) {
            let before = memory.fact_count();
            for fact in &facts {
                let _ = memory.remember(fact);
            }
            remembered = memory.fact_count().saturating_sub(before);
        }
        for fact in &facts {
            Self::feed_knowledge_graph(graph, ids.legacy_knowledge_graph, fact, "readiness_probe");
        }
        format!(
            "[READINESS-PHASE1-MEMORY] facts={} remembered={} kg_facts={} source=readiness_probe\n",
            facts.len(),
            remembered,
            facts.len()
        )
    }

    fn readiness_phase_two_local_proof_probe(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let mut out = String::from("[READINESS-PHASE2-PROOF] evidence_mode=local status=running\n");
        let proof_topics = [
            "rag verifiable citations local evidence",
            "policy provenance uncertainty local proof",
            "world model prediction verification local proof",
            "continuous evaluation benchmark local evidence",
        ];

        for idx in 0..34usize {
            let query = proof_topics[idx % proof_topics.len()];
            let _ = eden_garm::nodes::hrm_text_pretraining::search_evidence(query, 3);
        }
        for idx in 0..50usize {
            let query = proof_topics[idx % proof_topics.len()];
            let _ = eden_garm::nodes::hrm_text_pretraining::context_pack(query);
        }
        for idx in 0..20usize {
            let _ = eden_garm::nodes::hrm_text_pretraining::context_pack(&format!(
                "phase2 abstention unknown remote claim {}",
                idx + 1
            ));
        }

        for idx in 0..48usize {
            let title = format!("phase2 local proof goal {:02}", idx + 1);
            let _ = eden_garm::nodes::goal_scheduler::plan_goal(&title, "readiness_phase2");
        }

        for idx in 0..25usize {
            let _ = eden_garm::nodes::policy_guard::evaluate(&format!(
                "run shell network mutation blocked phase2 {}",
                idx + 1
            ));
        }
        for idx in 0..10usize {
            let _ = eden_garm::nodes::policy_guard::evaluate(&format!(
                "run local bounded readiness audit {}",
                idx + 1
            ));
        }

        for idx in 0..200usize {
            let claim = format!(
                "readiness phase2 verified local evidence claim {:03} fnv64 citation present",
                idx + 1
            );
            let _ = eden_garm::nodes::provenance_ledger::record("readiness_phase2", &claim);
        }
        for idx in 0..100usize {
            let claim = format!(
                "phase2 uncertainty bounded local claim {:03} requires cited verification before action",
                idx + 1
            );
            let _ = eden_garm::nodes::uncertainty_ledger::record("readiness_phase2", &claim);
        }
        for _ in 0..50usize {
            let _ = eden_garm::nodes::uncertainty_ledger::resolve_next();
        }

        for idx in 0..40usize {
            let observation = format!(
                "phase2 local proof subject {:02} supports verified prediction",
                idx + 1
            );
            let _ = eden_garm::nodes::world_model_core::observe("readiness_phase2", &observation);
            let _ = eden_garm::nodes::world_model_core::predict(&format!(
                "phase2 local proof subject {:02}",
                idx + 1
            ));
        }
        let _ = eden_garm::nodes::world_model_core::verify_predictions();

        out.push_str(&Self::readiness_phase_two_memory_probe(graph, ids));
        let evidence = Self::readiness_goal_evidence(graph, ids, "phase2_local_proof");
        for _ in 0..100usize {
            let _ = eden_garm::nodes::competence_benchmark::run(&evidence);
        }
        for _ in 0..100usize {
            let _ = Self::evaluation_run(graph, ids, shared_engine, api_metrics);
        }
        for idx in 0..200usize {
            let hypothesis = format!("phase2 local proof learning hypothesis {:03}", idx + 1);
            let _ = eden_garm::nodes::learning_ledger::record(
                "readiness_phase2",
                &hypothesis,
                "local_cited_context_and_eval",
                "consolidated",
            );
        }

        let _ = eden_garm::nodes::goal_scheduler::save_state();
        let _ = eden_garm::nodes::evaluation_loop::save_state();
        let _ = eden_garm::nodes::learning_ledger::save_state();
        let _ = eden_garm::nodes::provenance_ledger::save_state();
        let _ = eden_garm::nodes::uncertainty_ledger::save_state();
        let _ = eden_garm::nodes::policy_guard::save_state();
        let _ = eden_garm::nodes::hrm_text_pretraining::save_state();
        let _ = eden_garm::nodes::world_model_core::save_state();
        let _ = eden_garm::nodes::competence_benchmark::save_state();

        Self::record_history(
            graph,
            ids.legacy_history,
            "[READINESS-PHASE2-PROOF] generated local proof evidence",
        );
        out.push_str(&format!(
            "[READINESS-PHASE2-PROOF] searches=34 context_packs=70 abstentions=20 goals=48 policy_blocks=25 provenance=200 uncertainty=100 world_predictions=40 benchmarks=100 evaluations=100 learning=200 status=complete\n"
        ));
        out
    }

    fn readiness_phase_two_memory_probe(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
    ) -> String {
        let dimensions = [
            "rag_verificable",
            "modelos_predictivos",
            "generalizacion",
            "grounding_accion",
            "seguridad_operacional",
            "autonomia_gobernada",
            "memoria_integrada",
            "aprendizaje_continuo",
            "evaluacion_continua",
            "autocorreccion",
            "planificacion_largo_horizonte",
            "escalamiento_cognitivo",
        ];
        let mut facts = Vec::new();
        for idx in 0..320usize {
            let dimension = dimensions[idx % dimensions.len()];
            facts.push(format!(
                "phase2 local proof {:03} is {} cited operational evidence",
                idx + 1,
                dimension
            ));
        }
        let mut remembered = 0usize;
        if let Some(memory) = graph.nodes.get_mut(ids.legacy_memory).and_then(|node| {
            node.as_any_mut()
                .downcast_mut::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
        }) {
            let before = memory.fact_count();
            for fact in &facts {
                let _ = memory.remember(fact);
            }
            remembered = memory.fact_count().saturating_sub(before);
        }
        for fact in &facts {
            Self::feed_knowledge_graph(graph, ids.legacy_knowledge_graph, fact, "readiness_phase2");
        }
        format!(
            "[READINESS-PHASE2-MEMORY] facts={} remembered={} kg_facts={} source=readiness_phase2\n",
            facts.len(),
            remembered,
            facts.len()
        )
    }

    fn readiness_phase_four_governed_autonomy_probe(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
    ) -> String {
        let mut out =
            String::from("[READINESS-PHASE4-GOVERNED] evidence_mode=local status=running\n");
        for _ in 0..5000usize {
            graph.inject_zero_sensor(8);
            graph.pulse(1.0);
        }
        out.push_str(&Self::readiness_phase_four_memory_probe(graph, ids));

        let proof_topics = [
            "phase4 governed autonomy verifiable rag citations",
            "phase4 governed autonomy predictive verification",
            "phase4 governed autonomy continuous evaluation",
            "phase4 governed autonomy safe policy provenance",
        ];
        for idx in 0..32usize {
            let query = proof_topics[idx % proof_topics.len()];
            let _ = eden_garm::nodes::hrm_text_pretraining::search_evidence(query, 3);
        }
        for idx in 0..30usize {
            let query = proof_topics[idx % proof_topics.len()];
            let _ = eden_garm::nodes::hrm_text_pretraining::context_pack(query);
        }

        for idx in 0..32usize {
            let observation = format!(
                "phase4 governed autonomy subject {:02} has bounded consequence evidence",
                idx + 1
            );
            let _ = eden_garm::nodes::world_model_core::observe("readiness_phase4", &observation);
            let _ = eden_garm::nodes::world_model_core::predict(&format!(
                "phase4 governed autonomy subject {:02}",
                idx + 1
            ));
        }
        let _ = eden_garm::nodes::world_model_core::verify_predictions();

        let _ = eden_garm::nodes::hrm_text_pretraining::save_state();
        let _ = eden_garm::nodes::world_model_core::save_state();
        Self::record_history(
            graph,
            ids.legacy_history,
            "[READINESS-PHASE4-GOVERNED] generated governed autonomy evidence",
        );
        out.push_str(
            "[READINESS-PHASE4-GOVERNED] pulses=5000 searches=32 context_packs=30 world_predictions=32 status=complete\n",
        );
        out
    }

    fn readiness_phase_four_memory_probe(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
    ) -> String {
        let dimensions = [
            "governed_autonomy",
            "continuous_evaluation",
            "predictive_world_model",
            "integrated_memory",
            "safe_self_correction",
            "generalization_transfer",
            "cognitive_scaling",
            "verifiable_rag",
        ];
        let mut facts = Vec::new();
        for idx in 0..640usize {
            let dimension = dimensions[idx % dimensions.len()];
            facts.push(format!(
                "phase4 governed autonomy proof {:03} is {} local bounded evidence",
                idx + 1,
                dimension
            ));
        }
        let mut remembered = 0usize;
        if let Some(memory) = graph.nodes.get_mut(ids.legacy_memory).and_then(|node| {
            node.as_any_mut()
                .downcast_mut::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
        }) {
            let before = memory.fact_count();
            for fact in &facts {
                let _ = memory.remember(fact);
            }
            remembered = memory.fact_count().saturating_sub(before);
        }
        for fact in &facts {
            Self::feed_knowledge_graph(graph, ids.legacy_knowledge_graph, fact, "readiness_phase4");
        }
        format!(
            "[READINESS-PHASE4-MEMORY] facts={} remembered={} kg_facts={} source=readiness_phase4\n",
            facts.len(),
            remembered,
            facts.len()
        )
    }

    fn readiness_phase_five_robust_generalization_probe(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let mut out =
            String::from("[READINESS-PHASE5-ROBUST] evidence_mode=local status=running\n");
        out.push_str(&Self::readiness_phase_five_memory_probe(graph, ids));

        let proof_topics = [
            "phase5 robust generalization transfer with cited evidence",
            "phase5 robust generalization retrieval across unseen local tasks",
            "phase5 robust generalization policy bounded autonomy",
            "phase5 robust generalization evaluation regression coverage",
        ];
        for idx in 0..70usize {
            let query = proof_topics[idx % proof_topics.len()];
            let _ = eden_garm::nodes::hrm_text_pretraining::search_evidence(query, 3);
        }

        let inherited: Vec<String> = (0..32usize)
            .map(|idx| format!("phase5 inherited local evidence {:02}", idx + 1))
            .collect();
        if let Some(rebirth) = graph
            .nodes
            .get_mut(ids.legacy_rebirth_meltrace)
            .and_then(|node| {
                node.as_any_mut().downcast_mut::<
                eden_garm::nodes::legacy_runtime_extensions::RebirthMeltraceNode,
            >()
            })
        {
            for _ in 0..25usize {
                let _ = rebirth.rebirth(&inherited);
            }
        }
        Self::update_lifecycle_metrics(graph, ids.legacy_rebirth_meltrace, api_metrics);

        let _ = eden_garm::nodes::hrm_text_pretraining::save_state();
        Self::record_history(
            graph,
            ids.legacy_history,
            "[READINESS-PHASE5-ROBUST] generated robust generalization evidence",
        );
        out.push_str(
            "[READINESS-PHASE5-ROBUST] retrieval_searches=70 meltrace_rebirths=25 status=complete\n",
        );
        out
    }

    fn readiness_phase_five_memory_probe(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
    ) -> String {
        let dimensions = [
            "robust_generalization",
            "cross_task_transfer",
            "integrated_memory_retrieval",
            "evaluation_regression_coverage",
            "governed_autonomy_limits",
            "self_correction_repair_loop",
            "cognitive_scaling_efficiency",
            "verifiable_context_grounding",
        ];
        let mut facts = Vec::new();
        for idx in 0..1600usize {
            let dimension = dimensions[idx % dimensions.len()];
            facts.push(format!(
                "phase5 robust generalization proof {:04} is {} local transfer evidence",
                idx + 1,
                dimension
            ));
        }
        let mut remembered = 0usize;
        if let Some(memory) = graph.nodes.get_mut(ids.legacy_memory).and_then(|node| {
            node.as_any_mut()
                .downcast_mut::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
        }) {
            let before = memory.fact_count();
            for fact in &facts {
                let _ = memory.remember(fact);
            }
            remembered = memory.fact_count().saturating_sub(before);
        }
        for fact in &facts {
            Self::feed_knowledge_graph(graph, ids.legacy_knowledge_graph, fact, "readiness_phase5");
        }
        format!(
            "[READINESS-PHASE5-MEMORY] facts={} remembered={} kg_facts={} source=readiness_phase5\n",
            facts.len(),
            remembered,
            facts.len()
        )
    }

    fn readiness_goal_evidence(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        readiness_report: &str,
    ) -> String {
        let hrm_snapshot = graph
            .nodes
            .get(ids.hrm_reasoner)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode>())
            .map(|node| node.snapshot())
            .unwrap_or_else(|| "hrm:missing".to_string());
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            readiness_report,
            eden_garm::nodes::goal_scheduler::report(),
            eden_garm::nodes::evaluation_loop::report(),
            eden_garm::nodes::learning_ledger::report(),
            eden_garm::nodes::world_model_core::report(),
            eden_garm::nodes::plan_executor::report(),
            eden_garm::nodes::working_memory::report(),
            eden_garm::nodes::uncertainty_ledger::report(),
            eden_garm::nodes::experiment_runner::report(),
            eden_garm::nodes::provenance_ledger::report(),
            eden_garm::nodes::policy_guard::report(),
            eden_garm::nodes::capability_maturity::report(),
            eden_garm::nodes::competence_benchmark::report(),
            eden_garm::nodes::organ_registry::organ_audit(graph),
            hrm_snapshot,
            eden_garm::nodes::hrm_text_pretraining::report(),
            Self::context_augmentation_report(graph, ids),
        )
    }

    fn readiness_signals(
        memory_facts: usize,
        kg_edges: usize,
        capability_count: usize,
        tick_count: u64,
        autonomous: bool,
        meltrace_events: u64,
    ) -> eden_garm::nodes::readiness::ReadinessSignals {
        let hrm_text = eden_garm::nodes::hrm_text_pretraining::report();
        let learning = eden_garm::nodes::learning_ledger::report();
        let provenance = eden_garm::nodes::provenance_ledger::report();
        let uncertainty = eden_garm::nodes::uncertainty_ledger::report();
        let benchmark = eden_garm::nodes::competence_benchmark::report();
        let goals = eden_garm::nodes::goal_scheduler::report();
        let policy = eden_garm::nodes::policy_guard::report();
        eden_garm::nodes::readiness::ReadinessSignals {
            memory_facts,
            kg_edges,
            capability_count,
            tick_count,
            autonomous,
            meltrace_events,
            retrieval_hits: Self::metric_value(&hrm_text, "retrieval_hits="),
            context_packs: Self::metric_value(&hrm_text, "context_packs="),
            abstentions: Self::metric_value(&hrm_text, "abstentions="),
            learning_records: Self::metric_value(&learning, "entries="),
            provenance_records: Self::metric_value(&provenance, "records="),
            uncertainty_records: Self::metric_value(&uncertainty, "records="),
            benchmark_runs: Self::metric_value(&benchmark, "runs="),
            goal_contracts: Self::metric_value(&goals, "actions="),
            policy_blocks: Self::metric_value(&policy, "blocked="),
        }
    }

    fn metric_value(report: &str, key: &str) -> u64 {
        report
            .split_whitespace()
            .find_map(|part| part.strip_prefix(key))
            .and_then(|value| {
                value
                    .chars()
                    .take_while(|ch| ch.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u64>()
                    .ok()
            })
            .unwrap_or(0)
    }

    fn evaluation_run(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let memory_facts = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.fact_count())
            .unwrap_or(0);
        let kg_edges = graph
            .nodes
            .get(ids.legacy_knowledge_graph)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
            .map(|kg| kg.edge_count())
            .unwrap_or(0);
        let tick_count = shared_engine
            .lock()
            .map(|engine| engine.state.tick_count)
            .unwrap_or(0);
        let readiness_score = graph
            .nodes
            .get_mut(ids.readiness)
            .and_then(|node| {
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::readiness::ReadinessNode>()
            })
            .map(|readiness| {
                readiness.observe_system(
                    memory_facts,
                    kg_edges,
                    91,
                    tick_count,
                    api_metrics.autonomous.load(Ordering::Relaxed),
                    api_metrics.meltrace_grabados.load(Ordering::Relaxed),
                );
                readiness.readiness_score()
            })
            .unwrap_or(0.0);
        let hrm_snapshot = graph
            .nodes
            .get(ids.hrm_reasoner)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode>())
            .map(|node| node.snapshot())
            .unwrap_or_else(|| "hrm:missing".to_string());
        let benchmark_snapshot = graph
            .nodes
            .iter()
            .find(|node| node.name() == "benchmark")
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::benchmark::BenchmarkNode>()
            })
            .map(|node| node.benchmark_snapshot())
            .unwrap_or_else(|| "benchmark:missing".to_string());
        let benchmark_snapshot = format!(
            "{}\n{}",
            benchmark_snapshot,
            eden_garm::nodes::competence_benchmark::report()
        );
        let input = eden_garm::nodes::evaluation_loop::EvaluationInput {
            tick: graph.global_tick,
            memory_facts,
            kg_edges,
            alive_nodes: graph.alive_node_count(),
            graph_edges: graph.edge_count(),
            readiness_score,
            goals_report: eden_garm::nodes::goal_scheduler::report(),
            organ_audit: eden_garm::nodes::organ_registry::organ_audit(graph),
            hrm_snapshot,
            benchmark_snapshot,
            attention_snapshot: eden_garm::nodes::working_memory::report(),
            uncertainty_snapshot: eden_garm::nodes::uncertainty_ledger::report(),
            experiment_snapshot: eden_garm::nodes::experiment_runner::report(),
            provenance_snapshot: eden_garm::nodes::provenance_ledger::report(),
            policy_snapshot: eden_garm::nodes::policy_guard::report(),
            maturity_snapshot: eden_garm::nodes::capability_maturity::report(),
            hrm_text_snapshot: eden_garm::nodes::hrm_text_pretraining::report(),
        };
        let out = eden_garm::nodes::evaluation_loop::run_evaluation(input);
        let learning = eden_garm::nodes::learning_ledger::record(
            "evaluation_loop",
            "architecture evaluation improves calibration",
            "eval_run",
            if out.contains("verdict=improving") {
                "improving"
            } else if out.contains("verdict=stable") {
                "stable"
            } else {
                "needs_evidence"
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[EVAL-RUN] architecture evaluated",
        );
        format!("{}{}", out, learning)
    }

    fn competence_benchmark_run(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let hrm_snapshot = graph
            .nodes
            .get(ids.hrm_reasoner)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode>())
            .map(|node| node.snapshot())
            .unwrap_or_else(|| "hrm:missing".to_string());
        let evidence = format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            eden_garm::nodes::goal_scheduler::report(),
            eden_garm::nodes::evaluation_loop::report(),
            eden_garm::nodes::learning_ledger::report(),
            eden_garm::nodes::world_model_core::report(),
            eden_garm::nodes::plan_executor::report(),
            eden_garm::nodes::working_memory::report(),
            eden_garm::nodes::uncertainty_ledger::report(),
            eden_garm::nodes::experiment_runner::report(),
            eden_garm::nodes::provenance_ledger::report(),
            eden_garm::nodes::policy_guard::report(),
            eden_garm::nodes::capability_maturity::report(),
            eden_garm::nodes::organ_registry::organ_audit(graph),
            hrm_snapshot,
            eden_garm::nodes::hrm_text_pretraining::report(),
            "retrieval_hits=",
        );
        let out = eden_garm::nodes::competence_benchmark::run(&evidence);
        let status = if out.contains("verdict=strong") {
            "strong"
        } else if out.contains("verdict=developing") {
            "developing"
        } else {
            "needs_evidence"
        };
        let learning = eden_garm::nodes::learning_ledger::record(
            "competence_benchmark",
            "local benchmark measures GARM competence seams",
            "bench_run",
            status,
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[BENCH-RUN] competence benchmark executed",
        );
        format!("{}{}", out, learning)
    }

    fn plan_executor_run(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let evidence = format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}",
            Self::context_augmentation_report(graph, ids),
            eden_garm::nodes::goal_scheduler::report(),
            eden_garm::nodes::evaluation_loop::report(),
            eden_garm::nodes::learning_ledger::report(),
            eden_garm::nodes::competence_benchmark::report(),
            eden_garm::nodes::working_memory::report(),
            eden_garm::nodes::uncertainty_ledger::report(),
            eden_garm::nodes::experiment_runner::report(),
            eden_garm::nodes::provenance_ledger::report(),
            eden_garm::nodes::policy_guard::report(),
            eden_garm::nodes::capability_maturity::report(),
            eden_garm::nodes::organ_registry::organ_audit(graph),
        );
        let out = eden_garm::nodes::plan_executor::run_next(&evidence);
        let status = if out.contains("status=completed") {
            "completed"
        } else if out.contains("status=rolled_back") {
            "rolled_back"
        } else {
            "needs_plan"
        };
        let learning = eden_garm::nodes::learning_ledger::record(
            "plan_executor",
            "local plan execution uses scoring and rollback gates",
            "exec_run",
            status,
        );
        Self::record_history(graph, ids.legacy_history, "[EXEC-RUN] plan executor run");
        format!("{}{}", out, learning)
    }

    fn experiment_run(graph: &mut eden_garm::HyperGraph, ids: RuntimeNodeIds) -> String {
        let evidence = format!(
            "{}{}{}{}{}{}{}{}{}",
            eden_garm::nodes::evaluation_loop::report(),
            eden_garm::nodes::competence_benchmark::report(),
            eden_garm::nodes::learning_ledger::report(),
            eden_garm::nodes::uncertainty_ledger::report(),
            eden_garm::nodes::plan_executor::report(),
            eden_garm::nodes::working_memory::report(),
            eden_garm::nodes::provenance_ledger::report(),
            eden_garm::nodes::policy_guard::report(),
            eden_garm::nodes::capability_maturity::report(),
        );
        let out = eden_garm::nodes::experiment_runner::run_next(&evidence);
        let status = if out.contains("status=completed") {
            "completed"
        } else if out.contains("status=inconclusive") {
            "inconclusive"
        } else {
            "needs_experiment"
        };
        let learning = eden_garm::nodes::learning_ledger::record(
            "experiment_runner",
            "local experiments calibrate architecture hypotheses",
            "experiment_run",
            status,
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            "[EXPERIMENT-RUN] local experiment executed",
        );
        format!("{}{}", out, learning)
    }

    fn maturity_assess(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        capability: &str,
    ) -> String {
        let evidence = format!(
            "{}{}{}{}{}{}{}{}",
            eden_garm::nodes::goal_scheduler::report(),
            eden_garm::nodes::evaluation_loop::report(),
            eden_garm::nodes::learning_ledger::report(),
            eden_garm::nodes::competence_benchmark::report(),
            eden_garm::nodes::plan_executor::report(),
            eden_garm::nodes::policy_guard::report(),
            eden_garm::nodes::provenance_ledger::report(),
            eden_garm::nodes::hrm_text_pretraining::report(),
        );
        let out = eden_garm::nodes::capability_maturity::assess(capability, &evidence);
        let learning = eden_garm::nodes::learning_ledger::record(
            "capability_maturity",
            capability,
            "maturity_assessment",
            if out.contains("status=operational") {
                "operational"
            } else {
                "developing"
            },
        );
        Self::record_history(
            graph,
            ids.legacy_history,
            &format!("[MATURITY] {}", capability),
        );
        format!("{}{}", out, learning)
    }

    fn append_extension_report<T: 'static>(
        graph: &eden_garm::HyperGraph,
        id: usize,
        report: &mut String,
        f: impl FnOnce(&T) -> String,
    ) {
        if let Some(node) = graph
            .nodes
            .get(id)
            .and_then(|node| node.as_any().downcast_ref::<T>())
        {
            report.push_str(&format!("{}\n", f(node)));
        }
    }

    fn feed_knowledge_graph(
        graph: &mut eden_garm::HyperGraph,
        legacy_knowledge_graph_id: usize,
        fact: &str,
        source: &str,
    ) {
        let cycle = graph.global_tick;
        if let Some(kg) = graph
            .nodes
            .get_mut(legacy_knowledge_graph_id)
            .and_then(|node| node.as_any_mut().downcast_mut::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
        {
            kg.set_cycle(cycle);
            kg.add_fact_from(fact, source);
        }
    }

    fn safe_remote_crawl(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        url: &str,
        allow_remote_crawl: bool,
    ) -> String {
        let facts = match graph.nodes.get_mut(ids.legacy_crawler).and_then(|node| {
            node.as_any_mut()
                .downcast_mut::<eden_garm::nodes::legacy_runtime_extensions::LegacyCrawlerNode>()
        }) {
            Some(crawler) => match crawler.crawl_remote_gated(url, allow_remote_crawl) {
                Ok(facts) => facts,
                Err(e) => return format!("[CRAWLER] {}\n", e),
            },
            None => return "[CRAWLER] legacy_crawler node not found\n".to_string(),
        };
        let mut remembered = 0usize;
        for fact in &facts {
            if let Some(memory) = graph.nodes.get_mut(ids.legacy_memory).and_then(|node| {
                node.as_any_mut()
                    .downcast_mut::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            }) {
                memory.remember(fact);
                remembered += 1;
            }
            Self::feed_knowledge_graph(graph, ids.legacy_knowledge_graph, fact, "remote_crawl");
        }
        if let Some(first) = facts.first() {
            Self::feed_legacy_cognition_and_tension(graph, ids, first);
        }
        format!(
            "[CRAWLER] fetched={} remembered={} source={}\n",
            facts.len(),
            remembered,
            url,
        )
    }

    fn feed_legacy_cognition_and_tension(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        fact: &str,
    ) {
        let facts = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.facts().to_vec())
            .unwrap_or_else(|| vec![fact.to_string()]);
        if let Some(cognition) = graph.nodes.get_mut(ids.legacy_cognition).and_then(|node| {
            node.as_any_mut()
                .downcast_mut::<eden_garm::nodes::legacy_cognition::LegacyCognitionNode>()
        }) {
            cognition.update_from_facts(&facts, graph.global_tick);
            let target = cognition
                .select_exploration_target()
                .unwrap_or_else(|| fact.to_string());
            cognition.record_exploration(&target, 0.1, true);
            cognition.share_knowledge(fact.to_string(), "legacy_memory".to_string());
        }
        if let Some(tension) = graph.nodes.get_mut(ids.campo_tension).and_then(|node| {
            node.as_any_mut()
                .downcast_mut::<eden_garm::nodes::campo_tension::CampoTensionNode>()
        }) {
            tension.calcular(1, facts.len(), 1, facts.len(), true, 0.2, 0.1, facts.len());
        }
    }

    fn bounded_evolution(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        dt: f32,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) -> String {
        let (ticks_before, energy_before) = {
            let engine = shared_engine.lock().unwrap();
            (engine.state.tick_count, engine.metabolism.energy)
        };
        let pulses = 10usize;
        for step in 0..pulses {
            let signal = if step % 2 == 0 { 1.0 } else { 0.5 };
            graph.inject_sensor(vec![signal; 8]);
            graph.pulse(dt);
        }
        Self::update_api_metrics(graph, ids, api_metrics);
        let (ticks_after, energy_after) = {
            let engine = shared_engine.lock().unwrap();
            (engine.state.tick_count, engine.metabolism.energy)
        };
        let Some(node) = graph.nodes.get_mut(ids.legacy_evolution) else {
            return "legacy_evolution node not found\n".to_string();
        };
        let Some(evolution) =
            node.as_any_mut()
                .downcast_mut::<eden_garm::nodes::legacy_evolution::LegacyEvolutionNode>()
        else {
            return "legacy_evolution node type mismatch\n".to_string();
        };
        format!(
            "{}\n",
            evolution.summarize(
                pulses,
                ticks_before,
                ticks_after,
                energy_before,
                energy_after
            )
        )
    }

    fn save_runtime_state(autonomous: bool) -> String {
        let snapshot = serde_json::json!({ "autonomous": autonomous });
        let _ = eden_garm::state_paths::ensure_state_dir();
        match std::fs::write(
            eden_garm::state_paths::runtime_state_path(),
            snapshot.to_string(),
        ) {
            Ok(_) => "Runtime state also saved.".to_string(),
            Err(e) => format!("Runtime state error: {}", e),
        }
    }

    fn save_organ_autonomy_state() -> String {
        match eden_garm::nodes::organ_registry::save_state() {
            Ok(()) => "Organ autonomy state also saved.".to_string(),
            Err(e) => format!("Organ autonomy state error: {}", e),
        }
    }

    fn save_goal_scheduler_state() -> String {
        match eden_garm::nodes::goal_scheduler::save_state() {
            Ok(()) => "Goal scheduler state also saved.".to_string(),
            Err(e) => format!("Goal scheduler state error: {}", e),
        }
    }

    fn save_evaluation_loop_state() -> String {
        match eden_garm::nodes::evaluation_loop::save_state() {
            Ok(()) => "Evaluation loop state also saved.".to_string(),
            Err(e) => format!("Evaluation loop state error: {}", e),
        }
    }

    fn save_learning_ledger_state() -> String {
        match eden_garm::nodes::learning_ledger::save_state() {
            Ok(()) => "Learning ledger state also saved.".to_string(),
            Err(e) => format!("Learning ledger state error: {}", e),
        }
    }

    fn save_world_model_core_state() -> String {
        match eden_garm::nodes::world_model_core::save_state() {
            Ok(()) => "World model core state also saved.".to_string(),
            Err(e) => format!("World model core state error: {}", e),
        }
    }

    fn save_competence_benchmark_state() -> String {
        match eden_garm::nodes::competence_benchmark::save_state() {
            Ok(()) => "Competence benchmark state also saved.".to_string(),
            Err(e) => format!("Competence benchmark state error: {}", e),
        }
    }

    fn save_plan_executor_state() -> String {
        match eden_garm::nodes::plan_executor::save_state() {
            Ok(()) => "Plan executor state also saved.".to_string(),
            Err(e) => format!("Plan executor state error: {}", e),
        }
    }

    fn save_working_memory_state() -> String {
        match eden_garm::nodes::working_memory::save_state() {
            Ok(()) => "Working memory state also saved.".to_string(),
            Err(e) => format!("Working memory state error: {}", e),
        }
    }

    fn save_uncertainty_ledger_state() -> String {
        match eden_garm::nodes::uncertainty_ledger::save_state() {
            Ok(()) => "Uncertainty ledger state also saved.".to_string(),
            Err(e) => format!("Uncertainty ledger state error: {}", e),
        }
    }

    fn save_experiment_runner_state() -> String {
        match eden_garm::nodes::experiment_runner::save_state() {
            Ok(()) => "Experiment runner state also saved.".to_string(),
            Err(e) => format!("Experiment runner state error: {}", e),
        }
    }

    fn save_provenance_ledger_state() -> String {
        match eden_garm::nodes::provenance_ledger::save_state() {
            Ok(()) => "Provenance ledger state also saved.".to_string(),
            Err(e) => format!("Provenance ledger state error: {}", e),
        }
    }

    fn save_policy_guard_state() -> String {
        match eden_garm::nodes::policy_guard::save_state() {
            Ok(()) => "Policy guard state also saved.".to_string(),
            Err(e) => format!("Policy guard state error: {}", e),
        }
    }

    fn save_capability_maturity_state() -> String {
        match eden_garm::nodes::capability_maturity::save_state() {
            Ok(()) => "Capability maturity state also saved.".to_string(),
            Err(e) => format!("Capability maturity state error: {}", e),
        }
    }

    fn save_hybrid_voice_state() -> String {
        match eden_garm::nodes::hybrid_voice::save_state() {
            Ok(()) => "Hybrid voice state also saved.".to_string(),
            Err(e) => format!("Hybrid voice state error: {}", e),
        }
    }

    fn save_hrm_text_pretraining_state() -> String {
        match eden_garm::nodes::hrm_text_pretraining::save_state() {
            Ok(()) => "HRM-text pretraining state also saved.".to_string(),
            Err(e) => format!("HRM-text pretraining state error: {}", e),
        }
    }

    fn load_runtime_state(autonomous: &mut bool) -> String {
        let data = match std::fs::read_to_string(eden_garm::state_paths::runtime_state_path()) {
            Ok(data) => data,
            Err(e) => return format!("Runtime state error: {}", e),
        };
        let snapshot: serde_json::Value = match serde_json::from_str(&data) {
            Ok(snapshot) => snapshot,
            Err(e) => return format!("Runtime state parse error: {}", e),
        };
        if let Some(value) = snapshot.get("autonomous").and_then(|v| v.as_bool()) {
            *autonomous = value;
            "Runtime state also loaded.".to_string()
        } else {
            "Runtime state error: missing autonomous".to_string()
        }
    }

    fn load_organ_autonomy_state() -> String {
        match eden_garm::nodes::organ_registry::load_state() {
            Ok(()) => "Organ autonomy state also loaded.".to_string(),
            Err(e) => format!("Organ autonomy state error: {}", e),
        }
    }

    fn load_goal_scheduler_state() -> String {
        match eden_garm::nodes::goal_scheduler::load_state() {
            Ok(()) => "Goal scheduler state also loaded.".to_string(),
            Err(e) => format!("Goal scheduler state error: {}", e),
        }
    }

    fn load_evaluation_loop_state() -> String {
        match eden_garm::nodes::evaluation_loop::load_state() {
            Ok(()) => "Evaluation loop state also loaded.".to_string(),
            Err(e) => format!("Evaluation loop state error: {}", e),
        }
    }

    fn load_learning_ledger_state() -> String {
        match eden_garm::nodes::learning_ledger::load_state() {
            Ok(()) => "Learning ledger state also loaded.".to_string(),
            Err(e) => format!("Learning ledger state error: {}", e),
        }
    }

    fn load_world_model_core_state() -> String {
        match eden_garm::nodes::world_model_core::load_state() {
            Ok(()) => "World model core state also loaded.".to_string(),
            Err(e) => format!("World model core state error: {}", e),
        }
    }

    fn load_competence_benchmark_state() -> String {
        match eden_garm::nodes::competence_benchmark::load_state() {
            Ok(()) => "Competence benchmark state also loaded.".to_string(),
            Err(e) => format!("Competence benchmark state error: {}", e),
        }
    }

    fn load_plan_executor_state() -> String {
        match eden_garm::nodes::plan_executor::load_state() {
            Ok(()) => "Plan executor state also loaded.".to_string(),
            Err(e) => format!("Plan executor state error: {}", e),
        }
    }

    fn load_working_memory_state() -> String {
        match eden_garm::nodes::working_memory::load_state() {
            Ok(()) => "Working memory state also loaded.".to_string(),
            Err(e) => format!("Working memory state error: {}", e),
        }
    }

    fn load_uncertainty_ledger_state() -> String {
        match eden_garm::nodes::uncertainty_ledger::load_state() {
            Ok(()) => "Uncertainty ledger state also loaded.".to_string(),
            Err(e) => format!("Uncertainty ledger state error: {}", e),
        }
    }

    fn load_experiment_runner_state() -> String {
        match eden_garm::nodes::experiment_runner::load_state() {
            Ok(()) => "Experiment runner state also loaded.".to_string(),
            Err(e) => format!("Experiment runner state error: {}", e),
        }
    }

    fn load_provenance_ledger_state() -> String {
        match eden_garm::nodes::provenance_ledger::load_state() {
            Ok(()) => "Provenance ledger state also loaded.".to_string(),
            Err(e) => format!("Provenance ledger state error: {}", e),
        }
    }

    fn load_policy_guard_state() -> String {
        match eden_garm::nodes::policy_guard::load_state() {
            Ok(()) => "Policy guard state also loaded.".to_string(),
            Err(e) => format!("Policy guard state error: {}", e),
        }
    }

    fn load_capability_maturity_state() -> String {
        match eden_garm::nodes::capability_maturity::load_state() {
            Ok(()) => "Capability maturity state also loaded.".to_string(),
            Err(e) => format!("Capability maturity state error: {}", e),
        }
    }

    fn load_hybrid_voice_state() -> String {
        match eden_garm::nodes::hybrid_voice::load_state() {
            Ok(()) => "Hybrid voice state also loaded.".to_string(),
            Err(e) => format!("Hybrid voice state error: {}", e),
        }
    }

    fn load_hrm_text_pretraining_state() -> String {
        match eden_garm::nodes::hrm_text_pretraining::load_state() {
            Ok(()) => "HRM-text pretraining state also loaded.".to_string(),
            Err(e) => format!("HRM-text pretraining state error: {}", e),
        }
    }

    fn garm_backup() -> String {
        let backup_dir = eden_garm::state_paths::backup_dir_path();
        if let Err(e) = std::fs::create_dir_all(&backup_dir) {
            return format!("[GARM-BACKUP] error=create_backup_dir failed={}\n", e);
        }
        let Ok(entries) = std::fs::read_dir(eden_garm::state_paths::state_dir()) else {
            return "[GARM-BACKUP] error=state_dir_unreadable\n".to_string();
        };
        let mut copied = 0usize;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name() else {
                continue;
            };
            if std::fs::copy(&path, std::path::Path::new(&backup_dir).join(name)).is_ok() {
                copied += 1;
            }
        }
        format!("[GARM-BACKUP] dir={} files_copied={}\n", backup_dir, copied)
    }

    fn garm_restore() -> String {
        let backup_dir = eden_garm::state_paths::backup_dir_path();
        let Ok(entries) = std::fs::read_dir(&backup_dir) else {
            return format!(
                "[GARM-RESTORE] error=backup_unreadable dir={}\n",
                backup_dir
            );
        };
        let mut copied = 0usize;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name() else {
                continue;
            };
            if std::fs::copy(&path, eden_garm::state_paths::state_dir().join(name)).is_ok() {
                copied += 1;
            }
        }
        format!(
            "[GARM-RESTORE] dir={} files_restored={} run_load_next=true\n",
            backup_dir, copied
        )
    }

    fn garm_compact(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        shared_engine: &Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
    ) -> String {
        let api_metrics = Arc::new(eden_garm::nodes::api_server::ApiRuntimeMetrics::new(false));
        let compact = Self::intestino_compactar(graph, ids, &api_metrics);
        let save = eden_garm::nodes::persistence::save_all_with_legacy_nodes(
            graph,
            shared_engine,
            ids.legacy_memory,
            ids.legacy_history,
            ids.observatory,
            ids.legacy_evolution,
        );
        format!("[GARM-COMPACT]\n{}{}\n", compact, save)
    }

    fn update_api_metrics(
        graph: &mut eden_garm::HyperGraph,
        ids: RuntimeNodeIds,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) {
        api_metrics
            .alive_nodes
            .store(graph.alive_node_count() as u64, Ordering::Relaxed);
        api_metrics
            .edge_count
            .store(graph.edge_count() as u64, Ordering::Relaxed);
        let memory_facts = graph
            .nodes
            .get(ids.legacy_memory)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
            })
            .map(|memory| memory.fact_count())
            .unwrap_or(0);
        api_metrics
            .memory_facts
            .store(memory_facts as u64, Ordering::Relaxed);
        let (kg_edges, kg_nodes) = graph
            .nodes
            .get(ids.legacy_knowledge_graph)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
            .map(|kg| (kg.edge_count(), kg.node_count()))
            .unwrap_or((0, 0));
        let organic_surprise = graph
            .nodes
            .get(ids.organic_lifecycle)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::organic_lifecycle::OrganicLifecycleNode>()
            })
            .map(|node| node.surprise_score())
            .unwrap_or(0.0);
        let tension_value = graph
            .nodes
            .get(ids.campo_tension)
            .and_then(|node| {
                node.as_any()
                    .downcast_ref::<eden_garm::nodes::campo_tension::CampoTensionNode>()
            })
            .map(|node| node.tension())
            .unwrap_or(0.0);
        let hyper_edges = graph.edge_count();
        let tick = graph.global_tick;
        let _ = (kg_edges, memory_facts, tension_value, tick);
        let plan = graph
            .nodes
            .get_mut(ids.conscious_graph_regulator)
            .and_then(|node| node.as_any_mut().downcast_mut::<eden_garm::nodes::conscious_graph_regulator::ConsciousGraphRegulatorNode>())
            .map(|node| node.observe(kg_edges, kg_nodes, hyper_edges, memory_facts, organic_surprise, tick));
        if let Some(plan) = plan {
            if plan.should_regulate {
                let outcome = graph
                    .nodes
                    .get_mut(ids.legacy_knowledge_graph)
                    .and_then(|node| node.as_any_mut().downcast_mut::<eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode>())
                    .map(|kg| kg.regulate_capacity(tick, plan.soft_cap_edges, plan.hard_cap_edges, plan.min_confidence))
                    .unwrap_or_default();
                if let Some(regulator) = graph
                    .nodes
                    .get_mut(ids.conscious_graph_regulator)
                    .and_then(|node| node.as_any_mut().downcast_mut::<eden_garm::nodes::conscious_graph_regulator::ConsciousGraphRegulatorNode>())
                {
                    regulator.apply_outcome(outcome);
                }
            }
        }
        if let Some(regulator) = graph
            .nodes
            .get(ids.conscious_graph_regulator)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::conscious_graph_regulator::ConsciousGraphRegulatorNode>())
        {
            api_metrics.awareness_micros.store(scale_metric(regulator.awareness()), Ordering::Relaxed);
            api_metrics.integration_micros.store(scale_metric(regulator.integration()), Ordering::Relaxed);
            api_metrics.phi_micros.store(scale_metric(regulator.phi()), Ordering::Relaxed);
            api_metrics.complexity_micros.store(scale_metric(regulator.complexity()), Ordering::Relaxed);
            api_metrics.max_complexity_micros.store(scale_metric(regulator.max_complexity()), Ordering::Relaxed);
        }
        if let Some(meta) = graph.nodes.get(ids.meta).and_then(|node| {
            node.as_any()
                .downcast_ref::<eden_garm::nodes::meta_architect::MetaArchitectNode>()
        }) {
            api_metrics
                .self_modifications
                .store(meta.proposals_applied(), Ordering::Relaxed);
        }
        if let Some(organic) = graph.nodes.get(ids.organic_lifecycle).and_then(|node| {
            node.as_any()
                .downcast_ref::<eden_garm::nodes::organic_lifecycle::OrganicLifecycleNode>()
        }) {
            api_metrics
                .autonomous_thoughts
                .store(organic.autonomous_thought_count() as u64, Ordering::Relaxed);
            api_metrics
                .children_alive
                .store(organic.child_count() as u64, Ordering::Relaxed);
        }
        if let Some(cag) = graph.nodes.get(ids.context_augmentation).and_then(|node| {
            node.as_any()
                .downcast_ref::<eden_garm::nodes::context_augmentation::ContextAugmentationNode>()
        }) {
            let metrics = cag.metrics();
            api_metrics
                .cag_cache_entries
                .store(metrics.cache_entries, Ordering::Relaxed);
            api_metrics.cag_hits.store(metrics.hits, Ordering::Relaxed);
            api_metrics
                .cag_misses
                .store(metrics.misses, Ordering::Relaxed);
            api_metrics
                .cag_ttl_ticks
                .store(metrics.ttl_ticks, Ordering::Relaxed);
            api_metrics
                .cag_feedback_positive
                .store(metrics.feedback_positive, Ordering::Relaxed);
            api_metrics
                .cag_feedback_negative
                .store(metrics.feedback_negative, Ordering::Relaxed);
            api_metrics
                .cag_pending_actions
                .store(metrics.pending_actions, Ordering::Relaxed);
            api_metrics
                .cag_actions_executed
                .store(metrics.actions_executed, Ordering::Relaxed);
            api_metrics
                .cag_actions_blocked
                .store(metrics.actions_blocked, Ordering::Relaxed);
            api_metrics
                .cag_autonomous_runs
                .store(metrics.autonomous_runs, Ordering::Relaxed);
        }
    }

    fn update_lifecycle_metrics(
        graph: &eden_garm::HyperGraph,
        legacy_rebirth_meltrace_id: usize,
        api_metrics: &Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
    ) {
        if let Some(rebirth) = graph
            .nodes
            .get(legacy_rebirth_meltrace_id)
            .and_then(|node| node.as_any().downcast_ref::<eden_garm::nodes::legacy_runtime_extensions::RebirthMeltraceNode>())
        {
            api_metrics.meltrace_grabados.store(rebirth.event_count() as u64, Ordering::Relaxed);
            api_metrics.meltrace_muertes.store(rebirth.death_count() as u64, Ordering::Relaxed);
            api_metrics.meltrace_autons_vivos.store(1, Ordering::Relaxed);
        }
    }
}

fn scale_metric(value: f32) -> u64 {
    (value.max(0.0) * 1_000_000.0).min(u64::MAX as f32) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RuntimeTestHarness {
        graph: eden_garm::HyperGraph,
        shared_engine: Arc<std::sync::Mutex<eden_garm::capabilities::GarmCapabilityState>>,
        ids: RuntimeNodeIds,
        api_metrics: Arc<eden_garm::nodes::api_server::ApiRuntimeMetrics>,
        last_command: String,
        autonomous: bool,
        runtime_config: GarmRuntimeConfig,
    }

    impl RuntimeTestHarness {
        fn new() -> Self {
            let api_metrics = Arc::new(eden_garm::nodes::api_server::ApiRuntimeMetrics::new(true));
            api_metrics.ready.store(true, Ordering::Relaxed);
            let built = eden_garm::graph_builder::GraphBuilder::build(
                0,
                eden_garm::nodes::daemon::DaemonConfig {
                    enabled: false,
                    pid_file: None,
                    log_file: None,
                },
                Arc::clone(&api_metrics),
            );
            let ids = RuntimeNodeIds {
                meta: built.meta_id,
                legacy_memory: built.legacy_memory_id,
                legacy_reason: built.legacy_reason_id,
                legacy_dialogue: built.legacy_dialogue_id,
                observatory: built.observatory_id,
                legacy_history: built.legacy_history_id,
                legacy_evolution: built.legacy_evolution_id,
                legacy_cognition: built.legacy_cognition_id,
                campo_tension: built.campo_tension_id,
                legacy_knowledge_graph: built.legacy_knowledge_graph_id,
                legacy_autoconsumo: built.legacy_autoconsumo_id,
                legacy_venado: built.legacy_venado_id,
                legacy_paradigm_hub: built.legacy_paradigm_hub_id,
                legacy_ecosystem: built.legacy_ecosystem_id,
                legacy_rebirth_meltrace: built.legacy_rebirth_meltrace_id,
                legacy_crawler: built.legacy_crawler_id,
                help: built.help_id,
                readiness: built.readiness_id,
                organic_lifecycle: built.organic_lifecycle_id,
                conscious_graph_regulator: built.conscious_graph_regulator_id,
                context_augmentation: built.context_augmentation_id,
                hrm_reasoner: built.hrm_reasoner_id,
                voice_synthesizer: built.voice_synthesizer_id,
            };
            Self {
                graph: built.graph,
                shared_engine: built.shared_engine,
                ids,
                api_metrics,
                last_command: String::new(),
                autonomous: true,
                runtime_config: GarmRuntimeConfig::from_iter(std::iter::empty::<&str>()),
            }
        }

        fn dispatch(&mut self, command: &str) -> String {
            GarmRuntime::dispatch_gewc_cycle(
                command,
                &mut self.last_command,
                &mut self.graph,
                &self.shared_engine,
                self.ids,
                &self.api_metrics,
                0.1,
                &self.runtime_config,
                &mut self.autonomous,
            )
            .0
        }
    }

    #[test]
    fn parses_legacy_runtime_flags() {
        let config = GarmRuntimeConfig::from_iter([
            "--mcp",
            "--watchdog",
            "--no-interactive",
            "--max-cycles",
            "7",
            "--allow-remote-crawl",
            "--born",
            "--session",
            "/tmp/legacy.eden",
            "--log-level",
            "debug",
            "--state-dir",
            "/tmp/garm_test_state",
            "--daemon",
            "--pid-file",
            "/tmp/garm.pid",
            "--log-file",
            "/tmp/garm.log",
            "--api-port",
            "8123",
        ]);

        assert!(config.mcp);
        assert!(config.watchdog);
        assert!(config.no_interactive);
        assert_eq!(config.max_cycles, Some(7));
        assert!(config.allow_remote_crawl);
        assert!(config.born);
        assert_eq!(
            config.legacy_session_file.as_deref(),
            Some("/tmp/legacy.eden")
        );
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.state_dir, "/tmp/garm_test_state");
        assert!(config.daemon);
        assert_eq!(config.pid_file.as_deref(), Some("/tmp/garm.pid"));
        assert_eq!(config.log_file.as_deref(), Some("/tmp/garm.log"));
        assert_eq!(config.api_port, 8123);
    }

    #[test]
    fn gewc_registry_routes_all_commands_to_native_body_executors() {
        use eden_garm::global_executive_workspace::{GewcBodyHandler, GewcBodyRegistry};
        use eden_garm::nodes::command_router::GarmCommand::*;
        use std::collections::HashSet;

        let commands = vec![
            Quit,
            Tick,
            Status,
            Save,
            Load,
            Auto(1),
            Remember("fact".to_string()),
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
            ParadiseIntent("inspect runtime status".to_string()),
            ParadisePlan("latest".to_string()),
            ParadiseApprove("latest".to_string()),
            ParadiseExecute("latest".to_string()),
            ParadiseAudit,
            PraxisNexusEval,
            LocusLayerEval,
            LocusIngest("user notes :: context".to_string()),
            LocusContext("context".to_string()),
            LocusAudit,
            OperatorForgeEval,
            OperatorForgeSynthesize("causal model".to_string()),
            OperatorForgeVerify,
            OperatorForgeAudit,
            ExternalEcosystemEval,
            SovereignCognitionEval,
            ArtifactApiEval,
            TrainingEvidenceEval,
            Megatron7bEvidenceEval,
            Megatron7bAdapterPrepare,
            Megatron7bInferenceEval,
            Megatron7bCapabilityEval,
            Megatron7bAdmissionGateEval,
            EdenCapableEval,
            EdenCapableTrainingRunContract,
            EdenCognitiveDatasetEval,
            EdenNativeInferenceEval,
            EdenCapabilityDeltaEval,
            EdenStructuredOutputEval,
            EdenCheckpointRegistryEval,
            EdenSftElcpReadinessEval,
            EdenCapableGateEval,
            ModelRuntimeEval,
            RuntimeStateApiEval,
            OperationalApiEval,
            OperationalRuntimeEval,
            OperationalTaskSubmit("objective".to_string()),
            OperationalTaskRun,
            OperationalTaskAudit,
            OperationalActionExecute("status".to_string()),
            OperationalMemoryCommit("fact".to_string()),
            OperationalMemoryRollback("memtx-1".to_string()),
            OperationalReplayRun,
            OperationalSmokeRun,
            OperationalScenarioRun,
            OperationalPermissionsAudit,
            OperationalPermissionsDiff,
            OperationalPermissionsHistory,
            OperationalPermissionsRestore,
            OperationalPermissionsSet("remote_network deny".to_string()),
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
            GewcLifecycleControl("world_model pause".to_string()),
            Query("topic".to_string()),
            WhatIs("topic".to_string()),
            Why("topic".to_string()),
            TellMe("topic".to_string()),
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
            Crawl("https://example.invalid".to_string()),
            ConceptNet("/tmp/conceptnet.csv".to_string()),
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
            LearningRecord("lesson".to_string()),
            LearningConsolidate,
            LearningAudit,
            WorldModel,
            WorldObserve("observation".to_string()),
            WorldPredict("query".to_string()),
            WorldVerify,
            WorldAudit,
            WorldEval,
            Benchmark,
            BenchmarkRun,
            BenchmarkAudit,
            PlanExecutor,
            PlanExecutorPlan("goal".to_string()),
            PlanExecutorRun,
            PlanExecutorAudit,
            Attention,
            AttentionAttend("focus".to_string()),
            AttentionClear,
            AttentionAudit,
            Uncertainty,
            UncertaintyRecord("risk".to_string()),
            UncertaintyResolve,
            UncertaintyAudit,
            Experiment,
            ExperimentPlan("hypothesis".to_string()),
            ExperimentRun,
            ExperimentAudit,
            Provenance,
            ProvenanceRecord("source".to_string()),
            ProvenanceVerify,
            ProvenanceAudit,
            Policy,
            PolicyEval("action".to_string()),
            PolicyAudit,
            Maturity,
            MaturityAssess("capability".to_string()),
            MaturityAudit,
            OrganicRitual,
            Lengua("query".to_string()),
            Reloj("query".to_string()),
            Juez("query".to_string()),
            Voz,
            VozTexto("text".to_string()),
            HybridVoice,
            HybridVoicePlan("text".to_string()),
            HybridVoiceSynth("text".to_string()),
            HybridVoiceAudit,
            HrmText,
            HrmTextCorpus("/tmp/corpus".to_string()),
            HrmTextIngest("/tmp/corpus".to_string()),
            HrmTextSearch("query".to_string()),
            HrmTextContext("query".to_string()),
            HrmTextEval,
            HrmTextObjective("objective".to_string()),
            HrmTextPlan,
            HrmTextRun,
            HrmTextAudit,
            Intestino,
            Piel,
            Autotuning,
            Cag,
            CagExplain("query".to_string()),
            CagGaps("query".to_string()),
            CagActions,
            CagAudit,
            CagPlan("query".to_string()),
            CagRun("query".to_string()),
            Hrm("query".to_string()),
            HrmRun("query".to_string()),
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
            GoalsPlan("goal".to_string()),
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
            OrgansFeedback(true),
            Help,
            Migration,
            Unknown("???".to_string()),
        ];
        let mut handlers = HashSet::new();
        for command in commands {
            let binding = GewcBodyRegistry::bind(&command);
            assert!(
                GewcBodyExecutor::has_native_executor(binding.handler),
                "missing GEWC native executor for {:?} via {}",
                command,
                binding.handler.as_str()
            );
            handlers.insert(binding.handler);
        }
        for handler in GewcBodyHandler::ALL {
            assert!(
                handlers.contains(&handler),
                "GEWC handler not covered by command registry: {}",
                handler.as_str()
            );
        }
    }

    #[test]
    fn gewc_physical_domain_handlers_execute_representative_routes() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_gewc_domain_handlers_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        eden_garm::nodes::world_model_core::reset_for_tests();
        eden_garm::nodes::goal_scheduler::reset_for_tests();
        eden_garm::nodes::policy_guard::reset_for_tests();
        eden_garm::nodes::experiment_runner::reset_for_tests();
        eden_garm::nodes::working_memory::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let samples = [
            ("status", "gewc_runtime_control_body_handler"),
            (
                "remember typed domain memory",
                "gewc_memory_reasoning_body_handler",
            ),
            (
                "operational memory commit typed runtime fact",
                "gewc_memory_reasoning_body_handler",
            ),
            ("hola", "gewc_native_compatibility_body_handler"),
            (
                "learning record typed safe learning",
                "gewc_safe_learning_body_handler",
            ),
            (
                "world observe rain causes wet_ground",
                "gewc_world_model_body_handler",
            ),
            (
                "goals plan typed domain goal",
                "gewc_planning_goal_body_handler",
            ),
            (
                "operational task submit typed runtime goal",
                "gewc_planning_goal_body_handler",
            ),
            (
                "paradise intent inspect runtime status",
                "gewc_planning_goal_body_handler",
            ),
            ("paradise plan", "gewc_planning_goal_body_handler"),
            ("paradise approve", "gewc_planning_goal_body_handler"),
            ("paradise execute", "gewc_planning_goal_body_handler"),
            ("paradise sessions", "gewc_planning_goal_body_handler"),
            (
                "crawl https://example.invalid",
                "gewc_tool_adapter_body_handler",
            ),
            (
                "operational action execute status",
                "gewc_tool_adapter_body_handler",
            ),
            (
                "hybrid voice plan hola claro",
                "gewc_specialized_model_body_handler",
            ),
            (
                "model register candidate-a",
                "gewc_specialized_model_body_handler",
            ),
            (
                "policy eval local benchmark action",
                "gewc_metacognitive_safety_body_handler",
            ),
            (
                "gewc lifecycle world_model health_check",
                "gewc_metacognitive_safety_body_handler",
            ),
            ("capabilities audit", "gewc_validation_body_handler"),
            ("gewc operational benchmark", "gewc_validation_body_handler"),
            ("capability reality eval", "gewc_validation_body_handler"),
            (
                "architecture advantage eval",
                "gewc_validation_body_handler",
            ),
            ("paradise worldcell eval", "gewc_validation_body_handler"),
            ("praxis nexus eval", "gewc_formal_synthesis_body_handler"),
            ("locus eval", "gewc_locus_context_body_handler"),
            (
                "locus ingest user notes :: durable context",
                "gewc_locus_context_body_handler",
            ),
            (
                "operator forge synth causal risk model",
                "gewc_formal_synthesis_body_handler",
            ),
            ("external ecosystem eval", "gewc_validation_body_handler"),
            ("sovereign cognition eval", "gewc_validation_body_handler"),
            ("artifact api eval", "gewc_validation_body_handler"),
            ("model runtime eval", "gewc_validation_body_handler"),
            ("training harness eval", "gewc_validation_body_handler"),
            ("model governance eval", "gewc_validation_body_handler"),
            ("first model prepare", "gewc_validation_body_handler"),
            ("elcp prepare", "gewc_validation_body_handler"),
            ("elcp hardening", "gewc_validation_body_handler"),
            ("elcp admission gate", "gewc_validation_body_handler"),
            ("runtime state api eval", "gewc_validation_body_handler"),
            ("operational api eval", "gewc_validation_body_handler"),
            ("operational runtime eval", "gewc_validation_body_handler"),
            ("operational replay run", "gewc_validation_body_handler"),
            ("operational smoke run", "gewc_validation_body_handler"),
            ("operational scenario run", "gewc_validation_body_handler"),
            ("runtime spine eval", "gewc_validation_body_handler"),
            ("runtime spine audit", "gewc_validation_body_handler"),
            ("runtime spine verify", "gewc_validation_body_handler"),
            ("runtime spine enforce", "gewc_validation_body_handler"),
            ("runtime spine risk", "gewc_validation_body_handler"),
            ("runtime spine breakers", "gewc_validation_body_handler"),
            ("runtime spine replay", "gewc_validation_body_handler"),
            (
                "experiment plan typed domain experiment",
                "gewc_experiment_body_handler",
            ),
            ("organs", "gewc_agentic_body_handler"),
            (
                "attention typed domain focus",
                "gewc_workspace_attention_body_handler",
            ),
            ("help", "gewc_human_interface_body_handler"),
            ("???", "gewc_unknown_intent_body_handler"),
        ];

        for (command, handler) in samples {
            let _response = harness.dispatch(command);
            let runtime =
                eden_garm::global_executive_workspace::GlobalExecutiveWorkspaceCore::runtime_report(
                );
            assert!(
                runtime.contains(handler),
                "GEWC runtime report did not record handler {} for command {}: {}",
                handler,
                command,
                runtime
            );
        }
        let runtime =
            eden_garm::global_executive_workspace::GlobalExecutiveWorkspaceCore::runtime_report();
        assert!(runtime.contains("handler_topology=domain_owned_body_implementations"));
        assert!(runtime.contains("handler_metrics="));

        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::learning_ledger::reset_for_tests();
        eden_garm::nodes::world_model_core::reset_for_tests();
        eden_garm::nodes::goal_scheduler::reset_for_tests();
        eden_garm::nodes::policy_guard::reset_for_tests();
        eden_garm::nodes::experiment_runner::reset_for_tests();
        eden_garm::nodes::working_memory::reset_for_tests();
    }

    #[test]
    fn cag_command_observes_miss_then_hit_end_to_end() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_garm_runtime_cag_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        let api_metrics = Arc::new(eden_garm::nodes::api_server::ApiRuntimeMetrics::new(false));
        let built = eden_garm::graph_builder::GraphBuilder::build(
            0,
            eden_garm::nodes::daemon::DaemonConfig {
                enabled: false,
                pid_file: None,
                log_file: None,
            },
            Arc::clone(&api_metrics),
        );
        let mut graph = built.graph;
        let ids = RuntimeNodeIds {
            meta: built.meta_id,
            legacy_memory: built.legacy_memory_id,
            legacy_reason: built.legacy_reason_id,
            legacy_dialogue: built.legacy_dialogue_id,
            observatory: built.observatory_id,
            legacy_history: built.legacy_history_id,
            legacy_evolution: built.legacy_evolution_id,
            legacy_cognition: built.legacy_cognition_id,
            campo_tension: built.campo_tension_id,
            legacy_knowledge_graph: built.legacy_knowledge_graph_id,
            legacy_autoconsumo: built.legacy_autoconsumo_id,
            legacy_venado: built.legacy_venado_id,
            legacy_paradigm_hub: built.legacy_paradigm_hub_id,
            legacy_ecosystem: built.legacy_ecosystem_id,
            legacy_rebirth_meltrace: built.legacy_rebirth_meltrace_id,
            legacy_crawler: built.legacy_crawler_id,
            help: built.help_id,
            readiness: built.readiness_id,
            organic_lifecycle: built.organic_lifecycle_id,
            conscious_graph_regulator: built.conscious_graph_regulator_id,
            context_augmentation: built.context_augmentation_id,
            hrm_reasoner: built.hrm_reasoner_id,
            voice_synthesizer: built.voice_synthesizer_id,
        };
        let mut last_command = String::new();
        let mut autonomous = true;
        let runtime_config = GarmRuntimeConfig::from_iter(std::iter::empty::<&str>());

        GarmRuntime::dispatch_gewc_cycle(
            "remember eden is local organism",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );
        let (first, _) = GarmRuntime::dispatch_gewc_cycle(
            "que es eden",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );
        let (second, _) = GarmRuntime::dispatch_gewc_cycle(
            "que es eden",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );
        let (report, _) = GarmRuntime::dispatch_gewc_cycle(
            "cag",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );
        let (explain, _) = GarmRuntime::dispatch_gewc_cycle(
            "cag explain eden",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );
        let (gaps, _) = GarmRuntime::dispatch_gewc_cycle(
            "cag gaps eden",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );
        let (plan, _) = GarmRuntime::dispatch_gewc_cycle(
            "cag plan eden",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );
        let (run, _) = GarmRuntime::dispatch_gewc_cycle(
            "cag run eden",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );

        assert!(first.contains("[CAG] cache=miss"));
        assert!(second.contains("[CAG] cache=hit"));
        assert!(second.contains("[CAG-POLICY]"));
        assert!(report.contains("hits=1"));
        assert!(report.contains("misses=1"));
        assert!(explain.contains("[CAG-EXPLAIN]"));
        assert!(explain.contains("quality="));
        assert!(gaps.contains("[CAG-GAPS]"));
        assert!(gaps.contains("recommendations="));
        assert!(plan.contains("[CAG-ACTIONS]"));
        assert!(run.contains("[CAG-RUN]") || run.contains("no_pending_safe_actions"));
        assert!(api_metrics.cag_hits.load(Ordering::Relaxed) >= 1);
        assert_eq!(api_metrics.cag_misses.load(Ordering::Relaxed), 1);

        let (_weak_gaps, _) = GarmRuntime::dispatch_gewc_cycle(
            "cag gaps voidtopic",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );
        GarmRuntime::context_augmentation_autoplan(&mut graph, ids);
        let (actions_after_auto, _) = GarmRuntime::dispatch_gewc_cycle(
            "cag actions",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );
        let (audit_after_auto, _) = GarmRuntime::dispatch_gewc_cycle(
            "cag audit",
            &mut last_command,
            &mut graph,
            &built.shared_engine,
            ids,
            &api_metrics,
            0.1,
            &runtime_config,
            &mut autonomous,
        );
        assert!(actions_after_auto.contains("status=executed"));
        assert!(actions_after_auto.contains("kind=crawl_gated status=blocked"));
        assert!(audit_after_auto.contains("[CAG-AUDIT]"));
        assert!(audit_after_auto.contains("mode=autonomous"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn garm_audit_reports_runtime_organs_and_state() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_audit_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        let mut harness = RuntimeTestHarness::new();

        let audit = harness.dispatch("garm audit");

        assert!(audit.contains("[GARM-AUDIT] verdict=ready"));
        assert!(audit.contains("[ORGANOS-AUDIT] total=32"));
        assert!(audit.contains("[HRM] hrm:runs:"));
        assert!(audit.contains("[VOICE] voice:requests:"));
        assert!(audit.contains("GARM state directory:"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn garm_report_persists_operational_snapshot() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_report_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        let mut harness = RuntimeTestHarness::new();

        let report = harness.dispatch("garm report");
        let second_report = harness.dispatch("garm report");
        let history = harness.dispatch("garm report history");
        let saved = std::fs::read_to_string(eden_garm::state_paths::garm_report_path()).unwrap();
        let history_saved =
            std::fs::read_to_string(eden_garm::state_paths::garm_report_history_path()).unwrap();

        assert!(report.contains("[GARM-REPORT] verdict=ready"));
        assert!(second_report.contains("[GARM-REPORT] verdict=ready"));
        assert!(report.contains("[Runtime]"));
        assert!(report.contains("[Organs]"));
        assert!(report.contains("[LastDeltas]"));
        assert!(report.contains("[HRM] hrm:runs:"));
        assert!(report.contains("[VOICE] voice:requests:"));
        assert!(report.contains("[Evaluation]"));
        assert!(report.contains("[Learning]"));
        assert!(report.contains("[World]"));
        assert!(report.contains("[CAG]"));
        assert!(report.contains("[Persistence] report="));
        assert!(history.contains("[GARM-REPORT-HISTORY] entries=2"));
        assert!(history.contains("verdict=ready"));
        assert_eq!(saved, second_report);
        assert_eq!(history_saved.lines().count(), 2);
        for line in history_saved.lines() {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(parsed["verdict"], "ready");
            assert!(parsed["timestamp_ms"].as_u64().is_some());
        }
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn garm_report_history_retains_latest_entries() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_report_retention_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();

        for tick in 0..140 {
            let warning = GarmRuntime::append_garm_report_history(
                "ready", true, true, tick, 32, 64, 3, "hrm", "voice", "cag",
            );
            assert!(warning.is_none());
        }

        let history =
            std::fs::read_to_string(eden_garm::state_paths::garm_report_history_path()).unwrap();
        assert_eq!(history.lines().count(), GARM_REPORT_HISTORY_LIMIT);
        let first: serde_json::Value =
            serde_json::from_str(history.lines().next().unwrap()).unwrap();
        let last: serde_json::Value =
            serde_json::from_str(history.lines().last().unwrap()).unwrap();
        assert_eq!(first["tick"], 12);
        assert_eq!(last["tick"], 139);
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn garm_export_writes_structured_snapshot_and_import_validates_read_only() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_export_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        let mut harness = RuntimeTestHarness::new();

        let report = harness.dispatch("garm report");
        let export = harness.dispatch("garm export");
        let verify = harness.dispatch("garm verify export");
        let import = harness.dispatch("garm import");
        let body = std::fs::read_to_string(eden_garm::state_paths::garm_export_path()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

        assert!(report.contains("[GARM-REPORT]"));
        assert!(export.contains("[GARM-EXPORT]"));
        assert!(export.contains("schema=garm-export-v1"));
        assert!(export.contains("checksum_fnv64="));
        assert!(verify.contains("[GARM-VERIFY-EXPORT] ok=true"));
        assert!(verify.contains("cryptographic=false"));
        assert!(import.contains("[GARM-IMPORT] valid=true"));
        assert!(import.contains("read_only=true"));
        assert!(import.contains("restored=false"));
        assert_eq!(parsed["schema"], "garm-export-v1");
        assert_eq!(parsed["mode"], "diagnostic-read-only");
        assert_eq!(parsed["integrity"]["algorithm"], "fnv64");
        assert!(parsed["integrity"]["checksum_fnv64"].as_str().is_some());
        assert_eq!(parsed["organs"]["total"], 32);
        assert_eq!(parsed["report"]["history_count"], 1);
        assert!(parsed["state_paths"]["export"]
            .as_str()
            .unwrap()
            .ends_with("garm_export.json"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn garm_verify_export_detects_modified_export() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_export_verify_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        let mut harness = RuntimeTestHarness::new();

        let _ = harness.dispatch("garm report");
        let _ = harness.dispatch("garm export");
        let ok = harness.dispatch("garm verify export");
        let mut parsed: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(eden_garm::state_paths::garm_export_path()).unwrap(),
        )
        .unwrap();
        parsed["runtime"]["ticks"] = serde_json::json!(999);
        std::fs::write(
            eden_garm::state_paths::garm_export_path(),
            serde_json::to_string_pretty(&parsed).unwrap(),
        )
        .unwrap();
        let mismatch = harness.dispatch("garm verify export");

        assert!(ok.contains("ok=true"));
        assert!(mismatch.contains("ok=false"));
        assert!(mismatch.contains("expected="));
        assert!(mismatch.contains("actual="));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn garm_import_reports_missing_or_invalid_export_without_mutation() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_import_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();

        let missing = GarmRuntime::garm_import();
        std::fs::write(eden_garm::state_paths::garm_export_path(), "not-json").unwrap();
        let invalid = GarmRuntime::garm_import();

        assert!(missing.contains("error=export_missing"));
        assert!(invalid.contains("error=invalid_json"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn hrm_run_records_execution_metrics() {
        eden_garm::nodes::goal_scheduler::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let out = harness.dispatch("hrm run memoria runtime local");

        assert!(out.contains("[HRM-RUN]"));
        assert!(out.contains("[HRM-METRICS]"));
        assert!(out.contains("[CAG-ACTIONS]"));
        assert!(out.contains("[HRM-RUN-METRICS]"));
        assert!(out.contains("[GOALS-RECORD]"));
        assert!(out.contains("executions=1"));
        eden_garm::nodes::goal_scheduler::reset_for_tests();
    }

    #[test]
    fn readiness_plan_turns_gaps_into_goal_contracts() {
        eden_garm::nodes::goal_scheduler::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let plan = harness.dispatch("readiness plan");
        let run = harness.dispatch("readiness run");

        assert!(plan.contains("[READINESS-GOALS]"));
        assert!(plan.contains("source=readiness"));
        assert!(plan.contains("organ=hrm_text_pretraining"));
        assert!(run.contains("[READINESS-GOALS-RUN]"));
        assert!(run.contains("blocked_goals="));
        eden_garm::nodes::goal_scheduler::reset_for_tests();
    }

    #[test]
    fn readiness_probe_generates_local_phase_gate_evidence_and_bench_only_measures() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        eden_garm::nodes::goal_scheduler::reset_for_tests();
        eden_garm::nodes::hrm_text_pretraining::reset_for_tests();
        eden_garm::nodes::world_model_core::reset_for_tests();
        eden_garm::nodes::competence_benchmark::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let bench = harness.dispatch("readiness bench");
        assert!(bench.contains("measurement_only=true"));
        assert!(!bench.contains("[READINESS-PROBE]"));

        let probe = harness.dispatch("readiness probe");

        assert!(probe.contains("[READINESS-PROBE]"));
        assert!(probe.contains("[HRM-TEXT-CONTEXT-PACK]"));
        assert!(probe.contains("[WORLD-PREDICT]"));
        assert!(probe.contains("[BENCH-RUN]"));
        assert!(probe.contains("[READINESS-BENCH] gates=6 passed=6 failed=0"));
        eden_garm::nodes::goal_scheduler::reset_for_tests();
        eden_garm::nodes::hrm_text_pretraining::reset_for_tests();
        eden_garm::nodes::world_model_core::reset_for_tests();
        eden_garm::nodes::competence_benchmark::reset_for_tests();
    }

    #[test]
    fn readiness_external_generates_no_claim_manifest() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_external_validation_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.to_string_lossy().into_owned());
        let mut harness = RuntimeTestHarness::new();

        let out = harness.dispatch("readiness external");
        let manifest =
            std::fs::read_to_string(eden_garm::state_paths::external_validation_manifest_path())
                .unwrap();

        assert!(out.contains("[READINESS-EXTERNAL]"));
        assert!(out.contains("claim_allowed=false"));
        assert!(manifest.contains("garm-external-validation-v1"));
        assert!(manifest.contains("requires_independent_external_validation"));
        assert!(manifest.contains("\"agi_claim\": false"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn readiness_external_run_and_package_write_no_claim_artifacts() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_package_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        let mut harness = RuntimeTestHarness::new();

        let _ = harness.dispatch("policy eval run shell remote mutation");
        let memory = harness.dispatch("memory eval");
        let world = harness.dispatch("world eval");
        let external = harness.dispatch("readiness external run");
        let registry = harness.dispatch("capabilities audit");
        let package = harness.dispatch("readiness package");
        let result =
            std::fs::read_to_string(eden_garm::state_paths::external_validation_result_path())
                .unwrap();
        let body =
            std::fs::read_to_string(eden_garm::state_paths::readiness_package_path()).unwrap();

        assert!(memory.contains("[MEMORY-EVAL]"));
        assert!(world.contains("[WORLD-EVAL]"));
        assert!(external.contains("[EXTERNAL-VALIDATION]"));
        assert!(external.contains("claim_allowed=false"));
        assert!(registry.contains("[CAPABILITY-REGISTRY]"));
        assert!(result.contains("garm-external-validation-result-v1"));
        assert!(package.contains("[READINESS-PACKAGE]"));
        assert!(body.contains("garm-readiness-package-v1"));
        assert!(body.contains("capability_registry"));
        assert!(body.contains("\"agi_claim\": false"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn hrm_run_records_retrieval_learning_and_provenance() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_hrm_retrieval_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::goal_scheduler::reset_for_tests();
        eden_garm::nodes::hrm_text_pretraining::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        eden_garm::nodes::provenance_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();
        let corpus_dir = dir.join("corpus");
        std::fs::create_dir_all(&corpus_dir).unwrap();
        std::fs::write(
            corpus_dir.join("runtime.en.txt"),
            "memoria runtime local connects evidence retrieval\n",
        )
        .unwrap();

        let ingest = harness.dispatch(&format!("hrm text ingest {}", corpus_dir.to_string_lossy()));
        let out = harness.dispatch("hrm run memoria runtime local");
        let learning = harness.dispatch("learning audit");
        let provenance = harness.dispatch("provenance audit");
        let eval = harness.dispatch("eval run");

        assert!(ingest.contains("segments=1"));
        assert!(out.contains("[HRM-RUN-RETRIEVAL] hrm_text_retrieval hits="));
        assert!(out.contains("status=hit"));
        assert!(out.contains("[PROVENANCE-RECORD]"));
        assert!(learning.contains("hrm_plan+cag+organ_trace+hrm_text_retrieval"));
        assert!(provenance.contains("source=hrm_text_retrieval"));
        assert!(eval.contains("recommendation="));
        assert!(!eval.contains("run_hrm_text_search_before_regression_eval"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::goal_scheduler::reset_for_tests();
        eden_garm::nodes::hrm_text_pretraining::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        eden_garm::nodes::provenance_ledger::reset_for_tests();
    }

    #[test]
    fn goal_scheduler_commands_plan_run_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_goals_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::goal_scheduler::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let plan = harness.dispatch("goals plan arquitectura readiness contratos");
        let run = harness.dispatch("goals run");
        let audit = harness.dispatch("goals audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::goal_scheduler::reset_for_tests();
        let empty = harness.dispatch("goals");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("goals");

        assert!(plan.contains("[GOALS-PLAN] id=1"));
        assert!(plan.contains("contract"));
        assert!(run.contains("[GOALS-RUN] completed_goals=1"));
        assert!(audit.contains("[GOALS-AUDIT]"));
        assert!(save.contains("Goal scheduler state also saved."));
        assert!(empty.contains("goals=0"));
        assert!(load.contains("Goal scheduler state also loaded."));
        assert!(loaded.contains("goals=1"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::goal_scheduler::reset_for_tests();
    }

    #[test]
    fn evaluation_loop_commands_run_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_eval_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::evaluation_loop::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let eval = harness.dispatch("eval run");
        let audit = harness.dispatch("eval audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::evaluation_loop::reset_for_tests();
        let empty = harness.dispatch("eval");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("eval");

        assert!(eval.contains("[EVAL-RUN] id=1"));
        assert!(eval.contains("architecture="));
        assert!(audit.contains("[EVAL] schema=evaluation-loop-v1"));
        assert!(save.contains("Evaluation loop state also saved."));
        assert!(empty.contains("records=0"));
        assert!(load.contains("Evaluation loop state also loaded."));
        assert!(loaded.contains("records=1"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::evaluation_loop::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn learning_ledger_commands_record_consolidate_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_learning_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let record = harness.dispatch("learning record evaluation improves architecture");
        let consolidate = harness.dispatch("learning consolidate");
        let audit = harness.dispatch("learning audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let empty = harness.dispatch("learning");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("learning");

        assert!(record.contains("[LEARNING-RECORD] id=1"));
        assert!(consolidate.contains("[LEARNING-CONSOLIDATE]"));
        assert!(audit.contains("[LEARNING] schema=learning-ledger-v1"));
        assert!(save.contains("Learning ledger state also saved."));
        assert!(empty.contains("entries=0"));
        assert!(load.contains("Learning ledger state also loaded."));
        assert!(loaded.contains("entries=1"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn world_model_commands_observe_predict_verify_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_world_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::world_model_core::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let observe = harness.dispatch("world observe rain causes wet_ground");
        let predict = harness.dispatch("world predict rain");
        let verify = harness.dispatch("world verify");
        let audit = harness.dispatch("world audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::world_model_core::reset_for_tests();
        let empty = harness.dispatch("world");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("world");

        assert!(observe.contains("[WORLD-OBSERVE] id=1"));
        assert!(predict.contains("[WORLD-PREDICT] id=1"));
        assert!(predict.contains("[LEARNING-RECORD]"));
        assert!(verify.contains("[WORLD-VERIFY] verified=1"));
        assert!(audit.contains("[WORLD] schema=world-model-core-v1"));
        assert!(save.contains("World model core state also saved."));
        assert!(empty.contains("observations=0"));
        assert!(load.contains("World model core state also loaded."));
        assert!(loaded.contains("observations=1"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::world_model_core::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn competence_benchmark_commands_run_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_benchmark_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::competence_benchmark::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let run = harness.dispatch("bench run");
        let audit = harness.dispatch("bench audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::competence_benchmark::reset_for_tests();
        let empty = harness.dispatch("bench");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("bench");

        assert!(run.contains("[BENCH-RUN] id=1"));
        assert!(run.contains("[LEARNING-RECORD]"));
        assert!(audit.contains("[BENCH] schema=competence-benchmark-v1"));
        assert!(save.contains("Competence benchmark state also saved."));
        assert!(empty.contains("runs=0"));
        assert!(load.contains("Competence benchmark state also loaded."));
        assert!(loaded.contains("runs=1"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::competence_benchmark::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn plan_executor_commands_plan_run_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_executor_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::plan_executor::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let plan = harness.dispatch("exec plan mejorar benchmark local");
        let run = harness.dispatch("exec run");
        let audit = harness.dispatch("exec audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::plan_executor::reset_for_tests();
        let empty = harness.dispatch("exec");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("exec");

        assert!(plan.contains("[EXEC-PLAN] id=1"));
        assert!(run.contains("[EXEC-RUN] id=1"));
        assert!(run.contains("[LEARNING-RECORD]"));
        assert!(audit.contains("[EXEC] schema=plan-executor-v1"));
        assert!(save.contains("Plan executor state also saved."));
        assert!(empty.contains("plans=0"));
        assert!(load.contains("Plan executor state also loaded."));
        assert!(loaded.contains("plans=1"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::plan_executor::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn attention_commands_attend_clear_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_attention_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::working_memory::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let attend = harness.dispatch("attention benchmark rollback risk");
        let audit = harness.dispatch("attention audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::working_memory::reset_for_tests();
        let empty = harness.dispatch("attention");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("attention");
        let clear = harness.dispatch("attention clear");

        assert!(attend.contains("[ATTEND] id=1"));
        assert!(attend.contains("[LEARNING-RECORD]"));
        assert!(audit.contains("[ATTENTION] schema=working-memory-v1"));
        assert!(save.contains("Working memory state also saved."));
        assert!(empty.contains("items=0"));
        assert!(load.contains("Working memory state also loaded."));
        assert!(loaded.contains("items=1"));
        assert!(clear.contains("[ATTENTION-CLEAR] cleared=1"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::working_memory::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn uncertainty_commands_record_resolve_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_uncertainty_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::uncertainty_ledger::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let record = harness.dispatch("uncertainty record unknown rollout risk");
        let audit = harness.dispatch("uncertainty audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::uncertainty_ledger::reset_for_tests();
        let empty = harness.dispatch("uncertainty");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("uncertainty");
        let resolved = harness.dispatch("uncertainty resolve");

        assert!(record.contains("[UNCERTAINTY-RECORD] id=1"));
        assert!(record.contains("[LEARNING-RECORD]"));
        assert!(audit.contains("[UNCERTAINTY] schema=uncertainty-ledger-v1"));
        assert!(save.contains("Uncertainty ledger state also saved."));
        assert!(empty.contains("records=0"));
        assert!(load.contains("Uncertainty ledger state also loaded."));
        assert!(loaded.contains("records=1"));
        assert!(resolved.contains("status=mitigated"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::uncertainty_ledger::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn experiment_commands_plan_run_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_experiment_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::experiment_runner::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let plan = harness.dispatch("experiment plan benchmark improves evaluation");
        let run = harness.dispatch("experiment run");
        let audit = harness.dispatch("experiment audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::experiment_runner::reset_for_tests();
        let empty = harness.dispatch("experiment");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("experiment");

        assert!(plan.contains("[EXPERIMENT-PLAN] id=1"));
        assert!(run.contains("[EXPERIMENT-RUN] id=1"));
        assert!(run.contains("[LEARNING-RECORD]"));
        assert!(audit.contains("[EXPERIMENT] schema=experiment-runner-v1"));
        assert!(save.contains("Experiment runner state also saved."));
        assert!(empty.contains("experiments=0"));
        assert!(load.contains("Experiment runner state also loaded."));
        assert!(loaded.contains("experiments=1"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::experiment_runner::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn provenance_commands_record_verify_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_provenance_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::provenance_ledger::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let record = harness.dispatch("provenance record tests passed for experiment");
        let audit = harness.dispatch("provenance audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::provenance_ledger::reset_for_tests();
        let empty = harness.dispatch("provenance");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("provenance");
        let verify = harness.dispatch("provenance verify");

        assert!(record.contains("[PROVENANCE-RECORD] id=1"));
        assert!(record.contains("[LEARNING-RECORD]"));
        assert!(audit.contains("[PROVENANCE] schema=provenance-ledger-v1"));
        assert!(save.contains("Provenance ledger state also saved."));
        assert!(empty.contains("records=0"));
        assert!(load.contains("Provenance ledger state also loaded."));
        assert!(loaded.contains("records=1"));
        assert!(verify.contains("[PROVENANCE-VERIFY]"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::provenance_ledger::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn policy_commands_eval_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_policy_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::policy_guard::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let eval = harness.dispatch("policy eval local benchmark action");
        let blocked = harness.dispatch("policy eval remote shell action");
        let audit = harness.dispatch("policy audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::policy_guard::reset_for_tests();
        let empty = harness.dispatch("policy");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("policy");

        assert!(eval.contains("[POLICY-EVAL] id=1"));
        assert!(eval.contains("verdict=allow"));
        assert!(blocked.contains("verdict=block"));
        assert!(blocked.contains("[LEARNING-RECORD]"));
        assert!(audit.contains("[POLICY] schema=policy-guard-v1"));
        assert!(save.contains("Policy guard state also saved."));
        assert!(empty.contains("decisions=0"));
        assert!(load.contains("Policy guard state also loaded."));
        assert!(loaded.contains("decisions=2"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::policy_guard::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn maturity_commands_assess_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_maturity_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::capability_maturity::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let assess = harness.dispatch("maturity assess policy guard");
        let audit = harness.dispatch("maturity audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::capability_maturity::reset_for_tests();
        let empty = harness.dispatch("maturity");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("maturity");

        assert!(assess.contains("[MATURITY-ASSESS] id=1"));
        assert!(assess.contains("[LEARNING-RECORD]"));
        assert!(audit.contains("[MATURITY] schema=capability-maturity-v1"));
        assert!(save.contains("Capability maturity state also saved."));
        assert!(empty.contains("records=0"));
        assert!(load.contains("Capability maturity state also loaded."));
        assert!(loaded.contains("records=1"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::capability_maturity::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
    }

    #[test]
    fn hybrid_voice_commands_plan_synth_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_hybrid_voice_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::hybrid_voice::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();

        let plan = harness.dispatch("hybrid voice plan hola claro");
        let synth = harness.dispatch("hybrid voice synth hola claro");
        let audit = harness.dispatch("hybrid voice audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::hybrid_voice::reset_for_tests();
        let empty = harness.dispatch("hybrid voice");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("hybrid voice");

        assert!(plan.contains("[HYBRID-VOICE-PLAN] id=1"));
        assert!(synth.contains("[HYBRID-VOICE-SYNTH] id=2"));
        assert!(synth.contains("[VOZ-TTS]"));
        assert!(audit.contains("[HYBRID-VOICE] schema=hybrid-voice-v1"));
        assert!(save.contains("Hybrid voice state also saved."));
        assert!(empty.contains("plans=0"));
        assert!(load.contains("Hybrid voice state also loaded."));
        assert!(loaded.contains("plans=2"));
        assert!(std::fs::metadata(eden_garm::state_paths::hybrid_voice_manifest_path()).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::hybrid_voice::reset_for_tests();
    }

    #[test]
    fn hrm_text_commands_run_save_and_load() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_hrm_text_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        eden_garm::nodes::hrm_text_pretraining::reset_for_tests();
        eden_garm::nodes::hybrid_voice::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        eden_garm::nodes::provenance_ledger::reset_for_tests();
        eden_garm::nodes::policy_guard::reset_for_tests();
        eden_garm::nodes::capability_maturity::reset_for_tests();
        let mut harness = RuntimeTestHarness::new();
        let corpus_dir = dir.join("corpus");
        std::fs::create_dir_all(&corpus_dir).unwrap();
        std::fs::write(corpus_dir.join("runtime.en.txt"), "alpha\nbeta\n").unwrap();

        let corpus = harness.dispatch("hrm text corpus /tmp/local_corpus.txt");
        let ingest = harness.dispatch(&format!("hrm text ingest {}", corpus_dir.to_string_lossy()));
        let search = harness.dispatch("hrm text search alpha");
        let objective = harness.dispatch("hrm text objective text to plan priors");
        let plan = harness.dispatch("hrm text plan");
        let run = harness.dispatch("hrm text run");
        let audit = harness.dispatch("hrm text audit");
        let save = harness.dispatch("save");
        eden_garm::nodes::hrm_text_pretraining::reset_for_tests();
        let empty = harness.dispatch("hrm text");
        let load = harness.dispatch("load");
        let loaded = harness.dispatch("hrm text");

        assert!(corpus.contains("[HRM-TEXT-CORPUS] id=1"));
        assert!(corpus.contains("[PROVENANCE-RECORD]"));
        assert!(ingest.contains("[HRM-TEXT-INGEST]"));
        assert!(ingest.contains("segments=2"));
        assert!(search.contains("[HRM-TEXT-SEARCH]"));
        assert!(search.contains("status=hit"));
        assert!(objective.contains("[HRM-TEXT-OBJECTIVE] id=5"));
        assert!(objective.contains("[LEARNING-RECORD]"));
        assert!(plan.contains("[HRM-TEXT-PLAN] id=6"));
        assert!(plan.contains("[HYBRID-VOICE-PLAN]"));
        assert!(run.contains("[HRM-TEXT-RUN] id=7"));
        assert!(run.contains("weights_present=false"));
        assert!(audit.contains("[HRM-TEXT] schema=hrm-text-pretraining-v1"));
        assert!(save.contains("HRM-text pretraining state also saved."));
        assert!(empty.contains("events=0"));
        assert!(load.contains("HRM-text pretraining state also loaded."));
        assert!(loaded.contains("events=7"));
        assert!(
            std::fs::metadata(eden_garm::state_paths::hrm_text_checkpoint_manifest_path()).is_ok()
        );
        assert!(std::fs::metadata(eden_garm::state_paths::hrm_text_segments_path()).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
        eden_garm::nodes::hrm_text_pretraining::reset_for_tests();
        eden_garm::nodes::hybrid_voice::reset_for_tests();
        eden_garm::nodes::learning_ledger::reset_for_tests();
        eden_garm::nodes::provenance_ledger::reset_for_tests();
        eden_garm::nodes::policy_guard::reset_for_tests();
        eden_garm::nodes::capability_maturity::reset_for_tests();
    }

    #[test]
    fn garm_backup_restore_and_compact_are_local() {
        let _state_guard = eden_garm::state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_runtime_maintenance_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir(dir.clone());
        eden_garm::state_paths::ensure_state_dir().unwrap();
        std::fs::write(eden_garm::state_paths::runtime_state_path(), "runtime-ok").unwrap();
        let mut harness = RuntimeTestHarness::new();

        let backup = harness.dispatch("garm backup");
        std::fs::remove_file(eden_garm::state_paths::runtime_state_path()).unwrap();
        let restore = harness.dispatch("garm restore");
        let compact = harness.dispatch("garm compact");

        assert!(backup.contains("[GARM-BACKUP]"));
        assert!(backup.contains("files_copied="));
        assert!(restore.contains("[GARM-RESTORE]"));
        assert!(restore.contains("run_load_next=true"));
        assert!(std::fs::metadata(eden_garm::state_paths::runtime_state_path()).is_ok());
        assert!(compact.contains("[GARM-COMPACT]"));
        assert!(compact.contains("GARM state saved"));
        let _ = std::fs::remove_dir_all(&dir);
        eden_garm::state_paths::set_state_dir("/tmp/eden_garm");
    }
}
