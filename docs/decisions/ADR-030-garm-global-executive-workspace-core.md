# ADR-030: GARM Global Executive Workspace Core

## Status
Accepted

## Date
2026-05-22

## Context
EDEN GARM now validates memory, world modeling, cognitive architecture, embodied grounding, neural and symbolic architecture, bounded self-improvement, frontier layers, formal paradigms and integration governance. That breadth creates a second-order architecture risk: the system can become a collection of validated modules without an explicit core that decides what matters, which module should act, when to wait, when to verify, when to ask for supervision and when learning is safe.

The requested design name is `Global Executive Workspace Core` (`Nucleo Ejecutivo de Workspace Cognitivo Global`). This is not an official literature term. It is a design synthesis aligned with Global Workspace Theory, LIDA-like cognitive cycles, Soar/ACT-R-style memory-decision-learning patterns, modern LLM agent patterns and safety/control architecture.

## Decision
Add `global executive workspace eval` as the formal no-claim evaluation for the EDEN core coordinator.

The evaluation writes `global_executive_workspace_core.json` with:
- the explicit name `Global Executive Workspace Core`;
- `term_status=design_synthesis_not_official_literature_standard`;
- `claim_allowed=false` and `agi_claim=false`;
- 16 internal components: goal manager, situation model, working memory, global attention, global broadcast, cognitive router, hierarchical planner, action selector, long-term memory manager, world-model manager, causal/symbolic reasoner, agentic engine, metacognition, verifier/critic, continual learning and safety/corrigibility;
- 3 layers: global cognitive workspace, agentic-deliberative executive, and metacognitive safety regulation.

The artifact is included in the command router, help text, state paths, reproducible package, independent validator, capability registry and release-candidate Makefile path.

## Alternatives Considered

### Treat `integration governance eval` as sufficient
Rejected. Integration governance validates broad coordination and safety boundaries, but it does not name or verify the internal executive workspace structure with the 16 core components and 3 layers.

### Use an LLM as the core
Rejected. A foundation model can be a major cognitive engine, but the core must also coordinate memory, planning, tools, agents, world models, metacognition, safety, permissions and learning gates.

### Add another paradigm
Rejected. GEWC is not another paradigm. It is a core architecture layer that orchestrates existing paradigms and modules.

## Consequences
- EDEN has an explicit executive workspace artifact instead of relying on implicit module composition.
- The release candidate now requires `global_executive_workspace_core`.
- The capability registry gains one additional local validation capability.
- The architecture remains no-claim: local validation does not assert AGI completion.
