# How To Build On EDEN

Date: 2026-05-24

This guide explains how to integrate with the current Paradise public
runtime surface without depending on internal modules. The intended integration
model is local-first: start the runtime, inspect contracts, dry-run actions,
execute only through governed command routes, then collect evidence.

## 1. Start The Runtime

```sh
cargo build --bin eden-garm -p eden_core
target/debug/eden-garm \
  --daemon \
  --api-port 8080 \
  --pid-file /tmp/eden_garm.pid \
  --log-file /tmp/eden_garm.log \
  --state-dir /tmp/eden_garm
```

The same startup can be launched through the public operator CLI:

```sh
cargo run -p eden_core --bin edenctl -- start --port 8080 --state-dir /tmp/eden_garm
```

The operator console is served at:

```text
http://127.0.0.1:8080/
```

For local token protection, set `EDEN_API_TOKEN` before startup and pass the
same token to `edenctl`:

```sh
export EDEN_API_TOKEN='local-development-token'
cargo run -p eden_core --bin edenctl -- --token "$EDEN_API_TOKEN" status
```

## 2. Inspect Status

Use `edenctl` for a stable operator surface:

```sh
cargo run -p eden_core --bin edenctl -- status
cargo run -p eden_core --bin edenctl -- schemas operational_status
```

Equivalent HTTP endpoints:

```text
/api/operational/status
/api/operational/schema?name=operational_status
```

## 3. Discover Contracts Before Acting

Before executing a command, inspect:

```sh
cargo run -p eden_core --bin edenctl -- schemas
cargo run -p eden_core --bin edenctl -- permissions
cargo run -p eden_core --bin edenctl -- dry-run "status"
cargo run -p eden_core --bin edenctl -- dry-run "evolve"
```

The dry-run route classifies a command without queueing or executing it. A
client should treat dry-run output as a mandatory preflight for any action that
may mutate runtime state.

## 4. Execute Through GEWC

Only execute through the stable command routes or `edenctl command`:

```sh
cargo run -p eden_core --bin edenctl -- command "operational api eval"
cargo run -p eden_core --bin edenctl -- command "runtime state api eval"
cargo run -p eden_core --bin edenctl -- command "artifact api eval"
```

Do not call internal Rust modules directly from an integration. GEWC owns the
runtime command path, records decisions and applies safety gates.

For long-running commands, use the async queue/poll mode:

```sh
cargo run -p eden_core --bin edenctl -- command --wait-sec 180 "readiness package"
```

## 5. Run Locus And Operator Forge

Use Locus to admit operator context under authority and permission boundaries:

```sh
cargo run -p eden_core --bin edenctl -- locus eval
cargo run -p eden_core --bin edenctl -- locus ingest "operator preference :: governed context"
cargo run -p eden_core --bin edenctl -- locus context "operator permission boundary"
```

Use Operator Forge to synthesize and verify bounded formal candidates:

```sh
cargo run -p eden_core --bin edenctl -- forge eval
cargo run -p eden_core --bin edenctl -- forge synth "causal risk model for governed action"
cargo run -p eden_core --bin edenctl -- forge verify
```

Generate the governed bridge into CWM hypothesis evidence and read it back:

```sh
cargo run -p eden_core --bin edenctl -- command "operational runtime eval"
curl -fsS 'http://127.0.0.1:8080/api/runtime/state?name=locus_operator_bridge'
```

The bridge is intentionally conservative. It treats Locus context and Operator
Forge candidates as hypotheses routed through GEWC, not as direct writes to
memory, objectives or model weights.

The same flow is available as an executable operator example:

```sh
eden_core/src/garm/operator_examples/06_locus_forge_bridge.sh
```

## 6. Manage Runtime State Safely

Permission and recovery operations are command-routed:

```sh
cargo run -p eden_core --bin edenctl -- permissions audit
cargo run -p eden_core --bin edenctl -- permissions diff
cargo run -p eden_core --bin edenctl -- recovery run
```

For a degraded-mode exercise:

```sh
cargo run -p eden_core --bin edenctl -- command "gewc lifecycle world_model pause"
cargo run -p eden_core --bin edenctl -- status
cargo run -p eden_core --bin edenctl -- recovery run
```

## 7. Collect Evidence

Generate demo and replay evidence:

```sh
cargo run -p eden_core --bin edenctl -- demo run
cargo run -p eden_core --bin edenctl -- command "operational replay run"
```

Bundle local evidence:

```sh
cargo run -p eden_core --bin edenctl -- evidence bundle \
  --state-dir /tmp/eden_garm \
  --log-file /tmp/eden_garm.log \
  --stdout-file /tmp/eden_garm.stdout \
  --output /tmp/eden_garm/operational_evidence_bundle.json
```

## 8. Validate Compatibility

Use the public gates:

```sh
make eden-release-check
make eden-api-conformance
make operational-blackbox
make long-run-stability
```

`eden-release-check` is the full local release-candidate gate. It runs format,
build, native-layout, tests, security, smoke, public audit, API contracts,
conformance, black-box and long-run stability checks.

`eden-api-conformance` validates the live endpoint from outside the process.
`operational-blackbox` verifies runtime behavior as a local external process.
`long-run-stability` checks startup, command execution, degraded recovery,
replay, packaging, restart and independent validation.

Export live OpenAPI snapshots when you need runtime-generated contract files:

```sh
cargo run -p eden_core --bin edenctl -- openapi export --output-dir contracts/v1/openapi
```

## Integration Rules

- Treat Paradise as a governed runtime, not as a raw model endpoint.
- Use dry-run before mutation.
- Build against `docs/EDEN_PUBLIC_API_V1.md`, `contracts/v1/`, `edenctl` or
  `eden_core::sdk`.
- Treat LLM/model outputs as proposals unless accepted by the runtime contract.
- Read schema names from `/api/operational/schemas`.
- Use `/api/operational/replay` and evidence bundles for audits.
- Keep user, tool and system permissions outside prompts and inside explicit
  runtime policy.
- Do not depend on undocumented internal file paths or handler names unless
  they appear in `docs/EDEN_PUBLIC_API_V1.md`.
