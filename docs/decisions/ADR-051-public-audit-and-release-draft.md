# ADR-051: Public Audit Script And Draft Release Notes

## Status
Accepted

## Date
2026-05-23

## Context
The repository is private but being prepared for a possible public release. The
remaining gaps were project-structure clarity, repeatable secret hygiene,
history cleanup guidance, remote CI visibility and draft release notes.

Rewriting Git history would require force-pushing rewritten commits. That is a
destructive coordination event and should not be performed implicitly.

## Decision
Add non-destructive public-release preparation assets:

- `docs/PROJECT_STRUCTURE.md` for current, legacy and experimental areas;
- `docs/HISTORY_REWRITE_PLAYBOOK.md` for future history cleanup with explicit
  owner approval;
- `scripts/public_release_audit.sh` and `make public-audit` for local secret,
  artifact and document checks;
- `docs/releases/v0.1.0-public-draft.md` as draft release notes;
- README and docs index links to the new material.

The local tag `v0.1.0-public-draft` may be moved to the final preparation
commit, but no public GitHub release is created by this decision.

## Consequences
- The repo has a repeatable public-readiness audit.
- Secret scanning can use `gitleaks` or `trufflehog` when installed, with a
  built-in regex fallback.
- History rewrite remains documented and intentionally manual.
- Public release notes are prepared without publishing the repository or release.
