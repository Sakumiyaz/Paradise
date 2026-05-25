#!/usr/bin/env python3
"""Validate ELCP cognitive-transition JSONL fixtures.

This validator is intentionally stdlib-only. It checks shape, governance
markers and basic target consistency for Eden Latent Cognitive Prediction
without training or writing weights.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


AUTHORITY = "global_executive_workspace_core"
SCHEMA = "eden.elcp.validation_report.v1"
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
REQUIRED_GOVERNANCE_FALSE = {
    "claim_allowed",
    "agi_claim",
    "direct_memory_writes",
    "direct_objective_writes",
    "direct_tool_execution",
}


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as handle:
        for line_no, line in enumerate(handle, start=1):
            stripped = line.strip()
            if not stripped:
                continue
            try:
                item = json.loads(stripped)
            except json.JSONDecodeError as exc:
                raise ValueError(f"{path}:{line_no}: invalid JSON: {exc}") from exc
            if not isinstance(item, dict):
                raise ValueError(f"{path}:{line_no}: row must be an object")
            rows.append(item)
    return rows


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"{name} must be an object")
    return value


def require_fields(obj: dict[str, Any], fields: set[str], name: str) -> list[str]:
    return sorted(field for field in fields if field not in obj)


def validate_row(row: dict[str, Any], source: str, index: int) -> list[str]:
    errors: list[str] = []
    prefix = f"{source}[{index}]"

    for field in ("id", "input", "target", "governance"):
        if field not in row:
            errors.append(f"{prefix}.{field} is required")

    input_obj = require_object(row.get("input", {}), f"{prefix}.input")
    target_obj = require_object(row.get("target", {}), f"{prefix}.target")
    governance = require_object(row.get("governance", {}), f"{prefix}.governance")

    for missing in require_fields(input_obj, REQUIRED_INPUT, f"{prefix}.input"):
        errors.append(f"{prefix}.input.{missing} is required")
    for missing in require_fields(target_obj, REQUIRED_TARGET, f"{prefix}.target"):
        errors.append(f"{prefix}.target.{missing} is required")

    if governance.get("authority") != AUTHORITY:
        errors.append(f"{prefix}.governance.authority must be {AUTHORITY}")
    for field in REQUIRED_GOVERNANCE_FALSE:
        if governance.get(field) is not False:
            errors.append(f"{prefix}.governance.{field} must be false")

    if not isinstance(input_obj.get("working_memory"), list):
        errors.append(f"{prefix}.input.working_memory must be an array")
    if not isinstance(input_obj.get("available_tools"), list):
        errors.append(f"{prefix}.input.available_tools must be an array")
    if not isinstance(input_obj.get("world_state"), dict):
        errors.append(f"{prefix}.input.world_state must be an object")

    safety_gate = str(target_obj.get("safety_gate", ""))
    if not safety_gate:
        errors.append(f"{prefix}.target.safety_gate must be non-empty")
    if "direct" in safety_gate and "allow" in safety_gate:
        errors.append(f"{prefix}.target.safety_gate cannot allow direct authority")

    return errors


def validate_dataset(name: str, path: Path) -> tuple[list[dict[str, Any]], list[str]]:
    rows = load_jsonl(path)
    errors: list[str] = []
    if not rows:
        errors.append(f"{name} must contain at least one transition")
    ids: set[str] = set()
    for index, row in enumerate(rows):
        row_id = str(row.get("id", ""))
        if not row_id:
            errors.append(f"{name}[{index}].id must be non-empty")
        elif row_id in ids:
            errors.append(f"{name}[{index}].id duplicates {row_id}")
        ids.add(row_id)
        errors.extend(validate_row(row, name, index))
    return rows, errors


def build_report(config: dict[str, Any], train_path: Path, eval_path: Path) -> dict[str, Any]:
    train_rows, train_errors = validate_dataset("train", train_path)
    eval_rows, eval_errors = validate_dataset("eval", eval_path)
    config_errors: list[str] = []
    if config.get("objective") != "eden-latent-cognitive-prediction-v1":
        config_errors.append("config.objective must be eden-latent-cognitive-prediction-v1")
    boundaries = require_object(config.get("authority_boundaries", {}), "authority_boundaries")
    if boundaries.get("owner") != AUTHORITY:
        config_errors.append("authority_boundaries.owner must be global_executive_workspace_core")
    for field in (
        "direct_memory_writes",
        "direct_objective_writes",
        "direct_tool_execution",
    ):
        if boundaries.get(field) is not False:
            config_errors.append(f"authority_boundaries.{field} must be false")

    errors = config_errors + train_errors + eval_errors
    total = len(train_rows) + len(eval_rows)
    return {
        "schema": SCHEMA,
        "claim_allowed": False,
        "agi_claim": False,
        "training_executed": False,
        "weights_present": False,
        "objective": config.get("objective"),
        "authority": AUTHORITY,
        "datasets": {
            "train": str(train_path),
            "eval": str(eval_path),
            "train_rows": len(train_rows),
            "eval_rows": len(eval_rows),
        },
        "checks": {
            "config_valid": not config_errors,
            "train_valid": not train_errors,
            "eval_valid": not eval_errors,
            "governance_preserved": not errors,
        },
        "passed": 0 if errors else total,
        "total": total,
        "errors": errors,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ELCP transition fixtures.")
    parser.add_argument("--config", type=Path, default=Path("training/configs/elcp_latent_cognitive_prediction.json"))
    parser.add_argument("--train", type=Path, default=Path("training/data/elcp_transition_train.jsonl"))
    parser.add_argument("--eval", type=Path, default=Path("training/data/elcp_transition_eval.jsonl"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_elcp/validation_report.json"))
    args = parser.parse_args()

    config = load_json(args.config)
    report = build_report(config, args.train, args.eval)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if report["errors"]:
        for error in report["errors"]:
            print(f"ELCP validation error: {error}")
        return 1
    print(f"validated ELCP transitions: {report['passed']}/{report['total']} rows -> {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
