# ADR-055: Add the Seven-Component Operational Runtime Phase

## Status
Accepted

## Date
2026-05-23

## Context
EDEN had accumulated a strong architectural map: GEWC, model-plural routing,
memory, world model, action contracts, safety, APIs, lifecycle supervision and
validation artifacts. The next risk was architectural completeness without a
single local runtime path that exercises those pieces together.

The goal is not to add more architectural labels. The goal is to create an
executable local phase that turns the architecture into auditable operational
evidence while preserving the no-AGI-claim boundary.

## Decision
Add `operational_runtime` as a GEWC-governed runtime phase.

The `operational runtime eval` command now writes seven runtime artifacts:

- persistent task runtime;
- action contract executor;
- GEWC lifecycle command evidence;
- memory transaction layer;
- CWM operational state;
- governed agent runtime;
- replay and evaluation record.

The phase records GEWC decisions/completions, action evidence, lifecycle
health-checks, world-model observations/predictions/verification, memory
transaction metadata and replay metrics. It is exposed through runtime state
catalogs, reproducible artifacts and the operational API read surface.

## Alternatives Considered

### Continue adding architecture-only artifacts
Pros: easy to expand.
Cons: does not improve operational reality.
Rejected.

### Build seven independent runtimes
Pros: maximum separation.
Cons: recreates disconnected modules and weakens GEWC as the executive authority.
Rejected.

### Put all behavior directly inside GEWC
Pros: one file and one authority.
Cons: GEWC would become a monolith instead of an executive coordinator.
Rejected.

## Consequences
- EDEN now has a single command that exercises the seven operational runtime
  priorities together.
- Runtime state APIs and artifact APIs can expose the new phase.
- The phase remains local, no-claim and no external-side-effect by default.
- Future work can replace each artifact-backed component with deeper execution
  without changing the public operational contract.
