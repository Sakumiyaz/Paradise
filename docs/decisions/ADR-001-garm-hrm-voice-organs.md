# ADR-001: GARM-Native HRM And Optional Voz/TTS Organs

## Status
Accepted

## Date
2026-05-20

## Context
GARM is the single EDEN runtime entry point. Current constraints require Rust-only local execution, no local or external LLM dependency, no mandatory GPU dependency, local API binding, and auditable organ autonomy with persisted deltas.

Two capabilities were needed after organ autonomy was stabilized:

- HRM: hierarchical reasoning connected to KG, CAG and history.
- Voz/TTS: optional voice synthesis behavior that does not block runtime when no audio backend exists.

## Decision
Implement both as GARM-native organs:

- `hrm_reasoner` performs deterministic multi-layer planning over memory facts, KG hits, path explanations and history fragments.
- `hrm run` executes plans through existing CAG and organ autonomy surfaces rather than introducing a new execution engine, and records execution counters on the HRM organ.
- `voice_synthesizer` persists bounded local text manifests and exposes an optional TTS surface without requiring an audio backend. If `EDEN_GARM_TTS_BACKEND` is set, it writes a local backend request file but still does not spawn or require the backend. A separate helper script can consume the request and write a local output manifest.
- `garm audit`, `garm backup`, `garm restore`, and `garm compact` provide local maintenance without adding a separate CLI runtime.

Both organs are registered in the per-organ autonomy registry, included in save/load, exposed through commands, and covered by the local verification gate.

## Alternatives Considered

### External LLM HRM
Rejected because it violates the zero-LLM constraint and would make reasoning non-local and harder to audit.

### Mandatory TTS Backend
Rejected because runtime availability should not depend on optional audio tooling. The text-manifest fallback preserves behavior and auditability.

### Separate HRM/TTS Runtime
Rejected because GARM is the single runtime. Separate entry points would fragment persistence, audit trails and operational verification.

## Consequences

- Organ count is now `32`.
- HRM is auditable and deterministic; its multi-layer planner remains heuristic rather than model-driven, with persisted plan execution counters.
- Voz/TTS can later attach a real local audio backend behind the same organ without changing commands or persistence shape.
- The local gate remains `bash eden_core/examples/eden_garm/scripts/verify.sh`.
