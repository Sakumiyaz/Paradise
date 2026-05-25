use super::{GarmRuntime, GewcBodyExecutor, GewcBodyPorts};
use crate::eden_garm;
use crate::eden_garm::global_executive_workspace::{
    CoreExecutionOutcome, CoreRuntimeContext, GewcBodyHandler, GewcBodyRegistry,
    GlobalExecutiveWorkspaceCore,
};
use crate::eden_garm::nodes::command_router::GarmCommand;

fn domain_mismatch(command: &GarmCommand, handler: GewcBodyHandler) -> Option<(String, bool)> {
    let binding = GewcBodyRegistry::bind(command);
    if binding.handler == handler {
        None
    } else {
        Some(GewcBodyExecutor::execute_unexpected_handler_command(
            command.clone(),
            handler,
        ))
    }
}

pub(super) mod memory_reasoning {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::MemoryReasoning) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;
        let api_metrics = ports.api_metrics;

        match command {
            GarmCommand::Remember(fact) => {
                let mut out = String::new();
                if let Some(node) = graph.nodes.get_mut(ids.legacy_memory) {
                    if let Some(memory) =
                        node.as_any_mut()
                            .downcast_mut::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
                    {
                        out.push_str(&format!("{}\n", memory.remember(&fact)));
                    }
                }
                GarmRuntime::feed_knowledge_graph(
                    graph,
                    ids.legacy_knowledge_graph,
                    &fact,
                    "legacy_memory",
                );
                out.push_str(&GarmRuntime::legacy_kg_hypotheses(
                    graph,
                    ids.legacy_knowledge_graph,
                    &fact,
                ));
                GarmRuntime::feed_legacy_cognition_and_tension(graph, ids, &fact);
                GarmRuntime::update_api_metrics(graph, ids, api_metrics);
                (out, true)
            }
            GarmCommand::Memory => {
                let mut out = String::new();
                if let Some(node) = graph.nodes.get(ids.legacy_memory) {
                    if let Some(memory) =
                        node.as_any()
                            .downcast_ref::<eden_garm::nodes::legacy_memory::LegacyMemoryNode>()
                    {
                        out.push_str(&format!("{}\n", memory.recall()));
                    }
                }
                (out, true)
            }
            GarmCommand::MemoryEval => (GarmRuntime::memory_eval(graph, ids), true),
            GarmCommand::OperationalMemoryCommit(text) => (
                eden_garm::operational_runtime::commit_memory_transaction(&text),
                true,
            ),
            GarmCommand::OperationalMemoryRollback(transaction_id) => (
                eden_garm::operational_runtime::rollback_memory_transaction(&transaction_id),
                true,
            ),
            GarmCommand::Query(topic) => (
                GarmRuntime::reason_report(graph, ids, "query", &topic),
                true,
            ),
            GarmCommand::WhatIs(topic) => {
                let kg = GarmRuntime::legacy_kg_explain(graph, ids, &topic);
                if kg.is_empty() {
                    (
                        GarmRuntime::reason_report(graph, ids, "what_is", &topic),
                        true,
                    )
                } else {
                    (kg, true)
                }
            }
            GarmCommand::Why(topic) => {
                (GarmRuntime::reason_report(graph, ids, "why", &topic), true)
            }
            GarmCommand::TellMe(topic) => (
                GarmRuntime::reason_report(graph, ids, "tell_me", &topic),
                true,
            ),
            GarmCommand::Cag => {
                GarmRuntime::update_api_metrics(graph, ids, api_metrics);
                (GarmRuntime::context_augmentation_report(graph, ids), true)
            }
            GarmCommand::CagExplain(query) => {
                let pack = GarmRuntime::context_pack(graph, ids, &query, 3, 5);
                GarmRuntime::apply_cag_feedback(graph, ids, &pack, pack.context_quality >= 0.35);
                (
                    GarmRuntime::context_augmentation_explain(graph, ids, &query),
                    true,
                )
            }
            GarmCommand::CagGaps(query) => {
                let pack = GarmRuntime::context_pack(graph, ids, &query, 3, 5);
                GarmRuntime::apply_cag_feedback(graph, ids, &pack, pack.context_quality >= 0.35);
                (
                    GarmRuntime::context_augmentation_gaps(graph, ids, &query),
                    true,
                )
            }
            GarmCommand::CagActions => {
                (GarmRuntime::context_augmentation_actions(graph, ids), true)
            }
            GarmCommand::CagAudit => (GarmRuntime::context_augmentation_audit(graph, ids), true),
            GarmCommand::CagPlan(query) => {
                let _pack = GarmRuntime::context_pack(graph, ids, &query, 3, 5);
                (
                    GarmRuntime::context_augmentation_plan(graph, ids, &query),
                    true,
                )
            }
            GarmCommand::CagRun(query) => {
                let _pack = GarmRuntime::context_pack(graph, ids, &query, 3, 5);
                (
                    GarmRuntime::context_augmentation_run(graph, ids, &query),
                    true,
                )
            }
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::MemoryReasoning,
            ),
        }
    }
}

pub(super) mod native_compatibility {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::NativeCompatibility) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let shared_engine = ports.shared_engine;
        let ids = ports.ids;
        let api_metrics = ports.api_metrics;

        match command {
            GarmCommand::Greeting => (
                GarmRuntime::legacy_dialogue(
                    graph,
                    ids.legacy_dialogue,
                    shared_engine,
                    |dialogue, _| dialogue.greeting(),
                ),
                true,
            ),
            GarmCommand::SelfQuery => (
                GarmRuntime::legacy_dialogue(
                    graph,
                    ids.legacy_dialogue,
                    shared_engine,
                    |dialogue, _| dialogue.identity(),
                ),
                true,
            ),
            GarmCommand::Thinking => (
                GarmRuntime::legacy_dialogue(
                    graph,
                    ids.legacy_dialogue,
                    shared_engine,
                    |dialogue, engine| dialogue.thinking(engine),
                ),
                true,
            ),
            GarmCommand::Feeling => (
                GarmRuntime::legacy_dialogue(
                    graph,
                    ids.legacy_dialogue,
                    shared_engine,
                    |dialogue, engine| dialogue.feeling(engine),
                ),
                true,
            ),
            GarmCommand::Phi => {
                let mut out = GarmRuntime::legacy_dialogue(
                    graph,
                    ids.legacy_dialogue,
                    shared_engine,
                    |dialogue, engine| dialogue.phi(engine),
                );
                out.push_str(&GarmRuntime::conscious_graph_report(graph, ids));
                (out, true)
            }
            GarmCommand::Observatory => (
                GarmRuntime::observatory_report(graph, ids, shared_engine, api_metrics),
                true,
            ),
            GarmCommand::History => (GarmRuntime::history_report(graph, ids.legacy_history), true),
            GarmCommand::OrganicRitual => (GarmRuntime::organic_ritual(graph, ids), true),
            GarmCommand::Lengua(query) => (GarmRuntime::lengua_responder(graph, ids, &query), true),
            GarmCommand::Reloj(query) => (GarmRuntime::reloj_temporal(graph, ids, &query), true),
            GarmCommand::Juez(query) => (GarmRuntime::juez_validar(graph, ids, &query), true),
            GarmCommand::Intestino => (
                GarmRuntime::intestino_compactar(graph, ids, api_metrics),
                true,
            ),
            GarmCommand::Piel => (GarmRuntime::piel_report(graph, ids, api_metrics), true),
            GarmCommand::Autotuning => (
                GarmRuntime::autotuning_report(graph, ids, api_metrics),
                true,
            ),
            GarmCommand::Migration => (eden_garm::legacy_migration::migration_report(), true),
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::NativeCompatibility,
            ),
        }
    }
}

pub(super) mod safe_learning {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::SafeLearning) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let shared_engine = ports.shared_engine;
        let ids = ports.ids;
        let api_metrics = ports.api_metrics;
        let dt = ports.dt;

        match command {
            GarmCommand::Evolve => (
                GarmRuntime::bounded_evolution(graph, ids, shared_engine, dt, api_metrics),
                true,
            ),
            GarmCommand::SelfImprovementEval => (
                GarmRuntime::self_improvement_architecture_eval(graph, ids, shared_engine),
                true,
            ),
            GarmCommand::Learning => (eden_garm::nodes::learning_ledger::report(), true),
            GarmCommand::LearningRecord(text) => {
                let out = eden_garm::nodes::learning_ledger::record(
                    "manual",
                    &text,
                    "manual_operator_note",
                    "observed",
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[LEARNING] {}", text),
                );
                GarmRuntime::feed_knowledge_graph(
                    graph,
                    ids.legacy_knowledge_graph,
                    &format!("{} is learning_hypothesis", text),
                    "learning_ledger",
                );
                (out, true)
            }
            GarmCommand::LearningConsolidate => {
                (eden_garm::nodes::learning_ledger::consolidate(), true)
            }
            GarmCommand::LearningAudit => (eden_garm::nodes::learning_ledger::audit_report(), true),
            GarmCommand::Maturity => (eden_garm::nodes::capability_maturity::report(), true),
            GarmCommand::MaturityAssess(capability) => {
                (GarmRuntime::maturity_assess(graph, ids, &capability), true)
            }
            GarmCommand::MaturityAudit => {
                (eden_garm::nodes::capability_maturity::audit_report(), true)
            }
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::SafeLearning,
            ),
        }
    }
}

pub(super) mod world_model {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::WorldModel) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;

        match command {
            GarmCommand::WorldModel => (eden_garm::nodes::world_model_core::report(), true),
            GarmCommand::WorldObserve(text) => {
                let out = eden_garm::nodes::world_model_core::observe("manual", &text);
                GarmRuntime::feed_knowledge_graph(
                    graph,
                    ids.legacy_knowledge_graph,
                    &format!("{} is world_observation", text),
                    "world_model_core",
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[WORLD-OBSERVE] {}", text),
                );
                (out, true)
            }
            GarmCommand::WorldPredict(query) => {
                let out = eden_garm::nodes::world_model_core::predict(&query);
                let status = if out.contains("status=supported") {
                    "completed"
                } else {
                    "needs_evidence"
                };
                let learning = eden_garm::nodes::learning_ledger::record(
                    "world_model_core",
                    &query,
                    "world_prediction",
                    status,
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[WORLD-PREDICT] {}", query),
                );
                (format!("{}{}", out, learning), true)
            }
            GarmCommand::WorldVerify => (
                eden_garm::nodes::world_model_core::verify_predictions(),
                true,
            ),
            GarmCommand::WorldAudit => (eden_garm::nodes::world_model_core::audit_report(), true),
            GarmCommand::WorldEval => (GarmRuntime::world_eval(), true),
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::WorldModel,
            ),
        }
    }
}

pub(super) mod planning_goal {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::PlanningGoal) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let shared_engine = ports.shared_engine;
        let ids = ports.ids;
        let api_metrics = ports.api_metrics;

        match command {
            GarmCommand::PlanExecutor => (eden_garm::nodes::plan_executor::report(), true),
            GarmCommand::PlanExecutorPlan(query) => {
                let out = eden_garm::nodes::plan_executor::plan(&query);
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[EXEC-PLAN] {}", query),
                );
                (out, true)
            }
            GarmCommand::PlanExecutorRun => {
                let out = GarmRuntime::plan_executor_run(graph, ids);
                let execution = if out.contains("completed") {
                    "completed"
                } else {
                    "rolled_back"
                };
                let action_evidence = eden_garm::action_evidence::record_attempt(
                    "plan_executor",
                    "run ready local plan",
                    "allowed",
                    execution,
                    "local_plan_state_updated",
                    "plan_executor_report",
                    if execution == "rolled_back" {
                        "medium"
                    } else {
                        "low"
                    },
                );
                (format!("{}{}", out, action_evidence), true)
            }
            GarmCommand::PlanExecutorAudit => {
                (eden_garm::nodes::plan_executor::audit_report(), true)
            }
            GarmCommand::OperationalTaskSubmit(objective) => (
                eden_garm::operational_runtime::submit_task(&objective),
                true,
            ),
            GarmCommand::OperationalTaskRun => {
                (eden_garm::operational_runtime::run_next_task(), true)
            }
            GarmCommand::OperationalTaskAudit => {
                (eden_garm::operational_runtime::task_audit(), true)
            }
            GarmCommand::ParadiseIntent(intent) => {
                (eden_garm::paradise_worldcell::record_intent(&intent), true)
            }
            GarmCommand::ParadisePlan(spec) => {
                (eden_garm::paradise_worldcell::plan_session(&spec), true)
            }
            GarmCommand::ParadiseApprove(spec) => {
                (eden_garm::paradise_worldcell::approve_session(&spec), true)
            }
            GarmCommand::ParadiseExecute(spec) => execute_paradise_session(spec, ports),
            GarmCommand::ParadiseAudit => (eden_garm::paradise_worldcell::audit_sessions(), true),
            GarmCommand::Goals => (eden_garm::nodes::goal_scheduler::report(), true),
            GarmCommand::GoalsPlan(query) => {
                let out = eden_garm::nodes::goal_scheduler::plan_goal(&query, "manual");
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[GOALS-PLAN] {}", query),
                );
                GarmRuntime::feed_knowledge_graph(
                    graph,
                    ids.legacy_knowledge_graph,
                    &format!("{} is scheduled_goal", query),
                    "goal_scheduler",
                );
                (out, true)
            }
            GarmCommand::GoalsRun => {
                let out = eden_garm::nodes::goal_scheduler::run_ready_goals();
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    "[GOALS-RUN] ready goals executed",
                );
                (out, true)
            }
            GarmCommand::GoalsAudit => (eden_garm::nodes::goal_scheduler::audit_report(), true),
            GarmCommand::ReadinessPlan => (
                GarmRuntime::readiness_goal_plan(graph, ids, shared_engine, api_metrics, false),
                true,
            ),
            GarmCommand::ReadinessRun => (
                GarmRuntime::readiness_goal_plan(graph, ids, shared_engine, api_metrics, true),
                true,
            ),
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::PlanningGoal,
            ),
        }
    }

    fn execute_paradise_session(spec: String, ports: &mut GewcBodyPorts<'_>) -> (String, bool) {
        let request = match eden_garm::paradise_worldcell::prepare_execution(&spec) {
            eden_garm::paradise_worldcell::ParadiseExecutionPreparation::Ready(request) => request,
            eden_garm::paradise_worldcell::ParadiseExecutionPreparation::Blocked(output) => {
                return (output, true);
            }
        };
        let mut nested_last = String::new();
        let nested_command = eden_garm::nodes::command_router::CommandRouterNode::parse_raw(
            &request.raw_command,
            &mut nested_last,
        );
        let capability_status = ports
            .shared_engine
            .lock()
            .map(|engine| engine.status_summary())
            .unwrap_or_else(|_| "garm | capability_status_unavailable".to_string());
        let decision = GlobalExecutiveWorkspaceCore::decide(
            &nested_command,
            CoreRuntimeContext {
                raw_command: request.raw_command.clone(),
                autonomous: *ports.autonomous,
                allow_remote_crawl: ports.runtime_config.allow_remote_crawl,
                graph_nodes: ports.graph.alive_node_count(),
                graph_edges: ports.graph.edge_count(),
                global_tick: ports.graph.global_tick,
                capability_status,
            },
        );
        let decision_trace = GlobalExecutiveWorkspaceCore::record_decision(&decision);
        GarmRuntime::record_history(
            &mut *ports.graph,
            ports.ids.legacy_history,
            decision_trace.trim(),
        );
        if decision.is_blocked() {
            let blocked = decision.blocked_response();
            let completion = GlobalExecutiveWorkspaceCore::record_execution_completion(
                &decision,
                CoreExecutionOutcome::blocked(&blocked),
            );
            GarmRuntime::record_history(
                &mut *ports.graph,
                ports.ids.legacy_history,
                completion.trim(),
            );
            return (
                eden_garm::paradise_worldcell::complete_execution(
                    &request.session_id,
                    &request.raw_command,
                    &blocked,
                    "blocked",
                    "blocked_by_nested_gewc_decision",
                    true,
                ),
                true,
            );
        }

        let (nested_output, should_continue) =
            GewcBodyExecutor::execute(nested_command, &decision, ports);
        let completion = GlobalExecutiveWorkspaceCore::record_execution_completion(
            &decision,
            CoreExecutionOutcome::completed(&nested_output, should_continue),
        );
        GarmRuntime::record_history(
            &mut *ports.graph,
            ports.ids.legacy_history,
            completion.trim(),
        );
        let mut out = eden_garm::paradise_worldcell::complete_execution(
            &request.session_id,
            &request.raw_command,
            &nested_output,
            "completed",
            "nested_gewc_runtime_body_completed",
            should_continue,
        );
        out.push_str(&nested_output);
        (out, should_continue)
    }
}

pub(super) mod tool_adapter {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::ToolAdapter) {
            return mismatch;
        }
        let ids = ports.ids;
        let api_metrics = ports.api_metrics;
        let runtime_config = ports.runtime_config;

        match command {
            GarmCommand::Crawl(url) => (
                GarmRuntime::safe_remote_crawl(
                    &mut *ports.graph,
                    ids,
                    &url,
                    runtime_config.allow_remote_crawl,
                ),
                true,
            ),
            GarmCommand::ConceptNet(path) => (
                GarmRuntime::load_conceptnet(&mut *ports.graph, ids, &path, api_metrics),
                true,
            ),
            GarmCommand::OperationalActionExecute(raw) => execute_operational_action(raw, ports),
            GarmCommand::HrmTextCorpus(path) => {
                let out = eden_garm::nodes::hrm_text_pretraining::add_corpus(&path);
                GarmRuntime::record_history(
                    &mut *ports.graph,
                    ids.legacy_history,
                    &format!("[HRM-TEXT-CORPUS] {}", path),
                );
                let provenance = eden_garm::nodes::provenance_ledger::record(
                    "hrm_text_pretraining",
                    &format!("corpus={}", path),
                );
                (format!("{}{}", out, provenance), true)
            }
            GarmCommand::HrmTextIngest(path) => {
                let out = eden_garm::nodes::hrm_text_pretraining::ingest_directory(&path);
                GarmRuntime::record_history(
                    &mut *ports.graph,
                    ids.legacy_history,
                    &format!("[HRM-TEXT-INGEST] {}", path),
                );
                let provenance = eden_garm::nodes::provenance_ledger::record(
                    "hrm_text_pretraining",
                    &format!("ingest_dir={}", path),
                );
                let learning = eden_garm::nodes::learning_ledger::record(
                    "hrm_text_pretraining",
                    "local corpus directory segmented for HRM text priors",
                    "corpus_ingest",
                    if out.contains("status=indexed") {
                        "indexed"
                    } else {
                        "needs_corpus"
                    },
                );
                (format!("{}{}{}", out, provenance, learning), true)
            }
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::ToolAdapter,
            ),
        }
    }

    fn execute_operational_action(raw: String, ports: &mut GewcBodyPorts<'_>) -> (String, bool) {
        let mut nested_last = String::new();
        let nested_command =
            eden_garm::nodes::command_router::CommandRouterNode::parse_raw(&raw, &mut nested_last);
        let binding = GewcBodyRegistry::bind(&nested_command);
        let dry_run = serde_json::from_str::<serde_json::Value>(
            &eden_garm::operational_api::action_dry_run_json(&raw),
        )
        .unwrap_or_else(|_| serde_json::json!({}));
        let requires_supervision = dry_run
            .get("requires_supervision")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let mutates_runtime = dry_run
            .get("mutates_runtime")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        if requires_supervision || mutates_runtime {
            return (
                eden_garm::operational_runtime::record_action_result(
                    &raw,
                    &format!("{:?}", nested_command),
                    binding.route,
                    binding.handler.as_str(),
                    "blocked",
                    "blocked",
                    "not_executed_supervision_or_mutation_required",
                    "action_contract_blocked_before_runtime_dispatch",
                    requires_supervision,
                    mutates_runtime,
                ),
                true,
            );
        }

        let capability_status = ports
            .shared_engine
            .lock()
            .map(|engine| engine.status_summary())
            .unwrap_or_else(|_| "garm | capability_status_unavailable".to_string());
        let decision = GlobalExecutiveWorkspaceCore::decide(
            &nested_command,
            CoreRuntimeContext {
                raw_command: raw.clone(),
                autonomous: *ports.autonomous,
                allow_remote_crawl: ports.runtime_config.allow_remote_crawl,
                graph_nodes: ports.graph.alive_node_count(),
                graph_edges: ports.graph.edge_count(),
                global_tick: ports.graph.global_tick,
                capability_status,
            },
        );
        let decision_trace = GlobalExecutiveWorkspaceCore::record_decision(&decision);
        GarmRuntime::record_history(
            &mut *ports.graph,
            ports.ids.legacy_history,
            decision_trace.trim(),
        );
        if decision.is_blocked() {
            let blocked = decision.blocked_response();
            let completion = GlobalExecutiveWorkspaceCore::record_execution_completion(
                &decision,
                CoreExecutionOutcome::blocked(&blocked),
            );
            GarmRuntime::record_history(
                &mut *ports.graph,
                ports.ids.legacy_history,
                completion.trim(),
            );
            return (
                eden_garm::operational_runtime::record_action_result(
                    &raw,
                    &format!("{:?}", nested_command),
                    binding.route,
                    binding.handler.as_str(),
                    "blocked",
                    "blocked",
                    "blocked_by_nested_gewc_decision",
                    &blocked,
                    requires_supervision,
                    mutates_runtime,
                ),
                true,
            );
        }

        let (nested_output, should_continue) =
            GewcBodyExecutor::execute(nested_command.clone(), &decision, ports);
        let completion = GlobalExecutiveWorkspaceCore::record_execution_completion(
            &decision,
            CoreExecutionOutcome::completed(&nested_output, should_continue),
        );
        GarmRuntime::record_history(
            &mut *ports.graph,
            ports.ids.legacy_history,
            completion.trim(),
        );
        let mut out = eden_garm::operational_runtime::record_action_result(
            &raw,
            &format!("{:?}", nested_command),
            binding.route,
            binding.handler.as_str(),
            "allowed",
            "completed",
            "safe_read_or_eval_command_dispatched",
            &nested_output,
            requires_supervision,
            mutates_runtime,
        );
        out.push_str(&nested_output);
        (out, should_continue)
    }
}

pub(super) mod specialized_model {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::SpecializedModel) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;
        let api_metrics = ports.api_metrics;

        match command {
            GarmCommand::Voz => (GarmRuntime::voz_autodocumentar(graph, ids), true),
            GarmCommand::VozTexto(text) => (GarmRuntime::voz_sintetizar(graph, ids, &text), true),
            GarmCommand::HybridVoice => (eden_garm::nodes::hybrid_voice::report(), true),
            GarmCommand::HybridVoicePlan(text) => {
                let out = eden_garm::nodes::hybrid_voice::plan(&text);
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[HYBRID-VOICE-PLAN] text_len={}", text.len()),
                );
                (out, true)
            }
            GarmCommand::HybridVoiceSynth(text) => {
                (GarmRuntime::hybrid_voice_synth(graph, ids, &text), true)
            }
            GarmCommand::HybridVoiceAudit => (eden_garm::nodes::hybrid_voice::audit_report(), true),
            GarmCommand::HrmText => (eden_garm::nodes::hrm_text_pretraining::report(), true),
            GarmCommand::HrmTextSearch(query) => {
                let out = eden_garm::nodes::hrm_text_pretraining::search(&query);
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[HRM-TEXT-SEARCH] {}", query),
                );
                (out, true)
            }
            GarmCommand::HrmTextContext(query) => {
                let out = eden_garm::nodes::hrm_text_pretraining::context_pack(&query);
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[HRM-TEXT-CONTEXT-PACK] {}", query),
                );
                let provenance = eden_garm::nodes::provenance_ledger::record(
                    "hrm_text_context_pack",
                    &format!("query={} restricted_generation=true", query),
                );
                (format!("{}{}", out, provenance), true)
            }
            GarmCommand::HrmTextEval => {
                let out = eden_garm::nodes::hrm_text_pretraining::evaluate_retrieval();
                let learning = eden_garm::nodes::learning_ledger::record(
                    "hrm_text_eval",
                    "continuous rag retrieval evaluation",
                    "answerable+unanswerable+citations",
                    if out.contains("passed=3 total=3") {
                        "passed"
                    } else {
                        "needs_attention"
                    },
                );
                (format!("{}{}", out, learning), true)
            }
            GarmCommand::HrmTextObjective(objective) => {
                let out = eden_garm::nodes::hrm_text_pretraining::add_objective(&objective);
                let learning = eden_garm::nodes::learning_ledger::record(
                    "hrm_text_pretraining",
                    &objective,
                    "pretraining_objective",
                    "registered",
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[HRM-TEXT-OBJECTIVE] {}", objective),
                );
                (format!("{}{}", out, learning), true)
            }
            GarmCommand::HrmTextPlan => {
                let out = eden_garm::nodes::hrm_text_pretraining::plan();
                let hybrid = eden_garm::nodes::hybrid_voice::plan(
                    "hrm_text_prior_manifest connects text intent prosody plan evidence",
                );
                let policy = eden_garm::nodes::policy_guard::evaluate(
                    "hrm_text_pretraining local manifest no network no shell no code mutation",
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    "[HRM-TEXT-PLAN] local curriculum planned",
                );
                (format!("{}{}{}", out, hybrid, policy), true)
            }
            GarmCommand::HrmTextRun => (GarmRuntime::hrm_text_run(graph, ids), true),
            GarmCommand::HrmTextAudit => {
                (eden_garm::nodes::hrm_text_pretraining::audit_report(), true)
            }
            GarmCommand::Hrm(query) => (GarmRuntime::hrm_reason(graph, ids, &query), true),
            GarmCommand::HrmRun(query) => (
                GarmRuntime::hrm_run_plan(graph, ids, &query, api_metrics),
                true,
            ),
            GarmCommand::ModelRegister(model_id) => {
                (eden_garm::model_runtime::register_model(&model_id), true)
            }
            GarmCommand::ModelLoad(model_id) => {
                (eden_garm::model_runtime::load_model(&model_id), true)
            }
            GarmCommand::ModelEvaluate(model_id) => {
                (eden_garm::model_runtime::evaluate_model(&model_id), true)
            }
            GarmCommand::ModelUnload(model_id) => {
                (eden_garm::model_runtime::unload_model(&model_id), true)
            }
            GarmCommand::ModelAudit => (eden_garm::model_runtime::audit_model_runtime(), true),
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::SpecializedModel,
            ),
        }
    }
}

pub(super) mod metacognitive_safety {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::MetacognitiveSafety) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;

        match command {
            GarmCommand::ActionEvidence => (eden_garm::action_evidence::report(), true),
            GarmCommand::Uncertainty => (eden_garm::nodes::uncertainty_ledger::report(), true),
            GarmCommand::UncertaintyRecord(text) => {
                let out = eden_garm::nodes::uncertainty_ledger::record("manual", &text);
                let learning = eden_garm::nodes::learning_ledger::record(
                    "uncertainty_ledger",
                    &text,
                    "risk_calibration",
                    "open",
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[UNCERTAINTY] {}", text),
                );
                (format!("{}{}", out, learning), true)
            }
            GarmCommand::UncertaintyResolve => {
                (eden_garm::nodes::uncertainty_ledger::resolve_next(), true)
            }
            GarmCommand::UncertaintyAudit => {
                (eden_garm::nodes::uncertainty_ledger::audit_report(), true)
            }
            GarmCommand::Provenance => (eden_garm::nodes::provenance_ledger::report(), true),
            GarmCommand::ProvenanceRecord(text) => {
                let out = eden_garm::nodes::provenance_ledger::record("manual", &text);
                let learning = eden_garm::nodes::learning_ledger::record(
                    "provenance_ledger",
                    &text,
                    "evidence_provenance",
                    "recorded",
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[PROVENANCE] {}", text),
                );
                (format!("{}{}", out, learning), true)
            }
            GarmCommand::ProvenanceVerify => {
                (eden_garm::nodes::provenance_ledger::verify_next(), true)
            }
            GarmCommand::ProvenanceAudit => {
                (eden_garm::nodes::provenance_ledger::audit_report(), true)
            }
            GarmCommand::Policy => (eden_garm::nodes::policy_guard::report(), true),
            GarmCommand::PolicyEval(text) => {
                let out = eden_garm::nodes::policy_guard::evaluate(&text);
                let policy = if out.contains("verdict=allow") {
                    "allowed"
                } else {
                    "blocked"
                };
                let learning = eden_garm::nodes::learning_ledger::record(
                    "policy_guard",
                    &text,
                    "constraint_check",
                    policy,
                );
                let action_evidence = eden_garm::action_evidence::record_attempt(
                    "policy_guard",
                    &text,
                    policy,
                    "not_executed",
                    "policy_check_only",
                    "policy_guard_report",
                    if policy == "blocked" { "high" } else { "low" },
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[POLICY] {}", text),
                );
                (format!("{}{}{}", out, learning, action_evidence), true)
            }
            GarmCommand::PolicyAudit => (eden_garm::nodes::policy_guard::audit_report(), true),
            GarmCommand::OperationalPermissionsAudit => {
                (eden_garm::operational_runtime::permission_audit(), true)
            }
            GarmCommand::OperationalPermissionsDiff => {
                (eden_garm::operational_runtime::permission_diff(), true)
            }
            GarmCommand::OperationalPermissionsHistory => {
                (eden_garm::operational_runtime::permission_history(), true)
            }
            GarmCommand::OperationalPermissionsRestore => {
                (eden_garm::operational_runtime::permission_restore(), true)
            }
            GarmCommand::OperationalPermissionsSet(spec) => {
                (eden_garm::operational_runtime::permission_set(&spec), true)
            }
            GarmCommand::OperationalRecoveryAudit => {
                (eden_garm::operational_runtime::recovery_audit(), true)
            }
            GarmCommand::OperationalRecoveryRun => {
                (eden_garm::operational_runtime::run_recovery(), true)
            }
            GarmCommand::GewcLifecycleControl(spec) => (
                eden_garm::operational_runtime::control_lifecycle(&spec),
                true,
            ),
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::MetacognitiveSafety,
            ),
        }
    }
}

pub(super) mod experiment {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::Experiment) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;

        match command {
            GarmCommand::Experiment => (eden_garm::nodes::experiment_runner::report(), true),
            GarmCommand::ExperimentPlan(query) => {
                let out = eden_garm::nodes::experiment_runner::plan(&query);
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[EXPERIMENT-PLAN] {}", query),
                );
                (out, true)
            }
            GarmCommand::ExperimentRun => (GarmRuntime::experiment_run(graph, ids), true),
            GarmCommand::ExperimentAudit => {
                (eden_garm::nodes::experiment_runner::audit_report(), true)
            }
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::Experiment,
            ),
        }
    }
}

pub(super) mod agentic {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::Agentic) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;
        let api_metrics = ports.api_metrics;

        match command {
            GarmCommand::Rebirth => (GarmRuntime::legacy_rebirth(graph, ids, api_metrics), true),
            GarmCommand::Organs => (eden_garm::nodes::organ_registry::organ_report(graph), true),
            GarmCommand::OrgansAudit => (
                format!("{}\n", eden_garm::nodes::organ_registry::organ_audit(graph)),
                true,
            ),
            GarmCommand::OrgansPlan => (eden_garm::nodes::organ_registry::organ_plan(graph), true),
            GarmCommand::OrgansRun => (GarmRuntime::organs_safe_run(graph, ids, api_metrics), true),
            GarmCommand::OrgansHealth => (
                format!(
                    "{}\n",
                    eden_garm::nodes::organ_registry::organ_health_report(graph)
                ),
                true,
            ),
            GarmCommand::OrgansRepair => (
                GarmRuntime::organs_safe_repair(graph, ids, api_metrics),
                true,
            ),
            GarmCommand::OrgansActions => (
                format!(
                    "{}\n",
                    eden_garm::nodes::organ_registry::organ_actions_report()
                ),
                true,
            ),
            GarmCommand::OrgansFeedback(useful) => {
                eden_garm::nodes::organ_registry::record_feedback(useful);
                (
                    format!(
                        "[ORGANOS-FEEDBACK] useful={}\n{}\n",
                        useful,
                        eden_garm::nodes::organ_registry::organ_autonomy_audit_report()
                    ),
                    true,
                )
            }
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::Agentic,
            ),
        }
    }
}

pub(super) mod workspace_attention {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::WorkspaceAttention) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;

        match command {
            GarmCommand::Attention => (eden_garm::nodes::working_memory::report(), true),
            GarmCommand::AttentionAttend(text) => {
                let out = eden_garm::nodes::working_memory::attend("manual", &text);
                let learning = eden_garm::nodes::learning_ledger::record(
                    "working_memory",
                    &text,
                    "attention_focus",
                    "focused",
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[ATTEND] {}", text),
                );
                (format!("{}{}", out, learning), true)
            }
            GarmCommand::AttentionClear => (eden_garm::nodes::working_memory::clear(), true),
            GarmCommand::AttentionAudit => (eden_garm::nodes::working_memory::audit_report(), true),
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::WorkspaceAttention,
            ),
        }
    }
}

pub(super) mod locus_context {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::LocusContext) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;

        match command {
            GarmCommand::LocusLayerEval => {
                let out = eden_garm::eden_locus_layer::run(
                    eden_garm::eden_locus_layer::LocusLayerInput {
                        gewc_report: GlobalExecutiveWorkspaceCore::runtime_report(),
                        memory_report: GarmRuntime::memory_eval(graph, ids),
                        policy_report: eden_garm::nodes::policy_guard::report(),
                        provenance_report: eden_garm::nodes::provenance_ledger::report(),
                        uncertainty_report: eden_garm::nodes::uncertainty_ledger::report(),
                        action_evidence_report: eden_garm::action_evidence::report(),
                        world_report: format!(
                            "{}{}",
                            eden_garm::nodes::world_model_core::report(),
                            GarmRuntime::world_eval()
                        ),
                    },
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    "[EDEN-LOCUS-LAYER] personal context and authority substrate generated",
                );
                (out, true)
            }
            GarmCommand::LocusIngest(spec) => {
                let out = eden_garm::eden_locus_layer::ingest(&spec);
                let provenance = eden_garm::nodes::provenance_ledger::record(
                    "eden_locus_layer",
                    &format!("locus_ingest={}", spec),
                );
                let learning = eden_garm::nodes::learning_ledger::record(
                    "eden_locus_layer",
                    &spec,
                    "context_authority_observation",
                    if out.contains("status=quarantined") {
                        "quarantined"
                    } else {
                        "accepted_context"
                    },
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[LOCUS-INGEST] {}", spec),
                );
                (format!("{}{}{}", out, provenance, learning), true)
            }
            GarmCommand::LocusContext(query) => {
                let out = eden_garm::eden_locus_layer::context_packet(&query);
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[LOCUS-CONTEXT] {}", query),
                );
                (out, true)
            }
            GarmCommand::LocusAudit => (eden_garm::eden_locus_layer::audit_report(), true),
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::LocusContext,
            ),
        }
    }
}

pub(super) mod formal_synthesis {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::FormalSynthesis) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;
        let shared_engine = ports.shared_engine;
        let api_metrics = ports.api_metrics;

        match command {
            GarmCommand::PraxisNexusEval => (
                GarmRuntime::praxis_nexus_eval(graph, ids, shared_engine, api_metrics),
                true,
            ),
            GarmCommand::OperatorForgeEval => {
                let out = eden_garm::eden_operator_forge::run(
                    eden_garm::eden_operator_forge::OperatorForgeInput {
                        praxis_report: GarmRuntime::praxis_nexus_eval(
                            graph,
                            ids,
                            shared_engine,
                            api_metrics,
                        ),
                        world_report: format!(
                            "{}{}",
                            eden_garm::nodes::world_model_core::report(),
                            GarmRuntime::world_eval()
                        ),
                        policy_report: eden_garm::nodes::policy_guard::report(),
                        provenance_report: eden_garm::nodes::provenance_ledger::report(),
                        uncertainty_report: eden_garm::nodes::uncertainty_ledger::report(),
                        action_evidence_report: eden_garm::action_evidence::report(),
                    },
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    "[EDEN-OPERATOR-FORGE] formal primitive synthesis engine generated",
                );
                (out, true)
            }
            GarmCommand::OperatorForgeSynthesize(goal) => {
                let out = eden_garm::eden_operator_forge::synthesize(&goal);
                let provenance = eden_garm::nodes::provenance_ledger::record(
                    "eden_operator_forge",
                    &format!("synth_goal={}", goal),
                );
                let uncertainty = eden_garm::nodes::uncertainty_ledger::record(
                    "eden_operator_forge",
                    &format!(
                        "formal_candidate_requires_external_truth_validation: {}",
                        goal
                    ),
                );
                GarmRuntime::record_history(
                    graph,
                    ids.legacy_history,
                    &format!("[OPERATOR-FORGE-SYNTH] {}", goal),
                );
                (format!("{}{}{}", out, provenance, uncertainty), true)
            }
            GarmCommand::OperatorForgeVerify => (eden_garm::eden_operator_forge::verify(), true),
            GarmCommand::OperatorForgeAudit => {
                (eden_garm::eden_operator_forge::audit_report(), true)
            }
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::FormalSynthesis,
            ),
        }
    }
}

pub(super) mod human_interface {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::HumanInterface) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;

        match command {
            GarmCommand::Help => {
                let mut out = String::new();
                if let Some(node) = graph.nodes.get_mut(ids.help) {
                    if let Some(help) = node
                        .as_any_mut()
                        .downcast_mut::<eden_garm::nodes::help::HelpNode>()
                    {
                        out.push_str(&format!("{}\n", help.help()));
                    }
                }
                (out, true)
            }
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::HumanInterface,
            ),
        }
    }
}

pub(super) mod unknown_intent {
    use super::*;

    pub(in crate::eden_garm::runtime) fn execute(
        command: GarmCommand,
        ports: &mut GewcBodyPorts<'_>,
    ) -> (String, bool) {
        if let Some(mismatch) = domain_mismatch(&command, GewcBodyHandler::UnknownIntent) {
            return mismatch;
        }
        let graph = &mut *ports.graph;
        let ids = ports.ids;

        match command {
            GarmCommand::Unknown(raw) => {
                let fallback = GarmRuntime::legacy_unknown_fallback(graph, ids, &raw);
                if fallback.is_empty() {
                    (
                        format!(
                            "Comando no reconocido: '{}'\nUsa: tick | estado | auto N | save | load | quit\n",
                            raw
                        ),
                        true,
                    )
                } else {
                    (fallback, true)
                }
            }
            other => GewcBodyExecutor::execute_unexpected_handler_command(
                other,
                GewcBodyHandler::UnknownIntent,
            ),
        }
    }
}
