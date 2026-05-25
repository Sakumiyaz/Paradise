# ADR-019: Harden Readiness Validation Against Self-Scoring

## Status
Accepted

## Date
2026-05-22

## Context
GARM readiness can generate strong local evidence through probes, local ledgers, RAG context packs, policy gates, benchmarks and governed autonomy cycles. That is useful engineering instrumentation, but combining evidence generation and measurement in one command makes the score too easy to saturate with operator-authored data.

Phase 6 also needs more than a manifest. It needs a reproducible handoff packet and a first local held-out harness while still preserving ADR-018: no local command may claim independent external validation or AGI.

## Decision
Separate readiness measurement from local evidence generation:

- `readiness bench` measures current evidence only.
- `readiness probe` explicitly generates local phase evidence.
- `readiness external run` executes a small local held-out harness and writes `external_validation_result.json`.
- `readiness package` writes a reproducible artifact bundle with hashes in `readiness_package.json`.
- `action evidence` reports a unified intent -> policy -> execution -> consequence evidence log in `action_evidence.jsonl`.

The held-out harness includes negative memory/abstention, safety blocking, action trace, planning, RAG and generalization checks. It is a local pre-validation harness, not independent certification.

## Consequences

- Local readiness scores are less self-referential because measurement and probe generation are separate commands.
- Phase 6 has an executable local harness and reproducible package artifacts.
- Action attempts become easier to audit across policy, execution, consequence and uncertainty.
- External validation remains required before any AGI or general-intelligence claim.

## Alternatives Considered

### Keep `readiness bench` As A Probe Generator
Rejected because it hides evidence generation behind a measurement command and weakens trust in the score.

### Treat The Held-Out Harness As External Certification
Rejected. The harness is still shipped with GARM and cannot substitute independent task authorship, scoring and red-team review.
