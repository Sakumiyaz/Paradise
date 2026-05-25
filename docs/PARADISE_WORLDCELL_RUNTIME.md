# Paradise Worldcell Runtime

Paradise is the public name for EDEN's Worldcell Runtime surface.

It is not a rename of the internal engine. GARM, GEWC, Locus, Operator Forge
and ELCP remain the native implementation terms. Paradise is the product-level
identity for the governed cognitive world that those pieces create around an
autonomous task.

## Positioning

Paradise gives agents a bounded world before they touch the real world.

An agent should not move from prompt to tool call without an operational
membrane. Paradise makes the membrane explicit: context, authority, memory,
world state, risk, policy, action contracts, evidence and safe learning.

```text
Intent
  -> Locus context + authority
  -> Cognitive field: memory + goals + risk + world state
  -> GEWC executive decision
  -> Operator Forge action contract
  -> World-model simulation
  -> Safety and permission gate
  -> Runtime body execution
  -> Evidence memory
  -> Safe learning
```

## What Makes It EDEN

Paradise is not just a safety wrapper. It is a Worldcell:

| Worldcell part | EDEN implementation |
| --- | --- |
| Membrane | Local-first permissions, sandbox expectations, dry-runs and no-claim gates. |
| Locus | Context, authority, privacy and evidence quarantine. |
| Executive workspace | GEWC routing, decision traces and lifecycle control. |
| Praxis space | Intent, state, evidence, constraints, affordances, projections and traces. |
| Action contracts | Operator Forge candidates and bounded verification. |
| Runtime body | GARM operational APIs, replay, recovery and state persistence. |
| Evidence memory | Provenance, action evidence, learning ledger and runtime artifacts. |
| Safe learning | ELCP preparation and hardening without unapproved training. |

The public thesis is:

```text
Agents should not act directly on the world.
Paradise gives them a worldcell first.
```

## Runtime Artifact

The native command is:

```text
paradise worldcell eval
```

The Make target is:

```bash
make paradise-worldcell
```

The command writes:

```text
paradise_worldcell_runtime.json
```

The artifact schema is:

```text
eden-paradise-worldcell-runtime-v1
```

The artifact keeps:

```json
{
  "claim_allowed": false,
  "agi_claim": false,
  "name": "Paradise",
  "category": "worldcell_runtime"
}
```

## Operational Loop

The session loop is native now, not only descriptive:

```text
paradise intent inspect runtime status safely
paradise plan
paradise approve
paradise execute
paradise sessions
```

The Make target is:

```bash
make paradise-operational-loop
```

The loop writes:

```text
paradise_worldcell_sessions.json
```

The session schema is:

```text
eden-paradise-worldcell-session-v1
```

Execution remains conservative. Paradise requires an explicit approval record
and still blocks commands whose dry-run says they mutate runtime state, require
supervision or are not standalone-safe.

The loop now also writes into the GEWC Runtime Spine. Intent, plan, approval,
safety block, execution start and execution completion receive
RuntimeSpineGuard decisions before they are published as append-only runtime
events and validated global-state mutations. This keeps Paradise from becoming
a separate flow: it remains a GEWC-owned Worldcell session over the same event
bus, state log, guard log and replay spine used by the rest of the runtime.
`runtime spine enforce`, `runtime spine replay` and `runtime spine verify` can
then prove that Paradise session events remain in the same GEWC-authorized
append-only sequence as the rest of the runtime.

## Current Boundary

Paradise currently exposes the local runtime identity, session loop and evidence
surface. It does not claim completed AGI, trained production LMM capability,
internet-facing production hardening or external benchmark superiority.

The next product-facing work should deepen the operator console around live
session approval UX and richer plan comparison, not bypass the command gates.
