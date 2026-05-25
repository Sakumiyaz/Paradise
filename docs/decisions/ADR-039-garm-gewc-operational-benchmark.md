# ADR-039: Add GEWC Operational Benchmark Artifacts

## Status
Accepted

## Date
2026-05-22

## Context
GEWC is now the single runtime authority for GARM and legacy-compatible command bodies. The next architectural risk is not another named paradigm layer, but whether the integrated core can produce repeatable operational evidence for routing, governed autonomy, safe learning, runtime safety, traceability and restart stability.

The existing readiness package already includes formal architecture artifacts and local held-out validation. It did not yet include a GEWC-specific operational benchmark artifact set that separates architecture existence from operational behavior.

## Decision
Add `gewc operational benchmark` as a first-class validation command.

It writes three no-claim local prevalidation artifacts:

- `gewc_operational_benchmark.json`
- `gewc_runtime_safety_report.json`
- `gewc_long_run_stability.json`

The benchmark is included in the release-candidate command sequence, readiness package, independent package validator, external validation evidence and capability registry. It remains explicitly local evidence and preserves `claim_allowed=false` and `agi_claim=false`.

## Alternatives Considered

### Keep GEWC Validation Inside `global executive workspace eval`
- Pros: fewer commands and fewer artifacts.
- Cons: conflates structural architecture validation with operational behavior.
- Rejected: GEWC needs a separate measurable surface for runtime safety and stability.

### Add More Architecture Layers
- Pros: could model additional concepts.
- Cons: increases nominal complexity without improving evidence quality.
- Rejected: the next phase should strengthen measurement, not taxonomy.

### Only Use Existing Competence Benchmark
- Pros: reuses current benchmark harness.
- Cons: does not specifically validate GEWC authority, handler topology, safety reports or long-run stability artifacts.
- Rejected: competence benchmarking remains useful but too broad for GEWC-specific operational evidence.

## Consequences
- Release candidates now require GEWC operational artifacts.
- Local held-out validation includes an operational core benchmark criterion.
- Capability registry gains `gewc_operational_benchmark`.
- The architecture remains no-claim: artifacts show local readiness evidence, not external AGI validation.
