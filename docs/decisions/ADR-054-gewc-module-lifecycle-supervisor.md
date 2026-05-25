# ADR-054: Keep Module Lifecycle Supervision Native to GEWC

## Status
Accepted

## Date
2026-05-23

## Context
GEWC had become the single runtime authority for command routing, body handler
dispatch, model-plural routing and decision/completion traces. One gap remained:
the core could select a handler, but it did not formally own each handler's
lifecycle state. That made questions about pausing, restarting, isolating,
quarantining or disabling modules only partially answered.

The design constraint is that lifecycle control must not become a second core.
It needs to be part of GEWC's executive authority and share the same policy,
audit and no-claim boundaries.

## Decision
Add a native GEWC module lifecycle supervisor.

Each `CoreDecision` now includes a `ModuleLifecycleControl` with:

- handler identity;
- lifecycle state;
- allowed lifecycle actions;
- supervision requirement;
- policy gate;
- isolation scope;
- audit channel.

GEWC can also write explicit lifecycle control events through
`GlobalExecutiveWorkspaceCore::supervise_module`. Persisted lifecycle events are
read before later decisions, so paused or recovering handlers are deferred,
isolated/disabled/failed handlers are blocked and quarantined handlers request
supervision before execution.

The formal GEWC artifact now includes module lifecycle supervision, health
checks, isolation controls and recovery controls as first-class components.
Operational handler metadata exposes lifecycle state and allowed actions through
the GEWC handler API surface.

## Alternatives Considered

### Separate lifecycle manager module
Pros: smaller implementation surface.
Cons: creates another authority beside GEWC and weakens the single-core design.
Rejected.

### Trace-only lifecycle metadata
Pros: minimal code change.
Cons: would not affect future decisions and would leave pause/isolate/disable
as documentation rather than operational control.
Rejected.

### Command-specific lifecycle controls only
Pros: user-facing commands could be added later.
Cons: the core still needs native lifecycle semantics before exposing commands or
APIs.
Deferred.

## Consequences
- GEWC now has explicit authority over handler lifecycle states.
- Lifecycle state is auditable in `global_executive_workspace_runtime.jsonl`.
- Handler state can affect execution gating before module code runs.
- Safety and validation planes cannot be disabled as ordinary modules; they use
  stricter lifecycle actions and supervision boundaries.
- Future CLI/API lifecycle commands can reuse the same GEWC-native supervisor
  instead of introducing a parallel control plane.
