# ADR-017: GARM Readiness Architecture Gates

## Status
Accepted

## Date
2026-05-21

## Context
EDEN/GARM needs a path toward increasingly general capability without confusing readiness measurement with an identity claim. The runtime already has HRM, RAG, CAG, KG, provenance, learning, policy, uncertainty, benchmark and organ autonomy components. These need to be measured as one architecture rather than as isolated features.

## Decision
Define readiness as an auditable gate framework for EDEN architecture. The `readiness` command reports learning, planning, grounding, world modeling, memory, self-correction, generalization, scaling, verifiable RAG, operational safety, continuous evaluation and governed autonomy.

The readiness report includes explicit blockers and next actions. `readiness plan` converts those next actions into goal-scheduler contracts with local evidence requirements, and `readiness run` evaluates readiness contracts against current local evidence before marking them completed. Missing evidence blocks the contract instead of treating planning as success. It preserves the invariant `no_claim_until_all_gates_pass` and keeps GARM local-first, deterministic and evidence-driven.

## Consequences
- Readiness progress becomes measurable without overstating capability.
- RAG, safety, evaluation and autonomy signals contribute directly to readiness scoring.
- Low scores produce concrete next actions instead of vague roadmap language.
- Next actions become auditable goal contracts instead of remaining report-only diagnostics.
- Readiness execution can expose missing evidence as blocked contracts rather than false-positive completion.
- External validation remains required before any capability claim beyond the local evidence.

## Alternatives Considered

### Claim General Intelligence Based On Runtime Complexity
Rejected. Complexity, autonomy and reasoning traces are not sufficient evidence for identity-level claims.

### Keep Readiness As A Static Gap List
Rejected. Static gaps do not integrate new RAG, safety, evaluation and autonomy evidence.

### Replace GARM With A Model-Centric Stack
Rejected. That would break EDEN's local-first, no-LLM-core and auditability constraints.
