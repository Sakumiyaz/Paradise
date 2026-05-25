# GARM Legacy Final Audit

This audit records the current non-loss policy for `legacy_repl.rs`.

## Preservation Rule

GARM must not delete legacy functionality merely because it is currently inactive or not wired into the main runtime path. Removal or replacement is allowed only when one of these is true:

1. A clearly better implementation preserves the same observable functionality with no functional loss.
2. The old implementation does not work; repair and improve it first.
3. If it cannot be repaired, replace it from scratch while preserving all user-visible functions and compatibility surfaces.

Legacy source, dependency surface, wire formats, and latent extension points count as preservation concerns. When a replacement is chosen, the preserved behavior and compatibility boundary must be stated explicitly.

## Active In GARM

The following legacy domains are active as GARM runtime behavior, not only preserved source:

| Legacy domain | Active target |
| --- | --- |
| CLI/runtime commands | `GarmRuntime`, `CommandRouterNode`, `HelpNode`; compatibility accepts `--born`, `--session`, `--log-level` |
| Local API/daemon | `ApiServerNode`, `DaemonNode` |
| memory/session facts | `LegacyMemoryNode`, EDEN2/3/4 `.eden_session` fact import |
| reasoning/dialogue/status/phi | `LegacyReasonNode`, `LegacyDialogueNode`, `TelemetryNode` |
| history/observatory | `LegacyHistoryNode`, `ObservatoryNode` |
| evolution | `LegacyEvolutionNode`, GARM evolution capabilities |
| curiosity/mission/self/dream/shared knowledge | `LegacyCognitionNode` |
| tension/catharsis | `CampoTensionNode` |
| rich knowledge graph | `LegacyKnowledgeGraphNode` with ES/EN relation parsing, TTL, source trust, temporal query, RAG and hypotheses |
| autoconsumo/architecture scan | `AutoconsumoNode` |
| `.vena` crystal format | `VenadoCompatibilityNode` with fields, blocks, list, exists |
| paradigm coordination and autograd models | superseded by `paradigm architecture eval`; 43 legacy items are absorbed into `paradigm_architecture_technique_map.json` |
| eco/resonance lifecycle | `EcoSystemNode` |
| rebirth/meltrace lineage | `RebirthMeltraceNode`, `rebirth`/`renacimiento` command, death/rebirth events, inherited fact selection, legacy metrics |
| local KB/crawler/ConceptNet compatibility | `LegacyCrawlerNode`, bounded local KB and ConceptNet import with URI/TSV/weight support, remote crawl gated by `--allow-remote-crawl` |
| legacy source preservation | migration report includes byte counts and FNV64 hashes for preserved legacy source files |
| legacy dependency / wire surface | direct legacy dependency surface is preserved where it represents latent capability or compatibility; vulnerable dependencies are repaired by upgrade when possible instead of being removed for non-use |
| graph direct messages | `NodeAction::SendMessage` payloads are delivered to the target node context on later pulses rather than being treated as dead code |
| Readiness roadmap | `ReadinessNode` tracks learning, planning, grounding, prediction, memory, self-correction, generalization and scaling gaps |
| organic narrative / Umbra / child-autons | `OrganicLifecycleNode`, `ritual`/`umbra` command, bounded internal myth, child-auton ecology and emergent heuristics |

## Persisted State

`save`/`load` now includes active snapshots for:

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
legacy_memory.txt
```

Legacy `.eden_session` files are copied into the state directory as `legacy_session.eden` during path migration and imported into `LegacyMemoryNode` if no JSON memory snapshot is available.

## Replaced, Not Lost

These legacy mechanisms are intentionally not restored literally because GARM has active safer replacements:

| Legacy mechanism | Replacement |
| --- | --- |
| standalone `EdenREPL` loop | graph pulse runtime |
| monolithic autonomous cycle | GARM nodes/capabilities |
| direct remote crawler | `crawl URL` / `web URL` with `--allow-remote-crawl`, URL validation, SSRF blocking, byte limits, timeout, and memory/KG ingestion |
| ConceptNet text/CSV-style import | `conceptnet PATH` / `load conceptnet PATH`, bounded parser feeding memory and KG |
| explicit death/rebirth lifecycle command | `rebirth` / `renacimiento` through `RebirthMeltraceNode` with death events, selected inherited facts and metrics |
| manual source archive checking | `migration` / `legacy` report emits preserved file size/hash fingerprints |
| missing readiness requirements | `readiness` command reports measurable gaps without external claims |
| legacy organism feel | `ritual` / `umbra` command restores theatrical organism texture as a bounded node, not as monolithic control flow |
| legacy autograd model bank | legacy model identities are preserved in `paradigm_architecture_technique_map.json`; autonomous paradigm cycles are superseded |
| unsafe self-modification loops | `SelfModification`, `Gate`, `ConstitutionalSafety` |
| legacy HTTP control JSON | `/health`, `/api/health`, `/status`, `/api/status`, `/metrics`, `/api/metrics`, POST `/command`/`/api/command` body compatibility |
| old API bind to `0.0.0.0` | local-only `127.0.0.1` control plane |

## Remaining Intentional Differences

No known high-value functional legacy domain is only preserved without an active GARM target. Remaining differences are intentional safety or architecture differences:

```text
1. Remote crawling is disabled by default and only runs with `--allow-remote-crawl` plus URL/SSRF/size/timeout gates.
2. The old standalone REPL and V12-style monolith are not restored.
3. API network exposure is local-only even though legacy accepted external bind addresses.
4. GARM JSON snapshots are canonical; legacy formats are imported/compatible where useful.
5. Dependency cleanup must repair or preserve latent capability first; non-use alone is not evidence that removal is safe.
```

If a concrete external client or legacy data file requires stricter byte-level compatibility, add a focused compatibility adapter rather than restoring the monolith.

## Security Warning Triage

Current audit warnings are tracked as preservation tradeoffs, not ignored removals. The previous `bincode`, `lru`, `atomic-polyfill`, `paste`, and `bincode 2.x` warnings were removed from the active dependency graph by replacing binary gossip with local `wire-v1`, replacing `faer`/`nalgebra` math paths with local kernels, and removing unused heavy dependency chains.

| Advisory surface | Source | Current decision |
| --- | --- | --- |
| `bincode` unmaintained | former direct dependency | Replaced by local `wire-v1` UDP gossip serialization with size/UTF-8/magic validation. |
| `lru` unsound warning | former `surrealkv`/`surrealdb` transitive | Removed from active dependency graph with the unused `surrealdb` chain. |
| `atomic-polyfill` unmaintained | former target-specific transitive | Removed from active dependency graph with the unused heavy transitives. |
| `paste` unmaintained | former `faer`/`nalgebra` math stack transitive | Removed from active dependency graph by replacing those paths with local row-major/spectral kernels. |
| `bincode 2.x` unmaintained | former `polars` transitive | Removed from active dependency graph with the unused `polars` chain. |

If a future stable dependency release repairs any of these without reducing functionality, prefer upgrading over removing the capability.

## Performance Baseline

Preservation-safe performance measurements should record scenarios instead of removing latent behavior. Current release-mode baseline on this workspace:

| Scenario | Result |
| --- | --- |
| `--no-interactive --max-cycles 100000 --api-port 0` | ~5.15s wall time, ~283 MiB max RSS |
| API smoke lifecycle | Passed with local daemon/API command flow |
| ConceptNet structured import test | Passed; test workload ~9.77s including compile/check overhead, ~846 MiB max RSS |
| Paradigm/ecosystem/rebirth/crawler test | Passed; test workload ~9.90s including lock/build overhead, ~144 MiB max RSS |
| Legacy API shape/body command test | Passed; test workload ~13.30s including server wait and build overhead, ~284 MiB max RSS |

Future optimizations must preserve message delivery, legacy wire formats, dependency surface decisions, and command/API compatibility unless a focused replacement proves no functional loss.
