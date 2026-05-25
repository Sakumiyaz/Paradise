# ADR-002: GARM Goal Scheduler And Action Contracts

## Status
Accepted

## Date
2026-05-20

## Context
GARM already has auditable organ autonomy, HRM execution, CAG context planning, persistence, report/export and local verification. The next architectural gap is coordination: organs can act safely, but goals and action contracts were implicit in command handlers and organ audit trails.

Current constraints still apply: GARM is the single runtime, Rust-only local execution, no LLM dependency, local-first operation, no autonomous remote crawler, and all state must remain auditable and persistible.

## Decision
Add a GARM-native goal scheduler as an architectural seam for objectives and action contracts.

- `goal_scheduler` records goals with priority, risk, status, evidence requirements and result.
- Each planned goal creates explicit action contracts with organ, kind, preconditions, expected effect, required evidence, risk, status and result.
- `goals`, `goals plan X`, `goals run` and `goals audit` expose the scheduler without adding a second runtime.
- HRM execution and manual organ runs register completed or blocked external goals, preserving compatibility with existing `hrm run` and `organos run` behavior.
- Scheduler state persists to `goal_scheduler.json`, appears in `garm audit`, `garm report`, `garm export`, state reports and artifact indexes.

## Alternatives Considered

### Replace Organ Autonomy With A New Planner
Rejected because organ autonomy is already audited and verified. The scheduler should coordinate and record contracts, not erase working organ behavior.

### Put Goals Only Inside HRM
Rejected because goals are runtime-level architecture. CAG, organ autonomy, HRM and future learning loops all need the same seam.

### Add A Separate CLI Or Runtime
Rejected because GARM remains the single EDEN runtime and operational surface.

## Consequences

- Goals become a first-class persisted architectural concept.
- Action execution is still conservative: this phase records and closes local contracts; future phases can attach stronger executors, rollback and evaluation behind the same seam.
- Reports and exports now include goal scheduler status, improving auditability of autonomy decisions.
