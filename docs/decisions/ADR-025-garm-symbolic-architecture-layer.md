# ADR-025: Formalize the GARM Symbolic Architecture Layer

## Status
Accepted

## Date
2026-05-22

## Context
EDEN/GARM includes a hypergraph runtime, semantic symbol inventory, formal logic reasoning, goal contracts, policy gates, provenance records and causal/world-model structures. These are core symbolic AGI components but were not represented as one release artifact.

## Decision
Add `symbolic eval` and `symbolic_architecture.json`. The evaluation checks local evidence for:

- hypergraph symbol table
- formal logic reasoning
- explicit goal/policy contracts
- provenance traceability
- causal world symbols
- neuro-symbolic bridge

The artifact is included in the readiness package, capability registry and independent validator.

## Consequences
Symbolic AGI is now a formal layer of the Paradise architecture. The layer validates local symbolic structure and traceability while preserving `claim_allowed=false`.
