# ADR-016: GARM Verifiable RAG

## Status
Accepted

## Date
2026-05-21

## Context
GARM needs stronger retrieval-augmented reasoning without changing EDEN's local-first direction. The target is not a cloud RAG stack with a generative LLM, but a local, auditable retrieval layer that can prove why evidence was used or why it abstained.

## Decision
Extend HRM-text retrieval into verifiable RAG inside GARM. The implementation remains deterministic Rust and adds forensic ingestion metadata, duplicate detection, document versions, source trust, confidence scoring, citations, context packs, abstention, cache telemetry, and continuous retrieval evaluation.

Embeddings, ANN, and neural rerankers remain optional future layers. They are not required for correctness and cannot override provenance, citations, or insufficient-evidence gates.

## Consequences
- RAG quality improves without introducing LLM generation or external services.
- Every retrieved fragment carries citation data: document id, segment id, version, checksum, path, score, confidence, and exact text.
- Low-confidence retrieval produces an explicit abstention instead of letting weak context influence HRM.
- CI can test retrieval behavior through deterministic regression scripts and artifacts.
- Future semantic retrieval can be added as a secondary signal after local auditability is preserved.

## Alternatives Considered

### Full BM25 + Embeddings + ANN Immediately
Rejected for now. It improves semantic recall, but adds complexity and model/index lifecycle risks before EDEN has formal source trust, citations, and abstention controls.

### LLM-Based RAG Answering
Rejected. EDEN's runtime constraint is zero external/local LLM generation for core operation. HRM remains evidence-driven and auditable.

### Keep Simple Lexical Search Only
Rejected. Simple substring matching does not provide enough observability, confidence scoring, source policy, or hallucination fallback.
