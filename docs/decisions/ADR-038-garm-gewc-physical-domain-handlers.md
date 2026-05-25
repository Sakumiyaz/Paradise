# ADR-038: GEWC physical domain handler modules

## Status
Accepted

## Date
2026-05-22

## Context
ADR-037 made GEWC body handlers typed and exhaustively covered by tests. That removed string-only handler identity and silent top-level fallback, but most non-runtime and non-validation handlers still lived as thin methods inside `runtime.rs`.

The next integration step is to give GEWC body domains their own physical handler modules and move command-specific implementation into those domain bodies without changing command behavior or losing historical GARM and legacy implementation provenance.

## Decision
Add `runtime/gewc_body_handlers.rs` as the physical implementation layer for GEWC body domains.

The module owns domain entrypoints for:

- memory and reasoning;
- native compatibility;
- safe learning;
- world model;
- planning and goals;
- tool adapters;
- specialized models;
- metacognitive safety;
- experiments;
- agentic coordination;
- workspace attention;
- human interface;
- unknown intent.

`GewcBodyExecutor` still performs typed top-level dispatch, but non-runtime and non-validation domains now cross a physical module boundary and execute inside the owning domain module. Each module checks that `GewcBodyRegistry` assigned the command to the expected `GewcBodyHandler`; a mismatch returns a GEWC handler mismatch response instead of silently executing.

Runtime reports now expose `handler_topology=domain_owned_body_implementations` and `shared_body_engine=false`; capability validation requires those signals alongside `handler_dispatch` and `handler_metrics`.

## Consequences
- GEWC has physical domain-owned command implementations, not only enum dispatch.
- Domain implementations can now be expanded independently without changing the top-level executor.
- Handler misrouting is caught at the physical domain boundary.
- Existing behavior is preserved while the former shared non-runtime command engine is removed.
- Runtime-control and validation remain as native executor methods from previous phases; the non-runtime bodies are now owned by domain modules.

## Alternatives Considered

### Keep a shared internal command engine
- Pros: Smaller diff and less movement.
- Cons: It leaves a monolithic execution path behind the domain handlers.
- Rejected: It would preserve a long-term monolithic execution path after domain handlers already exist.

### Keep all domain methods inside `runtime.rs`
- Pros: Smaller change.
- Cons: GEWC would still be mostly a top-level dispatcher rather than a physically organized body.
- Rejected: The target architecture needs domain-owned files that can evolve independently.
