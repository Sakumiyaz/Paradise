#!/usr/bin/env python3
"""Review exported ELCP trace candidates before any dataset admission."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any


SCHEMA = "eden.elcp.trace_quality_gate.v1"
AUTHORITY = "global_executive_workspace_core"
REQUIRED_TOP = {"id", "input", "target", "governance"}
REQUIRED_INPUT = {
    "surface_text",
    "situation",
    "goal",
    "working_memory",
    "world_state",
    "plan_state",
    "available_tools",
    "risk_context",
}
REQUIRED_TARGET = {
    "next_situation",
    "next_goal_state",
    "memory_transition",
    "world_delta",
    "plan_transition",
    "action_affordance",
    "uncertainty",
    "safety_gate",
    "learning_update",
}
SENSITIVE_PATTERNS = [
    re.compile(r"\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b", re.IGNORECASE),
    re.compile(r"\b(?:\d[ -]*?){13,19}\b"),
    re.compile(r"\b(?:api[_-]?key|secret|token|password)\b", re.IGNORECASE),
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
            if not stripped:
                continue
            item = json.loads(stripped)
            if isinstance(item, dict):
                rows.append(item)
    return rows


def has_sensitive_text(value: Any) -> bool:
    if isinstance(value, str):
        return any(pattern.search(value) for pattern in SENSITIVE_PATTERNS)
    if isinstance(value, list):
        return any(has_sensitive_text(item) for item in value)
    if isinstance(value, dict):
        return any(has_sensitive_text(item) for item in value.values())
    return False


def review_candidate(row: dict[str, Any]) -> tuple[bool, list[str]]:
    reasons: list[str] = []
    if missing := sorted(REQUIRED_TOP - set(row)):
        reasons.append(f"missing_top={','.join(missing)}")

    input_obj = row.get("input", {})
    target_obj = row.get("target", {})
    governance = row.get("governance", {})
    if not isinstance(input_obj, dict):
        reasons.append("input_not_object")
        input_obj = {}
    if not isinstance(target_obj, dict):
        reasons.append("target_not_object")
        target_obj = {}
    if not isinstance(governance, dict):
        reasons.append("governance_not_object")
        governance = {}

    if missing := sorted(REQUIRED_INPUT - set(input_obj)):
        reasons.append(f"missing_input={','.join(missing)}")
    if missing := sorted(REQUIRED_TARGET - set(target_obj)):
        reasons.append(f"missing_target={','.join(missing)}")
    if governance.get("authority") != AUTHORITY:
        reasons.append("authority_mismatch")
    for field in ("claim_allowed", "agi_claim", "direct_memory_writes", "direct_objective_writes", "direct_tool_execution"):
        if governance.get(field) is not False:
            reasons.append(f"{field}_not_false")
    if has_sensitive_text(row):
        reasons.append("sensitive_pattern_detected")
    if "direct" in str(target_obj.get("safety_gate", "")) and "allow" in str(target_obj.get("safety_gate", "")):
        reasons.append("unsafe_direct_authority_safety_gate")
    return not reasons, reasons


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(row, sort_keys=True) + "\n")


def build_report(candidates: list[dict[str, Any]], trace_report: dict[str, Any], accepted_output: Path) -> dict[str, Any]:
    seen: set[str] = set()
    accepted: list[dict[str, Any]] = []
    rejected: list[dict[str, Any]] = []
    skipped_duplicates: list[str] = []
    duplicates = 0
    for row in candidates:
        key = json.dumps({"input": row.get("input"), "target": row.get("target")}, sort_keys=True)
        passed, reasons = review_candidate(row)
        if key in seen:
            duplicates += 1
            skipped_duplicates.append(str(row.get("id", "")))
            continue
        seen.add(key)
        if passed:
            accepted.append(row)
        else:
            rejected.append({"id": row.get("id"), "reasons": reasons})

    write_jsonl(accepted_output, accepted)
    checks = [
        {"check": "source_trace_export_passed", "passed": trace_report.get("schema") == "eden.elcp.trace_export.v1"},
        {"check": "candidate_rows_present", "passed": len(candidates) > 0},
        {"check": "all_candidates_reviewed", "passed": len(candidates) == len(accepted) + len(rejected) + duplicates},
        {"check": "no_rejected_candidates", "passed": not rejected},
        {"check": "accepted_output_deduplicated", "passed": len(accepted) == len(seen)},
        {"check": "accepted_output_written", "passed": accepted_output.exists()},
    ]
    passed = sum(1 for check in checks if check["passed"])
    return {
        "schema": SCHEMA,
        "artifact": "elcp_trace_quality_gate",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "training_executed": False,
        "weights_present": False,
        "source_candidate_rows": len(candidates),
        "accepted_rows": len(accepted),
        "rejected_rows": len(rejected),
        "duplicate_rows": duplicates,
        "skipped_duplicates": skipped_duplicates,
        "accepted_output": str(accepted_output),
        "passed": passed,
        "total": len(checks),
        "checks": checks,
        "rejections": rejected,
        "admission_status": "reviewed_candidates_not_training_truth",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Review ELCP trace candidates.")
    parser.add_argument("--candidates", type=Path, default=Path("target/eden_elcp/elcp_trace_candidates.jsonl"))
    parser.add_argument("--trace-report", type=Path, default=Path("target/eden_elcp/trace_export_report.json"))
    parser.add_argument("--accepted-output", type=Path, default=Path("target/eden_elcp/elcp_trace_candidates_reviewed.jsonl"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_elcp/trace_quality_gate_report.json"))
    args = parser.parse_args()

    report = build_report(load_jsonl(args.candidates), load_json(args.trace_report), args.accepted_output)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"ELCP trace quality gate {report['passed']}/{report['total']} checks -> {args.output}")
    return 0 if report["passed"] == report["total"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
