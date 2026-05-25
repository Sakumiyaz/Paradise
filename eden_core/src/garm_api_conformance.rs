//! External API conformance runner for EDEN GARM.
//!
//! The runner consumes a live EDEN HTTP endpoint through the client SDK and
//! verifies that API contracts, GEWC-routed command helpers, dry-run action
//! classification, and no-claim policy markers are externally observable.

use crate::sdk::{EdenClient, EdenClientError};
use serde_json::Value;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Default)]
struct ConformanceReport {
    base_url: String,
    checks: Vec<Value>,
    failures: Vec<String>,
}

impl ConformanceReport {
    fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            checks: Vec::new(),
            failures: Vec::new(),
        }
    }

    fn pass(&mut self, name: &str, evidence: impl Into<String>) {
        self.checks.push(serde_json::json!({
            "name": name,
            "passed": true,
            "evidence": evidence.into(),
        }));
    }

    fn fail(&mut self, name: &str, evidence: impl Into<String>) {
        let evidence = evidence.into();
        self.failures.push(format!("{name}: {evidence}"));
        self.checks.push(serde_json::json!({
            "name": name,
            "passed": false,
            "evidence": evidence,
        }));
    }

    fn ok(&self) -> bool {
        self.failures.is_empty()
    }

    fn as_json(&self) -> Value {
        serde_json::json!({
            "schema": "eden-sdk-conformance-report-v1",
            "status": if self.ok() { "passed" } else { "failed" },
            "base_url": self.base_url,
            "generated_at_unix": unix_now(),
            "claim_allowed": false,
            "agi_claim": false,
            "checks": self.checks,
            "failures": self.failures,
        })
    }
}

pub fn main_entry() {
    let args = Args::parse();
    let client = match EdenClient::new(&args.base_url) {
        Ok(client) => client.with_timeout(Duration::from_secs(args.timeout_sec)),
        Err(err) => {
            eprintln!("[EDEN-SDK-CONFORMANCE] status=failed error={err}");
            std::process::exit(2);
        }
    };

    let report = run_conformance(&client);
    if let Some(path) = args.report_path {
        let body =
            serde_json::to_string_pretty(&report.as_json()).unwrap_or_else(|_| "{}".to_string());
        if let Err(err) = std::fs::write(&path, body) {
            eprintln!(
                "[EDEN-SDK-CONFORMANCE] status=failed error=failed_to_write_report path={} err={}",
                path.to_string_lossy(),
                err
            );
            std::process::exit(2);
        }
    }

    println!(
        "[EDEN-SDK-CONFORMANCE] status={} checks={} failures={} claim_allowed=false base_url={}",
        if report.ok() { "passed" } else { "failed" },
        report.checks.len(),
        report.failures.len(),
        client.base_url()
    );
    for failure in &report.failures {
        println!("- {failure}");
    }
    if !report.ok() {
        std::process::exit(1);
    }
}

fn run_conformance(client: &EdenClient) -> ConformanceReport {
    let mut report = ConformanceReport::new(client.base_url());
    check_operator_console(client, &mut report);
    check_health_and_ready(client, &mut report);
    check_runtime_api(client, &mut report);
    check_artifact_api(client, &mut report);
    check_operational_api(client, &mut report);
    check_locus_and_operator_apis(client, &mut report);
    check_capability_and_gewc_api(client, &mut report);
    check_validation_and_action_api(client, &mut report);
    report
}

fn check_operator_console(client: &EdenClient, report: &mut ConformanceReport) {
    match client.get("/") {
        Ok(response)
            if response.status_code == 200
                && response.content_type.contains("text/html")
                && response.body.contains("EDEN Operator Console") =>
        {
            report.pass("operator_console", "root serves HTML console");
        }
        Ok(response) => report.fail(
            "operator_console",
            format!(
                "status={} content_type={} body_len={}",
                response.status_code,
                response.content_type,
                response.body.len()
            ),
        ),
        Err(err) => report.fail("operator_console", err.to_string()),
    }
}

fn check_health_and_ready(client: &EdenClient, report: &mut ConformanceReport) {
    match client.health() {
        Ok(value) if value.get("status").and_then(Value::as_str) == Some("ok") => {
            report.pass("health_endpoint", "status=ok");
        }
        Ok(value) => report.fail("health_endpoint", format!("unexpected body={value}")),
        Err(err) => report.fail("health_endpoint", err.to_string()),
    }
    match client.ready() {
        Ok(body) if body.contains("ready") => report.pass("ready_endpoint", "ready"),
        Ok(body) => report.fail("ready_endpoint", format!("unexpected body={body:?}")),
        Err(err) => report.fail("ready_endpoint", err.to_string()),
    }
}

fn check_runtime_api(client: &EdenClient, report: &mut ConformanceReport) {
    if let Some(catalog) = json_check(
        report,
        "runtime_catalog",
        client.runtime_catalog(),
        "eden-runtime-state-api-catalog-v1",
    ) {
        assert_claim_blocked(report, "runtime_catalog_policy", &catalog);
        assert_array_min(report, "runtime_catalog_records", &catalog, "records", 30);
        assert_contains_json(
            report,
            "runtime_catalog_state_route",
            &catalog,
            "/api/runtime/state?name=<state_name>",
        );
    }
    if let Some(openapi) = json_check(
        report,
        "runtime_openapi",
        client.runtime_openapi(),
        "eden-runtime-state-openapi-v1",
    ) {
        assert_claim_blocked(report, "runtime_openapi_policy", &openapi);
        assert_contains_json(
            report,
            "runtime_openapi_read_state",
            &openapi,
            "readRuntimeStateByName",
        );
    }
    if let Some(snapshot) = json_check(
        report,
        "runtime_snapshot",
        client.runtime_snapshot(),
        "eden-runtime-state-snapshot-v1",
    ) {
        assert_claim_blocked(report, "runtime_snapshot_policy", &snapshot);
        assert_pointer_bool(
            report,
            "runtime_snapshot_read_only",
            &snapshot,
            "/permissions/read_only",
            true,
        );
    }
    match client.runtime_state("../secret") {
        Ok(response) if response.status_code == 404 => {
            report.pass("runtime_state_path_whitelist", "path traversal rejected");
        }
        Ok(response) => report.fail(
            "runtime_state_path_whitelist",
            format!("expected 404 got {}", response.status_code),
        ),
        Err(err) => report.fail("runtime_state_path_whitelist", err.to_string()),
    }
}

fn check_artifact_api(client: &EdenClient, report: &mut ConformanceReport) {
    if let Some(catalog) = json_check(
        report,
        "artifact_catalog",
        client.artifact_catalog(),
        "eden-artifact-api-catalog-v1",
    ) {
        assert_claim_blocked(report, "artifact_catalog_policy", &catalog);
        assert_array_min(report, "artifact_catalog_records", &catalog, "records", 90);
        assert_contains_json(
            report,
            "artifact_catalog_read_route",
            &catalog,
            "/api/artifact?name=",
        );
    }
    if let Some(runtime) = json_check(
        report,
        "artifact_runtime",
        client.artifact_runtime(),
        "eden-artifact-api-runtime-v1",
    ) {
        assert_claim_blocked(report, "artifact_runtime_policy", &runtime);
    }
    match client.artifact("../secret") {
        Ok(response) if response.status_code == 404 => {
            report.pass("artifact_path_whitelist", "unknown artifact rejected");
        }
        Ok(response) => report.fail(
            "artifact_path_whitelist",
            format!("expected 404 got {}", response.status_code),
        ),
        Err(err) => report.fail("artifact_path_whitelist", err.to_string()),
    }
}

fn check_operational_api(client: &EdenClient, report: &mut ConformanceReport) {
    if let Some(catalog) = json_check(
        report,
        "operational_catalog",
        client.operational_catalog(),
        "eden-operational-api-catalog-v1",
    ) {
        assert_claim_blocked(report, "operational_catalog_policy", &catalog);
        assert_array_min(
            report,
            "operational_catalog_records",
            &catalog,
            "records",
            10,
        );
        assert_all_operational_records_read_only(report, &catalog);
    }
    if let Some(openapi) = json_check(
        report,
        "operational_openapi",
        client.operational_openapi(),
        "eden-operational-openapi-v1",
    ) {
        assert_claim_blocked(report, "operational_openapi_policy", &openapi);
        assert_contains_json(
            report,
            "operational_openapi_dry_run",
            &openapi,
            "dryRunActionCommand",
        );
    }
    if let Some(runtime) = json_check(
        report,
        "operational_runtime",
        client.operational_runtime(),
        "eden-operational-api-runtime-v1",
    ) {
        assert_claim_blocked(report, "operational_runtime_policy", &runtime);
        assert_pointer_bool(
            report,
            "operational_runtime_no_action_mutation",
            &runtime,
            "/action_mutation_allowed",
            false,
        );
    }
    if let Some(status) = json_check(
        report,
        "operational_status",
        client.operational_status(),
        "eden-operational-status-v1",
    ) {
        assert_claim_blocked(report, "operational_status_policy", &status);
        assert_contains_json(
            report,
            "operational_status_permissions_endpoint",
            &status,
            "/api/operational/permissions",
        );
        assert_contains_json(
            report,
            "operational_status_replay_endpoint",
            &status,
            "/api/operational/replay?decision_id=<id>",
        );
    }
    if let Some(contract) = json_check(
        report,
        "operational_contract",
        client.operational_contract(),
        "eden-operational-contract-v1",
    ) {
        assert_claim_blocked(report, "operational_contract_policy", &contract);
        assert_contains_json(
            report,
            "operational_contract_degraded_state",
            &contract,
            "\"degraded\"",
        );
        assert_contains_json(
            report,
            "operational_contract_replay",
            &contract,
            "operational replay run",
        );
    }
    if let Some(permissions) = json_check(
        report,
        "operational_permissions",
        client.operational_permissions(),
        "eden-operational-permissions-v1",
    ) {
        assert_claim_blocked(report, "operational_permissions_policy", &permissions);
        assert_contains_json(
            report,
            "operational_permissions_remote_network",
            &permissions,
            "remote_network",
        );
    }
    if let Some(replay) = json_check(
        report,
        "operational_replay",
        client.operational_replay(),
        "eden-gewc-replay-index-v1",
    ) {
        assert_claim_blocked(report, "operational_replay_policy", &replay);
        assert_contains_json(
            report,
            "operational_replay_read_decision",
            &replay,
            "/api/operational/replay?decision_id=<id>",
        );
    }
    if let Some(replay_missing) = json_check(
        report,
        "operational_replay_missing_decision",
        client.operational_replay_decision("missing"),
        "eden-gewc-decision-replay-v1",
    ) {
        assert_pointer_bool(
            report,
            "operational_replay_missing_found_false",
            &replay_missing,
            "/found",
            false,
        );
    }
    if let Some(recovery) = json_check(
        report,
        "operational_recovery",
        client.operational_recovery(),
        "eden-operational-recovery-plan-v1",
    ) {
        assert_claim_blocked(report, "operational_recovery_policy", &recovery);
        assert_contains_json(
            report,
            "operational_recovery_authority",
            &recovery,
            "global_executive_workspace_core",
        );
    }
    if let Some(demos) = json_check(
        report,
        "operational_demos",
        client.operational_demos(),
        "eden-operational-demo-suite-v1",
    ) {
        assert_claim_blocked(report, "operational_demos_policy", &demos);
        assert_contains_json(
            report,
            "operational_demos_memory",
            &demos,
            "memory_planning",
        );
    }
    if let Some(schemas) = json_check(
        report,
        "operational_schema_registry",
        client.operational_schemas(),
        "eden-schema-registry-v1",
    ) {
        assert_claim_blocked(report, "operational_schema_registry_policy", &schemas);
        assert_contains_json(
            report,
            "operational_schema_registry_status",
            &schemas,
            "eden-operational-status-v1",
        );
    }
    if let Some(schema_record) = json_check(
        report,
        "operational_schema_record",
        client.operational_schema("operational_status"),
        "eden-schema-registry-record-v1",
    ) {
        assert_pointer_bool(
            report,
            "operational_schema_record_found",
            &schema_record,
            "/found",
            true,
        );
    }
}

fn check_locus_and_operator_apis(client: &EdenClient, report: &mut ConformanceReport) {
    text_check(
        report,
        "locus_eval_command",
        client.locus_eval(),
        "[EDEN-LOCUS-LAYER]",
    );
    text_check(
        report,
        "locus_ingest_command",
        client.locus_ingest("operator preference :: sdk conformance context remains governed"),
        "[LOCUS-INGEST]",
    );
    text_check(
        report,
        "locus_context_command",
        client.locus_context("sdk conformance permission boundary"),
        "[LOCUS-CONTEXT]",
    );
    text_check(
        report,
        "operator_forge_eval_command",
        client.operator_forge_eval(),
        "[EDEN-OPERATOR-FORGE]",
    );
    text_check(
        report,
        "operator_forge_synth_command",
        client.operator_forge_synth("causal risk model for sdk conformance"),
        "[OPERATOR-FORGE-SYNTH]",
    );
    text_check(
        report,
        "operator_forge_verify_command",
        client.operator_forge_verify(),
        "[OPERATOR-FORGE-VERIFY]",
    );
    text_check(
        report,
        "locus_operator_bridge_runtime_command",
        client.run_command_sync("operational runtime eval"),
        "[OPERATIONAL-RUNTIME-PHASE]",
    );
    text_check(
        report,
        "runtime_state_api_refresh_after_locus_operator",
        client.run_command_sync("runtime state api eval"),
        "[RUNTIME-STATE-API]",
    );
    text_check(
        report,
        "artifact_api_refresh_after_locus_operator",
        client.run_command_sync("artifact api eval"),
        "[ARTIFACT-API]",
    );

    response_body_check(
        report,
        "locus_runtime_state",
        client.runtime_state("eden_locus_layer"),
        "eden-locus-layer-v1",
    );
    response_body_check(
        report,
        "locus_context_packet_state",
        client.runtime_state("locus_context_packet"),
        "eden-locus-context-packet-v1",
    );
    response_body_check(
        report,
        "operator_forge_runtime_state",
        client.runtime_state("eden_operator_forge"),
        "eden-operator-forge-v1",
    );
    response_body_check(
        report,
        "operator_expression_graphs_state",
        client.runtime_state("operator_expression_graphs"),
        "eden-operator-expression-graph-v1",
    );
    response_body_check(
        report,
        "locus_operator_bridge_state",
        client.runtime_state("locus_operator_bridge"),
        "eden-locus-operator-bridge-v1",
    );
    response_body_check(
        report,
        "locus_artifact_read",
        client.artifact("eden_locus_layer"),
        "eden-locus-layer-v1",
    );
    response_body_check(
        report,
        "operator_forge_artifact_read",
        client.artifact("eden_operator_forge"),
        "eden-operator-forge-v1",
    );
    response_body_check(
        report,
        "locus_operator_bridge_artifact_read",
        client.artifact("locus_operator_bridge"),
        "eden-locus-operator-bridge-v1",
    );

    if let Some(locus_dry_run) = json_check(
        report,
        "locus_action_dry_run",
        client.action_dry_run("locus ingest operator preference"),
        "eden-action-dry-run-v1",
    ) {
        assert_contains_json(
            report,
            "locus_dry_run_handler",
            &locus_dry_run,
            "gewc_locus_context_body_handler",
        );
        assert_contains_json(
            report,
            "locus_dry_run_permission",
            &locus_dry_run,
            "local_state_mutation",
        );
    }
    if let Some(forge_dry_run) = json_check(
        report,
        "operator_forge_action_dry_run",
        client.action_dry_run("operator forge synth causal risk"),
        "eden-action-dry-run-v1",
    ) {
        assert_contains_json(
            report,
            "operator_forge_dry_run_handler",
            &forge_dry_run,
            "gewc_formal_synthesis_body_handler",
        );
        assert_contains_json(
            report,
            "operator_forge_dry_run_permission",
            &forge_dry_run,
            "local_state_mutation",
        );
    }
}

fn check_capability_and_gewc_api(client: &EdenClient, report: &mut ConformanceReport) {
    if let Some(catalog) = json_check(
        report,
        "capabilities_catalog",
        client.capabilities_catalog(),
        "eden-capabilities-catalog-v1",
    ) {
        assert_claim_blocked(report, "capabilities_catalog_policy", &catalog);
        assert_array_min(
            report,
            "capabilities_catalog_records",
            &catalog,
            "records",
            1,
        );
    }
    if let Some(status) = json_check(
        report,
        "capabilities_status",
        client.capabilities_status(),
        "eden-capabilities-status-v1",
    ) {
        assert_claim_blocked(report, "capabilities_status_policy", &status);
        assert_contains_json(
            report,
            "capabilities_status_runtime",
            &status,
            "engine_status",
        );
    }
    if let Some(runtime) = json_check(
        report,
        "gewc_runtime",
        client.gewc_runtime(),
        "eden-gewc-runtime-api-v1",
    ) {
        assert_claim_blocked(report, "gewc_runtime_policy", &runtime);
        assert_contains_json(report, "gewc_runtime_report", &runtime, "[GEWC-RUNTIME]");
    }
    if let Some(handlers) = json_check(
        report,
        "gewc_handlers",
        client.gewc_handlers(),
        "eden-gewc-handlers-api-v1",
    ) {
        assert_claim_blocked(report, "gewc_handlers_policy", &handlers);
        assert_contains_json(
            report,
            "gewc_validation_handler_present",
            &handlers,
            "gewc_validation_body_handler",
        );
    }
}

fn check_validation_and_action_api(client: &EdenClient, report: &mut ConformanceReport) {
    if let Some(validation) = json_check(
        report,
        "validation_status",
        client.validation_status(),
        "eden-validation-status-api-v1",
    ) {
        assert_claim_blocked(report, "validation_status_policy", &validation);
        assert_array_min(
            report,
            "validation_status_records",
            &validation,
            "records",
            5,
        );
    }
    if let Some(contracts) = json_check(
        report,
        "action_contracts",
        client.action_contracts(),
        "eden-operational-action-contracts-v1",
    ) {
        assert_claim_blocked(report, "action_contracts_policy", &contracts);
        assert_contains_json(
            report,
            "action_contracts_dry_run_route",
            &contracts,
            "/api/actions/dry-run?cmd=<command>",
        );
        assert_contains_json(
            report,
            "action_contracts_sync_route",
            &contracts,
            "/api/command_sync?cmd=<command>",
        );
    }
    if let Some(dry_run) = json_check(
        report,
        "action_dry_run",
        client.action_dry_run("evolve"),
        "eden-action-dry-run-v1",
    ) {
        assert_claim_blocked(report, "action_dry_run_policy", &dry_run);
        assert_pointer_bool(
            report,
            "action_dry_run_no_execute",
            &dry_run,
            "/would_execute",
            false,
        );
        assert_pointer_bool(report, "action_dry_run_flag", &dry_run, "/dry_run", true);
        assert_pointer_bool(
            report,
            "action_dry_run_requires_supervision",
            &dry_run,
            "/requires_supervision",
            true,
        );
        assert_pointer_bool(
            report,
            "action_dry_run_would_mutate_if_executed",
            &dry_run,
            "/mutates_runtime",
            true,
        );
        assert_contains_json(
            report,
            "action_dry_run_persistent_permission",
            &dry_run,
            "persistent_permission",
        );
    }
}

fn json_check(
    report: &mut ConformanceReport,
    name: &str,
    result: Result<Value, EdenClientError>,
    schema: &str,
) -> Option<Value> {
    match result {
        Ok(value) => {
            if value.get("schema").and_then(Value::as_str) == Some(schema) {
                report.pass(name, schema);
            } else {
                report.fail(name, format!("unexpected schema in {value}"));
            }
            Some(value)
        }
        Err(err) => {
            report.fail(name, err.to_string());
            None
        }
    }
}

fn text_check(
    report: &mut ConformanceReport,
    name: &str,
    result: Result<String, EdenClientError>,
    expected: &str,
) {
    match result {
        Ok(body) if body.contains(expected) => report.pass(name, expected),
        Ok(body) => report.fail(
            name,
            format!("missing {expected:?} in body={}", compact(&body)),
        ),
        Err(err) => report.fail(name, err.to_string()),
    }
}

fn response_body_check(
    report: &mut ConformanceReport,
    name: &str,
    result: Result<crate::sdk::EdenHttpResponse, EdenClientError>,
    expected: &str,
) {
    match result {
        Ok(response) if response.status_code == 200 && response.body.contains(expected) => {
            report.pass(name, expected);
        }
        Ok(response) => report.fail(
            name,
            format!(
                "status={} missing {expected:?} in body={}",
                response.status_code,
                compact(&response.body)
            ),
        ),
        Err(err) => report.fail(name, err.to_string()),
    }
}

fn assert_claim_blocked(report: &mut ConformanceReport, name: &str, value: &Value) {
    let claim_allowed = value.get("claim_allowed").and_then(Value::as_bool);
    let agi_claim = value.get("agi_claim").and_then(Value::as_bool);
    if claim_allowed == Some(false) && agi_claim == Some(false) {
        report.pass(name, "claim_allowed=false agi_claim=false");
    } else {
        report.fail(
            name,
            format!("claim_allowed={claim_allowed:?} agi_claim={agi_claim:?}"),
        );
    }
}

fn assert_pointer_bool(
    report: &mut ConformanceReport,
    name: &str,
    value: &Value,
    pointer: &str,
    expected: bool,
) {
    match value.pointer(pointer).and_then(Value::as_bool) {
        Some(actual) if actual == expected => report.pass(name, format!("{pointer}={expected}")),
        Some(actual) => report.fail(name, format!("{pointer} expected {expected} got {actual}")),
        None => report.fail(name, format!("missing boolean pointer {pointer}")),
    }
}

fn assert_array_min(
    report: &mut ConformanceReport,
    name: &str,
    value: &Value,
    key: &str,
    expected_min: usize,
) {
    match value.get(key).and_then(Value::as_array) {
        Some(records) if records.len() >= expected_min => {
            report.pass(name, format!("{key}={}", records.len()));
        }
        Some(records) => report.fail(
            name,
            format!("{key} expected >= {expected_min} got {}", records.len()),
        ),
        None => report.fail(name, format!("missing array key {key}")),
    }
}

fn assert_contains_json(report: &mut ConformanceReport, name: &str, value: &Value, needle: &str) {
    let body = value.to_string();
    if body.contains(needle) {
        report.pass(name, needle);
    } else {
        report.fail(name, format!("missing {needle}"));
    }
}

fn assert_all_operational_records_read_only(report: &mut ConformanceReport, value: &Value) {
    let Some(records) = value.get("records").and_then(Value::as_array) else {
        report.fail("operational_records_read_only", "missing records");
        return;
    };
    let all_read_only = records.iter().all(|record| {
        record.get("read_only").and_then(Value::as_bool) == Some(true)
            && record.get("mutates_runtime").and_then(Value::as_bool) == Some(false)
    });
    if all_read_only {
        report.pass(
            "operational_records_read_only",
            format!("records={}", records.len()),
        );
    } else {
        report.fail(
            "operational_records_read_only",
            "one or more operational records mutate runtime",
        );
    }
}

fn compact(value: &str) -> String {
    let mut text = value.replace('\n', "\\n");
    if text.len() > 240 {
        text.truncate(240);
        text.push_str("...");
    }
    text
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

struct Args {
    base_url: String,
    report_path: Option<PathBuf>,
    timeout_sec: u64,
}

impl Args {
    fn parse() -> Self {
        let mut base_url = "http://127.0.0.1:8080".to_string();
        let mut report_path = None;
        let mut timeout_sec = 5;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--base-url" => {
                    if let Some(value) = args.next() {
                        base_url = value;
                    }
                }
                "--report" => {
                    if let Some(value) = args.next() {
                        report_path = Some(PathBuf::from(value));
                    }
                }
                "--timeout-sec" => {
                    if let Some(value) = args.next().and_then(|raw| raw.parse::<u64>().ok()) {
                        timeout_sec = value;
                    }
                }
                "--help" | "-h" => {
                    println!(
                        "Usage: eden-garm-api-conformance [--base-url http://127.0.0.1:8080] [--report PATH] [--timeout-sec N]"
                    );
                    std::process::exit(0);
                }
                _ => {}
            }
        }
        Self {
            base_url,
            report_path,
            timeout_sec,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_json_preserves_no_claim_policy() {
        let mut report = ConformanceReport::new("http://127.0.0.1:8110");
        report.pass("example", "ok");

        let body = report.as_json();

        assert_eq!(
            body.get("schema").and_then(Value::as_str),
            Some("eden-sdk-conformance-report-v1")
        );
        assert_eq!(
            body.get("claim_allowed").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(body.get("agi_claim").and_then(Value::as_bool), Some(false));
    }

    #[test]
    fn claim_block_checker_rejects_missing_policy() {
        let mut report = ConformanceReport::new("http://127.0.0.1:8110");

        assert_claim_blocked(&mut report, "policy", &serde_json::json!({}));

        assert!(!report.ok());
    }
}
