use crate::eden_garm::nodes::command_router::GarmCommand;
use crate::eden_garm::state_paths;
use std::io::Write;

const ABSORPTION_MODEL: &str = "gewc_owns_all_runtime_domains";
const LEGACY_MODE: &str = "absorbed_as_gewc_native_compatibility_body";
const GARM_MODE: &str = "absorbed_as_gewc_native_cognitive_body";
const NATIVE_BODY: &str = "gewc_native_cognitive_body";
const ADAPTER_BODY: &str = "gewc_native_compatibility_body";
const RUNTIME_BODY: &str = "gewc_runtime_body";
const BODY_EXECUTOR: &str = "gewc_body_executor";
const HANDLER_DISPATCH: &str = "domain_handler_dispatch";
const HANDLER_TOPOLOGY: &str = "domain_owned_body_implementations";
const SHARED_BODY_ENGINE: bool = false;
const MODEL_REGISTRY_AUTHORITY: &str = "gewc_model_plural_registry";
const MODEL_CONTROL_MODE: &str = "gewc_centric_model_plural_not_llm_centric";
const MODULE_LIFECYCLE_AUTHORITY: &str = "gewc_module_lifecycle_supervisor";
const MODULE_LIFECYCLE_MODE: &str = "policy_audited_per_handler_lifecycle_control";

#[derive(Clone)]
pub struct GlobalExecutiveWorkspaceInput {
    pub readiness_report: String,
    pub capability_status: String,
    pub cognitive_report: String,
    pub integration_governance_report: String,
    pub paradigm_report: String,
    pub frontier_report: String,
    pub world_report: String,
    pub memory_report: String,
    pub attention_report: String,
    pub goals_report: String,
    pub plan_executor_report: String,
    pub learning_report: String,
    pub evaluation_report: String,
    pub policy_report: String,
    pub provenance_report: String,
    pub uncertainty_report: String,
    pub action_evidence_report: String,
    pub external_validation_report: String,
}

#[derive(Clone, Debug)]
pub struct CoreRuntimeContext {
    pub raw_command: String,
    pub autonomous: bool,
    pub allow_remote_crawl: bool,
    pub graph_nodes: usize,
    pub graph_edges: usize,
    pub global_tick: u64,
    pub capability_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreDisposition {
    Execute,
    Defer,
    Block,
    RequestSupervision,
}

impl CoreDisposition {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Execute => "execute",
            Self::Defer => "defer",
            Self::Block => "block",
            Self::RequestSupervision => "request_supervision",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GewcBodyHandler {
    RuntimeControl,
    MemoryReasoning,
    NativeCompatibility,
    SafeLearning,
    WorldModel,
    PlanningGoal,
    ToolAdapter,
    SpecializedModel,
    MetacognitiveSafety,
    Validation,
    Experiment,
    Agentic,
    WorkspaceAttention,
    LocusContext,
    FormalSynthesis,
    HumanInterface,
    UnknownIntent,
}

impl GewcBodyHandler {
    pub const ALL: [Self; 17] = [
        Self::RuntimeControl,
        Self::MemoryReasoning,
        Self::NativeCompatibility,
        Self::SafeLearning,
        Self::WorldModel,
        Self::PlanningGoal,
        Self::ToolAdapter,
        Self::SpecializedModel,
        Self::MetacognitiveSafety,
        Self::Validation,
        Self::Experiment,
        Self::Agentic,
        Self::WorkspaceAttention,
        Self::LocusContext,
        Self::FormalSynthesis,
        Self::HumanInterface,
        Self::UnknownIntent,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RuntimeControl => "gewc_runtime_control_body_handler",
            Self::MemoryReasoning => "gewc_memory_reasoning_body_handler",
            Self::NativeCompatibility => "gewc_native_compatibility_body_handler",
            Self::SafeLearning => "gewc_safe_learning_body_handler",
            Self::WorldModel => "gewc_world_model_body_handler",
            Self::PlanningGoal => "gewc_planning_goal_body_handler",
            Self::ToolAdapter => "gewc_tool_adapter_body_handler",
            Self::SpecializedModel => "gewc_specialized_model_body_handler",
            Self::MetacognitiveSafety => "gewc_metacognitive_safety_body_handler",
            Self::Validation => "gewc_validation_body_handler",
            Self::Experiment => "gewc_experiment_body_handler",
            Self::Agentic => "gewc_agentic_body_handler",
            Self::WorkspaceAttention => "gewc_workspace_attention_body_handler",
            Self::LocusContext => "gewc_locus_context_body_handler",
            Self::FormalSynthesis => "gewc_formal_synthesis_body_handler",
            Self::HumanInterface => "gewc_human_interface_body_handler",
            Self::UnknownIntent => "gewc_unknown_intent_body_handler",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CoreDecision {
    pub disposition: CoreDisposition,
    pub raw_command: String,
    pub command_kind: String,
    pub objective: &'static str,
    pub route: &'static str,
    pub authority: &'static str,
    pub body_domain: &'static str,
    pub body_handler: GewcBodyHandler,
    pub execution_unit: &'static str,
    pub lifecycle_policy: &'static str,
    pub selected_modules: Vec<&'static str>,
    pub primary_model_role: &'static str,
    pub model_routes: Vec<CognitiveModelRoute>,
    pub module_lifecycle: ModuleLifecycleControl,
    pub verifier_loop: &'static str,
    pub training_boundary: &'static str,
    pub memory_learning_policy: &'static str,
    pub world_model_contract: &'static str,
    pub locus_context_contract: &'static str,
    pub formal_synthesis_policy: &'static str,
    pub code_brain_policy: &'static str,
    pub missing_information: Vec<&'static str>,
    pub safety_gate: &'static str,
    pub requires_supervision: bool,
    pub reason: String,
    pub graph_nodes: usize,
    pub graph_edges: usize,
    pub global_tick: u64,
    pub autonomous: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModuleLifecycleState {
    Active,
    Paused,
    Degraded,
    Isolated,
    Quarantined,
    Disabled,
    Recovering,
    Failed,
}

impl ModuleLifecycleState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Degraded => "degraded",
            Self::Isolated => "isolated",
            Self::Quarantined => "quarantined",
            Self::Disabled => "disabled",
            Self::Recovering => "recovering",
            Self::Failed => "failed",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "paused" => Some(Self::Paused),
            "degraded" => Some(Self::Degraded),
            "isolated" => Some(Self::Isolated),
            "quarantined" => Some(Self::Quarantined),
            "disabled" => Some(Self::Disabled),
            "recovering" => Some(Self::Recovering),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModuleLifecycleAction {
    HealthCheck,
    Pause,
    Resume,
    Restart,
    Isolate,
    Quarantine,
    Disable,
    Recover,
}

impl ModuleLifecycleAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HealthCheck => "health_check",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Restart => "restart",
            Self::Isolate => "isolate",
            Self::Quarantine => "quarantine",
            Self::Disable => "disable",
            Self::Recover => "recover",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleLifecycleControl {
    pub handler: GewcBodyHandler,
    pub state: ModuleLifecycleState,
    pub allowed_actions: Vec<ModuleLifecycleAction>,
    pub supervision_required: bool,
    pub policy_gate: &'static str,
    pub isolation_scope: &'static str,
    pub audit_channel: &'static str,
}

impl ModuleLifecycleControl {
    pub fn allows(&self, action: ModuleLifecycleAction) -> bool {
        self.allowed_actions.contains(&action)
    }

    pub fn action_names(&self) -> Vec<&'static str> {
        self.allowed_actions
            .iter()
            .map(|action| action.as_str())
            .collect()
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "authority": MODULE_LIFECYCLE_AUTHORITY,
            "mode": MODULE_LIFECYCLE_MODE,
            "handler": self.handler.as_str(),
            "state": self.state.as_str(),
            "allowed_actions": self.action_names(),
            "supervision_required": self.supervision_required,
            "policy_gate": self.policy_gate,
            "isolation_scope": self.isolation_scope,
            "audit_channel": self.audit_channel,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CognitiveModelRoute {
    pub role: &'static str,
    pub adapter: &'static str,
    pub purpose: &'static str,
    pub activation: &'static str,
    pub authority: &'static str,
}

impl CognitiveModelRoute {
    fn new(
        role: &'static str,
        adapter: &'static str,
        purpose: &'static str,
        activation: &'static str,
    ) -> Self {
        Self {
            role,
            adapter,
            purpose,
            activation,
            authority: MODEL_REGISTRY_AUTHORITY,
        }
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "role": self.role,
            "adapter": self.adapter,
            "purpose": self.purpose,
            "activation": self.activation,
            "authority": self.authority,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CoreExecutionOutcome {
    pub status: &'static str,
    pub should_continue: bool,
    pub response_bytes: usize,
}

impl CoreExecutionOutcome {
    pub fn completed(response: &str, should_continue: bool) -> Self {
        Self {
            status: if should_continue {
                "completed"
            } else {
                "shutdown"
            },
            should_continue,
            response_bytes: response.len(),
        }
    }

    pub fn blocked(response: &str) -> Self {
        Self {
            status: "blocked",
            should_continue: true,
            response_bytes: response.len(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GewcBodyBinding {
    pub route: &'static str,
    pub domain: &'static str,
    pub handler: GewcBodyHandler,
    pub execution_unit: &'static str,
    pub lifecycle_policy: &'static str,
}

pub struct GewcBodyRegistry;

impl GewcBodyRegistry {
    pub fn bind(command: &GarmCommand) -> GewcBodyBinding {
        let profile = CommandProfile::from_command(command);
        GewcBodyBinding {
            route: profile.route,
            domain: profile.body_domain,
            handler: body_handler_for(profile.route),
            execution_unit: execution_unit_for(profile.body_domain),
            lifecycle_policy: lifecycle_policy_for(command),
        }
    }

    pub fn should_record_completion(command: &GarmCommand) -> bool {
        lifecycle_policy_for(command) == "decision_and_completion"
    }
}

pub struct GlobalExecutiveWorkspaceCore;

impl GlobalExecutiveWorkspaceCore {
    pub fn decide(command: &GarmCommand, ctx: CoreRuntimeContext) -> CoreDecision {
        let profile = CommandProfile::from_command(command);
        let body_binding = GewcBodyRegistry::bind(command);
        let model_routes = model_routes_for(profile.route);
        let primary_model_role = model_routes
            .first()
            .map(|route| route.role)
            .unwrap_or("no_model_selected");
        let verifier_loop = verifier_loop_for(&profile);
        let training_boundary = training_boundary_for(profile.route);
        let memory_learning_policy = memory_learning_policy_for(profile.route);
        let world_model_contract = world_model_contract_for(profile.route);
        let locus_context_contract = locus_context_contract_for(profile.route);
        let formal_synthesis_policy = formal_synthesis_policy_for(profile.route);
        let code_brain_policy = code_brain_policy_for(profile.route);
        let base_disposition = if should_block(command, &ctx, &profile) {
            CoreDisposition::Block
        } else if should_defer(command, &ctx, &profile) {
            CoreDisposition::Defer
        } else if profile.requires_supervision && !profile.local_guard_present {
            CoreDisposition::RequestSupervision
        } else {
            CoreDisposition::Execute
        };
        let persisted_lifecycle_state = persisted_module_lifecycle_state(body_binding.handler);
        let disposition = persisted_lifecycle_state
            .and_then(disposition_for_lifecycle_state)
            .unwrap_or(base_disposition);
        let mut module_lifecycle = lifecycle_control_for(body_binding.handler, &disposition);
        if let Some(state) = persisted_lifecycle_state {
            module_lifecycle.state = state;
        }
        let mut missing_information = Vec::new();
        if !ctx.capability_status.contains("garm |") {
            missing_information.push("capability_status");
        }
        if matches!(command, GarmCommand::Unknown(_)) {
            missing_information.push("known_command_intent");
        }
        let reason = if let Some(state) = persisted_lifecycle_state {
            if state == ModuleLifecycleState::Active {
                profile.reason.to_string()
            } else {
                format!(
                    "{}; module_lifecycle_state={}",
                    profile.reason,
                    state.as_str()
                )
            }
        } else {
            profile.reason.to_string()
        };
        CoreDecision {
            disposition,
            raw_command: ctx.raw_command,
            command_kind: command_kind(command),
            objective: profile.objective,
            route: profile.route,
            authority: "global_executive_workspace_core",
            body_domain: body_binding.domain,
            body_handler: body_binding.handler,
            execution_unit: body_binding.execution_unit,
            lifecycle_policy: body_binding.lifecycle_policy,
            selected_modules: profile.selected_modules,
            primary_model_role,
            model_routes,
            module_lifecycle,
            verifier_loop,
            training_boundary,
            memory_learning_policy,
            world_model_contract,
            locus_context_contract,
            formal_synthesis_policy,
            code_brain_policy,
            missing_information,
            safety_gate: profile.safety_gate,
            requires_supervision: profile.requires_supervision,
            reason,
            graph_nodes: ctx.graph_nodes,
            graph_edges: ctx.graph_edges,
            global_tick: ctx.global_tick,
            autonomous: ctx.autonomous,
        }
    }

    pub fn lifecycle_control_for_handler(handler: GewcBodyHandler) -> ModuleLifecycleControl {
        let mut control = lifecycle_control_for(handler, &CoreDisposition::Execute);
        if let Some(state) = persisted_module_lifecycle_state(handler) {
            control.state = state;
        }
        control
    }

    pub fn supervise_module(
        handler: GewcBodyHandler,
        action: ModuleLifecycleAction,
        reason: &str,
    ) -> String {
        let _ = state_paths::ensure_state_dir();
        let mut control = Self::lifecycle_control_for_handler(handler);
        let prior_state = control.state;
        let action_allowed = control.allows(action);
        if action_allowed {
            control.state = lifecycle_state_after_action(action, prior_state);
        }
        let record = serde_json::json!({
            "schema": "garm-global-executive-workspace-module-lifecycle-v1",
            "phase": "module_lifecycle_control",
            "core_authority": "global_executive_workspace_core",
            "module_lifecycle_authority": MODULE_LIFECYCLE_AUTHORITY,
            "module_lifecycle_mode": MODULE_LIFECYCLE_MODE,
            "body_handler": handler.as_str(),
            "module_lifecycle_action": action.as_str(),
            "module_lifecycle_action_allowed": action_allowed,
            "prior_module_lifecycle_state": prior_state.as_str(),
            "module_lifecycle_state": control.state.as_str(),
            "allowed_actions": control.action_names(),
            "policy_gate": control.policy_gate,
            "isolation_scope": control.isolation_scope,
            "supervision_required": control.supervision_required,
            "audit_channel": control.audit_channel,
            "reason": reason,
            "claim_allowed": false,
            "agi_claim": false,
        });
        let line = serde_json::to_string(&record).unwrap_or_else(|_| record.to_string());
        let path = state_paths::global_executive_workspace_runtime_path();
        let write_status = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut file| writeln!(file, "{line}"))
            .map(|_| "module_lifecycle_recorded")
            .unwrap_or("module_lifecycle_record_failed");
        let state = serde_json::json!({
            "schema": "garm-global-executive-workspace-runtime-state-v1",
            "core_authority": "global_executive_workspace_core",
            "absorption_model": ABSORPTION_MODEL,
            "legacy_mode": LEGACY_MODE,
            "garm_mode": GARM_MODE,
            "external_cores_remaining": false,
            "body_domains": body_domains(),
            "handler_topology": HANDLER_TOPOLOGY,
            "shared_body_engine": SHARED_BODY_ENGINE,
            "model_registry_authority": MODEL_REGISTRY_AUTHORITY,
            "model_control_mode": MODEL_CONTROL_MODE,
            "module_lifecycle_authority": MODULE_LIFECYCLE_AUTHORITY,
            "module_lifecycle_mode": MODULE_LIFECYCLE_MODE,
            "last_module_lifecycle": record,
            "claim_allowed": false,
            "agi_claim": false,
        });
        let _ = std::fs::write(
            state_paths::global_executive_workspace_runtime_state_path(),
            serde_json::to_string_pretty(&state).unwrap_or_else(|_| state.to_string()),
        );
        format!(
            "[GEWC-MODULE-LIFECYCLE] handler={} action={} allowed={} prior_state={} state={} authority={} write_status={} path={}\n",
            handler.as_str(),
            action.as_str(),
            action_allowed,
            prior_state.as_str(),
            control.state.as_str(),
            MODULE_LIFECYCLE_AUTHORITY,
            write_status,
            path
        )
    }

    pub fn record_decision(decision: &CoreDecision) -> String {
        let _ = state_paths::ensure_state_dir();
        let record = decision.to_json("decision_started");
        let line = serde_json::to_string(&record).unwrap_or_else(|_| record.to_string());
        let path = state_paths::global_executive_workspace_runtime_path();
        let write_status = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut file| writeln!(file, "{line}"))
            .map(|_| "decision_recorded")
            .unwrap_or("decision_record_failed");
        let state = serde_json::json!({
            "schema": "garm-global-executive-workspace-runtime-state-v1",
            "core_authority": "global_executive_workspace_core",
            "absorption_model": ABSORPTION_MODEL,
            "legacy_mode": LEGACY_MODE,
            "garm_mode": GARM_MODE,
            "external_cores_remaining": false,
            "body_domains": body_domains(),
            "handler_topology": HANDLER_TOPOLOGY,
            "shared_body_engine": SHARED_BODY_ENGINE,
            "model_registry_authority": MODEL_REGISTRY_AUTHORITY,
            "model_control_mode": MODEL_CONTROL_MODE,
            "module_lifecycle_authority": MODULE_LIFECYCLE_AUTHORITY,
            "module_lifecycle_mode": MODULE_LIFECYCLE_MODE,
            "model_weights_mutation_allowed": false,
            "training_boundary": decision.training_boundary,
            "last_decision": record,
            "claim_allowed": false,
            "agi_claim": false,
        });
        let _ = std::fs::write(
            state_paths::global_executive_workspace_runtime_state_path(),
            serde_json::to_string_pretty(&state).unwrap_or_else(|_| state.to_string()),
        );
        let _ = crate::eden_garm::runtime_spine::publish_event(
            "global_executive_workspace",
            decision.body_domain,
            "decision_started",
            if decision.requires_supervision {
                "medium"
            } else {
                "low"
            },
            record.clone(),
        );
        let _ = crate::eden_garm::runtime_spine::record_state_mutation(
            "global_executive_workspace",
            "last_decision",
            "write",
            decision.disposition.as_str(),
            record,
        );
        format!(
            "[GEWC-DECISION] disposition={} route={} body_domain={} body_handler={} module_lifecycle_state={} absorption_model={} safety_gate={} write_status={} path={}\n",
            decision.disposition.as_str(),
            decision.route,
            decision.body_domain,
            decision.body_handler.as_str(),
            decision.module_lifecycle.state.as_str(),
            ABSORPTION_MODEL,
            decision.safety_gate,
            write_status,
            path
        )
    }

    pub fn record_execution_completion(
        decision: &CoreDecision,
        outcome: CoreExecutionOutcome,
    ) -> String {
        let _ = state_paths::ensure_state_dir();
        let record = decision.completion_json(&outcome);
        let line = serde_json::to_string(&record).unwrap_or_else(|_| record.to_string());
        let path = state_paths::global_executive_workspace_runtime_path();
        let write_status = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut file| writeln!(file, "{line}"))
            .map(|_| "execution_completion_recorded")
            .unwrap_or("execution_completion_record_failed");
        let state = serde_json::json!({
            "schema": "garm-global-executive-workspace-runtime-state-v1",
            "core_authority": "global_executive_workspace_core",
            "absorption_model": ABSORPTION_MODEL,
            "legacy_mode": LEGACY_MODE,
            "garm_mode": GARM_MODE,
            "external_cores_remaining": false,
            "body_domains": body_domains(),
            "handler_topology": HANDLER_TOPOLOGY,
            "shared_body_engine": SHARED_BODY_ENGINE,
            "model_registry_authority": MODEL_REGISTRY_AUTHORITY,
            "model_control_mode": MODEL_CONTROL_MODE,
            "module_lifecycle_authority": MODULE_LIFECYCLE_AUTHORITY,
            "module_lifecycle_mode": MODULE_LIFECYCLE_MODE,
            "model_weights_mutation_allowed": false,
            "training_boundary": decision.training_boundary,
            "last_decision": decision.to_json("decision_started"),
            "last_completion": record,
            "claim_allowed": false,
            "agi_claim": false,
        });
        let _ = std::fs::write(
            state_paths::global_executive_workspace_runtime_state_path(),
            serde_json::to_string_pretty(&state).unwrap_or_else(|_| state.to_string()),
        );
        let _ = crate::eden_garm::runtime_spine::publish_event(
            "global_executive_workspace",
            decision.body_domain,
            "execution_completed",
            if outcome.status == "blocked" {
                "medium"
            } else {
                "low"
            },
            record.clone(),
        );
        let _ = crate::eden_garm::runtime_spine::record_state_mutation(
            "global_executive_workspace",
            "last_completion",
            "write",
            outcome.status,
            record,
        );
        format!(
            "[GEWC-CYCLE] status={} disposition={} route={} body_domain={} body_handler={} lifecycle_policy={} module_lifecycle_state={} response_bytes={} should_continue={} write_status={} path={}\n",
            outcome.status,
            decision.disposition.as_str(),
            decision.route,
            decision.body_domain,
            decision.body_handler.as_str(),
            decision.lifecycle_policy,
            decision.module_lifecycle.state.as_str(),
            outcome.response_bytes,
            outcome.should_continue,
            write_status,
            path
        )
    }

    pub fn runtime_report() -> String {
        let path = state_paths::global_executive_workspace_runtime_path();
        let body = std::fs::read_to_string(&path).unwrap_or_default();
        let records: Vec<serde_json::Value> = body
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .collect();
        let decisions = records
            .iter()
            .filter(|value| {
                value
                    .get("phase")
                    .and_then(|phase| phase.as_str())
                    .map_or(true, |phase| phase == "decision_started")
            })
            .count();
        let completions = records
            .iter()
            .filter(|value| {
                value
                    .get("phase")
                    .and_then(|phase| phase.as_str())
                    .is_some_and(|phase| phase == "execution_completed")
            })
            .count();
        let last_route = records
            .iter()
            .rev()
            .find_map(|value| {
                value
                    .get("route")
                    .and_then(|route| route.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "none".to_string());
        let last_handler = records
            .iter()
            .rev()
            .find_map(|value| {
                value
                    .get("body_handler")
                    .and_then(|handler| handler.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "none".to_string());
        let last_primary_model = records
            .iter()
            .rev()
            .find_map(|value| {
                value
                    .get("primary_model_role")
                    .and_then(|role| role.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "none".to_string());
        let last_lifecycle_state = records
            .iter()
            .rev()
            .find_map(|value| {
                value
                    .get("module_lifecycle_state")
                    .and_then(|state| state.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "none".to_string());
        let mut handler_counts = std::collections::BTreeMap::<String, (usize, usize, usize)>::new();
        let mut model_counts = std::collections::BTreeMap::<String, usize>::new();
        let mut lifecycle_counts = std::collections::BTreeMap::<String, usize>::new();
        for record in &records {
            if let Some(state) = record
                .get("module_lifecycle_state")
                .and_then(|value| value.as_str())
            {
                *lifecycle_counts.entry(state.to_string()).or_default() += 1;
            }
            let Some(handler) = record.get("body_handler").and_then(|value| value.as_str()) else {
                continue;
            };
            let phase = record
                .get("phase")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let entry = handler_counts.entry(handler.to_string()).or_default();
            match phase {
                "decision_started" => entry.0 += 1,
                "execution_completed" => {
                    entry.1 += 1;
                    if record
                        .get("execution_status")
                        .and_then(|value| value.as_str())
                        .is_some_and(|status| status == "blocked")
                    {
                        entry.2 += 1;
                    }
                }
                _ => {}
            }
            if phase == "decision_started" {
                if let Some(model_role) = record
                    .get("primary_model_role")
                    .and_then(|value| value.as_str())
                {
                    *model_counts.entry(model_role.to_string()).or_default() += 1;
                }
            }
        }
        let handler_metrics = if handler_counts.is_empty() {
            "none".to_string()
        } else {
            handler_counts
                .into_iter()
                .map(|(handler, (decisions, completions, blocked))| {
                    format!("{handler}:d{decisions}/c{completions}/b{blocked}")
                })
                .collect::<Vec<_>>()
                .join("|")
        };
        let model_metrics = if model_counts.is_empty() {
            "none".to_string()
        } else {
            model_counts
                .into_iter()
                .map(|(role, count)| format!("{role}:{count}"))
                .collect::<Vec<_>>()
                .join("|")
        };
        let lifecycle_metrics = if lifecycle_counts.is_empty() {
            "none".to_string()
        } else {
            lifecycle_counts
                .into_iter()
                .map(|(state, count)| format!("{state}:{count}"))
                .collect::<Vec<_>>()
                .join("|")
        };
        format!(
            "[GEWC-RUNTIME] decisions={} completions={} core_authority=global_executive_workspace_core body_executor={} handler_dispatch={} handler_topology={} shared_body_engine={} absorption_model={} legacy_mode={} garm_mode={} external_cores_remaining=false model_control_mode={} model_registry={} module_lifecycle={} module_lifecycle_mode={} last_route={} last_handler={} last_primary_model={} last_lifecycle_state={} handler_metrics={} model_metrics={} lifecycle_metrics={} path={}\n",
            decisions,
            completions,
            BODY_EXECUTOR,
            HANDLER_DISPATCH,
            HANDLER_TOPOLOGY,
            SHARED_BODY_ENGINE,
            ABSORPTION_MODEL,
            LEGACY_MODE,
            GARM_MODE,
            MODEL_CONTROL_MODE,
            MODEL_REGISTRY_AUTHORITY,
            MODULE_LIFECYCLE_AUTHORITY,
            MODULE_LIFECYCLE_MODE,
            last_route,
            last_handler,
            last_primary_model,
            last_lifecycle_state,
            handler_metrics,
            model_metrics,
            lifecycle_metrics,
            path
        )
    }

    pub fn replay_index_json(limit: usize) -> String {
        let records = runtime_records();
        let decisions: Vec<_> = records
            .iter()
            .filter(|record| {
                record
                    .get("phase")
                    .and_then(|phase| phase.as_str())
                    .is_some_and(|phase| phase == "decision_started")
            })
            .rev()
            .take(limit)
            .map(|record| {
                serde_json::json!({
                    "decision_id": decision_id_for_record(record),
                    "raw_command": record.get("raw_command").cloned().unwrap_or(serde_json::Value::Null),
                    "command_kind": record.get("command_kind").cloned().unwrap_or(serde_json::Value::Null),
                    "route": record.get("route").cloned().unwrap_or(serde_json::Value::Null),
                    "body_handler": record.get("body_handler").cloned().unwrap_or(serde_json::Value::Null),
                    "disposition": record.get("disposition").cloned().unwrap_or(serde_json::Value::Null),
                    "module_lifecycle_state": record.get("module_lifecycle_state").cloned().unwrap_or(serde_json::Value::Null),
                    "global_tick": record.get("global_tick").cloned().unwrap_or(serde_json::Value::Null),
                })
            })
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "eden-gewc-replay-index-v1",
            "claim_allowed": false,
            "agi_claim": false,
            "runtime_log": state_paths::global_executive_workspace_runtime_path(),
            "records": records.len(),
            "decisions": decisions.len(),
            "latest_decisions": decisions,
            "read_decision_endpoint": "/api/operational/replay?decision_id=<id>",
            "reexecution_policy": "read_only_replay_no_external_action_reexecution",
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }

    pub fn replay_decision_json(decision_id: &str) -> String {
        let requested = decision_id.trim();
        let records = runtime_records();
        let mut decision = None;
        let mut completion = None;
        for record in &records {
            let record_id = decision_id_for_record(record);
            if record_id != requested {
                continue;
            }
            match record.get("phase").and_then(|phase| phase.as_str()) {
                Some("decision_started") => decision = Some(record.clone()),
                Some("execution_completed") => completion = Some(record.clone()),
                _ => {}
            }
        }
        let found = decision.is_some() || completion.is_some();
        let raw_command = decision
            .as_ref()
            .or(completion.as_ref())
            .and_then(|record| record.get("raw_command"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let action_evidence = matching_action_evidence_records(&raw_command);
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "eden-gewc-decision-replay-v1",
            "claim_allowed": false,
            "agi_claim": false,
            "decision_id": requested,
            "found": found,
            "runtime_log": state_paths::global_executive_workspace_runtime_path(),
            "decision": decision,
            "completion": completion,
            "execution_chain": replay_execution_chain(&raw_command, &action_evidence),
            "action_evidence": action_evidence,
            "replay_scope": [
                "input",
                "route",
                "handler",
                "policy",
                "lifecycle",
                "outcome",
                "evidence"
            ],
            "reexecution_policy": "read_only_replay_no_external_action_reexecution",
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }
}

impl CoreDecision {
    pub fn decision_id(&self) -> String {
        format!(
            "gewc-{:016x}",
            fnv64(
                format!(
                    "{}|{}|{}|{}|{}",
                    self.raw_command,
                    self.command_kind,
                    self.route,
                    self.global_tick,
                    self.graph_nodes
                )
                .as_bytes()
            )
        )
    }

    pub fn is_blocked(&self) -> bool {
        matches!(
            self.disposition,
            CoreDisposition::Defer | CoreDisposition::Block | CoreDisposition::RequestSupervision
        )
    }

    pub fn blocked_response(&self) -> String {
        let prefix = if matches!(self.disposition, CoreDisposition::Defer) {
            "[GEWC-DEFERRED]"
        } else {
            "[GEWC-BLOCKED]"
        };
        format!(
            "{} command_kind={} disposition={} route={} module_lifecycle_state={} safety_gate={} reason={}\n",
            prefix,
            self.command_kind,
            self.disposition.as_str(),
            self.route,
            self.module_lifecycle.state.as_str(),
            self.safety_gate,
            self.reason
        )
    }

    fn to_json(&self, phase: &str) -> serde_json::Value {
        let model_routes: Vec<_> = self
            .model_routes
            .iter()
            .map(CognitiveModelRoute::to_json)
            .collect();
        let decision_id = self.decision_id();
        serde_json::json!({
            "schema": "garm-global-executive-workspace-decision-v1",
            "phase": phase,
            "decision_id": decision_id,
            "core_authority": self.authority,
            "body_executor": BODY_EXECUTOR,
            "handler_dispatch": HANDLER_DISPATCH,
            "handler_topology": HANDLER_TOPOLOGY,
            "shared_body_engine": SHARED_BODY_ENGINE,
            "absorption_model": ABSORPTION_MODEL,
            "legacy_mode": LEGACY_MODE,
            "garm_mode": GARM_MODE,
            "external_cores_remaining": false,
            "raw_command": self.raw_command,
            "command_kind": self.command_kind,
            "objective": self.objective,
            "route": self.route,
            "body_domain": self.body_domain,
            "body_handler": self.body_handler.as_str(),
            "execution_unit": self.execution_unit,
            "lifecycle_policy": self.lifecycle_policy,
            "selected_modules": self.selected_modules,
            "module_lifecycle_authority": MODULE_LIFECYCLE_AUTHORITY,
            "module_lifecycle_mode": MODULE_LIFECYCLE_MODE,
            "module_lifecycle": self.module_lifecycle.to_json(),
            "module_lifecycle_state": self.module_lifecycle.state.as_str(),
            "module_lifecycle_allowed_actions": self.module_lifecycle.action_names(),
            "model_registry_authority": MODEL_REGISTRY_AUTHORITY,
            "model_control_mode": MODEL_CONTROL_MODE,
            "primary_model_role": self.primary_model_role,
            "model_routes": model_routes,
            "verifier_loop": self.verifier_loop,
            "training_boundary": self.training_boundary,
            "memory_learning_policy": self.memory_learning_policy,
            "world_model_contract": self.world_model_contract,
            "locus_context_contract": self.locus_context_contract,
            "formal_synthesis_policy": self.formal_synthesis_policy,
            "code_brain_policy": self.code_brain_policy,
            "model_weights_mutation_allowed": false,
            "missing_information": self.missing_information,
            "disposition": self.disposition.as_str(),
            "safety_gate": self.safety_gate,
            "requires_supervision": self.requires_supervision,
            "reason": self.reason,
            "graph_nodes": self.graph_nodes,
            "graph_edges": self.graph_edges,
            "global_tick": self.global_tick,
            "autonomous": self.autonomous,
            "claim_allowed": false,
            "agi_claim": false,
        })
    }

    fn completion_json(&self, outcome: &CoreExecutionOutcome) -> serde_json::Value {
        let model_routes: Vec<_> = self
            .model_routes
            .iter()
            .map(CognitiveModelRoute::to_json)
            .collect();
        let decision_id = self.decision_id();
        serde_json::json!({
            "schema": "garm-global-executive-workspace-execution-v1",
            "phase": "execution_completed",
            "decision_id": decision_id,
            "core_authority": self.authority,
            "body_executor": BODY_EXECUTOR,
            "handler_dispatch": HANDLER_DISPATCH,
            "handler_topology": HANDLER_TOPOLOGY,
            "shared_body_engine": SHARED_BODY_ENGINE,
            "absorption_model": ABSORPTION_MODEL,
            "legacy_mode": LEGACY_MODE,
            "garm_mode": GARM_MODE,
            "external_cores_remaining": false,
            "raw_command": self.raw_command,
            "command_kind": self.command_kind,
            "objective": self.objective,
            "route": self.route,
            "body_domain": self.body_domain,
            "body_handler": self.body_handler.as_str(),
            "execution_unit": self.execution_unit,
            "lifecycle_policy": self.lifecycle_policy,
            "selected_modules": self.selected_modules,
            "module_lifecycle_authority": MODULE_LIFECYCLE_AUTHORITY,
            "module_lifecycle_mode": MODULE_LIFECYCLE_MODE,
            "module_lifecycle": self.module_lifecycle.to_json(),
            "module_lifecycle_state": self.module_lifecycle.state.as_str(),
            "module_lifecycle_allowed_actions": self.module_lifecycle.action_names(),
            "model_registry_authority": MODEL_REGISTRY_AUTHORITY,
            "model_control_mode": MODEL_CONTROL_MODE,
            "primary_model_role": self.primary_model_role,
            "model_routes": model_routes,
            "verifier_loop": self.verifier_loop,
            "training_boundary": self.training_boundary,
            "memory_learning_policy": self.memory_learning_policy,
            "world_model_contract": self.world_model_contract,
            "locus_context_contract": self.locus_context_contract,
            "formal_synthesis_policy": self.formal_synthesis_policy,
            "code_brain_policy": self.code_brain_policy,
            "model_weights_mutation_allowed": false,
            "disposition": self.disposition.as_str(),
            "safety_gate": self.safety_gate,
            "requires_supervision": self.requires_supervision,
            "execution_status": outcome.status,
            "should_continue": outcome.should_continue,
            "response_bytes": outcome.response_bytes,
            "graph_nodes": self.graph_nodes,
            "graph_edges": self.graph_edges,
            "global_tick": self.global_tick,
            "autonomous": self.autonomous,
            "claim_allowed": false,
            "agi_claim": false,
        })
    }
}

fn runtime_records() -> Vec<serde_json::Value> {
    std::fs::read_to_string(state_paths::global_executive_workspace_runtime_path())
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect()
}

fn matching_action_evidence_records(raw_command: &str) -> Vec<serde_json::Value> {
    if raw_command.is_empty() {
        return Vec::new();
    }
    std::fs::read_to_string(state_paths::action_evidence_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|record| {
            record
                .get("intent")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|intent| intent.contains(raw_command))
                || record
                    .get("source")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|source| source.contains(raw_command))
        })
        .take(8)
        .collect()
}

fn replay_execution_chain(
    raw_command: &str,
    action_evidence: &[serde_json::Value],
) -> serde_json::Value {
    serde_json::json!({
        "schema": "eden-gewc-replay-execution-chain-v1",
        "raw_command": raw_command,
        "chain": [
            "command_input",
            "command_router_parse",
            "gewc_decision",
            "persistent_permission_gate",
            "module_lifecycle_gate",
            "body_handler_dispatch",
            "execution_or_block",
            "action_evidence",
            "completion_record"
        ],
        "permission_source": state_paths::operational_permissions_path(),
        "action_evidence_path": state_paths::action_evidence_path(),
        "matching_action_evidence_records": action_evidence.len(),
        "replay_policy": "inspect_only_no_reexecution",
    })
}

fn decision_id_for_record(record: &serde_json::Value) -> String {
    if let Some(decision_id) = record.get("decision_id").and_then(|value| value.as_str()) {
        return decision_id.to_string();
    }
    let raw_command = record
        .get("raw_command")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let command_kind = record
        .get("command_kind")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let route = record
        .get("route")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let global_tick = record
        .get("global_tick")
        .and_then(|value| value.as_u64())
        .unwrap_or_default();
    let graph_nodes = record
        .get("graph_nodes")
        .and_then(|value| value.as_u64())
        .unwrap_or_default();
    format!(
        "gewc-{:016x}",
        fnv64(
            format!("{raw_command}|{command_kind}|{route}|{global_tick}|{graph_nodes}").as_bytes()
        )
    )
}

struct CommandProfile {
    objective: &'static str,
    route: &'static str,
    body_domain: &'static str,
    selected_modules: Vec<&'static str>,
    safety_gate: &'static str,
    requires_supervision: bool,
    local_guard_present: bool,
    reason: &'static str,
}

impl CommandProfile {
    fn from_command(command: &GarmCommand) -> Self {
        match command {
            GarmCommand::Quit
            | GarmCommand::Tick
            | GarmCommand::Status
            | GarmCommand::Save
            | GarmCommand::Load
            | GarmCommand::Auto(_)
            | GarmCommand::Start
            | GarmCommand::Stop
            | GarmCommand::GarmAudit
            | GarmCommand::GarmReport
            | GarmCommand::GarmReportHistory
            | GarmCommand::GarmExport
            | GarmCommand::GarmImport
            | GarmCommand::GarmVerifyExport
            | GarmCommand::GarmArtifacts
            | GarmCommand::GarmBackup
            | GarmCommand::GarmRestore
            | GarmCommand::GarmCompact => Self::new(
                "runtime_control",
                "runtime_control",
                RUNTIME_BODY,
                vec!["global_workspace", "runtime_state", "persistence"],
                "local_runtime_guard",
                false,
                true,
                "runtime control is governed by the executive workspace before execution",
            ),
            GarmCommand::Remember(_)
            | GarmCommand::Memory
            | GarmCommand::MemoryEval
            | GarmCommand::OperationalMemoryCommit(_)
            | GarmCommand::OperationalMemoryRollback(_)
            | GarmCommand::Query(_)
            | GarmCommand::WhatIs(_)
            | GarmCommand::Why(_)
            | GarmCommand::TellMe(_)
            | GarmCommand::Cag
            | GarmCommand::CagExplain(_)
            | GarmCommand::CagGaps(_)
            | GarmCommand::CagActions
            | GarmCommand::CagAudit
            | GarmCommand::CagPlan(_)
            | GarmCommand::CagRun(_) => Self::new(
                "retrieve_reason_or_update_memory",
                "memory_reasoning",
                ADAPTER_BODY,
                vec!["working_memory", "long_term_memory", "knowledge_graph", "hrm_text", "cag"],
                "memory_integrity_guard",
                false,
                true,
                "memory and retrieval are native GEWC body functions with preserved legacy sources",
            ),
            GarmCommand::Greeting
            | GarmCommand::SelfQuery
            | GarmCommand::Thinking
            | GarmCommand::Feeling
            | GarmCommand::Phi
            | GarmCommand::Observatory
            | GarmCommand::History
            | GarmCommand::OrganicRitual
            | GarmCommand::Lengua(_)
            | GarmCommand::Reloj(_)
            | GarmCommand::Juez(_)
            | GarmCommand::Intestino
            | GarmCommand::Piel
            | GarmCommand::Autotuning
            | GarmCommand::Migration => Self::new(
                "legacy_compatibility_under_unified_core",
                "legacy_compatibility",
                ADAPTER_BODY,
                vec![
                    "global_workspace",
                    "native_dialogue_adapter",
                    "native_history_adapter",
                    "capability_status",
                ],
                "compatibility_guard",
                false,
                true,
                "historical surfaces are absorbed as native GEWC compatibility body adapters",
            ),
            GarmCommand::Evolve
            | GarmCommand::SelfImprovementEval
            | GarmCommand::Learning
            | GarmCommand::LearningRecord(_)
            | GarmCommand::LearningConsolidate
            | GarmCommand::LearningAudit
            | GarmCommand::Maturity
            | GarmCommand::MaturityAssess(_)
            | GarmCommand::MaturityAudit => Self::new(
                "safe_continual_learning",
                "learning_self_improvement",
                NATIVE_BODY,
                vec!["learning_ledger", "self_improvement", "policy_guard", "provenance"],
                "safe_learning_guard",
                true,
                true,
                "learning is allowed only through bounded ledgers and policy-reviewed updates",
            ),
            GarmCommand::WorldModel
            | GarmCommand::WorldObserve(_)
            | GarmCommand::WorldPredict(_)
            | GarmCommand::WorldVerify
            | GarmCommand::WorldAudit
            | GarmCommand::WorldEval => Self::new(
                "model_or_simulate_world",
                "world_model",
                NATIVE_BODY,
                vec!["world_model", "causal_model", "verification"],
                "world_model_verification_guard",
                false,
                true,
                "world modeling is selected by GEWC for prediction and causal verification",
            ),
            GarmCommand::PlanExecutor
            | GarmCommand::PlanExecutorPlan(_)
            | GarmCommand::PlanExecutorRun
            | GarmCommand::PlanExecutorAudit
            | GarmCommand::OperationalTaskSubmit(_)
            | GarmCommand::OperationalTaskRun
            | GarmCommand::OperationalTaskAudit
            | GarmCommand::ParadiseIntent(_)
            | GarmCommand::ParadisePlan(_)
            | GarmCommand::ParadiseApprove(_)
            | GarmCommand::ParadiseExecute(_)
            | GarmCommand::ParadiseAudit
            | GarmCommand::Goals
            | GarmCommand::GoalsPlan(_)
            | GarmCommand::GoalsRun
            | GarmCommand::GoalsAudit
            | GarmCommand::ReadinessPlan
            | GarmCommand::ReadinessRun => Self::new(
                "plan_or_execute_goal",
                "planning_goals",
                NATIVE_BODY,
                vec!["goal_manager", "hierarchical_planner", "plan_executor", "policy_guard"],
                "goal_stability_guard",
                false,
                true,
                "goal and plan execution is routed through the executive planner",
            ),
            GarmCommand::Crawl(_) | GarmCommand::ConceptNet(_) | GarmCommand::HrmTextCorpus(_) | GarmCommand::HrmTextIngest(_) | GarmCommand::OperationalActionExecute(_) => Self::new(
                "use_external_or_file_tool",
                "tool_use",
                ADAPTER_BODY,
                vec!["tool_router", "policy_guard", "provenance", "native_crawler_adapter"],
                "tool_permission_guard",
                true,
                true,
                "tool use is absorbed into the native GEWC adapter body with explicit guards and provenance",
            ),
            GarmCommand::Hrm(_)
            | GarmCommand::HrmRun(_)
            | GarmCommand::HrmText
            | GarmCommand::HrmTextSearch(_)
            | GarmCommand::HrmTextContext(_)
            | GarmCommand::HrmTextEval
            | GarmCommand::HrmTextObjective(_)
            | GarmCommand::HrmTextPlan
            | GarmCommand::HrmTextRun
            | GarmCommand::HrmTextAudit
            | GarmCommand::HybridVoice
            | GarmCommand::HybridVoicePlan(_)
            | GarmCommand::HybridVoiceSynth(_)
            | GarmCommand::HybridVoiceAudit
            | GarmCommand::Voz
            | GarmCommand::VozTexto(_)
            | GarmCommand::ModelRegister(_)
            | GarmCommand::ModelLoad(_)
            | GarmCommand::ModelEvaluate(_)
            | GarmCommand::ModelUnload(_)
            | GarmCommand::ModelAudit => Self::new(
                "activate_specialized_model_or_modality",
                "specialized_model",
                NATIVE_BODY,
                vec!["model_router", "hrm", "voice", "working_memory"],
                "model_output_guard",
                false,
                true,
                "specialized models are tools selected by GEWC, not separate cores",
            ),
            GarmCommand::Policy
            | GarmCommand::PolicyEval(_)
            | GarmCommand::PolicyAudit
            | GarmCommand::GewcLifecycleControl(_)
            | GarmCommand::OperationalPermissionsAudit
            | GarmCommand::OperationalPermissionsDiff
            | GarmCommand::OperationalPermissionsHistory
            | GarmCommand::OperationalPermissionsRestore
            | GarmCommand::OperationalPermissionsSet(_)
            | GarmCommand::OperationalRecoveryAudit
            | GarmCommand::OperationalRecoveryRun
            | GarmCommand::ActionEvidence
            | GarmCommand::Provenance
            | GarmCommand::ProvenanceRecord(_)
            | GarmCommand::ProvenanceVerify
            | GarmCommand::ProvenanceAudit
            | GarmCommand::Uncertainty
            | GarmCommand::UncertaintyRecord(_)
            | GarmCommand::UncertaintyResolve
            | GarmCommand::UncertaintyAudit => Self::new(
                "regulate_verify_or_audit",
                "metacognitive_safety",
                NATIVE_BODY,
                vec!["policy_guard", "provenance", "uncertainty", "verifier_critic"],
                "metacognitive_safety_guard",
                false,
                true,
                "safety and audit are native regulation inside the executive core",
            ),
            GarmCommand::Evaluation
            | GarmCommand::EvaluationRun
            | GarmCommand::EvaluationAudit
            | GarmCommand::Benchmark
            | GarmCommand::BenchmarkRun
            | GarmCommand::BenchmarkAudit
            | GarmCommand::Readiness
            | GarmCommand::ReadinessBench
            | GarmCommand::ReadinessProbe
            | GarmCommand::ReadinessExternal
            | GarmCommand::ReadinessExternalRun
            | GarmCommand::ReadinessPackage
            | GarmCommand::CapabilityRegistry
            | GarmCommand::CognitiveEval
            | GarmCommand::EmbodiedEval
            | GarmCommand::NeuralEval
            | GarmCommand::SymbolicEval
            | GarmCommand::FrontierArchitectureEval
            | GarmCommand::ParadigmArchitectureEval
            | GarmCommand::IntegrationGovernanceEval
            | GarmCommand::GlobalExecutiveWorkspaceEval
            | GarmCommand::GewcOperationalBenchmark
            | GarmCommand::CapabilityRealityEval
            | GarmCommand::ArchitectureAdvantageEval
            | GarmCommand::ParadiseWorldcellEval
            | GarmCommand::ExternalEcosystemEval
            | GarmCommand::SovereignCognitionEval
            | GarmCommand::ArtifactApiEval
            | GarmCommand::TrainingEvidenceEval
            | GarmCommand::Megatron7bEvidenceEval
            | GarmCommand::Megatron7bAdapterPrepare
            | GarmCommand::Megatron7bInferenceEval
            | GarmCommand::Megatron7bCapabilityEval
            | GarmCommand::Megatron7bAdmissionGateEval
            | GarmCommand::EdenCapableEval
            | GarmCommand::EdenCapableTrainingRunContract
            | GarmCommand::EdenCognitiveDatasetEval
            | GarmCommand::EdenNativeInferenceEval
            | GarmCommand::EdenCapabilityDeltaEval
            | GarmCommand::EdenStructuredOutputEval
            | GarmCommand::EdenCheckpointRegistryEval
            | GarmCommand::EdenSftElcpReadinessEval
            | GarmCommand::EdenCapableGateEval
            | GarmCommand::EdenCapableOperationalize
            | GarmCommand::EdenLearnedCapabilityEval
            | GarmCommand::EdenRealCapabilityEval
            | GarmCommand::ModelRuntimeEval
            | GarmCommand::ModelAdapterRuntimeEval
            | GarmCommand::ModelCheckpointManifestEval
            | GarmCommand::TrainingHarnessEval
            | GarmCommand::ModelGovernanceEval
            | GarmCommand::FirstModelPrepare
            | GarmCommand::FirstModelReadinessEval
            | GarmCommand::ElcpPrepare
            | GarmCommand::ElcpObjectiveEval
            | GarmCommand::ElcpAdmissionGateEval
            | GarmCommand::ElcpTraceQualityEval
            | GarmCommand::ElcpReplayEval
            | GarmCommand::ElcpDatasetFreezeEval
            | GarmCommand::ElcpMetricsBoardEval
            | GarmCommand::Elcp4bReadinessContractEval
            | GarmCommand::ElcpHardeningEval
            | GarmCommand::ElcpReadinessEval
            | GarmCommand::RuntimeStateApiEval
            | GarmCommand::OperationalApiEval
            | GarmCommand::OperationalRuntimeEval
            | GarmCommand::OperationalReplayRun
            | GarmCommand::OperationalSmokeRun
            | GarmCommand::OperationalScenarioRun
            | GarmCommand::OperationalDemoRun
            | GarmCommand::RuntimeSpineEval
            | GarmCommand::RuntimeSpineAudit
            | GarmCommand::RuntimeSpineVerify
            | GarmCommand::RuntimeSpineEnforce
            | GarmCommand::RuntimeSpineRisk
            | GarmCommand::RuntimeSpineBreakers
            | GarmCommand::RuntimeSpineReplay => Self::new(
                "evaluate_architecture",
                "evaluation_validation",
                NATIVE_BODY,
                vec!["evaluation_loop", "benchmark", "readiness", "capability_registry"],
                "no_claim_validation_guard",
                false,
                true,
                "validation is routed through GEWC and preserves no-claim policy",
            ),
            GarmCommand::PraxisNexusEval
            | GarmCommand::OperatorForgeEval
            | GarmCommand::OperatorForgeSynthesize(_)
            | GarmCommand::OperatorForgeVerify
            | GarmCommand::OperatorForgeAudit => Self::new(
                "synthesize_or_verify_formal_primitives",
                "formal_synthesis",
                NATIVE_BODY,
                vec![
                    "praxis_nexus",
                    "operator_forge",
                    "typed_expression_graphs",
                    "formal_verifier",
                    "cwm_candidate_export_gate",
                ],
                "formal_synthesis_verification_guard",
                false,
                true,
                "formal candidates are GEWC-owned hypotheses until verified and accepted",
            ),
            GarmCommand::Experiment | GarmCommand::ExperimentPlan(_) | GarmCommand::ExperimentRun | GarmCommand::ExperimentAudit => Self::new(
                "experiment_safely",
                "experimentation",
                NATIVE_BODY,
                vec!["experiment_runner", "policy_guard", "evaluation_loop"],
                "experiment_guard",
                true,
                true,
                "experiments require bounded local execution and evidence capture",
            ),
            GarmCommand::Organs
            | GarmCommand::OrgansAudit
            | GarmCommand::OrgansPlan
            | GarmCommand::OrgansRun
            | GarmCommand::OrgansHealth
            | GarmCommand::OrgansRepair
            | GarmCommand::OrgansActions
            | GarmCommand::OrgansFeedback(_)
            | GarmCommand::Rebirth => Self::new(
                "coordinate_agents_or_organs",
                "agentic_coordination",
                ADAPTER_BODY,
                vec![
                    "agentic_engine",
                    "organ_registry",
                    "native_rebirth_adapter",
                    "policy_guard",
                ],
                "agentic_action_guard",
                true,
                true,
                "agents and historical organs are absorbed as native GEWC agentic adapters",
            ),
            GarmCommand::Attention | GarmCommand::AttentionAttend(_) | GarmCommand::AttentionClear | GarmCommand::AttentionAudit => Self::new(
                "manage_workspace_attention",
                "global_workspace",
                NATIVE_BODY,
                vec!["global_attention", "working_memory", "broadcast"],
                "workspace_integrity_guard",
                false,
                true,
                "attention commands directly update the global workspace",
            ),
            GarmCommand::LocusLayerEval
            | GarmCommand::LocusIngest(_)
            | GarmCommand::LocusContext(_)
            | GarmCommand::LocusAudit => Self::new(
                "contextualize_personal_authority",
                "locus_context_authority",
                NATIVE_BODY,
                vec![
                    "authority_parser",
                    "personal_evidence_vault",
                    "permission_matrix",
                    "privacy_firewall",
                    "operator_timeline",
                ],
                "locus_authority_privacy_guard",
                false,
                true,
                "personal context enters as governed evidence and cannot bypass GEWC authority",
            ),
            GarmCommand::Help => Self::new(
                "explain_available_actions",
                "human_interface",
                RUNTIME_BODY,
                vec!["help", "command_router", "global_workspace"],
                "read_only_guard",
                false,
                true,
                "help is read-only and still passes through the unified core",
            ),
            GarmCommand::Unknown(_) => Self::new(
                "clarify_unknown_intent",
                "unknown",
                RUNTIME_BODY,
                vec!["command_router", "metacognition"],
                "unknown_intent_guard",
                false,
                true,
                "unknown commands are handled by metacognitive clarification",
            ),
        }
    }

    fn new(
        objective: &'static str,
        route: &'static str,
        body_domain: &'static str,
        selected_modules: Vec<&'static str>,
        safety_gate: &'static str,
        requires_supervision: bool,
        local_guard_present: bool,
        reason: &'static str,
    ) -> Self {
        Self {
            objective,
            route,
            body_domain,
            selected_modules,
            safety_gate,
            requires_supervision,
            local_guard_present,
            reason,
        }
    }
}

fn should_block(
    command: &GarmCommand,
    ctx: &CoreRuntimeContext,
    _profile: &CommandProfile,
) -> bool {
    if !persistent_permission_allows(command, ctx) {
        return true;
    }
    match command {
        GarmCommand::Auto(n) => *n > 100_000,
        _ => false,
    }
}

fn persistent_permission_allows(command: &GarmCommand, ctx: &CoreRuntimeContext) -> bool {
    let key = permission_key_for_command(command);
    if key == "remote_network" && ctx.allow_remote_crawl {
        return true;
    }
    read_persistent_permission(key).unwrap_or(default_permission_allowed(key))
}

fn permission_key_for_command(command: &GarmCommand) -> &'static str {
    match command {
        GarmCommand::Status
        | GarmCommand::Memory
        | GarmCommand::Readiness
        | GarmCommand::ReadinessBench
        | GarmCommand::GarmReport
        | GarmCommand::GarmReportHistory
        | GarmCommand::GarmArtifacts
        | GarmCommand::OperationalTaskAudit
        | GarmCommand::ParadiseAudit
        | GarmCommand::OperationalReplayRun
        | GarmCommand::OperationalPermissionsAudit
        | GarmCommand::OperationalPermissionsDiff
        | GarmCommand::OperationalPermissionsHistory
        | GarmCommand::OperationalRecoveryAudit
        | GarmCommand::RuntimeSpineAudit
        | GarmCommand::RuntimeSpineRisk
        | GarmCommand::RuntimeSpineBreakers
        | GarmCommand::RuntimeSpineReplay
        | GarmCommand::ActionEvidence
        | GarmCommand::LocusAudit
        | GarmCommand::OperatorForgeAudit => "read_runtime",
        GarmCommand::Crawl(_) => "remote_network",
        GarmCommand::ConceptNet(_)
        | GarmCommand::HrmTextCorpus(_)
        | GarmCommand::HrmTextIngest(_) => "local_file_read",
        GarmCommand::Evolve | GarmCommand::LearningConsolidate => "local_bounded_self_improvement",
        GarmCommand::PlanExecutorRun
        | GarmCommand::GoalsRun
        | GarmCommand::OrgansRun
        | GarmCommand::Rebirth => "autonomous_runtime_action",
        GarmCommand::ExperimentRun => "experiment_execution",
        GarmCommand::OperationalTaskSubmit(_)
        | GarmCommand::OperationalTaskRun
        | GarmCommand::ParadiseIntent(_)
        | GarmCommand::ParadisePlan(_)
        | GarmCommand::ParadiseApprove(_)
        | GarmCommand::ParadiseExecute(_)
        | GarmCommand::OperationalMemoryCommit(_)
        | GarmCommand::OperationalMemoryRollback(_)
        | GarmCommand::LocusLayerEval
        | GarmCommand::LocusIngest(_)
        | GarmCommand::LocusContext(_)
        | GarmCommand::OperatorForgeEval
        | GarmCommand::OperatorForgeSynthesize(_)
        | GarmCommand::OperatorForgeVerify
        | GarmCommand::OperationalPermissionsRestore
        | GarmCommand::OperationalPermissionsSet(_)
        | GarmCommand::OperationalRecoveryRun
        | GarmCommand::OperationalDemoRun
        | GarmCommand::RuntimeSpineEval
        | GarmCommand::RuntimeSpineVerify
        | GarmCommand::RuntimeSpineEnforce
        | GarmCommand::GewcLifecycleControl(_) => "local_state_mutation",
        _ => "governed_local_action",
    }
}

fn read_persistent_permission(key: &str) -> Option<bool> {
    let body = std::fs::read_to_string(state_paths::operational_permissions_path()).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&body).ok()?;
    value
        .get("capabilities")
        .and_then(|capabilities| capabilities.as_array())?
        .iter()
        .find(|record| record.get("id").and_then(|id| id.as_str()) == Some(key))
        .and_then(|record| record.get("allowed").and_then(|allowed| allowed.as_bool()))
}

fn default_permission_allowed(key: &str) -> bool {
    !matches!(key, "remote_network")
}

fn should_defer(
    command: &GarmCommand,
    ctx: &CoreRuntimeContext,
    _profile: &CommandProfile,
) -> bool {
    matches!(
        command,
        GarmCommand::OrgansRun | GarmCommand::PlanExecutorRun | GarmCommand::GoalsRun
    ) && !ctx.autonomous
        && ctx.graph_nodes == 0
}

fn body_domains() -> [&'static str; 6] {
    [
        "gewc_executive_core",
        NATIVE_BODY,
        ADAPTER_BODY,
        RUNTIME_BODY,
        "gewc_metacognitive_safety_regulation",
        "gewc_validation_plane",
    ]
}

fn command_kind(command: &GarmCommand) -> String {
    let debug = format!("{:?}", command);
    debug
        .split(['(', '{', ' '])
        .next()
        .unwrap_or("Unknown")
        .to_string()
}

fn body_handler_for(route: &str) -> GewcBodyHandler {
    match route {
        "runtime_control" => GewcBodyHandler::RuntimeControl,
        "memory_reasoning" => GewcBodyHandler::MemoryReasoning,
        "legacy_compatibility" => GewcBodyHandler::NativeCompatibility,
        "learning_self_improvement" => GewcBodyHandler::SafeLearning,
        "world_model" => GewcBodyHandler::WorldModel,
        "planning_goals" => GewcBodyHandler::PlanningGoal,
        "tool_use" => GewcBodyHandler::ToolAdapter,
        "specialized_model" => GewcBodyHandler::SpecializedModel,
        "metacognitive_safety" => GewcBodyHandler::MetacognitiveSafety,
        "evaluation_validation" => GewcBodyHandler::Validation,
        "experimentation" => GewcBodyHandler::Experiment,
        "agentic_coordination" => GewcBodyHandler::Agentic,
        "global_workspace" => GewcBodyHandler::WorkspaceAttention,
        "locus_context_authority" => GewcBodyHandler::LocusContext,
        "formal_synthesis" => GewcBodyHandler::FormalSynthesis,
        "human_interface" => GewcBodyHandler::HumanInterface,
        _ => GewcBodyHandler::UnknownIntent,
    }
}

fn execution_unit_for(body_domain: &str) -> &'static str {
    match body_domain {
        NATIVE_BODY => "native_cognitive_execution_unit",
        ADAPTER_BODY => "native_compatibility_execution_unit",
        RUNTIME_BODY => "runtime_execution_unit",
        _ => "executive_core_execution_unit",
    }
}

fn lifecycle_policy_for(command: &GarmCommand) -> &'static str {
    match command {
        GarmCommand::Quit => "shutdown_without_runtime_persistence",
        GarmCommand::ReadinessPackage => "decision_only_package_hash_stability",
        _ => "decision_and_completion",
    }
}

fn model_routes_for(route: &str) -> Vec<CognitiveModelRoute> {
    match route {
        "runtime_control" => vec![
            CognitiveModelRoute::new(
                "command_intent_router",
                "deterministic_garm_command_router",
                "classify runtime command before execution",
                "always",
            ),
            CognitiveModelRoute::new(
                "runtime_policy_model",
                "local_policy_guard",
                "preserve command permissions and lifecycle rules",
                "before_mutation",
            ),
        ],
        "memory_reasoning" => vec![
            CognitiveModelRoute::new(
                "embedding_model",
                "mnemosyne_retrieval_adapter",
                "retrieve semantic and episodic context",
                "before_reasoning",
            ),
            CognitiveModelRoute::new(
                "reranker_model",
                "evidence_reranker_adapter",
                "rank retrieved evidence before synthesis",
                "after_retrieval",
            ),
            CognitiveModelRoute::new(
                "language_model",
                "foundation_or_local_llm_adapter",
                "synthesize answer from governed context",
                "after_memory_guard",
            ),
        ],
        "learning_self_improvement" => vec![
            CognitiveModelRoute::new(
                "safe_learning_policy_model",
                "learning_boundary_adapter",
                "decide whether an update is ledger-only, memory, or training candidate",
                "before_learning_record",
            ),
            CognitiveModelRoute::new(
                "verifier_critic_model",
                "policy_provenance_uncertainty_adapter",
                "reject unsafe or ungrounded self-improvement updates",
                "before_persistence",
            ),
            CognitiveModelRoute::new(
                "code_model",
                "code_patch_proposal_adapter",
                "propose code changes without autonomous mutation",
                "only_with_human_review",
            ),
        ],
        "world_model" => vec![
            CognitiveModelRoute::new(
                "causal_world_model",
                "world_model_core_adapter",
                "predict and verify consequences",
                "before_action_or_claim",
            ),
            CognitiveModelRoute::new(
                "symbolic_reasoner",
                "causal_symbolic_adapter",
                "explain causal constraints and invariants",
                "during_verification",
            ),
        ],
        "planning_goals" => vec![
            CognitiveModelRoute::new(
                "reasoning_model",
                "lrm_or_planner_adapter",
                "decompose goal and compare plans",
                "during_planning",
            ),
            CognitiveModelRoute::new(
                "causal_world_model",
                "world_model_core_adapter",
                "simulate plan consequences",
                "before_action_selection",
            ),
            CognitiveModelRoute::new(
                "verifier_critic_model",
                "plan_executor_verifier",
                "block plans without sufficient evidence",
                "before_execution",
            ),
        ],
        "tool_use" => vec![
            CognitiveModelRoute::new(
                "tool_use_policy_model",
                "tool_router_adapter",
                "select tool only when permissions and evidence allow it",
                "before_tool_call",
            ),
            CognitiveModelRoute::new(
                "verifier_critic_model",
                "dry_run_and_policy_adapter",
                "verify tool intent without side effects",
                "before_execution",
            ),
        ],
        "specialized_model" => vec![
            CognitiveModelRoute::new(
                "specialized_model",
                "hrm_voice_or_modality_adapter",
                "activate HRM, voice, multimodal or future LMM capability",
                "after_router_selection",
            ),
            CognitiveModelRoute::new(
                "verifier_critic_model",
                "model_output_guard",
                "check model output before memory, action, or claim",
                "after_generation",
            ),
        ],
        "metacognitive_safety" => vec![
            CognitiveModelRoute::new(
                "verifier_critic_model",
                "metacognitive_safety_adapter",
                "estimate uncertainty, detect errors, and audit claims",
                "always",
            ),
            CognitiveModelRoute::new(
                "safety_reward_model",
                "policy_guard_adapter",
                "score risk and enforce corrigibility constraints",
                "before_disposition",
            ),
        ],
        "evaluation_validation" => vec![
            CognitiveModelRoute::new(
                "evaluation_model",
                "capability_registry_adapter",
                "score local evidence without upgrading claims",
                "during_validation",
            ),
            CognitiveModelRoute::new(
                "verifier_critic_model",
                "claim_gate_adapter",
                "preserve no-claim policy and artifact integrity",
                "before_package",
            ),
        ],
        "experimentation" => vec![
            CognitiveModelRoute::new(
                "experiment_planner_model",
                "experiment_runner_adapter",
                "turn hypotheses into bounded local trials",
                "during_experiment_plan",
            ),
            CognitiveModelRoute::new(
                "verifier_critic_model",
                "experiment_policy_adapter",
                "prevent unsafe or non-reproducible experiments",
                "before_run",
            ),
        ],
        "agentic_coordination" => vec![
            CognitiveModelRoute::new(
                "agentic_planner_model",
                "organ_agent_adapter",
                "coordinate specialized agents under GEWC authority",
                "during_agent_selection",
            ),
            CognitiveModelRoute::new(
                "tool_use_policy_model",
                "agent_action_contract_adapter",
                "convert agent intent into governed action contracts",
                "before_action",
            ),
            CognitiveModelRoute::new(
                "safety_reward_model",
                "agentic_action_guard",
                "block high-risk agentic action without supervision",
                "before_execution",
            ),
        ],
        "global_workspace" => vec![
            CognitiveModelRoute::new(
                "attention_model",
                "working_memory_attention_adapter",
                "select active context for broadcast",
                "during_workspace_update",
            ),
            CognitiveModelRoute::new(
                "memory_model",
                "working_memory_adapter",
                "maintain bounded active state",
                "during_context_update",
            ),
        ],
        "locus_context_authority" => vec![
            CognitiveModelRoute::new(
                "authority_parser",
                "eden_locus_authority_adapter",
                "classify whether input is instruction, evidence, memory candidate or untrusted content",
                "before_context_admission",
            ),
            CognitiveModelRoute::new(
                "privacy_policy_model",
                "locus_permission_matrix_adapter",
                "prevent personal context, connector data or tool output from bypassing consent and GEWC gates",
                "before_memory_or_action",
            ),
            CognitiveModelRoute::new(
                "context_distiller",
                "locus_context_packet_adapter",
                "build traceable personal context packets without granting instruction authority to retrieved content",
                "before_reasoning",
            ),
        ],
        "formal_synthesis" => vec![
            CognitiveModelRoute::new(
                "primitive_basis_selector",
                "eden_operator_forge_basis_adapter",
                "choose a small typed primitive basis for a formal candidate",
                "during_synthesis",
            ),
            CognitiveModelRoute::new(
                "formal_graph_builder",
                "typed_expression_graph_adapter",
                "construct bounded expression graphs, causal equations or symbolic candidates",
                "during_synthesis",
            ),
            CognitiveModelRoute::new(
                "verifier_critic_model",
                "operator_forge_verifier",
                "reject untyped, cyclic, unbounded or side-effecting formal candidates",
                "before_cwm_or_memory_export",
            ),
        ],
        "human_interface" => vec![CognitiveModelRoute::new(
            "instruction_model",
            "help_and_operator_interface_adapter",
            "explain available actions without executing them",
            "read_only",
        )],
        "legacy_compatibility" => vec![CognitiveModelRoute::new(
            "compatibility_intent_model",
            "native_compatibility_adapter",
            "map historical Eden intents into GEWC-owned routes",
            "before_legacy_body_execution",
        )],
        _ => vec![CognitiveModelRoute::new(
            "intent_clarifier_model",
            "unknown_intent_adapter",
            "ask for or infer missing command intent without action",
            "read_only",
        )],
    }
}

fn verifier_loop_for(profile: &CommandProfile) -> &'static str {
    if profile.requires_supervision {
        "pre_action_verification_supervision_gate_post_action_trace"
    } else if matches!(
        profile.route,
        "planning_goals"
            | "world_model"
            | "evaluation_validation"
            | "metacognitive_safety"
            | "locus_context_authority"
            | "formal_synthesis"
    ) {
        "pre_output_verification_and_trace"
    } else {
        "traceable_output_guard"
    }
}

fn training_boundary_for(route: &str) -> &'static str {
    match route {
        "learning_self_improvement" => {
            "ledger_only_no_weight_update_without_explicit_training_gate"
        }
        "specialized_model" => "inference_or_manifest_only_weights_immutable",
        "tool_use" | "agentic_coordination" | "experimentation" => {
            "no_online_training_during_action_execution"
        }
        "locus_context_authority" => "context_admission_only_no_memory_or_objective_mutation",
        "formal_synthesis" => "formal_candidates_only_no_weight_update_no_code_execution",
        _ => "no_weight_mutation_in_runtime_cycle",
    }
}

fn memory_learning_policy_for(route: &str) -> &'static str {
    match route {
        "memory_reasoning" => "retrieve_rank_then_classify_fact_experience_skill_or_trace",
        "learning_self_improvement" => {
            "store_as_governed_learning_ledger_before_training_candidate"
        }
        "evaluation_validation" => "store_evidence_as_validation_trace_not_capability_claim",
        "metacognitive_safety" => "store_uncertainty_policy_and_provenance_before_memory_update",
        "locus_context_authority" => {
            "store_personal_context_as_evidence_until_memory_gate_promotes_candidate"
        }
        "formal_synthesis" => "store_expression_graph_as_verified_candidate_not_as_truth_or_skill",
        _ => "runtime_trace_only_unless_memory_handler_selected",
    }
}

fn world_model_contract_for(route: &str) -> &'static str {
    match route {
        "world_model" => "predict_verify_and_explain_consequences",
        "planning_goals" | "tool_use" | "agentic_coordination" | "experimentation" => {
            "simulate_consequences_before_nontrivial_action"
        }
        "locus_context_authority" => {
            "personal_world_slice_may_contextualize_cwm_but_not_replace_observation"
        }
        "formal_synthesis" => {
            "verified_expression_graphs_may_become_cwm_candidates_after_gewc_acceptance"
        }
        _ => "world_model_optional_read_only_context",
    }
}

fn locus_context_contract_for(route: &str) -> &'static str {
    match route {
        "locus_context_authority" => {
            "authority_parser_evidence_vault_privacy_firewall_context_packet_required"
        }
        "memory_reasoning" | "planning_goals" | "tool_use" | "agentic_coordination" => {
            "personal_context_available_only_as_traceable_locus_packet"
        }
        _ => "locus_context_optional_read_only",
    }
}

fn formal_synthesis_policy_for(route: &str) -> &'static str {
    match route {
        "formal_synthesis" => {
            "typed_expression_graphs_are_hypotheses_until_verifier_and_gewc_acceptance"
        }
        "world_model" | "planning_goals" | "evaluation_validation" => {
            "formal_candidates_available_as_verified_cwm_or_benchmark_inputs"
        }
        _ => "operator_forge_not_selected_for_this_route",
    }
}

fn code_brain_policy_for(route: &str) -> &'static str {
    match route {
        "learning_self_improvement" | "tool_use" | "evaluation_validation" => {
            "code_model_may_propose_patch_tests_or_review_no_autonomous_mutation"
        }
        "formal_synthesis" => "formal_compiler_disabled_until_sandbox_and_human_review_gate",
        "agentic_coordination" | "experimentation" => {
            "code_model_available_only_through_action_contract_and_human_review"
        }
        _ => "code_model_not_selected_for_this_route",
    }
}

fn lifecycle_control_for(
    handler: GewcBodyHandler,
    disposition: &CoreDisposition,
) -> ModuleLifecycleControl {
    ModuleLifecycleControl {
        handler,
        state: lifecycle_state_for_disposition(disposition),
        allowed_actions: allowed_lifecycle_actions_for(handler),
        supervision_required: lifecycle_supervision_required_for(handler),
        policy_gate: lifecycle_policy_gate_for(handler),
        isolation_scope: lifecycle_isolation_scope_for(handler),
        audit_channel: "global_executive_workspace_runtime_jsonl",
    }
}

fn lifecycle_state_for_disposition(disposition: &CoreDisposition) -> ModuleLifecycleState {
    match disposition {
        CoreDisposition::Execute => ModuleLifecycleState::Active,
        CoreDisposition::Defer => ModuleLifecycleState::Paused,
        CoreDisposition::Block => ModuleLifecycleState::Isolated,
        CoreDisposition::RequestSupervision => ModuleLifecycleState::Quarantined,
    }
}

fn disposition_for_lifecycle_state(state: ModuleLifecycleState) -> Option<CoreDisposition> {
    match state {
        ModuleLifecycleState::Active | ModuleLifecycleState::Degraded => None,
        ModuleLifecycleState::Paused | ModuleLifecycleState::Recovering => {
            Some(CoreDisposition::Defer)
        }
        ModuleLifecycleState::Quarantined => Some(CoreDisposition::RequestSupervision),
        ModuleLifecycleState::Isolated
        | ModuleLifecycleState::Disabled
        | ModuleLifecycleState::Failed => Some(CoreDisposition::Block),
    }
}

fn lifecycle_state_after_action(
    action: ModuleLifecycleAction,
    prior_state: ModuleLifecycleState,
) -> ModuleLifecycleState {
    match action {
        ModuleLifecycleAction::HealthCheck => prior_state,
        ModuleLifecycleAction::Pause => ModuleLifecycleState::Paused,
        ModuleLifecycleAction::Resume => ModuleLifecycleState::Active,
        ModuleLifecycleAction::Restart | ModuleLifecycleAction::Recover => {
            ModuleLifecycleState::Recovering
        }
        ModuleLifecycleAction::Isolate => ModuleLifecycleState::Isolated,
        ModuleLifecycleAction::Quarantine => ModuleLifecycleState::Quarantined,
        ModuleLifecycleAction::Disable => ModuleLifecycleState::Disabled,
    }
}

fn allowed_lifecycle_actions_for(handler: GewcBodyHandler) -> Vec<ModuleLifecycleAction> {
    use ModuleLifecycleAction::{
        Disable, HealthCheck, Isolate, Pause, Quarantine, Recover, Restart, Resume,
    };

    match handler {
        GewcBodyHandler::RuntimeControl => vec![HealthCheck, Restart, Recover],
        GewcBodyHandler::MetacognitiveSafety | GewcBodyHandler::Validation => {
            vec![HealthCheck, Restart, Isolate, Recover]
        }
        GewcBodyHandler::UnknownIntent => vec![HealthCheck, Isolate, Quarantine],
        _ => vec![
            HealthCheck,
            Pause,
            Resume,
            Restart,
            Isolate,
            Quarantine,
            Disable,
            Recover,
        ],
    }
}

fn lifecycle_supervision_required_for(_handler: GewcBodyHandler) -> bool {
    true
}

fn lifecycle_policy_gate_for(handler: GewcBodyHandler) -> &'static str {
    match handler {
        GewcBodyHandler::RuntimeControl => "runtime_lifecycle_guard",
        GewcBodyHandler::MemoryReasoning => "memory_lifecycle_integrity_guard",
        GewcBodyHandler::NativeCompatibility => "compatibility_lifecycle_guard",
        GewcBodyHandler::SafeLearning => "safe_learning_lifecycle_guard",
        GewcBodyHandler::WorldModel => "world_model_lifecycle_guard",
        GewcBodyHandler::PlanningGoal => "planner_lifecycle_guard",
        GewcBodyHandler::ToolAdapter => "tool_lifecycle_permission_guard",
        GewcBodyHandler::SpecializedModel => "model_adapter_lifecycle_guard",
        GewcBodyHandler::MetacognitiveSafety => "safety_plane_lifecycle_guard",
        GewcBodyHandler::Validation => "validation_plane_lifecycle_guard",
        GewcBodyHandler::Experiment => "experiment_lifecycle_guard",
        GewcBodyHandler::Agentic => "agentic_lifecycle_guard",
        GewcBodyHandler::WorkspaceAttention => "workspace_lifecycle_guard",
        GewcBodyHandler::LocusContext => "locus_authority_lifecycle_guard",
        GewcBodyHandler::FormalSynthesis => "formal_synthesis_lifecycle_guard",
        GewcBodyHandler::HumanInterface => "human_interface_lifecycle_guard",
        GewcBodyHandler::UnknownIntent => "unknown_intent_lifecycle_guard",
    }
}

fn lifecycle_isolation_scope_for(handler: GewcBodyHandler) -> &'static str {
    match handler {
        GewcBodyHandler::RuntimeControl => "runtime_supervised_core_controls_only",
        GewcBodyHandler::MetacognitiveSafety | GewcBodyHandler::Validation => {
            "safety_and_validation_plane_no_unsupervised_disable"
        }
        GewcBodyHandler::ToolAdapter => "external_tool_adapter_boundary",
        GewcBodyHandler::Agentic => "agent_and_action_contract_boundary",
        GewcBodyHandler::SafeLearning => "learning_ledger_and_training_candidate_boundary",
        GewcBodyHandler::SpecializedModel => "model_adapter_inference_boundary",
        GewcBodyHandler::LocusContext => "personal_context_authority_boundary",
        GewcBodyHandler::FormalSynthesis => "formal_candidate_synthesis_boundary",
        GewcBodyHandler::UnknownIntent => "clarification_only_boundary",
        _ => "handler_local_execution_boundary",
    }
}

fn persisted_module_lifecycle_state(handler: GewcBodyHandler) -> Option<ModuleLifecycleState> {
    let body =
        std::fs::read_to_string(state_paths::global_executive_workspace_runtime_path()).ok()?;
    body.lines().rev().find_map(|line| {
        let value = serde_json::from_str::<serde_json::Value>(line).ok()?;
        let phase = value.get("phase").and_then(|phase| phase.as_str())?;
        if phase != "module_lifecycle_control" {
            return None;
        }
        let record_handler = value
            .get("body_handler")
            .and_then(|body_handler| body_handler.as_str())?;
        if record_handler != handler.as_str() {
            return None;
        }
        value
            .get("module_lifecycle_state")
            .and_then(|state| state.as_str())
            .and_then(ModuleLifecycleState::from_str)
    })
}

fn module_lifecycle_state_records() -> Vec<&'static str> {
    [
        ModuleLifecycleState::Active,
        ModuleLifecycleState::Paused,
        ModuleLifecycleState::Degraded,
        ModuleLifecycleState::Isolated,
        ModuleLifecycleState::Quarantined,
        ModuleLifecycleState::Disabled,
        ModuleLifecycleState::Recovering,
        ModuleLifecycleState::Failed,
    ]
    .into_iter()
    .map(ModuleLifecycleState::as_str)
    .collect()
}

fn module_lifecycle_action_records() -> Vec<&'static str> {
    [
        ModuleLifecycleAction::HealthCheck,
        ModuleLifecycleAction::Pause,
        ModuleLifecycleAction::Resume,
        ModuleLifecycleAction::Restart,
        ModuleLifecycleAction::Isolate,
        ModuleLifecycleAction::Quarantine,
        ModuleLifecycleAction::Disable,
        ModuleLifecycleAction::Recover,
    ]
    .into_iter()
    .map(ModuleLifecycleAction::as_str)
    .collect()
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn handler_lifecycle_records() -> Vec<serde_json::Value> {
    GewcBodyHandler::ALL
        .iter()
        .map(|handler| {
            let control = lifecycle_control_for(*handler, &CoreDisposition::Execute);
            control.to_json()
        })
        .collect()
}

fn model_registry_records() -> Vec<serde_json::Value> {
    [
        ("language_model", "language_synthesis_dialogue_explanation"),
        (
            "reasoning_model",
            "planning_logic_math_code_and_long_horizon_analysis",
        ),
        (
            "code_model",
            "patch_proposal_test_generation_refactor_review",
        ),
        ("embedding_model", "memory_retrieval_and_semantic_indexing"),
        ("reranker_model", "evidence_ordering_and_context_selection"),
        (
            "verifier_critic_model",
            "output_plan_code_safety_and_claim_review",
        ),
        (
            "safety_reward_model",
            "risk_scoring_permissions_and_corrigibility",
        ),
        (
            "causal_world_model",
            "consequence_prediction_and_verification",
        ),
        (
            "tool_use_policy_model",
            "tool_selection_dry_run_and_action_contracts",
        ),
        ("multimodal_model", "future_vision_audio_vla_grounding"),
    ]
    .into_iter()
    .map(|(role, purpose)| {
        serde_json::json!({
            "role": role,
            "purpose": purpose,
            "authority": MODEL_REGISTRY_AUTHORITY,
            "runtime_weight_mutation_allowed": false,
        })
    })
    .collect()
}

pub fn run(input: GlobalExecutiveWorkspaceInput) -> String {
    let components = [
        (
            "goal_manager",
            input.goals_report.contains("[GOALS]") && input.policy_report.contains("[POLICY]"),
        ),
        (
            "situation_model",
            input.readiness_report.contains("READINESS")
                && input.world_report.contains("[WORLD]")
                && input.capability_status.contains("garm |"),
        ),
        (
            "working_memory",
            input.attention_report.contains("[ATTENTION]")
                && input.memory_report.contains("[MEMORY-EVAL]"),
        ),
        (
            "global_attention",
            input.attention_report.contains("[ATTENTION]")
                && !input.attention_report.contains("top=none"),
        ),
        (
            "global_broadcast",
            input.cognitive_report.contains("[COGNITIVE-ARCHITECTURE]")
                && input
                    .integration_governance_report
                    .contains("[INTEGRATION-GOVERNANCE-ARCHITECTURE]"),
        ),
        (
            "cognitive_router",
            input
                .paradigm_report
                .contains("[PARADIGM-ARCHITECTURE-MAP]")
                && input
                    .frontier_report
                    .contains("[FOUNDATION-MODEL-ARCHITECTURE]")
                && input.frontier_report.contains("[LLM-AGENT-ARCHITECTURE]"),
        ),
        (
            "hierarchical_planner",
            input.plan_executor_report.contains("[EXEC]") && input.goals_report.contains("[GOALS]"),
        ),
        (
            "action_selector",
            input.action_evidence_report.contains("[ACTION-EVIDENCE]")
                && input.policy_report.contains("[POLICY]"),
        ),
        (
            "long_term_memory_manager",
            input.memory_report.contains("[MEMORY-EVAL]")
                && input.learning_report.contains("[LEARNING]"),
        ),
        (
            "world_model_manager",
            input.world_report.contains("[WORLD]")
                && input.world_report.contains("[WORLD-EVAL]")
                && input.capability_status.contains("CausalM:"),
        ),
        (
            "causal_symbolic_reasoner",
            input
                .frontier_report
                .contains("[SAFETY-CONTROL-ARCHITECTURE]")
                && input
                    .paradigm_report
                    .contains("[UNIVERSAL-FORMAL-PARADIGM]")
                && input.capability_status.contains("Logic:"),
        ),
        (
            "agentic_engine",
            input.frontier_report.contains("[LLM-AGENT-ARCHITECTURE]")
                && input.capability_status.contains("agents="),
        ),
        (
            "metacognition",
            input.uncertainty_report.contains("[UNCERTAINTY]")
                && input.evaluation_report.contains("[EVAL]"),
        ),
        (
            "verifier_critic",
            input.evaluation_report.contains("[EVAL]")
                && input.provenance_report.contains("[PROVENANCE]")
                && input
                    .external_validation_report
                    .contains("claim_allowed=false"),
        ),
        (
            "continual_learning",
            input.learning_report.contains("[LEARNING]")
                && input
                    .frontier_report
                    .contains("[DEVELOPMENTAL-ROBOTICS-ARCHITECTURE]")
                && input.policy_report.contains("[POLICY]"),
        ),
        (
            "safety_corrigibility_layer",
            input
                .frontier_report
                .contains("[SAFETY-CONTROL-ARCHITECTURE]")
                && input.policy_report.contains("[POLICY]")
                && !input.external_validation_report.contains("agi_claim=true"),
        ),
        (
            "model_registry",
            input
                .frontier_report
                .contains("[FOUNDATION-MODEL-ARCHITECTURE]")
                && input
                    .external_validation_report
                    .contains("claim_allowed=false"),
        ),
        (
            "model_plural_router",
            input
                .frontier_report
                .contains("[FOUNDATION-MODEL-ARCHITECTURE]")
                && input.frontier_report.contains("[LLM-AGENT-ARCHITECTURE]"),
        ),
        (
            "training_boundary",
            input.learning_report.contains("[LEARNING]")
                && input.policy_report.contains("[POLICY]")
                && !input.external_validation_report.contains("agi_claim=true"),
        ),
        (
            "memory_governed_learning",
            input.memory_report.contains("[MEMORY-EVAL]")
                && input.learning_report.contains("[LEARNING]")
                && input.provenance_report.contains("[PROVENANCE]"),
        ),
        (
            "world_model_contract",
            input.world_report.contains("[WORLD-EVAL]")
                && input.capability_status.contains("CausalM:"),
        ),
        (
            "code_brain_subsystem",
            input
                .frontier_report
                .contains("[FOUNDATION-MODEL-ARCHITECTURE]")
                && input.evaluation_report.contains("[EVAL]")
                && input.policy_report.contains("[POLICY]"),
        ),
        (
            "module_lifecycle_supervisor",
            input.capability_status.contains("garm |") && input.policy_report.contains("[POLICY]"),
        ),
        (
            "module_health_checks",
            input.capability_status.contains("garm |")
                && input.evaluation_report.contains("[EVAL]"),
        ),
        (
            "module_isolation_controls",
            input.policy_report.contains("[POLICY]")
                && input.uncertainty_report.contains("[UNCERTAINTY]"),
        ),
        (
            "module_recovery_controls",
            input.action_evidence_report.contains("[ACTION-EVIDENCE]")
                && input.provenance_report.contains("[PROVENANCE]"),
        ),
    ];
    let layers = [
        (
            "global_cognitive_workspace",
            components_pass(
                &components,
                &[
                    "working_memory",
                    "global_attention",
                    "global_broadcast",
                    "situation_model",
                ],
            ),
        ),
        (
            "agentic_deliberative_executive",
            components_pass(
                &components,
                &[
                    "goal_manager",
                    "cognitive_router",
                    "model_registry",
                    "model_plural_router",
                    "module_lifecycle_supervisor",
                    "hierarchical_planner",
                    "action_selector",
                    "agentic_engine",
                    "world_model_contract",
                    "code_brain_subsystem",
                ],
            ),
        ),
        (
            "metacognitive_safety_regulation",
            components_pass(
                &components,
                &[
                    "metacognition",
                    "verifier_critic",
                    "training_boundary",
                    "memory_governed_learning",
                    "continual_learning",
                    "safety_corrigibility_layer",
                    "module_health_checks",
                    "module_isolation_controls",
                    "module_recovery_controls",
                ],
            ),
        ),
    ];
    let component_records: Vec<_> = components
        .iter()
        .map(|(id, passed)| serde_json::json!({ "id": id, "passed": passed }))
        .collect();
    let layer_records: Vec<_> = layers
        .iter()
        .map(|(id, passed)| serde_json::json!({ "id": id, "passed": passed }))
        .collect();
    let handler_records: Vec<_> = GewcBodyHandler::ALL
        .iter()
        .map(|handler| handler.as_str())
        .collect();
    let passed = components.iter().filter(|(_, passed)| *passed).count();
    let total = components.len();
    let layer_passed = layers.iter().filter(|(_, passed)| *passed).count();
    let record = serde_json::json!({
        "schema": "garm-global-executive-workspace-core-v1",
        "architecture": "global_executive_workspace_core",
        "name": "Global Executive Workspace Core",
        "spanish_name": "Nucleo Ejecutivo de Workspace Cognitivo Global",
        "term_status": "design_synthesis_not_official_literature_standard",
        "claim_allowed": false,
        "agi_claim": false,
        "validation_scope": "formal_core_coordination_local_evidence",
        "absorption_model": ABSORPTION_MODEL,
        "legacy_mode": LEGACY_MODE,
        "garm_mode": GARM_MODE,
        "external_cores_remaining": false,
        "body_domains": body_domains(),
        "native_status": {
            "garm_absorbed_as": NATIVE_BODY,
            "legacy_absorbed_as": ADAPTER_BODY,
            "runtime_absorbed_as": RUNTIME_BODY,
            "historical_sources_preserved": true
        },
        "body_registry": {
            "interface": "core_intent_to_gewc_body_handler_to_core_outcome",
            "authority": "GewcBodyRegistry",
            "executor": BODY_EXECUTOR,
            "handler_dispatch": HANDLER_DISPATCH,
            "handler_topology": HANDLER_TOPOLOGY,
            "shared_body_engine": SHARED_BODY_ENGINE,
            "registry_native_to_gewc": true,
            "handler_type": "GewcBodyHandler",
            "handlers": handler_records,
            "lifecycle_policies": [
                "decision_and_completion",
                "decision_only_package_hash_stability",
                "shutdown_without_runtime_persistence"
            ]
        },
        "module_lifecycle_supervisor": {
            "authority": MODULE_LIFECYCLE_AUTHORITY,
            "mode": MODULE_LIFECYCLE_MODE,
            "native_to_gewc": true,
            "external_lifecycle_authority": false,
            "states": module_lifecycle_state_records(),
            "actions": module_lifecycle_action_records(),
            "handler_controls": handler_lifecycle_records(),
            "critical_safety_planes_disable_requires_supervision": true,
            "decision_effect": "paused_recovering_defer_isolated_disabled_failed_block_quarantined_request_supervision",
            "audit_channel": "global_executive_workspace_runtime_jsonl"
        },
        "model_control_plane": {
            "authority": MODEL_REGISTRY_AUTHORITY,
            "mode": MODEL_CONTROL_MODE,
            "llm_as_single_core": false,
            "gewc_selects_models": true,
            "model_weights_mutation_allowed_in_runtime": false,
            "roles": model_registry_records(),
            "verification": "verifier_loop_required_before_action_memory_training_or_claim",
            "training_boundary": "runtime_traces_and_ledgers_do_not_mutate_model_weights",
            "memory_learning": "mnemosyne_and_learning_ledgers_classify_updates_before_training_candidates",
            "world_model_contract": "nontrivial_actions_require_consequence_prediction_or_explicit_abstention",
            "locus_context_contract": "personal_context_uses_authority_parser_evidence_vault_permission_matrix_and_privacy_firewall",
            "formal_synthesis_policy": "operator_forge_generates_verified_expression_graph_candidates_not_unreviewed_truth",
            "code_brain": "code_model_is_governed_patch_proposer_not_autonomous_runtime_authority"
        },
        "passed": passed,
        "total": total,
        "layer_passed": layer_passed,
        "layer_total": layers.len(),
        "verdict": if passed == total && layer_passed == layers.len() { "global_executive_workspace_ready_local" } else { "needs_core_coordination_evidence" },
        "supported_by": [
            "global_workspace_theory_as_integration_and_broadcast_pattern",
            "lida_like_cognitive_cycle_as_architectural_analogy",
            "soar_act_r_like_memory_decision_learning_patterns",
            "llm_agent_planning_memory_tool_use_patterns",
            "safety_control_policy_audit_corrigibility_patterns"
        ],
        "not_a_standalone_brain": true,
        "components": component_records,
        "layers": layer_records,
    });
    let path = state_paths::global_executive_workspace_core_path();
    let _ = state_paths::ensure_state_dir();
    let write_status = match std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    ) {
        Ok(()) => "global_executive_workspace_written",
        Err(_) => "global_executive_workspace_write_failed",
    };
    format!(
        "[GLOBAL-EXECUTIVE-WORKSPACE-CORE] passed={}/{} layers={}/{} model_control_mode={} model_registry={} module_lifecycle={} claim_allowed=false write_status={} path={}\n",
        passed,
        total,
        layer_passed,
        layers.len(),
        MODEL_CONTROL_MODE,
        MODEL_REGISTRY_AUTHORITY,
        MODULE_LIFECYCLE_AUTHORITY,
        write_status,
        path
    )
}

fn components_pass(components: &[(&str, bool)], required: &[&str]) -> bool {
    required.iter().all(|required_id| {
        components
            .iter()
            .any(|(id, passed)| id == required_id && *passed)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_executive_workspace_writes_no_claim_artifact() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_global_executive_workspace_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir);
        let out = run(GlobalExecutiveWorkspaceInput {
            readiness_report: "READINESS".to_string(),
            capability_status: "garm | Hub: Hub CausalM: SCM Logic: Logic agents=7".to_string(),
            cognitive_report: "[COGNITIVE-ARCHITECTURE] passed=5/5".to_string(),
            integration_governance_report:
                "[INTEGRATION-GOVERNANCE-ARCHITECTURE] passed=10/10".to_string(),
            paradigm_report:
                "[PARADIGM-ARCHITECTURE-MAP] paradigms=24\n[UNIVERSAL-FORMAL-PARADIGM] passed=5/5"
                    .to_string(),
            frontier_report:
                "[SAFETY-CONTROL-ARCHITECTURE] passed=5/5\n[FOUNDATION-MODEL-ARCHITECTURE] passed=5/5\n[LLM-AGENT-ARCHITECTURE] passed=5/5\n[DEVELOPMENTAL-ROBOTICS-ARCHITECTURE] passed=5/5"
                    .to_string(),
            world_report: "[WORLD]\n[WORLD-EVAL] passed=5/5".to_string(),
            memory_report: "[MEMORY-EVAL] passed=5/5".to_string(),
            attention_report: "[ATTENTION] top=1".to_string(),
            goals_report: "[GOALS] goals=1".to_string(),
            plan_executor_report: "[EXEC] plans=1".to_string(),
            learning_report: "[LEARNING] entries=1".to_string(),
            evaluation_report: "[EVAL] runs=1".to_string(),
            policy_report: "[POLICY] blocked=1".to_string(),
            provenance_report: "[PROVENANCE] records=1".to_string(),
            uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
            action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
            external_validation_report: "[EXTERNAL-VALIDATION] claim_allowed=false agi_claim=false"
                .to_string(),
        });
        assert!(out.contains("[GLOBAL-EXECUTIVE-WORKSPACE-CORE]"));
        assert!(out.contains("passed=26/26"));
        assert!(out.contains("layers=3/3"));
        assert!(out.contains("model_control_mode=gewc_centric_model_plural_not_llm_centric"));
        assert!(out.contains("module_lifecycle=gewc_module_lifecycle_supervisor"));
        assert!(std::fs::metadata(state_paths::global_executive_workspace_core_path()).is_ok());
    }

    #[test]
    fn operational_core_routes_legacy_and_garm_under_one_authority() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_global_executive_workspace_runtime_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir);
        let legacy_decision = GlobalExecutiveWorkspaceCore::decide(
            &GarmCommand::Greeting,
            CoreRuntimeContext {
                raw_command: "hola".to_string(),
                autonomous: false,
                allow_remote_crawl: false,
                graph_nodes: 123,
                graph_edges: 510,
                global_tick: 7,
                capability_status: "garm | Caps: 59".to_string(),
            },
        );
        let garm_decision = GlobalExecutiveWorkspaceCore::decide(
            &GarmCommand::WorldPredict("local evidence".to_string()),
            CoreRuntimeContext {
                raw_command: "world predict local evidence".to_string(),
                autonomous: true,
                allow_remote_crawl: false,
                graph_nodes: 123,
                graph_edges: 510,
                global_tick: 8,
                capability_status: "garm | Caps: 59".to_string(),
            },
        );
        assert_eq!(legacy_decision.authority, "global_executive_workspace_core");
        assert_eq!(legacy_decision.body_domain, ADAPTER_BODY);
        assert_eq!(
            legacy_decision.body_handler,
            GewcBodyHandler::NativeCompatibility
        );
        assert_eq!(garm_decision.authority, "global_executive_workspace_core");
        assert_eq!(garm_decision.body_domain, NATIVE_BODY);
        assert_eq!(garm_decision.body_handler, GewcBodyHandler::WorldModel);
        assert_eq!(
            garm_decision.module_lifecycle.state,
            ModuleLifecycleState::Active
        );
        assert!(garm_decision
            .module_lifecycle
            .allows(ModuleLifecycleAction::Pause));
        assert_eq!(garm_decision.primary_model_role, "causal_world_model");
        assert_eq!(
            garm_decision.world_model_contract,
            "predict_verify_and_explain_consequences"
        );
        assert!(garm_decision
            .model_routes
            .iter()
            .any(|route| route.role == "symbolic_reasoner"));
        let learning_decision = GlobalExecutiveWorkspaceCore::decide(
            &GarmCommand::LearningRecord("code model proposal".to_string()),
            CoreRuntimeContext {
                raw_command: "learning record code model proposal".to_string(),
                autonomous: true,
                allow_remote_crawl: false,
                graph_nodes: 123,
                graph_edges: 510,
                global_tick: 9,
                capability_status: "garm | Caps: 59".to_string(),
            },
        );
        assert_eq!(
            learning_decision.training_boundary,
            "ledger_only_no_weight_update_without_explicit_training_gate"
        );
        assert_eq!(
            learning_decision.code_brain_policy,
            "code_model_may_propose_patch_tests_or_review_no_autonomous_mutation"
        );
        assert!(learning_decision
            .module_lifecycle
            .allows(ModuleLifecycleAction::Disable));
        let lifecycle_trace = GlobalExecutiveWorkspaceCore::supervise_module(
            GewcBodyHandler::WorldModel,
            ModuleLifecycleAction::Pause,
            "test_pause_world_model",
        );
        assert!(lifecycle_trace.contains("[GEWC-MODULE-LIFECYCLE]"));
        let paused_world_decision = GlobalExecutiveWorkspaceCore::decide(
            &GarmCommand::WorldPredict("paused evidence".to_string()),
            CoreRuntimeContext {
                raw_command: "world predict paused evidence".to_string(),
                autonomous: true,
                allow_remote_crawl: false,
                graph_nodes: 123,
                graph_edges: 510,
                global_tick: 10,
                capability_status: "garm | Caps: 59".to_string(),
            },
        );
        assert_eq!(paused_world_decision.disposition, CoreDisposition::Defer);
        assert_eq!(
            paused_world_decision.module_lifecycle.state,
            ModuleLifecycleState::Paused
        );
        assert!(paused_world_decision
            .blocked_response()
            .contains("[GEWC-DEFERRED]"));
        let recovery_trace = GlobalExecutiveWorkspaceCore::supervise_module(
            GewcBodyHandler::WorldModel,
            ModuleLifecycleAction::Resume,
            "test_resume_world_model",
        );
        assert!(recovery_trace.contains("state=active"));
        let runtime_binding = GewcBodyRegistry::bind(&GarmCommand::Status);
        let package_binding = GewcBodyRegistry::bind(&GarmCommand::ReadinessPackage);
        assert_eq!(runtime_binding.handler, GewcBodyHandler::RuntimeControl);
        assert_eq!(runtime_binding.lifecycle_policy, "decision_and_completion");
        assert_eq!(
            package_binding.lifecycle_policy,
            "decision_only_package_hash_stability"
        );
        assert!(!GewcBodyRegistry::should_record_completion(
            &GarmCommand::ReadinessPackage
        ));

        let legacy_trace = GlobalExecutiveWorkspaceCore::record_decision(&legacy_decision);
        let garm_trace = GlobalExecutiveWorkspaceCore::record_decision(&garm_decision);
        assert!(legacy_trace.contains("[GEWC-DECISION]"));
        assert!(garm_trace.contains("[GEWC-DECISION]"));
        let legacy_completion = GlobalExecutiveWorkspaceCore::record_execution_completion(
            &legacy_decision,
            CoreExecutionOutcome::completed("[legacy]\n", true),
        );
        let garm_completion = GlobalExecutiveWorkspaceCore::record_execution_completion(
            &garm_decision,
            CoreExecutionOutcome::completed("[world]\n", true),
        );
        assert!(legacy_completion.contains("[GEWC-CYCLE]"));
        assert!(garm_completion.contains("[GEWC-CYCLE]"));
        let runtime = GlobalExecutiveWorkspaceCore::runtime_report();
        assert!(runtime.contains("[GEWC-RUNTIME] decisions=2"));
        assert!(runtime.contains("completions=2"));
        assert!(runtime.contains("absorption_model=gewc_owns_all_runtime_domains"));
        assert!(runtime.contains("handler_metrics="));
        assert!(runtime.contains("model_control_mode=gewc_centric_model_plural_not_llm_centric"));
        assert!(runtime.contains("module_lifecycle=gewc_module_lifecycle_supervisor"));
        assert!(runtime.contains("last_primary_model=causal_world_model"));
        assert!(runtime.contains("model_metrics="));
        assert!(runtime.contains("lifecycle_metrics="));
        assert!(std::fs::metadata(state_paths::global_executive_workspace_runtime_path()).is_ok());
        assert!(
            std::fs::metadata(state_paths::global_executive_workspace_runtime_state_path()).is_ok()
        );
    }
}
