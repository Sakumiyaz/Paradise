# ADR-095: Paradise Release And Checkpoint Registry Gates

## Status
Accepted

## Date
2026-05-27

## Context
Paradise needs a public release path that remains useful while GPU training and
checkpoint inference are paused. The previous package had documents and
manifests, but checkpoint registry admission and release packaging were not
fully executable through the native runtime.

## Decision
Add a GEWC-owned checkpoint registry admission command and a non-confidential
release package manifest.

The checkpoint registry command reads `training/models/checkpoint_registry.json`
and writes `paradise_checkpoint_registry_admission.json` under runtime state. It
keeps `checkpoint_admission_allowed=false`, `production_model_allowed=false`,
`claim_allowed=false` and `agi_claim=false`.

The release package target writes
`target/paradise_release/release_package_manifest.json` with commit, suggested
tag, required commands and public artifact inventory. It excludes checkpoint
weights, private datasets, credentials, GPU workspaces and generated runtime
directories.

## Consequences
- Checkpoint registry policy is executable, not just documented.
- Public release preparation is reproducible without GPU.
- The operator console can read the checkpoint registry admission artifact from
  the local artifact API.
- Secret scanning has a strict mode for release environments that install
  `gitleaks` and `trufflehog`, while fallback regex remains available locally.
- GPU training and checkpoint inference remain explicitly out of scope.
