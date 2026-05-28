# ADR-083: EDEN 7B Governed Inference Adapter

## Status
Accepted

## Date
2026-05-26

## Context
The MI300X Megatron pilot produced a 7B-shape EDEN-owned checkpoint from the
repo-local corpus and tokenizer. The previous artifact boundary admitted the
training run only as evidence that the ROCm/Megatron path executed. It did not
make the checkpoint usable inside EDEN, and it intentionally kept checkpoint
admission, production inference and AGI claims blocked.

The next step is to prove a narrower runtime capability: the checkpoint can be
loaded and can generate tokens without external models, external APIs or network
access. That capability still must remain subordinate to GEWC. A generated text
sample is not semantic validation, benchmark performance or autonomy.

## Decision
Add a governed 7B inference path:

- `training/rocm/megatron_eden_7b_inference_probe.sh` loads the checkpoint in the
  ROCm Megatron container with Docker `--network none` and runs Megatron Core
  batch inference.
- The probe sets `TORCH_FORCE_NO_WEIGHTS_ONLY_LOAD=1` because Megatron's torch
  checkpoint contains trusted local training metadata such as
  `argparse.Namespace`; no external checkpoint source is accepted by this path.
- `training/rocm/build_megatron_7b_inference_report.py` validates the inference
  output and writes `eden.megatron.7b.inference_report.v1`.
- GEWC/GARM commands admit the checkpoint as `megatron_7b_model_adapter`,
  `megatron_7b_inference_report`, `megatron_7b_capability_report` and
  `megatron_7b_admission_gate`.
- The admitted capability is limited to checkpoint-load and candidate
  token-generation probes. Direct memory writes, objective changes, tool
  execution, checkpoint admission and production inference remain blocked.

## Alternatives Considered

### Use the Megatron HTTP text-generation server

- Pros: Matches Megatron's documented server path.
- Cons: The ROCm image used for the pilot does not include `flask_restful`, and
  installing packages would break the offline/no-network training boundary.
- Rejected: Direct Megatron Core batch inference avoids extra dependencies.

### Admit the checkpoint as a model release

- Pros: Simpler public story after a successful training run.
- Cons: A short pilot checkpoint has not passed semantic, safety, benchmark,
  calibration or release review.
- Rejected: The checkpoint is usable only as a governed probe.

### Keep inference outside GEWC

- Pros: Less runtime integration work.
- Cons: It would leave the trained artifact disconnected from EDEN's authority,
  audit and safety model.
- Rejected: Model outputs must enter EDEN as subordinate hypotheses.

## Consequences

- EDEN now has an executable route from architecture to trained checkpoint to
  real inference evidence.
- The runtime can expose the 7B checkpoint as a native cognitive capacity
  without giving it authority over memory, goals or tools.
- Future work can add semantic evaluations, ELCP-style cognitive-state targets
  and safety probes on top of the same adapter.
- Public claims remain constrained: this is checkpoint usability evidence, not
  AGI evidence or production model admission.
