# ADR-031: Make GEWC the Operational GARM Runtime Core

## Status
Superseded by ADR-032

## Date
2026-05-22

## Context
ADR-030 introduced `Global Executive Workspace Core` as a formal architecture artifact. That was not enough: the runtime still parsed commands and dispatched them directly through `GarmRuntime::dispatch_command`, leaving GEWC as a report rather than the actual decision point. The architecture also still read as two nuclei: native GARM modules and migrated legacy nodes.

## Decision
Make `GlobalExecutiveWorkspaceCore` the operational command decision authority.

Every non-shutdown parsed `GarmCommand` now persists a GEWC decision trace after passing through:

```text
CommandRouterNode::parse_raw
        -> GlobalExecutiveWorkspaceCore::decide
        -> GlobalExecutiveWorkspaceCore::record_decision
        -> governed dispatch
```

The core classifies each command by objective, route, target plane, selected modules, safety gate, supervision requirement and disposition. Legacy surfaces remain available only as compatibility adapters under the core:

```text
legacy_mode=compatibility_adapter_not_separate_core
garm_mode=native_modules_under_executive_workspace
```

The runtime writes:
- `global_executive_workspace_runtime.jsonl` for every core decision;
- `global_executive_workspace_runtime_state.json` for the latest decision state;
- `[GEWC-RUNTIME]` in the evaluation output.

`Quit` still passes through parsing and shutdown handling, but it is not persisted as a cognitive decision because it mutates artifacts after `readiness package` has written hashes.

The reproducible package and independent validator now require these runtime artifacts. The capability registry validates both the formal GEWC artifact and the operational runtime trace.

## Alternatives Considered

### Keep GEWC as an evaluation layer
Rejected. This left the core partial and did not remove the practical split between GARM dispatch and legacy adapters.

### Rewrite all legacy nodes into native modules immediately
Rejected. That would be larger and riskier than necessary. The architectural requirement is one operational nucleus; legacy code can remain as adapters while control authority moves to GEWC.

### Put the logic in `CommandRouterNode`
Rejected. The router should parse intent. It should not own goals, safety, module selection, workspace routing and governance.

## Consequences
- GEWC is now in the runtime path for every command, not just a validation artifact.
- Legacy and native GARM paths are unified under one executive workspace decision record.
- Release candidates fail if the operational GEWC trace is missing.
- The runtime remains no-claim: this is core integration, not an AGI capability claim.
