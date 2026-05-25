#!/usr/bin/env python3
"""Aggregate ELCP preparation evidence into one metrics board."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.elcp.metrics_board.v1"
AUTHORITY = "global_executive_workspace_core"


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def passed(report: dict[str, Any]) -> bool:
    return report.get("passed") == report.get("total") and int(report.get("total", 0)) > 0


def build_board(args: argparse.Namespace) -> dict[str, Any]:
    validation = load_json(args.validation_report)
    baseline = load_json(args.baseline_report)
    trace_quality = load_json(args.trace_quality_report)
    replay = load_json(args.replay_report)
    freeze = load_json(args.dataset_freeze_manifest)
    dry_run = load_json(args.training_dry_run)
    admission = load_json(args.admission_gate)
    sources = {
        "validation": validation,
        "baseline": baseline,
        "trace_quality": trace_quality,
        "replay": replay,
        "dataset_freeze": freeze,
        "training_dry_run": dry_run,
        "admission_gate": admission,
    }
    checks = [
        {"check": f"{name}_passed_or_safe", "passed": passed(report) or name in {"baseline", "training_dry_run"}}
        for name, report in sources.items()
    ]
    checks.append({
        "check": "no_training_or_weights",
        "passed": all(report.get("training_executed") is False and report.get("weights_present") is False for report in sources.values()),
    })
    checks.append({
        "check": "claims_blocked",
        "passed": all(report.get("claim_allowed") is False and report.get("agi_claim") is False for report in sources.values()),
    })
    passed_checks = sum(1 for check in checks if check["passed"])
    return {
        "schema": SCHEMA,
        "artifact": "elcp_metrics_board",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "training_executed": False,
        "weights_present": False,
        "4b_training_allowed": False,
        "metrics": {
            "validation_rows": validation.get("total"),
            "baseline_field_score": baseline.get("summary", {}).get("field_score"),
            "trace_accepted_rows": trace_quality.get("accepted_rows"),
            "trace_rejected_rows": trace_quality.get("rejected_rows"),
            "replay_rows_passed": replay.get("rows_passed"),
            "replay_rows_total": replay.get("rows_total"),
            "replay_field_score": replay.get("field_score"),
            "dataset_freeze_id": freeze.get("freeze_id"),
            "admission_gate_passed": admission.get("passed"),
            "admission_gate_total": admission.get("total"),
        },
        "source_schemas": {name: report.get("schema") for name, report in sources.items()},
        "passed": passed_checks,
        "total": len(checks),
        "checks": checks,
        "decision": "architecture_and_data_prep_ready_for_operator_review_not_training",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Write ELCP metrics board.")
    parser.add_argument("--validation-report", type=Path, default=Path("target/eden_elcp/validation_report.json"))
    parser.add_argument("--baseline-report", type=Path, default=Path("target/eden_elcp/baseline_report.json"))
    parser.add_argument("--trace-quality-report", type=Path, default=Path("target/eden_elcp/trace_quality_gate_report.json"))
    parser.add_argument("--replay-report", type=Path, default=Path("target/eden_elcp/replay_eval_report.json"))
    parser.add_argument("--dataset-freeze-manifest", type=Path, default=Path("target/eden_elcp/dataset_freeze_manifest.json"))
    parser.add_argument("--training-dry-run", type=Path, default=Path("target/eden_elcp/training_dry_run.json"))
    parser.add_argument("--admission-gate", type=Path, default=Path("target/eden_elcp/admission_gate_report.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_elcp/metrics_board.json"))
    args = parser.parse_args()

    report = build_board(args)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"ELCP metrics board {report['passed']}/{report['total']} checks -> {args.output}")
    return 0 if report["passed"] == report["total"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
