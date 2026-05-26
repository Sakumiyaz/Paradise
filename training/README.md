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
| `data/manifest.json` | Explicit dataset manifest; synthetic repo-local data only. |
| `data/capability_smoke.jsonl` | Tiny deterministic benchmark dataset for current operational capabilities. |
| `data/first_model_memory_*.jsonl` | Tiny train/eval fixtures for the first memory-retrieval baseline. |
| `data/elcp_transition_*.jsonl` | Tiny train/eval fixtures for ELCP cognitive-transition contracts. |
| `data/eden_cognitive_sft_elcp_*.jsonl` | Deterministic SFT/ELCP v2 train/eval data for the learned-capability pilot. |
| `data/build_eden_cognitive_sft_elcp.py` | Stdlib-only SFT/ELCP v2 generator; no private data and no external model. |
| `benchmarks/eden_capability_benchmark.py` | Stdlib-only benchmark runner and tiny trainable memory baseline. |
| `benchmarks/validate_capability_report.py` | Stdlib-only contract validator for `capability_report.json`. |
| `benchmarks/validate_elcp_transitions.py` | Stdlib-only validator for ELCP cognitive-transition fixtures. |
| `benchmarks/elcp_baseline_eval.py` | CPU-safe rule baseline for ELCP target fields. |
| `elcp/export_trace_candidates.py` | Redacted GEWC trace exporter for candidate ELCP transitions. |
| `elcp/train_elcp.py` | Dry-run-only training interface for future ELCP 4B work. |
| `elcp/admission_gate.py` | Pre-checkpoint ELCP admission policy report. |
| `elcp/trace_quality_gate.py` | Reviews exported traces for shape, duplication, authority and sensitive markers. |
| `elcp/replay_eval.py` | Replays reviewed traces through a CPU-safe transition predictor. |
| `elcp/dataset_freeze_manifest.py` | Hashes train/eval fixtures and candidate traces for review-only dataset freeze evidence. |
| `elcp/metrics_board.py` | Aggregates validation, baseline, replay, freeze and admission metrics. |
| `elcp/readiness_contract.py` | Writes the 4B readiness contract while keeping training blocked. |
| `models/README.md` | Checkpoint policy; generated model artifacts stay out of git. |
| `rocm/rocm_env.sh` | Local ROCm environment probe; it reports availability without requiring a GPU. |
| `rocm/build_megatron_7b_evidence.py` | Stdlib-only builder/validator for the 7B Megatron pilot evidence contract. |
| `rocm/build_megatron_7b_inference_report.py` | Stdlib-only builder/validator for the 7B checkpoint inference probe contract. |
| `rocm/megatron_offline_smoke.sh` | Optional MI300X/Megatron smoke launcher. It uses `NullTokenizer`, mock data, random initialization and Docker `--network none`; it does not pull images or use external models. |
| `rocm/megatron_eden_corpus_pilot.sh` | Optional MI300X/Megatron pilot launcher. It trains a local SentencePiece tokenizer from `eden_core/corpus`, preprocesses EDEN-owned text into Megatron format and runs a tiny random-weight GPT pilot without network access or external models. |
| `rocm/megatron_eden_7b_base_pilot.sh` | Optional MI300X/Megatron 7B-shape launcher. It uses EDEN-owned corpus/tokenizer, random initialization, Docker `--network none`, no external models, formal evidence JSON and no checkpoint admission. |
| `rocm/megatron_eden_7b_inference_probe.sh` | Optional MI300X/Megatron inference probe. It loads the EDEN-owned 7B checkpoint and generates tokens locally through Megatron Core inference with Docker `--network none`. |
| `rocm/eden_sft_elcp_gpu_pilot.sh` | Optional MI300X learned-capability pilot. It trains a compact EDEN-owned SFT/ELCP transition module on GPU and writes pre/post eval, repeated inference packets and checkpoint-admission evidence. |

GEWC model runtime artifacts are generated from the Rust runtime, not from
Python training code:

| Artifact | Purpose |
| --- | --- |
| `model_adapter_runtime.json` | Registers model adapters as subordinate GEWC modules. |
| `model_checkpoint_manifest.json` | Records checkpoint admission policy; no weights are committed. |
| `training_harness_report.json` | Describes train/evaluate/compare/admit phases without running production training. |
| `model_governance_report.json` | Defines permissions, verification and circuit breakers for model adapters. |
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
