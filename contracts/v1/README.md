# EDEN Public Contracts v1

Date: 2026-05-24

This directory contains the first versioned public contract set for EDEN
Hybrid. It is a stability surface for local operators, SDK users, conformance
tests and release evidence. It is not an AGI capability claim.

## Files

| Path | Purpose |
| --- | --- |
| `manifest.json` | Contract index, route families, gates and claim boundary. |
| `openapi/eden-runtime-api-v1.openapi.json` | Versioned runtime-state API contract. |
| `openapi/eden-operational-api-v1.openapi.json` | Versioned operational API contract. |
| `schemas/*.json` | JSON schemas for stable generated artifacts. |
| `examples/*.json` | Minimal examples for SDK and integration tests. |

Live OpenAPI snapshots can also be exported from a running local runtime:

```sh
cargo run -p eden_core --bin edenctl -- openapi export --output-dir contracts/v1/openapi
```

The export command writes runtime-generated `runtime.openapi.json` and
`operational.openapi.json` snapshots next to these versioned contracts.

## Stability Rules

- Additive fields are allowed within v1.
- Existing stable field names should not be removed or repurposed.
- Breaking changes require a v2 directory and an ADR.
- Public integrations should use `docs/EDEN_PUBLIC_API_V1.md`, this contract
  directory, `eden_core::sdk`, `edenctl`, or local HTTP.
- All validation and release contracts preserve `claim_allowed=false` and
  `agi_claim=false`.
- `schemas/paradise-non-gpu-readiness-v1.json` covers the local product/runtime
  readiness gate that does not require GPU, network, hardware devices or
  checkpoints.
- `schemas/paradise-public-contract-validation-v1.json` covers the stdlib-only
  manifest/schema/OpenAPI validation report.
- `schemas/paradise-dataset-license-manifest-v1.json` and
  `schemas/paradise-checkpoint-registry-v1.json` cover public dataset and
  checkpoint-admission boundaries.
- `schemas/paradise-checkpoint-registry-admission-v1.json` covers the native
  GEWC registry audit artifact.

Validate the public contract surface locally:

```sh
make contracts-validate
```

The command writes `target/public_contracts/validation_report.json`.
