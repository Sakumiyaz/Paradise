# Paradise Non-GPU Readiness Release Package

This package is the public, non-GPU release surface for Paradise. It validates
runtime, product, API and governance readiness. It does not validate learned
model capability and does not admit checkpoints.

## Required Commands

```sh
make contracts-validate
make paradise-non-gpu-readiness
make paradise-checkpoint-registry-smoke
make check
make eden-api-conformance
make public-audit
```

## Expected Artifacts

| Artifact | Purpose |
| --- | --- |
| `target/public_contracts/validation_report.json` | Contract manifest, schema and OpenAPI validation. |
| `target/paradise_non_gpu_readiness/non_gpu_readiness_report.json` | Product/runtime readiness report. |
| `paradise_checkpoint_registry_admission.json` | Native GEWC audit that keeps checkpoint admission blocked. |
| `target/paradise_release/release_package_manifest.json` | Commit, suggested tag, commands and artifact inventory. |
| API conformance report | SDK/API behavior and no-claim markers. |
| Operator console preview | Human-facing explanation of runtime state, gates and checkpoint boundary. |

## Claim Boundary

- `claim_allowed=false`
- `agi_claim=false`
- `production_model_allowed=false`
- no checkpoint weights are included
- no private datasets are included
- GPU training is out of scope for this package

For release environments with secret scanners installed, use:

```sh
make public-audit-strict
```
