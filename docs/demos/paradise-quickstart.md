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

$ cargo run -p eden_core --bin paradise -- checkpoint review
[PARADISE-CHECKPOINT-REGISTRY] schema=paradise.checkpoint_registry_admission.v1 status=written authority=global_executive_workspace_core claim_allowed=false agi_claim=false path=/tmp/paradise/paradise_checkpoint_registry_admission.json
registry: training/models/checkpoint_registry.json evidence: /tmp/paradise/paradise_checkpoint_registry_admission.json admission=false

$ cargo run -p eden_core --bin paradise -- checkpoint gate
[PARADISE-CHECKPOINT-ADMISSION-GATE] schema=paradise.checkpoint_admission_gate.v1 status=written authority=global_executive_workspace_core claim_allowed=false agi_claim=false path=/tmp/paradise/paradise_checkpoint_admission_gate.json
gate: /tmp/paradise/paradise_checkpoint_admission_gate.json admission=false

$ cargo run -p eden_core --bin paradise -- inference status
[EDEN-70B-INFERENCE-RUNTIME] schema=eden.modular_70b.inference_runtime.v1 status=written authority=global_executive_workspace_core claim_allowed=false agi_claim=false path=/tmp/paradise/eden_70b_inference_runtime.json
runtime: /tmp/paradise/eden_70b_inference_runtime.json real_checkpoint_inference_available=false

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
- checkpoint registry review is visible but does not admit weights;
- the admission gate is real and stays blocked until required evidence exists;
- native inference stays blocked until checkpoint admission exists;
- evidence files remain inspectable in the selected state directory.
