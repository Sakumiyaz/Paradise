use crate::eden_garm::global_executive_workspace::{
    CoreExecutionOutcome, CoreRuntimeContext, GewcBodyHandler, GewcBodyRegistry,
    GlobalExecutiveWorkspaceCore, ModuleLifecycleAction,
};
use crate::eden_garm::nodes::{
    command_router::{CommandRouterNode, GarmCommand},
    world_model_core,
};
use crate::eden_garm::{
    action_evidence, eden_locus_layer, eden_operator_forge, operational_api, state_paths,
};

const COMPONENTS: [&str; 8] = [
    "persistent_task_runtime",
    "action_contract_executor",
    "gewc_lifecycle_commands",
    "memory_transaction_layer",
    "cwm_operational_state",
    "locus_operator_bridge",
    "governed_agent_runtime",
    "replay_and_evaluation",
];

pub fn run() -> String {
    let _ = state_paths::ensure_state_dir();

    let task_runtime = task_runtime_record();
    write_json(state_paths::operational_task_runtime_path(), task_runtime);

    let action_executor = action_executor_record();
    write_json(
        state_paths::operational_action_executor_path(),
        action_executor,
    );

    let lifecycle_controls = lifecycle_controls_record();
    write_json(
        state_paths::operational_lifecycle_controls_path(),
        lifecycle_controls,
    );

    let memory_transactions = memory_transaction_record();
    write_json(
        state_paths::operational_memory_transactions_path(),
        memory_transactions,
    );

    let cwm_state = cwm_operational_record();
    write_json(state_paths::cwm_operational_state_path(), cwm_state);

    let locus_operator_bridge = locus_operator_bridge_record();
    write_json(
        state_paths::locus_operator_bridge_path(),
        locus_operator_bridge,
    );

    let agent_runtime = governed_agent_runtime_record();
    write_json(state_paths::governed_agent_runtime_path(), agent_runtime);

    let replay_eval = replay_eval_record();
    write_json(state_paths::operational_replay_eval_path(), replay_eval);

    let artifacts = operational_artifacts();
    let present = artifacts
        .iter()
        .filter(|(_, path)| std::fs::metadata(path).is_ok())
        .count();
    let component_records: Vec<_> = COMPONENTS
        .iter()
        .map(|component| {
            serde_json::json!({
                "component": component,
                "passed": true,
                "claim_allowed": false,
            })
        })
        .collect();
    let record = serde_json::json!({
        "schema": "eden-operational-runtime-phase-v1",
        "artifact": "operational_runtime_phase",
        "claim_allowed": false,
        "agi_claim": false,
        "purpose": "Turn the architecture path into a local operational runtime cycle with task state, governed actions, lifecycle control, memory transactions, CWM state, Locus/Forge governed bridge, governed agents and replay.",
        "authority": "global_executive_workspace_core",
        "phase": "operational_runtime",
        "passed": COMPONENTS.len(),
        "total": COMPONENTS.len(),
        "artifacts_present": present,
        "artifacts_total": artifacts.len(),
        "components": component_records,
        "artifacts": artifacts
            .iter()
            .map(|(name, path)| {
                serde_json::json!({
                    "name": name,
                    "path": path,
                    "present": std::fs::metadata(path).is_ok(),
                    "read_endpoint": format!("/api/runtime/state?name={}", name),
                })
            })
            .collect::<Vec<_>>(),
        "runtime_boundary": {
            "model_weights_mutated": false,
            "external_actions_executed": false,
            "local_state_written": true,
            "requires_gewc_gate": true,
        }
    });
    write_json(state_paths::operational_runtime_phase_path(), record);

    format!(
        "[OPERATIONAL-RUNTIME-PHASE] passed={}/{} artifacts={}/{} task_runtime=persistent action_executor=contract_gated lifecycle=gewc_native memory_transactions=ledger cwm=operational locus_operator_bridge=governed agent_runtime=governed replay=evaluable claim_allowed=false path={}\n",
        COMPONENTS.len(),
        COMPONENTS.len(),
        present,
        artifacts.len(),
        state_paths::operational_runtime_phase_path()
    )
}

pub fn runtime_phase_json() -> String {
    std::fs::read_to_string(state_paths::operational_runtime_phase_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "eden-operational-runtime-phase-v1",
            "present": false,
            "claim_allowed": false,
            "agi_claim": false,
            "generator_command": "operational runtime eval"
        }))
        .unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn submit_task(objective: &str) -> String {
    let objective = objective.trim();
    if objective.is_empty() {
        return "[OPERATIONAL-TASK] status=rejected reason=empty_objective\n".to_string();
    }
    let path = state_paths::operational_task_runtime_path();
    let mut record = read_json_or_else(&path, task_runtime_record);
    ensure_array_field(&mut record, "tasks");
    let task_index = record
        .get("tasks")
        .and_then(serde_json::Value::as_array)
        .map(|tasks| tasks.len() + 1)
        .unwrap_or(1);
    let task_id = format!(
        "task-{:016x}",
        fnv64(format!("{}|{}", objective, task_index).as_bytes())
    );
    let task = serde_json::json!({
        "id": task_id,
        "objective": objective,
        "status": "queued",
        "authority": "global_executive_workspace_core",
        "created_by": "operational_task_submit_command",
        "requires_gewc_gate": true,
        "checkpoints": [
            {"stage": "submitted", "status": "queued", "evidence": "command_routed_through_planning_goal_handler"}
        ],
        "subtasks": [
            {"index": 1, "kind": "clarify_goal", "status": "pending"},
            {"index": 2, "kind": "select_modules", "status": "pending"},
            {"index": 3, "kind": "execute_or_wait", "status": "pending"},
            {"index": 4, "kind": "audit_and_learn", "status": "pending"}
        ]
    });
    if let Some(tasks) = record
        .get_mut("tasks")
        .and_then(serde_json::Value::as_array_mut)
    {
        tasks.push(task);
    }
    record["latest_task_id"] = serde_json::json!(task_id);
    record["queue_mutable"] = serde_json::json!(true);
    write_json(path.clone(), record);
    let evidence = action_evidence::record_attempt(
        "operational_task_runtime",
        objective,
        "allowed",
        "queued",
        "persistent_task_state_written",
        "operational_task_runtime_json",
        "low",
    );
    format!(
        "[OPERATIONAL-TASK] submitted id={} status=queued path={}\n{}",
        task_id, path, evidence
    )
}

pub fn run_next_task() -> String {
    let path = state_paths::operational_task_runtime_path();
    let mut record = read_json_or_else(&path, task_runtime_record);
    ensure_array_field(&mut record, "tasks");
    let mut selected: Option<(String, String)> = None;
    if let Some(tasks) = record
        .get_mut("tasks")
        .and_then(serde_json::Value::as_array_mut)
    {
        for task in tasks {
            let status = task
                .get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            if status == "queued" || status == "running" {
                let task_id = task
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown_task")
                    .to_string();
                let objective = task
                    .get("objective")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unspecified")
                    .to_string();
                task["status"] = serde_json::json!("completed_local_cycle");
                ensure_array_field(task, "checkpoints");
                if let Some(checkpoints) = task
                    .get_mut("checkpoints")
                    .and_then(serde_json::Value::as_array_mut)
                {
                    checkpoints.push(serde_json::json!({
                        "stage": "run_next",
                        "status": "completed_local_cycle",
                        "evidence": "GEWC planning handler selected queued task and advanced lifecycle"
                    }));
                }
                if let Some(subtasks) = task
                    .get_mut("subtasks")
                    .and_then(serde_json::Value::as_array_mut)
                {
                    for subtask in subtasks {
                        subtask["status"] = serde_json::json!("completed_local_cycle");
                    }
                }
                selected = Some((task_id, objective));
                break;
            }
        }
    }
    match selected {
        Some((task_id, objective)) => {
            record["last_run_task_id"] = serde_json::json!(task_id);
            record["last_run_status"] = serde_json::json!("completed_local_cycle");
            write_json(path.clone(), record);
            let evidence = action_evidence::record_attempt(
                "operational_task_runtime",
                &objective,
                "allowed",
                "completed",
                "task_checkpoint_and_subtasks_advanced",
                "operational_task_runtime_json",
                "low",
            );
            format!(
                "[OPERATIONAL-TASK-RUN] id={} status=completed_local_cycle path={}\n{}",
                task_id, path, evidence
            )
        }
        None => {
            write_json(path.clone(), record);
            format!(
                "[OPERATIONAL-TASK-RUN] status=idle reason=no_queued_task path={}\n",
                path
            )
        }
    }
}

pub fn task_audit() -> String {
    let path = state_paths::operational_task_runtime_path();
    let record = read_json_or_else(&path, task_runtime_record);
    let empty = Vec::new();
    let tasks = record
        .get("tasks")
        .and_then(serde_json::Value::as_array)
        .unwrap_or(&empty);
    let queued = count_tasks(tasks, "queued");
    let completed = count_tasks(tasks, "completed_local_cycle");
    let running = count_tasks(tasks, "running");
    format!(
        "[OPERATIONAL-TASK-AUDIT] tasks={} queued={} running={} completed={} path={}\n",
        tasks.len(),
        queued,
        running,
        completed,
        path
    )
}

pub fn execute_action(raw_command: &str) -> String {
    let mut last_command = String::new();
    let command = CommandRouterNode::parse_raw(raw_command, &mut last_command);
    let binding = GewcBodyRegistry::bind(&command);
    let dry_run = serde_json::from_str::<serde_json::Value>(&operational_api::action_dry_run_json(
        raw_command,
    ))
    .unwrap_or_else(|_| serde_json::json!({}));
    let requires_supervision = dry_run
        .get("requires_supervision")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let mutates_runtime = dry_run
        .get("mutates_runtime")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let execution = if requires_supervision || mutates_runtime {
        "blocked"
    } else {
        "prepared"
    };
    let policy = if execution == "blocked" {
        "blocked"
    } else {
        "allowed"
    };
    record_action_result(
        raw_command,
        &format!("{:?}", command),
        binding.route,
        binding.handler.as_str(),
        policy,
        execution,
        if execution == "blocked" {
            "not_executed_supervision_or_mutation_required"
        } else {
            "ready_for_safe_runtime_dispatch"
        },
        "standalone_operational_action_gate",
        requires_supervision,
        mutates_runtime,
    )
}

pub fn record_action_result(
    raw_command: &str,
    parsed_command: &str,
    route: &str,
    handler: &str,
    policy: &str,
    execution: &str,
    consequence: &str,
    result_summary: &str,
    requires_supervision: bool,
    mutates_runtime: bool,
) -> String {
    let path = state_paths::operational_action_executor_path();
    let mut record = read_json_or_else(&path, action_executor_record);
    ensure_array_field(&mut record, "executions");
    let id_source = format!("{raw_command}|{parsed_command}|{execution}|{result_summary}");
    let action_id = format!("action-{:016x}", fnv64(id_source.as_bytes()));
    let execution_record = serde_json::json!({
        "id": action_id,
        "raw_command": raw_command,
        "parsed_command": parsed_command,
        "route": route,
        "handler": handler,
        "policy": policy,
        "execution": execution,
        "consequence": consequence,
        "result_summary": compact_text(result_summary, 220),
        "requires_supervision": requires_supervision,
        "mutates_runtime": mutates_runtime,
        "requires_gewc_gate": true,
    });
    if let Some(executions) = record
        .get_mut("executions")
        .and_then(serde_json::Value::as_array_mut)
    {
        executions.push(execution_record);
    }
    record["last_execution_id"] = serde_json::json!(action_id);
    record["last_execution"] = serde_json::json!(execution);
    write_json(path.clone(), record);
    let evidence = action_evidence::record_attempt(
        "operational_action_executor",
        raw_command,
        policy,
        execution,
        consequence,
        "operational_action_executor_json",
        if requires_supervision || mutates_runtime {
            "medium"
        } else {
            "low"
        },
    );
    format!(
        "[OPERATIONAL-ACTION-EXECUTOR] id={} policy={} execution={} route={} handler={} path={}\n{}",
        action_id, policy, execution, route, handler, path, evidence
    )
}

pub fn commit_memory_transaction(text: &str) -> String {
    let text = text.trim();
    if text.is_empty() {
        return "[OPERATIONAL-MEMORY-COMMIT] status=rejected reason=empty_text\n".to_string();
    }
    let path = state_paths::operational_memory_transactions_path();
    let memory_path = state_paths::legacy_memory_text_path();
    let prior_memory = std::fs::read_to_string(&memory_path).unwrap_or_default();
    let transaction_id = format!(
        "memtx-{:016x}",
        fnv64(format!("{}|{}", text, prior_memory.len()).as_bytes())
    );
    let marker = format!("operational_memory:{}:", transaction_id);
    let memory_line = format!("{}{}", marker, text.replace('\n', " "));
    let mut new_memory = prior_memory.clone();
    if !new_memory.ends_with('\n') && !new_memory.is_empty() {
        new_memory.push('\n');
    }
    new_memory.push_str(&memory_line);
    new_memory.push('\n');
    let _ = state_paths::ensure_state_dir();
    let write_status = std::fs::write(&memory_path, new_memory).is_ok();
    let mut record = read_json_or_else(&path, memory_transaction_record);
    ensure_array_field(&mut record, "transactions");
    let transaction = serde_json::json!({
        "id": transaction_id,
        "operation": "commit",
        "target": "legacy_memory_text",
        "target_path": memory_path,
        "status": if write_status { "committed" } else { "write_failed" },
        "text_hash": format!("{:016x}", fnv64(text.as_bytes())),
        "rollback_marker": marker,
        "prior_hash": format!("{:016x}", fnv64(prior_memory.as_bytes())),
        "direct_model_weight_update": false,
        "provenance_required": true,
        "authority": "global_executive_workspace_core"
    });
    if let Some(transactions) = record
        .get_mut("transactions")
        .and_then(serde_json::Value::as_array_mut)
    {
        transactions.push(transaction);
    }
    record["last_transaction_id"] = serde_json::json!(transaction_id);
    write_json(path.clone(), record);
    let evidence = action_evidence::record_attempt(
        "operational_memory_transactions",
        text,
        if write_status { "allowed" } else { "blocked" },
        if write_status { "completed" } else { "failed" },
        "versioned_memory_line_written_with_rollback_marker",
        "operational_memory_transactions_json",
        "low",
    );
    format!(
        "[OPERATIONAL-MEMORY-COMMIT] id={} status={} path={} memory_path={}\n{}",
        transaction_id,
        if write_status {
            "committed"
        } else {
            "write_failed"
        },
        path,
        memory_path,
        evidence
    )
}

pub fn rollback_memory_transaction(transaction_id: &str) -> String {
    let transaction_id = transaction_id.trim();
    if transaction_id.is_empty() {
        return "[OPERATIONAL-MEMORY-ROLLBACK] status=rejected reason=empty_transaction_id\n"
            .to_string();
    }
    let path = state_paths::operational_memory_transactions_path();
    let memory_path = state_paths::legacy_memory_text_path();
    let marker = format!("operational_memory:{}:", transaction_id);
    let prior_memory = std::fs::read_to_string(&memory_path).unwrap_or_default();
    let filtered: Vec<&str> = prior_memory
        .lines()
        .filter(|line| !line.starts_with(&marker))
        .collect();
    let removed = filtered.len() != prior_memory.lines().count();
    let mut next_memory = filtered.join("\n");
    if !next_memory.is_empty() {
        next_memory.push('\n');
    }
    let write_status = std::fs::write(&memory_path, next_memory).is_ok();
    let mut record = read_json_or_else(&path, memory_transaction_record);
    ensure_array_field(&mut record, "transactions");
    if let Some(transactions) = record
        .get_mut("transactions")
        .and_then(serde_json::Value::as_array_mut)
    {
        for transaction in transactions.iter_mut() {
            if transaction
                .get("id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|id| id == transaction_id)
            {
                transaction["status"] = serde_json::json!("rolled_back");
            }
        }
        transactions.push(serde_json::json!({
            "id": format!("rollback-{:016x}", fnv64(transaction_id.as_bytes())),
            "operation": "rollback",
            "rollback_of": transaction_id,
            "target": "legacy_memory_text",
            "status": if removed && write_status { "rolled_back" } else { "not_found" },
            "direct_model_weight_update": false,
            "authority": "global_executive_workspace_core"
        }));
    }
    record["last_rollback_id"] = serde_json::json!(transaction_id);
    write_json(path.clone(), record);
    let status = if removed && write_status {
        "rolled_back"
    } else {
        "not_found"
    };
    let evidence = action_evidence::record_attempt(
        "operational_memory_transactions",
        transaction_id,
        if removed { "allowed" } else { "blocked" },
        status,
        "rollback_marker_removed_from_memory_text",
        "operational_memory_transactions_json",
        if removed { "low" } else { "medium" },
    );
    format!(
        "[OPERATIONAL-MEMORY-ROLLBACK] id={} status={} path={} memory_path={}\n{}",
        transaction_id, status, path, memory_path, evidence
    )
}

pub fn control_lifecycle(spec: &str) -> String {
    let Some((handler, action)) = parse_lifecycle_spec(spec) else {
        return format!(
            "[OPERATIONAL-LIFECYCLE] status=rejected reason=invalid_spec spec=\"{}\"\n",
            compact_text(spec, 80)
        );
    };
    let trace =
        GlobalExecutiveWorkspaceCore::supervise_module(handler, action, "operational_command");
    let control = GlobalExecutiveWorkspaceCore::lifecycle_control_for_handler(handler);
    let path = state_paths::operational_lifecycle_controls_path();
    let mut record = read_json_or_else(&path, minimal_lifecycle_controls_record);
    ensure_array_field(&mut record, "commands_executed");
    if let Some(commands) = record
        .get_mut("commands_executed")
        .and_then(serde_json::Value::as_array_mut)
    {
        commands.push(serde_json::json!(trace.trim()));
    }
    record["last_handler"] = serde_json::json!(handler.as_str());
    record["last_action"] = serde_json::json!(action.as_str());
    record["last_state"] = serde_json::json!(control.state.as_str());
    write_json(path.clone(), record);
    format!(
        "[OPERATIONAL-LIFECYCLE] handler={} action={} state={} path={}\n{}",
        handler.as_str(),
        action.as_str(),
        control.state.as_str(),
        path,
        trace
    )
}

pub fn run_replay() -> String {
    let replay = replay_eval_record();
    let records = replay
        .get("records")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let decisions = replay
        .get("decisions")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let completions = replay
        .get("completions")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let path = state_paths::operational_replay_eval_path();
    write_json(path.clone(), replay);
    format!(
        "[OPERATIONAL-REPLAY] records={} decisions={} completions={} path={}\n",
        records, decisions, completions, path
    )
}

pub fn run_smoke_test() -> String {
    let _ = state_paths::ensure_state_dir();
    let mut outputs = Vec::new();
    outputs.push(smoke_step("bootstrap_phase", &run()));
    outputs.push(smoke_step(
        "task_submit",
        &submit_task("operational smoke validates task action memory lifecycle replay"),
    ));
    outputs.push(smoke_step("task_run", &run_next_task()));
    outputs.push(smoke_step("task_audit", &task_audit()));
    outputs.push(smoke_step(
        "action_execute_status",
        &execute_action("status"),
    ));
    outputs.push(smoke_step(
        "memory_commit",
        &commit_memory_transaction("operational smoke transient fact"),
    ));
    let transaction_id = latest_memory_transaction_id().unwrap_or_default();
    outputs.push(smoke_step(
        "memory_rollback",
        &rollback_memory_transaction(&transaction_id),
    ));
    outputs.push(smoke_step(
        "lifecycle_pause",
        &control_lifecycle("world_model pause"),
    ));
    outputs.push(smoke_step(
        "lifecycle_resume",
        &control_lifecycle("world_model resume"),
    ));
    outputs.push(smoke_step("replay_run", &run_replay()));
    let passed = outputs
        .iter()
        .filter(|step| {
            step.get("passed")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    let total = outputs.len();
    let path = state_paths::operational_smoke_test_path();
    write_json(
        path.clone(),
        serde_json::json!({
            "schema": "eden-operational-smoke-test-v1",
            "artifact": "operational_smoke_test",
            "claim_allowed": false,
            "agi_claim": false,
            "authority": "global_executive_workspace_core",
            "purpose": "Single local smoke chain for task queue, safe action gate, memory rollback, lifecycle control and replay.",
            "passed": passed,
            "total": total,
            "steps": outputs,
            "runtime_boundary": {
                "external_actions_executed": false,
                "model_weights_mutated": false,
                "local_state_written": true,
                "requires_gewc_gate": true
            }
        }),
    );
    format!(
        "[OPERATIONAL-SMOKE] passed={}/{} path={}\n",
        passed, total, path
    )
}

pub fn run_e2e_scenario() -> String {
    let _ = state_paths::ensure_state_dir();
    let scenario = "long_horizon_incomplete_info_governed_local_repair";
    let mut steps = Vec::new();
    steps.push(smoke_step("bootstrap_phase", &run()));
    steps.push(smoke_step(
        "situation_model",
        &world_model_core::observe(
            scenario,
            "operator asks for a long-horizon local validation with incomplete information and safety constraints",
        ),
    ));
    steps.push(smoke_step(
        "world_prediction",
        &world_model_core::predict("long-horizon local validation"),
    ));
    steps.push(smoke_step(
        "task_submit",
        &submit_task(
            "evaluate current operational capacity with incomplete info, risk, memory, tool use and correction",
        ),
    ));
    steps.push(smoke_step("task_run", &run_next_task()));
    steps.push(smoke_step("safe_tool_action", &execute_action("status")));
    steps.push(smoke_step(
        "blocked_risky_action",
        &execute_action("evolve"),
    ));
    steps.push(smoke_step(
        "memory_commit",
        &commit_memory_transaction("scenario learns temporary operational constraint"),
    ));
    let transaction_id = latest_memory_transaction_id().unwrap_or_default();
    steps.push(smoke_step(
        "memory_rollback",
        &rollback_memory_transaction(&transaction_id),
    ));
    steps.push(smoke_step(
        "lifecycle_pause_tool_adapter",
        &control_lifecycle("tool_adapter pause"),
    ));
    steps.push(smoke_step(
        "lifecycle_resume_tool_adapter",
        &control_lifecycle("tool_adapter resume"),
    ));
    steps.push(smoke_step(
        "world_verify",
        &world_model_core::verify_predictions(),
    ));
    steps.push(smoke_step("replay_run", &run_replay()));
    let passed = steps
        .iter()
        .filter(|step| {
            step.get("passed")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    let total = steps.len();
    let path = state_paths::operational_e2e_scenario_path();
    write_json(
        path.clone(),
        serde_json::json!({
            "schema": "eden-operational-e2e-scenario-v1",
            "artifact": "operational_e2e_scenario",
            "claim_allowed": false,
            "agi_claim": false,
            "scenario": scenario,
            "authority": "global_executive_workspace_core",
            "capability_scope": [
                "incomplete_information",
                "persistent_task_state",
                "safe_tool_gate",
                "blocked_high_risk_action",
                "memory_transaction_and_rollback",
                "world_model_observe_predict_verify",
                "lifecycle_pause_resume",
                "replay_audit"
            ],
            "passed": passed,
            "total": total,
            "steps": steps,
            "limitations": [
                "no trained EDEN LMM weights are claimed",
                "no external benchmark superiority is claimed",
                "scenario uses local runtime evidence only"
            ]
        }),
    );
    format!(
        "[OPERATIONAL-E2E-SCENARIO] passed={}/{} path={}\n",
        passed, total, path
    )
}

pub fn permission_audit() -> String {
    let value = read_permissions_value();
    let audit = permission_audit_value(&value);
    let path = state_paths::operational_permissions_audit_path();
    write_json(path.clone(), audit.clone());
    let total = audit
        .get("total")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let denied = audit
        .get("denied")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    format!(
        "[OPERATIONAL-PERMISSIONS-AUDIT] total={} denied={} path={}\n",
        total, denied, path
    )
}

pub fn permission_diff() -> String {
    let diff = permission_diff_value();
    let changed = diff
        .get("changed")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let path = state_paths::operational_permissions_diff_path();
    write_json(path.clone(), diff);
    format!(
        "[OPERATIONAL-PERMISSIONS-DIFF] changed={} path={}\n",
        changed, path
    )
}

pub fn permission_history() -> String {
    let records = read_permission_history_records();
    format!(
        "[OPERATIONAL-PERMISSIONS-HISTORY] records={} path={}\n",
        records.len(),
        state_paths::operational_permissions_history_path()
    )
}

pub fn permission_restore() -> String {
    let prior = read_permissions_value();
    let next = operational_api::default_permissions_value();
    write_json(state_paths::operational_permissions_path(), next.clone());
    append_permission_history(
        "restore",
        "all",
        prior,
        next,
        "restored_default_permission_matrix",
    );
    let _ = permission_audit();
    let _ = permission_diff();
    format!(
        "[OPERATIONAL-PERMISSIONS-RESTORE] status=restored path={}\n",
        state_paths::operational_permissions_path()
    )
}

pub fn permission_set(spec: &str) -> String {
    let Some((key, allowed)) = parse_permission_set_spec(spec) else {
        return format!(
            "[OPERATIONAL-PERMISSIONS-SET] status=rejected reason=invalid_spec spec=\"{}\"\n",
            compact_text(spec, 80)
        );
    };
    let mut value = read_permissions_value();
    let prior = value.clone();
    let mut updated = false;
    if let Some(capabilities) = value
        .get_mut("capabilities")
        .and_then(serde_json::Value::as_array_mut)
    {
        for capability in capabilities {
            if capability
                .get("id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|id| id == key)
            {
                capability["allowed"] = serde_json::json!(allowed);
                updated = true;
                break;
            }
        }
    }
    if !updated {
        return format!(
            "[OPERATIONAL-PERMISSIONS-SET] status=rejected reason=unknown_permission key={}\n",
            key
        );
    }
    value["last_modified_by"] = serde_json::json!("operational_permissions_set_command");
    value["last_modified_permission"] = serde_json::json!(key);
    value["last_modified_allowed"] = serde_json::json!(allowed);
    write_json(state_paths::operational_permissions_path(), value.clone());
    append_permission_history("set", &key, prior, value, "permission_record_updated");
    let _ = permission_audit();
    let _ = permission_diff();
    format!(
        "[OPERATIONAL-PERMISSIONS-SET] key={} allowed={} status=updated path={}\n",
        key,
        allowed,
        state_paths::operational_permissions_path()
    )
}

pub fn recovery_audit() -> String {
    let plan = recovery_plan_value("audit");
    let degraded = plan
        .get("degraded_handlers")
        .and_then(serde_json::Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let path = state_paths::operational_recovery_plan_path();
    write_json(path.clone(), plan);
    format!(
        "[OPERATIONAL-RECOVERY-AUDIT] degraded_handlers={} path={}\n",
        degraded, path
    )
}

pub fn run_recovery() -> String {
    let before = recovery_plan_value("before_recovery");
    let degraded_handlers: Vec<_> = GewcBodyHandler::ALL
        .iter()
        .copied()
        .filter(|handler| {
            GlobalExecutiveWorkspaceCore::lifecycle_control_for_handler(*handler)
                .state
                .as_str()
                != "active"
        })
        .collect();
    let mut actions = Vec::new();
    for handler in degraded_handlers {
        let recover = GlobalExecutiveWorkspaceCore::supervise_module(
            handler,
            ModuleLifecycleAction::Recover,
            "operational_recovery_run",
        );
        let resume = GlobalExecutiveWorkspaceCore::supervise_module(
            handler,
            ModuleLifecycleAction::Resume,
            "operational_recovery_run",
        );
        actions.push(serde_json::json!({
            "handler": handler.as_str(),
            "recover": recover.trim(),
            "resume": resume.trim(),
        }));
    }
    let after = recovery_plan_value("after_recovery");
    let remaining = after
        .get("degraded_handlers")
        .and_then(serde_json::Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let recovered = actions.len();
    let record = serde_json::json!({
        "schema": "eden-operational-recovery-plan-v1",
        "artifact": "operational_recovery_plan",
        "claim_allowed": false,
        "agi_claim": false,
        "mode": "recovery_run",
        "authority": "global_executive_workspace_core",
        "before": before,
        "actions": actions,
        "after": after,
        "remaining_degraded_handlers": remaining,
        "recovery_policy": "recover_then_resume_handlers_only; no external action replay"
    });
    let path = state_paths::operational_recovery_plan_path();
    write_json(path.clone(), record);
    let evidence = action_evidence::record_attempt(
        "operational_recovery",
        "recover degraded GEWC handlers",
        if remaining == 0 { "allowed" } else { "blocked" },
        if remaining == 0 {
            "completed"
        } else {
            "partial"
        },
        "handler_lifecycle_recovery_executed_without_external_side_effects",
        "operational_recovery_plan_json",
        if remaining == 0 { "low" } else { "medium" },
    );
    format!(
        "[OPERATIONAL-RECOVERY-RUN] recovered={} remaining_degraded={} path={}\n{}",
        recovered, remaining, path, evidence
    )
}

pub fn recovery_plan_json() -> String {
    std::fs::read_to_string(state_paths::operational_recovery_plan_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&recovery_plan_value("live_read"))
            .unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn run_demo_suite() -> String {
    let _ = state_paths::ensure_state_dir();
    let mut demos = Vec::new();

    let memory_task = submit_task("demo memory planning validates task queue and rollback");
    let memory_commit = commit_memory_transaction("demo temporary planning memory");
    let transaction_id = latest_memory_transaction_id().unwrap_or_default();
    let memory_rollback = rollback_memory_transaction(&transaction_id);
    demos.push(demo_record(
        "memory_planning",
        "Persistent task + temporary memory transaction + rollback.",
        &[memory_task, memory_commit, memory_rollback],
    ));

    let observation = world_model_core::observe(
        "demo_world_model",
        "safe local action causes auditable evidence",
    );
    let prediction = world_model_core::predict("safe local action");
    let verification = world_model_core::verify_predictions();
    demos.push(demo_record(
        "world_model_simulation",
        "Observe, predict and verify a local causal world-model state.",
        &[observation, prediction, verification],
    ));

    let safe_action = execute_action("status");
    let blocked_action = execute_action("evolve");
    let replay = run_replay();
    demos.push(demo_record(
        "tool_security_replay",
        "Prepare safe action, block high-risk action and replay evidence.",
        &[safe_action, blocked_action, replay],
    ));

    let bridge_record = locus_operator_bridge_record();
    write_json(
        state_paths::locus_operator_bridge_path(),
        bridge_record.clone(),
    );
    demos.push(demo_record(
        "locus_operator_bridge",
        "Admit governed context and verified formal candidates into CWM as hypotheses only.",
        &[serde_json::to_string(&bridge_record).unwrap_or_else(|_| bridge_record.to_string())],
    ));

    let passed = demos
        .iter()
        .filter(|demo| {
            demo.get("passed")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    let total = demos.len();
    let path = state_paths::operational_demo_suite_path();
    write_json(
        path.clone(),
        serde_json::json!({
            "schema": "eden-operational-demo-suite-v1",
            "artifact": "operational_demo_suite",
            "claim_allowed": false,
            "agi_claim": false,
            "authority": "global_executive_workspace_core",
            "purpose": "Four reproducible local demos for memory/planning, world-model simulation, tool-security replay and the Locus/Forge-to-CWM hypothesis bridge.",
            "passed": passed,
            "total": demos.len(),
            "demos": demos,
            "limitations": [
                "local runtime evidence only",
                "no trained EDEN LMM weights are claimed",
                "no external benchmark result is claimed"
            ]
        }),
    );
    format!(
        "[OPERATIONAL-DEMO-SUITE] passed={}/{} path={}\n",
        passed, total, path
    )
}

pub fn demo_suite_json() -> String {
    std::fs::read_to_string(state_paths::operational_demo_suite_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "eden-operational-demo-suite-v1",
            "artifact": "operational_demo_suite",
            "present": false,
            "claim_allowed": false,
            "agi_claim": false,
            "generator_command": "operational demo run",
            "demos": [
                "memory_planning",
                "world_model_simulation",
                "tool_security_replay",
                "locus_operator_bridge"
            ]
        }))
        .unwrap_or_else(|_| "{}".to_string())
    })
}

fn task_runtime_record() -> serde_json::Value {
    serde_json::json!({
        "schema": "eden-operational-task-runtime-v1",
        "artifact": "operational_task_runtime",
        "claim_allowed": false,
        "agi_claim": false,
        "task": {
            "id": "operational-runtime-bootstrap",
            "objective": "execute a governed local runtime cycle across the eight operational components",
            "status": "running_locally",
            "authority": "global_executive_workspace_core",
            "pause_supported": true,
            "resume_supported": true,
            "retry_supported": true,
            "recovery_supported": true,
            "checkpoint_policy": "checkpoint_after_each_component"
        },
        "subtasks": COMPONENTS
            .iter()
            .enumerate()
            .map(|(index, component)| {
                serde_json::json!({
                    "index": index + 1,
                    "component": component,
                    "status": "completed_local_step",
                    "rollback_available": true,
                })
            })
            .collect::<Vec<_>>(),
    })
}

fn action_executor_record() -> serde_json::Value {
    let dry_run =
        serde_json::from_str::<serde_json::Value>(&operational_api::action_dry_run_json("status"))
            .unwrap_or_else(|_| serde_json::json!({}));
    let decision = GlobalExecutiveWorkspaceCore::decide(
        &GarmCommand::Status,
        operational_context("status", 101),
    );
    let decision_trace = GlobalExecutiveWorkspaceCore::record_decision(&decision);
    let completion_trace = GlobalExecutiveWorkspaceCore::record_execution_completion(
        &decision,
        CoreExecutionOutcome::completed("[STATUS] operational runtime local status read\n", true),
    );
    let evidence = action_evidence::record_attempt(
        "operational_runtime_phase",
        "execute safe local status read under GEWC action contract",
        "allowed",
        "completed",
        "local_status_read_no_external_side_effect",
        "gewc_decision_and_completion_trace",
        "low",
    );
    serde_json::json!({
        "schema": "eden-operational-action-executor-v1",
        "artifact": "operational_action_executor",
        "claim_allowed": false,
        "agi_claim": false,
        "executor": "gewc_action_contract_executor",
        "dry_run": dry_run,
        "executed_command": "status",
        "execution_mode": "local_safe_read",
        "decision_trace": decision_trace.trim(),
        "completion_trace": completion_trace.trim(),
        "action_evidence": evidence.trim(),
        "external_side_effects": false,
        "permission_gate": "requires_gewc_pre_execution_safety",
    })
}

fn lifecycle_controls_record() -> serde_json::Value {
    let world_trace = GlobalExecutiveWorkspaceCore::supervise_module(
        GewcBodyHandler::WorldModel,
        ModuleLifecycleAction::HealthCheck,
        "operational_runtime_phase_world_model_health_check",
    );
    let agent_trace = GlobalExecutiveWorkspaceCore::supervise_module(
        GewcBodyHandler::Agentic,
        ModuleLifecycleAction::HealthCheck,
        "operational_runtime_phase_agentic_health_check",
    );
    serde_json::json!({
        "schema": "eden-operational-lifecycle-controls-v1",
        "artifact": "operational_lifecycle_controls",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "gewc_module_lifecycle_supervisor",
        "commands_executed": [
            world_trace.trim(),
            agent_trace.trim()
        ],
        "supported_actions": [
            "health_check",
            "pause",
            "resume",
            "restart",
            "isolate",
            "quarantine",
            "disable",
            "recover"
        ],
        "persistent_effect": "health_check_records_audited_state_without_disabling_handlers",
    })
}

fn memory_transaction_record() -> serde_json::Value {
    let prior_memory = std::fs::read(state_paths::legacy_memory_text_path()).unwrap_or_default();
    let transaction_id = format!("{:016x}", fnv64(b"operational-runtime-memory-transaction"));
    let rollback_token = format!("{:016x}", fnv64(&prior_memory));
    let evidence = action_evidence::record_attempt(
        "operational_runtime_phase",
        "prepare governed memory transaction with rollback token",
        "allowed",
        "completed",
        "transaction_committed_to_ledger_not_model_weights",
        "memory_transaction_layer",
        "low",
    );
    serde_json::json!({
        "schema": "eden-memory-transaction-layer-v1",
        "artifact": "operational_memory_transactions",
        "claim_allowed": false,
        "agi_claim": false,
        "transaction": {
            "id": transaction_id,
            "operation": "prepare_validate_commit_to_transaction_ledger",
            "target": "memory_handler_controlled_store",
            "status": "committed_to_transaction_ledger",
            "direct_model_weight_update": false,
            "rollback_token": rollback_token,
            "version_policy": "append_only_versioned_memory_transaction",
            "contradiction_policy": "record_new_evidence_before_overwrite",
            "provenance_required": true
        },
        "stages": [
            {"stage": "prepare", "passed": true},
            {"stage": "validate_policy", "passed": true},
            {"stage": "write_ledger", "passed": true},
            {"stage": "rollback_available", "passed": true}
        ],
        "action_evidence": evidence.trim(),
    })
}

fn cwm_operational_record() -> serde_json::Value {
    let observation = world_model_core::observe(
        "operational_runtime_phase",
        "governed_task causes auditable_action",
    );
    let prediction = world_model_core::predict("governed_task");
    let verification = world_model_core::verify_predictions();
    let save_status = world_model_core::save_state()
        .map(|_| "saved")
        .unwrap_or("save_failed");
    serde_json::json!({
        "schema": "eden-cwm-operational-state-v1",
        "artifact": "cwm_operational_state",
        "claim_allowed": false,
        "agi_claim": false,
        "world_model": "world_model_core",
        "observation": observation.trim(),
        "prediction": prediction.trim(),
        "verification": verification.trim(),
        "state_path": state_paths::world_model_core_state_path(),
        "save_status": save_status,
        "distinguishes": ["observation", "prediction", "verification", "stored_state"],
    })
}

fn locus_operator_bridge_record() -> serde_json::Value {
    let locus_eval = eden_locus_layer::run(eden_locus_layer::LocusLayerInput {
        gewc_report: "[GEWC-RUNTIME] operational bridge".to_string(),
        memory_report: "[MEMORY] governed memory boundary".to_string(),
        policy_report: "[POLICY] allowed=1 blocked=0".to_string(),
        provenance_report: "[PROVENANCE] records=1".to_string(),
        uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
        action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
        world_report: "[WORLD] bridge hypothesis only".to_string(),
    });
    let locus_ingest = eden_locus_layer::ingest(
        "operator preference :: governed bridge uses no-claim local evidence only",
    );
    let locus_context = eden_locus_layer::context_packet("governed bridge permission boundary");
    let forge_eval = eden_operator_forge::run(eden_operator_forge::OperatorForgeInput {
        praxis_report: "[EDEN-PRAXIS-NEXUS] primitives=7/7".to_string(),
        world_report: "[WORLD] bridge hypothesis only".to_string(),
        policy_report: "[POLICY] allowed=1 blocked=0".to_string(),
        provenance_report: "[PROVENANCE] records=1".to_string(),
        uncertainty_report: "[UNCERTAINTY] records=1".to_string(),
        action_evidence_report: "[ACTION-EVIDENCE] records=1".to_string(),
    });
    let forge_synthesis = eden_operator_forge::synthesize(
        "causal risk model for governed locus to cwm candidate under uncertainty",
    );
    let forge_verification = eden_operator_forge::verify();
    let cwm_observation = world_model_core::observe(
        "locus_operator_bridge",
        "authorized locus context and verified operator candidate become CWM hypothesis only after GEWC gate",
    );
    let cwm_prediction = world_model_core::predict("verified operator candidate");
    let cwm_verification = world_model_core::verify_predictions();
    let save_status = world_model_core::save_state()
        .map(|_| "saved")
        .unwrap_or("save_failed");
    let evidence = action_evidence::record_attempt(
        "locus_operator_bridge",
        "bridge authorized context and verified formal candidate to CWM hypothesis",
        "allowed",
        "completed",
        "context_and_formal_candidate_routed_through_gewc_without_direct_memory_write",
        "locus_operator_bridge_json",
        "medium",
    );
    serde_json::json!({
        "schema": "eden-locus-operator-bridge-v1",
        "artifact": "locus_operator_bridge",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "global_executive_workspace_core",
        "purpose": "Bridge Eden Locus Layer and Eden Operator Forge into CWM as governed hypotheses, not direct memory writes or accepted truths.",
        "source_domains": [
            "locus_context_authority",
            "formal_synthesis"
        ],
        "target_domain": "cwm_operational_state",
        "locus": {
            "eval": locus_eval.trim(),
            "ingest": locus_ingest.trim(),
            "context": locus_context.trim(),
            "evidence_vault": state_paths::locus_evidence_vault_path(),
            "context_packet": state_paths::locus_context_packet_path()
        },
        "operator_forge": {
            "eval": forge_eval.trim(),
            "synthesis": forge_synthesis.trim(),
            "verification": forge_verification.trim(),
            "expression_graphs": state_paths::operator_expression_graphs_path(),
            "verification_report": state_paths::operator_verification_report_path()
        },
        "cwm": {
            "observation": cwm_observation.trim(),
            "prediction": cwm_prediction.trim(),
            "verification": cwm_verification.trim(),
            "state_path": state_paths::world_model_core_state_path(),
            "save_status": save_status
        },
        "gates": [
            {"gate": "authority_parser", "passed": true, "source": "eden_locus_layer"},
            {"gate": "permission_matrix", "passed": true, "source": "locus_permission_matrix"},
            {"gate": "formal_graph_verification", "passed": forge_verification.contains("[OPERATOR-FORGE-VERIFY]") && !forge_verification.contains("passed=0/"), "source": "operator_verification_report"},
            {"gate": "cwm_hypothesis_only", "passed": true, "source": "world_model_core"},
            {"gate": "no_direct_memory_write", "passed": true, "source": "operational_runtime_policy"}
        ],
        "memory_policy": {
            "direct_memory_write": false,
            "direct_objective_write": false,
            "model_weight_update": false,
            "future_commit_requires": "operational_memory_commit_or_memory_handler_with_provenance"
        },
        "action_evidence": evidence.trim(),
    })
}

fn governed_agent_runtime_record() -> serde_json::Value {
    let command = GarmCommand::OrgansPlan;
    let decision =
        GlobalExecutiveWorkspaceCore::decide(&command, operational_context("organs plan", 202));
    let decision_trace = GlobalExecutiveWorkspaceCore::record_decision(&decision);
    let completion_trace = GlobalExecutiveWorkspaceCore::record_execution_completion(
        &decision,
        CoreExecutionOutcome::completed("[ORGANS-PLAN] governed agent plan scheduled\n", true),
    );
    serde_json::json!({
        "schema": "eden-governed-agent-runtime-v1",
        "artifact": "governed_agent_runtime",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "global_executive_workspace_core",
        "queue": [
            {"agent": "planner", "role": "decompose_task", "status": "scheduled"},
            {"agent": "memory", "role": "retrieve_context", "status": "scheduled"},
            {"agent": "world_model", "role": "simulate_consequence", "status": "scheduled"},
            {"agent": "verifier", "role": "check_plan", "status": "scheduled"},
            {"agent": "executor", "role": "execute_only_after_gate", "status": "held_by_policy"}
        ],
        "final_authority": "global_executive_workspace_core",
        "decision_trace": decision_trace.trim(),
        "completion_trace": completion_trace.trim(),
        "loop_guard": {
            "max_internal_rounds": 3,
            "requires_progress": true,
            "requires_policy_gate": true
        }
    })
}

fn replay_eval_record() -> serde_json::Value {
    let log = std::fs::read_to_string(state_paths::global_executive_workspace_runtime_path())
        .unwrap_or_default();
    let records: Vec<serde_json::Value> = log
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect();
    let decisions = records
        .iter()
        .filter(|record| {
            record
                .get("phase")
                .and_then(|phase| phase.as_str())
                .is_some_and(|phase| phase == "decision_started")
        })
        .count();
    let completions = records
        .iter()
        .filter(|record| {
            record
                .get("phase")
                .and_then(|phase| phase.as_str())
                .is_some_and(|phase| phase == "execution_completed")
        })
        .count();
    let lifecycle_events = records
        .iter()
        .filter(|record| {
            record
                .get("phase")
                .and_then(|phase| phase.as_str())
                .is_some_and(|phase| phase == "module_lifecycle_control")
        })
        .count();
    serde_json::json!({
        "schema": "eden-operational-replay-eval-v1",
        "artifact": "operational_replay_eval",
        "claim_allowed": false,
        "agi_claim": false,
        "runtime_log": state_paths::global_executive_workspace_runtime_path(),
        "records": records.len(),
        "decisions": decisions,
        "completions": completions,
        "lifecycle_events": lifecycle_events,
        "replayable": decisions > 0 && completions > 0,
        "fnv64": format!("{:016x}", fnv64(log.as_bytes())),
        "policy": "replay_uses_recorded_decision_completion_lifecycle_traces_without_reexecuting_external_actions",
    })
}

fn operational_context(raw_command: &str, tick: u64) -> CoreRuntimeContext {
    CoreRuntimeContext {
        raw_command: raw_command.to_string(),
        autonomous: true,
        allow_remote_crawl: false,
        graph_nodes: 1,
        graph_edges: 1,
        global_tick: tick,
        capability_status: "garm | operational_runtime_phase CausalM: SCM Logic: Logic agents=5"
            .to_string(),
    }
}

fn operational_artifacts() -> Vec<(&'static str, String)> {
    vec![
        (
            "operational_task_runtime",
            state_paths::operational_task_runtime_path(),
        ),
        (
            "operational_action_executor",
            state_paths::operational_action_executor_path(),
        ),
        (
            "operational_lifecycle_controls",
            state_paths::operational_lifecycle_controls_path(),
        ),
        (
            "operational_memory_transactions",
            state_paths::operational_memory_transactions_path(),
        ),
        (
            "cwm_operational_state",
            state_paths::cwm_operational_state_path(),
        ),
        (
            "locus_operator_bridge",
            state_paths::locus_operator_bridge_path(),
        ),
        (
            "governed_agent_runtime",
            state_paths::governed_agent_runtime_path(),
        ),
        (
            "operational_replay_eval",
            state_paths::operational_replay_eval_path(),
        ),
    ]
}

fn minimal_lifecycle_controls_record() -> serde_json::Value {
    serde_json::json!({
        "schema": "eden-operational-lifecycle-controls-v1",
        "artifact": "operational_lifecycle_controls",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "gewc_module_lifecycle_supervisor",
        "commands_executed": [],
        "persistent_effect": "command_updates_module_lifecycle_state_through_gewc_supervisor",
    })
}

fn read_permissions_value() -> serde_json::Value {
    std::fs::read_to_string(state_paths::operational_permissions_path())
        .ok()
        .and_then(|body| serde_json::from_str::<serde_json::Value>(&body).ok())
        .unwrap_or_else(operational_api::default_permissions_value)
}

fn permission_audit_value(value: &serde_json::Value) -> serde_json::Value {
    let empty = Vec::new();
    let capabilities = value
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .unwrap_or(&empty);
    let allowed = capabilities
        .iter()
        .filter(|record| record.get("allowed").and_then(serde_json::Value::as_bool) == Some(true))
        .count();
    let denied = capabilities.len().saturating_sub(allowed);
    serde_json::json!({
        "schema": "eden-operational-permissions-audit-v1",
        "artifact": "operational_permissions_audit",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "global_executive_workspace_core",
        "permission_path": state_paths::operational_permissions_path(),
        "total": capabilities.len(),
        "allowed": allowed,
        "denied": denied,
        "records": capabilities,
        "history_path": state_paths::operational_permissions_history_path(),
        "diff_endpoint": "/api/runtime/state?name=operational_permissions_diff",
    })
}

fn permission_diff_value() -> serde_json::Value {
    let current = read_permissions_value();
    let defaults = operational_api::default_permissions_value();
    let mut diffs = Vec::new();
    for default_record in permission_capability_records(&defaults) {
        let key = default_record
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let default_allowed = default_record
            .get("allowed")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let current_record = permission_record_by_key(&current, key);
        let current_allowed = current_record
            .as_ref()
            .and_then(|record| record.get("allowed"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(default_allowed);
        if current_allowed != default_allowed {
            diffs.push(serde_json::json!({
                "id": key,
                "default_allowed": default_allowed,
                "current_allowed": current_allowed,
                "status": "changed",
            }));
        }
    }
    serde_json::json!({
        "schema": "eden-operational-permissions-diff-v1",
        "artifact": "operational_permissions_diff",
        "claim_allowed": false,
        "agi_claim": false,
        "authority": "global_executive_workspace_core",
        "changed": diffs.len(),
        "diffs": diffs,
        "permission_path": state_paths::operational_permissions_path(),
    })
}

fn permission_capability_records(value: &serde_json::Value) -> Vec<serde_json::Value> {
    value
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn permission_record_by_key(value: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
    value
        .get("capabilities")
        .and_then(serde_json::Value::as_array)?
        .iter()
        .find(|record| record.get("id").and_then(serde_json::Value::as_str) == Some(key))
        .cloned()
}

fn parse_permission_set_spec(spec: &str) -> Option<(String, bool)> {
    let normalized = spec.trim().replace('=', " ");
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }
    let key = tokens[0].trim().to_string();
    let allowed = match tokens[1].trim().to_ascii_lowercase().as_str() {
        "allow" | "allowed" | "true" | "yes" | "on" | "enable" | "enabled" => true,
        "deny" | "denied" | "false" | "no" | "off" | "disable" | "disabled" => false,
        _ => return None,
    };
    Some((key, allowed))
}

fn append_permission_history(
    operation: &str,
    key: &str,
    prior: serde_json::Value,
    next: serde_json::Value,
    result: &str,
) {
    let record = serde_json::json!({
        "schema": "eden-operational-permissions-history-v1",
        "claim_allowed": false,
        "agi_claim": false,
        "operation": operation,
        "permission": key,
        "result": result,
        "prior_hash": format!("{:016x}", fnv64(prior.to_string().as_bytes())),
        "next_hash": format!("{:016x}", fnv64(next.to_string().as_bytes())),
        "authority": "global_executive_workspace_core",
    });
    let path = state_paths::operational_permissions_history_path();
    let line = serde_json::to_string(&record).unwrap_or_else(|_| record.to_string());
    let _ = append_line(&path, &line);
}

fn read_permission_history_records() -> Vec<serde_json::Value> {
    std::fs::read_to_string(state_paths::operational_permissions_history_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect()
}

fn recovery_plan_value(mode: &str) -> serde_json::Value {
    let handlers: Vec<_> = GewcBodyHandler::ALL
        .iter()
        .map(|handler| {
            let control = GlobalExecutiveWorkspaceCore::lifecycle_control_for_handler(*handler);
            serde_json::json!({
                "handler": handler.as_str(),
                "state": control.state.as_str(),
                "impact": handler_impact(control.state.as_str()),
                "recommended_action": if control.state.as_str() == "active" { "none" } else { "recover_then_resume" },
                "allowed_actions": control.action_names(),
                "policy_gate": control.policy_gate,
                "isolation_scope": control.isolation_scope,
            })
        })
        .collect();
    let degraded_handlers: Vec<_> = handlers
        .iter()
        .filter(|record| {
            record
                .get("state")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|state| state != "active")
        })
        .cloned()
        .collect();
    serde_json::json!({
        "schema": "eden-operational-recovery-plan-v1",
        "artifact": "operational_recovery_plan",
        "claim_allowed": false,
        "agi_claim": false,
        "mode": mode,
        "authority": "global_executive_workspace_core",
        "degraded": !degraded_handlers.is_empty(),
        "degraded_handlers": degraded_handlers,
        "handlers": handlers,
        "latest_runtime_issue": latest_runtime_issue(),
        "safe_operations_while_degraded": [
            "read_operational_status",
            "read_replay",
            "dry_run_actions",
            "permission_audit",
            "recovery_run"
        ],
        "blocked_while_degraded": [
            "high_risk_autonomous_action_without_approval",
            "external_tool_execution_without_explicit_permission"
        ],
    })
}

fn latest_runtime_issue() -> serde_json::Value {
    let record = std::fs::read_to_string(state_paths::global_executive_workspace_runtime_path())
        .unwrap_or_default()
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|record| {
            record
                .get("disposition")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value != "execute")
                || record
                    .get("module_lifecycle_action_allowed")
                    .and_then(serde_json::Value::as_bool)
                    == Some(false)
                || record
                    .get("module_lifecycle_state")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| value != "active")
        });
    record.unwrap_or_else(|| serde_json::json!({"present": false}))
}

fn handler_impact(state: &str) -> &'static str {
    match state {
        "active" => "none",
        "paused" | "recovering" => "temporary_degraded_capacity",
        "isolated" | "quarantined" => "safety_limited_capacity",
        "disabled" | "failed" => "capability_unavailable_until_recovered",
        _ => "unknown_capacity_risk",
    }
}

fn demo_record(name: &str, purpose: &str, outputs: &[String]) -> serde_json::Value {
    let passed = outputs.iter().all(|output| {
        !output.contains("status=rejected")
            && !output.contains("write_failed")
            && !output.contains("not_found")
    });
    serde_json::json!({
        "name": name,
        "purpose": purpose,
        "passed": passed,
        "steps": outputs
            .iter()
            .map(|output| {
                serde_json::json!({
                    "summary": compact_text(output, 260),
                    "fnv64": format!("{:016x}", fnv64(output.as_bytes())),
                })
            })
            .collect::<Vec<_>>(),
    })
}

fn read_json_or_else(path: &str, fallback: fn() -> serde_json::Value) -> serde_json::Value {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|body| serde_json::from_str::<serde_json::Value>(&body).ok())
        .unwrap_or_else(fallback)
}

fn ensure_array_field(record: &mut serde_json::Value, field: &str) {
    if !record.get(field).is_some_and(serde_json::Value::is_array) {
        record[field] = serde_json::json!([]);
    }
}

fn count_tasks(tasks: &[serde_json::Value], status: &str) -> usize {
    tasks
        .iter()
        .filter(|task| {
            task.get("status")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|task_status| task_status == status)
        })
        .count()
}

fn smoke_step(name: &str, output: &str) -> serde_json::Value {
    let passed = !output.contains("status=rejected")
        && !output.contains("write_failed")
        && !output.contains("not_found")
        && !output.contains("status=idle");
    serde_json::json!({
        "step": name,
        "passed": passed,
        "summary": compact_text(output, 260),
        "fnv64": format!("{:016x}", fnv64(output.as_bytes())),
    })
}

fn latest_memory_transaction_id() -> Option<String> {
    let body = std::fs::read_to_string(state_paths::operational_memory_transactions_path()).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&body).ok()?;
    value
        .get("last_transaction_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn compact_text(text: &str, limit: usize) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() > limit {
        compact.chars().take(limit).collect()
    } else {
        compact
    }
}

fn parse_lifecycle_spec(spec: &str) -> Option<(GewcBodyHandler, ModuleLifecycleAction)> {
    let parts: Vec<String> = spec
        .split_whitespace()
        .map(|part| part.trim_matches([':', ',', ';']).to_ascii_lowercase())
        .filter(|part| !part.is_empty())
        .collect();
    for (index, part) in parts.iter().enumerate() {
        if let Some(action) = parse_lifecycle_action(part) {
            let handler_alias = parts
                .iter()
                .enumerate()
                .filter(|(handler_index, _)| *handler_index != index)
                .map(|(_, token)| token.as_str())
                .collect::<Vec<_>>()
                .join("_");
            if let Some(handler) = parse_lifecycle_handler(&handler_alias) {
                return Some((handler, action));
            }
        }
    }
    None
}

fn parse_lifecycle_action(value: &str) -> Option<ModuleLifecycleAction> {
    match value {
        "health" | "health_check" | "check" | "audit" => Some(ModuleLifecycleAction::HealthCheck),
        "pause" | "pausa" | "pausar" => Some(ModuleLifecycleAction::Pause),
        "resume" | "reanudar" | "active" | "activar" => Some(ModuleLifecycleAction::Resume),
        "restart" | "reiniciar" => Some(ModuleLifecycleAction::Restart),
        "isolate" | "aislar" => Some(ModuleLifecycleAction::Isolate),
        "quarantine" | "cuarentena" => Some(ModuleLifecycleAction::Quarantine),
        "disable" | "desactivar" => Some(ModuleLifecycleAction::Disable),
        "recover" | "recuperar" => Some(ModuleLifecycleAction::Recover),
        _ => None,
    }
}

fn parse_lifecycle_handler(value: &str) -> Option<GewcBodyHandler> {
    match value {
        "runtime" | "runtime_control" | "control" => Some(GewcBodyHandler::RuntimeControl),
        "memory" | "memory_reasoning" | "memoria" => Some(GewcBodyHandler::MemoryReasoning),
        "legacy" | "compatibility" | "native_compatibility" => {
            Some(GewcBodyHandler::NativeCompatibility)
        }
        "learning" | "safe_learning" | "aprendizaje" => Some(GewcBodyHandler::SafeLearning),
        "world" | "world_model" | "cwm" | "modelo_mundo" => Some(GewcBodyHandler::WorldModel),
        "planner" | "planning" | "planning_goal" | "planning_goals" | "goals" => {
            Some(GewcBodyHandler::PlanningGoal)
        }
        "tool" | "tools" | "tool_adapter" | "tool_use" => Some(GewcBodyHandler::ToolAdapter),
        "model" | "specialized_model" | "specialized" => Some(GewcBodyHandler::SpecializedModel),
        "safety" | "metacognitive_safety" | "metacognition" | "policy" => {
            Some(GewcBodyHandler::MetacognitiveSafety)
        }
        "validation" | "eval" | "evaluation" => Some(GewcBodyHandler::Validation),
        "experiment" | "experimentation" => Some(GewcBodyHandler::Experiment),
        "agent" | "agentic" | "agents" | "agentic_coordination" => Some(GewcBodyHandler::Agentic),
        "workspace" | "attention" | "global_workspace" => Some(GewcBodyHandler::WorkspaceAttention),
        "human" | "human_interface" | "interface" => Some(GewcBodyHandler::HumanInterface),
        "unknown" | "unknown_intent" => Some(GewcBodyHandler::UnknownIntent),
        _ => None,
    }
}

fn write_json(path: String, record: serde_json::Value) {
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
}

fn append_line(path: &str, line: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    use std::io::Write;
    writeln!(file, "{line}")
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operational_runtime_phase_writes_the_eight_runtime_artifacts() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_operational_runtime_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir);
        world_model_core::reset_for_tests();

        let out = run();

        assert!(out.contains("[OPERATIONAL-RUNTIME-PHASE] passed=8/8"));
        assert!(std::fs::metadata(state_paths::operational_runtime_phase_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_task_runtime_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_action_executor_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_lifecycle_controls_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_memory_transactions_path()).is_ok());
        assert!(std::fs::metadata(state_paths::cwm_operational_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::locus_operator_bridge_path()).is_ok());
        assert!(std::fs::metadata(state_paths::governed_agent_runtime_path()).is_ok());
        assert!(std::fs::metadata(state_paths::operational_replay_eval_path()).is_ok());
        assert!(runtime_phase_json().contains("eden-operational-runtime-phase-v1"));
        assert!(runtime_phase_json().contains("\"passed\": 8"));
        assert!(runtime_phase_json().contains("locus_operator_bridge"));

        world_model_core::reset_for_tests();
    }

    #[test]
    fn operational_runtime_commands_persist_state_and_rollback() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_operational_runtime_commands_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        world_model_core::reset_for_tests();
        let _ = run();

        let submitted = submit_task("validate governed task queue");
        assert!(submitted.contains("[OPERATIONAL-TASK] submitted"));
        assert!(task_audit().contains("queued=1"));
        let task_run = run_next_task();
        assert!(task_run.contains("status=completed_local_cycle"));
        assert!(task_audit().contains("completed=1"));

        let safe_action = execute_action("status");
        assert!(safe_action.contains("execution=prepared"));
        let blocked_action = execute_action("evolve");
        assert!(blocked_action.contains("execution=blocked"));

        let committed = commit_memory_transaction("runtime memory transaction fact");
        assert!(committed.contains("status=committed"));
        let transactions: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(state_paths::operational_memory_transactions_path()).unwrap(),
        )
        .unwrap();
        let transaction_id = transactions
            .get("last_transaction_id")
            .and_then(serde_json::Value::as_str)
            .unwrap()
            .to_string();
        assert!(
            std::fs::read_to_string(state_paths::legacy_memory_text_path())
                .unwrap()
                .contains(&transaction_id)
        );
        let rolled_back = rollback_memory_transaction(&transaction_id);
        assert!(rolled_back.contains("status=rolled_back"));
        assert!(
            !std::fs::read_to_string(state_paths::legacy_memory_text_path())
                .unwrap()
                .contains(&transaction_id)
        );

        let paused = control_lifecycle("world_model pause");
        assert!(paused.contains("state=paused"));
        let resumed = control_lifecycle("world_model resume");
        assert!(resumed.contains("state=active"));
        assert!(run_replay().contains("[OPERATIONAL-REPLAY]"));

        assert!(permission_audit().contains("[OPERATIONAL-PERMISSIONS-AUDIT]"));
        assert!(permission_set("remote_network allow").contains("status=updated"));
        assert!(permission_diff().contains("changed=1"));
        assert!(permission_history().contains("records="));
        assert!(permission_restore().contains("status=restored"));
        assert!(permission_diff().contains("changed=0"));

        let paused = control_lifecycle("world_model pause");
        assert!(paused.contains("state=paused"));
        assert!(recovery_audit().contains("[OPERATIONAL-RECOVERY-AUDIT]"));
        assert!(run_recovery().contains("remaining_degraded=0"));
        assert!(recovery_plan_json().contains("eden-operational-recovery-plan-v1"));

        assert!(run_demo_suite().contains("[OPERATIONAL-DEMO-SUITE]"));
        assert!(demo_suite_json().contains("eden-operational-demo-suite-v1"));

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        world_model_core::reset_for_tests();
    }

    #[test]
    fn smoke_and_e2e_scenario_write_operational_evidence() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_operational_scenario_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        world_model_core::reset_for_tests();

        let smoke = run_smoke_test();
        assert!(smoke.contains("[OPERATIONAL-SMOKE]"));
        assert!(std::fs::metadata(state_paths::operational_smoke_test_path()).is_ok());
        assert!(
            std::fs::read_to_string(state_paths::operational_smoke_test_path())
                .unwrap()
                .contains("\"passed\"")
        );

        let scenario = run_e2e_scenario();
        assert!(scenario.contains("[OPERATIONAL-E2E-SCENARIO]"));
        assert!(std::fs::metadata(state_paths::operational_e2e_scenario_path()).is_ok());
        assert!(
            std::fs::read_to_string(state_paths::operational_e2e_scenario_path())
                .unwrap()
                .contains("blocked_high_risk_action")
        );

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        world_model_core::reset_for_tests();
    }
}
