# ADR-071: Eden Latent Cognitive Prediction 4A Preparation

## Status
Accepted

## Date
2026-05-25

## Context
EDEN needs a future training objective that fits the architecture rather than
treating a conventional language model objective as the whole brain. Next-token
prediction is useful for surface language, but EDEN's runtime is organized
around GEWC, CWM, memory, planning, action contracts, metacognition and safety.

The first model 4A gate prepared a subordinate memory-retrieval candidate. The
next step is to define a native objective for future training without starting
training, creating weights or admitting checkpoints.

## Decision
Add Eden Latent Cognitive Prediction (ELCP) as a formal 4A preparation gate.
ELCP trains no model in this phase. It defines:

- `elcp_objective_spec.json`
- `elcp_transition_dataset.json`
- `elcp_training_plan.json`
- `elcp_readiness.json`
- `training/configs/elcp_latent_cognitive_prediction.json`
- synthetic `training/data/elcp_transition_*.jsonl` fixtures

The command `elcp prepare` refreshes model-runtime and first-model artifacts,
then writes the ELCP objective, transition dataset contract, training plan and
readiness report. It keeps:

- `training_executed=false`
- `weights_present=false`
- `gpu_job_submitted=false`
- `4b_training_allowed=false`

## Objective Shape
ELCP predicts governed cognitive transitions rather than only the next token:

```text
C_t -> C_t+1
```

Targets include situation state, goal state, memory transition, world delta,
plan transition, action affordance, uncertainty, risk, safety gate and learning
update. Surface-token prediction remains available, but subordinate.

## Alternatives Considered

### Conventional next-token-only objective
- Pros: Simple, mature tooling.
- Cons: Does not directly train EDEN's runtime-native state, action, memory,
  world-model and safety targets.
- Rejected: Useful as one loss term, not sufficient as the native objective.

### Copy an external implicit-token objective
- Pros: Existing conceptual inspiration for hidden-state alignment.
- Cons: Would not be EDEN-native and would not cover memory, action, world
  deltas, governance or safety gates.
- Rejected: ELCP is designed around EDEN runtime traces and authority limits.

### Start real GPU training immediately
- Pros: Produces empirical model data sooner.
- Cons: Premature before objective, data, privacy, checkpoint and governance
  contracts are frozen.
- Rejected: 4A is preparation only; 4B must be explicitly requested.

## Consequences
- EDEN now has a native training-objective contract for future LMM/LLM-side
  work without claiming trained capability.
- API artifacts and schemas can validate ELCP preparation.
- Future training must use a separate 4B command/path and cannot be inferred
  from 4A readiness.
