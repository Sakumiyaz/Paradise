# ADR-069: Governed Model Runtime, Checkpoints And Training Harness

## Status
Accepted

## Date
2026-05-25

## Context
EDEN now has a reproducible training/evaluation surface, but the next step
cannot be a real model release yet. The LMM/LLM side is not trained, and the
architecture must preserve GEWC authority before any GPU-backed training work
begins.

The repo needs executable interfaces for model lifecycle and training evidence:

- registering, loading, evaluating and unloading model adapters;
- recording checkpoint policy without committing weights;
- describing the train/evaluate/compare/admit harness;
- enforcing model governance before model outputs can influence state.

## Decision
Add a native `model_runtime` module under GARM/GEWC. It writes four governed
artifacts:

- `model_adapter_runtime.json`
- `model_checkpoint_manifest.json`
- `training_harness_report.json`
- `model_governance_report.json`

GEWC routes `model register/load/evaluate/unload <id>` through the specialized
model body handler. Validation commands such as `model runtime eval`,
`training harness eval` and `model governance eval` write reproducible
no-claim artifacts. The path does not train a production model, does not admit
weights and does not allow direct model writes to memory, objectives or tools.

## Alternatives Considered

### Train the first EDEN model immediately
- Pros: Produces stronger capability evidence.
- Cons: Requires GPU scope, checkpoint policy and release criteria that are not
  yet established.
- Rejected: This ADR intentionally excludes real model training.

### Keep model lifecycle only in Python training scripts
- Pros: Simple for experiments.
- Cons: Bypasses GEWC authority and makes model governance optional.
- Rejected: Model lifecycle belongs in the runtime authority path.

### Treat model outputs as direct runtime state
- Pros: Lower integration overhead.
- Cons: Weakens safety, provenance and rollback boundaries.
- Rejected: Models propose hypotheses; GEWC decides state changes.

## Consequences
- Model lifecycle is executable through native GARM commands.
- Release packages and artifact APIs can expose model runtime governance.
- Future GPU training has a stable checkpoint and harness contract to target.
- The repo still makes no production LLM/LMM or AGI capability claim.
