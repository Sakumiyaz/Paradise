# ADR-085: EDEN Capable Operationalization

## Status
Accepted

## Date
2026-05-26

## Context
ADR-084 created the seven no-GPU artifacts that bridge EDEN's 7B checkpoint
probe to a governed capability surface. That was necessary but still mostly
preparatory: it proved the contracts around training evidence, dataset manifest,
structured output, checkpoint registry and SFT/ELCP readiness.

The next gap is operational. EDEN needs a local runtime path that answers: can
GEWC call the checkpoint-backed model surface, wrap output as an untrusted
packet, verify it, route memory/action effects through policy and produce a
demo trace without starting new GPU work?

## Decision
Add `eden capable operationalize` and `make eden-capable-operationalize`. This
generates seven operational artifacts:

1. `eden_live_inference_runtime.json` exposes the 7B probe as a callable,
   subordinate, report-backed candidate generator.
2. `eden_cognitive_call_contract.json` defines the GEWC -> model router ->
   checkpoint probe -> packet parser -> verifier -> memory/action gate flow.
3. `eden_cognitive_dataset_expansion.json` reports expanded synthetic cognitive
   task coverage.
4. `eden_capability_eval_suite.json` turns the seed data into a local capability
   contract eval suite.
5. `eden_sft_elcp_activation_gate.json` keeps SFT/ELCP training blocked until
   explicit operator GPU approval and reviewed data.
6. `eden_memory_action_loop.json` demonstrates how a model packet can produce
   draft plans and audit metadata while memory facts, objectives and tools
   remain blocked.
7. `eden_capable_demo_trace.json` records a public-facing governed demo path.

`eden_capable_operational_gate.json` aggregates those seven artifacts.

## Alternatives Considered

### Start another GPU run

- Pros: Could generate fresh model outputs.
- Cons: The user explicitly wanted no additional GPU use at this point.
- Rejected: This phase is local operationalization only.

### Treat the checkpoint as a production model

- Pros: Simpler user story.
- Cons: The checkpoint has only load/token-generation evidence, not semantic
  reliability, held-out evaluation or release admission.
- Rejected: The checkpoint remains a subordinate hypothesis generator.

### Keep the operational loop in documentation

- Pros: Smaller code change.
- Cons: EDEN's direction is executable evidence, not prose-only architecture.
- Rejected: The loop is implemented as runtime commands and artifact API entries.

## Consequences

- EDEN now has a local "capable" path that can be generated and inspected
  without consuming GPU.
- The public story is clearer: EDEN can route, gate, verify, audit and demo a
  checkpoint-backed cognitive surface, while refusing AGI or production claims.
- Future training remains gated by reviewed data, pre/post evals, checkpoint
  rollback policy and explicit operator approval.
