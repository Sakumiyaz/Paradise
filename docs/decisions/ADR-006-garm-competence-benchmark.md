# ADR-006: GARM Competence Benchmark Harness

## Status

Accepted.

## Context

GARM already exposes local seams for goals, evaluation, learning and world modelling. Those reports are useful independently, but operators also need a reproducible local harness that checks whether the seams are present together without introducing CI, network calls, external models or a separate runtime.

## Decision

Add a GARM-native competence benchmark node with commands `bench`, `bench run` and `bench audit`. The benchmark inspects local evidence only: goal contracts, evaluation records, learning entries, world predictions, organ autonomy and HRM reasoning. Runs are persisted to `competence_benchmark.json`, included in reports, audits, exports and artifact listings, and `bench run` writes a learning ledger trace.

## Consequences

- Benchmark state is local, deterministic and auditable.
- The harness measures operational competence seams; it does not assert external general intelligence.
- Evaluation can consume the benchmark report as execution evidence.
- No CI, network access, LLM or GPU dependency is introduced.
