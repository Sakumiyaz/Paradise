# EDEN GARM

GARM is the native unified EDEN operator runtime.

Release audit for the current local checkpoint: `RELEASE_AUDIT.md`.

## Build And Test

From the workspace root:

```bash
cargo build --bin eden-garm -p eden_core
cargo test -p eden_core eden_garm --lib
```

## Run Interactively

```bash
cargo run --bin eden-garm -p eden_core
```

Useful commands:

```text
help
observatorio
historial
aprende rust es seguro
que es rust
hrm rust
hrm run rust
goals plan mejorar arquitectura
goals run
eval run
learning record evaluacion mejora calibracion
world observe lluvia causes suelo_mojado
world predict lluvia
bench run
exec plan mejorar benchmark local
exec run
attention riesgo rollback benchmark
uncertainty record unknown rollout risk
experiment plan benchmark improves evaluation calibration
experiment run
provenance record tests passed for local experiment
policy eval local benchmark action
maturity assess policy guard
hybrid voice synth hola desde arquitectura hibrida
hrm text corpus /tmp/local_corpus.txt
hrm text ingest /tmp/local_corpus_dir
hrm text search runtime evidence
rag answer runtime evidence
rag eval
hrm text objective text to HRM runtime priors
hrm text plan
hrm text run
locus eval
locus ingest user notes :: local context stays evidence until GEWC accepts it
locus context local context
locus audit
operator forge eval
operator forge synth causal risk model for local action cost under uncertainty
operator forge verify
operator forge audit
model runtime eval
model audit
eden 70b modular eval
artifact api eval
operational runtime eval
tts hola desde GARM
garm audit
garm backup
garm compact
evolve
stop
start
save
load
quit
```

## Run As Daemon

```bash
target/debug/eden-garm \
  --daemon \
  --api-port 8080 \
  --pid-file /tmp/eden_garm.pid \
  --log-file /tmp/eden_garm.log \
  --state-dir /tmp/eden_garm
```

Default state directory:

```text
/tmp/eden_garm
```

Snapshot files:

```text
graph.json
capabilities.json
runtime.json
legacy_memory.json
legacy_history.json
observatory.json
legacy_evolution.json
legacy_cognition.json
organ_autonomy.json
hrm_reasoner.json
voice_synthesizer.json
voice_last.txt
voice_backend_request.txt
voice_backend_output.txt
hrm_text_pretraining.json
hrm_text_corpus_manifest.txt
hrm_text_segments.jsonl
hrm_text_checkpoint_manifest.txt
garm_report.txt
garm_report_history.jsonl
garm_export.json
eden_locus_layer.json
locus_authority_model.json
locus_evidence_vault.json
locus_permission_matrix.json
locus_context_packet.json
locus_operator_timeline.jsonl
eden_operator_forge.json
operator_primitive_basis.json
operator_expression_graphs.jsonl
operator_verification_report.json
operator_model_registry.json
locus_operator_bridge.json
eden_70b_modular_target.json
eden_70b_module_router.json
eden_70b_dataset_manifest.json
eden_70b_launcher_manifest.json
eden_70b_checkpoint_admission.json
eden_70b_inference_runtime.json
eden_70b_operational_demo.json
eden_70b_operational_gate.json
backup/
legacy_memory.txt
```

`eden_70b_modular_target.json` is a governed target definition only. It records
EDEN's future 70B modular family as six GEWC-routed models, not a single dense
checkpoint, and does not execute training or admit weights.

`eden 70b modular eval` writes the seven follow-up artifacts that make that
target operationally inspectable: router, datasets, launchers, checkpoint
admission, inference runtime, demo trace and aggregate gate.

## API Endpoints

Base URL defaults to `http://127.0.0.1:8080`.

```text
/
/console
/api/console
/api/help
/ready
/state
/status
/metrics
/organs
/organs/actions
/organs/audit
/organs/recovery
/command?cmd=...
/command_result?id=...
/command_forget?id=...
/command_sync?cmd=...
```

Examples:

```bash
curl -s 'http://127.0.0.1:8080/ready'
curl -s 'http://127.0.0.1:8080/api/help'
curl -s 'http://127.0.0.1:8080/state'
curl -s 'http://127.0.0.1:8080/metrics'
curl -s 'http://127.0.0.1:8080/organs'
curl -s 'http://127.0.0.1:8080/organs/recovery'
curl -s 'http://127.0.0.1:8080/command_sync?cmd=observatorio'
curl -s 'http://127.0.0.1:8080/command_sync?cmd=hrm+rust'
curl -s 'http://127.0.0.1:8080/command_sync?cmd=hrm+run+rust'
curl -s 'http://127.0.0.1:8080/command_sync?cmd=tts+hola+desde+GARM'
curl -s 'http://127.0.0.1:8080/command_sync?cmd=garm+audit'
curl -s 'http://127.0.0.1:8080/command?cmd=historial'
```

Async command flow:

```bash
curl -s 'http://127.0.0.1:8080/command?cmd=historial'
curl -s 'http://127.0.0.1:8080/command_result?id=1'
curl -s 'http://127.0.0.1:8080/command_forget?id=1'
```

`/command_result` is non-destructive and retained for bounded polling. `/command_forget` removes a stored result explicitly.

## Legacy Intents Migrated

Migrated REPL behavior includes:

```text
remember/recuerda/aprende/learn
memory/memoria/que recuerdas
search/busca/memoria X
que sabes de X
que es X / what is X
why X / por que X
tell me about X / cuentame X
hola / quien eres / que piensas / como te sientes / phi
observatorio / dashboard
historial / log
start / stop
evolve / improve
save / load
guarda / guardar / persiste
carga / recupera / restore
adios / bye / salir
que tal / buenos
que haces / que pasa
cual es la razon X
cognicion profunda legacy: curiosity, mission, self-model, dream consolidation, shared knowledge
```

## Operational Checkpoint

Current checkpoint expectations:

```text
single runtime: src/bin/eden_garm.rs -> eden_garm::runtime::GarmRuntime
api bind: 127.0.0.1 only
state dir default: /tmp/eden_garm
async result retention: bounded to 128 completed commands
organ autonomy: per-organ audit/action state in organ_autonomy.json
organ deltas: real API effects expose observable before/after counters where available
warnings: cargo build/test for eden_garm should be clean
tests: command/API/persistence/legacy cognition coverage should pass
legacy source: preserved under eden_garm/legacy_repl.rs and legacy_sources/
hrm: native local hierarchical planner; no LLM dependency
voice: optional local TTS organ; text manifest fallback when no audio backend exists
```

Recommended verification before handoff:

```bash
bash eden_core/src/garm/scripts/verify.sh
```

`verify.sh` is the minimum local gate before changing GARM. It runs formatting, GARM tests, all `eden_core` examples/binaries, the API smoke test, `cargo deny check advisories`, and `cargo audit`.

Long-cycle local verification without opening the API:

```bash
cargo run --bin eden-garm -p eden_core -- \
  --no-interactive \
  --max-cycles 226 \
  --api-port 0 \
  --state-dir /tmp/eden_garm_long_cycle
```

## Smoke API Test

Run the daemon/API smoke test from the workspace root:

```bash
bash eden_core/src/garm/scripts/smoke_api.sh
bash eden_core/src/garm/scripts/smoke_restart_persistence.sh
```

Optional environment variables:

```bash
GARM_SMOKE_PORT=8111 \
GARM_SMOKE_STATE_DIR=/tmp/eden_garm_smoke_custom \
bash eden_core/src/garm/scripts/smoke_api.sh
```

The script verifies `/ready`, `/state`, `/metrics`, `/organs`, `/organs/actions`, `/organs/audit`, `/organs/recovery`, async command results, `save/load`, `stop/start`, `observatorio`, `evolve`, and legacy reasoning.

## Non-CI Local Workflows

These scripts are intentionally local and are not CI configuration:

```bash
bash eden_core/src/garm/scripts/local_hardening_audit.sh
bash eden_core/src/garm/scripts/performance_smoke.sh
bash eden_core/src/garm/scripts/operator_summary.sh --state-dir /tmp/eden_garm_smoke
```

`local_hardening_audit.sh` checks locked dependency fetch, cargo-deny advisories, cargo-audit and duplicate dependency inventory. `performance_smoke.sh` builds the release example and measures bounded no-API cycles. `operator_summary.sh` gives a compact view of report/history/export/TTS artifacts for an existing state directory.

## Organ Autonomy

The 32 primary organs use a per-organ autonomy registry. Each organ has a bounded action queue, audit trail, feedback counters, execution counters and persisted recovery metadata. The crawler remains intentionally blocked for remote network autonomy unless a separate explicit crawl flag is used.

Useful commands:

```text
organos
organos audit
organos plan
organos run
organos health
organos actions
organos feedback good
organos feedback bad
```

`organos run` records an `effect` and `delta` for every action. Deltas prefer concrete counters such as memory facts, KG edges, CAG cache/action metrics, lifecycle thoughts, readiness observations, Venado reads/writes, ecosystem births/deaths and Meltrace events. If a real API call has no counter change, the audit records the concrete `real_api:*` effect plus the unchanged observable snapshot; blocked actions record `not_executed`.

## Goal Scheduler

The goal scheduler is a GEWC native-body architectural seam for objectives and action contracts. `goals plan QUERY` creates a persisted goal with priority, risk, status, evidence requirements and three explicit local action contracts for CAG, HRM and organ autonomy. `goals run` closes ready contracts conservatively and `goals audit` shows recent goals/contracts. HRM executions and manual `organos run` also register external goals so reports and exports include objective-level traces.

## Evaluation Loop

The evaluation loop is a local measurement seam for architecture progress. `eval run` records a bounded snapshot in `evaluation_loop.json` and scores architecture, evidence, execution and learning using local signals from goals, organ audit, HRM, benchmark, memory, KG and readiness. `eval audit` shows recent evaluations and regression risk. These scores are operational heuristics, not external proof.

## Readiness Architecture

`readiness` reports EDEN architecture readiness as an auditable gate framework. The report scores learning, planning, grounding, world modeling, memory, self-correction, generalization, scaling, verifiable RAG, operational safety, continuous evaluation and governed autonomy, then emits blockers and next actions while preserving `no_claim_until_all_gates_pass`. `readiness bench` measures current evidence only; `readiness probe` explicitly generates local phase evidence. `readiness plan` converts next actions into local goal contracts with explicit evidence requirements. `readiness run` evaluates only readiness goals against current local evidence and blocks contracts whose required evidence is missing. `readiness external run` executes the local held-out pre-validation harness without claim, and `readiness package` writes a reproducible artifact bundle with hashes. `memory eval`, `world eval`, `cognitive eval`, `embodied eval`, `neural eval`, `symbolic eval`, `self improvement eval`, `frontier architecture eval`, `paradigm architecture eval`, `integration governance eval`, `global executive workspace eval`, `gewc operational benchmark`, `capability reality eval`, `architecture advantage eval`, `praxis nexus eval`, `operational runtime eval`, `external ecosystem eval`, `sovereign cognition eval`, `runtime state api eval`, `operational api eval`, `artifact api eval` and `capabilities audit` add robust memory, predictive-loop, cognitive-architecture, embodied-grounding, neural-architecture, symbolic-architecture, bounded self-improvement, frontier formal layers, paradigm-gap layers, executive integration governance, HEC/GEWC core coordination, operational GEWC benchmark evidence, current capability reality classification, architecture advantage movement contracts, Praxis Nexus formal substrate evidence, executable operational runtime evidence, EDEN external ecosystem fabric, sovereign EDEN-vs-Hyperon architecture win sectors, typed runtime state APIs, operational control APIs, executable artifact APIs and capability-registry artifacts to the validation path. `frontier architecture eval` writes formal no-claim artifacts for safety-control, foundation-model, multimodal, LLM-agent, probabilistic programming, hierarchical RL, cognitive robotics, VLA, sim-to-real, open-ended evolution, developmental robotics, whole-brain/neurocognitive and neuromorphic/spiking layers. `paradigm architecture eval` is the single paradigm authority: it writes a no-claim map for the 24 AGI paradigms, absorbs the 43 legacy ParadigmHub items into `paradigm_architecture_technique_map.json`, classifies each legacy item as a subtype, alias, implementation detail, archived, future, replaced or formalized item under those 24 paradigms, and promotes none of them to a new formal paradigm. `integration governance eval` writes a no-claim artifact for executive integration, safe continual learning, physical/social grounding, causal world modeling, metacognition, stable goals, complete evaluation, scalable alignment and action boundaries. `global executive workspace eval` writes a no-claim artifact for the Global Executive Workspace Core design synthesis: global cognitive workspace, agentic-deliberative executive, metacognitive safety regulation and 16 internal coordination components. `gewc operational benchmark` writes `gewc_operational_benchmark.json`, `gewc_runtime_safety_report.json` and `gewc_long_run_stability.json` for local prevalidation of generality, transfer, governed autonomy, safe learning, robustness, runtime safety and restart stability. `capability reality eval` writes `capability_reality_eval.json`, `capability_reality_matrix.json` and `lmm_training_dependency_report.json`, separating runtime capabilities from local heuristics, architecture-only artifacts, simulated/stubbed areas, LMM-training dependencies, external-validation dependencies and safety-blocked actions. `architecture advantage eval` writes `gewc_trace_spec.json`, `capability_reality_matrix_v2.json`, `cognitive_task_suite.json`, `eden_agent_sdk_contract.json`, `model_adapter_layer.json`, `reproducible_demos.json` and `architecture_advantage_eval.json` to formalize the six movements for competing architecturally against LangGraph, AutoGen/Magentic-One, OpenHands and OpenCog Hyperon. `praxis nexus eval` writes `eden_praxis_nexus.json`, `praxis_primitives.json`, `praxis_blocks.json`, `praxis_space.json`, `praxis_rules.json`, `praxis_trace_semantics.json`, `praxis_reasoner.json` and `praxis_bench.json` to formalize EDEN's original governed cognitive-operational substrate: intent, state, evidence, constraint, affordance, projection and trace over PraxisSpace, Praxis Rules, GEWC Trace Semantics, Praxis Reasoner and PraxisBench. `locus eval` writes `eden_locus_layer.json`, authority, evidence-vault, permission, context-packet and operator-timeline artifacts for native personal-context governance. `operator forge eval` writes `eden_operator_forge.json`, primitive-basis, expression-graph, verification and model-registry artifacts for typed formal synthesis candidates. `operational runtime eval` writes the eight-component runtime phase, including `locus_operator_bridge.json`, which admits Locus context and Operator Forge candidates into CWM only as governed hypotheses. `external ecosystem eval` writes `eden_external_ecosystem.json`, `ecosystem_participation_contract.json`, `ecosystem_interop_matrix.json`, `ecosystem_certification_ladder.json`, `ecosystem_onboarding_runbook.json`, `ecosystem_governance_model.json` and `ecosystem_benchmark_exchange.json` so EDEN's ecosystem path is contract-first, certifiable, reproducible, governed and original rather than an AtomSpace/MeTTa clone. `sovereign cognition eval` writes `eden_sovereign_cognition.json`, `sovereign_sector_wins.json`, `praxis_calculus_formalism.json`, `cognitive_contract_language.json`, `evidence_memory_fabric.json`, `federated_runtime_fabric.json` and `symbolic_reasoning_fabric.json` for EDEN's original superiority targets over Hyperon: central core, formalism, distributed scalability, originality, memory/knowledge, multi-paradigm integration, ecosystem, maturity, symbolic reasoning, cognitive language and formal representation. `runtime state api eval` writes `runtime_state_api_catalog.json`, `runtime_state_api_contracts.json`, `runtime_state_api_openapi.json` and `runtime_state_api_runtime.json`, using the runtime state inventory as the whitelist for `/api/runtime/catalog`, `/api/runtime/state?name=...`, `/api/runtime/snapshot` and `/api/runtime/openapi`. `operational api eval` writes `operational_api_catalog.json`, `operational_api_contracts.json`, `operational_api_openapi.json`, `operational_api_runtime.json` and `operational_action_contracts.json`, exposing capability, GEWC, validation and action dry-run contracts through the operational API surface. `artifact api eval` writes `artifact_api_catalog.json`, `artifact_api_contracts.json` and `artifact_api_runtime.json`, using the reproducible package inventory as the whitelist for `/api/artifact/catalog`, `/api/artifact?name=...` and `/api/artifact/runtime`. At runtime every non-shutdown parsed command now runs inside `dispatch_gewc_cycle`: `GewcBodyRegistry` maps each core intent to a typed `GewcBodyHandler`, execution unit and lifecycle policy; `GewcBodyExecutor` dispatches every registered handler through domain-owned implementation modules with `shared_body_engine=false`, applies pre-execution safety checks and reports per-handler decision/completion/block metrics in `global_executive_workspace_runtime.jsonl` and `global_executive_workspace_runtime_state.json`. Historical GARM and legacy sources remain as implementation provenance inside GEWC-owned body domains rather than separate nuclei. `eden-garm-package-validator` is the independent native release-candidate runner: it consumes `readiness_package.json`, verifies critical artifact hashes, checks suite/result consistency, runs local adversarial controls and writes `independent_validation_report.json` plus `release_candidate_manifest.json`.

`model runtime eval` writes `model_adapter_runtime.json`, `model_checkpoint_manifest.json`, `training_harness_report.json` and `model_governance_report.json`. `model register/load/evaluate/unload <id>` records lifecycle events for subordinate model adapters under GEWC authority. This path does not train a production model, does not admit weights and does not let model outputs mutate memory, objectives or tools directly.

`first model prepare` is the formal 4A gate for the first EDEN model candidate. It writes `first_model_card.json`, `first_model_training_plan.json` and `first_model_readiness.json` after refreshing the model runtime artifacts. It keeps `training_executed=false`, `weights_present=false`, `gpu_job_submitted=false` and `4b_training_allowed=false`; future training remains a separate explicit 4B request.

`elcp prepare` is the formal gate for Eden Latent Cognitive Prediction. It writes `elcp_objective_spec.json`, `elcp_transition_dataset.json`, `elcp_training_plan.json`, `elcp_admission_gate.json`, `elcp_trace_quality_gate.json`, `elcp_replay_eval.json`, `elcp_dataset_freeze_manifest.json`, `elcp_metrics_board.json`, `elcp_4b_readiness_contract.json` and `elcp_readiness.json` after refreshing the first-model/runtime artifacts. ELCP makes token prediction subordinate to governed cognitive-state transition prediction, and it keeps `training_executed=false`, `weights_present=false`, `gpu_job_submitted=false`, `checkpoint_admission_allowed=false` and `4b_training_allowed=false`.

## Learning Ledger

The learning ledger is a persisted seam for learning claims. `learning record TEXT` stores a hypothesis with source, evidence, outcome, confidence and status in `learning_ledger.json`. `learning consolidate` promotes sufficiently evidenced observations and `learning audit` shows recent entries. HRM execution and `eval run` also write learning entries so architecture progress has a learning trail, not just command output.

## World Model Core

The world model core is a persisted seam for local observations, predictions and verification. `world observe TEXT` records simple relations such as `A causes B` or `A is B` in `world_model_core.json`. `world predict QUERY` produces a supported or unverified local prediction and writes a learning ledger entry. `world verify` marks supported predictions as locally verified, and `world audit` shows recent observations and predictions.

## Competence Benchmark

The competence benchmark is a local reproducibility seam for measuring whether GARM's architectural capabilities are present together. `bench run` scores planning, measurement, learning, prediction, execution and reasoning evidence from local reports only, persists the result in `competence_benchmark.json`, and records a learning ledger entry. `bench audit` shows recent runs. It is an operational benchmark harness, not an external capability claim.

## Plan Executor

The plan executor is a bounded local execution seam with scoring and rollback. `exec plan TEXT` records a safe plan with local-only steps, `exec run` scores available GARM evidence before marking the plan completed or rolled back, and `exec audit` shows recent plans. It persists to `plan_executor.json` and writes a learning ledger trace; it does not execute shell commands, mutate code or open network access.

## Working Memory / Attention

Working memory is a bounded local focus seam. `attention TEXT` records an attention item with a deterministic weight, `attention clear` clears active focus, and `attention audit` shows recent focus items. It persists to `working_memory.json`, feeds learning traces and is included in benchmark/evaluation evidence. It does not replace long-term memory or KG storage.

## Uncertainty Ledger

The uncertainty ledger records local risk calibration. `uncertainty record TEXT` stores a claim with deterministic confidence, risk class and mitigation, `uncertainty resolve` marks the next open item mitigated, and `uncertainty audit` shows recent records. It persists to `uncertainty_ledger.json`, feeds learning traces and is included in benchmark/evaluation evidence.

## Experiment Runner

The experiment runner records local architecture hypotheses and evaluates them against GARM evidence. `experiment plan TEXT` stores a hypothesis, `experiment run` scores local evaluation, benchmark, learning, uncertainty, executor and attention evidence, and `experiment audit` shows recent outcomes. It persists to `experiment_runner.json` and writes learning traces without shell execution, network access or external models.

## Provenance Ledger

The provenance ledger records local evidence sources for claims. `provenance record TEXT` stores a claim with evidence kind, source trust and status, `provenance verify` verifies the next pending record, and `provenance audit` shows recent records. It persists to `provenance_ledger.json`, feeds learning traces and is included in benchmark/evaluation evidence.

## Policy Guard

The policy guard records local constraint decisions. `policy eval TEXT` evaluates an action against local-only, no-shell, no-remote-network and no-code-mutation rules, while `policy audit` shows recent decisions. It persists to `policy_guard.json`, feeds learning traces and is included in benchmark/evaluation evidence.

## Capability Maturity

The capability maturity ledger records local maturity assessments for GARM seams. `maturity assess TEXT` scores a capability from local evidence across goals, evaluation, learning, benchmark, executor, policy and provenance reports. `maturity audit` shows recent assessments and `capability_maturity.json` stores the ledger.

## HRM And Voz/TTS

HRM is a GEWC native-body organ named `hrm_reasoner`. It uses deterministic Rust logic over memory, KG retrieval, CAG context packs and history fragments. It does not call an LLM. The command `hrm QUERY` emits layered reasoning and an auditable plan across strategic, episodic, semantic and action layers.

`hrm run QUERY` executes the HRM plan through existing CAG and organ autonomy surfaces. It remains bounded and auditable: no code mutation, no remote network, and no separate runtime.

Voz/TTS is a GEWC native-body organ named `voice_synthesizer`. It is optional and local. The command `tts TEXT` writes a bounded manifest to `voice_last.txt` under `--state-dir` when no audio backend is configured, so the runtime never blocks or fails because TTS is unavailable. If `EDEN_GARM_TTS_BACKEND` is set, the GEWC native body writes `voice_backend_request.txt` for an external local backend to consume; it still does not spawn or require that backend. The helper `scripts/local_tts_backend.sh --state-dir DIR` consumes that request and writes `voice_backend_output.txt` as a local text-rendered manifest.

Hybrid voice is an architecture seam for a future neural TTS backend: `hybrid voice plan TEXT` records a stacked-transformer backbone plus GARM hierarchical loop plan, and `hybrid voice synth TEXT` writes `hybrid_voice_manifest.txt` before delegating to the existing optional TTS manifest flow. It does not ship neural weights or generate waveform audio by itself.

HRM-text pretraining is an operational seam for future text pretraining priors. `hrm text corpus PATH` and `hrm text objective TEXT` register local corpus/objective metadata, `hrm text ingest DIR` indexes supported local corpus files into bounded JSONL segments with dedupe, document versions and source trust, `hrm text search QUERY` retrieves top local segments with deterministic lexical scoring plus confidence/citation metadata, `rag answer QUERY` writes a restricted `hrm_text_context_pack.json` or abstains on insufficient evidence, `rag eval` runs deterministic answerable/unanswerable/citation checks, `hrm text plan` builds a curriculum bridge to HRM runtime and hybrid voice, and `hrm text run` writes `hrm_text_checkpoint_manifest.txt` while feeding learning, provenance, policy and maturity ledgers. It does not train neural weights; the manifest records `weights_present=false` and `training_executed=false`.

Corpus registration writes `hrm_text_corpus_manifest.txt` with local file metadata: existence, bytes, FNV64 checksum, line count, inferred language/domain and source. The local helper `scripts/local_tts_backend.sh` can consume both `voice_backend_request.txt` and `hybrid_voice_manifest.txt`; it still emits a text manifest only and records `waveform_generated=false`.

Directory ingestion writes `hrm_text_segments.jsonl` with bounded local text segments and is included in benchmark, evaluation and maturity evidence. `hrm QUERY` and `hrm run QUERY` include matching HRM-text segments as additional local evidence when present. Context packs cite exact `doc:ID#seg:ID@vVERSION:fnv64` fragments and low-confidence retrieval emits `[HRM-TEXT-ABSTAIN]`. `scripts/checkpoint_compare.sh --left DIR --right DIR` compares two local checkpoints using export checksum, report verdict and HRM-text corpus/segment stats.

`scripts/release_checkpoint.sh` runs a local daemon checkpoint, executes HRM-text/hybrid voice/report/export/verify/artifacts/backup commands, writes an operator summary, and creates a handoff archive under `/tmp`.

## Maintenance Commands

```text
garm audit   - integrated runtime/organs/HRM/Voz/CAG/state health report
garm report  - compact operational snapshot persisted to garm_report.txt and served at /report
garm report history - latest bounded report entries from garm_report_history.jsonl
garm export  - write diagnostic garm_export.json and serve it at /export
garm import  - validate garm_export.json read-only; does not restore state
garm verify export - verify local FNV64 export integrity; non-cryptographic
garm artifacts - list operational files, bytes, JSONL entries and FNV64 checksums
garm backup  - copy current state files to state_dir/backup
garm restore - copy state_dir/backup files back; run load afterwards
garm compact - run safe KG compaction and save state
```
