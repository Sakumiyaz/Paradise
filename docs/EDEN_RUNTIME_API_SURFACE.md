# EDEN Runtime API Surface

Date: 2026-05-25

This document describes the current local runtime API surface. It is an
operational contract map, not an AGI capability claim. All listed generated
contracts keep `claim_allowed=false` and `agi_claim=false`.

## Read-Only Surfaces

| Surface | Routes | Purpose |
| --- | --- | --- |
| Operator Console | `/`, `/console`, `/api/console`, `/api/help` | Serve the static operator console and plain-text endpoint help. |
| Artifact API | `/api/artifact/catalog`, `/api/artifact?name=...`, `/api/artifact/runtime` | Read reproducible release artifacts by whitelisted artifact name. |
| Runtime State API | `/api/runtime/catalog`, `/api/runtime/state?name=...`, `/api/runtime/snapshot`, `/api/runtime/openapi` | Read whitelisted runtime state files and live runtime metrics. |
| Operational API | `/api/operational/catalog`, `/api/operational/openapi`, `/api/operational/runtime`, `/api/operational/contract`, `/api/operational/runtime-phase`, `/api/operational/spine`, `/api/operational/events`, `/api/operational/global-state`, `/api/operational/replay-spine`, `/api/operational/spine-verification`, `/api/operational/spine-enforcement`, `/api/operational/workflow-risk`, `/api/operational/circuit-breakers`, `/api/operational/spine-replay`, `/api/operational/status`, `/api/operational/permissions`, `/api/operational/replay`, `/api/operational/recovery`, `/api/operational/demos`, `/api/operational/schemas` | Read the operational API contract set, runtime status, persistent permission boundary, runtime phase artifact, runtime spine, event bus, global-state head, spine verification/enforcement/risk/breaker/replay reports, recovery plan, schema registry, demo evidence and GEWC decision replay evidence. |
| Paradise API | `/api/paradise/sessions`, `/api/paradise/worldcell` | Read Paradise Worldcell session evidence and runtime identity artifacts. |
| Capability API | `/api/capabilities/catalog`, `/api/capabilities/status` | Read current capability inventory and runtime health status. |
| GEWC API | `/api/gewc/runtime`, `/api/gewc/handlers` | Read GEWC runtime trace summary and handler topology. |
| Validation API | `/api/validation/status` | Read validation artifact presence and no-claim policy status. |
| Action Contract API | `/api/actions/contracts`, `/api/actions/dry-run?cmd=...` | Read action permissions and classify commands without execution. |

## Mutation Surfaces

Mutation remains intentionally narrow:

| Route | Mode | Guard |
| --- | --- | --- |
| `/api/command?cmd=...` | queued async command | GEWC command routing and pre-execution safety |
| `/api/command_sync?cmd=...` | queued sync command with timeout | ready-runtime check plus GEWC command routing |
| `POST /api/command` | legacy-compatible command submission | GEWC command routing |

The dry-run route does not enqueue commands. It parses a command, returns its
GEWC body handler, route, domain, lifecycle policy and whether the command would
require supervision if executed through the mutation route.

## Optional Local Auth

Set `EDEN_API_TOKEN` before starting the runtime to require a local token on
non-public routes. The SDK and `edenctl --token` send the token through
`Authorization: Bearer <token>`. The server also accepts
`X-EDEN-API-Token: <token>`.

This guard is intentionally local-first. It does not replace network isolation,
process sandboxing or production identity controls.

## Generated Artifacts

`artifact api eval` writes:

- `artifact_api_catalog.json`
- `artifact_api_contracts.json`
- `artifact_api_runtime.json`

`runtime state api eval` writes:

- `runtime_state_api_catalog.json`
- `runtime_state_api_contracts.json`
- `runtime_state_api_openapi.json`
- `runtime_state_api_runtime.json`

`operational api eval` writes:

- `operational_api_catalog.json`
- `operational_api_contracts.json`
- `operational_api_openapi.json`
- `operational_api_runtime.json`
- `operational_action_contracts.json`
- `operational_contract.json`
- `operational_permissions.json`
- `schema_registry.json`

Command-routed operational management may also write:

- `operational_permissions_audit.json`
- `operational_permissions_diff.json`
- `operational_permissions_history.jsonl`
- `operational_recovery_plan.json`
- `operational_demo_suite.json`
- `paradise_worldcell_sessions.json`
- `runtime_spine.json`
- `runtime_event_bus.jsonl`
- `runtime_event_bus_state.json`
- `runtime_global_state.json`
- `runtime_global_state_log.jsonl`
- `runtime_replay_spine.json`
- `runtime_spine_verification.json`
- `runtime_guard_decisions.jsonl`
- `runtime_spine_enforcement.json`
- `runtime_workflow_risk.json`
- `runtime_circuit_breakers.json`
- `runtime_replay_reconstruction.json`
- `runtime_security_gates.json`
- `runtime_model_router_contract.json`
- `runtime_memory_fabric_contract.json`
- `runtime_world_simulation_contract.json`
- `runtime_multiagent_contract.json`

`paradise worldcell eval` writes `paradise_worldcell_runtime.json`.
`paradise intent ...`, `paradise plan`, `paradise approve`,
`paradise execute` and `paradise sessions` write or read the
`paradise_worldcell_sessions.json` session loop.

`runtime spine eval` writes the common GEWC-owned operational substrate:
universal internal message contracts, append-only event bus, validated global
state, replay spine, enforcement artifacts, verification report, security
gates, model-router contract, memory fabric, world-simulation contract and
multiagent coordination contract. `runtime spine enforce` refreshes the
mandatory guard report and guard-decision log; `runtime spine risk` computes
chain-level workflow risk; `runtime spine breakers` reports the circuit-breaker
state; and `runtime spine replay` reconstructs the recent operational timeline
from recorded evidence only. `runtime spine verify` checks artifact presence,
append-only event/state sequences, GEWC authority, snapshot/replay count
consistency, payload hashes, claim boundaries, guard coverage, breaker health
and validated state writes. GEWC decision and completion traces now publish into
this substrate, and Paradise sessions record intent, plan, approval, block and
completion events there as well.

`locus eval`, `locus ingest ...`, `locus context ...` and `locus audit` write or
read the native Locus evidence set:

- `eden_locus_layer.json`
- `locus_authority_model.json`
- `locus_evidence_vault.json`
- `locus_permission_matrix.json`
- `locus_context_packet.json`
- `locus_operator_timeline.jsonl`

`operator forge eval`, `operator forge synth ...`, `operator forge verify` and
`operator forge audit` write or read the native formal-synthesis evidence set:

- `eden_operator_forge.json`
- `operator_primitive_basis.json`
- `operator_expression_graphs.jsonl`
- `operator_verification_report.json`
- `operator_model_registry.json`

`operational runtime eval` writes the eight-component runtime phase evidence
served at `/api/operational/runtime-phase`, including
`locus_operator_bridge.json` for the governed Locus/Forge-to-CWM hypothesis
bridge. `operational scenario run` writes the long-horizon black-box scenario
artifact served through `/api/runtime/state?name=operational_e2e_scenario`.

## Validation

The local API contract gate is:

```sh
make eden-api-contracts
```

It generates API evidence, runs `readiness package`, and then executes the
native `eden-garm-package-validator` runner against the generated state
directory. CI runs the same target in the `API Contract Package` job.

The external consumer conformance gate is:

```sh
make eden-api-conformance
```

It starts a temporary EDEN runtime, generates the API contracts, then validates
the live endpoint through the Rust SDK in `eden_core/src/sdk.rs` by running the
native `eden-garm-api-conformance` runner. The conformance report is written
as `sdk_conformance_report.json` with schema
`eden-sdk-conformance-report-v1`. See `docs/EDEN_SDK_CONFORMANCE.md`.

The operational black-box gate is:

```sh
make operational-blackbox
```

It starts EDEN as a real local process, verifies health/readiness, generates the
operational contract, runs the operational runtime phase and scenario, executes
Locus/Forge commands, checks dry-run hardening for safe, high-risk, Locus and
Forge commands, reads replay, bridge, action-evidence, recovery, permissions,
schema and demo artifacts from the API, writes `operational_evidence_bundle.json`,
then shuts the runtime down.

The long-run stability gate is:

```sh
make long-run-stability
```

It starts the runtime, exercises public `edenctl` commands including
`edenctl locus ...` and `edenctl forge ...`, forces a degraded GEWC handler
state, recovers it, runs replay and demos, writes a readiness package and
evidence bundle, restarts against the same state directory, then executes the
independent package validator.

## Public v1 Contract

The stable public runtime surface is frozen in:

```text
docs/EDEN_PUBLIC_API_V1.md
contracts/v1/
```

External consumers should build against that contract, `edenctl` or
`eden_core/src/sdk.rs`, not against internal handler
modules.
