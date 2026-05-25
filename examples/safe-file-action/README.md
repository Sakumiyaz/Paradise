# Safe File Action

Represent a local file operation as intent and dry-run evidence before any
execution path exists:

```bash
cargo run -p eden_core --bin paradise -- \
  --state-dir /tmp/paradise_safe_file_action \
  run --dry-run "organize temporary files into reviewable groups"
```

Paradise should produce a session with:

- intent;
- context authority;
- dry-run metadata;
- world-model simulation field;
- Operator Forge action contract field;
- explicit approval requirement;
- `would_execute=false` at the CLI layer.

This example is documentation-first. A future executable recipe must add a
typed tool contract, simulator and rollback policy before real file writes.
