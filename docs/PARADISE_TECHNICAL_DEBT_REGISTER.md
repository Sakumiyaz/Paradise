# Paradise Technical Debt Register

This register keeps known non-GPU gaps explicit so they do not get confused
with model-training blockers.

## Current Items

| Area | Status | Follow-up |
| --- | --- | --- |
| Benchmark local tick harness | `eden_core/src/benchmark/mod.rs` now measures runtime state and release artifact catalog hot paths behind the `benchmark` feature. | Add criterion-grade latency baselines once release performance budgets are defined. |
| `escoria_total` runtime aggregate | Runtime broadcasts a stable zero until Mar exposes a stable aggregate. | Wire the field once the Mar state exposes a stable aggregate. |
| Homeostasis real values | Monitor uses deterministic local vitals until MemBrain exposes a stable API. | Read from the stable MemBrain interface once that API is finalized. |
| Hardware tests | Isolated from default tests. | Run `make external-tests` only on hosts with GPIO/I2C/network access. |
| GPU model checkpoints | Paused. | Resume on 8x MI300X or equivalent. |

## Policy

- Do not delete placeholder code only to reduce comment counts.
- Convert broad placeholders into gated tests, docs or issue-sized follow-ups.
- Keep hardware/GPU/network blockers separate from local runtime blockers.
