# ADR-048: EDEN SDK And API Conformance Runner

## Status
Accepted

## Date
2026-05-23

## Context
EDEN now exposes runtime state, operational, action-contract and artifact APIs.
Those APIs are useful only if external consumers can interact with them through
stable contracts instead of reading implementation files directly.

We need a small client surface and a repeatable conformance suite that verifies
the live runtime from outside the process. The suite must preserve the existing
no-claim policy: it can validate interoperability and operational safety
contracts, but it must not imply AGI capability validation.

## Decision
Add a lightweight Rust SDK at `eden_core/examples/eden_garm/client_sdk.rs` and
a standalone external conformance runner at
`eden_core/examples/eden_garm_api_conformance.rs`.

The SDK uses local HTTP through `TcpStream` and exposes typed helpers for:

- runtime state APIs
- artifact APIs
- operational APIs
- capability APIs
- GEWC APIs
- validation APIs
- action contract APIs
- action dry-run classification
- explicit command mutation methods

The conformance runner uses only read-only methods and `action_dry_run`. It
checks externally visible schemas, no-claim markers, path whitelist behavior,
read-only operational records, GEWC handler visibility and dry-run
non-execution. It writes `sdk_conformance_report.json` with schema
`eden-sdk-conformance-report-v1`.

Add `make eden-api-conformance` and a CI `API Conformance` job that starts a
temporary EDEN runtime, generates API contracts, runs the external conformance
runner and uploads the report/logs as evidence.

## Alternatives Considered

### Keep using curl-only smoke tests
- Pros: Simple and already available.
- Cons: Shell assertions do not provide a reusable SDK contract for external
  consumers.
- Rejected: Curl smoke tests remain useful, but they are not enough for API
  conformance.

### Add a full generated OpenAPI client
- Pros: Better for broad ecosystem integration.
- Cons: Adds generator/tooling complexity before the API surface has had enough
  consumer feedback.
- Deferred: The current SDK is intentionally small and can later feed a
  generated client.

### Put conformance inside the GARM runtime
- Pros: Easier access to internal state.
- Cons: Would not validate external consumer behavior.
- Rejected: Conformance must observe EDEN from outside the process.

## Consequences
- EDEN now has a reusable local SDK surface for runtime/API consumers.
- CI verifies that the live HTTP endpoint conforms to the published runtime API
  surface.
- The conformance report is explicit no-claim evidence, not AGI validation.
- Mutation-capable client methods are available but named explicitly; the
  conformance suite does not call them except during setup through the shell
  harness.
