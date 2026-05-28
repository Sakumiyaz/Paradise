# ADR-096: Paradise Non-GPU Validation Deepening

## Status
Accepted

## Date
2026-05-28

## Context
Paradise now has public contracts, a quickstart CLI, checkpoint-registry review
and a release package. GPU training remains paused, so the repo still needs a
stronger local validation path that makes operational capability boundaries
visible without pretending that checkpoint capability exists.

The prior non-GPU gate checked that the public surface was coherent. It did not
compose the evidence into module-family results, and it did not generate a
single external-review manifest.

## Decision
Add three no-claim validation surfaces:

- A blocked checkpoint admission dry-run artifact exposed by the native
  `paradise checkpoint dry-run-admit` command.
- A `paradise_strong_eval` report that composes module semantic coverage,
  dataset governance, contracts, readiness and checkpoint evidence review into
  family-level evidence.
- A `paradise_external_validation_package` manifest that lists public commands,
  artifacts and excluded confidential material for outside reviewers.

These surfaces remain non-GPU. They must not admit checkpoints, publish weights,
authorize production model use or strengthen AGI claims.

## Consequences

- Operators can inspect why checkpoint admission is still blocked without
  reading several separate artifacts.
- External reviewers receive one non-confidential package manifest instead of a
  loose list of files.
- The public console can display stronger readiness state while preserving the
  same GEWC authority boundary.
- Real model capability evaluation still depends on future admitted checkpoints
  and held-out learned-model benchmarks.
