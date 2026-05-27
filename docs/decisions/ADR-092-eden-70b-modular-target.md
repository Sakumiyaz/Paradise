# ADR-092: EDEN 70B Modular Target

## Status
Accepted

## Date
2026-05-26

## Context
EDEN v0.4 proved a governed 7B Megatron path on ROCm and prepared a 14B dense
preflight ceiling. That was useful evidence for the training stack, checkpoint
loading, GEWC admission and runtime hypothesis boundaries. It should not remain
the long-term capability target if EDEN is meant to compete as a broader
cognitive architecture rather than as a single language model.

The operator chose to move future capability work directly toward a 70B modular
architecture. This must not be interpreted as one dense 70B model. EDEN's
architecture keeps GEWC as the executive authority and routes specialized
models as subordinate hypothesis producers.

## Decision
Adopt `EDEN-70B Modular Target` as the future training target:

| Module | Parameters | Role |
| --- | ---: | --- |
| `eden_33b_elcp_primary` | 33B | Primary ELCP cognitive model for language, reasoning and governed state prediction. |
| `eden_cwm_12b_causal_world_model` | 12B | Causal world deltas, counterfactuals and simulation summaries. |
| `eden_multimodal_vla_12b` | 12B | Multimodal grounding, perception and VLA preparation. |
| `eden_planner_code_tool_6b` | 6B | Planning, code, tools and workflow synthesis. |
| `eden_safety_verifier_4b` | 4B | Safety, verifier, critic, uncertainty and corrigibility checks. |
| `eden_memory_router_retrieval_3b` | 3B | Retrieval, ranking, memory conflict detection and source calibration. |

Total target: 70B modular parameters across six separately admitted model
families. This is a capacity budget for a GEWC-routed system, not a single
checkpoint, not one active context, and not one dense LLM brain.

Default active target: 33B primary only. GEWC activates auxiliary modules by
task, risk, modality, uncertainty and cost.

The runtime now emits `eden_70b_modular_target.json` with
`schema=eden.modular_70b.target.v1`. The artifact is claim-gated:

- no training is executed by defining the target;
- no checkpoint is admitted;
- no AGI claim is allowed;
- no model receives direct memory, objective or tool authority;
- a single dense 70B core is explicitly forbidden;
- all-module activation for every request is explicitly rejected.

The follow-up command `eden 70b modular eval` makes this target executable as
runtime evidence. It writes:

- `eden_70b_module_router.json`;
- `eden_70b_dataset_manifest.json`;
- `eden_70b_launcher_manifest.json`;
- `eden_70b_checkpoint_admission.json`;
- `eden_70b_inference_runtime.json`;
- `eden_70b_operational_demo.json`;
- `eden_70b_operational_gate.json`.

These artifacts prepare routing, data, launch, admission, inference and demo
surfaces. They still do not train weights or admit checkpoints.

## Supersedes
This ADR supersedes the 14B dense ceiling as EDEN's future capability target.
It does not delete or invalidate 7B/14B evidence. Those remain historical
pipeline and admission evidence.

## Required Before Training
- Freeze `training/configs/eden_70b_modular_target.json`.
- Add per-module dataset manifests and license/privacy policy.
- Add per-module ROCm/Megatron or equivalent training launchers.
- Add per-module eval suites and checkpoint admission gates.
- Add multi-GPU budget, sharding and retention policy.
- Preserve GEWC authority and model-output-as-hypothesis semantics.

## Alternatives Considered

### Continue the 7B ladder
- Pros: cheap and already validated.
- Cons: too small to be the main long-term target.
- Rejected as the future target, retained as pipeline evidence.

### Keep 14B as final dense model
- Pros: coherent with the previous ceiling and easier to train.
- Cons: too narrow for the desired full architecture when multimodal, CWM,
  planning, safety and memory all need trained capacity.
- Rejected as the long-term target.

### Train one dense 70B model
- Pros: simpler public framing.
- Cons: contradicts EDEN's GEWC-centered architecture and overloads a single
  model with memory, planning, safety, world modeling and action authority.
- Rejected.

## Consequences
- EDEN's future model roadmap is modular by construction.
- 7B artifacts stay as historical evidence and compatibility probes.
- 14B remains useful as an intermediate experiment only if needed, not as the
  target architecture.
- New training work should target module-specific evidence rather than a single
  monolithic checkpoint.
