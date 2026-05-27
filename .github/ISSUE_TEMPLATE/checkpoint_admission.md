---
name: Checkpoint admission
about: Propose admitting a trained checkpoint into the governed runtime registry
title: "checkpoint: "
labels: checkpoint, validation
assignees: ""
---

## Checkpoint

- Module ID:
- Checkpoint ID:
- Checkpoint hash:
- Storage location:

## Evidence

- [ ] Dataset manifest and license/privacy review attached.
- [ ] Training report attached.
- [ ] Inference report attached.
- [ ] Held-out evaluation attached.
- [ ] Safety/verifier evaluation attached.
- [ ] Rollback plan attached.
- [ ] Operator approval requested.

## Authority Boundary

- [ ] The checkpoint cannot write persistent memory directly.
- [ ] The checkpoint cannot execute tools directly.
- [ ] The checkpoint cannot modify goals, policies or permissions.
- [ ] `claim_allowed=false`
- [ ] `agi_claim=false`

## Validation Commands

```bash
make contracts-validate
make paradise-non-gpu-readiness
make eden-api-conformance
```
