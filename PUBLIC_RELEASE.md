# Public Source Checkpoint

Status: public readiness package prepared. A versioned source tag can point to
the current non-GPU readiness state, but production checkpoint admission,
training and AGI claims remain blocked.

## What Paradise Is

Paradise is the public local-first Rust runtime for Eden's governed
operator/runtime surface: GARM, GEWC, Worldcell sessions, runtime APIs,
reproducible validation artifacts, operator console and an external API
conformance suite.

It is a public surface for operating, inspecting and validating an evolving Eden
architecture. It is not the complete Eden system packaged as a finished product.
## What It Is Not

Paradise is not presented as a completed AGI system. Local architecture
evidence, runtime conformance and readiness packages are engineering evidence,
not proof of general intelligence.

## Current Public-Ready Surfaces

- `README.md` repo front door.
- `docs/EDEN_SYSTEM_LAYERS.md` system layer and terminology map.
- `docs/EDEN_OPERATOR_CONSOLE.html` static runtime console.
- `docs/EDEN_RUNTIME_API_SURFACE.md` API contract map.
- `docs/EDEN_PUBLIC_API_V1.md` stable public API contract.
- `docs/EDEN_SDK_CONFORMANCE.md` SDK and conformance suite.
- `docs/HOW_TO_BUILD_ON_EDEN.md` integration guide.
- `eden_core/src/garm/operator_examples/` executable operator workflows,
  including the Locus/Forge-to-CWM bridge example.
- `contracts/v1/` versioned public contract directory.
- `docs/CLAIMS_AND_LIMITATIONS.md` claim boundary.
- `docs/THREAT_MODEL.md` local runtime threat model.
- `docs/PROJECT_STRUCTURE.md` maturity and ownership map.
- `docs/HISTORY_REWRITE_PLAYBOOK.md` history cleanup procedure.
- `docs/PARADISE_USABLE_TODAY.md` explicit usable-vs-blocked surface.
- `docs/releases/v0.1.0-public-draft.md` draft release notes.
- `docs/releases/v0.1.0-rc1.md` local release-candidate notes.
- `docs/releases/v0.2.0-public-readiness.md` current public readiness notes.
- `SECURITY.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `LICENSE`.

## Recommended Public Checkpoint Checklist

- Confirm GitHub Actions pass on the public remote.
- Run a fresh secret scan.
- Confirm no `.env`, key, token or generated runtime state is tracked.
- Run `make public-audit`.
- Optionally run `make install-secret-scanners` and `make public-audit-strict`.
- Run `make paradise-dataset-manifest`.
- Run `make paradise-module-semantic-eval`.
- Run `make paradise-checkpoint-evidence-review`.
- Run `cargo run -p eden_core --bin paradise -- checkpoint review`.
- Run `cargo run -p eden_core --bin paradise -- inference status`.
- Run `make paradise-public-demo`.
- Run `make paradise-release-package`.
- Run `make eden-release-check`.
- Run `make eden-api-conformance`.
- Run `make eden-api-contracts`.
- Run `make long-run-stability`.
- Decide whether to run the history rewrite playbook.
- Review `docs/CLAIMS_AND_LIMITATIONS.md` before writing release notes.

## Current Public Readiness Tag

Suggested tag pattern: `paradise-public-readiness-<commit12>`.

Do not attach checkpoint weights, private datasets, raw GPU workspaces or
credentials to a GitHub release. Public release artifacts should be limited to
validation reports, demo transcripts, release package manifests and docs.
