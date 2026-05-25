# Eden System Layers

This document explains how the public Paradise repository relates to the larger
Eden architecture. It exists to prevent two common misreadings:

- Paradise is not the complete Eden organism.
- Eden is not only the A-life/autopoietic substrate.

Paradise is the public, local-first operator/runtime surface for an evolving
hybrid architecture. It exposes the parts that can be run, inspected, validated
and discussed reproducibly today.

## Layer Model

| Layer | Name | Current role | Current location |
| --- | --- | --- | --- |
| 0 | Paradise public identity | Product-level Worldcell Runtime framing and session loop for governed autonomous work. | `README.md`, `docs/PARADISE_WORLDCELL_RUNTIME.md`, `eden_core/src/garm/paradise_worldcell.rs` |
| 1 | Paradise workspace | Public repository, build, CI, docs, release evidence and local verification. | workspace root |
| 2 | Eden substrate | Broader Eden domains: A-life, autopoiesis, cognition, memory, safety, physics, governance and experiments. | `eden_core/src/` |
| 3 | GARM operator runtime | Local command/runtime surface, state persistence, reports, readiness packages and API serving. | `eden_core/src/garm/` |
| 4 | GEWC executive core | Executive workspace for routing, decision traces, body handlers, safety gates and lifecycle ownership. | `eden_core/src/garm/global_executive_workspace.rs` and related modules |
| 5 | Runtime spine | Common executable contracts and verification for internal messages, event bus, global state, replay, security, model routing, memory, simulation and multiagent coordination. | `eden_core/src/garm/runtime_spine.rs` |
| 6 | Public APIs and evidence | Runtime API, operational API, artifact API, SDK conformance, dry-run contracts and release packages. | `docs/`, `Makefile`, GARM API modules |

GARM/GEWC now lives as native library code under `eden_core/src/garm/`. The
old `eden_core/examples/eden_garm.rs` target is retained only as a compatibility
wrapper over the native runtime. A future refactor can move this module into a
dedicated crate without changing the public contract.

## Terminology

| Term | Definition |
| --- | --- |
| Eden | The broader hybrid architecture. It includes executive cognition, memory, reasoning, agentic action, safety, A-life/autopoietic substrate and experimental domains. |
| Paradise | The public Worldcell Runtime identity for EDEN. It describes and now records the bounded cognitive world that forms around autonomous tasks before action. |
| A-life/autopoietic substrate | Eden domains for Autons, structural fields, morphodynamic processes, membranes, homeostasis and life-like simulation. This is important, but it is not all of Eden. |
| GARM | The current operator runtime. It turns architecture into local commands, state, reports, API surfaces and reproducible packages. |
| GEWC | Global Executive Workspace Core. It is the executive coordination core inside GARM, not a separate product and not a language model. |
| GEWC handler | A typed runtime domain owned by GEWC. Handlers keep former GARM and legacy bodies under one executive lifecycle. |
| GEWC model control plane | Model-plural routing inside GEWC. It selects language, reasoning, code, memory, verifier, safety, world-model, tool-use or future multimodal roles without making any one model the core. |
| GEWC module lifecycle supervisor | Native GEWC control plane for handler health, pause, resume, restart, isolation, quarantine, disable and recovery decisions. It is policy-gated, audited in the GEWC runtime log and cannot become a second core. |
| Runtime Spine | GEWC-owned operational substrate that makes contracts executable and verifiable across internal messages, append-only events, global state, replay, safety, model routing, memory retrieval, world simulation and multiagent coordination. |
| Operational Runtime Phase | The runtime path that executes the eight operational components: persistent task state, governed action execution, lifecycle controls, memory transactions, CWM state, Locus/Forge bridge, governed agents and replay/evaluation. |
| Mnemosyne | Workspace memory crate retained for memory-related development. |
| Praxis Nexus | Eden's governed cognitive-operational substrate for intent, state, evidence, constraints, affordances, projections and traces. |
| Eden Locus Layer | GEWC-native personal context and authority substrate. It admits context as evidence with authority, trust, quarantine, permission and privacy metadata instead of letting tools or documents write directly into memory. |
| Eden Operator Forge | GEWC-native formal synthesis domain. It builds typed primitive bases and expression-graph candidates, then verifies graph hygiene before any CWM or memory export. |
| Conformance package | Local evidence that the live API endpoint, SDK surface and policy markers match the published contracts. It is engineering evidence, not AGI validation. |
| Worldcell | A bounded operational world for an agent task: membrane, Locus, cognitive field, executive decision, action contract, simulation, safety gate, runtime body, evidence memory and safe learning. The native session artifact is `paradise_worldcell_sessions.json`. |

## Current Relationship Between Layers

```text
Operator / SDK
    -> Paradise Worldcell Runtime identity
    -> Public local APIs
    -> GARM operator runtime
    -> GEWC executive core
    -> Eden domains
       -> memory, reasoning, tools, safety, validation
       -> A-life/autopoietic and other substrate modules
```

GARM/GEWC should be read as the operational and executive surface over Eden, not
as a replacement for the broader Eden substrate. The public repository is useful
because it makes the runtime path, API contracts, evidence packages and claim
boundaries inspectable while the broader architecture continues to mature.

GEWC is intentionally model-plural rather than LLM-centric. A conventional LLM
can be one cognitive engine, but GEWC remains the authority that routes work to
language, reasoning, code, memory, verifier, safety, world-model, tool-use and
future multimodal roles under explicit training and action boundaries.

GEWC also owns module lifecycle supervision. Each handler receives a lifecycle
state and an allowed action set from the core, so paused, recovering, isolated,
quarantined, disabled or failed handlers can be held before execution instead of
acting outside the executive workspace. Lifecycle events are written to the same
GEWC runtime audit channel as decisions and completions.

Two newer GEWC handlers deepen the runtime without creating new cores.
`gewc_locus_context_body_handler` owns personal context admission, authority
classification, evidence quarantine, permission boundaries and operator
timeline records. `gewc_formal_synthesis_body_handler` owns Praxis Nexus and
Eden Operator Forge commands for typed formal candidates. Both remain command
routed, lifecycle supervised and exposed through the existing artifact and
runtime-state APIs.

The operational runtime phase is the bridge from architecture to executable
runtime evidence. The `operational runtime eval` command writes eight local
runtime artifacts, including `locus_operator_bridge.json`, and exposes the phase
through the runtime state and operational API surfaces. The bridge admits Locus
context and Operator Forge candidates into CWM only as governed hypotheses, not
as direct memory writes or accepted truths. It still preserves the no-claim
boundary: the phase demonstrates local governed operation, not completed AGI.

The executable operational surface now includes GEWC-routed commands for the
five runtime controls: `operational task submit|run|audit`,
`operational action execute <command>`, `operational memory commit|rollback`,
`operational replay run`, `operational smoke run`,
`operational scenario run` and `gewc lifecycle <handler> <action>`. These
commands mutate only local runtime state, action evidence, memory ledgers or
GEWC lifecycle traces, and they remain available through the existing command
API path rather than a second control plane. Action permissions are classified
through a small risk matrix: read, local mutation, external tool,
destructive/autonomous and clarification-required.

`runtime spine eval` now writes the common runtime contracts that keep these
pieces from becoming disconnected modules. `runtime spine enforce` makes the
guard path explicit: runtime events and global-state mutations receive
append-only guard decisions before they are committed, unauthorized state
writers are blocked, workflow risk is scored at chain level and circuit
breakers can degrade the runtime without deleting evidence. `runtime spine
verify` checks the spine's own event/state sequence integrity, GEWC authority
boundary, replay counts, guard coverage, breaker health and no-claim policy.
GEWC decisions and completions emit append-only runtime events and global-state
mutations. Paradise sessions also publish intent, plan, approval, block and
completion events into the same spine, so replay can reconstruct the
operational path from GEWC traces, events, state mutations and action evidence.

The runtime contract is now explicit and executable. `operational api eval`
writes `operational_contract.json`, `/api/operational/contract` serves it, and
`make operational-blackbox` validates the contract from outside the process by
checking health, readiness, dry-run boundaries, runtime phase execution,
scenario evidence, replay and action evidence.

## What This Repo Is Useful For Today

- Running the current Eden operator runtime locally.
- Inspecting runtime state, GEWC traces, handler topology and API contracts.
- Producing reproducible evidence packages.
- Testing local conformance from outside the process.
- Reviewing safety gates, dry-run behavior and no-claim policy markers.
- Understanding how Eden's broader domains are intended to become governed,
  inspectable runtime surfaces.

## What This Repo Does Not Prove

- It does not prove completed AGI.
- It does not prove that the local LMM/foundation-model side is trained.
- It does not prove external benchmark superiority.
- It does not mean every experimental module under `eden_core/src/` is equally
  mature.
- It does not make the local API suitable for public internet exposure.
