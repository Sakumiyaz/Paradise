# ADR-079: Eden-Only Megatron Offline Smoke

## Status
Accepted

## Date
2026-05-25

## Context
Paradise now has a public training surface, ELCP preparation gates and a
formal GEWC model-runtime boundary. The first AMD MI300X environment proved
that Megatron-LM can run under ROCm, but the tested command existed only as an
operator command sequence on a VM.

The project needs a reproducible GPU smoke that validates the execution path
without quietly depending on external models, provider APIs, gated tokenizers or
network access.

## Decision
Add `training/rocm/megatron_offline_smoke.sh` and expose it through:

```bash
make training-megatron-offline-smoke
```

The launcher:

- requires the Megatron ROCm Docker image to already exist locally;
- does not run `docker pull`;
- starts Docker with `--network none`;
- uses Megatron `NullTokenizer`;
- uses Megatron mock data;
- initializes a tiny GPT model from random weights;
- writes log and summary artifacts under `target/eden_megatron_offline_smoke`;
- keeps the result as smoke evidence only, not checkpoint admission.

## Consequences

- EDEN can validate the ROCm/Megatron path without Hugging Face, OpenAI,
  Anthropic, external checkpoints or remote datasets.
- The GPU smoke remains separate from default CI because it requires ROCm,
  Docker and a GPU host.
- Future real training must replace mock data with EDEN-owned datasets and keep
  checkpoint admission under GEWC governance.

## Alternatives Considered

### Use Megatron's Llama script directly

That script defaults to a Hugging Face tokenizer reference and is easy to run in
a way that depends on external assets. EDEN needs a stricter default.

### Pull the image automatically

This is convenient but violates the offline smoke contract. Image acquisition
should be an explicit operator action before the smoke.

### Treat the smoke as training completion

Rejected. The smoke proves GPU execution plumbing only. It does not produce a
useful EDEN model, admit weights or support stronger capability claims.
