# ADR-018: Gate Phase 6 On External Validation

## Status
Accepted

## Date
2026-05-22

## Context
EDEN/GARM can produce high local readiness scores from auditable runtime evidence: memory/KG, RAG citations, provenance, uncertainty, policy gates, benchmarks, goals, and governed autonomy cycles. Those scores are useful engineering signals, but they are not an external measurement of AGI or general intelligence.

Phase 6 must therefore remain separate from local readiness automation. Completing it locally would create a misleading claim because the operators define the tasks, scoring, and evidence probes.

## Decision
Add a Phase 6 external-validation manifest command, `readiness external`, that:

- Reports the current local readiness score as a local engineering metric.
- Writes `external_validation_manifest.json` under the GARM state directory.
- Declares `claim_allowed=false` and `agi_claim=false`.
- Lists required independent validation suites and artifacts.
- Does not mark Phase 6 complete.

Phase 6 can only be completed after independent external task suites, predeclared scoring, reproducibility review, and safety red-team evaluation.

## Alternatives Considered

### Complete Phase 6 From Local Score
Rejected. Local score saturation proves the internal gates can be exercised, not that EDEN generalizes beyond operator-authored evidence.

### Remove Phase 6 From The Runtime
Rejected. Operators need a concrete handoff artifact for evaluators, and the runtime should make the no-claim policy explicit.

### Add External Validation Manifest Only In Documentation
Rejected. A runtime-generated manifest is more auditable because it is tied to the current state directory and gate snapshot.

## Consequences

- EDEN/GARM can prepare a reproducible validation packet without claiming AGI.
- Phase 6 remains blocked until external validators run independent suites.
- The local readiness score remains useful as architecture readiness, not as a scientific AGI measurement.
