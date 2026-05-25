# ADR-061: Versioned Public Runtime Contracts

## Status
Accepted

## Date
2026-05-24

## Context
Paradise now exposes enough public runtime surface that relying only on
example files makes the integration boundary ambiguous. External users need a
stable CLI, SDK, contract directory and release candidate checklist without
depending on internal GEWC/GARM implementation paths.

The API server also needs a local operator guard that can be enabled without
changing the default local-first developer workflow.

## Decision
Promote the lightweight Rust client to `eden_core::sdk` and promote `edenctl`
to an official Cargo binary at `eden_core/src/bin/edenctl.rs`. Keep the old
example entry points as compatibility wrappers only.

Add `contracts/v1/` as the versioned public contract directory with:

- contract manifest;
- OpenAPI snapshots for runtime and operational APIs;
- JSON schemas for stable generated artifacts;
- minimal examples for SDK and conformance consumers.

Add optional local API token enforcement through `EDEN_API_TOKEN`. When the
variable is unset, the local API behaves as before. When set, non-public routes
require either `Authorization: Bearer <token>` or `X-EDEN-API-Token: <token>`.
This is a local operator guard, not internet-grade production authentication.

Release and conformance gates set `EDEN_GARM_SKIP_LEGACY_MIGRATION=1` and clean
their temporary state directories before execution. This keeps public evidence
reproducible and prevents stale `/tmp/eden_garm_*` legacy files from being
silently imported into a fresh validation run.

## Alternatives Considered

### Keep SDK and CLI under examples only

Pros: less code movement.

Cons: examples communicate instability and make downstream integration depend
on historical layout. Rejected because the repo is now presenting a public API
surface.

### Require auth by default

Pros: stronger default barrier.

Cons: would break the local-first quick start and existing no-network sandbox
tests. Rejected for v1. The secure mode is explicit through `EDEN_API_TOKEN`.

### Generate all contracts only at runtime

Pros: always reflects implementation.

Cons: there is no static public contract to review before running the runtime.
Rejected; static v1 contracts and runtime-generated exports both remain useful.

## Consequences

- Public integrations should target `eden_core::sdk`, `edenctl`,
  `docs/EDEN_PUBLIC_API_V1.md` and `contracts/v1/`.
- Historical example wrappers remain to avoid breaking old scripts abruptly.
- Future breaking API changes require a new contract version and ADR.
- Local auth is available for operator workflows but must not be described as a
  hardened internet-facing security model.
- Public release evidence is isolated from legacy global runtime state unless
  an operator explicitly chooses to run without the skip flag.
