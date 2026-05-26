# ADR-087: EDEN Real-Capability Stage

Date: 2026-05-26

## Status

Accepted

## Context

EDEN now has a governed 7B training/inference evidence path and a compact GPU
SFT/ELCP learned-capability pilot. The next step is not another architecture
artifact; it is a stricter operational stage that connects real repo-owned data,
bounded 7B evidence, inference, evaluation, checkpoint decisioning, demo traces
and scaling policy.

## Decision

Add the `eden real capability eval` command and the supporting training target
`make training-eden-real-capability-stage`.

The stage has seven artifacts:

1. repo-owned capability corpus manifest;
2. 7B training evidence admission;
3. integrated inference bridge;
4. operational capability eval;
5. checkpoint decision;
6. governed operational demo;
7. scaling ladder.

The corpus is built from EDEN documentation, ADRs, training configs and runtime
source excerpts. No private data, external model weights or network calls are
used. The checkpoint can become reviewable, but it is not admitted automatically.

## Consequences

- EDEN now has a concrete path from architecture to evidence-backed capability
  staging.
- The gate can pass while still keeping `checkpoint_admission_allowed=false`.
- Future GPU runs must beat this stage on eval score, safety and rollback before
  review.
