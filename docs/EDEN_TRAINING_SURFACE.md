# EDEN Training Surface

EDEN's training surface is currently a prepared evaluation and experiment path,
not a trained production model release.

The repository now contains a CPU-safe capability smoke benchmark, a first
subordinate trainable-memory contract, a report schema, and an AMD ROCm profile
so future GPU work can attach to a reproducible path instead of starting from
ad hoc notebooks or private scripts.

## Current Guarantees

- `make training-smoke` runs without GPU hardware.
- `capability_report.json` is validated against
  `contracts/v1/schemas/eden-training-capability-report-v1.json`.
- `capability_report.md` gives a human-readable evidence summary for CI and
  release review.
- `training evidence eval` admits the report as GEWC-governed runtime evidence
  only after claim and authority boundaries pass.
- Reports preserve `claim_allowed=false` and `agi_claim=false`.
- The first trainable baseline is a tiny lexical memory model inside the smoke
  benchmark, used only to verify the evaluation path.
- `model runtime eval` writes a governed model adapter runtime, checkpoint
  manifest, training harness report and model governance report without
  training or admitting weights.
- `first model prepare` writes the 4A first-model card, training plan and
  readiness gate while keeping 4B training blocked.
- `elcp prepare` writes the 4A Eden Latent Cognitive Prediction objective,
  transition-dataset contract, training plan, hardening artifacts and readiness
  gate while keeping 4B training blocked.
- `make elcp-admission-gate` validates ELCP transitions, runs a CPU baseline,
  exports candidate GEWC traces, dry-runs the future training interface and
  prepares the checkpoint admission policy without admitting a checkpoint.
- `make elcp-hardening` adds trace-quality review, replay evaluation, dataset
  freeze manifest, metrics board and 4B readiness contract without training.
- GEWC remains the runtime authority. Models are subordinate modules.
- No script writes model weights, objectives, memory or tool actions directly.
- `make training-megatron-offline-smoke` is available as an optional ROCm
  hardware smoke. It requires a local Megatron ROCm Docker image, starts the
  container with `--network none`, uses `NullTokenizer` plus mock data, and does
  not consume external model weights, tokenizers, APIs or datasets.
- `make training-megatron-eden-corpus-pilot` is available as the first
  EDEN-owned corpus pilot. It trains a local SentencePiece tokenizer from
  `eden_core/corpus`, preprocesses repo-local text into Megatron indexed data
  and runs a tiny random-weight GPT pilot without network access.
- `make training-megatron-eden-7b-base-pilot` is available as the first
  7B-shape base-model path validation. It uses EDEN-owned corpus/tokenizer,
  random initialization and no external network, but remains evidence of
  training plumbing only, not AGI capability.
- `make training-megatron-7b-evidence` converts the 7B Megatron run output into
  `eden.megatron.7b.training_evidence.v1` and admits it through GEWC while
  preserving `claim_allowed=false`, `agi_claim=false` and
  `checkpoint_admission=false`.
- `make training-megatron-7b-evidence-json` is available for GPU-only hosts
  that can generate portable evidence but do not have the Rust runtime toolchain
  installed.

## Future AMD GPU Use

The first GPU-backed experiments should be narrow and measurable:

1. memory/retrieval modules;
2. small language-model experiments;
3. world-model proxy prediction tasks;
4. safety and permission classifiers;
5. multimodal adapters after the text/memory path is reproducible.

Each experiment should write an explicit report under `target/` or a future
versioned artifact path. Generated checkpoints should not be committed unless a
future public release process explicitly defines where they belong.

The current model runtime path makes that boundary executable:

- `model register/load/evaluate/unload <id>` records lifecycle events under
  GEWC authority.
- `model_checkpoint_manifest.json` keeps `weights_present=false` and
  `training_executed=false` until real training is explicitly requested.
- `model_governance_report.json` blocks direct memory writes, objective writes
  and tool execution by model adapters.
- `first_model_card.json`, `first_model_training_plan.json` and
  `first_model_readiness.json` define the first EDEN model candidate and prove
  preparation without submitting a GPU job.
- `elcp_objective_spec.json`, `elcp_transition_dataset.json`,
  `elcp_training_plan.json` and `elcp_readiness.json` define the native EDEN
  latent cognitive prediction objective without submitting a GPU job.
- `target/eden_elcp/validation_report.json`,
  `target/eden_elcp/baseline_report.json`,
  `target/eden_elcp/trace_export_report.json`,
  `target/eden_elcp/training_dry_run.json` and
  `target/eden_elcp/admission_gate_report.json` are local 4B-prep evidence
  only; they do not permit training or checkpoint admission.
- `target/eden_elcp/trace_quality_gate_report.json`,
  `target/eden_elcp/replay_eval_report.json`,
  `target/eden_elcp/dataset_freeze_manifest.json`,
  `target/eden_elcp/metrics_board.json` and
  `target/eden_elcp/4b_readiness_contract.json` harden that evidence for
  operator review while still keeping `4b_training_allowed=false`.

## Validation Commands

```bash
make training-rocm-profile
make training-megatron-offline-smoke
make training-megatron-eden-corpus-pilot
make training-megatron-eden-7b-base-pilot
make training-megatron-7b-evidence-json
make training-megatron-7b-evidence
make training-smoke
make training-evidence
make model-runtime
make first-model-prepare
make elcp-admission-gate
make elcp-hardening
make elcp-prepare
make doctest
make workspace-test
```

External hardware/network checks are intentionally separate:

```bash
make external-tests
```

That command requires real GPIO, I2C and network access.
