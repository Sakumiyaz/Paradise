# EDEN SDK And API Conformance

Date: 2026-05-24

This document describes the current external API client and conformance runner.
It is an interoperability contract, not an AGI capability claim.

## SDK

The lightweight Rust SDK lives at:

- `eden_core/src/sdk.rs`

The operator CLI lives at:

- `eden_core/src/bin/edenctl.rs`

Compatibility wrappers/helpers remain for older local workflows:

- `eden_core/src/garm/client_sdk.rs`
- `eden_core/examples/edenctl.rs`

It intentionally uses plain local HTTP over `TcpStream` so the conformance path
does not add a new runtime dependency or require a browser, proxy or external
service. The client separates read-only inspection from explicit command
mutation methods.

Read-oriented methods include:

- `health`
- `ready`
- `runtime_catalog`
- `runtime_openapi`
- `runtime_snapshot`
- `runtime_state`
- `artifact_catalog`
- `artifact_runtime`
- `artifact`
- `operational_catalog`
- `operational_openapi`
- `operational_runtime`
- `operational_contract`
- `operational_status`
- `operational_permissions`
- `operational_replay`
- `operational_replay_decision`
- `operational_recovery`
- `operational_demos`
- `operational_schemas`
- `operational_schema`
- `capabilities_catalog`
- `capabilities_status`
- `gewc_runtime`
- `gewc_handlers`
- `validation_status`
- `action_contracts`
- `action_dry_run`

Mutation-capable methods are explicit:

- `queue_command`
- `run_command_sync`
- `command_result`
- `forget_command`
- `locus_eval`
- `locus_ingest`
- `locus_context`
- `locus_audit`
- `operator_forge_eval`
- `operator_forge_synth`
- `operator_forge_verify`
- `operator_forge_audit`

Locus and Operator Forge helpers are command-routed convenience methods over
`run_command_sync`; they do not bypass GEWC.

The SDK reads `EDEN_API_TOKEN` automatically when present. Callers may also use
`with_api_token` to send `Authorization: Bearer <token>` explicitly.

## Conformance Runner

The external runner lives at:

- native module: `eden_core/src/garm_api_conformance.rs`
- native binary: `cargo run -p eden_core --bin eden-garm-api-conformance --`
- compatibility wrapper: `eden_core/examples/eden_garm_api_conformance.rs`

It validates a live EDEN endpoint from the outside through the SDK. The runner
checks that the runtime exposes the expected API families and that no-claim
policy markers are externally visible:

- operator console root surface
- health and readiness
- runtime state API
- artifact API
- operational API
- capability API
- GEWC API
- validation API
- action contract API
- dry-run action classification without command execution
- runtime/artifact path whitelist rejection
- Locus command execution and artifact/state reads
- Operator Forge command execution and artifact/state reads
- governed Locus/Forge-to-CWM bridge evidence

The runner writes:

- `sdk_conformance_report.json`

The report schema is `eden-sdk-conformance-report-v1` and preserves
`claim_allowed=false` and `agi_claim=false`.

## Operator CLI

`edenctl` wraps the same local API and command routes used by the SDK:

```sh
cargo run -p eden_core --bin edenctl -- status
cargo run -p eden_core --bin edenctl -- schemas operational_status
cargo run -p eden_core --bin edenctl -- permissions audit
cargo run -p eden_core --bin edenctl -- recovery run
cargo run -p eden_core --bin edenctl -- demo run
cargo run -p eden_core --bin edenctl -- locus ingest "operator preference :: governed context"
cargo run -p eden_core --bin edenctl -- forge synth "causal risk model"
cargo run -p eden_core --bin edenctl -- command --wait-sec 180 "readiness package"
cargo run -p eden_core --bin edenctl -- doctor
cargo run -p eden_core --bin edenctl -- openapi export --output-dir contracts/v1/openapi
```

It is intended for operators and integration smoke tests. Programmatic clients
should use the SDK or direct HTTP when they need structured responses.

## Local Command

```sh
make eden-api-conformance
```

The target starts a temporary EDEN runtime, generates the runtime state,
operational and artifact API contracts, runs the external conformance runner,
writes the report and shuts the runtime down.

## CI

GitHub Actions runs the same target in the `API Conformance` job and uploads
the report plus runtime logs as evidence.

## Scope

This suite verifies API interoperability and safety-contract visibility. It
does not validate AGI capability, model intelligence, benchmark superiority or
external real-world performance.
