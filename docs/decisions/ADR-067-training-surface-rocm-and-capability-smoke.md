# ADR-067: Training Surface, ROCm Profile and Capability Smoke Gate

## Status
Accepted

## Date
2026-05-24

## Context
Paradise has a stable local runtime, GEWC/GARM operational surface and
validation gates, but it does not yet ship a trained production LLM/LMM.
Future AMD GPU work needs a reproducible place to live before heavy training
starts.

The project needs to avoid two failure modes:

- treating architecture artifacts as trained capability;
- adding GPU scripts or notebooks that bypass GEWC authority, claim gates or CI.

## Decision
Create `training/` as the prepared public training and evaluation surface.

The initial surface includes:

- `training/configs/rocm_smoke.json` for AMD ROCm execution policy with CPU
  fallback;
- `training/data/capability_smoke.jsonl` for deterministic capability smoke
  cases;
- `training/benchmarks/eden_capability_benchmark.py` as a stdlib-only runner
  with a tiny trainable memory baseline;
- `training/rocm/rocm_env.sh` as a non-failing ROCm probe;
- `make training-smoke`, `make training-rocm-profile`, `make doctest`,
  `make workspace-test` and optional `make external-tests`;
- CI jobs for full workspace tests, doctests and training smoke evidence.

External GPIO, I2C and network checks remain available but optional through the
`external-tests` feature and manual workflow dispatch.

## Alternatives Considered

### Start With Heavy LLM Training

- Pros: More directly aligned with future model work.
- Cons: Requires GPU availability, data policy, checkpoints, training budget and
  evaluation discipline before the repo has a public trainable path.
- Rejected: Too much risk of unreproducible artifacts and overstated claims.

### Keep Training Out Of The Repo

- Pros: Keeps the public repo smaller.
- Cons: Splits architecture, runtime and training evidence; future contributors
  cannot see how models are supposed to integrate.
- Rejected: EDEN needs a public, claim-gated path from runtime to measurable
  capability.

### Run Hardware/Network Tests In Default CI

- Pros: Catches external integration failures.
- Cons: Default CI and local sandboxes lack GPIO/I2C/network guarantees.
- Rejected: These tests are valuable but must remain opt-in.

## Consequences

- EDEN now has a concrete path for AMD GPU experiments without pretending that
  trained capability already exists.
- CI validates the training/evaluation path with CPU-safe smoke evidence.
- The report format can become the seed for future ROCm-backed benchmark
  artifacts.
- GEWC remains the authority boundary for future trainable modules.
- External tests can be run manually when real hardware and network access are
  available.
