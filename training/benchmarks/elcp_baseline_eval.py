#!/usr/bin/env python3
"""CPU-safe baseline evaluator for Eden Latent Cognitive Prediction."""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any


SCHEMA = "eden.elcp.baseline_report.v1"
TARGET_FIELDS = [
    "memory_transition",
    "world_delta",
    "plan_transition",
    "action_affordance",
    "uncertainty",
    "safety_gate",
]


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            stripped = line.strip()
            if stripped:
                rows.append(json.loads(stripped))
    return rows


def most_common_target(rows: list[dict[str, Any]], field: str, fallback: str) -> str:
    values = [
        str(row.get("target", {}).get(field, ""))
        for row in rows
        if row.get("target", {}).get(field)
    ]
    if not values:
        return fallback
    return Counter(values).most_common(1)[0][0]


def predict(row: dict[str, Any], train_rows: list[dict[str, Any]]) -> dict[str, str]:
    input_obj = row.get("input", {})
    surface = str(input_obj.get("surface_text", "")).lower()
    situation = str(input_obj.get("situation", "")).lower()
    risk = str(input_obj.get("risk_context", "")).lower()

    defaults = {
        "memory_transition": most_common_target(train_rows, "memory_transition", "retrieve_relevant_memory"),
        "world_delta": most_common_target(train_rows, "world_delta", "no_external_state_change"),
        "plan_transition": most_common_target(train_rows, "plan_transition", "verify_then_continue"),
        "action_affordance": most_common_target(train_rows, "action_affordance", "read_only_allowed"),
        "uncertainty": most_common_target(train_rows, "uncertainty", "medium"),
        "safety_gate": most_common_target(train_rows, "safety_gate", "allow_read_only"),
    }

    text = f"{surface} {situation} {risk}"
    if "irreversible" in text or "high" in risk:
        return {
            "memory_transition": "no_persistent_update",
            "world_delta": "no_action_executed",
            "plan_transition": "stop_and_request_permission",
            "action_affordance": "request_approval_only",
            "uncertainty": "high",
            "safety_gate": "block_and_escalate",
        }
    if "failed" in text or "rollback" in text:
        return {
            "memory_transition": "store_failure_as_audit_evidence_only",
            "world_delta": "no_external_action",
            "plan_transition": "rollback_or_replan_with_constraints",
            "action_affordance": "audit_and_replay_allowed",
            "uncertainty": "low_on_failure_cause",
            "safety_gate": "block_original_action",
        }
    if "untrusted" in text or "injection" in text:
        return {
            "memory_transition": "do_not_store_untrusted_instruction_as_policy",
            "world_delta": "document_content_loaded_as_untrusted_data_only",
            "plan_transition": "fetch_then_sanitize_then_verify",
            "action_affordance": "read_allowed_execute_blocked",
            "uncertainty": "medium_due_to_untrusted_source",
            "safety_gate": "allow_read_block_instruction_transfer",
        }
    if "memory write" in text or "state mutation" in text:
        return {
            "memory_transition": "quarantine_candidate_until_verified",
            "world_delta": "no_memory_change_yet",
            "plan_transition": "open_review_or_reject_unverified_write",
            "action_affordance": "deny_direct_model_write",
            "uncertainty": "low_on_authority_boundary",
            "safety_gate": "block_direct_write",
        }
    return defaults


def evaluate(train_rows: list[dict[str, Any]], eval_rows: list[dict[str, Any]]) -> dict[str, Any]:
    results: list[dict[str, Any]] = []
    field_totals = {field: {"passed": 0, "total": 0} for field in TARGET_FIELDS}
    for row in eval_rows:
        prediction = predict(row, train_rows)
        target = row.get("target", {})
        field_results: dict[str, bool] = {}
        for field in TARGET_FIELDS:
            passed = prediction.get(field) == target.get(field)
            field_results[field] = passed
            field_totals[field]["total"] += 1
            if passed:
                field_totals[field]["passed"] += 1
        row_passed = all(field_results.values())
        results.append(
            {
                "id": row.get("id"),
                "passed": row_passed,
                "prediction": prediction,
                "target": {field: target.get(field) for field in TARGET_FIELDS},
                "field_results": field_results,
            }
        )

    total_fields = sum(bucket["total"] for bucket in field_totals.values())
    passed_fields = sum(bucket["passed"] for bucket in field_totals.values())
    return {
        "passed_fields": passed_fields,
        "total_fields": total_fields,
        "field_score": passed_fields / total_fields if total_fields else 0.0,
        "passed_rows": sum(1 for result in results if result["passed"]),
        "total_rows": len(results),
        "by_field": field_totals,
        "results": results,
    }


def write_markdown(report: dict[str, Any], output: Path) -> None:
    lines = [
        "# EDEN ELCP Baseline Report",
        "",
        "| Field | Value |",
        "| --- | --- |",
        f"| schema | `{report['schema']}` |",
        f"| training_executed | `{report['training_executed']}` |",
        f"| weights_present | `{report['weights_present']}` |",
        f"| field_score | `{report['summary']['field_score']:.3f}` |",
        f"| rows | `{report['summary']['passed_rows']}/{report['summary']['total_rows']}` |",
        "",
        "## Field Scores",
        "",
        "| Target | Passed | Total |",
        "| --- | ---: | ---: |",
    ]
    for field, bucket in report["summary"]["by_field"].items():
        lines.append(f"| `{field}` | {bucket['passed']} | {bucket['total']} |")
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Run an ELCP CPU-safe baseline.")
    parser.add_argument("--train", type=Path, default=Path("training/data/elcp_transition_train.jsonl"))
    parser.add_argument("--eval", type=Path, default=Path("training/data/elcp_transition_eval.jsonl"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_elcp/baseline_report.json"))
    parser.add_argument("--markdown-output", type=Path, default=Path("target/eden_elcp/baseline_report.md"))
    args = parser.parse_args()

    train_rows = load_jsonl(args.train)
    eval_rows = load_jsonl(args.eval)
    summary = evaluate(train_rows, eval_rows)
    report = {
        "schema": SCHEMA,
        "claim_allowed": False,
        "agi_claim": False,
        "training_executed": False,
        "weights_present": False,
        "baseline": "rule_based_cognitive_transition_safety_baseline",
        "datasets": {"train": str(args.train), "eval": str(args.eval)},
        "summary": summary,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_markdown(report, args.markdown_output)
    print(
        "ELCP baseline "
        f"{summary['passed_fields']}/{summary['total_fields']} target fields -> {args.output}"
    )
    return 0 if summary["passed_rows"] == summary["total_rows"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
