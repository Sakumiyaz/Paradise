# ADR-012: GARM Policy Guard And Constraint Ledger

## Status

Accepted.

## Context

GARM has execution, experiment, uncertainty and provenance seams, but constraint decisions should be explicit rather than implicit in command handlers. A local policy guard makes safety gates auditable without changing GARM into a separate policy runtime.

## Decision

Add a GARM-native policy guard with commands `policy`, `policy eval TEXT` and `policy audit`. Decisions include action, verdict, matched rule and reason. State is persisted to `policy_guard.json`, included in reports, exports and artifact listings, and manual evaluations write learning ledger traces.

## Consequences

- Constraint decisions become auditable local state.
- Rules stay conservative: local-only, no shell execution, no remote network, no code mutation.
- Evaluation and benchmark scoring can account for policy coverage.
- No LLM, network, shell execution, CI or separate runtime is introduced.
