# ADR-028: Formalize the GARM Paradigm Architecture Map

## Status
Accepted

## Date
2026-05-22

## Context
EDEN/GARM already has formal artifacts for several AGI architecture families: cognitive, embodied, neural, symbolic, world-model, bounded self-improvement, safety-control and frontier layers. A separate list of AGI paradigms overlaps with those layers, so implementing every paradigm as a standalone artifact would create shallow duplicates.

The remaining gap is a formal map that distinguishes existing coverage, replacement decisions and true missing paradigm layers. The legacy `ParadigmHub` also kept 43 mixed items, but those items combine macro-paradigms, ML techniques, helper models and experimental controls. Keeping it as a separate paradigm authority would duplicate the formal map.

## Decision
Add `paradigm architecture eval` as the single paradigm authority.

The command writes `paradigm_architecture_map.json` for the 24 paradigms, absorbs the 43 legacy `ParadigmHub` items into `paradigm_architecture_technique_map.json`, and creates new no-claim artifacts only for non-duplicate gaps:

- `neuro_symbolic_paradigm.json`
- `universal_formal_paradigm.json`
- `active_inference_paradigm.json`
- `ecological_systemic_paradigm.json`
- `computational_programmatic_paradigm.json`
- `affective_motivational_paradigm.json`
- `human_in_the_loop_paradigm.json`
- `emergence_metrics_paradigm.json`

Covered paradigms remain mapped to their existing architecture layers. Generic reinforcement learning is replaced by safe hierarchical/model-based interpretation. Emergentism is treated as an evaluation metric layer, not as an autonomous subsystem. Hybrid remains the meta-architecture, not a module. None of the 43 legacy items is promoted to a new formal paradigm; each is classified as `subtype`, `alias`, `implementation_detail`, `archived`, `future`, `replaced` or `formalized` under the 24-paradigm authority.

The legacy `ParadigmHubNode` is retained only for state compatibility and historical snapshots. It is superseded by `paradigm architecture eval`; autonomous budget cycling is no longer the goal.

## Consequences
EDEN can discuss AGI paradigms without duplicating architecture artifacts. The readiness package, capability registry and independent validator now include the paradigm map, the absorbed technique map and gap layers while preserving `claim_allowed=false` and `agi_claim=false`.
