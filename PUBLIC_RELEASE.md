# Public Source Checkpoint

Status: public source checkpoint. No versioned GitHub release has been
published.

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
- `docs/releases/v0.1.0-public-draft.md` draft release notes.
- `docs/releases/v0.1.0-rc1.md` local release-candidate notes.
- `SECURITY.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `LICENSE`.

## Recommended Public Checkpoint Checklist

- Confirm GitHub Actions pass on the public remote.
- Run a fresh secret scan.
- Confirm no `.env`, key, token or generated runtime state is tracked.
- Run `make public-audit`.
- Run `make eden-release-check`.
- Run `make eden-api-conformance`.
- Run `make eden-api-contracts`.
- Run `make long-run-stability`.
- Decide whether to run the history rewrite playbook.
- Review `docs/CLAIMS_AND_LIMITATIONS.md` before writing release notes.

## Draft Version

Suggested local tag: `v0.1.0-rc1`.

Do not publish a versioned GitHub release until the owner explicitly approves
release notes and the claim boundary in `docs/CLAIMS_AND_LIMITATIONS.md`.
