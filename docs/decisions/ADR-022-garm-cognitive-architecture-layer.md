# ADR-022: Formalize the GARM Cognitive Architecture Layer

## Status
Accepted

## Date
2026-05-22

## Context
EDEN/GARM already had working memory, attention, metacognition, goal scheduling, policy and evaluation loops, but those signals were distributed across runtime reports. To make Cognitive Architecture AGI readiness auditable, the architecture needs a first-class artifact that validates those signals as one layer.

## Decision
Add `cognitive eval` and `cognitive_architecture.json`. The evaluation checks local evidence for:

- attention control and working memory
- episodic/semantic memory via memory eval
- executive goal control through goals, plan executor and policy
- metacognitive monitoring
- evaluation feedback loop

The artifact is included in `readiness_package.json`, the capability registry and independent release validation.

## Consequences
EDEN can now describe Cognitive Architecture AGI as a formal layer of the hybrid architecture, while preserving `claim_allowed=false` and requiring external validation before any AGI claim.
