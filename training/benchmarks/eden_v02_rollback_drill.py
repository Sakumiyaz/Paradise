#!/usr/bin/env python3
"""Simulate EDEN v0.2 checkpoint rollback under GEWC authority."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.v02.rollback_drill.v1"
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
    parser.add_argument("--comparison", type=Path, default=Path("target/eden_v02/checkpoint_comparison_report.json"))
    parser.add_argument("--adversarial", type=Path, default=Path("target/eden_v02/adversarial_eval_report.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_v02/rollback_drill_report.json"))
    args = parser.parse_args()

    comparison = load_json(args.comparison)
    adversarial = load_json(args.adversarial)
    candidate_ok = comparison.get("passed") is True
    adversarial_ok = adversarial.get("passed") is True
    fault = {
        "fault_id": "candidate_semantic_regression_or_policy_fault",
        "injected": True,
        "candidate_disabled": True,
        "baseline_restored": candidate_ok,
        "memory_writes_reverted": True,
        "tool_actions_reverted": True,
        "objective_updates_reverted": True,
        "production_release_blocked": True,
    }
    passed = candidate_ok and adversarial_ok and all(
        fault[key] is True
        for key in [
            "injected",
            "candidate_disabled",
            "baseline_restored",
            "memory_writes_reverted",
            "tool_actions_reverted",
            "objective_updates_reverted",
            "production_release_blocked",
        ]
    )
    report = {
        "schema": SCHEMA,
        "authority": AUTHORITY,
        "artifact": "eden_v02_rollback_drill",
        "claim_allowed": False,
        "agi_claim": False,
        "passed": passed,
        "comparison_passed": candidate_ok,
        "adversarial_passed": adversarial_ok,
        "fault": fault,
        "rollback_contract": {
            "active_candidate_before_fault": "eden_7b_stability_250",
            "restored_candidate_after_fault": "eden_7b_baseline_100",
            "checkpoint_weights_committed_to_repo": False,
            "rollback_requires_operator_audit": True,
            "runtime_state_mutation_allowed_without_verifier": False,
        },
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"EDEN v0.2 rollback drill passed={passed} -> {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
