# ADR-091: EDEN v0.4 Operational Capability Stage

## Status
Accepted

## Date
2026-05-26

## Context
EDEN v0.3 proved a governed 7B-shape path on AMD ROCm: train longer than the
initial pilot, write a checkpoint, reload it, run token generation and admit the
checkpoint only as a GEWC-subordinate candidate. The next step needs to raise the
bar without overstating capability.

The operator asked for seven GPU-backed processes:

- train the 7B beyond the pilot;
- evaluate generated outputs more directly;
- include EDEN cognitive fine-tuning evidence;
- apply a harder checkpoint admission gate;
- prepare native persistent inference under GEWC;
- preflight 14B scaling;
- test continuity of memory, objectives and policy boundaries.

## Decision
Add a v0.4 stage with its own corpus, evaluator, schemas and ROCm stage script.
The default 7B target is 10,000 iterations. A checkpoint can be admitted only as
a local candidate when all v0.4 gates pass. Production inference, autonomous
authority and AGI claims remain blocked.

v0.4 uses EDEN-owned repo data and local ROCm/Megatron execution. The model is
treated as a subordinate hypothesis generator. Memory writes, objective updates,
tool execution and production release still require GEWC authorization and
external review.

## Consequences
- The 7B path now has a measurable promotion ladder from 1k to 10k iterations.
- Generative probes are tracked separately from semantic competence claims.
- Cognitive SFT evidence is incorporated without allowing checkpoint admission
  by itself.
- Persistent inference is represented as a guarded service contract before it is
  allowed to become a resident production daemon.
- The 14B path is explicitly capped at 14B dense parameters unless a future ADR
  changes that boundary.
- Checkpoints stay out of git; only manifests and evidence reports may be
  committed.

## Non-Goals
- This stage does not claim AGI.
- This stage does not release a production model.
- This stage does not upload private data or use external model weights.
- This stage does not authorize the model to mutate memory, objectives or tools.
