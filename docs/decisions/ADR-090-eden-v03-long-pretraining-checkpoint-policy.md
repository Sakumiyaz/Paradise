# ADR-090: EDEN v0.3 Long Pretraining and Checkpoint Policy

## Status
Accepted

## Date
2026-05-26

## Context
EDEN v0.2 produced a 250-iteration 7B candidate with checkpoint comparison,
inference load, adversarial checks, rollback, model-card limits, checkpoint
storage policy and a native inference-service boundary. That is enough for a
stable candidate-runtime signal, but not enough to justify the next stage of
training or any production claim.

The next step must turn the informal "seven remaining GPU processes" into one
reproducible stage:

- train beyond the v0.2 pilot window;
- use a larger curated EDEN-owned corpus;
- admit checkpoints only through GEWC evidence;
- expose inference as a persistent runtime contract, not as a one-off probe;
- preserve the 14B dense ceiling;
- evaluate held-out generalization structure;
- retain or purge checkpoints only under explicit policy.

## Decision
Add an EDEN v0.3 capability stage with these properties:

- build a deterministic 6144+ row generalization corpus from repo-owned sources;
- run an optional 1000-iteration 7B Megatron job on ROCm with Docker network
  disabled and no external model dependency;
- load the resulting checkpoint through the existing Megatron inference probe;
- compare the long run against the v0.2 candidate and require no regression;
- inherit the v0.2 safety chain before any candidate admission;
- generate a checkpoint-admission report, live inference runtime report,
  checkpoint registry, 14B scaling plan and operational demo trace;
- admit only candidate runtime use when all checks pass;
- keep production release, autonomous authority and AGI claims blocked.

The stage is local-first and evidence-driven. Without GPU evidence it still
writes blocked-but-valid reports, so CI and operators can inspect what is
missing without pretending that capability exists.

## Alternatives Considered

### Start 14B immediately

- Pros: aligns with the stated maximum dense parameter target.
- Cons: bypasses the 7B long-run evidence needed to know whether the current
  data, tokenizer, checkpoint policy and inference boundary are stable.
- Rejected: 14B work should start only after the 7B long-run path passes.

### Treat v0.3 as a production model release

- Pros: simpler public story.
- Cons: the evidence remains internal, local and architecture-focused; it does
  not measure production safety or AGI.
- Rejected: v0.3 is candidate runtime evidence only.

### Always purge checkpoints from GPU VMs

- Pros: keeps disposable GPU hosts clean.
- Cons: prevents operator review or promotion if the checkpoint passed.
- Rejected: checkpoint purge remains explicit. The registry records that
  weights stay out of git and that ephemeral GPU purging is allowed after
  portable evidence is copied.

## Consequences

- `make training-eden-v03-stage` becomes the entry point for local or ROCm v0.3
  capability evaluation.
- `make eden-v03-capability` admits the resulting reports into GEWC.
- The 1000-iteration checkpoint can become a candidate runtime only if corpus,
  long training, inference, v0.2 safety, registry, runtime and demo checks pass.
- The path to 14B is explicitly gated by v0.3 evidence, dataset freeze, GPU
  budget and a separate multi-GPU plan.
