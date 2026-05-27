# EDEN Training Surface

This directory is the public, reproducible entry point for EDEN's future
trainable components. It does not contain production checkpoints and does not
claim trained AGI capability.

The intent is to let contributors validate the path from architecture to
measurable capability before expensive GPU work:

- define ROCm/AMD execution profiles;
- keep datasets and manifests explicit;
- run small CPU-safe smoke benchmarks in CI;
- later attach GPU-backed training jobs without changing GEWC authority;
- preserve claim gating in every generated report.

## Layout

| Path | Purpose |
| --- | --- |
| `configs/rocm_smoke.json` | Minimal AMD ROCm profile with CPU fallback for local validation. |
| `configs/first_model_memory_retrieval.json` | First trainable module contract: subordinate memory retrieval under GEWC. |
| `configs/elcp_latent_cognitive_prediction.json` | ELCP objective contract: latent cognitive transition prediction under GEWC. |
| `configs/eden_70b_modular_target.json` | EDEN-70B modular target freeze: 33B primary plus 12B CWM, 12B multimodal/VLA, 6B planner/tool, 4B safety and 3B memory/router modules. It is not one dense 70B model. |
| `data/manifest.json` | Explicit dataset manifest; synthetic repo-local data only. |
| `data/license_manifest.json` | Public dataset/license boundary; no private data, no external model dependency. |
| `data/capability_smoke.jsonl` | Tiny deterministic benchmark dataset for current operational capabilities. |
| `data/first_model_memory_*.jsonl` | Tiny train/eval fixtures for the first memory-retrieval baseline. |
| `data/elcp_transition_*.jsonl` | Tiny train/eval fixtures for ELCP cognitive-transition contracts. |
| `data/eden_cognitive_sft_elcp_*.jsonl` | Deterministic SFT/ELCP v2 train/eval data for the learned-capability pilot. |
| `data/build_eden_cognitive_sft_elcp.py` | Stdlib-only SFT/ELCP v2 generator; no private data and no external model. |
| `data/eden_real_capability_*.jsonl` | Repo-owned capability corpus built from EDEN docs, ADRs, training configs and runtime source excerpts. |
| `data/build_eden_real_capability_corpus.py` | Stdlib-only real-capability corpus builder; no private data and no external model. |
| `data/eden_v01_semantic_*.jsonl` | Larger curated semantic capability corpus for situation modeling, planning, memory, world model, tool policy, uncertainty, admission and rollback. |
| `data/build_eden_v01_semantic_corpus.py` | Stdlib-only EDEN v0.1 semantic corpus builder; no private data and no external model. |
| `data/eden_v02_stability_*.jsonl` | Stability corpus split for comparing 100-iteration and 250-iteration 7B checkpoint evidence. |
| `data/build_eden_v02_stability_corpus.py` | Stdlib-only EDEN v0.2 stability corpus builder; no private data and no external model. |
| `data/eden_v03_generalization_*.jsonl` | Larger v0.3 generalization corpus for long pretraining, checkpoint admission, native inference runtime, registry policy and 14B readiness. |
| `data/build_eden_v03_generalization_corpus.py` | Stdlib-only EDEN v0.3 corpus builder; repo-owned inputs only, no private data and no external model. |
| `data/eden_70b_modular_*.jsonl` | Tiny module-specific seed splits for the 70B modular family: primary, CWM, multimodal/VLA, planner/tool, safety and memory/router. |
| `data/build_eden_70b_modular_datasets.py` | Stdlib-only EDEN-70B modular seed builder; no private data and no external model. |
| `benchmarks/eden_capability_benchmark.py` | Stdlib-only benchmark runner and tiny trainable memory baseline. |
| `benchmarks/validate_capability_report.py` | Stdlib-only contract validator for `capability_report.json`. |
| `benchmarks/validate_elcp_transitions.py` | Stdlib-only validator for ELCP cognitive-transition fixtures. |
| `benchmarks/elcp_baseline_eval.py` | CPU-safe rule baseline for ELCP target fields. |
| `benchmarks/eden_real_capability_eval.py` | Operational eval for dataset coverage, 7B evidence, inference, SFT/ELCP packets and no-claim gates. |
| `benchmarks/eden_v01_semantic_eval.py` | Stronger semantic eval requiring the larger corpus, 7B training beyond the pilot, checkpoint inference and no-claim gates. |
| `benchmarks/eden_v02_stability_eval.py` | Checkpoint-stability eval comparing 100-iteration baseline evidence against a 250-iteration candidate. |
| `benchmarks/eden_v02_adversarial_eval.py` | Deterministic adversarial architecture eval for injection, permissions, rollback and model-authority boundaries. |
| `benchmarks/eden_v02_rollback_drill.py` | Rollback-readiness drill that requires stability and adversarial reports before candidate runtime admission. |
| `benchmarks/eden_v02_model_card.py` | Internal model-card and checkpoint-storage manifest for the v0.2 candidate. |
| `benchmarks/eden_v03_generalization_eval.py` | v0.3 checkpoint-admission evaluator requiring a larger corpus, 1000-iteration 7B evidence, inference load, safety inheritance, checkpoint registry and 14B scaling policy. |
| `benchmarks/paradise_non_gpu_readiness.py` | Paradise non-GPU product/runtime readiness gate. It validates docs, model-interface authority, dataset governance, checkpoint admission policy and public boundaries without GPU or checkpoints. |
| `demos/eden_v01_operational_demo.py` | Non-mutating operational demo trace that connects semantic eval, inference and SFT/ELCP packets through GEWC. |
| `demos/eden_v02_stability_demo.py` | Non-mutating stability demo trace that exercises the v0.2 checkpoint-admission path. |
| `demos/eden_v03_operational_demo.py` | Non-mutating v0.3 trace for long-checkpoint admission, registry, native runtime candidate and 14B readiness. |
| `elcp/export_trace_candidates.py` | Redacted GEWC trace exporter for candidate ELCP transitions. |
| `elcp/train_elcp.py` | Dry-run-only training interface for future ELCP 4B work. |
| `elcp/admission_gate.py` | Pre-checkpoint ELCP admission policy report. |
| `elcp/trace_quality_gate.py` | Reviews exported traces for shape, duplication, authority and sensitive markers. |
| `elcp/replay_eval.py` | Replays reviewed traces through a CPU-safe transition predictor. |
| `elcp/dataset_freeze_manifest.py` | Hashes train/eval fixtures and candidate traces for review-only dataset freeze evidence. |
| `elcp/metrics_board.py` | Aggregates validation, baseline, replay, freeze and admission metrics. |
| `elcp/readiness_contract.py` | Writes the 4B readiness contract while keeping training blocked. |
| `models/README.md` | Checkpoint policy; generated model artifacts stay out of git. |
| `models/checkpoint_registry.json` | Empty public checkpoint registry. It preserves no-claim/no-production admission until evidence exists. |
| `rocm/rocm_env.sh` | Local ROCm environment probe; it reports availability without requiring a GPU. |
| `rocm/build_megatron_7b_evidence.py` | Stdlib-only builder/validator for the 7B Megatron pilot evidence contract. |
| `rocm/build_megatron_7b_inference_report.py` | Stdlib-only builder/validator for the 7B checkpoint inference probe contract. |
| `rocm/megatron_offline_smoke.sh` | Optional MI300X/Megatron smoke launcher. It uses `NullTokenizer`, mock data, random initialization and Docker `--network none`; it does not pull images or use external models. |
| `rocm/megatron_eden_corpus_pilot.sh` | Optional MI300X/Megatron pilot launcher. It trains a local SentencePiece tokenizer from `eden_core/corpus`, preprocesses EDEN-owned text into Megatron format and runs a tiny random-weight GPT pilot without network access or external models. |
| `rocm/megatron_eden_7b_base_pilot.sh` | Optional MI300X/Megatron 7B-shape launcher. It uses EDEN-owned corpus/tokenizer, random initialization, Docker `--network none`, no external models, formal evidence JSON and no checkpoint admission. |
| `rocm/megatron_eden_7b_inference_probe.sh` | Optional MI300X/Megatron inference probe. It loads the EDEN-owned 7B checkpoint and generates tokens locally through Megatron Core inference with Docker `--network none`. |
| `rocm/eden_sft_elcp_gpu_pilot.sh` | Optional MI300X learned-capability pilot. It trains a compact EDEN-owned SFT/ELCP transition module on GPU and writes pre/post eval, repeated inference packets and checkpoint-admission evidence. |
| `rocm/eden_real_capability_stage.sh` | Seven-part capability stage wrapper. It builds the real corpus, optionally runs the bounded 7B ROCm job, evaluates evidence and keeps checkpoint admission blocked. |
| `rocm/eden_v01_capability_stage.sh` | EDEN v0.1 stage wrapper. It builds the semantic corpus, optionally runs 7B training beyond the pilot, evaluates, writes demo evidence and records GPU hygiene. |
| `rocm/eden_v02_stability_stage.sh` | EDEN v0.2 stage wrapper. It can run 100-iteration baseline plus 250-iteration candidate jobs, compare evidence, write rollback/model-card artifacts and keep production release blocked. |
| `rocm/eden_v03_capability_stage.sh` | EDEN v0.3 stage wrapper. It can run a 1000-iteration 7B job, checkpoint inference, generalization eval, checkpoint registry, native runtime candidate, 14B scaling plan and demo trace. |
| `rocm/eden_70b_modular_stage.sh` | EDEN-70B modular launcher planner. It writes per-module ROCm/Megatron launch intent while keeping GPU training blocked unless separately approved. |
| `rocm/megatron_eden_70b_module_pilot.sh` | Optional per-module EDEN-70B ROCm launcher. On one MI300X it can pilot the 3B, 4B and 6B modules; it writes blocked evidence for the 12B, 12B and 33B modules until multi-GPU training is available. |
| `rocm/eden_gpu_workspace_hygiene.sh` | Non-destructive GPU workspace hygiene report for run roots, Docker image presence and cleanup policy. |

GEWC model runtime artifacts are generated from the Rust runtime, not from
Python training code:

| Artifact | Purpose |
| --- | --- |
| `model_adapter_runtime.json` | Registers model adapters as subordinate GEWC modules. |
| `model_checkpoint_manifest.json` | Records checkpoint admission policy; no weights are committed. |
| `training_harness_report.json` | Describes train/evaluate/compare/admit phases without running production training. |
| `model_governance_report.json` | Defines permissions, verification and circuit breakers for model adapters. |
| `eden_70b_modular_target.json` | Freezes the future 70B modular family under GEWC. It blocks single-checkpoint 70B training, monolithic LLM-brain authority and direct model writes. |
| `eden_70b_module_router.json` | Makes the 70B family routable by task, risk, modality, uncertainty, cost and permission. |
| `eden_70b_dataset_manifest.json` | Admits the module-specific seed splits as dataset contracts, not as sufficient training data. |
| `eden_70b_launcher_manifest.json` | Records per-module ROCm/Megatron launcher intent while keeping training blocked. |
| `eden_70b_module_training_pilot.json` | Contract for local per-module pilot evidence. This evidence stays under `target/` and does not admit checkpoints. |
| `eden_70b_module_training_blocked.json` | Contract for modules blocked by single-GPU memory limits. |
| `eden_70b_modular_training_summary.json` | Local summary for module pilots and blocked modules; it is evidence, not a release artifact. |
| `paradise_non_gpu_readiness_report.json` | Non-GPU product/runtime readiness report written by `make paradise-non-gpu-readiness`. |
| `eden_70b_checkpoint_admission.json` | Defines per-module checkpoint admission gates and keeps every module unadmitted until evidence exists. |
| `eden_70b_inference_runtime.json` | Defines the GEWC -> router -> module -> verifier inference contract while checkpoint loading remains false. |
| `eden_70b_operational_demo.json` | Records a non-mutating demo trace across memory, planner, CWM, safety and primary modules. |
| `eden_70b_operational_gate.json` | Aggregates the 70B modular operational surface and preserves no-claim/no-checkpoint-admission policy. |
| `first_model_card.json` | Defines the first EDEN model candidate and its authority boundary. |
| `first_model_training_plan.json` | Plans 4B training without submitting GPU work. |
| `first_model_readiness.json` | Confirms 4A preparation while keeping training blocked. |
| `elcp_objective_spec.json` | Defines Eden Latent Cognitive Prediction as a native state-transition objective. |
| `elcp_transition_dataset.json` | Defines the governed cognitive-transition data contract. |
| `elcp_training_plan.json` | Plans future ELCP 4B training without submitting GPU work. |
| `elcp_admission_gate.json` | Defines checkpoint admission requirements and blocks admission until 4B evidence exists. |
| `elcp_trace_quality_gate.json` | Admits trace-quality evidence into the runtime artifact API. |
| `elcp_replay_eval.json` | Admits replay evaluation evidence into the runtime artifact API. |
| `elcp_dataset_freeze_manifest.json` | Admits dataset-freeze evidence into the runtime artifact API. |
| `elcp_metrics_board.json` | Admits the aggregate metrics board into the runtime artifact API. |
| `elcp_4b_readiness_contract.json` | Admits the final pre-training 4B contract while blocking training. |
| `elcp_readiness.json` | Confirms ELCP 4A preparation while keeping training blocked. |
| `megatron_7b_model_adapter.json` | Exposes the 7B checkpoint as a GEWC-subordinate candidate generator. |
| `megatron_7b_inference_report.json` | Admits real checkpoint-load/token-generation evidence after the ROCm probe. |
| `megatron_7b_capability_report.json` | Marks only the usable probe capability, not semantic competence or AGI. |
| `megatron_7b_admission_gate.json` | Keeps checkpoint, production and autonomy admission blocked after the probe. |
| `eden_capable_training_run_contract.json` | Step 1: prepares the longer 7B training run contract without starting GPU work. |
| `eden_cognitive_dataset_manifest.json` | Step 2: admits the synthetic cognitive capability seed dataset. |
| `eden_native_inference_api.json` | Step 3: defines native GEWC structured inference request/response boundaries. |
| `eden_capability_delta_eval.json` | Step 4: compares architecture-only state against checkpoint-load/token-generation evidence. |
| `eden_structured_output_report.json` | Step 5: converts raw model text into untrusted EDEN hypothesis packets. |
| `eden_checkpoint_registry.json` | Step 6: registers the checkpoint as a probe, not a release. |
| `eden_sft_elcp_readiness.json` | Step 7: prepares SFT/ELCP readiness while keeping training blocked. |
| `eden_capable_gate.json` | Aggregates the seven EDEN-capable steps under no-claim policy. |
| `eden_live_inference_runtime.json` | Operational step 1: exposes the 7B probe as a callable subordinate runtime surface. |
| `eden_cognitive_call_contract.json` | Operational step 2: defines the GEWC -> router -> model -> verifier -> memory/action transaction. |
| `eden_cognitive_dataset_expansion.json` | Operational step 3: reports expanded synthetic cognitive coverage for future SFT/ELCP. |
| `eden_capability_eval_suite.json` | Operational step 4: turns seed records into a local capability contract eval suite. |
| `eden_sft_elcp_activation_gate.json` | Operational step 5: keeps future SFT/ELCP activation blocked until explicit GPU approval. |
| `eden_memory_action_loop.json` | Operational step 6: demonstrates model packets passing through memory/action gates. |
| `eden_capable_demo_trace.json` | Operational step 7: records a user-visible governed demo trace. |
| `eden_capable_operational_gate.json` | Aggregates the seven operational EDEN-capable steps under no-claim policy. |
| `eden_sft_elcp_dataset_v2_manifest.json` | Learned step 1: admits the deterministic SFT/ELCP v2 train/eval split. |
| `eden_sft_elcp_gpu_training_report.json` | Learned step 2: admits the GPU pilot training report without checkpoint admission. |
| `eden_sft_elcp_prepost_eval.json` | Learned step 3: records pre/post contract performance. |
| `eden_sft_elcp_repeated_inference_eval.json` | Learned step 4: records repeatable hypothesis packets from the trained module. |
| `eden_sft_elcp_checkpoint_admission_review.json` | Learned step 5: keeps checkpoint admission blocked after the pilot. |
| `eden_sft_elcp_operational_demo.json` | Learned step 6: demonstrates the learned packet flowing through GEWC boundaries. |
| `eden_external_tests_ci_gate.json` | Learned step 7: confirms external tests are explicit CI/manual gates, not ignored evidence. |
| `eden_learned_capability_gate.json` | Aggregates the seven learned-capability checks under no-claim policy. |
| `eden_real_capability_dataset_manifest.json` | Real step 1: admits the repo-owned capability corpus. |
| `eden_real_capability_7b_training.json` | Real step 2: admits bounded 7B training evidence. |
| `eden_real_capability_inference_bridge.json` | Real step 3: integrates 7B checkpoint inference and SFT/ELCP packets as hypotheses. |
| `eden_real_capability_operational_eval.json` | Real step 4: admits the operational eval report. |
| `eden_real_capability_checkpoint_decision.json` | Real step 5: marks the checkpoint reviewable but not admitted. |
| `eden_real_capability_demo.json` | Real step 6: records the governed operational demo. |
| `eden_real_capability_scaling_ladder.json` | Real step 7: defines the next scaling runs and comparison policy. |
| `eden_real_capability_gate.json` | Aggregates the seven real-capability checks under no-claim policy. |
| `eden_v01_dataset_manifest.json` | v0.1 step 1: admits the larger curated semantic corpus. |
| `eden_v01_semantic_eval.json` | v0.1 step 2: admits semantic eval evidence requiring `>=100` training iterations. |
| `eden_v01_training_beyond_pilot.json` | v0.1 step 3: confirms the 7B run moved beyond the 50-iteration pilot. |
| `eden_v01_native_inference_runtime.json` | v0.1 step 4: exposes checkpoint inference as a GEWC-subordinate runtime candidate. |
| `eden_v01_operational_demo.json` | v0.1 step 5: records a non-mutating operational demo trace. |
| `eden_v01_checkpoint_admission.json` | v0.1 step 6: can admit candidate runtime use while production release remains blocked. |
| `eden_v01_scaling_plan.json` | v0.1 step 7: keeps dense-model scaling capped at 14B. |
| `eden_v01_gpu_workspace_hygiene.json` | v0.1 step 8: records GPU workspace hygiene and non-destructive cleanup policy. |
| `eden_v01_capability_gate.json` | Aggregates the v0.1 semantic runtime candidate gate. |
| `eden_v02_stability_corpus_manifest.json` | v0.2 step 1: admits the stability train/eval/challenge corpus manifest. |
| `eden_v02_stability_eval.json` | v0.2 step 2: compares 100-iteration baseline and 250-iteration candidate evidence. |
| `eden_v02_checkpoint_comparison.json` | v0.2 step 3: records checkpoint loss, iteration, parameter and inference deltas. |
| `eden_v02_adversarial_eval.json` | v0.2 step 4: records deterministic adversarial architecture checks. |
| `eden_v02_rollback_drill.json` | v0.2 step 5: proves the candidate can be rejected and baseline fallback preserved. |
| `eden_v02_model_card_internal.json` | v0.2 step 6: records intended use, limits, gates and non-AGI claim boundaries. |
| `eden_v02_checkpoint_storage.json` | v0.2 step 7: records checkpoint retention policy; weights stay out of git and are purged from the GPU VM after evidence capture. |
| `eden_v02_native_inference_service.json` | v0.2 step 8: defines the permanent subordinate inference-service boundary. |
| `eden_v02_stability_demo.json` | v0.2 step 9: records the non-mutating operational stability demo. |
| `eden_v02_stability_gate.json` | Aggregates the v0.2 stability gate while blocking production release and AGI claims. |
| `eden_v03_generalization_corpus_manifest.json` | v0.3 step 1: admits the larger train/eval/challenge corpus manifest. |
| `eden_v03_generalization_eval.json` | v0.3 step 2: evaluates dataset coverage, inherited v0.2 safety, 1000-iteration evidence, inference load and non-regression. |
| `eden_v03_checkpoint_admission.json` | v0.3 step 3: admits the 1000-iteration checkpoint as candidate runtime only when all checks pass. |
| `eden_v03_live_inference_runtime.json` | v0.3 step 4: defines the persistent native inference-service contract for the admitted candidate. |
| `eden_v03_checkpoint_registry.json` | v0.3 step 5: registers candidate checkpoints without committing weights to git. |
| `eden_v03_scaling_14b_plan.json` | v0.3 step 6: opens the 14B prototype path only after v0.3 passes and keeps the dense ceiling at 14B. |
| `eden_v03_operational_demo.json` | v0.3 step 7: records the non-mutating operational demo trace. |
| `eden_v03_capability_gate.json` | Aggregates the v0.3 long-pretraining candidate gate while blocking production release and AGI claims. |

## Local Smoke Run

```bash
make training-smoke
```

This writes:

```text
target/eden_training_smoke/capability_report.json
target/eden_training_smoke/capability_report.md
```

The report is evidence for the training/evaluation pipeline only. It is not an
external validation result and does not permit stronger AGI claims.

To admit that report into the GARM/GEWC evidence surface:

```bash
make training-evidence
```

This writes a governed runtime artifact under `/tmp/eden_garm_training_evidence`
and preserves the same no-claim boundary.

To validate the model runtime and governance path without training or shipping
weights:

```bash
make model-runtime
```

To validate the public dataset/license and checkpoint-registry boundaries:

```bash
make contracts-validate
make training-dataset-license-manifest
make paradise-checkpoint-registry-smoke
```

To freeze the EDEN-70B modular target without starting training:

```bash
make eden-70b-modular-target
```

This emits `eden_70b_modular_target.json` as a governed runtime artifact. The
target is six models coordinated by GEWC: 33B primary, 12B causal world model,
12B multimodal/VLA, 6B planner-code-tool, 4B safety verifier and 3B
memory-router. It deliberately rejects a single dense 70B checkpoint and treats
the 7B path as historical pipeline evidence.

To generate the module seed splits, write the ROCm launcher plan and execute
the seven 70B modular runtime artifacts:

```bash
make eden-70b-operationalize
```

This is the operational bridge after the target freeze. It still does not train
or admit checkpoints. It proves that the 70B plan is a GEWC-routed runtime
surface with module datasets, launchers, admission gates, inference contract and
demo evidence.

To run one bounded module pilot on a ROCm host:

```bash
EDEN_70B_MODULE_ID=eden_memory_router_retrieval_3b \
make training-eden-70b-module-pilot
```

Supported module IDs:

```text
eden_memory_router_retrieval_3b
eden_safety_verifier_4b
eden_planner_code_tool_6b
eden_cwm_12b_causal_world_model
eden_multimodal_vla_12b
eden_33b_elcp_primary
```

On a single MI300X, the 3B, 4B and 6B modules can run as one-iteration pilots.
The 12B, 12B and 33B modules write `blocked_before_gpu_training` evidence by
default because they require multi-GPU tensor/pipeline parallelism or a smaller
proxy run. This target never trains one dense 70B checkpoint.

To prepare the first EDEN model candidate as a formal 4A artifact set:

```bash
make first-model-prepare
```

To validate ELCP fixtures, run the CPU baseline, export candidate traces,
dry-run the training interface and prepare the checkpoint admission gate:

```bash
make elcp-admission-gate
```

To harden ELCP evidence for operator review without starting training:

```bash
make elcp-hardening
```

To prepare ELCP as EDEN's native latent cognitive objective without training:

```bash
make elcp-prepare
```

To generate and validate the SFT/ELCP v2 learned-capability dataset:

```bash
make training-eden-sft-elcp-dataset
```

## Optional MI300X Megatron Smoke

On an AMD ROCm host with `rocm/megatron-lm:v25.3` already present locally:

```bash
make training-megatron-offline-smoke
```

This is an execution-path smoke only. It starts a tiny randomly initialized GPT
model, runs with Docker `--network none`, uses Megatron `NullTokenizer` and
mock data, and writes:

```text
target/eden_megatron_offline_smoke/offline_megatron_smoke.log
target/eden_megatron_offline_smoke/offline_megatron_smoke.summary
```

The script deliberately does not download checkpoints, tokenizers or datasets.
It also does not admit a production checkpoint. It proves that the EDEN training
surface can drive Megatron on ROCm without depending on an external model.

For the first EDEN-owned data pilot:

```bash
make training-megatron-eden-corpus-pilot
```

This builds all trainable inputs from repo-local EDEN corpus files:

```text
target/eden_megatron_corpus_pilot/data/eden_corpus.jsonl
target/eden_megatron_corpus_pilot/tokenizer/eden_sp.model
target/eden_megatron_corpus_pilot/data/eden_corpus_text_document.bin
target/eden_megatron_corpus_pilot/data/eden_corpus_text_document.idx
target/eden_megatron_corpus_pilot/eden_corpus_pilot.log
target/eden_megatron_corpus_pilot/eden_corpus_pilot.summary
```

The pilot starts from random weights and remains checkpoint-admission blocked.
Its purpose is to prove the EDEN-only data path before larger training.

For the first 7B-shape base-model path validation:

```bash
make training-megatron-eden-7b-base-pilot
```

This runs a large random-weight GPT-style model with:

```text
layers=32
hidden_size=4096
ffn_hidden_size=12288
attention_heads=32
sequence_length=128 by default
```

The run is intentionally tiny by default. It is useful as evidence that EDEN can
build and step through a 7B-scale base-model path on ROCm; it is not evidence of
AGI behavior, useful language competence or checkpoint admission.

The script writes:

```text
target/eden_megatron_7b_base_pilot/eden_7b_base_pilot.log
target/eden_megatron_7b_base_pilot/eden_7b_base_pilot.summary
target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json
```

To rebuild and admit the formal evidence into GEWC after a run:

```bash
make training-megatron-7b-evidence
```

On a GPU-only VM without the Rust toolchain, build just the portable JSON
evidence and copy it to a runtime host for GEWC admission:

```bash
make training-megatron-7b-evidence-json
```

The resulting artifact remains claim-gated:

```text
claim_allowed=false
agi_claim=false
checkpoint_admission=false
weights_admitted=false
```

After a checkpoint exists, run a real local inference probe on the ROCm host:

```bash
make training-megatron-7b-inference-probe
```

This writes:

```text
target/eden_megatron_7b_base_pilot/eden_7b_inference_probe.log
target/eden_megatron_7b_base_pilot/eden_7b_inference.summary
target/eden_megatron_7b_base_pilot/eden_7b_inference_response.json
target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json
```

To admit that probe as a governed EDEN cognitive capacity on a host with Rust:

```bash
make training-megatron-7b-adapter
```

The adapter is intentionally narrow: GEWC may use it as a subordinate candidate
token generator, but it still cannot write memory, change objectives, execute
tools, claim semantic competence, or admit its own checkpoint.

To prepare the seven no-GPU steps that make the checkpoint usable as an EDEN
capability surface:

```bash
make eden-capable
```

This does not start training. It writes the training-run contract, cognitive
dataset manifest, native inference API boundary, before/after capability eval,
structured output packets, checkpoint registry, SFT/ELCP readiness and aggregate
gate.

To generate only the operational "EDEN capable" runtime surface:

```bash
make eden-capable-operationalize
```

This also does not start training or use GPU. It writes the callable inference
runtime contract, cognitive call contract, expanded dataset report, capability
eval suite, SFT/ELCP activation gate, memory/action loop and demo trace.

To run the compact learned-capability pilot on a ROCm GPU:

```bash
make training-eden-sft-elcp-gpu-pilot
```

To admit the pilot evidence through GEWC after the GPU run:

```bash
make eden-learned-capability
```

The learned-capability gate remains narrow: it requires dataset, GPU training,
pre/post eval, repeated inference packets, blocked checkpoint admission,
operational demo evidence and the explicit external-tests CI gate. It does not
permit production model release, direct memory writes, tool authority or AGI
claims.

To build the larger repo-owned capability corpus and evaluate the current 7B
and SFT/ELCP evidence:

```bash
make training-eden-real-capability-stage
make eden-real-capability
```

By default this consumes existing 7B evidence. To launch the bounded ROCm 7B job
first, set:

```bash
EDEN_REAL_CAPABILITY_RUN_GPU=true make training-eden-real-capability-stage
```

To move from the 50-iteration pilot to the EDEN v0.1 semantic runtime candidate:

```bash
make training-eden-v01-stage
make eden-v01-capability
```

By default this consumes existing evidence and will fail until the 7B evidence
has at least 100 completed iterations. To run the bounded ROCm continuation:

```bash
EDEN_V01_RUN_GPU=true make training-eden-v01-stage
```

To compare a 100-iteration baseline against a 250-iteration 7B candidate and
write the v0.2 stability artifacts:

```bash
make training-eden-v02-stage
make eden-v02-stability
```

By default this consumes existing evidence and emits a failed gate if the GPU
evidence is absent. To launch the bounded ROCm baseline and candidate jobs:

```bash
EDEN_V02_RUN_GPU=true make training-eden-v02-stage
```

For disposable GPU VMs, purge local checkpoint directories after portable
evidence is written:

```bash
EDEN_V02_RUN_GPU=true \
EDEN_V02_PURGE_LOCAL_CHECKPOINTS=true \
make training-eden-v02-stage
```

The v0.2 gate can allow candidate runtime admission only when the stability
comparison, checkpoint inference, adversarial eval, rollback drill, model card,
storage policy, native inference boundary and demo all pass. It still keeps
production release and AGI claims blocked.

To move from the 250-iteration candidate to the v0.3 long-pretraining
candidate path:

```bash
make training-eden-v03-stage
make eden-v03-capability
```

By default this writes a valid blocked report if the 1000-iteration GPU
evidence is absent. To launch the ROCm long run:

```bash
EDEN_V03_RUN_GPU=true make training-eden-v03-stage
```

The v0.3 gate can allow candidate runtime admission only when the larger corpus,
1000-iteration 7B checkpoint evidence, checkpoint-load inference, v0.2 safety
chain, checkpoint registry, native runtime contract, 14B scaling policy and
demo all pass. It still keeps production release, autonomous authority and AGI
claims blocked.

To move from v0.3 into the seven v0.4 GPU-backed capability processes:

```bash
make training-eden-v04-stage
make eden-v04-capability
```

By default this writes a blocked report if the 10k 7B evidence is absent. To
launch or reuse the ROCm 10k run, checkpoint inference probe and compact
cognitive SFT/ELCP pilot:

```bash
EDEN_V04_RUN_GPU=true make training-eden-v04-stage
```

The v0.4 gate can allow candidate runtime admission only when the 8192-row
cognitive capability corpus, 10k 7B checkpoint evidence, generative probe,
cognitive SFT evidence, hard checkpoint admission, persistent inference service
contract, continuity eval and 14B preflight pass. It still blocks production
release, autonomous authority and AGI claims.

To validate all non-GPU product/runtime readiness work while GPU training is
paused:

```bash
make paradise-non-gpu-readiness
```

This writes `target/paradise_non_gpu_readiness/non_gpu_readiness_report.json`
and checks product docs, model-interface authority, dataset governance,
evaluation/admission policy, external-public boundaries, hardware-test
isolation and known technical debt. It does not start GPU work and does not
admit checkpoints.

For a longer controlled pilot that writes a checkpoint but keeps admission
blocked:

```bash
EDEN_MEGATRON_7B_TRAIN_ITERS=50 \
EDEN_MEGATRON_7B_SAVE_CHECKPOINT=true \
EDEN_MEGATRON_7B_SAVE_INTERVAL=50 \
make training-megatron-eden-7b-base-pilot
```

## ROCm Direction

The first GPU-backed work should stay narrow:

1. train or fit memory/retrieval modules;
2. add small language-model experiments;
3. add world-model proxy tasks;
4. route every model through GEWC as a subordinate module;
5. compare releases by reproducible capability reports.

No model should gain direct authority over memory writes, objective updates or
tool execution. GEWC remains the executive runtime owner.
