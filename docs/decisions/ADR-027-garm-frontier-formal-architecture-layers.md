# ADR-027: Formalize Frontier AGI Architecture Layers

## Status
Accepted

## Date
2026-05-22

## Context
EDEN/GARM already formalizes the core hybrid layers: cognitive, embodied, neural, symbolic, world-model, memory and bounded self-improvement. The extended AGI taxonomy also contains several non-duplicate frontier categories that should not be collapsed into the existing layers:

- safety-control architecture
- foundation models
- multimodal models
- LLM/foundation-model agents
- probabilistic programming
- hierarchical reinforcement learning
- cognitive robotics
- vision-language-action
- sim-to-real
- open-ended evolution
- developmental robotics
- whole-brain/neurocognitive architecture
- neuromorphic/spiking architecture

Other taxonomy items are subtypes of existing layers and should improve those parent branches instead of becoming shallow duplicate artifacts.

## Decision
Add `frontier architecture eval` and thirteen formal artifacts:

- `safety_control_architecture.json`
- `foundation_model_architecture.json`
- `multimodal_model_architecture.json`
- `llm_agent_architecture.json`
- `probabilistic_programming_architecture.json`
- `hierarchical_rl_architecture.json`
- `cognitive_robotics_architecture.json`
- `vla_architecture.json`
- `sim_to_real_architecture.json`
- `open_ended_evolution_architecture.json`
- `developmental_robotics_architecture.json`
- `whole_brain_neurocognitive_architecture.json`
- `neuromorphic_spiking_architecture.json`

Each artifact records local architecture evidence, parent layer, pass/fail cases, `claim_allowed=false` and `agi_claim=false`. They are included in the readiness package, capability registry and independent package validator.

## Consequences
The frontier categories are now formal layers without overstating capability. EDEN keeps duplicate taxonomy items attached to the main branches while preserving clear no-claim boundaries for model weights, real robotics, sim-to-real transfer, open-endedness, whole-brain mapping and neuromorphic backends.
