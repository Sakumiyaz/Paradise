# ADR-007: GARM Plan Executor With Rollback Scoring

## Status

Accepted.

## Context

GARM can plan goals, evaluate architecture, learn from outcomes, model local observations and run competence benchmarks. The missing seam is an explicit execution ledger that can decide whether a local plan has enough evidence to commit, or should be rolled back without external side effects.

## Decision

Add a GARM-native plan executor with commands `exec`, `exec plan TEXT`, `exec run` and `exec audit`. Plans are local records with bounded steps. `exec run` scores local reports from CAG, organs, goals, evaluation, learning and benchmark state, then marks a plan `completed` or `rolled_back`. State is persisted to `plan_executor.json`, included in reports, exports and artifact listings, and execution attempts write learning ledger traces.

## Consequences

- Execution outcomes become auditable and reproducible.
- Rollback is conservative and local: no shell execution, code mutation, remote network or separate runtime.
- Benchmark and evaluation can use executor state as evidence of controlled action.
- This remains an operational seam, not a claim of autonomous general intelligence.
