# Paradise Developer Surface

Paradise exposes a small public surface over the larger Eden/GARM/GEWC runtime.
The goal is to make the first integration path understandable without requiring
new users to learn every internal research term.

## Stable Entry Points

| Surface | Purpose | Stability |
| --- | --- | --- |
| `cargo run -p eden_core --bin paradise -- status` | Socket-free local status and evidence path. | Public quickstart |
| `cargo run -p eden_core --bin paradise -- worldcell` | Generate the Paradise Worldcell Runtime artifact. | Public quickstart |
| `cargo run -p eden_core --bin paradise -- checkpoint review` | Review checkpoint registry policy without admitting weights. | Public quickstart |
| `cargo run -p eden_core --bin paradise -- inference status` | Show native inference readiness without loading production checkpoints. | Public quickstart |
| `cargo run -p eden_core --bin paradise -- run --dry-run <intent>` | Record intent and produce a reviewable dry-run session without execution. | Public quickstart |
| `make paradise-quickstart` | Run status, worldcell generation, checkpoint review, inference status and dry-run planning together. | Public quickstart |
| `edenctl` | Operator CLI for a live local API server. | Public runtime API |
| `/api/actions/dry-run?cmd=...` | Inspect command risk, permissions and route without execution. | Public runtime API |
| `contracts/v1/` | Versioned API schemas, examples and manifests. | Public contract |
| `.github/actions/paradise-verify` | Reusable GitHub Action for quick Paradise evidence generation. | Public CI helper |

## Internal Names Behind The Surface

| Public term | Internal owner |
| --- | --- |
| Runtime | GARM |
| Executive core | GEWC |
| Context authority | Eden Locus Layer |
| Action contracts | Eden Operator Forge |
| Worldcell evidence | Paradise Worldcell Runtime |
| Evidence bundle | Runtime state, artifact API, action evidence and provenance ledgers |

These names remain available in advanced documentation, but public examples
should start with Paradise commands and only introduce internal names when they
explain a real boundary.

## Extension Rules

- Add new user-facing commands behind `paradise` only when they are safe without
  sockets, network, model keys or training checkpoints.
- Keep dangerous or mutating behavior behind dry-run, permission and evidence
  gates.
- Treat `eden_core/src/garm/` as the native runtime implementation, not as a
  public plugin directory.
- Treat `contracts/v1/` as the public compatibility boundary.
- Do not let model output, retrieved documents or tool results write directly to
  objectives, policy, memory or execution state.

## Minimal Integration Path

1. Use `paradise status` to check local evidence generation.
2. Use `paradise worldcell` to generate the public runtime identity artifact.
3. Use `paradise checkpoint review` and `paradise inference status` to confirm
   model/checkpoint boundaries remain blocked.
4. Use `paradise run --dry-run "<intent>"` to produce a reviewable plan.
5. Move to `edenctl` and the local API only when you need a live runtime daemon.
6. Validate contracts through `make eden-api-conformance` before publishing an
   integration.

## Non-Goals

- The public CLI is not a replacement for the full GARM REPL.
- The public Action does not certify AGI capability.
- The quickstart path does not train or mutate model weights.
- The dry-run path does not execute tool calls, shell commands, network calls,
  file writes or repository mutations.
