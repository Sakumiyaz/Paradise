# ADR-065: Add Eden Locus And Operator Forge As Native GEWC Handlers

## Status
Accepted

## Date
2026-05-24

## Context
EDEN already has a native GARM/GEWC runtime and a Praxis Nexus formal substrate.
Two gaps remained in the executable architecture:

- personal context, connector input, documents, tool output and model output
  needed a native authority and quarantine path before they could influence
  memory, goals, plans or actions;
- formal synthesis needed to be more than an architecture note, but should not
  copy external single-operator ideas or treat generated formulas as truth.

Both gaps must remain under GEWC authority. They should not become separate
cores, direct memory writers or autonomous action paths.

## Decision
Add two native GEWC body handlers:

- `gewc_locus_context_body_handler` for Eden Locus Layer commands:
  `locus eval`, `locus ingest ...`, `locus context ...` and `locus audit`.
- `gewc_formal_synthesis_body_handler` for Praxis Nexus and Eden Operator Forge
  commands: `praxis nexus eval`, `operator forge eval`,
  `operator forge synth ...`, `operator forge verify` and
  `operator forge audit`.

The Locus Layer writes versioned local artifacts for authority classes, evidence
vault, permission matrix, context packet and operator timeline. It treats
documents, web content, tools and model text as evidence, not instruction
authority.

Operator Forge writes versioned local artifacts for primitive basis selection,
typed expression-graph candidates, verification reports and a model registry.
It explicitly keeps candidates as hypotheses until verified and accepted by
GEWC.

Both surfaces are exposed through existing command routing, artifact inventory,
runtime state inventory, schema discovery and GEWC lifecycle supervision.

## Alternatives Considered

### Keep Them As Documentation Only
- Pros: no runtime complexity.
- Cons: preserves architecture as non-executable artifacts and does not answer
  how personal context or formal candidates enter the runtime safely.
- Rejected: EDEN's current direction is executable architecture evidence.

### Put Locus Under Memory And Forge Under Praxis Only
- Pros: fewer handlers.
- Cons: hides authority parsing inside memory and hides formal synthesis inside
  the substrate, reducing auditability and lifecycle control.
- Rejected: both are distinct GEWC-owned runtime domains.

### Use A Single External Operator Formalism
- Pros: smaller formal surface.
- Cons: risks copying an external idea and overfitting EDEN's formal substrate
  to one mathematical primitive.
- Rejected: EDEN uses a typed primitive-basis forge with verification, not a
  single-operator claim.

## Consequences
- GEWC handler count increases from 15 to 17.
- `praxis nexus eval` now routes through the formal-synthesis handler rather
  than the generic validation handler.
- Runtime state and reproducible artifact APIs include Locus and Operator Forge
  artifacts.
- The no-claim boundary is preserved: artifacts are local engineering evidence,
  not AGI validation, scientific proof or trained LMM capability.
