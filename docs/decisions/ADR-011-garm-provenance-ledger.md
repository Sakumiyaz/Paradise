# ADR-011: GARM Provenance Ledger

## Status

Accepted.

## Context

GARM now records learning, uncertainty and experiments, but claims still need explicit source provenance. Without a provenance seam, reports can show outcomes without enough information about where supporting evidence came from.

## Decision

Add a GARM-native provenance ledger with commands `provenance`, `provenance record TEXT`, `provenance verify` and `provenance audit`. Records include claim, source, evidence kind, trust and status. State is persisted to `provenance_ledger.json`, included in reports, exports and artifact listings, and manual records write learning ledger traces.

## Consequences

- Evidence sources become auditable local state.
- Evaluation and benchmark scoring can account for provenance coverage.
- The seam remains deterministic and local: no network, shell execution, LLM, CI or separate runtime.
- Provenance complements, but does not replace, learning and uncertainty ledgers.
