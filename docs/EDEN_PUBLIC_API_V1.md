# EDEN Public API v1

Date: 2026-05-24

This document freezes the first public operator/runtime contract for EDEN
Hybrid. It defines the stable surface for external tools, SDK consumers,
operator workflows and validation scripts. It is an API stability contract, not
an AGI capability claim.

All public v1 contracts preserve:

- `claim_allowed=false`
- `agi_claim=false`
- local-first operation
- explicit action boundaries
- no mutation through dry-run or read endpoints

## Stable Runtime Entry Points

| Surface | Stable v1 routes |
| --- | --- |
| Console and help | `/`, `/console`, `/api/console`, `/api/help` |
| Health/readiness | `/api/health`, `/ready`, `/status`, `/metrics` |
| Runtime state | `/api/runtime/catalog`, `/api/runtime/state?name=...`, `/api/runtime/snapshot`, `/api/runtime/openapi` |
| Artifacts | `/api/artifact/catalog`, `/api/artifact?name=...`, `/api/artifact/runtime` |
| Operational | `/api/operational/catalog`, `/api/operational/openapi`, `/api/operational/runtime`, `/api/operational/contract`, `/api/operational/runtime-phase`, `/api/operational/spine`, `/api/operational/events`, `/api/operational/global-state`, `/api/operational/replay-spine`, `/api/operational/spine-verification`, `/api/operational/status`, `/api/operational/permissions`, `/api/operational/replay`, `/api/operational/recovery`, `/api/operational/demos`, `/api/operational/schemas`, `/api/operational/schema?name=...` |
| Paradise | `/api/paradise/sessions`, `/api/paradise/worldcell` |
| Capability | `/api/capabilities/catalog`, `/api/capabilities/status` |
| GEWC | `/api/gewc/runtime`, `/api/gewc/handlers` |
| Validation | `/api/validation/status` |
| Action contracts | `/api/actions/contracts`, `/api/actions/dry-run?cmd=...` |

## Stable Mutation Routes

The only stable v1 mutation routes are:

```text
/api/command?cmd=...
/api/command_sync?cmd=...
POST /api/command
```

These routes are not raw tool access. They enter GEWC command routing and pass
through runtime safety gates before execution. External tools should prefer
`/api/actions/dry-run?cmd=...` before calling a mutation route.

## Stable Command Families

The following command families are considered public v1 operator commands:

| Family | Commands |
| --- | --- |
| API generation | `operational api eval`, `runtime state api eval`, `artifact api eval` |
| Operational runtime | `operational runtime eval`, `operational scenario run`, `operational replay run`, `operational demo run` |
| Runtime spine | `runtime spine eval`, `runtime spine enforce`, `runtime spine risk`, `runtime spine breakers`, `runtime spine replay`, `runtime spine audit`, `runtime spine verify` |
| Locus/Forge | `locus eval`, `locus ingest <text>`, `locus context <query>`, `locus audit`, `operator forge eval`, `operator forge synth <goal>`, `operator forge verify`, `operator forge audit` |
| Permissions | `operational permissions audit`, `operational permissions diff`, `operational permissions history`, `operational permissions set <key> <allow|deny>`, `operational permissions restore` |
| Recovery | `operational recovery audit`, `operational recovery run` |
| GEWC lifecycle | `gewc lifecycle <handler> <active|pause|degrade|recover>` |
| Paradise | `paradise worldcell eval`, `paradise intent <text>`, `paradise plan`, `paradise approve`, `paradise execute`, `paradise sessions` |
| Packaging | `readiness package` |

Other commands may exist, but they should be treated as internal or
experimental unless they are documented here or in
`docs/EDEN_RUNTIME_API_SURFACE.md`.

## Stable Generated Artifacts

External validation and release tooling may depend on these artifact schemas:

| Artifact | Schema |
| --- | --- |
| `operational_contract.json` | `eden-operational-contract-v1` |
| `operational_permissions.json` | `eden-operational-permissions-v1` |
| `operational_permissions_audit.json` | `eden-operational-permissions-audit-v1` |
| `operational_recovery_plan.json` | `eden-operational-recovery-plan-v1` |
| `operational_demo_suite.json` | `eden-operational-demo-suite-v1` |
| `eden_locus_layer.json` | `eden-locus-layer-v1` |
| `eden_operator_forge.json` | `eden-operator-forge-v1` |
| `locus_operator_bridge.json` | `eden-locus-operator-bridge-v1` |
| `schema_registry.json` | `eden-schema-registry-v1` |
| `operational_evidence_bundle.json` | `eden-operational-evidence-bundle-v1` |
| `runtime_state_api_catalog.json` | `eden-runtime-state-api-catalog-v1` |
| `artifact_api_catalog.json` | `eden-artifact-api-catalog-v1` |
| `paradise_worldcell_runtime.json` | `eden-paradise-worldcell-runtime-v1` |
| `paradise_worldcell_sessions.json` | `eden-paradise-worldcell-session-v1` |
| `runtime_spine.json` | `eden-runtime-spine-v1` |
| `runtime_event_bus_state.json` | `eden-runtime-event-bus-v1` |
| `runtime_global_state.json` | `eden-runtime-global-state-v1` |
| `runtime_replay_spine.json` | `eden-runtime-replay-spine-v1` |
| `runtime_spine_verification.json` | `eden-runtime-spine-verification-v1` |
| `runtime_guard_decisions.jsonl` | `eden-runtime-guard-decision-v1` |
| `runtime_spine_enforcement.json` | `eden-runtime-spine-enforcement-v1` |
| `runtime_workflow_risk.json` | `eden-runtime-workflow-risk-v1` |
| `runtime_circuit_breakers.json` | `eden-runtime-circuit-breakers-v1` |
| `runtime_replay_reconstruction.json` | `eden-runtime-replay-reconstruction-v1` |
| `runtime_security_gates.json` | `eden-runtime-security-gates-v1` |
| `readiness_package.json` | `garm-readiness-package-v1` |
| `sdk_conformance_report.json` | `eden-sdk-conformance-report-v1` |

The schema registry is the canonical discovery point for generated contract
names. Consumers should resolve schema records through
`/api/operational/schema?name=...` instead of assuming every file is present.

## Versioning Rules

- Additive fields are allowed in v1 JSON objects.
- Existing stable field names should not be removed or repurposed in v1.
- New command families must be documented before being treated as public.
- Breaking changes require a v2 document and an ADR.
- `contracts/v1/` contains the versioned manifest, OpenAPI snapshots, schemas
  and examples for this API generation.
- Internal files under `eden_core/src/garm/` remain implementation
  details unless they are exposed through this document, the SDK or a generated
  schema.

## Optional Local Auth

By default, Paradise remains local-first and requires no token for loopback
development. If `EDEN_API_TOKEN` is set before runtime startup, non-public
routes require one of:

```text
Authorization: Bearer <token>
X-EDEN-API-Token: <token>
```

This is a local operator guard for development and validation workflows. Public
v1 does not claim internet-facing production authentication.

## Recommended Client

Use one of:

- `cargo run -p eden_core --bin edenctl -- ...`
- `eden_core/src/sdk.rs`
- direct local HTTP against the routes above

The `edenctl` CLI is the operator-friendly wrapper over the same v1 API
surface.

## Non-Goals

Public API v1 does not promise:

- external internet exposure;
- production authentication;
- AGI capability validation;
- trained LLM/LMM behavior;
- direct access to internal GEWC handlers;
- compatibility for undocumented internal commands.
