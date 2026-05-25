# ADR-040: GARM Capability Reality Evaluation

## Status

Accepted

## Context

EDEN now has GEWC runtime routing, architecture artifacts, local validation harnesses and release-candidate packaging. That is not the same thing as a trained LMM, independent AGI validation, physical embodiment, open-ended autonomous learning or unrestricted external action.

The validation path needs a first-class report that answers a narrower question: what can the current system actually execute today, what is only represented as architecture, what is simulated or stubbed, what depends on LMM training, what requires independent validation, and what is intentionally blocked by safety policy.

## Decision

Add `capability reality eval` as a GEWC-routed validation command. It writes:

- `capability_reality_eval.json`
- `capability_reality_matrix.json`
- `lmm_training_dependency_report.json`

The report classifies capabilities into:

- `implemented_runtime`
- `implemented_local_heuristic`
- `implemented_architecture_artifact`
- `simulated_or_stub`
- `requires_lmm_training`
- `requires_external_validation`
- `blocked_by_safety_policy`

The command remains no-claim by construction: `claim_allowed=false`, `agi_claim=false`, `weights_present=false` and `training_executed=false`.

## Consequences

- Release-candidate validation now includes the reality matrix before external handoff and capability registry audit.
- The external local harness includes a held-out case for the capability reality matrix.
- The package validator treats the three new artifacts as required.
- EDEN can report current operational capacity honestly without implying that architecture artifacts are trained AGI capability.
