# EDEN Documentation Index

This directory is the visual and architectural handoff for Paradise, the
governed local runtime from the Eden project.

## Start Here

| Document | Use |
| --- | --- |
| `PARADISE_WORLDCELL_RUNTIME.md` | Public Paradise identity: Worldcell Runtime loop, session artifact and boundaries. |
| `PARADISE_DEVELOPER_SURFACE.md` | Public CLI, API, contracts, GitHub Action and extension boundaries. |
| `PARADISE_PRODUCT_SPEC.md` | Public product definition and commercial boundary. |
| `PARADISE_MODEL_INTERFACE.md` | GEWC-owned model adapter packet contract and model authority limits. |
| `PARADISE_DATASET_GOVERNANCE.md` | Dataset source, privacy, split and public-export policy. |
| `PARADISE_EVALUATION_AND_ADMISSION.md` | Non-GPU readiness, eval families and checkpoint admission policy. |
| `PARADISE_USABLE_TODAY.md` | Explicit split between usable runtime surfaces and blocked model/checkpoint/AGI claims. |
| `PARADISE_EXTERNAL_BRIEF.md` | Non-confidential technical package for compute partners. |
| `PARADISE_TECHNICAL_DEBT_REGISTER.md` | Known non-GPU follow-ups separated from GPU blockers. |
| `PARADISE_ROADMAP.md` | Runtime-first roadmap, GPU lane and non-goals. |
| `releases/paradise-non-gpu-readiness.md` | Public non-GPU readiness release package. |
| `releases/v0.2.0-public-readiness.md` | Public readiness release notes. |
| `demos/paradise-quickstart.md` | Short transcript for the socket-free public quickstart. |
| `EDEN_SYSTEM_LAYERS.md` | Layer model and terminology for Paradise, Eden, GARM, GEWC and the Runtime Spine. |
| `EDEN_ENGINEERING_PRACTICES.md` | Project engineering standard for review scope, evidence, contracts and safety boundaries. |
| `EDEN_OPERATOR_CONSOLE.html` | Static operator console served by the runtime root endpoint. |
| `EDEN_RUNTIME_API_SURFACE.md` | Map of runtime, operational, artifact and action API surfaces. |
| `EDEN_PUBLIC_API_V1.md` | Frozen public v1 runtime/API/command contract. |
| `HOW_TO_BUILD_ON_EDEN.md` | Integration guide for external tools and operators. |
| `EDEN_OPERATIONAL_CONTRACT.md` | Health, readiness, degraded-mode, shutdown and action-boundary contract. |
| `EDEN_TRAINING_SURFACE.md` | ROCm/AMD training entry point, smoke benchmark and claim-gated model policy. |
| `EDEN_SDK_CONFORMANCE.md` | SDK and external live-endpoint conformance contract. |
| `CLAIMS_AND_LIMITATIONS.md` | Public claim boundary and current limitations. |
| `THREAT_MODEL.md` | Threat model for repository, local API and command surfaces. |
| `PROJECT_STRUCTURE.md` | Current, legacy and experimental code ownership map. |
| `HISTORY_REWRITE_PLAYBOOK.md` | Safe procedure for future destructive history cleanup. |
| `releases/v0.1.0-public-draft.md` | Draft release notes; not a published release. |
| `releases/v0.1.0-rc1.md` | Local release-candidate notes; not a published release. |
| `decisions/` | ADR history for runtime, GEWC, API and validation decisions. |

Versioned public contracts live outside this directory at `contracts/v1/`.

## Runtime Visual Surface

Open the static console directly:

```text
docs/EDEN_OPERATOR_CONSOLE.html
```

Or run EDEN and open:

```text
http://127.0.0.1:8080/
```

The console is static by design. Live state remains available through JSON
endpoints so operators and SDK consumers can inspect the runtime without a
frontend build chain.

## Public Readiness Gates

The public release surface can be checked without GPU access:

```sh
make contracts-validate
make paradise-non-gpu-readiness
make paradise-dataset-manifest
make paradise-module-semantic-eval
make paradise-checkpoint-evidence-review
make paradise-strong-eval
make paradise-public-demo
make paradise-release-package
make paradise-external-validation-package
```

These commands validate contracts, schema/OpenAPI manifest shape, dataset
license boundaries, module semantic coverage, checkpoint evidence review,
checkpoint registry policy, strong non-GPU family evidence, external validation
packaging and non-GPU product/runtime readiness. They do not use GPU, admit
checkpoints or certify learned model capability.

## Paradise CLI

`paradise` is the socket-free public quickstart CLI:

```sh
cargo run -p eden_core --bin paradise -- status
cargo run -p eden_core --bin paradise -- worldcell
cargo run -p eden_core --bin paradise -- checkpoint review
cargo run -p eden_core --bin paradise -- checkpoint dry-run-admit
cargo run -p eden_core --bin paradise -- checkpoint gate
cargo run -p eden_core --bin paradise -- inference status
cargo run -p eden_core --bin paradise -- run --dry-run "inspect runtime status safely"
```

Copyable public examples live under:

```text
examples/
```

## Operator CLI

`edenctl` is the supported local CLI wrapper over the public API:

```sh
cargo run -p eden_core --bin edenctl -- status
cargo run -p eden_core --bin edenctl -- schemas
cargo run -p eden_core --bin edenctl -- dry-run "status"
cargo run -p eden_core --bin edenctl -- locus eval
cargo run -p eden_core --bin edenctl -- forge eval
cargo run -p eden_core --bin edenctl -- doctor
```

Executable example workflows live under:

```text
eden_core/src/garm/operator_examples/
```

The current end-to-end Locus/Forge/CWM bridge example is:

```text
eden_core/src/garm/operator_examples/06_locus_forge_bridge.sh
```

## Public-Ready Documents

Root-level documents:

- `LICENSE`
- `SECURITY.md`
- `CONTRIBUTING.md`
- `CHANGELOG.md`
- `CODE_OF_CONDUCT.md`
- `PUBLIC_RELEASE.md`

These documents keep the public repository reviewable without making any AGI
capability claim.
