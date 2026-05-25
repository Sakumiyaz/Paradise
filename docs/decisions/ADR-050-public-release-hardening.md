# ADR-050: Public Release Hardening Without Publishing

## Status
Accepted

## Date
2026-05-23

## Context
The repository is intended to look professional and be ready for future public
review, but the owner has not requested changing repository visibility or
publishing a GitHub release.

The tracked tree contained generated or backup artifacts that are not suitable
for a public handoff, and the project lacked standard public-facing documents
such as a license, security policy, contribution guide and explicit claim
limitations.

## Decision
Prepare the repository for future publication without changing visibility:

- remove tracked generated, backup and debug artifacts;
- harden `.gitignore` for secrets, reports, Python bytecode, backups and local
  runtime outputs;
- add `LICENSE`, `SECURITY.md`, `CONTRIBUTING.md`, `CHANGELOG.md`,
  `CODE_OF_CONDUCT.md` and `PUBLIC_RELEASE.md`;
- add `docs/CLAIMS_AND_LIMITATIONS.md` and `docs/THREAT_MODEL.md`;
- add GitHub issue and pull request templates;
- add a visual preview asset for the operator console;
- reserve a local draft tag name for a future release without publishing it.

## Consequences
- The repo has a clearer public handoff posture.
- The claim boundary is explicit before any external audience sees the code.
- Secret hygiene and local API risks are documented.
- No GitHub visibility change or public release publication is performed by
  this decision.
