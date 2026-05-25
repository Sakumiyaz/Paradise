# ADR-080: Eden-Owned Megatron Corpus Pilot

## Status
Accepted

## Date
2026-05-25

## Context
ADR-079 adds an offline Megatron smoke that proves ROCm/GPU execution without
external models. That smoke intentionally uses mock data, so it does not yet
prove that EDEN can build a training input stack from its own corpus.

EDEN needs a second, still-small pilot that uses repo-local text and a
repo-generated tokenizer before any larger model run is attempted.

## Decision
Add `training/rocm/megatron_eden_corpus_pilot.sh` and expose it through:

```bash
make training-megatron-eden-corpus-pilot
```

The launcher:

- requires the Megatron ROCm Docker image to already exist locally;
- starts Docker with `--network none`;
- reads only `eden_core/corpus/**/*.txt`;
- trains a local SentencePiece tokenizer under `target/`;
- preprocesses EDEN-owned JSONL into Megatron indexed data;
- initializes a tiny GPT model from random weights;
- runs a short ROCm train pilot;
- writes logs and summary artifacts under `target/eden_megatron_corpus_pilot`;
- keeps checkpoint admission blocked.

## Consequences

- The first GPU-backed language pilot is EDEN-owned end to end at the data,
  tokenizer and random-initialized model level.
- No Hugging Face tokenizer, external checkpoint, provider API or downloaded
  dataset is part of the default pilot.
- The pilot is still not a production model and does not justify stronger AGI
  claims.

## Alternatives Considered

### Use an existing Llama tokenizer

Rejected. It would make the first pilot depend on an external model artifact
and blur the EDEN-owned training boundary.

### Keep using only mock data

Rejected as incomplete. Mock data is useful for plumbing but does not validate
the corpus/tokenizer path.

### Commit generated tokenizer and indexed data

Rejected. Generated artifacts belong under `target/` or release storage, not
the source tree.
