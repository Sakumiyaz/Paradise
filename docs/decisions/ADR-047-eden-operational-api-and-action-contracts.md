# ADR-047: Eden Operational API and Action Contracts

Date: 2026-05-23

## Status

Accepted

## Context

ADR-045 made release artifacts executable through read APIs. ADR-046 made
runtime state-management files executable through typed read APIs. The remaining
operational gap was the control plane: capabilities, GEWC runtime state,
validation status and command-action permissions were still spread across
existing diagnostic routes and command submission routes.

This created two risks:

- Read-only operational inspection was mixed with action-oriented routes.
- Operators had no typed dry-run contract for understanding how a command would
  route through GEWC before submitting it to a mutation endpoint.

## Decision

Add `operational api eval` as a GEWC validation command and introduce the Eden
Operational API surface. The command writes five no-claim artifacts:

- `operational_api_catalog.json`
- `operational_api_contracts.json`
- `operational_api_openapi.json`
- `operational_api_runtime.json`
- `operational_action_contracts.json`

The API server exposes read-only operational routes:

- `/api/operational/catalog`
- `/api/operational/openapi`
- `/api/operational/runtime`
- `/api/capabilities/catalog`
- `/api/capabilities/status`
- `/api/gewc/runtime`
- `/api/gewc/handlers`
- `/api/validation/status`
- `/api/actions/contracts`
- `/api/actions/dry-run?cmd=<command>`

Mutation remains restricted to the existing command submission routes:

- `/api/command?cmd=<command>`
- `/api/command_sync?cmd=<command>`
- `POST /api/command`

Dry-run never queues or executes commands. It only parses the command, binds it
through `GewcBodyRegistry`, and returns the route, domain, handler, lifecycle
policy and supervision/mutation classification.

## Consequences

- Capability, GEWC, validation and action contracts are now typed API surfaces.
- Read operations and action submission are documented as separate interfaces.
- The capability registry now tracks `operational_api`.
- The local held-out validation harness includes an operational API case.
- The independent package validator requires the five operational API artifacts.
- CI runs `make eden-api-contracts` as a dedicated API contract package gate.
- The no-claim policy remains explicit with `claim_allowed=false` and
  `agi_claim=false`.
