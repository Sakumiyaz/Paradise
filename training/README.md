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
| `rocm/megatron_offline_smoke.sh` | Optional MI300X/Megatron smoke launcher. It uses `NullTokenizer`, mock data, random initialization and Docker `--network none`; it does not pull images or use external models. |
| `rocm/megatron_eden_corpus_pilot.sh` | Optional MI300X/Megatron pilot launcher. It trains a local SentencePiece tokenizer from `eden_core/corpus`, preprocesses EDEN-owned text into Megatron format and runs a tiny random-weight GPT pilot without network access or external models. |
| `rocm/megatron_eden_7b_base_pilot.sh` | Optional MI300X/Megatron 7B-shape launcher. It uses EDEN-owned corpus/tokenizer, random initialization, Docker `--network none`, no external models and no checkpoint admission. |

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

## ROCm Direction

The first GPU-backed work should stay narrow:

1. train or fit memory/retrieval modules;
2. add small language-model experiments;
3. add world-model proxy tasks;
4. route every model through GEWC as a subordinate module;
5. compare releases by reproducible capability reports.

No model should gain direct authority over memory writes, objective updates or
tool execution. GEWC remains the executive runtime owner.
