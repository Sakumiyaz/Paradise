# ADR-081: EDEN 7B Base-Model Pilot

## Status
Accepted

## Date
2026-05-25

## Context
ADR-080 proves that EDEN can build a Megatron dataset and tokenizer from
repo-local corpus files, then train a tiny random-weight pilot on ROCm. The next
operator request is to validate a larger base-model path while preserving the
EDEN-only boundary.

The project must not confuse a single large optimization step with AGI
capability. A 7B-shape pilot can prove memory, ROCm, Megatron, tokenizer and
data plumbing, but it does not prove language competence or general
intelligence.

## Decision
Add `training/rocm/megatron_eden_7b_base_pilot.sh` and expose it through:

```bash
make training-megatron-eden-7b-base-pilot
```

The launcher:

- requires the Megatron ROCm Docker image to already exist locally;
- starts Docker with `--network none`;
- reads only `eden_core/corpus/**/*.txt`;
- trains a local SentencePiece tokenizer under `target/`;
- preprocesses EDEN-owned JSONL into Megatron indexed data;
- initializes a 7B-shape GPT-style model from random weights;
- runs a tiny ROCm pilot train;
- uses recomputation to lower activation memory;
- writes logs and summary artifacts under `target/eden_megatron_7b_base_pilot`;
- keeps checkpoint admission and AGI claims blocked.

The initial shape is:

```text
layers=32
hidden_size=4096
ffn_hidden_size=12288
attention_heads=32
sequence_length=128
```

## Consequences

- EDEN has a reproducible first 7B-scale base-model path.
- The path remains EDEN-owned at the data and tokenizer level.
- The output is training plumbing evidence only.
- Future useful 7B training requires a real corpus plan, long-run schedule,
  evaluation suite, checkpoint manifest and GEWC admission gate.

## Alternatives Considered

### Use an external 7B checkpoint

Rejected. The current goal is EDEN-owned base-model training, not adaptation of
an existing model.

### Claim AGI evidence after a short pilot

Rejected. Short optimization evidence is not behavioral evidence.

### Skip 7B until the corpus is much larger

Rejected for path validation. It is acceptable to validate the hardware and
training stack now while clearly blocking capability claims.
