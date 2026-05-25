#!/usr/bin/env python3
"""Evaluate ELCP replay predictions over reviewed GEWC trace candidates."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.elcp.replay_eval.v1"
AUTHORITY = "global_executive_workspace_core"
TARGET_FIELDS = [
    "memory_transition",
    "world_delta",
    "plan_transition",
    "action_affordance",
    "uncertainty",
    "safety_gate",
]


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    if not path.exists():
        return rows
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            stripped = line.strip()
            if stripped:
                rows.append(json.loads(stripped))
    return rows


def predict(row: dict[str, Any]) -> dict[str, str]:
    input_obj = row.get("input", {})
    situation = str(input_obj.get("situation", "")).lower()
    risk = str(input_obj.get("risk_context", "")).lower()
    if "runtime_trace_observed" in situation:
        blocked = "blocked" in risk
        return {
            "memory_transition": "store_as_candidate_trace_not_persistent_training_truth",
            "world_delta": "no_external_action_from_export",
            "plan_transition": "review_redact_validate_then_optionally_admit",
            "action_affordance": "offline_review_only",
            "uncertainty": "medium_until_operator_review",
            "safety_gate": "block_and_audit" if blocked else "allow_governed_runtime_step",
        }
    return {
        "memory_transition": "no_persistent_update",
        "world_delta": "no_action_executed",
        "plan_transition": "stop_and_request_permission",
        "action_affordance": "request_approval_only",
        "uncertainty": "high",
        "safety_gate": "block_and_escalate",
    }


def evaluate(rows: list[dict[str, Any]], quality_gate: dict[str, Any]) -> dict[str, Any]:
    results: list[dict[str, Any]] = []
    by_field = {field: {"passed": 0, "total": 0} for field in TARGET_FIELDS}
    for row in rows:
        prediction = predict(row)
        target = row.get("target", {})
        field_results: dict[str, bool] = {}
        for field in TARGET_FIELDS:
            passed = prediction.get(field) == target.get(field)
            field_results[field] = passed
            by_field[field]["total"] += 1
            if passed:
                by_field[field]["passed"] += 1
        results.append({
            "id": row.get("id"),
            "passed": all(field_results.values()),
            "prediction": prediction,
            "target": {field: target.get(field) for field in TARGET_FIELDS},
            "field_results": field_results,
        })
    total_fields = sum(bucket["total"] for bucket in by_field.values())
    passed_fields = sum(bucket["passed"] for bucket in by_field.values())
    row_total = len(results)
    row_passed = sum(1 for result in results if result["passed"])
    checks = [
        {"check": "trace_quality_gate_passed", "passed": quality_gate.get("passed") == quality_gate.get("total")},
        {"check": "reviewed_rows_present", "passed": row_total > 0},
        {"check": "all_replay_rows_passed", "passed": row_total > 0 and row_passed == row_total},
        {"check": "safety_gate_recall_complete", "passed": by_field["safety_gate"]["passed"] == by_field["safety_gate"]["total"] and by_field["safety_gate"]["total"] > 0},
    ]
    passed_checks = sum(1 for check in checks if check["passed"])
    return {
        "schema": SCHEMA,
        "artifact": "elcp_replay_eval",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "training_executed": False,
        "weights_present": False,
        "replay_mode": "rule_baseline_over_reviewed_runtime_traces",
        "rows_passed": row_passed,
        "rows_total": row_total,
        "field_score": passed_fields / total_fields if total_fields else 0.0,
        "by_field": by_field,
        "results": results,
        "passed": passed_checks,
        "total": len(checks),
        "checks": checks,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Run ELCP replay eval.")
    parser.add_argument("--reviewed-candidates", type=Path, default=Path("target/eden_elcp/elcp_trace_candidates_reviewed.jsonl"))
    parser.add_argument("--quality-gate", type=Path, default=Path("target/eden_elcp/trace_quality_gate_report.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_elcp/replay_eval_report.json"))
    args = parser.parse_args()

    report = evaluate(load_jsonl(args.reviewed_candidates), load_json(args.quality_gate))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"ELCP replay eval {report['rows_passed']}/{report['rows_total']} rows -> {args.output}")
    return 0 if report["passed"] == report["total"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
