# ADR-033: GEWC Owns the Runtime Cycle

## Status
Accepted

## Date
2026-05-22

## Context
ADR-032 made GEWC the owner of native and legacy runtime body domains, but the implementation still exposed a practical split: `GarmRuntime::dispatch_command` parsed and executed commands, while `GlobalExecutiveWorkspaceCore` recorded only the pre-dispatch decision. That left too much of the runtime shape in the former GARM surface and made GEWC look like a called submodule instead of the owning cognitive runtime.

The desired architecture is stricter: GEWC must own the cognitive cycle around runtime work, including decision, blocked/deferred outcomes, execution completion and trace state.

## Decision
Rename the runtime command path to `dispatch_gewc_cycle` and make each eligible command run inside a GEWC-owned lifecycle:

```text
CommandRouterNode::parse_raw
        -> GlobalExecutiveWorkspaceCore::decide
        -> record decision_started
        -> execute absorbed GEWC body domain
        -> record execution_completed
        -> update GEWC runtime state
```

`global_executive_workspace_runtime.jsonl` now records both `decision_started` and `execution_completed` phases. `global_executive_workspace_runtime_state.json` stores the latest decision and completion record. `[GEWC-RUNTIME]` reports both decision and completion counts.

`Quit` is still excluded from persistence because shutdown can follow a release package. `ReadinessPackage` records its pre-package decision but skips post-package completion so the package does not hash an artifact and then mutate it immediately afterward.

## Alternatives Considered

### Keep a decision-only GEWC trace
Rejected. This preserved the old shape where GEWC authorized work but did not own the runtime cycle.

### Move the full command match into `global_executive_workspace.rs` immediately
Rejected for this step. It would create a very large risky move without improving the observable authority model more than the lifecycle refactor. The runtime cycle is now GEWC-owned; future file moves can happen gradually without changing behavior.

### Persist shutdown and package completions
Rejected. Those commands can mutate release artifacts after reproducible hashes are written, so they remain intentionally excluded from post-execution cycle persistence.

## Consequences
- GEWC no longer records only intent; it records execution lifecycle closure.
- Former GARM and legacy bodies are executed as absorbed GEWC body domains with explicit start and completion traces.
- Capability and package validation can distinguish missing runtime decisions from missing execution completions.
- The file layout may still contain historical names, but the operational cycle is now named and traced as GEWC-owned.
- The runtime remains no-claim: lifecycle ownership is an integration property, not an AGI capability claim.
