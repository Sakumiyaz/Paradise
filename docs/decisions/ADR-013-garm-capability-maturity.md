# ADR-013: GARM Capability Maturity Ledger

## Status

Accepted.

## Context

GARM has many local seams, but operators need a compact way to track whether a capability is merely present, developing or operational. This should be local evidence, not a claim of external certification.

## Decision

Add a GARM-native capability maturity ledger with commands `maturity`, `maturity assess TEXT` and `maturity audit`. Assessments compute a deterministic level from local GARM evidence and persist to `capability_maturity.json`. Maturity reports are included in audits, reports, exports and artifact listings, and assessments write learning ledger traces.

## Consequences

- Capability readiness becomes auditable local state.
- Evaluation and benchmarks can use maturity coverage as evidence.
- The seam is deterministic and local: no network, shell execution, LLM, CI or separate runtime.
- Maturity levels are operational heuristics, not external proof.
