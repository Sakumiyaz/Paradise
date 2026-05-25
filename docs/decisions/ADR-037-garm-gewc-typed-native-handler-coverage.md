# ADR-037: GEWC typed native handler coverage

## Status
Accepted

## Date
2026-05-22

## Context
ADR-036 introduced GEWC domain handler dispatch for the runtime-control and validation bodies. The registry already described every runtime command as a GEWC-owned route, but most routes still used string handler identifiers and the executor could continue through a shared absorbed command body when a handler did not have an explicit dispatch entrypoint.

That made the integration operationally correct, but weaker than the target architecture: GEWC should not depend on string-only handler names or silent fallback behavior for core body ownership.

## Decision
Represent GEWC body handlers as the typed `GewcBodyHandler` enum and make `GewcBodyExecutor` dispatch every registered handler through an explicit native entrypoint.

The executor now has native entrypoints for:

- runtime control;
- memory and reasoning;
- native compatibility;
- safe learning;
- world model;
- planning and goals;
- tool adapters;
- specialized models;
- metacognitive safety;
- validation;
- experiments;
- agentic coordination;
- workspace attention;
- human interface;
- unknown intent.

The current implementation keeps a shared internal command execution engine for behavior preservation, but it is no longer a silent top-level fallback. Each registered GEWC handler must resolve through typed dispatch first. A coverage test binds every `GarmCommand` sample through `GewcBodyRegistry` and asserts that each registered `GewcBodyHandler` has a native executor.

GEWC runtime reports also include per-handler decision, completion and blocked counts through `handler_metrics`.

## Consequences
- Handler identity is compiler-visible instead of string-only.
- New handlers must be added to `GewcBodyHandler`, dispatch, artifact evidence and coverage tests.
- Missing native executor coverage becomes a test failure.
- Runtime evidence can distinguish which GEWC body is active, completed or blocked.
- Historical GARM and legacy implementations remain preserved as internal provenance and implementation bodies under GEWC authority.

## Alternatives Considered

### Keep string handlers
- Pros: Smaller diff.
- Cons: Missing or mistyped handlers remain runtime-only failures.
- Rejected: Does not provide enough long-term safety for GEWC body ownership.

### Split every command body into separate files immediately
- Pros: Stronger physical separation.
- Cons: High churn and higher regression risk across a large command surface.
- Deferred: Typed dispatch and coverage establish the enforcement boundary first; deeper physical extraction can proceed by domain without changing the public behavior.
