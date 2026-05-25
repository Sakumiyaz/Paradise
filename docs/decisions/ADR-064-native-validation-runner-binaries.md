# ADR-064: Native Validation Runner Binaries

## Status

Accepted

## Date

2026-05-24

## Context

ADR-063 moved the primary GARM/GEWC runtime out of `examples` and into native
library and binary paths. Two operational validation runners still carried real
implementation logic as examples:

- `eden_garm_api_conformance`
- `eden_garm_package_validator`

Those runners are not tutorial examples. They are part of the public validation
surface: CI uses them to prove live API conformance, readiness package
integrity, release-candidate evidence and no-claim policy markers.

Keeping their implementation under `examples` preserved a confusing second
runtime surface after the main runtime had already become native.

## Decision

Move the validation runner implementations into native library modules:

- `eden_core::garm_api_conformance`
- `eden_core::garm_package_validator`

Expose official binary entry points:

```text
cargo run -p eden_core --bin eden-garm-api-conformance --
cargo run -p eden_core --bin eden-garm-package-validator --
```

Keep the old example targets only as thin compatibility wrappers over the
native modules.

Add `make native-runtime-layout` and wire it into CI so operational scripts and
current docs cannot drift back to `target/debug/examples/*` or
`--example eden_garm_*` runner invocations.

## Alternatives Considered

### Keep validation runners as examples

- Pros: smallest short-term change.
- Cons: keeps operational validation in a misleading location and contradicts
  the native runtime direction set by ADR-063.
- Rejected because conformance and package validation are release surfaces, not
  examples.

### Move runners directly into a separate validation crate

- Pros: cleaner long-term packaging.
- Cons: higher churn while the GARM/GEWC runtime is still consolidating inside
  `eden_core`.
- Deferred. Native modules and bins give the repo a stable surface first.

## Consequences

- GARM/GEWC runtime, conformance and package validation now share the same
  native layout discipline.
- CI validates the layout explicitly.
- Existing example commands can keep compiling through wrappers, but official
  scripts and docs use native binaries.
