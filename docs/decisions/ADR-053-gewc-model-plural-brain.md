# ADR-053: Make GEWC Model-Plural Instead Of LLM-Centric

## Status
Accepted

## Date
2026-05-23

## Context
GEWC already owns the Eden/GARM runtime cycle, command routing, handler
topology, safety gates and runtime traces. The next architectural risk is
mistaking the future Eden model layer for a single conventional LLM. Eden will
need language, reasoning, code, memory retrieval, verification, safety, world
modeling, tool-use and multimodal capabilities, but none of those should become
an independent core.

## Decision
Keep GEWC as the brain authority and make the model layer explicitly plural.

Each GEWC decision now records:

- the primary model role selected for the route;
- supporting model routes under `gewc_model_plural_registry`;
- the verifier loop required before action, memory update, training candidate
  or claim;
- the training boundary that prevents runtime model-weight mutation;
- the memory-governed learning policy;
- the world-model consequence contract;
- the code-brain policy that allows patch/test/review proposals but blocks
  autonomous code mutation.

The formal GEWC artifact now includes a `model_control_plane` with model roles
for language, reasoning, code, embeddings, reranking, verifier/critic, safety,
world model, tool-use and future multimodal grounding.

## Alternatives Considered

### Put a single LLM at the center

Rejected. A conventional LLM is useful for language and synthesis but should not
own objectives, memory integrity, tool permissions, training gates, safety or
runtime authority.

### Create a separate model supervisor outside GEWC

Rejected. That would reintroduce a second brain after GEWC absorbed GARM and
legacy runtime bodies.

### Defer model-routing until training starts

Rejected. Training should be constrained by architecture, not added later as an
uncontrolled capability layer.

## Consequences

- Eden stays GEWC-centric and model-plural.
- Future LLM/LMM/code/reasoning training has clear runtime boundaries.
- Code models are governed proposal engines, not autonomous mutators.
- Runtime traces now show which model role was selected and which verification
  and training policies governed the decision.
- This is architecture integration evidence, not an AGI capability claim.
