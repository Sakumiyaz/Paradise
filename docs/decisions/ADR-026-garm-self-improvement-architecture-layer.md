# ADR-026: Formalize the GARM Self-Improvement Architecture Layer

## Status
Accepted

## Date
2026-05-22

## Context
EDEN/GARM includes bounded self-modification, self-improvement audits, policy gates, provenance records, uncertainty tracking, action evidence and a rollback-capable local executor. These are the local building blocks for self-improving AGI, but they were not represented as one release artifact.

## Decision
Add `self improvement eval` and `self_improvement_architecture.json`. The evaluation checks local evidence for:

- bounded parameter proposals
- self-improvement audit loops
- policy blocks for source/code mutation
- rollback-capable execution
- provenance and uncertainty traces
- explicit no-source-mutation evidence

The artifact is included in the readiness package, capability registry and independent validator.

## Consequences
Self-Improving AGI is now a formal layer of the Paradise architecture. The layer validates local bounded improvement behavior while preserving `claim_allowed=false`, `agi_claim=false` and `source_code_mutation_allowed=false`.
