use crate::sdk::{EdenClient, EdenClientError};
use serde_json::Value;
use std::env;
use std::fmt;
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8080";

pub fn main_entry(args: Vec<String>) {
    match parse_cli(args) {
        Ok(cli) => {
            if let Err(err) = run(cli) {
                eprintln!("edenctl: {err}");
                std::process::exit(1);
            }
        }
        Err(CliError::Help) => {
            print_usage();
        }
        Err(err) => {
            eprintln!("edenctl: {err}");
            eprintln!();
            print_usage();
            std::process::exit(2);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cli {
    base_url: String,
    token: Option<String>,
    json: bool,
    command: CliCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliCommand {
    Status,
    Schemas(Option<String>),
    Permissions(PermissionCommand),
    Recovery(RecoveryCommand),
    Demo(DemoCommand),
    Locus(LocusCommand),
    Forge(ForgeCommand),
    DryRun(String),
    Command { raw: String, wait_sec: Option<u64> },
    EvidenceBundle(EvidenceBundleOptions),
    ApiConformance,
    Doctor,
    Start(StartOptions),
    Stop,
    Verify,
    OpenApiExport(ExportOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PermissionCommand {
    Read,
    Audit,
    Diff,
    History,
    Restore,
    Set { key: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecoveryCommand {
    Read,
    Audit,
    Run,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DemoCommand {
    Read,
    Run,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocusCommand {
    Eval,
    Ingest(String),
    Context(String),
    Audit,
    State,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ForgeCommand {
    Eval,
    Synth(String),
    Verify,
    Audit,
    State,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceBundleOptions {
    state_dir: String,
    output: Option<String>,
    log_file: Option<String>,
    stdout_file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StartOptions {
    port: u16,
    state_dir: String,
    pid_file: String,
    log_file: String,
    stdout_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExportOptions {
    output_dir: String,
}

#[derive(Debug)]
enum CliError {
    Help,
    Message(String),
    Client(EdenClientError),
    Io(std::io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Help => write!(f, "help requested"),
            CliError::Message(message) => write!(f, "{message}"),
            CliError::Client(err) => write!(f, "{err}"),
            CliError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl From<EdenClientError> for CliError {
    fn from(value: EdenClientError) -> Self {
        Self::Client(value)
    }
}

impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

fn parse_cli(args: Vec<String>) -> Result<Cli, CliError> {
    let mut base_url = env::var("EDEN_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let mut token = env::var("EDEN_API_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let mut json = false;
    let mut cursor = 0;
    while cursor < args.len() {
        match args[cursor].as_str() {
            "-h" | "--help" => return Err(CliError::Help),
            "--base-url" => {
                let value = args
                    .get(cursor + 1)
                    .ok_or_else(|| CliError::Message("missing value for --base-url".to_string()))?;
                base_url = value.clone();
                cursor += 2;
            }
            "--json" => {
                json = true;
                cursor += 1;
            }
            "--token" => {
                token = Some(option_value(&args, cursor, "--token")?);
                cursor += 2;
            }
            _ => break,
        }
    }

    let command = parse_command(&args[cursor..])?;
    Ok(Cli {
        base_url,
        token,
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
        "schemas" | "schema" => Ok(CliCommand::Schemas(args.get(1).cloned())),
        "permissions" => Ok(CliCommand::Permissions(parse_permissions(&args[1..])?)),
        "recovery" => Ok(CliCommand::Recovery(parse_recovery(&args[1..])?)),
        "demo" | "demos" => Ok(CliCommand::Demo(parse_demo(&args[1..])?)),
        "locus" => Ok(CliCommand::Locus(parse_locus(&args[1..])?)),
        "forge" | "operator-forge" => Ok(CliCommand::Forge(parse_forge(&args[1..])?)),
        "operator" if args.get(1).map(String::as_str) == Some("forge") => {
            Ok(CliCommand::Forge(parse_forge(&args[2..])?))
        }
        "dry-run" => Ok(CliCommand::DryRun(join_required(
            &args[1..],
            "missing command for dry-run",
        )?)),
        "command" | "cmd" => parse_raw_command(&args[1..]),
        "evidence" => parse_evidence(&args[1..]),
        "api" => parse_api(&args[1..]),
        "doctor" => Ok(CliCommand::Doctor),
        "start" => parse_start(&args[1..]),
        "stop" => Ok(CliCommand::Stop),
        "verify" => Ok(CliCommand::Verify),
        "openapi" => parse_openapi(&args[1..]),
        _ => Err(CliError::Message(format!("unknown command: {command}"))),
    }
}

fn parse_permissions(args: &[String]) -> Result<PermissionCommand, CliError> {
    match args.first().map(String::as_str) {
        None => Ok(PermissionCommand::Read),
        Some("audit") => Ok(PermissionCommand::Audit),
        Some("diff") => Ok(PermissionCommand::Diff),
        Some("history") => Ok(PermissionCommand::History),
        Some("restore") => Ok(PermissionCommand::Restore),
        Some("set") => {
            let key = args
                .get(1)
                .ok_or_else(|| CliError::Message("missing permission key".to_string()))?
                .clone();
            let value = args
                .get(2)
                .ok_or_else(|| CliError::Message("missing permission value".to_string()))?
                .clone();
            if args.len() > 3 {
                return Err(CliError::Message(
                    "permissions set accepts exactly <key> <allow|deny>".to_string(),
                ));
            }
            Ok(PermissionCommand::Set { key, value })
        }
        Some(other) => Err(CliError::Message(format!(
            "unknown permissions command: {other}"
        ))),
    }
}

fn parse_recovery(args: &[String]) -> Result<RecoveryCommand, CliError> {
    match args.first().map(String::as_str) {
        None => Ok(RecoveryCommand::Read),
        Some("audit") => Ok(RecoveryCommand::Audit),
        Some("run") => Ok(RecoveryCommand::Run),
        Some(other) => Err(CliError::Message(format!(
            "unknown recovery command: {other}"
        ))),
    }
}

fn parse_demo(args: &[String]) -> Result<DemoCommand, CliError> {
    match args.first().map(String::as_str) {
        None => Ok(DemoCommand::Read),
        Some("run") => Ok(DemoCommand::Run),
        Some(other) => Err(CliError::Message(format!("unknown demo command: {other}"))),
    }
}

fn parse_locus(args: &[String]) -> Result<LocusCommand, CliError> {
    match args.first().map(String::as_str) {
        None | Some("eval") => Ok(LocusCommand::Eval),
        Some("ingest") => Ok(LocusCommand::Ingest(join_required(
            &args[1..],
            "missing context for locus ingest",
        )?)),
        Some("context") => Ok(LocusCommand::Context(join_required(
            &args[1..],
            "missing query for locus context",
        )?)),
        Some("audit") => Ok(LocusCommand::Audit),
        Some("state") => Ok(LocusCommand::State),
        Some(other) => Err(CliError::Message(format!("unknown locus command: {other}"))),
    }
}

fn parse_forge(args: &[String]) -> Result<ForgeCommand, CliError> {
    match args.first().map(String::as_str) {
        None | Some("eval") => Ok(ForgeCommand::Eval),
        Some("synth") | Some("synthesize") => Ok(ForgeCommand::Synth(join_required(
            &args[1..],
            "missing goal for forge synth",
        )?)),
        Some("verify") => Ok(ForgeCommand::Verify),
        Some("audit") => Ok(ForgeCommand::Audit),
        Some("state") => Ok(ForgeCommand::State),
        Some(other) => Err(CliError::Message(format!("unknown forge command: {other}"))),
    }
}

fn parse_evidence(args: &[String]) -> Result<CliCommand, CliError> {
    if args.first().map(String::as_str) != Some("bundle") {
        return Err(CliError::Message(
            "expected evidence bundle subcommand".to_string(),
        ));
    }
    let mut state_dir = env::var("GARM_EVIDENCE_STATE_DIR")
        .or_else(|_| env::var("GARM_BLACKBOX_STATE_DIR"))
        .unwrap_or_else(|_| "/tmp/eden_garm_blackbox".to_string());
    let mut output = env::var("GARM_EVIDENCE_OUTPUT").ok();
    let mut log_file = env::var("GARM_EVIDENCE_LOG_FILE")
        .or_else(|_| env::var("GARM_BLACKBOX_LOG_FILE"))
        .ok();
    let mut stdout_file = env::var("GARM_EVIDENCE_STDOUT_FILE")
        .or_else(|_| env::var("GARM_BLACKBOX_STDOUT_FILE"))
        .ok();
    let mut cursor = 1;
    while cursor < args.len() {
        match args[cursor].as_str() {
            "--state-dir" => {
                state_dir = option_value(args, cursor, "--state-dir")?;
                cursor += 2;
            }
            "--output" => {
                output = Some(option_value(args, cursor, "--output")?);
                cursor += 2;
            }
            "--log-file" => {
                log_file = Some(option_value(args, cursor, "--log-file")?);
                cursor += 2;
            }
            "--stdout-file" => {
                stdout_file = Some(option_value(args, cursor, "--stdout-file")?);
                cursor += 2;
            }
            other => {
                return Err(CliError::Message(format!(
                    "unknown evidence bundle option: {other}"
                )));
            }
        }
    }

    Ok(CliCommand::EvidenceBundle(EvidenceBundleOptions {
        state_dir,
        output,
        log_file,
        stdout_file,
    }))
}

fn parse_api(args: &[String]) -> Result<CliCommand, CliError> {
    match args.first().map(String::as_str) {
        Some("conformance") => Ok(CliCommand::ApiConformance),
        _ => Err(CliError::Message(
            "expected api conformance subcommand".to_string(),
        )),
    }
}

fn parse_start(args: &[String]) -> Result<CliCommand, CliError> {
    let mut options = StartOptions {
        port: env::var("EDEN_API_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080),
        state_dir: env::var("EDEN_STATE_DIR").unwrap_or_else(|_| "/tmp/eden_garm".to_string()),
        pid_file: env::var("EDEN_PID_FILE").unwrap_or_else(|_| "/tmp/eden_garm.pid".to_string()),
        log_file: env::var("EDEN_LOG_FILE").unwrap_or_else(|_| "/tmp/eden_garm.log".to_string()),
        stdout_file: env::var("EDEN_STDOUT_FILE")
            .unwrap_or_else(|_| "/tmp/eden_garm.stdout".to_string()),
    };
    let mut cursor = 0;
    while cursor < args.len() {
        match args[cursor].as_str() {
            "--port" => {
                options.port = option_value(args, cursor, "--port")?
                    .parse::<u16>()
                    .map_err(|_| CliError::Message("invalid --port value".to_string()))?;
                cursor += 2;
            }
            "--state-dir" => {
                options.state_dir = option_value(args, cursor, "--state-dir")?;
                cursor += 2;
            }
            "--pid-file" => {
                options.pid_file = option_value(args, cursor, "--pid-file")?;
                cursor += 2;
            }
            "--log-file" => {
                options.log_file = option_value(args, cursor, "--log-file")?;
                cursor += 2;
            }
            "--stdout-file" => {
                options.stdout_file = option_value(args, cursor, "--stdout-file")?;
                cursor += 2;
            }
            other => {
                return Err(CliError::Message(format!("unknown start option: {other}")));
            }
        }
    }
    Ok(CliCommand::Start(options))
}

fn parse_openapi(args: &[String]) -> Result<CliCommand, CliError> {
    if args.first().map(String::as_str) != Some("export") {
        return Err(CliError::Message(
            "expected openapi export subcommand".to_string(),
        ));
    }
    let mut output_dir =
        env::var("EDEN_OPENAPI_DIR").unwrap_or_else(|_| "contracts/v1/openapi".to_string());
    let mut cursor = 1;
    while cursor < args.len() {
        match args[cursor].as_str() {
            "--output-dir" => {
                output_dir = option_value(args, cursor, "--output-dir")?;
                cursor += 2;
            }
            other => {
                return Err(CliError::Message(format!(
                    "unknown openapi export option: {other}"
                )));
            }
        }
    }
    Ok(CliCommand::OpenApiExport(ExportOptions { output_dir }))
}

fn parse_raw_command(args: &[String]) -> Result<CliCommand, CliError> {
    let mut wait_sec = None;
    let mut cursor = 0;
    if args.first().map(String::as_str) == Some("--wait-sec") {
        let raw_wait = args
            .get(1)
            .ok_or_else(|| CliError::Message("missing value for --wait-sec".to_string()))?;
        wait_sec = Some(
            raw_wait
                .parse::<u64>()
                .map_err(|_| CliError::Message("invalid --wait-sec value".to_string()))?,
        );
        cursor = 2;
    }
    Ok(CliCommand::Command {
        raw: join_required(&args[cursor..], "missing raw command")?,
        wait_sec,
    })
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
    match cli.command {
        CliCommand::EvidenceBundle(options) => run_evidence_bundle(options),
        CliCommand::ApiConformance => run_api_conformance(),
        CliCommand::Start(options) => run_start(options),
        CliCommand::Verify => run_verify(),
        command => {
            let mut client = EdenClient::new(cli.base_url)?.with_timeout(Duration::from_secs(20));
            if let Some(token) = cli.token {
                client = client.with_api_token(token);
            }
            run_http_command(&client, command, cli.json)
        }
    }
}

fn run_http_command(
    client: &EdenClient,
    command: CliCommand,
    json_output: bool,
) -> Result<(), CliError> {
    match command {
        CliCommand::Status => {
            let status = client.operational_status()?;
            if json_output {
                print_json(&status);
            } else {
                print_status(&status);
            }
        }
        CliCommand::Schemas(name) => {
            let schemas = match name {
                Some(name) => client.operational_schema(&name)?,
                None => client.operational_schemas()?,
            };
            print_json(&schemas);
        }
        CliCommand::Permissions(command) => run_permission_command(client, command)?,
        CliCommand::Recovery(command) => run_recovery_command(client, command)?,
        CliCommand::Demo(command) => run_demo_command(client, command)?,
        CliCommand::Locus(command) => run_locus_command(client, command)?,
        CliCommand::Forge(command) => run_forge_command(client, command)?,
        CliCommand::DryRun(raw) => print_json(&client.action_dry_run(&raw)?),
        CliCommand::Doctor => run_doctor(client)?,
        CliCommand::Stop => print!("{}", client.queue_command("quit")?),
        CliCommand::OpenApiExport(options) => run_openapi_export(client, options)?,
        CliCommand::Command { raw, wait_sec } => {
            if let Some(wait_sec) = wait_sec {
                run_waited_command(client, &raw, Duration::from_secs(wait_sec))?;
            } else {
                print!("{}", client.run_command_sync(&raw)?);
            }
        }
        CliCommand::EvidenceBundle(_)
        | CliCommand::ApiConformance
        | CliCommand::Start(_)
        | CliCommand::Verify => unreachable!(),
    }
    Ok(())
}

fn run_waited_command(client: &EdenClient, raw: &str, timeout: Duration) -> Result<(), CliError> {
    let queued = client.queue_command(raw)?;
    let id = parse_queued_id(&queued)?;
    let started = Instant::now();
    while started.elapsed() < timeout {
        let response = client.command_result(id)?;
        match response.status_code {
            200 => {
                print!("{}", response.body);
                let _ = client.forget_command(id);
                return Ok(());
            }
            202 => thread::sleep(Duration::from_millis(250)),
            status => {
                return Err(CliError::Message(format!(
                    "command {id} returned unexpected status {status}: {}",
                    response.body
                )));
            }
        }
    }
    Err(CliError::Message(format!(
        "timeout waiting {timeout:?} for command id {id}: {raw}"
    )))
}

fn parse_queued_id(response: &str) -> Result<u64, CliError> {
    let id = response
        .strip_prefix("queued: ")
        .and_then(|rest| rest.split_whitespace().next())
        .ok_or_else(|| CliError::Message(format!("could not parse queued id from: {response}")))?;
    id.parse::<u64>()
        .map_err(|_| CliError::Message(format!("invalid queued id in: {response}")))
}

fn run_permission_command(client: &EdenClient, command: PermissionCommand) -> Result<(), CliError> {
    match command {
        PermissionCommand::Read => print_json(&client.operational_permissions()?),
        PermissionCommand::Audit => print!(
            "{}",
            client.run_command_sync("operational permissions audit")?
        ),
        PermissionCommand::Diff => print!(
            "{}",
            client.run_command_sync("operational permissions diff")?
        ),
        PermissionCommand::History => {
            print!(
                "{}",
                client.run_command_sync("operational permissions history")?
            )
        }
        PermissionCommand::Restore => {
            print!(
                "{}",
                client.run_command_sync("operational permissions restore")?
            )
        }
        PermissionCommand::Set { key, value } => {
            let raw = format!("operational permissions set {key} {value}");
            print!("{}", client.run_command_sync(&raw)?);
        }
    }
    Ok(())
}

fn run_recovery_command(client: &EdenClient, command: RecoveryCommand) -> Result<(), CliError> {
    match command {
        RecoveryCommand::Read => print_json(&client.operational_recovery()?),
        RecoveryCommand::Audit => {
            print!("{}", client.run_command_sync("operational recovery audit")?)
        }
        RecoveryCommand::Run => print!("{}", client.run_command_sync("operational recovery run")?),
    }
    Ok(())
}

fn run_demo_command(client: &EdenClient, command: DemoCommand) -> Result<(), CliError> {
    match command {
        DemoCommand::Read => print_json(&client.operational_demos()?),
        DemoCommand::Run => print!("{}", client.run_command_sync("operational demo run")?),
    }
    Ok(())
}

fn run_locus_command(client: &EdenClient, command: LocusCommand) -> Result<(), CliError> {
    match command {
        LocusCommand::Eval => print!("{}", client.locus_eval()?),
        LocusCommand::Ingest(spec) => print!("{}", client.locus_ingest(&spec)?),
        LocusCommand::Context(query) => print!("{}", client.locus_context(&query)?),
        LocusCommand::Audit => print!("{}", client.locus_audit()?),
        LocusCommand::State => print_response_body(client.locus_state()?)?,
    }
    Ok(())
}

fn run_forge_command(client: &EdenClient, command: ForgeCommand) -> Result<(), CliError> {
    match command {
        ForgeCommand::Eval => print!("{}", client.operator_forge_eval()?),
        ForgeCommand::Synth(goal) => print!("{}", client.operator_forge_synth(&goal)?),
        ForgeCommand::Verify => print!("{}", client.operator_forge_verify()?),
        ForgeCommand::Audit => print!("{}", client.operator_forge_audit()?),
        ForgeCommand::State => print_response_body(client.operator_forge_state()?)?,
    }
    Ok(())
}

fn print_response_body(response: crate::sdk::EdenHttpResponse) -> Result<(), CliError> {
    if response.is_success() {
        print!("{}", response.body);
        Ok(())
    } else {
        Err(CliError::Message(format!(
            "HTTP {} {}: {}",
            response.status_code, response.status_text, response.body
        )))
    }
}

fn run_doctor(client: &EdenClient) -> Result<(), CliError> {
    let health = client.health()?;
    let ready = client.ready()?;
    let status = client.operational_status()?;
    println!(
        "[EDENCTL-DOCTOR] health={} ready={} operational_state={}",
        path_str(&health, &["status"]).unwrap_or("unknown"),
        ready.trim(),
        path_str(&status, &["state"]).unwrap_or("unknown")
    );
    Ok(())
}

fn run_openapi_export(client: &EdenClient, options: ExportOptions) -> Result<(), CliError> {
    std::fs::create_dir_all(&options.output_dir)?;
    write_json_file(
        &Path::new(&options.output_dir).join("runtime.openapi.json"),
        &client.runtime_openapi()?,
    )?;
    write_json_file(
        &Path::new(&options.output_dir).join("operational.openapi.json"),
        &client.operational_openapi()?,
    )?;
    println!("[EDENCTL-OPENAPI-EXPORT] output_dir={}", options.output_dir);
    Ok(())
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), CliError> {
    let body = serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(path, body)?;
    Ok(())
}

fn print_status(status: &Value) {
    let state = path_str(status, &["state"]).unwrap_or("unknown");
    let ready = path_bool(status, &["health", "ready"]).unwrap_or(false);
    let autonomous = path_bool(status, &["health", "autonomous"]).unwrap_or(false);
    let uptime_sec = path_u64(status, &["health", "uptime_sec"]).unwrap_or(0);
    let latest_decision = path_str(status, &["latest", "decision_id"]).unwrap_or("none");
    let permission_count = status
        .pointer("/permissions/permissions")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    println!(
        "[EDENCTL-STATUS] state={state} ready={ready} autonomous={autonomous} uptime_sec={uptime_sec} latest_decision={latest_decision} permissions={permission_count}"
    );
}

fn print_json(value: &Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
    );
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

fn path_u64(value: &Value, path: &[&str]) -> Option<u64> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_u64()
}

fn run_evidence_bundle(options: EvidenceBundleOptions) -> Result<(), CliError> {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("garm")
        .join("scripts")
        .join("operational_evidence_bundle.sh");
    let mut command = ProcessCommand::new(script);
    command.arg("--state-dir").arg(options.state_dir);
    if let Some(output) = options.output {
        command.arg("--output").arg(output);
    }
    if let Some(log_file) = options.log_file {
        command.arg("--log-file").arg(log_file);
    }
    if let Some(stdout_file) = options.stdout_file {
        command.arg("--stdout-file").arg(stdout_file);
    }
    run_process(command)
}

fn run_api_conformance() -> Result<(), CliError> {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("garm")
        .join("scripts")
        .join("conformance_api.sh");
    run_process(ProcessCommand::new(script))
}

fn run_start(options: StartOptions) -> Result<(), CliError> {
    let exe = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("target")
        .join("debug")
        .join("eden-garm");
    if !exe.exists() {
        return Err(CliError::Message(format!(
            "missing runtime binary {}; run cargo build --bin eden-garm -p eden_core",
            exe.to_string_lossy()
        )));
    }
    std::fs::create_dir_all(&options.state_dir)?;
    ensure_parent_dir(&options.pid_file)?;
    ensure_parent_dir(&options.log_file)?;
    ensure_parent_dir(&options.stdout_file)?;
    let stdout = std::fs::File::create(&options.stdout_file)?;
    let stderr = stdout.try_clone()?;
    let child = ProcessCommand::new(exe)
        .arg("--daemon")
        .arg("--api-port")
        .arg(options.port.to_string())
        .arg("--pid-file")
        .arg(&options.pid_file)
        .arg("--log-file")
        .arg(&options.log_file)
        .arg("--state-dir")
        .arg(&options.state_dir)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()?;
    println!(
        "[EDENCTL-START] pid={} base_url=http://127.0.0.1:{} state_dir={} log_file={} stdout_file={}",
        child.id(),
        options.port,
        options.state_dir,
        options.log_file,
        options.stdout_file
    );
    Ok(())
}

fn run_verify() -> Result<(), CliError> {
    let mut command = ProcessCommand::new("make");
    command.arg("long-run-stability");
    run_process(command)
}

fn ensure_parent_dir(path: &str) -> Result<(), CliError> {
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn run_process(mut command: ProcessCommand) -> Result<(), CliError> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(CliError::Message(format!(
            "subprocess exited with status {status}"
        )))
    }
}

fn print_usage() {
    println!(
        "Usage:
  edenctl [--base-url URL] [--json] status
  edenctl [--base-url URL] schemas [name]
  edenctl [--base-url URL] permissions [audit|diff|history|restore|set <key> <allow|deny>]
  edenctl [--base-url URL] recovery [audit|run]
  edenctl [--base-url URL] demo [run]
  edenctl [--base-url URL] locus [eval|ingest <text>|context <query>|audit|state]
  edenctl [--base-url URL] forge [eval|synth <goal>|verify|audit|state]
  edenctl [--base-url URL] operator forge [eval|synth <goal>|verify|audit|state]
  edenctl [--base-url URL] dry-run <command>
  edenctl [--base-url URL] command [--wait-sec N] <raw command>
  edenctl [--base-url URL] doctor
  edenctl [--base-url URL] stop
  edenctl [--base-url URL] openapi export [--output-dir DIR]
  edenctl start [--port N] [--state-dir DIR] [--pid-file FILE] [--log-file FILE] [--stdout-file FILE]
  edenctl verify
  edenctl evidence bundle [--state-dir DIR] [--output FILE] [--log-file FILE] [--stdout-file FILE]
  edenctl api conformance

Global options:
  --base-url URL    Local EDEN API URL (default http://127.0.0.1:8080)
  --token TOKEN     Optional local API token, also read from EDEN_API_TOKEN
  --json            Print supported command output as JSON"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn without_api_token<T>(test: impl FnOnce() -> T) -> T {
        let previous = std::env::var("EDEN_API_TOKEN").ok();
        std::env::remove_var("EDEN_API_TOKEN");
        let result = test();
        if let Some(token) = previous {
            std::env::set_var("EDEN_API_TOKEN", token);
        }
        result
    }

    #[test]
    fn parses_status_with_base_url_and_json() {
        let cli = without_api_token(|| {
            parse_cli(args(&[
                "--base-url",
                "http://127.0.0.1:8114",
                "--json",
                "status",
            ]))
            .unwrap()
        });

        assert_eq!(cli.base_url, "http://127.0.0.1:8114");
        assert_eq!(cli.token, None);
        assert!(cli.json);
        assert_eq!(cli.command, CliCommand::Status);
    }

    #[test]
    fn parses_token_option() {
        let cli = parse_cli(args(&["--token", "secret", "status"])).unwrap();

        assert_eq!(cli.token.as_deref(), Some("secret"));
    }

    #[test]
    fn parses_permission_set() {
        let cli = parse_cli(args(&["permissions", "set", "remote_network", "deny"])).unwrap();

        assert_eq!(
            cli.command,
            CliCommand::Permissions(PermissionCommand::Set {
                key: "remote_network".to_string(),
                value: "deny".to_string()
            })
        );
    }

    #[test]
    fn parses_joined_raw_command() {
        let cli = parse_cli(args(&["command", "operational", "demo", "run"])).unwrap();

        assert_eq!(
            cli.command,
            CliCommand::Command {
                raw: "operational demo run".to_string(),
                wait_sec: None
            }
        );
    }

    #[test]
    fn parses_waited_raw_command() {
        let cli = parse_cli(args(&[
            "command",
            "--wait-sec",
            "120",
            "readiness",
            "package",
        ]))
        .unwrap();

        assert_eq!(
            cli.command,
            CliCommand::Command {
                raw: "readiness package".to_string(),
                wait_sec: Some(120)
            }
        );
    }

    #[test]
    fn parses_queued_command_id() {
        assert_eq!(
            parse_queued_id("queued: 42 cmd: readiness package\n").unwrap(),
            42
        );
    }

    #[test]
    fn parses_start_options() {
        let cli = parse_cli(args(&[
            "start",
            "--port",
            "8118",
            "--state-dir",
            "/tmp/eden_start",
        ]))
        .unwrap();

        match cli.command {
            CliCommand::Start(options) => {
                assert_eq!(options.port, 8118);
                assert_eq!(options.state_dir, "/tmp/eden_start");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_openapi_export() {
        let cli = parse_cli(args(&[
            "openapi",
            "export",
            "--output-dir",
            "/tmp/contracts",
        ]))
        .unwrap();

        assert_eq!(
            cli.command,
            CliCommand::OpenApiExport(ExportOptions {
                output_dir: "/tmp/contracts".to_string()
            })
        );
    }

    #[test]
    fn parses_locus_commands() {
        let eval = parse_cli(args(&["locus"])).unwrap();
        assert_eq!(eval.command, CliCommand::Locus(LocusCommand::Eval));

        let ingest = parse_cli(args(&["locus", "ingest", "operator", "preference"])).unwrap();
        assert_eq!(
            ingest.command,
            CliCommand::Locus(LocusCommand::Ingest("operator preference".to_string()))
        );

        let context = parse_cli(args(&["locus", "context", "permission", "boundary"])).unwrap();
        assert_eq!(
            context.command,
            CliCommand::Locus(LocusCommand::Context("permission boundary".to_string()))
        );
    }

    #[test]
    fn parses_forge_commands() {
        let eval = parse_cli(args(&["forge", "eval"])).unwrap();
        assert_eq!(eval.command, CliCommand::Forge(ForgeCommand::Eval));

        let synth = parse_cli(args(&["forge", "synth", "causal", "risk"])).unwrap();
        assert_eq!(
            synth.command,
            CliCommand::Forge(ForgeCommand::Synth("causal risk".to_string()))
        );

        let verify = parse_cli(args(&["operator-forge", "verify"])).unwrap();
        assert_eq!(verify.command, CliCommand::Forge(ForgeCommand::Verify));

        let operator_forge = parse_cli(args(&["operator", "forge", "audit"])).unwrap();
        assert_eq!(
            operator_forge.command,
            CliCommand::Forge(ForgeCommand::Audit)
        );
    }

    #[test]
    fn parses_evidence_bundle_options() {
        let cli = parse_cli(args(&[
            "evidence",
            "bundle",
            "--state-dir",
            "/tmp/eden",
            "--output",
            "/tmp/eden/bundle.json",
        ]))
        .unwrap();

        assert_eq!(
            cli.command,
            CliCommand::EvidenceBundle(EvidenceBundleOptions {
                state_dir: "/tmp/eden".to_string(),
                output: Some("/tmp/eden/bundle.json".to_string()),
                log_file: None,
                stdout_file: None
            })
        );
    }
}
