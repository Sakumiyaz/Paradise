# ADR-088: EDEN v0.1 Capability Stage

Date: 2026-05-26

## Status

Accepted

## Context

The first real-capability stage proved that EDEN can build a repo-owned corpus,
train a 7B-shape checkpoint for 50 iterations, load that checkpoint, generate
tokens and keep checkpoint admission blocked. That is pipeline evidence, not a
useful semantic runtime.

The next stage needs to move beyond the pilot without changing the dense-model
ceiling. EDEN's maximum dense model target remains 14B; near-term progress
should come from pretraining quality, curated data, semantic evaluation and GEWC
integration rather than parameter growth.

## Decision

Add the `eden v01 capability eval` command and the supporting target
`make training-eden-v01-stage`.

The v0.1 stage requires:

1. a larger curated semantic corpus;
2. semantic capability evaluation;
3. 7B training beyond the 50-iteration pilot;
4. native inference runtime candidate admission;
5. checkpoint candidate admission while production release remains blocked;
6. a non-mutating operational demo trace;
7. a scaling plan with a 14B dense ceiling;
8. GPU workspace hygiene evidence.

The stage may admit a checkpoint as a GEWC-subordinate candidate generator only
when the evidence passes. It still does not admit a production model, autonomous
tool authority or AGI claims.

## Consequences

- EDEN now has a concrete transition from pipeline proof to semantic runtime
  candidate.
- The checkpoint admission process becomes real but scoped: candidate runtime
  admission can pass, while production release remains blocked.
- Future scaling must compare against this v0.1 gate and must not exceed 14B
  dense parameters without a new ADR.
