#!/usr/bin/env python3
"""Prepare the ELCP 4B readiness contract without authorizing training."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.elcp.4b_readiness_contract.v1"
AUTHORITY = "global_executive_workspace_core"


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def passed(report: dict[str, Any]) -> bool:
    return report.get("passed") == report.get("total") and int(report.get("total", 0)) > 0


def build_contract(metrics: dict[str, Any], freeze: dict[str, Any]) -> dict[str, Any]:
    checks = [
        {"check": "metrics_board_passed", "passed": passed(metrics)},
        {"check": "dataset_freeze_manifest_passed", "passed": passed(freeze)},
        {"check": "dataset_not_locked_for_training", "passed": freeze.get("dataset_locked_for_training") is False},
        {"check": "candidate_pool_locked_for_review", "passed": freeze.get("candidate_pool_locked_for_review") is True},
        {"check": "no_training_executed", "passed": metrics.get("training_executed") is False},
        {"check": "no_weights_present", "passed": metrics.get("weights_present") is False},
        {"check": "claims_blocked", "passed": metrics.get("claim_allowed") is False and metrics.get("agi_claim") is False},
    ]
    passed_checks = sum(1 for check in checks if check["passed"])
    return {
        "schema": SCHEMA,
        "artifact": "elcp_4b_readiness_contract",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "training_executed": False,
        "weights_present": False,
        "checkpoint_admitted": False,
        "4b_training_allowed": False,
        "contract_prepared": passed_checks == len(checks),
        "contract_scope": "pre_training_operator_review_contract",
        "passed": passed_checks,
        "total": len(checks),
        "checks": checks,
        "required_before_4b_training": [
            "explicit_operator_approval",
            "gpu_budget_approval",
            "dataset_lock_for_training_with_hashes",
            "trace_privacy_review",
            "checkpoint_output_path",
            "post_train_eval_plan",
            "rollback_plan",
            "GEWC governance review",
        ],
        "current_blockers": [
            "operator_approval_missing",
            "gpu_budget_not_approved",
            "dataset_locked_for_review_not_training",
            "checkpoint_path_not_admitted",
            "post_train_eval_missing",
        ],
        "metrics_board": metrics.get("metrics", {}),
        "freeze_id": freeze.get("freeze_id"),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Write ELCP 4B readiness contract.")
    parser.add_argument("--metrics-board", type=Path, default=Path("target/eden_elcp/metrics_board.json"))
    parser.add_argument("--dataset-freeze-manifest", type=Path, default=Path("target/eden_elcp/dataset_freeze_manifest.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_elcp/4b_readiness_contract.json"))
    args = parser.parse_args()

    report = build_contract(load_json(args.metrics_board), load_json(args.dataset_freeze_manifest))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"ELCP 4B readiness contract {report['passed']}/{report['total']} checks -> {args.output}")
    return 0 if report["passed"] == report["total"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
