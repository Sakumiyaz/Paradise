# ADR-060: EDEN Developer And Operator Runtime Surface

## Status
Accepted

## Date
2026-05-24

## Context
Paradise now exposes a governed local runtime, GEWC command routing,
operational permissions, recovery, replay, schemas, conformance reports and an
operator console. External users still need a smaller entry point than the
internal module tree. Without that entry point, consumers may couple to
implementation details under `eden_core/examples/eden_garm/` or treat the
runtime as a collection of unrelated artifacts.

## Decision
Add a first-class developer/operator surface:

- `edenctl`, a local CLI wrapper over the public API and governed command
  routes;
- executable operator examples for status, schemas, permissions, dry-run,
  degraded recovery, demos and evidence bundling;
- `docs/EDEN_PUBLIC_API_V1.md` as the frozen public v1 contract;
- `docs/HOW_TO_BUILD_ON_EDEN.md` as the integration guide;
- `make long-run-stability` as the end-to-end stability gate covering startup,
  commands, degraded recovery, replay, packaging, restart and independent
  validation.

## Alternatives Considered

### Add a separate SDK crate immediately

This would make the API look more formal, but it would add workspace structure
before the public contract has enough external consumers. Rejected for now.
`edenctl` and the existing lightweight SDK are sufficient for v1.

### Keep only shell scripts

Shell scripts are useful for CI and black-box testing, but they are not a
clean operator interface. Rejected because operators need a discoverable
command surface with stable terminology.

### Expose internal GEWC handlers directly

This would make experiments easier, but it would bypass the executive routing
and safety model. Rejected because public integrations should enter through
GEWC-governed commands and read-only APIs.

## Consequences

- Paradise has a practical onboarding path for external users without
  claiming completed AGI capability.
- Public integration should target API v1, `edenctl` or the SDK instead of
  internal modules.
- Long-run stability becomes measurable as an explicit gate.
- Future breaking API changes must be documented with a v2 contract and a new
  ADR.
