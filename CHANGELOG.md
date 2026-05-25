# Changelog

All notable changes are tracked here.

## Unreleased

- No unreleased changes.

## v0.1.0

- Published Paradise as a clean public repository with a single root commit.
- Verified the first public `GARM Verify` GitHub Actions run on `master`.
- Documented Paradise as the public Worldcell Runtime identity over Eden's
  broader architecture substrate.
- Kept the no-claim boundary explicit: this is not an AGI capability claim and
  not a trained model release.
- Included local-first runtime APIs, operator console, contracts, conformance
  evidence, runtime spine, Worldcell loop and public release documentation.

## v0.1.0-rc1

- Moved the GARM/GEWC runtime implementation into native library code under
  `eden_core/src/garm/` and added the official `eden-garm` runtime binary.
- Added native Locus and Operator Forge public command surfaces through SDK,
  `edenctl`, conformance, black-box and long-run gates.
- Added the governed `locus_operator_bridge` runtime artifact so Locus context
  and Operator Forge candidates enter CWM as hypotheses only.
- Added EDEN Engineering Practices for review scope, evidence, contract and
  safety expectations.
- Promoted `edenctl` to an official Cargo binary and `eden_core::sdk` to a
  public Rust client surface.
- Added `contracts/v1` with manifest, OpenAPI snapshots, schemas and examples.
- Added optional local API token enforcement through `EDEN_API_TOKEN`.
- Added v0.1.0-rc1 release-candidate notes and a stricter release gate.
- Validated the local RC path with `make eden-release-check`.
- Prepared the repository for a future public handoff without changing GitHub
  visibility.
- Added public-facing security, contribution, limitation and threat-model
  documentation.
- Added the EDEN operator console and API conformance evidence path.
- Removed generated or backup artifacts from the tracked tree.
- Added GitHub issue and pull request templates.

## v0.1.0-public-draft

- Draft tag name reserved for a future public-ready checkpoint.
- Not published as a GitHub release.

- Local release candidate notes added in `docs/releases/v0.1.0-rc1.md`.
- Not published as a GitHub release.
