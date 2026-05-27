# Paradise Non-GPU Readiness Release Package

This package is the public, non-GPU release surface for Paradise. It validates
runtime, product, API and governance readiness. It does not validate learned
model capability and does not admit checkpoints.

## Required Commands

```sh
make contracts-validate
make paradise-non-gpu-readiness
make check
make eden-api-conformance
```

## Expected Artifacts

| Artifact | Purpose |
| --- | --- |
| `target/public_contracts/validation_report.json` | Contract manifest, schema and OpenAPI validation. |
| `target/paradise_non_gpu_readiness/non_gpu_readiness_report.json` | Product/runtime readiness report. |
| API conformance report | SDK/API behavior and no-claim markers. |
| Operator console preview | Human-facing explanation of runtime state, gates and checkpoint boundary. |

## Claim Boundary

- `claim_allowed=false`
- `agi_claim=false`
- `production_model_allowed=false`
- no checkpoint weights are included
- no private datasets are included
- GPU training is out of scope for this package
