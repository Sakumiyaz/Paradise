# ADR-023: Formalize the GARM Embodied Grounding Layer

## Status
Accepted

## Date
2026-05-22

## Context
EDEN/GARM already includes a virtual body, proprioception, motor actions, `World3D` physics and world-model feedback. Those capabilities existed as runtime state, but release readiness needed a direct artifact proving that body action can create measurable local consequences.

## Decision
Add `embodied eval` and `embodied_grounding.json`. The evaluation runs a bounded local sensorimotor probe, advances the virtual body and simulated world, records action evidence, observes the consequence in the world model and checks:

- sensorimotor loop
- simulated world consequence
- world-model feedback
- action-evidence grounding
- physical grounding bridge

The artifact is included in the readiness package, capability registry and independent validator.

## Consequences
Embodied AGI is now a formal layer in the Paradise architecture. The artifact remains local readiness evidence, not an external AGI claim, and preserves `claim_allowed=false`.
