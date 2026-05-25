# ADR-077: Make Runtime Spine Enforcement Executable

## Status
Accepted

## Date
2026-05-25

## Context
ADR-076 made the GEWC Runtime Spine the shared substrate for messages, event
bus, global state, replay, safety, model routing, memory, simulation and
multiagent coordination. The next gap was operational authority: the substrate
needed to prove that modules cannot silently mutate runtime-critical state
outside the spine.

Without a native enforcement path, Eden could still look integrated while
letting modules behave like independent writers. That would weaken replay,
auditing, rollback and safety claims.

## Decision
Add RuntimeSpineGuard enforcement to `eden_core/src/garm/runtime_spine.rs`.
Every runtime event and global-state mutation now receives an append-only guard
decision before commit. Unauthorized state writers are blocked before they can
append to the global-state log.

The spine now writes and exposes:

- `runtime_guard_decisions.jsonl`
- `runtime_spine_enforcement.json`
- `runtime_workflow_risk.json`
- `runtime_circuit_breakers.json`
- `runtime_replay_reconstruction.json`

New commands refresh the derived reports:

- `runtime spine enforce`
- `runtime spine risk`
- `runtime spine breakers`
- `runtime spine replay`

The guard path is intentionally split into fast inline decisions and derived
reports. Event/state writes append guard evidence in the hot path; aggregate
workflow risk, breaker state and replay reconstruction are refreshed by explicit
commands, API reads and verification. This keeps the runtime enforceable
without making every event append recalculate the full evidence set.

`runtime spine verify` now also checks guard coverage, mandatory enforcement
presence, circuit-breaker health and replay-reconstruction availability.

## Alternatives Considered

### Keep Enforcement As Documentation
- Pros: Smaller code change.
- Cons: Does not prevent direct state writes or produce per-write decisions.
- Rejected: The architecture review requires executable authority, not only a
  description.

### Recalculate Every Enforcement Artifact On Every Event
- Pros: Derived reports are always fresh.
- Cons: Turns the event bus into an expensive aggregate-report path and slows
  representative runtime execution.
- Rejected: Inline guard decisions are the required enforcement primitive;
  aggregate reports can be refreshed explicitly.

### Delegate Enforcement To Individual Modules
- Pros: Less central logic.
- Cons: Recreates the disconnected-module problem that the Runtime Spine exists
  to remove.
- Rejected: Runtime-critical writes need one GEWC-owned authority path.

## Consequences
- Runtime events and global-state mutations have prior guard evidence.
- Unauthorized global-state writers are blocked before append.
- Workflow risk is evaluated at chain level rather than action-only level.
- Circuit breakers can degrade or pause capabilities while preserving evidence.
- Replay reconstruction is evidence-only and does not re-execute external
  actions.
- The runtime remains no-claim gated: this is stronger operational architecture,
  not proof of completed AGI capability.
