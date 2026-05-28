# ADR-093: EDEN 70B Single-MI300X Module Pilots

## Status
Accepted

## Date
2026-05-27

## Context
ADR-092 established EDEN-70B as a modular family, not one dense 70B
checkpoint. The first available GPU environment was a single AMD MI300X. That
environment is useful for proving launcher, ROCm, dataset, tokenizer and
Megatron execution paths, but it is not sufficient for all EDEN-70B modules.

The operator approved starting the 70B route, then paused further GPU work until
a future 8x MI300X environment is available.

## Decision
Add `training/rocm/megatron_eden_70b_module_pilot.sh` and
`make training-eden-70b-module-pilot` as the bounded per-module launcher.

The launcher:

- trains one EDEN-70B module at a time;
- uses repo-owned modular seed data;
- starts from random weights;
- uses Docker `--network none`;
- keeps `claim_allowed=false` and `agi_claim=false`;
- writes evidence under `target/`;
- keeps checkpoint admission blocked;
- blocks modules that exceed the single-MI300X memory budget instead of
  launching a known-invalid run.

The single-MI300X pilot produced evidence for:

| Module | Target | Observed | Status |
| --- | ---: | ---: | --- |
| `eden_memory_router_retrieval_3b` | 3B | 2.89B | Pilot executed |
| `eden_safety_verifier_4b` | 4B | 3.93B | Pilot executed |
| `eden_planner_code_tool_6b` | 6B | 6.14B | Pilot executed |

The same launcher records blocked evidence for:

| Module | Target | Single-MI300X status |
| --- | ---: | --- |
| `eden_cwm_12b_causal_world_model` | 12B | Blocked; needs multi-GPU or smaller proxy |
| `eden_multimodal_vla_12b` | 12B | Blocked; needs multi-GPU or smaller proxy |
| `eden_33b_elcp_primary` | 33B | Blocked; needs multi-GPU tensor/pipeline parallelism |

## Consequences
- EDEN now has a real module-by-module training entry point for the 70B family.
- The 70B route is no longer only an architectural target; it has executable
  ROCm/Megatron pilot evidence for the modules that fit one MI300X.
- The 33B primary module remains paused until an 8x MI300X or equivalent
  distributed ROCm topology is available.
- Single-GPU pilot evidence does not admit checkpoints, does not imply semantic
  competence and does not support AGI claims.

## Next Step
When an 8x MI300X environment is available, add a distributed 33B launcher using
Megatron tensor/pipeline parallelism and run a one-iteration no-checkpoint
preflight before any longer training job.
