# Repo Inspection

Plan a repository inspection without granting execution authority:

```bash
cargo run -p eden_core --bin paradise -- \
  --state-dir /tmp/paradise_repo_inspection \
  run --dry-run "inspect this repository and report risks"
```

Expected result:

- a Paradise intent session is recorded;
- a candidate command is selected;
- risk, permission and route metadata are written;
- no files are changed;
- no shell, network or GitHub action is executed.

Review the evidence:

```bash
test -s /tmp/paradise_repo_inspection/paradise_worldcell_sessions.json
```
