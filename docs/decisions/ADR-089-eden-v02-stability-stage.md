# ADR-089: EDEN v0.2 Stability Stage

## Status
Accepted

## Date
2026-05-26

## Context
EDEN v0.1 proved that the 7B-shaped Megatron path can move beyond the original
pilot, load a checkpoint and pass a semantic runtime-candidate gate while still
blocking production release and AGI claims.

The next risk is not adding more architecture labels. The next risk is whether
a longer checkpoint candidate remains operationally stable, comparable against a
known baseline, rollback-safe and usable only through GEWC-controlled inference
boundaries. The repository also needs portable evidence because disposable GPU
VMs should not retain checkpoints after a run.

## Decision
Add an EDEN v0.2 stability stage with these properties:

- build a repo-owned stability corpus with train, eval and challenge splits;
- compare a 100-iteration baseline against a 250-iteration candidate;
- require real checkpoint-load and token-generation reports for both runs;
- add deterministic adversarial checks for injection, permissions, rollback and
  model-authority boundaries;
- add a rollback drill before candidate runtime admission;
- generate an internal model card and checkpoint-storage manifest;
- expose a native GEWC inference-service contract for the candidate;
- keep production release, autonomous operation and AGI claims blocked.

The stage is implemented as normal repository tooling, not as a private
notebook. It can run locally without GPU and emit a failed-but-valid gate, or it
can run on a ROCm GPU host with `EDEN_V02_RUN_GPU=true`.

## Alternatives Considered

### Train directly toward 14B

- Pros: moves toward the planned maximum dense-model size sooner.
- Cons: skips stability comparison, rollback and storage discipline.
- Rejected: larger training without admission controls would make the evidence
  weaker, not stronger.

### Admit the v0.1 checkpoint as production

- Pros: simpler operator story.
- Cons: v0.1 is a candidate runtime signal, not a production model release.
- Rejected: production admission still needs stronger eval, safety and
  reproducibility evidence.

### Keep evidence only on the GPU VM

- Pros: avoids copying artifacts back.
- Cons: disposable GPU VMs should be cleanable, and reviewers need portable
  evidence.
- Rejected: portable JSON reports are required; generated checkpoints remain
  out of git and can be purged from the VM.

## Consequences

- `make training-eden-v02-stage` becomes the entry point for local or ROCm v0.2
  stability evaluation.
- `make eden-v02-stability` admits the resulting reports into GEWC as runtime
  artifacts.
- A 250-iteration checkpoint can become a candidate runtime only if comparison,
  inference, adversarial, rollback, model-card, storage, service and demo checks
  pass.
- The gate remains conservative: production release, autonomy and AGI claims are
  false even when candidate runtime admission is true.
