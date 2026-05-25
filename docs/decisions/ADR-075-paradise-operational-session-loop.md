# ADR-075: Paradise Operational Session Loop

## Status
Accepted

## Date
2026-05-25

## Context
ADR-074 introduced Paradise as the public Worldcell Runtime identity, but the
first implementation only generated a runtime identity artifact. That was useful
for positioning and evidence, but it did not yet prove the product idea as an
operational loop.

The next step needs to show that Paradise can receive an intent and move it
through context, dry-run, approval, execution and evidence without creating a
second runtime beside GARM/GEWC.

## Decision
Add a native Paradise session loop:

- `paradise intent <text>`
- `paradise plan`
- `paradise approve`
- `paradise execute`
- `paradise sessions`

The loop writes `paradise_worldcell_sessions.json` with schema
`eden-paradise-worldcell-session-v1`.

Paradise sessions are not a separate controller. They are GEWC-routed planning
commands that compose existing EDEN mechanisms:

- Locus for context and authority.
- Operational API dry-run for command classification.
- World model for consequence simulation.
- Operator Forge for action-contract evidence.
- GEWC for final runtime dispatch.
- Action evidence and learning ledger for post-action traceability.

Execution remains conservative. A session requires explicit approval and still
blocks commands whose dry-run says they mutate runtime state, require
supervision or are not standalone-safe.

## Alternatives Considered

### Separate Paradise Runtime
Rejected. It would split authority from GEWC and reintroduce the two-core
problem the project has been removing.

### Prompt-Only Paradise Flow
Rejected. It would look good in docs but would not create executable artifacts,
approval records or replayable evidence.

### Let Approval Override All Dry-Run Gates
Rejected. Operator approval should be necessary for execution, but it should not
override safety classification for commands that need stronger runtime authority.

## Consequences

- Paradise now has a real operational artifact, not only a positioning artifact.
- The operator console and APIs can read `/api/paradise/sessions`.
- Existing GARM/GEWC authority remains intact.
- Risky or mutating actions are still blocked until future explicit execution
  policies are designed and tested.
