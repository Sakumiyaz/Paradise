# ADR-084: EDEN Capable Seven-Step Surface

## Status
Accepted

## Date
2026-05-26

## Context
EDEN now has a governed 7B checkpoint probe: the checkpoint was trained in a
short MI300X pilot, loaded through Megatron Core inference and admitted into
GEWC as checkpoint-load/token-generation evidence. That still does not make EDEN
semantically capable or AGI-like. The missing work is the bridge from "model can
load and emit tokens" to "runtime can use model outputs as governed cognitive
capacity".

The user requested the seven remaining steps without using GPU again.

## Decision
Add an `eden capable eval` surface that writes seven no-GPU artifacts:

1. `eden_capable_training_run_contract.json` prepares the longer 7B training run
   contract while explicitly keeping GPU execution off.
2. `eden_cognitive_dataset_manifest.json` admits a repo-local synthetic
   cognitive capability seed dataset.
3. `eden_native_inference_api.json` defines the GEWC native structured
   inference request/response boundary.
4. `eden_capability_delta_eval.json` compares architecture-only state against
   checkpoint-load/token-generation evidence.
5. `eden_structured_output_report.json` turns raw model text into untrusted EDEN
   hypothesis packets.
6. `eden_checkpoint_registry.json` registers the checkpoint as a probe, not a
   release.
7. `eden_sft_elcp_readiness.json` prepares SFT/ELCP readiness and keeps training
   blocked until explicit approval.

`eden_capable_gate.json` aggregates those steps under GEWC authority. All
artifacts preserve `claim_allowed=false`, `agi_claim=false`, no direct memory
writes, no objective writes and no direct tool execution.

## Alternatives Considered

### Start a longer GPU training run immediately

- Pros: Moves toward learned behavior.
- Cons: The user asked to avoid further GPU use for now, and the dataset and
  evaluation gates need to be prepared first.
- Rejected: The correct next step is no-GPU capability preparation.

### Treat generated text as native EDEN cognition

- Pros: Simpler integration.
- Cons: Raw model text is not verified state, memory, policy or action.
- Rejected: Model output becomes an untrusted hypothesis packet until GEWC
  verifies it.

### Keep these steps only in documentation

- Pros: Low implementation cost.
- Cons: EDEN needs executable runtime evidence, not prose-only plans.
- Rejected: Each step is a generated artifact and artifact-API entry.

## Consequences

- EDEN has a concrete, executable bridge from the 7B checkpoint probe to a
  governed capability surface.
- The project can continue locally without consuming GPU.
- Future GPU work has clearer prerequisites: reviewed cognitive data, SFT/ELCP
  target definitions, before/after evals, checkpoint registry and rollback.
- The no-claim boundary remains intact.
