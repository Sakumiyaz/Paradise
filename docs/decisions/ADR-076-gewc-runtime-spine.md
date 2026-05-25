# ADR-076: Add GEWC Runtime Spine Contracts

## Status
Accepted

## Date
2026-05-25

## Context
The Paradise operational loop made intent, planning, approval and safe execution
native, but the broader runtime still needed a common executable substrate for
the questions raised by the architecture review: universal internal contracts,
event bus, global state, replay, model routing, memory retrieval, world-model
simulation, multiagent arbitration, security gates and observability.

Without a common substrate, Eden risks becoming a collection of individually
useful modules whose state, evidence and authority rules are similar but not
formally shared.

## Decision
Add `eden_core/src/garm/runtime_spine.rs` as a GEWC-owned operational substrate.
It writes:

- `runtime_spine.json`
- `runtime_internal_contracts.json`
- `runtime_event_bus.jsonl`
- `runtime_event_bus_state.json`
- `runtime_global_state.json`
- `runtime_global_state_log.jsonl`
- `runtime_replay_spine.json`
- `runtime_spine_verification.json`
- `runtime_security_gates.json`
- `runtime_model_router_contract.json`
- `runtime_memory_fabric_contract.json`
- `runtime_world_simulation_contract.json`
- `runtime_multiagent_contract.json`

Expose the substrate through `runtime spine eval`, `runtime spine audit`,
`runtime spine verify`, runtime-state artifacts, artifact APIs and operational
read routes. GEWC decision/completion recording now publishes append-only
runtime events and validated global-state mutations. Paradise sessions also
publish intent, plan, approval, block and completion events into the same
substrate. The verification command checks artifact presence, contiguous
event/state sequences, GEWC authority, snapshot consistency, replay counts,
payload hashes, claim boundaries and validated state writes.

## Alternatives Considered

### Keep Per-Module Artifacts Only
- Pros: No new module.
- Cons: Repeats authority, state and replay logic across modules.
- Rejected: It does not solve the disconnected-module risk.

### Make Paradise The State Kernel
- Pros: Product-facing and simple to explain.
- Cons: Paradise is the public Worldcell identity, while GEWC remains the
  executive authority.
- Rejected: It would blur identity/runtime branding with the internal kernel.

### Build A Separate Runtime Service
- Pros: Cleaner service boundary.
- Cons: Creates a second control plane and undermines the GEWC absorption work.
- Rejected: The spine must be native to GEWC/GARM.

## Consequences
- Future model, memory, tool, simulation and agent work has a shared contract
  target instead of one-off artifacts.
- Replay can be built from GEWC traces, runtime events, state mutations and
  action evidence.
- The spine has a local verification report rather than only a descriptive
  contract.
- The runtime remains no-claim gated: these are executable architecture
  contracts, not proof of AGI capability.
