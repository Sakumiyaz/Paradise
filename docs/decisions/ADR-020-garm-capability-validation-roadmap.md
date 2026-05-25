# ADR-020: Capability Validation Roadmap Artifacts

## Status
Accepted

## Date
2026-05-22

## Context
EDEN's next architecture step is adding capabilities without returning to self-scoring. ADR-019 separated readiness measurement from probe generation and added an initial local held-out harness. The remaining roadmap needs concrete artifacts for robust memory, world-model prediction, capability inventory and reproducible release checks.

## Decision
Add local validation artifacts and commands for the 90-day capability path:

- `memory eval` writes `memory_eval.json` for positive recall, abstention, distractor resistance, temporal/source preference and contradiction handling.
- `world eval` writes `world_eval.json` for prediction-loop evidence.
- `capabilities audit` writes `capability_registry.json` with `validated_local` versus `needs_evidence` capability states.
- `readiness external run` uses a versioned held-out local suite with 60 cases.
- `readiness package` includes memory/world/registry artifacts and expected reproduction commands.
- Make targets `eden-validate-local`, `eden-package` and `eden-release-check` provide a repeatable operator path.

All artifacts remain local pre-validation signals and include no AGI claim.

## Consequences

- EDEN has a clearer implementation path for adding capabilities while keeping evidence auditable.
- Capability state becomes an explicit registry instead of being inferred from unrelated counters.
- Memory and world-model claims get dedicated test artifacts.
- Independent external validation remains required before any external capability claim.
