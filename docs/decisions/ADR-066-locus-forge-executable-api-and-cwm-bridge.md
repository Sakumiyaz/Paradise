# ADR-066: Expose Locus and Operator Forge Through Executable APIs and a Governed CWM Bridge

## Status
Accepted

## Date
2026-05-24

## Context
ADR-065 made Eden Locus Layer and Eden Operator Forge native GEWC handlers.
That was necessary, but still left a practical risk: a user or external SDK
could inspect the artifacts, yet the two domains were not first-class operator
commands in `edenctl` and their relationship with CWM was only implicit.

The runtime needs a small, auditable seam where personal context and formal
candidate synthesis can be exercised from the public API surface without letting
either domain write directly into memory, objectives or model weights.

## Decision
Expose Locus and Operator Forge through the existing command-routed API surface:

- SDK helpers for Locus eval, ingest, context and audit.
- SDK helpers for Operator Forge eval, synth, verify and audit.
- `edenctl locus ...` and `edenctl forge ...` commands that call the same
  `command_sync` path as other governed runtime actions.
- Black-box and SDK conformance checks that execute the commands, read runtime
  states, read artifacts and verify dry-run permission classification.

Extend `operational runtime eval` from seven to eight runtime components by
adding `locus_operator_bridge.json`. The bridge records:

- Locus authority/context evidence.
- Operator Forge graph synthesis and verification evidence.
- CWM observation/prediction/verification output.
- A policy boundary that treats the result as a hypothesis only.

## Alternatives Considered

### Add standalone HTTP mutation endpoints
Pros: shorter client calls.
Cons: would bypass the existing command queue, GEWC dispatch, lifecycle and
permission machinery.
Rejected.

### Let Locus write directly to memory and Forge write directly to CWM
Pros: fewer artifacts.
Cons: weak provenance, difficult rollback and unclear authority.
Rejected.

### Keep bridge behavior only in docs
Pros: no runtime change.
Cons: does not prove operational integration.
Rejected.

## Consequences
- Locus and Operator Forge are executable through the same GEWC-governed
  interface as other runtime actions.
- The SDK, CLI, black-box scripts and conformance runner now cover these
  domains.
- CWM receives only governed hypotheses from the bridge; memory writes still
  require an operational memory command or future memory-handler transaction.
- The no-claim boundary remains explicit: bridge evidence is local runtime
  integration evidence, not AGI validation or scientific proof.
