# ADR-059: Operator Console V2, Permission Management, Recovery, Schemas And Demos

## Status
Accepted

## Date
2026-05-24

## Context
After operational status, persistent permissions and replay were exposed, EDEN
still needed a more usable control surface:

- the operator console was mostly static;
- permission changes had no governed command workflow;
- degraded mode had status visibility but no recovery plan;
- schemas were scattered across runtime APIs and artifacts;
- demos were present as broad scenarios, not as three focused operational proofs.

## Decision
Extend the operational layer without adding public mutation endpoints:

- The operator console now reads live status, permissions, recovery, replay,
  schema and demo APIs.
- Permission management is command-routed through GEWC with audit, diff,
  history, set and restore commands.
- Recovery is command-routed with audit and recover-run commands; recovery uses
  GEWC lifecycle controls and does not replay external actions.
- `/api/operational/schemas` exposes a local schema registry assembled from
  curated operational contracts, runtime state specs and reproducible artifacts.
- `operational demo run` writes three reproducible local demos:
  memory/planning, world-model simulation and tool-security replay.

## Alternatives Considered

### Add HTTP write endpoints for permissions and recovery

Rejected. EDEN's current public API posture keeps mutation behind the command
queue and GEWC body registry so dry-runs, permissions and action evidence remain
auditable.

### Keep schemas only inside OpenAPI documents

Rejected. OpenAPI covers endpoint shape, but EDEN also has generated artifacts
and runtime state JSON files. A registry gives operators one inventory.

### Treat demos as benchmark claims

Rejected. The demos are operational evidence only and preserve
`claim_allowed=false` and `agi_claim=false`.

## Consequences

- Operators can inspect the runtime from the console instead of jumping between
  raw endpoints.
- Permission changes are persistent, auditable and reversible.
- Degraded-mode recovery is now an executable local workflow.
- API consumers can discover schema contracts from one endpoint.
- CI and black-box validation have broader coverage, with no new cloud or model
  dependency.
