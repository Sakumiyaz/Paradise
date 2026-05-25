# Paradise Quickstart Demo

This transcript shows the intended first-run experience. It avoids sockets,
network access, model keys and training.

```text
$ cargo run -p eden_core --bin paradise -- status
[PARADISE-STATUS] state=ready ready=true latest_decision=gewc-... permissions=... claim_allowed=false state_dir=/tmp/paradise
evidence: /tmp/paradise/operational_runtime_phase.json
next: cargo run -p eden_core --bin paradise -- worldcell

$ cargo run -p eden_core --bin paradise -- worldcell
[PARADISE-WORLDCELL-RUNTIME] passed=8/8 claim_allowed=false path=/tmp/paradise/paradise_worldcell_runtime.json
sessions: /tmp/paradise/paradise_worldcell_sessions.json

$ cargo run -p eden_core --bin paradise -- run --dry-run "inspect runtime status safely"
[PARADISE-RUN-DRY-RUN] session=paradise-... status=planned would_execute=false candidate="status"
[PARADISE-INTENT] id=paradise-... status=intent_recorded path=/tmp/paradise/paradise_worldcell_sessions.json
[PARADISE-PLAN] id=paradise-... status=planned risk=low approval_required=true path=/tmp/paradise/paradise_worldcell_sessions.json
evidence: /tmp/paradise/paradise_worldcell_sessions.json
```

The visible contract is:

- `claim_allowed=false`;
- `agi_claim=false`;
- dry-run before action;
- no candidate action executed by `run --dry-run`;
- evidence files remain inspectable in the selected state directory.
