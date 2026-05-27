# Paradise Product Specification

Paradise is the public product surface of the Eden project: a local-first
runtime for governed autonomous agents.

The commercial problem is narrow and concrete: agents should not be able to
touch files, tools, repositories, APIs or long-lived memory just because a model
produced a plausible tool call. Paradise inserts a bounded Worldcell between
intent and action.

## Product Promise

Run autonomous agent work locally with:

- explicit intent capture;
- structured context and authority parsing;
- dry-run before sensitive action;
- permission gates;
- typed action contracts;
- rollback and recovery plans;
- append-only evidence;
- model outputs treated as hypotheses;
- local API conformance and release artifacts.

## What Paradise Is

| Product term | Runtime owner |
| --- | --- |
| Paradise Worldcell | Public bounded execution surface |
| GEWC | Executive authority and coordination core |
| GARM | Native operator runtime |
| Locus Layer | Context, authority and quarantine |
| Operator Forge | Typed action/formal-candidate contract synthesis |
| Runtime Spine | Event bus, state mutation log, replay and safety gates |
| Evidence bundle | Reproducible local artifacts and conformance reports |

## Buyer/User Fit

Paradise is useful for:

- developers testing local agents without handing over direct machine authority;
- AI infrastructure teams that need audit trails and permissioned tool use;
- labs evaluating agent architecture without claiming AGI;
- compliance and security reviewers that need replayable evidence;
- future Eden model operators that need safe checkpoint admission.

## Non-Goals

- Paradise is not marketed as completed AGI.
- Paradise is not a hosted SaaS in the current repo.
- Paradise does not ship production model weights.
- Paradise does not execute high-risk actions by default.
- Paradise does not rely on external model output as final truth.

## Product Readiness Boundary

The current public-ready boundary is runtime and governance, not learned model
capability. GPU training is a separate model-development track. The product can
still be useful before final Eden checkpoints exist because it validates:

- action governance;
- API contracts;
- evidence generation;
- local operator workflows;
- dry-run and permissions;
- model-adapter boundaries.

The current executable product gate is:

```sh
make paradise-release-package
```

It bundles public contract validation with the non-GPU readiness report and
preserves the no-claim checkpoint boundary.
