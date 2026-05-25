# ADR-052: Clarify Paradise As Public Operator Runtime Surface

## Status
Accepted

## Date
2026-05-23

## Context
After publication, the repository could be misread in two opposite ways:

- as the complete Eden system, even though local LMM/model training and broader
  Eden integration remain incomplete;
- as only the A-life/autopoietic substrate, even though Eden also includes
  executive cognition, memory, reasoning, governed action, safety, validation
  and API surfaces.

The code layout added to that ambiguity. The workspace has two crates, while
the current public runtime lives under `eden_core/examples/eden_garm/` and the
broader Eden substrate remains under `eden_core/src/` with mixed maturity.

## Decision
Position Paradise as the public local-first operator/runtime surface for the
broader Eden architecture.

The public docs now distinguish:

- Eden: the broader hybrid architecture;
- Paradise: this public workspace for local runtime, APIs, operator console,
  validation and release evidence;
- GARM: the current operator runtime;
- GEWC: the executive coordination core inside GARM;
- Eden substrate: broader modules under `eden_core/src/`, including A-life,
  autopoietic, cognitive, memory, safety, physics and experimental domains;
- conformance packages: engineering evidence for API/contracts/policy markers,
  not AGI validation.

No runtime behavior changes in this decision. The current `examples/eden_garm/`
placement remains accepted for now but is documented as historical. A future
refactor can extract it into `eden_core/src/garm/` or a dedicated
`eden_garm_runtime` crate.

## Alternatives Considered

### Present the repo as the complete Eden runtime

Rejected. It overstates the current state and makes the no-claim policy weaker.

### Present the repo as only the A-life/autopoietic runtime

Rejected. It ignores GEWC, GARM, memory, reasoning, safety, APIs, validation and
operator surfaces that are already central to the public workspace.

### Move GARM out of `examples/` immediately

Deferred. The layout is a real source of confusion, but moving the runtime is a
larger code refactor. Documentation should first make the current architecture
understandable without changing behavior.

## Consequences

- Public readers get a clearer mental model before reading code.
- The repository can be useful before Eden is complete because it exposes the
  operator, API, validation and evidence layer.
- The no-claim boundary becomes stricter: local evidence is not presented as
  completed AGI or external validation.
- Future architecture work has a clearer next refactor target: extracting the
  GARM/GEWC runtime out of `examples/`.
