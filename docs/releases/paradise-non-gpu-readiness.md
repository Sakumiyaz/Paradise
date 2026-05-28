# Paradise Non-GPU Readiness Release Package

This package is the public, non-GPU release surface for Paradise. It validates
runtime, product, API and governance readiness. It does not validate learned
model capability and does not admit checkpoints.

## Required Commands

```sh
make contracts-validate
make paradise-non-gpu-readiness
make paradise-dataset-manifest
make paradise-module-semantic-eval
make paradise-checkpoint-evidence-review
make paradise-checkpoint-registry-smoke
make paradise-public-demo
make paradise-release-package
make check
make eden-api-conformance
make public-audit
```

## Expected Artifacts

| Artifact | Purpose |
| --- | --- |
| `target/public_contracts/validation_report.json` | Contract manifest, schema and OpenAPI validation. |
| `target/paradise_non_gpu_readiness/non_gpu_readiness_report.json` | Product/runtime readiness report. |
| `target/paradise_dataset_manifest/paradise_dataset_manifest.json` | Public dataset hashes, row counts, schemas and private-data/license posture. |
| `target/paradise_module_semantic_eval/module_semantic_eval_report.json` | Module-level semantic coverage across runtime, memory, world model, tools, safety and observability routes. |
| `target/paradise_checkpoint_evidence_review/checkpoint_evidence_review.json` | Review of local checkpoint/inference probe evidence while keeping public admission blocked. |
| `target/paradise_public_demo/demo_transcript.md` | Human-readable non-GPU demo transcript. |
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
