# Project Structure

This document separates current operational surfaces from the broader Eden
substrate, preserved legacy sources and experimental research modules. It is
meant to reduce ambiguity for public readers.

## Logical Layers

| Layer | Status | Purpose |
| --- | --- | --- |
| Paradise workspace | Public surface | Build, CI, docs, operator quick start, conformance and release evidence. |
| Eden substrate | Mixed maturity | Broader modules for A-life, autopoiesis, cognition, memory, safety, physics, governance and experiments. |
| GARM operator runtime | Current | Local command/runtime surface, API server, state, reports and readiness packages. |
| GEWC executive core | Current | Runtime coordination, decision traces, handler topology, safety gates and lifecycle ownership. |
| Eden Locus Layer | Current | Native GEWC domain for personal context, authority parsing, evidence quarantine, permissions and privacy. |
| Eden Operator Forge | Current | Native GEWC domain for typed formal primitive synthesis and bounded expression-graph verification. |
| Locus/Forge bridge | Current | Operational-runtime bridge that admits authorized context and verified formal candidates into CWM as governed hypotheses only. |
| Model runtime governance | Current | Native GEWC path for model adapter lifecycle, first-model preparation, checkpoint manifests, training harness reports and model permissions without training weights. |
| Public APIs/evidence | Current | Runtime, operational, artifact, capability, validation, action and SDK conformance surfaces. |

## Current Primary Surface

| Path | Status | Purpose |
| --- | --- | --- |
| `README.md` | Current | Repository front door and operator quick start. |
| `docs/EDEN_SYSTEM_LAYERS.md` | Current | Layer model and terminology for the broader Eden architecture and this public repo. |
| `eden_core/src/bin/eden_garm.rs` | Current | Main GARM runtime binary entry point. |
| `eden_core/src/garm/` | Current | Runtime modules, GEWC, APIs, validation and scripts. |
| `eden_core/src/bin/edenctl.rs` | Current | Official local operator CLI binary. |
| `eden_core/src/sdk.rs` | Current | Public Rust SDK for the local runtime API. |
| `docs/EDEN_OPERATOR_CONSOLE.html` | Current | Static operator console served by the runtime root endpoint. |
| `docs/EDEN_RUNTIME_API_SURFACE.md` | Current | Runtime/API contract map. |
| `contracts/v1/` | Current | Versioned public manifest, OpenAPI snapshots, schemas and examples. |
| `docs/EDEN_SDK_CONFORMANCE.md` | Current | SDK and live API conformance contract. |
| `scripts/public_release_audit.sh` | Current | Public-readiness and secret hygiene audit. |
| `training/` | Prepared | CPU-safe capability smoke benchmark, AMD ROCm profile and future trainable-module entry point. |

## Validation And Release Evidence

| Path | Status | Purpose |
| --- | --- | --- |
| `Makefile` | Current | Local verification, conformance and package targets. |
| `.github/workflows/garm-verify.yml` | Current | Remote CI for format, tests, checks, API contracts, conformance, smoke and security. |
| `eden_core/src/garm_package_validator.rs` | Current | Independent readiness package validator implementation. |
| `eden_core/src/bin/eden_garm_package_validator.rs` | Current | Native `eden-garm-package-validator` binary entry point. |
| `eden_core/examples/eden_garm_package_validator.rs` | Compatibility | Thin wrapper over the native package validator. |
| `docs/decisions/` | Current | ADR history for architecture and operational decisions. |
| `docs/releases/` | Prepared | Draft release notes; not a published GitHub release. |
| `training/benchmarks/eden_capability_benchmark.py` | Current | Stdlib-only benchmark runner for memory, planning, tool safety and continual-learning smoke cases. |
| `training/configs/rocm_smoke.json` | Current | ROCm/AMD execution profile with CPU fallback and GEWC authority constraints. |
| `training/configs/elcp_latent_cognitive_prediction.json` | Current | ELCP latent cognitive prediction objective contract; prepared but not trained. |
| `training/data/capability_smoke.jsonl` | Current | Minimal deterministic dataset for current trainable-path smoke validation. |
| `training/data/elcp_transition_*.jsonl` | Current | Synthetic cognitive-transition fixtures for ELCP 4A contract validation. |
| `training/benchmarks/*elcp*.py` and `training/elcp/*.py` | Current | ELCP validation, CPU baseline, trace export, trace quality, replay eval, dataset freeze, metrics board, dry-run training interface and admission/readiness contract preparation. |
| `eden_core/src/garm/model_runtime.rs` | Current | GEWC-governed model adapter lifecycle, first-model 4A preparation, ELCP objective/admission/hardening preparation, checkpoint manifest, training harness and governance report generator. |
| `eden_core/src/garm/paradise_worldcell.rs` | Current | Public Paradise Worldcell Runtime and session loop over intent, dry-run planning, approval, GEWC execution and evidence memory. |
| `eden_core/src/garm/runtime_spine.rs` | Current | GEWC-owned contracts, guard decisions, workflow risk, circuit breakers, replay reconstruction and verification for internal messages, event bus, global state, security gates, model routing, memory fabric, world simulation and multiagent coordination. |

## Core Rust Crates

| Path | Status | Purpose |
| --- | --- | --- |
| `eden_core/` | Current | Main Rust crate and examples. |
| `mnemosyne/` | Workspace member | Secondary memory-related crate retained in the workspace. |

## Legacy And Experimental Areas

| Path | Status | Purpose |
| --- | --- | --- |
| `eden_core/src/` | Mixed | Broader Eden substrate modules, including A-life/autopoietic, cognitive, safety, memory, physics and experimental domains. |
| `eden_core/src/paradigms/` | Experimental | Paradigm examples and prototypes. |
| `eden_core/src/garm/legacy_sources/` | Preserved legacy | Historical source context retained for provenance. |
| `eden_core/corpus/` | Data/corpus | Local text corpus inputs for experiments and runtime tests. |

## Naming Caveat

`eden_core/examples/eden_garm.rs`,
`eden_core/examples/eden_garm_api_conformance.rs` and
`eden_core/examples/eden_garm_package_validator.rs` remain only as compatibility
wrappers. Runtime, conformance and package validation implementations are now
native library code under `eden_core/src/`, with official binaries under
`eden_core/src/bin/`. The runtime can later be extracted into a dedicated
`eden_garm_runtime` crate without changing the public API contract.

## Generated Or Local-Only Artifacts

These should not be committed:

- `target/`
- `.env`, `.env.*`
- local keys and certificates
- `__pycache__/`, `*.pyc`
- `*.bak`, `*.debug`, `*.tmp`
- `eden_core/reports/`
- `eden_core/language_reports/`
- runtime state directories such as `/tmp/eden_garm*`

## Public Claim Boundary

Use `docs/CLAIMS_AND_LIMITATIONS.md` as the source of truth. The project is a
local-first operator runtime, API surface and validation framework for Eden's
broader hybrid architecture; it is not presented as completed or externally
validated AGI.
