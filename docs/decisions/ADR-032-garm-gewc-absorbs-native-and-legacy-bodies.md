# ADR-032: GEWC Absorbs Native and Legacy Runtime Bodies

## Status
Accepted

## Date
2026-05-22

## Context
ADR-031 made `GlobalExecutiveWorkspaceCore` the operational decision authority, but the runtime traces still described two subordinate worlds:

- `garm_mode=native_modules_under_executive_workspace`
- `legacy_mode=compatibility_adapter_not_separate_core`

That was better than two nuclei, but it still left a long-term conceptual fault line. The desired architecture is stricter: GEWC is the complete runtime architecture. Nothing is external to it; historical GARM and legacy sources are preserved as implementation provenance inside GEWC-owned native body domains.

## Decision
GEWC now owns all runtime domains:

```text
absorption_model=gewc_owns_all_runtime_domains
external_cores_remaining=false

GEWC
├── gewc_executive_core
├── gewc_native_cognitive_body
├── gewc_native_compatibility_body
├── gewc_runtime_body
├── gewc_metacognitive_safety_regulation
└── gewc_validation_plane
```

Former GARM-native modules are absorbed as `gewc_native_cognitive_body`.
Former legacy runtime surfaces are absorbed as `gewc_native_compatibility_body`.
Runtime control is absorbed as `gewc_runtime_body`.

Historical implementation names may remain in file paths, module names and provenance records, but runtime decisions and artifacts must treat them as GEWC-owned native domains rather than external systems.

## Consequences
- GEWC is no longer only an executive sitting over GARM and legacy; it is the owning architecture for both.
- Legacy behavior is preserved, but its runtime status is native compatibility-body adapter.
- Release artifacts record `absorption_model=gewc_owns_all_runtime_domains`.
- No capability is removed or rewritten destructively.
- Future refactors can move files gradually without changing the authority model.
