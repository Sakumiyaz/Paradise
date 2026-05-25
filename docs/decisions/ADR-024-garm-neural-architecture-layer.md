# ADR-024: Formalize the GARM Neural Architecture Layer

## Status
Accepted

## Date
2026-05-22

## Context
EDEN/GARM includes native neural components such as transformers, BigTransformer, MoE, BPTT, DNC, semantic embeddings and HRM-text pretraining manifests. These were visible in runtime status and HRM-text reports, but not represented as a release artifact.

## Decision
Add `neural eval` and `neural_architecture.json`. The evaluation checks local evidence for:

- transformer and BigTransformer backbone
- neural memory and expert routing
- semantic embedding bridge
- HRM-text checkpoint manifest
- retrieval/inference loop
- explicit weight policy

The artifact records `weights_present=false` and `training_executed=false` when only the local prior/checkpoint manifest exists. It is included in the readiness package, capability registry and independent validator.

## Consequences
Neural AGI is now a formal layer of the Paradise architecture without overstating the current state. The layer validates neural architecture readiness locally while preserving `claim_allowed=false`.
