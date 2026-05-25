# ADR-072: ELCP 4B-Prep Validation Surface

## Status
Accepted

## Date
2026-05-25

## Context
ADR-071 defined Eden Latent Cognitive Prediction (ELCP) as a native objective
for future model training, but it did not yet make the objective measurable.
Before any real 4B training can be requested, EDEN needs local evidence that
the objective data shape, baseline behavior, runtime trace export, training
interface and checkpoint admission policy are executable.

The repository must still avoid starting GPU jobs, writing weights or implying
trained AGI capability.

## Decision
Add an ELCP 4B-prep surface made of five local gates:

1. `elcp-validate`: validates cognitive-transition JSONL fixtures.
2. `elcp-baseline`: runs a CPU-safe rule baseline over ELCP target fields.
3. `elcp-trace-export`: exports redacted GEWC traces as candidate transitions.
4. `elcp-training-dry-run`: prepares the future training interface with
   `--dry-run` required.
5. `elcp-admission-gate`: writes a pre-checkpoint admission policy report.

The native runtime also writes `elcp_admission_gate.json`, preserving:

- `training_executed=false`
- `weights_present=false`
- `gpu_job_submitted=false`
- `checkpoint_admission_allowed=false`
- `4b_training_allowed=false`

## Consequences
- ELCP is now measurable before training.
- Future 4B work has a clear contract for data validation, baseline comparison,
  trace review, dry-run scheduling and checkpoint admission.
- No training path is implicitly enabled by these gates.

## Alternatives Considered

### Start 4B training after ADR-071
- Pros: Produces model results sooner.
- Cons: Skips data validation, baseline measurement and admission policy.
- Rejected: Premature and unsafe.

### Keep ELCP as documentation only
- Pros: Simpler.
- Cons: Does not make the objective executable or testable.
- Rejected: EDEN architecture progress should produce local evidence.

### Put all ELCP logic only in Rust
- Pros: Single implementation language.
- Cons: Training and data-prep workflows are more naturally scripted, while
  Rust should keep runtime authority and artifact admission.
- Rejected: Use Python for local training-surface tools and Rust for GEWC
  artifact governance.
