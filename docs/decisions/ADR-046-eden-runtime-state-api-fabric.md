# ADR-046: Eden Runtime State API Fabric

Date: 2026-05-23

## Status

Accepted

## Context

The Eden Artifact API Fabric made reproducible release artifacts inspectable
through executable read contracts. Runtime state still had a separate problem:
live state-management files existed across scheduler, memory, world-model,
GEWC runtime, safety, model and reporting domains, but there was no typed
runtime-state API surface that exposed them as a governed inventory.

Leaving those files as independent paths makes long-term operations harder:
operators need a stable catalog, API contracts, a live snapshot endpoint and a
whitelist that prevents arbitrary filesystem reads.

## Decision

Add `runtime state api eval` as a GEWC validation command and introduce the
Eden Runtime State API Fabric. The command writes four no-claim artifacts:

- `runtime_state_api_catalog.json`: inventory of whitelisted runtime state
  surfaces, presence state, bytes, FNV64, content type, domain and read
  endpoint.
- `runtime_state_api_contracts.json`: read, inspect, snapshot and OpenAPI
  contracts for each runtime state surface.
- `runtime_state_api_openapi.json`: OpenAPI-style contract for the read-only
  runtime-state routes.
- `runtime_state_api_runtime.json`: runtime status for the state API surface.

The source of truth is `runtime_state_api::state_specs()`. HTTP routes serve
only whitelisted runtime state names:

- `/api/runtime/catalog`
- `/api/runtime/state?name=<state_name>`
- `/api/runtime/snapshot`
- `/api/runtime/openapi`

The read route does not accept arbitrary filesystem paths. Mutation remains
command-routed through GEWC policy, body handlers and safety checks.

## Consequences

- Runtime state-management surfaces become executable read APIs instead of
  disconnected files.
- The capability registry now tracks `runtime_state_api`.
- The local held-out validation harness includes a runtime state API case.
- The independent package validator requires the four runtime-state API
  artifacts.
- The API server exposes a live read-only snapshot for runtime and organ
  metrics.
- The no-claim policy remains explicit with `claim_allowed=false` and
  `agi_claim=false`.
