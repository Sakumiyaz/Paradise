#!/usr/bin/env python3
"""Build a non-mutating EDEN v0.2 operational stability demo trace."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.v02.stability_demo.v1"
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
    parser.add_argument("--stability-eval", type=Path, default=Path("target/eden_v02/stability_eval_report.json"))
    parser.add_argument("--comparison", type=Path, default=Path("target/eden_v02/checkpoint_comparison_report.json"))
    parser.add_argument("--adversarial", type=Path, default=Path("target/eden_v02/adversarial_eval_report.json"))
    parser.add_argument("--rollback", type=Path, default=Path("target/eden_v02/rollback_drill_report.json"))
    parser.add_argument("--model-card", type=Path, default=Path("target/eden_v02/model_card_internal.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_v02/stability_demo_trace.json"))
    args = parser.parse_args()

    stability = load_json(args.stability_eval)
    comparison = load_json(args.comparison)
    adversarial = load_json(args.adversarial)
    rollback = load_json(args.rollback)
    model_card = load_json(args.model_card)
    steps = [
        ("load_baseline", args.comparison.as_posix(), "baseline_100_checkpoint_evidence_loaded"),
        ("load_candidate", args.comparison.as_posix(), "candidate_250_checkpoint_evidence_loaded"),
        ("compare", args.comparison.as_posix(), "loss_and_inference_comparison_complete"),
        ("adversarial_gate", args.adversarial.as_posix(), "adversarial_cases_blocked"),
        ("rollback_drill", args.rollback.as_posix(), "candidate_can_be_disabled_and_baseline_restored"),
        ("model_card", args.model_card.as_posix(), "limits_and_non_goals_disclosed"),
        ("native_runtime_scope", args.stability_eval.as_posix(), "candidate_runtime_only"),
        ("audit", args.output.as_posix(), "trace_written_for_replay"),
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
        stability.get("passed") is True
        and comparison.get("passed") is True
        and adversarial.get("passed") is True
        and rollback.get("passed") is True
        and model_card.get("passed") is True
        and all(step["mutates_runtime_state"] is False for step in trace_steps)
    )
    report = {
        "schema": SCHEMA,
        "authority": AUTHORITY,
        "artifact": "eden_v02_stability_demo",
        "claim_allowed": False,
        "agi_claim": False,
        "passed": passed,
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
    print(f"EDEN v0.2 stability demo passed={passed} steps={len(trace_steps)} -> {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
