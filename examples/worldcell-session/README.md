# Worldcell Session

Generate the public Worldcell artifact and then plan one session:

```bash
cargo run -p eden_core --bin paradise -- \
  --state-dir /tmp/paradise_worldcell_session \
  worldcell

cargo run -p eden_core --bin paradise -- \
  --state-dir /tmp/paradise_worldcell_session \
  run --dry-run "inspect runtime status safely"
```

Verify the artifacts:

```bash
test -s /tmp/paradise_worldcell_session/paradise_worldcell_runtime.json
test -s /tmp/paradise_worldcell_session/paradise_worldcell_sessions.json
```

The first file describes the bounded runtime identity. The second file records
the intent, dry-run, permission and evidence session.
