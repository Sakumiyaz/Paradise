# ADR-086: EDEN SFT/ELCP GPU Learned-Capability Pilot

Date: 2026-05-26

## Status

Accepted

## Context

EDEN already had architecture, model-runtime governance, 7B checkpoint evidence
surfaces and operational no-claim gates. The missing step was a small, real
learned-capability loop that could run on the AMD ROCm GPU without depending on
external model weights or private data.

## Decision

Add a deterministic SFT/ELCP v2 dataset generator and a ROCm GPU pilot that
trains a compact multi-head cognitive-transition module. The pilot writes:

- GPU training report;
- pre/post evaluation;
- repeated inference packets;
- checkpoint admission review.

GEWC admits those records through `eden learned capability eval` as evidence
only. The trained module remains a hypothesis generator; it cannot write memory,
change objectives, execute tools or admit a checkpoint.

## Consequences

- EDEN gains a repeatable path from dataset to GPU training to governed runtime
  evidence.
- The path remains EDEN-owned: synthetic repo data, no external checkpoints and
  Docker `--network none`.
- Checkpoint admission remains blocked until a separate GEWC review, adversarial
  evaluation, rollback drill and operator approval exist.
