# ADR-045: Eden Artifact API Fabric

Date: 2026-05-22

## Status

Accepted

## Context

EDEN now produces a large release-candidate artifact set: readiness reports,
architecture maps, GEWC runtime traces, Praxis substrate files, ecosystem
contracts and independent validation material. Before this ADR, those files
were reproducible but mostly passive. Operators could inspect them through the
package or filesystem, but there was no single executable API contract for
reading, cataloging and validating every artifact.

## Decision

Add `artifact api eval` as a GEWC validation command and introduce the Eden
Artifact API Fabric. The command writes three no-claim artifacts:

- `artifact_api_catalog.json`: inventory of every reproducible artifact,
  presence state, bytes, FNV64, content type, domain and read endpoint.
- `artifact_api_contracts.json`: read, inspect, validate and generation
  contracts for each artifact.
- `artifact_api_runtime.json`: runtime status for the read-only artifact API
  surface.

The source of truth is `reproducible_package::artifact_specs()`. HTTP routes
serve only whitelisted artifact names:

- `/api/artifact/catalog`
- `/api/artifact?name=<artifact_name>`
- `/api/artifact/runtime`

The read route does not accept arbitrary filesystem paths.

## Consequences

- Release artifacts become executable read APIs instead of disconnected files.
- The capability registry now tracks `artifact_api`.
- The local held-out validation harness includes an artifact API case.
- The independent package validator requires the three API artifacts.
- The no-claim policy remains explicit with `claim_allowed=false` and
  `agi_claim=false`.
