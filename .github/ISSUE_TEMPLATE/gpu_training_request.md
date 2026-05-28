---
name: GPU training request
about: Request a controlled GPU training, SFT or evaluation run
title: "gpu: "
labels: gpu, training
assignees: ""
---

## Goal

What capability, module or evaluation should this GPU run support?

## Requested Hardware

- GPU type:
- GPU count:
- Estimated wall time:
- Estimated GPU-hours:

## Dataset Boundary

- [ ] Dataset is listed in `training/data/license_manifest.json`.
- [ ] No private data, credentials or user files are included.
- [ ] License/provenance is documented.

## Expected Outputs

- [ ] Training report
- [ ] Evaluation report
- [ ] Checkpoint manifest
- [ ] Inference probe
- [ ] Rollback/admission recommendation

## Non-Goals

- [ ] This run does not make an AGI claim.
- [ ] Checkpoint admission remains a separate decision.
