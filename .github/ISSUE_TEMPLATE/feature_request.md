---
name: Feature request
about: Propose a scoped improvement
title: "feat: "
labels: enhancement
assignees: ""
---

## Problem

What problem should this solve?

## Proposal

Describe the smallest useful change.

## Acceptance Criteria

- [ ] The change is observable through docs, tests, API output or generated artifacts.
- [ ] The change has a clear validation command.
- [ ] The change does not expand claim language beyond the documented boundary.

## Scope

What is explicitly out of scope?

## Non-Goals

List related work that should not be included in this issue.

## API Or Runtime Impact

Does this affect GEWC routing, command mutation, read-only APIs, validation
artifacts or SDK conformance?

## Safety And Claim Boundary

Confirm whether this changes any no-claim policy markers:

- `claim_allowed=false`
- `agi_claim=false`

## Validation Plan

```bash
make fmt
make check
make eden-api-conformance
```
