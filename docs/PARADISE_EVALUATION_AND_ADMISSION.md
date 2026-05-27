# Paradise Evaluation And Checkpoint Admission

Paradise separates engineering readiness from model capability. A runtime can
be operationally healthy while model checkpoints remain blocked.

## Non-GPU Evaluation Surface

The non-GPU path validates:

- product positioning and claim boundaries;
- public API/contract shape;
- model interface authority;
- dataset governance;
- evaluation plan coverage;
- checkpoint admission policy;
- operator console visibility;
- external/public deliverable readiness;
- hardware/network test isolation;
- known technical debt registration.

Run:

```sh
make contracts-validate
make paradise-non-gpu-readiness
```

The command writes:

```text
target/paradise_non_gpu_readiness/non_gpu_readiness_report.json
target/public_contracts/validation_report.json
```

## Checkpoint Admission Policy

A checkpoint is not accepted just because training completed. Admission requires:

- dataset manifest and license/privacy review;
- checkpoint manifest with hash and module id;
- held-out evaluation;
- safety/verifier evaluation;
- runtime inference probe;
- GEWC hypothesis packet compatibility;
- memory/tool/objective authority checks;
- regression comparison against prior accepted checkpoint;
- rollback path;
- model card or internal release note;
- explicit operator approval for production-like use.

The public readiness report preserves `production_model_allowed=false` until
those conditions pass.

The public checkpoint registry is `training/models/checkpoint_registry.json`.
It intentionally starts empty and keeps every trained checkpoint blocked until
the admission policy above is satisfied.

Audit the registry through the native GEWC runtime command:

```sh
make paradise-checkpoint-registry-smoke
```

This writes `paradise_checkpoint_registry_admission.json` under the selected
runtime state directory and keeps `checkpoint_admission_allowed=false`.

## Required Reject Conditions

A checkpoint must remain blocked if:

- `claim_allowed` is true;
- `agi_claim` is true;
- direct memory writes are enabled;
- direct tool execution is enabled;
- direct objective updates are enabled;
- eval data overlaps with training data without disclosure;
- safety or uncertainty eval is missing;
- inference cannot be reproduced;
- checkpoint hash or provenance is missing;
- rollback is unavailable.

## Evaluation Families

| Family | Purpose |
| --- | --- |
| Memory/retrieval | Recall, source grounding, conflict detection. |
| Planning/tool-use | Hierarchical decomposition, cost/risk, dry-run compatibility. |
| Safety/verifier | Policy, prompt injection, exfiltration and privilege boundaries. |
| World model | Entity/state/causal deltas and counterfactual consistency. |
| VLA/multimodal | Grounding, affordances and action constraints. |
| Runtime integration | GEWC routing, traceability, rollback and audit. |

## Hardware And Network Tests

GPIO, I2C and external crawler tests are intentionally isolated behind
`make external-tests`. They are not part of the default local test gate because
they require real devices or network access.
