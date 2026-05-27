# ADR-094: Paradise Non-GPU Product Hardening

## Status
Accepted

## Date
2026-05-27

## Context
GPU training for the Eden 70B modular family is paused until a suitable
multi-GPU environment is available. That does not block Paradise product and
runtime work. The repo still needs a clear public product boundary, model
interface, dataset governance, checkpoint admission policy, operator surface
and local readiness gate.

## Decision
Add a non-GPU hardening layer for Paradise:

- product specification;
- model interface contract;
- dataset governance policy;
- evaluation and checkpoint admission policy;
- external technical brief;
- technical debt register;
- local non-GPU readiness benchmark;
- public schema for the readiness report;
- Makefile target for reproducible execution.

The new gate is engineering evidence only. It does not train models, admit
checkpoints or make AGI capability claims.

## Consequences
- Paradise can keep maturing while GPU work is paused.
- Public/product documentation is separated from research claims.
- Model checkpoints remain blocked until training and admission evidence exist.
- Hardware, network and GPU tests remain explicit opt-in paths.
