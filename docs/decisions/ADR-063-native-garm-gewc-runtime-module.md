# ADR-063: Native GARM/GEWC Runtime Module

## Status

Accepted

## Date

2026-05-24

## Context

The public Paradise runtime had become the primary operator surface, but the
implementation still lived under `eden_core/examples/eden_garm/`. That path was
historical and increasingly misleading: examples should demonstrate usage,
while GARM/GEWC owns the executable runtime, API server, validation artifacts,
state routing and GEWC command lifecycle.

Keeping the runtime under `examples/` also made engineering practice harder to
enforce. Scripts, docs and `edenctl start` treated an example binary as the
production local runtime, while library consumers had no native module path for
GARM/GEWC.

## Decision

Move the GARM/GEWC runtime implementation into native library code under
`eden_core/src/garm/` and expose it through `eden_core::garm`.

The compatibility module path `eden_core::eden_garm` is preserved for existing
internal imports. A new binary target, `eden-garm`, starts the native runtime
from `eden_core/src/bin/eden_garm.rs`. The old `eden_core/examples/eden_garm.rs`
target remains only as a thin wrapper over `eden_core::garm::main_entry()`.

The supporting paradigm helpers moved from `eden_core/examples/paradigms/` to
`eden_core/src/paradigms/` because the native runtime still consumes them.

Scripts and operator flows now build and run the native binary:

```text
cargo run -p eden_core --bin eden-garm --
target/debug/eden-garm
```

## Alternatives Considered

### Keep GARM under examples

- Pros: smallest immediate diff.
- Cons: keeps the primary runtime in a misleading location and makes public
  docs harder to explain.
- Rejected because the runtime is no longer an example.

### Move directly to separate crates

- Pros: cleaner long-term package boundaries.
- Cons: higher risk because many modules still use `crate::eden_garm` imports
  and share state paths, tests and validation scripts.
- Deferred. The native module move is the safer intermediate step before a
  future `eden_garm_runtime` crate.

### Rename every internal import from `eden_garm` to `garm`

- Pros: cleaner final naming.
- Cons: broad mechanical churn across hundreds of references with little
  runtime benefit in this phase.
- Deferred. The public native alias `eden_core::garm` now exists, while
  `eden_core::eden_garm` remains a compatibility path.

## Consequences

- GARM/GEWC is now native runtime code, not an implementation hidden inside
  examples.
- `eden-garm` becomes the official runtime binary.
- Existing example workflows can continue through the compatibility wrapper.
- The next architectural cleanup can focus on crate extraction or internal
  import renaming without changing public API contracts.
- Follow-up ADR-064 extends the same native layout rule to the API conformance
  and package validation runners.
