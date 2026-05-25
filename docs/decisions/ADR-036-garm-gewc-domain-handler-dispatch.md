# ADR-036: GEWC Domain Handler Dispatch

## Status
Accepted

## Date
2026-05-22

## Context
ADR-035 introduced `GewcBodyExecutor`, moving physical command execution behind the GEWC cycle host. The executor still contained one large absorbed match. That was a stable intermediate state, but it did not yet show how body handlers would become independently owned execution domains.

The next safe step is to split clear domains first, without changing command behavior.

## Decision
Add domain handler dispatch inside `GewcBodyExecutor`.

The executor now routes selected handlers before falling back to the remaining absorbed body implementation:

```text
GewcBodyExecutor::execute
  -> GewcBodyRegistry::bind
  -> gewc_runtime_control_body_handler
  -> gewc_validation_body_handler
  -> remaining absorbed body execution
```

The first extracted handler domains are:

- `gewc_runtime_control_body_handler`
- `gewc_validation_body_handler`

These domains were chosen because they are high-leverage and low-risk:

- runtime-control owns save/load, shutdown, status, autonomy, reports and artifact operations;
- validation owns readiness, benchmarks, architecture evals, capability registry and package generation.

Runtime reports now include:

```text
handler_dispatch=domain_handler_dispatch
```

Capability validation requires this marker.

## Alternatives Considered

### Extract all handlers in one commit
Rejected. That would maximize churn in the riskiest command surface. Incremental extraction keeps each domain testable.

### Keep one executor match
Rejected. That preserved a deep enough interface but not enough locality inside the implementation.

### Extract legacy compatibility first
Rejected for this phase. Legacy compatibility has more cross-coupling with memory, reasoning and CAG, so runtime-control and validation are better first domains.

## Consequences
- GEWC now has real domain dispatch, not only a single executor.
- The runtime-control and validation bodies can be evolved independently behind GEWC.
- Remaining command families can be extracted one domain at a time.
- Existing command semantics are preserved by keeping unextracted bodies in the absorbed fallback.
