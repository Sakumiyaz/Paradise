#!/usr/bin/env python3
"""Build a non-mutating EDEN v0.3 operational capability demo trace."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.v03.operational_demo.v1"
AUTHORITY = "global_executive_workspace_core"


def load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    return value if isinstance(value, dict) else {}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--generalization", type=Path, default=Path("target/eden_v03/generalization_eval_report.json"))
    parser.add_argument("--admission", type=Path, default=Path("target/eden_v03/checkpoint_admission_report.json"))
    parser.add_argument("--runtime", type=Path, default=Path("target/eden_v03/live_inference_runtime_report.json"))
    parser.add_argument("--registry", type=Path, default=Path("target/eden_v03/checkpoint_registry.json"))
    parser.add_argument("--scaling", type=Path, default=Path("target/eden_v03/scaling_14b_plan.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_v03/operational_demo_trace.json"))
    args = parser.parse_args()

    generalization = load_json(args.generalization)
    admission = load_json(args.admission)
    runtime = load_json(args.runtime)
    registry = load_json(args.registry)
    scaling = load_json(args.scaling)
    steps = [
        ("load_curated_dataset", args.generalization.as_posix(), "v03_generalization_corpus_loaded"),
        ("load_long_checkpoint", args.admission.as_posix(), "1000_iter_candidate_reviewed"),
        ("register_checkpoint", args.registry.as_posix(), "candidate_recorded_without_git_weights"),
        ("activate_runtime_candidate", args.runtime.as_posix(), "native_inference_contract_ready"),
        ("verify_safety_boundary", args.admission.as_posix(), "production_release_blocked"),
        ("prepare_14b_scale_path", args.scaling.as_posix(), "14b_requires_separate_budget_and_gate"),
        ("record_audit_trace", args.output.as_posix(), "demo_trace_written"),
    ]
    trace_steps = [
        {
            "step": step,
            "artifact": artifact,
            "result": result,
            "mutates_runtime_state": False,
        }
        for step, artifact, result in steps
    ]
    passed = (
        generalization.get("passed") is True
        and admission.get("candidate_runtime_admission_allowed") is True
        and runtime.get("status") == "candidate_ready"
        and isinstance(registry.get("checkpoints"), list)
        and scaling.get("current_dense_ceiling") == 14_000_000_000
        and all(step["mutates_runtime_state"] is False for step in trace_steps)
    )
    report = {
        "schema": SCHEMA,
        "artifact": "eden_v03_operational_demo",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "passed": passed,
        "checkpoint_id": admission.get("checkpoint_id"),
        "steps": trace_steps,
        "safety_boundary": {
            "direct_memory_writes": False,
            "direct_objective_writes": False,
            "direct_tool_execution": False,
            "candidate_runtime_only": True,
            "production_release_allowed": False,
        },
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"EDEN v0.3 operational demo passed={passed} steps={len(trace_steps)} -> {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
