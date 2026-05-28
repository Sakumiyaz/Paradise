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
- `make eden-70b-modular-target` writes the future EDEN-70B modular target as
  a GEWC-governed artifact. This is six specialized models totaling 70B, not a
  single dense 70B checkpoint or monolithic LLM brain.
- `make eden-70b-operationalize` writes the seven executable follow-up
  artifacts for that target: router, dataset manifest, launcher manifest,
  checkpoint admission, inference runtime, operational demo and aggregate gate.
- `make training-eden-70b-module-pilot` is the bounded ROCm launcher for one
  module in that family. On a single MI300X it can execute the 3B memory-router,
  4B safety-verifier and 6B planner/tool pilots; it records formal blocked
  evidence for the 12B, 12B and 33B modules until multi-GPU training exists.
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
- `make training-megatron-7b-inference-probe` loads the EDEN-owned 7B
  checkpoint through Megatron Core inference and writes a local token-generation
  report with Docker `--network none`.
- `make training-megatron-7b-adapter` admits the checkpoint-load/token-generation
  evidence into GEWC as a subordinate cognitive capacity while still blocking
  semantic competence, production inference, checkpoint admission and autonomy.
- `make eden-capable` prepares the seven remaining no-GPU capability steps:
  longer-training contract, cognitive dataset manifest, native structured
  inference boundary, capability delta eval, structured output packets,
  checkpoint registry and SFT/ELCP readiness.
- `make eden-capable-operationalize` prepares the seven no-GPU operational
  steps: callable probe-backed inference runtime, GEWC cognitive call contract,
  expanded cognitive dataset report, capability eval suite, SFT/ELCP activation
  gate, memory/action loop and governed demo trace.
- `make training-eden-sft-elcp-dataset` generates the deterministic SFT/ELCP v2
  train/eval split and validates it against the ELCP transition contract.
- `make training-eden-sft-elcp-gpu-pilot` runs the compact learned-capability
  pilot on a ROCm GPU with Docker `--network none`. It writes pre/post eval,
  repeated hypothesis packets and checkpoint-admission review evidence.
- `make eden-learned-capability` admits that evidence into GEWC as a governed
  learned-capability surface while preserving `claim_allowed=false`,
  `agi_claim=false` and `checkpoint_admission_allowed=false`.
- `make training-eden-real-capability-stage` builds the repo-owned capability
  corpus, optionally runs the bounded 7B ROCm job, evaluates real evidence and
  keeps checkpoint admission blocked.
- `make eden-real-capability` admits the seven real-capability artifacts into
  GEWC: dataset, 7B training, inference bridge, operational eval, checkpoint
  decision, demo and scaling ladder.
- `make training-eden-v01-stage` builds the larger semantic corpus and requires
  7B evidence beyond the 50-iteration pilot before semantic runtime candidate
  admission can pass. With `EDEN_V01_RUN_GPU=true`, it runs the bounded 7B
  continuation, checkpoint inference, semantic eval, operational demo and GPU
  hygiene report.
- `make eden-v01-capability` admits the v0.1 artifacts into GEWC: dataset,
  semantic eval, training beyond pilot, native inference runtime, operational
  demo, checkpoint candidate admission, scaling plan, GPU hygiene and gate.
- `make training-eden-v02-stage` compares a 100-iteration baseline against a
  250-iteration candidate, runs adversarial and rollback checks, writes an
  internal model card and preserves the no-external-model boundary.
- `make eden-v02-stability` admits the v0.2 artifacts into GEWC: stability
  corpus, stability eval, checkpoint comparison, adversarial eval, rollback
  drill, internal model card, checkpoint-storage policy, native inference
  service, demo and gate.

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
- `eden_70b_modular_target.json` freezes the future modular model family:
  33B primary, 12B causal world model, 12B multimodal/VLA, 6B
  planner-code-tool, 4B safety verifier and 3B memory-router. It keeps
  `not_a_single_model=true`, `single_checkpoint_training_allowed=false` and
  `claim_allowed=false`.
- `/tmp/eden_garm_70b_modular/eden_70b_operational_gate.json` proves the target
  is now a runtime surface rather than a document-only plan. It keeps
  `training_allowed=false`, `checkpoint_admission_allowed=false` and
  `agi_claim=false`.
- `target/eden_70b_modular_training_evidence/eden_70b_modular_training_summary.json`
  records the first single-MI300X module pilots. It is local evidence only:
  three modules were started as one-iteration pilots, while the 12B/12B/33B
  modules remain blocked until distributed ROCm topology is available.
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
- `target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json` proves only
  that the checkpoint can be loaded and can generate candidate tokens. It is a
  runtime usability signal, not evidence of AGI, semantic reliability or
  release readiness.
- `/tmp/eden_garm_capable/eden_capable_gate.json` aggregates the seven
  EDEN-capable preparation artifacts while preserving `claim_allowed=false` and
  `agi_claim=false`.
- `/tmp/eden_garm_capable_operational/eden_capable_operational_gate.json`
  aggregates the seven operational artifacts and keeps the checkpoint as a
  subordinate hypothesis generator, not a production model.
- `/tmp/eden_garm_learned_capability/eden_learned_capability_gate.json`
  aggregates the seven learned-capability checks. It only passes when the
  SFT/ELCP v2 dataset exists, GPU training evidence exists, pre/post eval
  improves, repeated inference packets stay hypothesis-gated, checkpoint
  admission stays blocked, the demo has a packet and the external-tests CI gate
  exists.
- `/tmp/eden_garm_real_capability/eden_real_capability_gate.json` aggregates
  the next seven checks: real repo-owned corpus, bounded 7B training,
  checkpoint-load inference, operational eval, reviewable-but-blocked checkpoint
  decision, governed demo and scaling ladder.
- `/tmp/eden_garm_v01_capability/eden_v01_capability_gate.json` aggregates the
  semantic runtime candidate checks: larger curated dataset, semantic eval,
  `>=100` 7B iterations, native inference, candidate checkpoint admission,
  demo trace, 14B scaling cap and non-destructive GPU hygiene.
- `/tmp/eden_garm_v02_stability/eden_v02_stability_gate.json` aggregates the
  stability candidate checks: 250-iteration candidate evidence, baseline
  comparison, checkpoint-load inference, adversarial eval, rollback drill,
  internal model card, checkpoint-storage policy, native inference service and
  demo trace. Candidate runtime admission may pass, but production release and
  AGI claims remain blocked.

## Validation Commands

```bash
make training-rocm-profile
make training-megatron-offline-smoke
make training-megatron-eden-corpus-pilot
make training-megatron-eden-7b-base-pilot
make training-megatron-7b-evidence-json
make training-megatron-7b-evidence
make training-megatron-7b-inference-probe
make training-megatron-7b-adapter
make eden-70b-modular-target
make eden-70b-operationalize
make training-eden-70b-module-pilot
make eden-capable
make eden-capable-operationalize
make training-eden-sft-elcp-dataset
make training-eden-sft-elcp-gpu-pilot
make eden-learned-capability
make training-eden-real-capability-stage
make eden-real-capability
make training-eden-v01-stage
make eden-v01-capability
make training-eden-v02-stage
make eden-v02-stability
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
