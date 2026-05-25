# ADR-034: GEWC Body Registry Owns Intent-to-Body Binding

## Status
Accepted

## Date
2026-05-22

## Context
ADR-033 made GEWC own the runtime cycle, but command execution still risked reading as a legacy dispatch table with GEWC metadata around it. The remaining architectural issue is not file naming by itself; it is the absence of a formal GEWC-owned registry that binds a parsed core intent to the body handler that executes it.

Without that registry, future changes could silently reintroduce a split where command routing, runtime control, legacy compatibility and native cognitive bodies evolve independently.

## Decision
Introduce `GewcBodyRegistry` inside the GEWC module.

The registry maps each parsed `GarmCommand` to a `GewcBodyBinding`:

```text
Core intent
  -> GEWC body handler
  -> GEWC execution unit
  -> lifecycle policy
  -> Core outcome
```

Runtime decisions and completions now persist:

- `body_domain`
- `body_handler`
- `execution_unit`
- `lifecycle_policy`

The current lifecycle policies are:

- `decision_and_completion`
- `decision_only_package_hash_stability`
- `shutdown_without_runtime_persistence`

`dispatch_gewc_cycle` still hosts the physical Rust match that calls existing implementation functions, but the authority model now comes from `GewcBodyRegistry`. The match is implementation detail of absorbed GEWC body execution, not the architectural source of truth.

## Alternatives Considered

### Move every command arm into separate files immediately
Rejected for this step. It would be large mechanical churn with high regression risk. The important architectural move is to make intent-to-body binding explicit and GEWC-owned first.

### Keep body routing implicit in `CommandProfile`
Rejected. `CommandProfile` describes decision context; it should not be the runtime body registry. The registry gives a distinct interface for body execution policy and future handler extraction.

### Treat legacy and native handlers as separate registries
Rejected. That would recreate the split this phase is removing. Legacy behavior is preserved only as GEWC native compatibility-body execution.

## Consequences
- GEWC now owns both the cognitive cycle and the intent-to-body binding.
- Runtime traces show the exact GEWC handler used for each command.
- Package validation can detect missing GEWC cycle completion while preserving package hash stability.
- Future refactors can extract handler implementations behind the registry without changing command semantics.
- The implementation remains no-claim: this is architecture integration, not an AGI capability claim.
