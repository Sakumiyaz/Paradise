# ADR-082: Formalize Megatron 7B Evidence and Block Checkpoint Admission

## Status
Accepted

## Date
2026-05-25

## Context
EDEN can now execute an EDEN-owned 7B-shape Megatron pilot on AMD ROCm using
repo-local corpus text, a locally trained SentencePiece tokenizer, random
initialization and Docker `--network none`. The result proves the training path
can initialize and step a 7B-scale model, but that output must not be treated as
AGI capability, production inference readiness or checkpoint admission.

The runtime also needs this GPU evidence to enter the same GEWC-governed
artifact surface as CPU-safe training evidence instead of remaining only as a
log file under `target/`.

## Decision
Add `eden.megatron.7b.training_evidence.v1` as the formal contract for 7B pilot
evidence. The evidence builder reads the Megatron log and summary, records
parameter count, iterations, loss, memory, corpus metadata and checkpoint
status, and enforces:

- `claim_allowed=false`
- `agi_claim=false`
- `network=none`
- `mock_data=false`
- `external_model_dependency=false`
- `checkpoint_admission=false`
- `weights_admitted=false`
- `production_model=false`

Add `megatron 7b evidence eval` as a GARM command that validates the evidence
and writes it into the GEWC state artifact surface. Add
`make training-megatron-7b-evidence` to rebuild and admit that artifact after a
GPU run.

The 7B pilot may optionally write a checkpoint for controlled local evaluation,
but that checkpoint remains blocked from admission until a separate capability,
safety and governance process explicitly promotes it.

## Alternatives Considered

### Keep only the Megatron log

- Pros: Minimal code.
- Cons: Hard to audit, hard to expose through artifact APIs, and easy to
  overstate.
- Rejected because EDEN needs structured, claim-gated evidence.

### Admit any written checkpoint automatically

- Pros: Faster experimentation.
- Cons: Conflates training execution with model readiness and weakens GEWC
  authority.
- Rejected because checkpoints must remain subordinate to evaluation and
  governance.

### Require external benchmark validation before recording the pilot

- Pros: Stronger external signal.
- Cons: Blocks basic infrastructure evidence and is not necessary to prove the
  local training path.
- Rejected for this stage. External validation remains a later gate.

## Consequences

- EDEN has a reproducible, machine-readable 7B training-path artifact.
- The artifact API can expose the 7B evidence when generated.
- A 50-iteration run can produce a loss trajectory and optional local checkpoint
  without admitting weights or making AGI claims.
- The public repo remains explicit that the current signal is infrastructure
  and early training behavior, not general intelligence.
