# ADR-062: EDEN Engineering Practices

## Status

Accepted

## Date

2026-05-24

## Context

Paradise now exposes a public local-first runtime surface with SDK, CLI,
OpenAPI snapshots, versioned contracts, CI gates, operational evidence and a
claim boundary. The repository also contains broader Eden substrate modules with
mixed maturity. Without a project-specific engineering standard, future changes
can become difficult to review, mix architecture work with runtime behavior, or
weaken the distinction between public evidence and unsupported AGI claims.

Public engineering-practice guides are useful as general background, especially
around small reviewable changes and code health, but EDEN needs a standard that
is specific to its runtime, GEWC authority model, public contracts, safety
boundaries and validation artifacts.

## Decision

Adopt `docs/EDEN_ENGINEERING_PRACTICES.md` as the repository's engineering
standard for change authoring, review, validation and documentation.

The guide defines:

- the EDEN change unit;
- non-negotiable safety and claim boundaries;
- evidence expectations by change class;
- author and reviewer checklists;
- runtime-specific rules;
- contract-specific rules;
- AI agent contributor rules;
- ADR expectations.

## Alternatives Considered

### Link only to an external engineering-practices repository

- Pros: concise and familiar.
- Cons: not specific to EDEN's runtime gates, public claim boundary, GEWC
  authority model or local-first operational contract.
- Rejected because contributors need a project-native rule set.

### Put all guidance in `CONTRIBUTING.md`

- Pros: simple discovery.
- Cons: would make the root contribution guide too dense and harder to scan.
- Rejected in favor of a short `CONTRIBUTING.md` that links to a fuller practice
  guide.

### Rely only on CI

- Pros: automated and objective.
- Cons: CI cannot fully judge architecture fit, claim discipline, review scope
  or whether a change creates hidden authority paths.
- Rejected because EDEN requires both automated gates and human/agent review
  standards.

## Consequences

- Future PRs have a clear standard for scope, evidence and review.
- Public contract, GEWC, security and operational changes have explicit minimum
  evidence expectations.
- The repository gains a durable standard for AI agent contributions.
- The guide will need maintenance as EDEN's runtime surface and model/training
  surfaces evolve.
