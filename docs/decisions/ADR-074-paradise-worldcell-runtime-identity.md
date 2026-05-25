# ADR-074: Paradise Worldcell Runtime Identity

## Status
Accepted

## Date
2026-05-25

## Context
Paradise had a strong technical surface but the public entry point still read
like an internal research runtime: GARM, GEWC, conformance packages, validation
artifacts and cognitive architecture terminology appeared before a reader could
understand the product idea.

The project needs a public identity that is more original than "agent runtime"
or "safety layer" while still being technically honest and compatible with the
existing local-first evidence boundary.

## Decision
Use **Paradise** as the public name for the **Worldcell Runtime** identity.

Paradise describes the bounded cognitive world created around autonomous work:
context, authority, memory, world state, risk, action contracts, permissions,
runtime execution, evidence and safe learning.

The internal implementation names remain unchanged:

- GARM: local operator/runtime body.
- GEWC: executive workspace core.
- Locus: context and authority layer.
- Operator Forge: action contract/formal synthesis layer.
- ELCP: safe cognitive-transition preparation and hardening.

The repository now exposes a native `paradise worldcell eval` command and
`paradise_worldcell_runtime.json` artifact with schema
`eden-paradise-worldcell-runtime-v1`.

## Alternatives Considered

### Safety Layer for Agents
Clear and market-readable, but too generic. It describes a defensive wrapper,
not EDEN's world, memory, Locus, simulation and evidence model.

### Cognitive Contract Runtime
Technically accurate, but too narrow. Contracts are one part of the loop; the
runtime also needs context, simulation, memory and learning.

### Praxis Runtime
Close to EDEN's internal architecture, but abstract for first-time users.

### Locus Runtime
Very EDEN-specific, but Locus is a component rather than the whole system.

### Cognitive Vivarium
Original, but more metaphorical than operational.

### EDEN Worldcell Runtime
Strong concept, but the requested public name is Paradise. "Worldcell Runtime"
remains the category.

## Consequences

- README and docs can lead with a clearer product-level concept.
- Existing EDEN/GARM/GEWC contracts remain stable.
- Paradise becomes a public identity, not a wholesale rename of internals.
- Future console and CLI work should make the Worldcell loop visible as plan,
  dry-run, risk, permission, execution, evidence and memory.
- The no-claim boundary remains explicit: Paradise is not presented as a
  completed AGI or trained production LMM.
