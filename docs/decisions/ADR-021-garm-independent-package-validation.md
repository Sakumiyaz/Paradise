# ADR-021: Add Independent GARM Package Validation

## Status
Accepted

## Date
2026-05-22

## Context
ADR-020 added local capability, memory, world-model and held-out validation artifacts. Those checks improved the evidence package, but they still ran from the live `eden_garm` runtime. A release candidate needs a separate reproducibility step that consumes exported artifacts, verifies integrity and refuses claim escalation without depending on runtime state.

## Decision
Add `eden_garm_package_validator` as a standalone example runner. It reads `readiness_package.json` from a state directory, verifies required artifacts and FNV64 checksums, checks suite/result consistency, runs local adversarial controls for missing artifacts, corrupted checksums and unsupported claim escalation, and writes:

- `independent_validation_report.json`
- `release_candidate_manifest.json`

The Make targets are now:

- `make eden-independent-validate`
- `make eden-release-candidate`
- `make eden-release-check`

## Consequences
Release readiness now has a two-step boundary: GARM generates the package, then an independent runner validates it. This does not certify external AGI claims; it only promotes a local package to an auditable release candidate when artifacts are complete, internally consistent and claim-safe.
