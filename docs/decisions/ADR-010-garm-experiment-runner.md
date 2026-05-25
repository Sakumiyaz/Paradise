# ADR-010: GARM Local Experiment Runner

## Status

Accepted.

## Context

GARM now tracks goals, evaluation, learning, world observations, benchmarks, execution, attention and uncertainty. The remaining gap is a local experiment seam that turns architecture hypotheses into auditable trials using existing GARM evidence without relying on CI or external services.

## Decision

Add a GARM-native experiment runner with commands `experiment`, `experiment plan TEXT`, `experiment run` and `experiment audit`. Experiments are persisted in `experiment_runner.json`. `experiment run` scores local evidence from evaluation, benchmark, learning, uncertainty, executor and attention reports, then records completed or inconclusive outcomes. Runs write learning ledger traces and are included in reports, exports and artifact listings.

## Consequences

- Architecture hypotheses become reproducible local records.
- Experiments stay bounded and deterministic: no shell execution, network, LLM, CI or separate runtime.
- Evaluation and benchmarks can use experiment state as evidence of empirical calibration.
- Inconclusive outcomes preserve uncertainty instead of being hidden as success.
