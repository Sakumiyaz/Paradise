# Paradise Usable Today

Paradise is public runtime infrastructure, not a completed AGI or a production
model release. This page separates what is usable now from what is intentionally
blocked.

## Usable Now

| Surface | Status | Evidence |
| --- | --- | --- |
| Local Paradise CLI | Usable without sockets or network. | `make paradise-quickstart` |
| Worldcell dry-run loop | Usable for intent, plan, approval, execution trace and evidence. | `make paradise-operational-loop` |
| GARM/GEWC runtime | Native Rust runtime under `eden_core/src/garm/`. | `make test`, `make check` |
| Public API contracts | Versioned schemas and OpenAPI files are validated. | `make contracts-validate` |
| API conformance | Live local API can be checked from outside the process. | `make eden-api-conformance` |
| Operator console | Static local console for runtime/readiness artifacts. | `docs/EDEN_OPERATOR_CONSOLE.html` |
| Dataset manifest | Public corpus files can be counted, hashed and checked for private-data flags. | `make paradise-dataset-manifest` |
| Module semantic eval | Corpus coverage can be checked across memory, planner, world model, safety, router and observability routes. | `make paradise-module-semantic-eval` |
| Checkpoint evidence review | Existing local probe reports can be reviewed without admitting weights. | `make paradise-checkpoint-evidence-review` |
| Public release audit | Secret/path/document scans are available. | `make public-audit` |

## Blocked Until Evidence Exists

| Surface | Current decision | Why |
| --- | --- | --- |
| Production checkpoint admission | Blocked. | The public registry has no active checkpoint and no weights are committed. |
| Production model release | Blocked. | Requires checkpoint hashes, held-out eval, safety eval, rollback drill and operator approval. |
| AGI claim | Blocked. | Runtime and local evidence are engineering evidence, not external proof of general intelligence. |
| Autonomous high-risk actions | Blocked by default. | Actions must pass dry-run, policy, approval, audit and rollback boundaries. |
| GPU training/evaluation | Pending. | Deliberately outside the non-GPU public readiness path. |
| Native inference from released checkpoint | Pending. | Requires an admitted checkpoint artifact store entry; probe evidence alone is not enough. |

## Practical Interpretation

Paradise can be used today to run and inspect the governed runtime surface,
validate contracts, generate public evidence packages and review local model
probe evidence. It should not be described as a finished model, a released AGI
or a production autonomous agent.
