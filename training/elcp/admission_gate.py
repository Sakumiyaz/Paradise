#!/usr/bin/env python3
"""Prepare the ELCP checkpoint admission gate without admitting a checkpoint."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.elcp.admission_gate.v1"
AUTHORITY = "global_executive_workspace_core"


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def gate_report(
    validation: dict[str, Any],
    baseline: dict[str, Any],
    dry_run: dict[str, Any],
    trace_export: dict[str, Any],
) -> dict[str, Any]:
    validation_passed = not validation.get("errors")
    baseline_score = float(baseline.get("summary", {}).get("field_score", 0.0))
    dry_run_safe = (
        dry_run.get("training_executed") is False
        and dry_run.get("weights_written") is False
        and dry_run.get("gpu_job_submitted") is False
    )
    checks = [
        {"check": "transition_validation_passed", "passed": validation_passed},
        {"check": "baseline_report_present", "passed": baseline.get("schema") == "eden.elcp.baseline_report.v1"},
        {"check": "dry_run_no_training", "passed": dry_run_safe},
        {"check": "trace_export_report_present", "passed": trace_export.get("schema") == "eden.elcp.trace_export.v1"},
        {"check": "claim_boundary_preserved", "passed": all(
            report.get("claim_allowed") is False and report.get("agi_claim") is False
            for report in (validation, baseline, dry_run, trace_export)
        )},
    ]
    passed = sum(1 for check in checks if check["passed"])
    return {
        "schema": SCHEMA,
        "artifact": "elcp_admission_gate",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "training_executed": False,
        "weights_present": False,
        "checkpoint_admission_allowed": False,
        "4b_training_allowed": False,
        "mode": "pre_checkpoint_policy_gate",
        "passed": passed,
        "total": len(checks),
        "checks": checks,
        "current_baseline": {
            "field_score": baseline_score,
            "passed_rows": baseline.get("summary", {}).get("passed_rows"),
            "total_rows": baseline.get("summary", {}).get("total_rows"),
        },
        "future_minimums": {
            "state_transition_accuracy": "future_4b_threshold",
            "memory_transition_precision": "future_4b_threshold",
            "world_delta_consistency": "future_4b_threshold",
            "action_affordance_validity": "future_4b_threshold",
            "risk_calibration": "future_4b_threshold",
            "safety_gate_recall": "strict_high_recall",
            "direct_authority_violations": 0,
        },
        "required_before_checkpoint_admission": [
            "explicit_4b_operator_approval",
            "dataset_freeze_manifest",
            "trace_privacy_review",
            "pre_train_baseline_report",
            "post_train_eval_report",
            "checkpoint_manifest_with_hash",
            "GEWC governance review",
        ],
        "blockers": [
            "no_checkpoint_exists",
            "real_training_not_requested",
            "post_training_evaluation_missing",
            "human_release_review_missing",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Prepare ELCP admission gate.")
    parser.add_argument("--validation-report", type=Path, default=Path("target/eden_elcp/validation_report.json"))
    parser.add_argument("--baseline-report", type=Path, default=Path("target/eden_elcp/baseline_report.json"))
    parser.add_argument("--dry-run-report", type=Path, default=Path("target/eden_elcp/training_dry_run.json"))
    parser.add_argument("--trace-report", type=Path, default=Path("target/eden_elcp/trace_export_report.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_elcp/admission_gate_report.json"))
    args = parser.parse_args()

    report = gate_report(
        load_json(args.validation_report),
        load_json(args.baseline_report),
        load_json(args.dry_run_report),
        load_json(args.trace_report),
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "ELCP admission gate prepared "
        f"{report['passed']}/{report['total']} checks; checkpoint_admission_allowed=false -> {args.output}"
    )
    return 0 if report["passed"] == report["total"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
