# ADR-008: GARM Working Memory And Attention

## Status

Accepted.

## Context

GARM has long-term memory, KG retrieval, goals, evaluation, learning, world modelling, benchmarks and controlled execution. It still needs a short-horizon focus seam so operators and future local policies can see what the runtime is currently attending to without overloading long-term memory.

## Decision

Add a GARM-native working memory node with commands `attention`, `attention TEXT`, `attention clear` and `attention audit`. Items are bounded, locally weighted and persisted to `working_memory.json`. Attention reports are included in audits, reports, exports, artifact listings, evaluation evidence and benchmark evidence. Manual focus writes a learning ledger trace.

## Consequences

- Short-term focus becomes explicit and auditable.
- Long-term memory and KG remain separate from transient attention.
- The seam is deterministic and local: no LLM, network, shell execution or separate runtime.
- Attention evidence can improve evaluation and benchmark coverage without making identity-level claims.
