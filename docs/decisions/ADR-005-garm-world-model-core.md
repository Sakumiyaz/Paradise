# ADR-005: GARM World Model Core

## Status
Accepted

## Date
2026-05-20

## Context
GARM has memory, KG, goals, evaluation and learning ledger seams. However, world-model behavior was still implicit: facts could be remembered or linked, but EDEN did not have a first-class persisted seam for observations, predictions and verification.

The constraints remain: Rust-only, local-first, no LLM dependency, no autonomous remote crawler, and GARM remains the single runtime and command surface.

## Decision
Add a GARM-native world model core.

- `world_model_core` stores local observations and predictions in `world_model_core.json`.
- `world`, `world observe X`, `world predict X`, `world verify` and `world audit` expose the seam through existing GARM commands.
- Observations parse simple local relations such as `A causes B` and `A is B`.
- Predictions are explicitly marked `supported`, `unverified` or `verified_local` based on local observations.
- `world predict` records a learning ledger entry so predictions become part of the learning trail.
- `save/load`, `garm audit`, `garm report`, `garm export`, state reports and artifact indexes include world model state.

## Alternatives Considered

### Use Only Legacy Knowledge Graph
Rejected because KG edges store semantic relations, while the world model needs prediction status and verification lifecycle.

### Use Only Learning Ledger
Rejected because learning records claims and evidence outcomes; the world model is the source of observations and predictions that learning can later consume.

### Add An External Simulator First
Rejected for this phase. The architecture first needs a local persisted seam; external simulator adapters can be added behind it later.

## Consequences

- EDEN/GARM now has an explicit local observation/prediction/verification loop.
- The current predictor is heuristic and local; it is not a robust physical world model yet.
- Future embodiment, simulator and benchmark adapters can write observations and verification outcomes behind this seam without changing commands.
