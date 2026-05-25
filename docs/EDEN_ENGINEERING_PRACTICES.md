# EDEN Engineering Practices

This document defines the engineering standard for changes to Paradise. It is
informed by common large-codebase review practices, but it is EDEN-specific and
should not be treated as a copy of any external guide.

The goal is simple: every change should make the runtime, public API, evidence
surface or documentation healthier without weakening safety boundaries or
claim discipline.

## Core Standard

A change is ready when it is small enough to review, improves the project in a
clear way, preserves public contracts or documents intentional contract changes,
and includes the evidence needed to trust it.

Do not block progress for polish that can safely follow later. Do block changes
that reduce correctness, safety, reproducibility, auditability or public
truthfulness.

## EDEN Change Unit

An EDEN change should have one primary purpose:

- fix one bug;
- add one runtime/API capability;
- clarify one public document path;
- add one validation gate;
- refactor one boundary without changing behavior;
- update one contract with matching evidence.

Large work should be split by contract, runtime layer, API family, validation
gate or documentation surface. Refactors should be separate from behavior
changes unless the refactor is tiny and directly required.

## Non-Negotiables

- Preserve the public claim boundary: no completed-AGI or externally validated
  AGI claims unless future evidence genuinely supports them.
- Preserve `claim_allowed=false` and `agi_claim=false` in generated validation
  artifacts unless the claim system itself is intentionally redesigned.
- Keep mutation surfaces explicit and routed through governed runtime paths.
- Do not add credentials, private keys, `.env` files, generated private reports
  or runtime state directories.
- Keep EDEN local-first by default.
- Treat GEWC as the executive authority for runtime coordination; modules should
  not gain independent action authority by accident.
- Treat Locus and Operator Forge artifacts as governed evidence and formal
  candidates, not as direct memory writes, direct objective changes or proof of
  scientific truth.
- Treat `locus_operator_bridge.json` as a CWM hypothesis bridge only. It must
  preserve no direct memory writes, no objective writes and no model-weight
  updates.
- Treat `eden_core/src/garm/` and `cargo run -p eden_core --bin eden-garm --`
  as the native runtime surface. Examples may wrap it, but should not become a
  second implementation.
- Public API or contract changes require documentation and validation evidence.

## Change Classes

| Class | Examples | Minimum evidence |
| --- | --- | --- |
| Documentation only | README, docs index, release notes | `git diff --check`; link check by review |
| Rust internal refactor | module cleanup, naming, extraction | `make fmt`, focused `cargo test`, `make check` if examples/bins are touched |
| Runtime/API behavior | endpoint, command, SDK, CLI | `make fmt`, `make check`, focused tests, `make eden-api-conformance` |
| Public contract | OpenAPI, schemas, examples, manifests | `make eden-api-contracts`, `make eden-api-conformance` |
| Operational evidence | black-box, smoke, long-run gates | relevant gate plus artifact review |
| Security/policy boundary | auth, permissions, sandbox, command routing | focused tests, `make public-audit`, relevant runtime gate, threat-model update |
| GEWC authority/core routing | scheduler, handlers, lifecycle, action routing | focused tests, `make test`, `make eden-api-conformance`, ADR update |
| Native runtime layout | GARM/GEWC module, runtime binary, script entry points | `make fmt`, `make check`, `make native-runtime-layout`, focused runtime tests, docs/ADR update |
| Training/evaluation surface | ROCm profile, datasets, training scripts, capability benchmarks | `make training-smoke`, relevant focused tests, docs/ADR update; no stronger capability claims |

Use the smallest set that proves the changed surface. For broad or public
changes, run the larger gates before merge.

## Author Checklist

Before opening or merging a change:

- State the one primary purpose.
- List behavior changes separately from refactors.
- Identify touched public contracts, if any.
- Run the smallest meaningful validation set.
- Update docs when runtime behavior, public API or operator workflow changes.
- Add or update an ADR for durable architecture, API, validation or safety
  decisions.
- Keep generated local evidence out of Git unless it belongs under an explicit
  versioned contract path.
- Check that the change does not blur Paradise with a completed AGI claim.

## Reviewer Checklist

Reviewers should prioritize:

- **Design fit:** Does the change fit the EDEN layer model instead of creating a
  parallel control path?
- **Correctness:** Does the implementation handle error cases and degraded
  states?
- **Complexity:** Can future maintainers understand and safely modify it?
- **GEWC authority:** Does GEWC retain final routing, lifecycle or action
  authority where expected?
- **State and memory:** Is persistent state versioned, auditable and compatible
  with clean local runs?
- **Public contracts:** Are OpenAPI, schemas, SDK behavior and docs consistent?
- **Safety:** Are command-capable paths local, explicit, permissioned and
  auditable?
- **Tests:** Do tests cover the changed behavior rather than only the happy path?
- **Evidence:** Are the claimed checks actually relevant to the changed surface?
- **Claims:** Does public language remain precise and non-overstated?

Minor style preferences should not block a change when tooling already enforces
the style and the change improves the codebase.

## Review Outcomes

Use clear review outcomes:

- **Approve:** The change improves the project and required evidence is present.
- **Comment:** The change is acceptable, but follow-up polish or questions exist.
- **Request changes:** The change breaks correctness, contracts, safety,
  reproducibility, auditability or public claim discipline.
- **Split required:** The change mixes unrelated purposes or is too large to
  review with confidence.

## Runtime-Specific Rules

Runtime changes need extra care because they affect operator trust:

- Command execution must remain separate from dry-run.
- Dry-run endpoints must never queue or execute commands.
- API auth must fail closed when enabled.
- Clean validation runs should not depend on old `/tmp` state.
- Long-running gates should write evidence to explicit temporary directories.
- Local socket tests should be isolated from sandbox-safe unit tests.
- Recovery and degraded-mode behavior should be visible through operational
  status APIs.
- Operational scripts should execute native bins such as `eden-garm`,
  `eden-garm-api-conformance` and `eden-garm-package-validator`; examples are
  compatibility wrappers only.
- Training scripts must keep models subordinate to GEWC, block direct memory,
  objective or tool authority, and write claim-gated reports by default.

## Contract-Specific Rules

Versioned contracts under `contracts/v1/` are public promises. Changes there
should be treated as API changes:

- update examples and schemas together;
- keep OpenAPI snapshots aligned with runtime exports;
- document compatibility impact;
- run contract and conformance gates;
- avoid changing generated-looking files without explaining the source of truth.

## Agent And AI Contributor Rules

When an AI agent changes the repo, the same standard applies:

- read the local docs before broad edits;
- prefer established project patterns over new abstractions;
- do not remove user work or unrelated local changes;
- keep edits scoped and reviewable;
- run validation or clearly report why it could not be run;
- never turn architecture notes into unsupported capability claims.

## ADR Policy

Write or update an ADR when a change creates a decision that future maintainers
would need to understand. Examples:

- new public API surface;
- new safety boundary;
- new runtime authority model;
- new contract version;
- replacement of an architectural component;
- new validation or release gate.

ADRs should explain context, decision, alternatives and consequences. They are
not changelog entries; they are decision records.

## Maintenance

This guide should evolve when EDEN's public surface changes. If a future runtime
extracts GARM/GEWC out of `examples/`, or if training/model artifacts become a
public surface, this guide should be updated in the same change.
