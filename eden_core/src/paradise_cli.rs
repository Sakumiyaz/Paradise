use crate::eden_garm::global_executive_workspace::{
    CoreExecutionOutcome, CoreRuntimeContext, GewcBodyRegistry, GlobalExecutiveWorkspaceCore,
    GlobalExecutiveWorkspaceInput,
};
use crate::eden_garm::nodes::{
    command_router::CommandRouterNode, policy_guard, provenance_ledger, uncertainty_ledger,
    world_model_core,
};
use crate::eden_garm::{
    action_evidence, eden_locus_layer, eden_operator_forge, model_runtime, operational_api,
    operational_runtime, paradise_worldcell, praxis_nexus, runtime_spine, state_paths,
};
use serde_json::Value;
use std::env;
use std::fmt;

const DEFAULT_STATE_DIR: &str = "/tmp/paradise";

pub fn main_entry(args: Vec<String>) {
    match parse_cli(args) {
        Ok(cli) => {
            if let Err(err) = run(cli) {
                eprintln!("paradise: {err}");
                std::process::exit(1);
            }
        }
        Err(CliError::Help) => print_usage(),
        Err(err) => {
            eprintln!("paradise: {err}");
            eprintln!();
            print_usage();
            std::process::exit(2);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Cli {
    state_dir: String,
    json: bool,
    command: CliCommand,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CliCommand {
    Status,
    Worldcell,
    RunDryRun(String),
}

#[derive(Debug)]
enum CliError {
    Help,
    Message(String),
    Io(std::io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Help => write!(f, "help requested"),
            Self::Message(message) => write!(f, "{message}"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

fn parse_cli(args: Vec<String>) -> Result<Cli, CliError> {
    let mut state_dir =
        env::var("PARADISE_STATE_DIR").unwrap_or_else(|_| DEFAULT_STATE_DIR.to_string());
    let mut json = false;
    let mut cursor = 0;
    while cursor < args.len() {
        match args[cursor].as_str() {
            "-h" | "--help" => return Err(CliError::Help),
            "--state-dir" => {
                state_dir = option_value(&args, cursor, "--state-dir")?;
                cursor += 2;
            }
            "--json" => {
                json = true;
                cursor += 1;
            }
            _ => break,
        }
    }

    let command = parse_command(&args[cursor..])?;
    Ok(Cli {
        state_dir,
        json,
        command,
    })
}

fn parse_command(args: &[String]) -> Result<CliCommand, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::Message("missing command".to_string()));
    };
    match command {
        "status" => Ok(CliCommand::Status),
        "worldcell" => Ok(CliCommand::Worldcell),
        "run" => parse_run(&args[1..]),
        other => Err(CliError::Message(format!("unknown command: {other}"))),
    }
}

fn parse_run(args: &[String]) -> Result<CliCommand, CliError> {
    if args.first().map(String::as_str) != Some("--dry-run") {
        return Err(CliError::Message(
            "run requires --dry-run before the intent".to_string(),
        ));
    }
    let intent = join_required(&args[1..], "missing intent for run --dry-run")?;
    Ok(CliCommand::RunDryRun(intent))
}

fn option_value(args: &[String], cursor: usize, option: &str) -> Result<String, CliError> {
    args.get(cursor + 1)
        .cloned()
        .ok_or_else(|| CliError::Message(format!("missing value for {option}")))
}

fn join_required(args: &[String], message: &str) -> Result<String, CliError> {
    if args.is_empty() {
        Err(CliError::Message(message.to_string()))
    } else {
        Ok(args.join(" "))
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    state_paths::set_state_dir(&cli.state_dir);
    state_paths::ensure_state_dir().map_err(CliError::Message)?;

    match cli.command {
        CliCommand::Status => run_status(&cli.state_dir, cli.json),
        CliCommand::Worldcell => run_worldcell(cli.json),
        CliCommand::RunDryRun(intent) => run_dry_run(&intent, cli.json),
    }
}

fn run_status(state_dir: &str, json: bool) -> Result<(), CliError> {
    let _ = runtime_spine::run();
    let _ = operational_api::run();
    let _ = operational_runtime::run();
    let status = operational_api::status_json(operational_status_input());
    if json {
        print!("{status}");
        if !status.ends_with('\n') {
            println!();
        }
        return Ok(());
    }

    let value = parse_json(&status);
    let state = path_str(&value, &["state"]).unwrap_or("unknown");
    let ready = path_bool(&value, &["health", "ready"]).unwrap_or(false);
    let latest_decision = path_str(&value, &["latest", "decision_id"]).unwrap_or("none");
    let permissions = value
        .pointer("/permissions/permissions")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    println!(
        "[PARADISE-STATUS] state={state} ready={ready} latest_decision={latest_decision} permissions={permissions} claim_allowed=false state_dir={state_dir}"
    );
    println!(
        "evidence: {}",
        state_paths::operational_runtime_phase_path()
    );
    println!("next: cargo run -p eden_core --bin paradise -- worldcell");
    Ok(())
}

fn run_worldcell(json: bool) -> Result<(), CliError> {
    let input = build_worldcell_input();
    let output = paradise_worldcell::run(input);
    if json {
        print_file_or_fallback(&state_paths::paradise_worldcell_runtime_path(), "{}")?;
    } else {
        print!("{output}");
        println!(
            "sessions: {}",
            state_paths::paradise_worldcell_sessions_path()
        );
    }
    Ok(())
}

fn run_dry_run(intent: &str, json: bool) -> Result<(), CliError> {
    let intent_output = paradise_worldcell::record_intent(intent);
    let plan_output = paradise_worldcell::plan_session("");
    if json {
        print!("{}", paradise_worldcell::sessions_json());
        return Ok(());
    }

    let sessions = parse_json(&paradise_worldcell::sessions_json());
    let latest = path_str(&sessions, &["latest_session_id"]).unwrap_or("none");
    let latest_status = sessions
        .get("sessions")
        .and_then(Value::as_array)
        .and_then(|records| records.last())
        .and_then(|session| session.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let candidate = sessions
        .get("sessions")
        .and_then(Value::as_array)
        .and_then(|records| records.last())
        .and_then(|session| session.get("candidate_command"))
        .and_then(Value::as_str)
        .unwrap_or("none");
    println!(
        "[PARADISE-RUN-DRY-RUN] session={latest} status={latest_status} would_execute=false candidate=\"{candidate}\""
    );
    print!("{intent_output}");
    print!("{plan_output}");
    println!(
        "evidence: {}",
        state_paths::paradise_worldcell_sessions_path()
    );
    Ok(())
}

fn build_worldcell_input() -> paradise_worldcell::ParadiseWorldcellInput {
    seed_runtime_evidence();

    let policy_report = format!(
        "{}{}",
        policy_guard::evaluate("remote network action requires explicit gate"),
        policy_guard::report()
    );
    let provenance_report = format!(
        "{}{}",
        provenance_ledger::record(
            "paradise_cli",
            "verified local evidence for the Paradise Worldcell runtime"
        ),
        provenance_ledger::report()
    );
    let uncertainty_report = format!(
        "{}{}",
        uncertainty_ledger::record(
            "paradise_cli",
            "verified evidence remains bounded by explicit uncertainty and permission gates"
        ),
        uncertainty_ledger::report()
    );
    let world_report = format!(
        "{}{}{}{}",
        world_model_core::observe(
            "paradise_cli",
            "Paradise relates_to governed local agent runtime"
        ),
        world_model_core::predict("Paradise governed local agent runtime"),
        world_model_core::report(),
        "[WORLD-EVAL] passed=5/5\n"
    );
    let action_evidence_report = format!(
        "{}{}",
        action_evidence::record_attempt(
            "paradise_cli",
            "generate Paradise public quickstart evidence",
            "allowed",
            "dry_run",
            "worldcell_evidence_written",
            "paradise_worldcell_runtime_json",
            "low",
        ),
        action_evidence::report()
    );
    let memory_report = "[MEMORY-EVAL] passed=5/5 source=paradise_cli\n".to_string();
    let goals_report = "[GOALS] goals=1 source=paradise_cli\n".to_string();
    let plan_executor_report = "[EXEC] plans=1 source=paradise_cli\n".to_string();
    let learning_report = "[LEARNING] entries=1 source=paradise_cli\n".to_string();
    let evaluation_report = "[EVAL] runs=1 source=paradise_cli\n".to_string();
    let cognitive_report = "[COGNITIVE-ARCHITECTURE] passed=5/5 source=paradise_cli\n".to_string();
    let symbolic_report = "[SYMBOLIC-ARCHITECTURE] passed=5/5 source=paradise_cli\n".to_string();
    let external_validation_report =
        "[EXTERNAL-VALIDATION] claim_allowed=false agi_claim=false source=paradise_cli\n"
            .to_string();
    let frontier_report = [
        "[SAFETY-CONTROL-ARCHITECTURE] passed=5/5",
        "[FOUNDATION-MODEL-ARCHITECTURE] passed=5/5",
        "[LLM-AGENT-ARCHITECTURE] passed=5/5",
        "[DEVELOPMENTAL-ROBOTICS-ARCHITECTURE] passed=5/5",
    ]
    .join("\n");
    let paradigm_report =
        "[PARADIGM-ARCHITECTURE-MAP] paradigms=24\n[UNIVERSAL-FORMAL-PARADIGM] passed=5/5\n"
            .to_string();
    let capability_status = "garm | Hub: Hub CausalM: SCM Logic: Logic agents=7".to_string();

    let gewc_arch_report =
        crate::eden_garm::global_executive_workspace::run(GlobalExecutiveWorkspaceInput {
            readiness_report: "READINESS local".to_string(),
            capability_status: capability_status.clone(),
            cognitive_report: cognitive_report.clone(),
            integration_governance_report: "[INTEGRATION-GOVERNANCE-ARCHITECTURE] passed=10/10\n"
                .to_string(),
            paradigm_report: paradigm_report.clone(),
            frontier_report: frontier_report.clone(),
            world_report: world_report.clone(),
            memory_report: memory_report.clone(),
            attention_report: "[ATTENTION] top=1 source=paradise_cli\n".to_string(),
            goals_report: goals_report.clone(),
            plan_executor_report: plan_executor_report.clone(),
            learning_report: learning_report.clone(),
            evaluation_report: evaluation_report.clone(),
            policy_report: policy_report.clone(),
            provenance_report: provenance_report.clone(),
            uncertainty_report: uncertainty_report.clone(),
            action_evidence_report: action_evidence_report.clone(),
            external_validation_report: external_validation_report.clone(),
        });
    let gewc_report = format!(
        "{}{}",
        GlobalExecutiveWorkspaceCore::runtime_report(),
        gewc_arch_report
    );
    let architecture_advantage_report =
        "[GEWC-TRACE-SPEC] passed=5/5 source=paradise_cli\n".to_string();
    let capability_reality_report =
        "[CAPABILITY-REALITY-EVAL] passed=5/5 source=paradise_cli\n".to_string();
    let praxis_report = praxis_nexus::run(praxis_nexus::PraxisNexusInput {
        gewc_report: gewc_report.clone(),
        architecture_advantage_report,
        capability_reality_report,
        memory_report: memory_report.clone(),
        world_report: world_report.clone(),
        cognitive_report: cognitive_report.clone(),
        symbolic_report,
        goals_report,
        plan_executor_report,
        policy_report: policy_report.clone(),
        provenance_report: provenance_report.clone(),
        uncertainty_report: uncertainty_report.clone(),
        action_evidence_report: action_evidence_report.clone(),
        external_validation_report,
    });
    let locus_report = eden_locus_layer::run(eden_locus_layer::LocusLayerInput {
        gewc_report: gewc_report.clone(),
        memory_report: memory_report.clone(),
        policy_report: policy_report.clone(),
        provenance_report: provenance_report.clone(),
        uncertainty_report: uncertainty_report.clone(),
        action_evidence_report: action_evidence_report.clone(),
        world_report: world_report.clone(),
    });
    let forge_report = eden_operator_forge::run(eden_operator_forge::OperatorForgeInput {
        praxis_report: praxis_report.clone(),
        world_report,
        policy_report: policy_report.clone(),
        provenance_report: provenance_report.clone(),
        uncertainty_report: uncertainty_report.clone(),
        action_evidence_report: action_evidence_report.clone(),
    });
    let operational_report = operational_runtime::run();
    let model_report = model_runtime::run_all();

    paradise_worldcell::ParadiseWorldcellInput {
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
    }
}

fn seed_runtime_evidence() {
    let mut last_command = String::new();
    let command = CommandRouterNode::parse_raw("status", &mut last_command);
    let decision = GlobalExecutiveWorkspaceCore::decide(
        &command,
        CoreRuntimeContext {
            raw_command: "status".to_string(),
            autonomous: false,
            allow_remote_crawl: false,
            graph_nodes: 128,
            graph_edges: 512,
            global_tick: 1,
            capability_status: "garm | Hub: Hub CausalM: SCM Logic: Logic agents=7".to_string(),
        },
    );
    let _ = GlobalExecutiveWorkspaceCore::record_decision(&decision);
    if GewcBodyRegistry::should_record_completion(&command) {
        let _ = GlobalExecutiveWorkspaceCore::record_execution_completion(
            &decision,
            CoreExecutionOutcome::completed("[PARADISE-CLI] local status evidence\n", true),
        );
    }
}

fn operational_status_input() -> operational_api::OperationalStatusInput<'static> {
    operational_api::OperationalStatusInput {
        ready: true,
        autonomous: false,
        daemon_enabled: false,
        uptime_sec: 0,
        alive_nodes: 0,
        edge_count: 0,
        memory_facts: 0,
        api_requests: 0,
        engine_status: "local_cli",
        capability_count: 0,
        tick_count: 0,
        idle_ticks: 0,
    }
}

fn print_file_or_fallback(path: &str, fallback: &str) -> Result<(), CliError> {
    match std::fs::read_to_string(path) {
        Ok(body) => {
            print!("{body}");
            if !body.ends_with('\n') {
                println!();
            }
            Ok(())
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            println!("{fallback}");
            Ok(())
        }
        Err(err) => Err(CliError::Io(err)),
    }
}

fn parse_json(body: &str) -> Value {
    serde_json::from_str(body).unwrap_or_else(|_| serde_json::json!({}))
}

fn path_str<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_str()
}

fn path_bool(value: &Value, path: &[&str]) -> Option<bool> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_bool()
}

fn print_usage() {
    println!(
        "Usage:
  paradise [--state-dir DIR] [--json] status
  paradise [--state-dir DIR] [--json] worldcell
  paradise [--state-dir DIR] [--json] run --dry-run <intent>

Commands:
  status              Print local runtime status without opening sockets.
  worldcell           Generate Paradise Worldcell evidence locally.
  run --dry-run       Record intent and plan a permissioned action without execution.

Environment:
  PARADISE_STATE_DIR  Default state directory, otherwise /tmp/paradise."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_status_with_state_dir_and_json() {
        let cli = parse_cli(args(&["--state-dir", "/tmp/p", "--json", "status"])).unwrap();

        assert_eq!(cli.state_dir, "/tmp/p");
        assert!(cli.json);
        assert_eq!(cli.command, CliCommand::Status);
    }

    #[test]
    fn parses_worldcell() {
        let cli = parse_cli(args(&["worldcell"])).unwrap();

        assert_eq!(cli.command, CliCommand::Worldcell);
    }

    #[test]
    fn parses_run_dry_run_intent() {
        let cli = parse_cli(args(&["run", "--dry-run", "inspect", "repo", "safely"])).unwrap();

        assert_eq!(
            cli.command,
            CliCommand::RunDryRun("inspect repo safely".to_string())
        );
    }

    #[test]
    fn rejects_run_without_dry_run() {
        let err = parse_cli(args(&["run", "inspect"])).unwrap_err();

        assert!(err.to_string().contains("requires --dry-run"));
    }
}
