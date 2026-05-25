# ADR-056: Operational Runtime Command Controls

## Status
Accepted

## Date
2026-05-24

## Context
The operational runtime phase produced local evidence for tasks, actions,
lifecycle control, memory transactions and replay, but those capabilities were
mostly artifact generation. That made the architecture inspectable, but the next
runtime step needed explicit commands that could mutate local state through GEWC
instead of remaining passive records.

The constraint is that these commands must not create a second kernel, bypass
GEWC, or claim completed AGI capability. They should exercise current runtime
surfaces only: task state, safe command dispatch, memory transaction ledgers,
module lifecycle control and replay.

## Decision
Add GEWC-routed operational commands for the five runtime controls:

- `operational task submit <objective>`, `operational task run` and
  `operational task audit`;
- `operational action execute <command>`;
- `operational memory commit <text>` and
  `operational memory rollback <transaction_id>`;
- `operational replay run`;
- `operational smoke run`;
- `operational scenario run`;
- `gewc lifecycle <handler> <action>`.

Each command is parsed by `CommandRouterNode`, classified by
`GewcBodyRegistry`, executed by a domain-owned GEWC body handler and recorded in
the existing operational artifacts or GEWC runtime log. The action executor only
dispatches commands classified as non-mutating and non-supervised by the action
contract dry-run; higher-risk commands are recorded as blocked action evidence.
The operational API exposes the permission matrix as read, local mutation,
external tool, destructive/autonomous and clarification-required classes.

`operational smoke run` executes the current local chain:
task submit, task run, task audit, safe action dry-run, memory commit, memory
rollback, lifecycle pause/resume and replay. `operational scenario run` records
the longer evidence path for incomplete information, blocked high-risk action,
world-model observe/predict/verify, memory rollback, lifecycle repair and replay.

## Alternatives Considered

### Keep The Runtime Phase Artifact-Only
Pros: lower implementation risk.
Cons: it leaves task, action, memory, lifecycle and replay as generated evidence
rather than runtime controls.
Rejected because the repo now needs operational API behavior that can be tested
through the normal command path.

### Add Separate REST Mutation Endpoints
Pros: familiar API shape.
Cons: creates another control plane and duplicates GEWC authority.
Rejected because mutation should remain command-routed through GEWC. Existing
`/api/command` and `/api/command_sync` can execute the same commands.

### Allow Action Executor To Run Any Nested Command
Pros: more flexible.
Cons: unsafe; it could bypass supervision for mutating or high-risk commands.
Rejected. The executor gates nested commands through the existing action
dry-run contract and records blocked attempts.

## Consequences
- The operational runtime now has executable local controls, not only artifacts.
- GEWC remains the sole authority for command routing and lifecycle state.
- Memory writes are versioned local transactions with rollback markers, not
  model-weight updates.
- Action execution is intentionally conservative until stronger permission and
  supervision policies exist.
- The no-claim boundary remains unchanged.
