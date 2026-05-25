# ADR-073: Harden ELCP Evidence Before Any Training Execution

## Status
Accepted

## Date
2026-05-25

## Context
EDEN now has an ELCP preparation gate that validates transition fixtures,
runs a CPU baseline, exports GEWC traces, dry-runs the future training entry
point and blocks checkpoint admission. That is enough to prove the preparation
path exists, but not enough to justify real GPU training.

Before training, ELCP needs evidence that traces are clean, replayable,
measurable, frozen by hash and governed by an explicit 4B contract.

## Decision
Add an ELCP hardening phase before any 4B training request:

- trace quality gate for exported GEWC candidates;
- replay evaluation over reviewed traces;
- dataset freeze manifest with hashes and split roles;
- metrics board aggregating validation, baseline, replay, freeze and admission;
- 4B readiness contract that lists required approvals while keeping training
  blocked.

The hardening phase is exposed through `make elcp-hardening` and the runtime
command `elcp hardening`. It writes both local reports under `target/eden_elcp/`
and GEWC-native artifacts under the runtime state dir.

## Alternatives Considered

### Train directly after admission gate
Rejected. The admission gate was a policy boundary, not a dataset-quality or
replay-evaluation boundary.

### Keep hardening as Python-only reports
Rejected. EDEN's public architecture is runtime/API oriented, so hardening
evidence must become GEWC-native artifacts discoverable through the artifact
API and release package.

### Lock traces as training data immediately
Rejected. Runtime traces are candidate evidence only until privacy review,
operator approval and dataset-lock policy are explicit.

## Consequences

- `elcp prepare` now expects the hardening reports to exist before full
  readiness passes.
- `elcp readiness` remains no-claim and still blocks 4B training.
- Future GPU work has a clearer contract: approve budget, lock dataset for
  training, admit checkpoint path, define rollback and run post-train eval.
