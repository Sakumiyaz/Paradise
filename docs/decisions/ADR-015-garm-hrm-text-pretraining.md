# ADR-015: GARM HRM-Text Pretraining Seam

## Status
Accepted

## Date
2026-05-21

## Context
GARM already has a deterministic HRM runtime, local evidence ledgers, and a hybrid voice architecture seam. The next useful step is to represent how text pretraining would feed HRM runtime priors without claiming that neural training or weights exist on the current CPU-only local runtime.

Key constraints:

- GARM remains the single runtime and entry point.
- No local or external LLM calls.
- No invented neural weights or hidden training execution.
- Rust-only local operational artifacts.
- Evidence must connect to learning, provenance, policy, maturity, HRM runtime and hybrid voice.

## Decision
Add a GARM-native `hrm_text_pretraining` seam with commands:

- `hrm text`
- `hrm text corpus PATH`
- `hrm text ingest DIR`
- `hrm text search QUERY`
- `hrm text objective TEXT`
- `hrm text plan`
- `hrm text run`
- `hrm text audit`

The seam persists `hrm_text_pretraining.json`, writes `hrm_text_corpus_manifest.txt` for local corpus metadata, writes bounded local text segments to `hrm_text_segments.jsonl`, and writes `hrm_text_checkpoint_manifest.txt` as a local checkpoint manifest. The checkpoint manifest explicitly records `weights_present=false` and `training_executed=false`.

`hrm text run` evaluates local evidence from HRM runtime, hybrid voice, learning, provenance and policy, then records follow-on entries in learning, provenance, policy and maturity ledgers. It also feeds the KG with a local relation describing HRM-text as a prior-manifest seam for HRM runtime.

`hrm text search QUERY` performs deterministic lexical retrieval over `hrm_text_segments.jsonl`. `hrm QUERY` and `hrm run QUERY` include matching segments as additional local evidence when available. `hrm run QUERY` also records the retrieval summary in learning and provenance ledgers, and `eval run` treats non-zero HRM-text `retrieval_hits` as a regression signal that local evidence is connected before assessment.

## Alternatives Considered

### Implement Real Neural Pretraining

- Pros: Would be closer to the name “pretraining”.
- Cons: Requires corpus processing, model definitions, checkpoints, long-running training, likely GPU/cluster resources and real weight governance.
- Rejected: Violates current constraints and would overclaim capability.

### Store Only Documentation

- Pros: Minimal implementation risk.
- Cons: Does not create runtime-visible artifacts, commands, reports or auditable state.
- Rejected: The seam needs to be operationally inspectable by GARM.

### Fold Into Existing HRM Runtime

- Pros: Fewer modules.
- Cons: Blurs runtime reasoning with future pretraining provenance and checkpoint manifests.
- Rejected: Separate state makes the boundary explicit and auditable.

## Consequences

- GARM now exposes an operational HRM-text pretraining lifecycle without claiming trained weights.
- Corpus registration now captures local existence, bytes, FNV64, line count, inferred language/domain and source metadata.
- Directory ingestion creates a bounded local segment index for evaluation and future training preparation.
- HRM execution now leaves an auditable trail from retrieved text segment to HRM plan, learning entry and provenance entry.
- Reports, exports and artifacts include HRM-text state and checkpoint manifest paths.
- The seam can be extended later with real local corpus/objective/checkpoint handling if hardware and weights become available.
- The design preserves safety constraints: no shell execution, no network, no code mutation and no separate runtime.
