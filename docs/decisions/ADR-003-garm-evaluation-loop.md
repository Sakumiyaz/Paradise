# ADR-003: GARM Evaluation Loop

## Status
Accepted

## Date
2026-05-20

## Context
GARM now has organ autonomy, HRM, CAG, a goal scheduler and operational reports. The next architectural risk is self-deception: the runtime can plan and execute local goals, but without a persisted evaluation loop it cannot compare architecture progress, evidence quality, execution reliability or learning signals over time.

The project constraints remain unchanged: Rust-only, local-first, no LLM dependency, no autonomous remote crawler, and GARM remains the single runtime surface.

## Decision
Add a GARM-native evaluation loop as the measurement seam for architecture progress.

- `evaluation_loop` records bounded evaluation snapshots in `evaluation_loop.json`.
- `eval`, `eval run` and `eval audit` expose evaluation through existing GARM commands.
- `eval run` scores four local dimensions: architecture, evidence, execution and learning.
- Inputs come from existing local state: memory facts, KG edges, graph shape, readiness score, goal scheduler report, organ audit, HRM snapshot and benchmark snapshot.
- `garm audit`, `garm report`, `garm export`, state reports and artifact indexes include evaluation state.

## Alternatives Considered

### External Benchmark Harness First
Rejected for this phase. External benchmarks are useful later, but GARM first needs a local persisted seam that can be called from runtime commands, smokes and reports.

### Fold Evaluation Into ReadinessNode
Rejected because readiness estimates and evaluation records are different concepts. Readiness summarizes capability gaps; evaluation tracks architectural progress and regression risk over time.

### Fold Evaluation Into Goal Scheduler
Rejected because goals describe intent and action contracts. Evaluation measures outcomes and regression risk across goals, organs, HRM, CAG and benchmark signals.

## Consequences

- EDEN/GARM now has an auditable local measurement loop, not just planning/execution traces.
- Scores are heuristic and local; they are not proof of external competence and must not be treated as external competence benchmarks.
- Future external benchmark adapters can write behind this seam without changing the public command surface.
