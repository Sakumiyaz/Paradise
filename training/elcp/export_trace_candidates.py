#!/usr/bin/env python3
"""Export GEWC runtime traces into ELCP candidate transitions.

The exporter is conservative: it treats runtime traces as candidate training
data only, preserves no-claim markers and writes a redacted JSONL file for
future operator review.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


AUTHORITY = "global_executive_workspace_core"
SCHEMA = "eden.elcp.trace_export.v1"


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    if not path.exists():
        return rows
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            stripped = line.strip()
            if not stripped:
                continue
            try:
                item = json.loads(stripped)
            except json.JSONDecodeError:
                continue
            if isinstance(item, dict):
                rows.append(item)
    return rows


def candidate_from_trace(index: int, trace: dict[str, Any]) -> dict[str, Any]:
    route = str(trace.get("route") or trace.get("command") or trace.get("last_route") or "unknown")
    handler = str(trace.get("handler") or trace.get("last_handler") or "unknown")
    lifecycle = str(trace.get("lifecycle") or trace.get("last_lifecycle_state") or "observed")
    blocked = bool(trace.get("blocked") or trace.get("safety_blocked") or False)
    safety_gate = "block_and_audit" if blocked else "allow_governed_runtime_step"
    return {
        "id": f"elcp_trace_{index:04d}",
        "input": {
            "surface_text": f"GEWC routed {route} through {handler}.",
            "situation": "runtime_trace_observed",
            "goal": "preserve_governed_runtime_continuity",
            "working_memory": [
                "GEWC remains final authority",
                "runtime traces are candidate evidence only",
            ],
            "world_state": {
                "route": route,
                "handler": handler,
                "lifecycle": lifecycle,
            },
            "plan_state": "runtime_step_recorded",
            "available_tools": ["audit_log", "replay", "artifact_api"],
            "risk_context": "blocked_runtime_step" if blocked else "low_governed_runtime_step",
        },
        "target": {
            "next_situation": "runtime_trace_candidate_ready_for_review",
            "next_goal_state": "audit_before_training_admission",
            "memory_transition": "store_as_candidate_trace_not_persistent_training_truth",
            "world_delta": "no_external_action_from_export",
            "plan_transition": "review_redact_validate_then_optionally_admit",
            "action_affordance": "offline_review_only",
            "uncertainty": "medium_until_operator_review",
            "safety_gate": safety_gate,
            "learning_update": "candidate_trace_requires_validation_before_training",
        },
        "governance": {
            "authority": AUTHORITY,
            "claim_allowed": False,
            "agi_claim": False,
            "direct_memory_writes": False,
            "direct_objective_writes": False,
            "direct_tool_execution": False,
        },
    }


def export_candidates(state_dir: Path, output: Path, report_output: Path, limit: int) -> dict[str, Any]:
    trace_path = state_dir / "global_executive_workspace_runtime.jsonl"
    traces = load_jsonl(trace_path)
    candidates = [candidate_from_trace(index + 1, trace) for index, trace in enumerate(traces[:limit])]
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", encoding="utf-8") as handle:
        for candidate in candidates:
            handle.write(json.dumps(candidate, sort_keys=True) + "\n")

    report = {
        "schema": SCHEMA,
        "claim_allowed": False,
        "agi_claim": False,
        "training_executed": False,
        "weights_present": False,
        "authority": AUTHORITY,
        "state_dir": str(state_dir),
        "source_trace": str(trace_path),
        "source_trace_present": trace_path.exists(),
        "source_trace_rows": len(traces),
        "candidate_rows": len(candidates),
        "output": str(output),
        "admission_status": "candidate_only_requires_review",
    }
    report_output.parent.mkdir(parents=True, exist_ok=True)
    report_output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description="Export GEWC traces into ELCP candidate transitions.")
    parser.add_argument("--state-dir", type=Path, default=Path("/tmp/eden_garm_elcp_prepare"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_elcp/elcp_trace_candidates.jsonl"))
    parser.add_argument("--report-output", type=Path, default=Path("target/eden_elcp/trace_export_report.json"))
    parser.add_argument("--limit", type=int, default=128)
    args = parser.parse_args()

    report = export_candidates(args.state_dir, args.output, args.report_output, args.limit)
    print(
        "exported ELCP trace candidates "
        f"{report['candidate_rows']}/{report['source_trace_rows']} -> {args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
