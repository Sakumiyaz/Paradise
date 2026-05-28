# ADR-097: Finalize Paradise Release Validation Gates

## Status
Accepted

## Date
2026-05-28

## Context
Paradise already exposed non-GPU readiness, public contracts, checkpoint
registry review, dry-run checkpoint admission and public release packaging.
That was enough to inspect the boundary, but it still left three gaps:

- checkpoint admission had a dry-run artifact but no single real gate that could
  flip only when all evidence exists;
- native inference status did not consume checkpoint admission state;
- CI uploaded release evidence but did not expose the strong eval and external
  validation package as first-class artifacts in the non-GPU readiness job.

GPU training and checkpoint weights remain intentionally outside this change.

## Decision
Add `paradise_checkpoint_admission_gate.json` as the real admission decision
artifact. The gate reads the public checkpoint registry, evidence review,
semantic eval, strong eval, non-GPU readiness, public contracts, inference probe
status and release package. It requires active checkpoint selection, an admitted
registry entry, checkpoint hash review, held-out eval, safety eval, rollback
verification and operator release approval before
`checkpoint_admission_allowed=true`.

The existing dry-run command now delegates to the same gate but always writes
`checkpoint_admission_allowed=false` and records whether the real gate would
allow admission. The Eden 70B modular inference runtime now reports inference
availability from the admission gate while still loading no checkpoint in the
status command.

CI now runs the checkpoint admission gate smoke, strong eval and external
validation package in the non-GPU readiness job and uploads their public
artifacts. The local benchmark feature now exercises runtime state and release
artifact catalog construction rather than a purely synthetic tick loop.

## Consequences
- Public release evidence has a direct checkpoint admission gate instead of only
  a blocked dry-run.
- Native inference status remains blocked today, but it is connected to the
  same evidence path that will admit future checkpoints.
- No checkpoint, private data, GPU VM state or model weight is committed.
- The public operator console can show dry-run checks, gate checks and native
  inference availability from live artifact APIs.
