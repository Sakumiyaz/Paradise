# ADR-009: GARM Uncertainty And Risk Ledger

## Status

Accepted.

## Context

GARM can plan, execute, evaluate, learn, model observations, benchmark competence and track working focus. It also needs an explicit way to represent uncertainty and risk so local autonomy can distinguish low-confidence claims from supported evidence.

## Decision

Add a GARM-native uncertainty ledger with commands `uncertainty`, `uncertainty record TEXT`, `uncertainty resolve` and `uncertainty audit`. Records contain a claim, source, deterministic confidence estimate, risk class, mitigation and status. State is persisted to `uncertainty_ledger.json`, included in reports, exports and artifact listings, and manual records write learning ledger traces.

## Consequences

- Risk and unknowns become auditable local state rather than hidden text in logs.
- Mitigation is conservative and local: high risk blocks until verified, medium risk requires local evidence.
- Evaluation and benchmarks can use uncertainty evidence for calibration.
- No LLM, network, shell execution, CI or separate runtime is introduced.
