# GARM Legacy Migration Map

This file tracks the remaining historical REPL surface preserved in `legacy_repl.rs` and `legacy_sources/`.

## Preservation Rule

Do not remove legacy behavior, dependency surface, wire compatibility, or latent extension points only because they are currently unused. First repair and improve broken pieces. If repair is not viable, replace them with a focused GARM-native implementation that preserves all user-visible behavior and compatibility boundaries.

## Migrated Into Active GARM

Active command routing now covers:

```text
greeting: hola, hello, hi, hey, que tal, buenos
goodbye: adios, bye, salir, exit, quit, hasta luego
identity: quien eres, who are you, tu mismo, yourself, tu identidad
status: estas, estado, status, que pasa, que haces
help: ayuda, help, comandos, commands, que puedes
phi: phi, conciencia, consciousness, consciencia, medir, medicion
memory write: recuerda, remember, aprende, aprendizaje, recorder, learn
memory read: memoria, memory, que sabes, que recuerdas, what do you know, what do you remember
memory query: que sabes de X, what do you know about X, busca X, search X, memoria X
reasoning: que es X, what is X, definicion de X, definition of X, explicame X, explain X
why: por que X, why X, cual es la razon X, causa de X, motivo de X
tell me: cuentame X, hablame de X, dime sobre X, tell me about X, que opinas de X
persistence: save, load, guarda, guardar, persiste, carga, recupera, restore
history: historial, mi historial, ver historial, registro, log, eventos evolutivos, log de eventos, registro evolutivo
observatory: observatorio, dashboard, metricas, sistemas, estado global, panorama, ver todo
autonomy: start, stop, iniciar, despierta, empieza, vivir, awake, corre, run, detener, para, duerme, pausa, halt, quieto
evolution: evoluciona, mejorate, evolve, improve, grow, self-improve, mutate
```

The legacy `SimpleNLP` keyword lists in `legacy_repl.rs` are fully represented in `CommandRouterNode` and covered by `covers_all_legacy_simple_nlp_keywords`.

## Rich Conversational Responses

Legacy conversational response behavior is migrated into active nodes:

```text
LegacyDialogueNode: rich greeting, identity, thinking, feeling, phi/medicion responses
HelpNode: expanded command help with all legacy aliases
LegacyReasonNode: what-is, why, tell-me, and memory-grounded query responses
ObservatoryNode: runtime/dashboard response surface
LegacyHistoryNode: historical log response surface
LegacyEvolutionNode: bounded evolve/improve response surface
```

The active responses are GARM-native and runtime-aware. They do not restore the standalone legacy REPL loop.

## Deep Cognitive Structures Migrated

The deep legacy cognitive structures are represented by `LegacyCognitionNode`:

```text
CuriosityDrive -> knowledge_gaps, exploration_history, curiosity_threshold, total_information_gain, unexplored_domains
KnowledgeGap -> topic, uncertainty, last_explored, exploration_count, information_potential
ExplorationTarget -> target, information_gain, energy_cost, timestamp, success
Mission/SubGoal -> current_mission, sub_goals, progress, relevance, status, success_criteria
SelfModel -> capabilities, limitations, known_topics, unknown_topics, skills, learning_goals, lineage metadata
EmotionalState baseline -> valence/arousal/dominance and Plutchik-like channels
DreamState/MemoryConsolidation -> dream consolidation targets, processed memories, creativity output
SharedKnowledge -> shared knowledge records with trust/usefulness/tags
```

`LegacyCognitionNode` is part of the active graph, has tests, and participates in optional persistence as `legacy_cognition.json`.

## Preserved But Not Active As Standalone Runtime

Historical daemon/API scaffolding, old interactive REPL flow, and prior orchestration remain preserved for reference, not as an active entry point:

```text
legacy_repl.rs
legacy_sources/eden_repl.rs
legacy_sources/eden_repl.rs.bak
legacy_sources/eden_repl.rs.debug
```

## Remaining Historical Code

The remaining historical code is preserved source material, not command/intents or required cognitive runtime surface:

```text
1. Old standalone daemon JSON shapes, only needed if an external legacy client requires compatibility.
2. Historical self-modification implementation details that are intentionally not restored outside bounded GARM evolution.
3. Old REPL orchestration code that duplicates the active GARM runtime.
```

Do not restore a standalone REPL or V12 runtime. New behavior should enter through GARM nodes, capabilities, or local API control-plane endpoints.

## Audit Matrix

This matrix tracks the large functional blocks in `legacy_repl.rs` against active GARM replacements. Status terms:

```text
migrated: active GARM code exposes the same user/runtime behavior.
replaced: GARM has a smaller or stronger native subsystem for the same role.
preserved: historical source remains available but is intentionally not active.
partial: active GARM covers the useful behavior, but not every historical implementation detail.
repaired: the legacy surface remains available, but vulnerable or broken dependencies were upgraded/fixed rather than deleted.
```

| Legacy area | Active GARM target | Status | Evidence |
| --- | --- | --- | --- |
| CLI parsing, standalone REPL loop, `print_help` | `GarmRuntime::from_args().run()`, `CommandRouterNode`, `HelpNode` | migrated/replaced | Single entry point is `src/bin/eden_garm.rs`; no separate REPL runtime is restored. Legacy `--born`, `--session`, and `--log-level` are accepted for compatibility. |
| Unix daemonization, pid/log files | `DaemonNode`, runtime `--daemon --pid-file --log-file` | migrated | Covered by smoke daemon lifecycle and local API stop/start path. |
| Legacy HTTP `/health`, `/status`, `/metrics`, `/command` JSON | `ApiServerNode` endpoints `/health`, `/api/health`, `/ready`, `/state`, `/status`, `/api/status`, `/metrics`, `/api/metrics`, `/command*` | migrated/replaced | Legacy health/status/metrics JSON and POST body command compatibility are active; GARM also exposes richer async/sync command APIs. |
| `EdenLogger` and `LogLevel` | Runtime/daemon stdout and configured daemon log file | replaced | GARM keeps operational logging without restoring the old in-process logger API. |
| `EdenMetrics` and `ComplexityTracker` | `TelemetryNode`, `/metrics`, `BenchmarkNode`, `ApiRuntimeMetrics`, graph FE | replaced/partial | Active metrics are runtime-native; exact historical complexity formula is preserved only in source. |
| `SimpleNLP` and `Intent` keyword routing | `CommandRouterNode` and parser tests | migrated | `covers_all_legacy_simple_nlp_keywords` verifies the legacy keyword surface. |
| `EdenSession`, model weights, graph snapshots/WAL, meltrace/session files | `PersistenceNode`, `state_paths.rs`, `HyperGraph::save/load_state`, capability/runtime/legacy snapshots | migrated/replaced | State-dir snapshots replace ad hoc `/tmp` and dotfile persistence while migrating legacy paths non-destructively. |
| Eidetic/episodic memory and recall/search commands | `LegacyMemoryNode`, `LegacyReasonNode`, `Hippocampus`, `WorkingMemory`, `DNC` | migrated/replaced | Active commands persist `legacy_memory.json`, `legacy_memory.txt`; capability memory systems run under GARM. |
| Curiosity, knowledge gaps, exploration targets | `LegacyCognitionNode`, `Exploration` capability | migrated | Struct fields and update/selection behavior are represented and persisted as `legacy_cognition.json`. |
| Mission/subgoals/self-model/lineage | `LegacyCognitionNode`, `SelfModel`, `GoalStack`, `GoalExecutor`, `IntentionHierarchy` | migrated/replaced | Legacy mission state is active in `LegacyCognitionNode`; broader planning uses GARM capabilities. |
| Emotional baseline, Plutchik-like channels, mood | `LegacyCognitionNode`, `Mood`, `EmotionalModulation`, `Homeostasis` | migrated/replaced | Baseline fields are persisted; active regulation is delegated to GARM mood/homeostasis capabilities. |
| Dream/sleep memory consolidation | `LegacyCognitionNode::consolidate_dream`, `MemoryClustering`, `Hippocampus` | migrated/replaced | Dream state and consolidation records are represented without restoring the old REPL sleep scheduler. |
| Shared knowledge/hive mind/multi-agent pool | `LegacyCognitionNode::share_knowledge`, `MultiAgent`, `Swarm`, `AgentMesh`, `SocialComplex` | migrated/replaced | Shared records are persisted; active multi-agent behavior is GARM-native. |
| `EvolutionEngine`, `handle_evolve`, open-endedness events | `LegacyEvolutionNode`, `Evolution`, `MetaEvolution`, `OpenEndedness`, `SelfImprovement` | migrated/replaced | User-visible evolution command and bounded event history are active; old monolithic loop is not. |
| Organic timers `SistemaMaduro` | GARM temporal scales, graph pulses, capability-specific tick cadence | replaced | GARM scheduling is graph/FEP based rather than recreating each legacy timer object. |
| `CampoDeTension` tension-driven evolution | `CampoTensionNode`, `FEPEngine`, `Surprise`, `Epistemic`, `Homeostasis`, `ActiveInference`, `Gate` | migrated/replaced | The legacy five-source accumulator is active as `CampoTensionNode`; GARM also keeps FE/surprise/homeostasis regulation. |
| `RitmoCardiaco`, `EcoSistema`, resonance pool | `AgentMesh`, `Swarm`, `MultiAgent`, `SocialComplex`, graph node energy | replaced/partial | Multi-agent dynamics are active; named Eco lifecycle narrative is preserved, not reactivated. |
| `VenadoDeMemoria` custom `.vena` crystal format | `VenadoCompatibilityNode`, `state_paths.rs`, JSON snapshots, legacy text memory export | migrated/replaced | `.vena` read/write compatibility is active under `--state-dir`; JSON snapshots remain canonical for runtime state. |
| `TejidoDeConocimiento` organic KB files | `LegacyCrawlerNode`, `CorpusReader`, `CorpusMassive`, `LegacyMemoryNode`, `Semantics`, `MemoryClustering` | migrated/replaced | Local bounded `.txt` KB loading is active; corpus/memory clustering remains GARM-native. |
| `Autoconsumo` and `MapaDeArquitectura` self-source parser/hash | `AutoconsumoNode`, `ArchitectureModel`, `SelfModification`, `SelfAwareness`, `MetaArchitectNode` | migrated/replaced | Bounded self-source architecture extraction is active; unsafe self-modifying loops remain gated by GARM. |
| Language of thought, values, predictor/judge/autodoc/autodebug | `InternalLanguage`, `ConstitutionalSafety`, `BusPredictor`, `PredictiveLoop`, `AutoDebug`, `MetaArchitectNode` | replaced | Specialized GARM capabilities split these responsibilities. |
| Rich `KnowledgeGraph` with relation types, TTL, source trust, temporal query, hybrid RAG | `LegacyKnowledgeGraphNode`, `HyperGraph`, `Semantics`, `Causality`, `CausalModel`, `Evidence`, `Epistemic`, `LogicReasoning`, `WorldModel`, `LegacyReasonNode` | migrated/replaced | ES/EN relation parsing, source trust, TTL expiry, temporal query, hybrid retrieval, KG path explanations, and hypotheses are active in `LegacyKnowledgeGraphNode`; broader reasoning remains GARM-native. |
| Crawling, HTTP fetch, local KB, ConceptNet/Wikidata-style loaders | `LegacyCrawlerNode`, `ToolCalling`, `McpClient`, `ComputerUse`, `Sandbox`, `CorpusReader` | migrated/replaced | Local KB and bounded ConceptNet URI/TSV/weight import are active; remote crawling is available via `crawl URL` only when `--allow-remote-crawl` is set and URL/SSRF/size/timeout gates pass. |
| `ParadigmHub` and experimental AI paradigms | `paradigm architecture eval`, `paradigm_architecture_technique_map.json`, compatibility `ParadigmHubNode` snapshot | superseded | The 43 legacy items are absorbed into the formal paradigm architecture technique map as subtype, alias, implementation-detail, archived, future, replaced or formalized records under the 24 official paradigms. The compatibility node and `legacy_paradigm_hub.json` remain for historical snapshots, but autonomous paradigm cycling is superseded by `paradigm architecture eval`. |
| Narrative/experimental entities (`Umbra`, `Eco`, `MarMorfoseo`, rebirth variants) | `EcoSystemNode`, `RebirthMeltraceNode`, `LegacyCognitionNode`, `Phenomenology`, `SelfAwareness`, `Evolution` | migrated/replaced | Eco lifecycle, resonance pool, death/rebirth Meltrace events, selected inherited facts, metrics, and `rebirth`/`renacimiento` command behavior are active in bounded GARM nodes. |
| Preserved legacy source auditability | `legacy_migration::legacy_source_archive_report` | migrated/replaced | The migration report includes byte counts and FNV64 fingerprints for preserved legacy source files so archive drift is visible without compiling the monolith. |
| Legacy dependency surface and binary gossip format | `Cargo.toml`, `src/paradigms/distributed.rs`, `deny.toml` | repaired/replaced | Direct legacy-capability dependencies remain present when they represent latent functionality; unused heavy chains were removed only after replacement/preservation. Gossip now uses local `wire-v1` serialization instead of `bincode`. |
| Security warning triage | `LEGACY_FINAL_AUDIT.md`, `deny.toml`, `cargo audit` | repaired | Stable upgrades or local replacements are preferred when they preserve function; removals are not allowed solely because a dependency is inactive in the main runtime path. Current warning surface was reduced by replacing functionality first, then removing obsolete dependencies. |
| Graph direct message delivery | `HyperGraph::pulse`, `NodeAction::SendMessage`, `NodeContext::neighbor_outputs` | repaired/migrated | Direct node messages are now delivered to the target node context on later pulses, preserving the previously latent message-delivery intent instead of discarding it as unused work. |
| Explicit readiness gap tracking | `ReadinessNode`, `readiness` command, observatory append | active roadmap | The missing readiness requirements are represented as measurable dimensions: continuous skill learning, long-horizon planning, grounding/action, predictive models, integrated memory, self-correction, generalization, and cognitive scaling. |
| Organic internal narrative, Umbra, child-autons, theatrical rebirth | `OrganicLifecycleNode`, `ritual`/`umbra` command, observatory append | migrated/better | The legacy organism feel is restored as bounded graph behavior: Umbra marks, child-auton ecology, death/rebirth narration, and mixed experimental heuristics without returning to monolithic REPL control flow. |

## Known Non-Goals From The Audit

The audit does not justify reactivating these as-is:

```text
1. A second interactive REPL or V12-style monolithic runtime.
2. Exact legacy JSON response compatibility without a concrete external client.
3. Unbounded crawler/self-modifying/self-file-parsing loops outside GARM safety gates.
4. Historical custom persistence formats when state-dir JSON snapshots already preserve active state and byte-level compatibility is not required.
5. Narrative-only lifecycle mechanics that do not affect current command/API/runtime behavior.
```

If any item above becomes a requirement, implement it as a focused GARM node/capability with tests rather than restoring the old monolith.
