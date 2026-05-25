# ADR-004: GARM Learning Ledger

## Status
Accepted

## Date
2026-05-20

## Context
GARM has goals, action contracts and an evaluation loop, but learning was still distributed across memory facts, KG facts, HRM traces and evaluation reports. That made it hard to answer what EDEN actually learned, what evidence supported it, and whether a result was consolidated or contradicted.

The same constraints apply: Rust-only, local-first, no LLM dependency, no autonomous remote crawler, and GARM remains the single runtime and command surface.

## Decision
Add a GARM-native learning ledger as the persisted seam for learning claims.

- `learning_ledger` records hypothesis, source, evidence, outcome, confidence and status.
- `learning`, `learning record X`, `learning consolidate` and `learning audit` expose the ledger through GARM commands.
- HRM execution records completed or blocked learning entries.
- `eval run` records a learning entry about architecture calibration.
- `save/load`, `garm audit`, `garm report`, `garm export`, state reports and artifact indexes include `learning_ledger.json`.

## Alternatives Considered

### Store Learning Only In Legacy Memory
Rejected because memory facts do not distinguish hypothesis, evidence, outcome, confidence and consolidation status.

### Store Learning Only In KG
Rejected because the KG is a semantic relation store. Learning needs audit semantics and contradiction tracking, not only edges.

### Fold Learning Into Evaluation Loop
Rejected because evaluation measures system state; the ledger records claims and their evidence lifecycle.

## Consequences

- EDEN/GARM now has an explicit local trail for what was learned and why.
- Confidence is heuristic and operational, not epistemic certainty.
- Future benchmark adapters and world-model tests can write learning outcomes behind this seam without changing the command surface.
