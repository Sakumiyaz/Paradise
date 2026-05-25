# ADR-049: EDEN Operator Console As The Runtime Root Surface

## Status
Accepted

## Date
2026-05-23

## Context
The repository and runtime had strong technical surfaces but a plain visual
handoff: the README was mostly a long list, and the HTTP root endpoint returned
only raw text. That made EDEN look rough despite the available API contracts,
SDK, validation and GEWC runtime evidence.

The visual layer should improve operator comprehension without adding a
frontend build chain, JavaScript dependency or new runtime service.

## Decision
Add `docs/EDEN_OPERATOR_CONSOLE.html` as a static operator console and embed it
in the GARM API server. The runtime now serves the console at:

- `/`
- `/console`
- `/api/console`

The previous endpoint list remains available as plain text at:

- `/api/help`
- `/help`

The README is reorganized as a repo front door with status badges, system map,
quick start, command table, API family map and policy note.

## Alternatives Considered

### Add a full frontend application
- Pros: More interactive UI potential.
- Cons: Adds toolchain and maintenance cost before a live product UI is needed.
- Rejected: The current need is visual clarity and operator navigation.

### Keep root as plain text only
- Pros: Minimal.
- Cons: Poor first impression and weak discoverability.
- Rejected: Plain text is preserved at `/api/help`.

### Generate HTML dynamically
- Pros: Could show live metrics directly.
- Cons: Increases server complexity and couples layout to runtime state.
- Deferred: The static console links to live JSON endpoints instead.

## Consequences
- The repo and runtime now have a polished first visual surface.
- Existing API consumers keep a text help endpoint.
- No frontend dependencies or external services are introduced.
- The conformance runner now verifies that the root serves the HTML console.
