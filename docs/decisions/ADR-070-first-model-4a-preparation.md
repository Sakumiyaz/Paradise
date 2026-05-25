# ADR-070: First EDEN Model 4A Preparation

## Status
Accepted

## Date
2026-05-25

## Context
ADR-069 created the governed model runtime, checkpoint manifest, training
harness and model governance path. The next step is to prepare the first EDEN
model candidate without starting real GPU training.

The important distinction is:

- 4A prepares the first model contract, data boundary, metrics and admission
  gates.
- 4B may later execute training, create weights and evaluate checkpoints, but
  only after explicit approval.

## Decision
Add `first model prepare` as a GEWC-routed validation command. It writes:

- `first_model_card.json`
- `first_model_training_plan.json`
- `first_model_readiness.json`

The candidate model is `eden-memory-retrieval-baseline`, scoped to governed
memory retrieval and evidence ranking. It is not a central LLM, not a chatbot
core and not an autonomous agent. The command refreshes model-runtime
governance artifacts first, then writes the 4A artifacts.

Every 4A artifact preserves:

- `claim_allowed=false`
- `agi_claim=false`
- `training_executed=false`
- `weights_present=false`
- `4b_training_allowed=false` where applicable

## Alternatives Considered

### Start 4B training immediately
- Pros: Produces checkpoints sooner.
- Cons: Conflates preparation with GPU execution, checkpoint admission and
  release criteria.
- Rejected: 4A must be a safe preparation gate.

### Keep first-model planning only in Markdown
- Pros: Easy to read.
- Cons: Not executable, not packageable and not visible through the artifact
  API.
- Rejected: 4A should produce machine-readable contracts.

### Make the first model a central language model
- Pros: Familiar LLM-style path.
- Cons: Conflicts with GEWC as the executive authority and increases risk.
- Rejected: The first model is a subordinate memory/evidence adapter.

## Consequences
- Operators can inspect first-model readiness before any training run.
- Release packages include first-model preparation artifacts.
- Future 4B work has a clear boundary: it must add explicit training execution,
  checkpoint output, post-training evaluation and admission review.
