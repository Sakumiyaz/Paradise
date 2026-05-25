# EDEN Operator Examples

These scripts exercise the public `edenctl` and local HTTP surfaces against a
live EDEN runtime. They are examples for operators and integrators, not private
test fixtures.

Start EDEN before running them:

```sh
cargo run -p eden_core --bin edenctl -- start --port 8080 --state-dir /tmp/eden_garm
```

Then run an example from the repository root:

```sh
eden_core/src/garm/operator_examples/01_status_and_schemas.sh
```

Set `EDEN_BASE_URL` to target another local port:

```sh
EDEN_BASE_URL=http://127.0.0.1:8114 \
  eden_core/src/garm/operator_examples/06_locus_forge_bridge.sh
```

## Workflows

| Script | Purpose |
| --- | --- |
| `01_status_and_schemas.sh` | Read runtime status and schema metadata. |
| `02_permissions_audit.sh` | Inspect permission policy, audit and diff output. |
| `03_action_dry_run.sh` | Classify safe and high-risk commands without execution. |
| `04_degraded_recovery.sh` | Pause a GEWC handler, inspect degraded state and recover. |
| `05_demo_and_evidence.sh` | Generate operational demos and evidence bundle. |
| `06_locus_forge_bridge.sh` | Run Locus, Operator Forge and the governed CWM bridge. |

All examples preserve the public no-claim boundary. They generate local
runtime evidence only.
