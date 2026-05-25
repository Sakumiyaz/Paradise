# GARM Release Audit

Date: 2026-05-20

Commit audited: `6463a06d GARM: index operational artifacts`

## Scope

This audit freezes the current local GARM runtime phase. GARM is the single EDEN runtime entry point through `eden_core/src/bin/eden_garm.rs`, backed by `eden_core/src/garm/runtime.rs` and the node graph under `eden_core/src/garm/`.

## Runtime Summary

- Single Rust runtime: `GarmRuntime`.
- API bind: `127.0.0.1` only.
- Default state directory: `/tmp/eden_garm`.
- Registered primary organs: `32`.
- Organ autonomy: per-organ action queues, audit entries, feedback counters and observable deltas in `organ_autonomy.json`.
- HRM: native deterministic hierarchical reasoning organ, no LLM dependency.
- Voz/TTS: optional local organ with text-manifest fallback.
- CAG: explicit context augmentation node with cache, actions, audit and metrics.
- Maintenance surface: audit, report, report history, export/import, export verification, artifacts index, backup, restore and compact.

## Operational Commands

```text
garm audit
garm report
garm report history
garm export
garm import
garm verify export
garm artifacts
garm backup
garm restore
garm compact
hrm QUERY
hrm run QUERY
tts TEXT
organos
organos audit
organos plan
organos run
organos health
organos actions
organos repair
organos feedback good
organos feedback bad
save
load
start
stop
```

## Local API Endpoints

```text
/ready
/state
/status
/api/status
/health
/api/health
/metrics
/api/metrics
/report
/api/report
/report/history
/api/report/history
/export
/api/export
/export/verify
/api/export/verify
/artifacts
/api/artifacts
/organs
/api/organs
/organs/actions
/api/organs/actions
/organs/audit
/api/organs/audit
/organs/recovery
/api/organs/recovery
/command?cmd=...
/api/command?cmd=...
/command_sync?cmd=...
/command_result?id=...
/command_forget?id=...
```

## Persisted Artifacts

Expected under `--state-dir`:

```text
graph.json
capabilities.json
runtime.json
legacy_memory.json
legacy_history.json
observatory.json
legacy_evolution.json
legacy_cognition.json
campo_tension.json
legacy_knowledge_graph.json
legacy_autoconsumo.json
legacy_venado.json
legacy_paradigm_hub.json
legacy_ecosystem.json
legacy_rebirth_meltrace.json
legacy_crawler.json
conscious_graph_regulator.json
context_augmentation.json
organ_autonomy.json
coordinator.json
human_interface.json
meta_architect.json
fast_reflexes.json
benchmark.json
command_router.json
persistence.json
telemetry.json
api_server.json
daemon.json
help.json
hrm_reasoner.json
voice_synthesizer.json
voice_last.txt
voice_backend_request.txt
voice_backend_output.txt
garm_report.txt
garm_report_history.jsonl
garm_export.json
backup/
legacy_memory.txt
```

`garm artifacts` is the local index for these artifacts. It reports existence, file bytes, JSONL entries, FNV64 checksums and a summary verdict.

## Verification Evidence

Last full local gate executed:

```bash
bash eden_core/src/garm/scripts/verify.sh
```

Observed result:

```text
cargo fmt --check -p eden_core: ok
cargo test -p eden_core eden_garm --lib: 75 passed
cargo check -p eden_core --examples --bins: ok
smoke_api.sh: passed
smoke_restart_persistence.sh: passed
cargo deny check advisories: advisories ok
cargo audit: completed with 5 allowed warnings
```

The restart smoke validates HRM execution counters, Voz/TTS artifacts, report/history, export/import, export integrity, artifact indexing, backup/restore, load after restart and organ deltas without `legacy:no_delta`.

## Security Matrix

```text
API bind: local-only 127.0.0.1
LLM dependency: none, local or external
TTS backend: optional; GARM writes request files but does not spawn backend processes
Import behavior: read-only validation; no state restoration or mutation
Export integrity: FNV64 operational checksum, non-cryptographic
Remote crawler: gated by explicit allow-remote-crawl path
Autonomous legacy_crawler: blocked by default; records not_executed
Code mutation by organs: not enabled
External network autonomy: not enabled by default
CI: intentionally not added in this phase
```

## Supply-Chain Warning Status

The prior warning set was resolved by replacing functionality first and removing obsolete dependencies last:

```text
atomic-polyfill: removed from active dependency graph
bincode 1.3.3: replaced by local wire-v1 gossip codec
bincode 2.0.1: removed with unused polars chain
paste: removed by replacing faer/nalgebra paths with local math kernels
lru 0.12.5: removed with unused surrealdb chain
```

These warnings were not removed by deleting runtime capability; replacement or preservation happened first.

## Limits

- FNV64 integrity is operational drift detection, not cryptographic tamper resistance.
- `garm import` validates exports only; it does not restore runtime state.
- `garm restore` copies from `state_dir/backup` and requires `load` to apply persisted state to the live runtime.
- Voz/TTS audio output requires an external local backend consuming `voice_backend_request.txt`.
- Remote crawling remains intentionally gated and is not autonomous by default.

## Handoff Position

This phase is releasable as a local-first, Rust-only, auditable runtime checkpoint. The minimum gate before further changes remains:

```bash
bash eden_core/src/garm/scripts/verify.sh
```

Additional non-CI local workflows are available for focused checks:

```bash
bash eden_core/src/garm/scripts/local_hardening_audit.sh
bash eden_core/src/garm/scripts/performance_smoke.sh
bash eden_core/src/garm/scripts/operator_summary.sh --state-dir /tmp/eden_garm_smoke
```
