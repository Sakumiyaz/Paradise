# ADR-035: GEWC Body Executor Owns Physical Command Execution

## Status
Accepted

## Date
2026-05-22

## Context
ADR-034 introduced `GewcBodyRegistry`, making GEWC the authority for intent-to-body binding. The remaining friction was that `dispatch_gewc_cycle` still contained the physical command match. Even though the match was governed by GEWC, the runtime surface still carried too much implementation weight.

The desired direction is for `dispatch_gewc_cycle` to remain the cycle host only: parse, decide, record, delegate execution, record completion. Physical command execution should sit behind a GEWC executor interface so it can later be split into body-specific handlers without changing command semantics.

## Decision
Introduce `GewcBodyExecutor`.

The runtime path is now:

```text
dispatch_gewc_cycle
  -> CommandRouterNode::parse_raw
  -> GlobalExecutiveWorkspaceCore::decide
  -> GewcBodyRegistry::bind
  -> GewcBodyExecutor::execute
  -> GlobalExecutiveWorkspaceCore::record_execution_completion
```

`GewcBodyExecutor` receives `GarmCommand`, the GEWC decision and explicit runtime ports. It validates the decision handler against `GewcBodyRegistry` in debug builds before executing the absorbed body implementation.

The existing Rust command arms are preserved inside the executor to avoid semantic churn. This is a deeper module interface, not a destructive rewrite.

## Alternatives Considered

### Move every handler into separate files now
Rejected for this phase. The next safe step is extracting the executor interface first, then moving body handlers incrementally behind that interface.

### Leave execution inside `dispatch_gewc_cycle`
Rejected. That kept the cycle host too shallow and left too much long-term authority in the former runtime shape.

### Make `global_executive_workspace.rs` contain all execution arms
Rejected. GEWC should own the execution interface and authority model, but a single giant file would reduce locality and increase regression risk.

## Consequences
- `dispatch_gewc_cycle` is now a cycle host instead of the physical command executor.
- `GewcBodyRegistry` owns binding and `GewcBodyExecutor` owns execution.
- Runtime reports include `body_executor=gewc_body_executor`.
- Capability validation requires the GEWC body executor marker and a real last handler.
- Future phases can extract per-body handlers from `GewcBodyExecutor` without changing the public command surface.
- This remains a no-claim architecture integration change.
