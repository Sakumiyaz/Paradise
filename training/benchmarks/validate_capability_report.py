#!/usr/bin/env python3
"""Validate the EDEN training capability report without external packages."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def fail(message: str) -> None:
    raise SystemExit(f"capability report validation failed: {message}")


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(f"{name} must be an object")
    return value


def require_array(value: Any, name: str) -> list[Any]:
    if not isinstance(value, list):
        fail(f"{name} must be an array")
    return value


def require_bool(value: Any, name: str) -> bool:
    if not isinstance(value, bool):
        fail(f"{name} must be a boolean")
    return value


def require_number(value: Any, name: str) -> int | float:
    if not isinstance(value, (int, float)):
        fail(f"{name} must be numeric")
    return value


def validate_report(report: dict[str, Any], schema: dict[str, Any]) -> None:
    expected_schema = schema["properties"]["schema"]["const"]
    if report.get("schema") != expected_schema:
        fail(f"schema must be {expected_schema!r}")

    if report.get("claim_allowed") is not False:
        fail("claim_allowed must remain false")
    if report.get("agi_claim") is not False:
        fail("agi_claim must remain false")

    require_object(report.get("profile"), "profile")
    require_object(report.get("model_config"), "model_config")
    require_object(report.get("device"), "device")
    summary = require_object(report.get("summary"), "summary")
    first_model_eval = require_object(report.get("first_model_eval"), "first_model_eval")
    results = require_array(report.get("results"), "results")

    passed = int(require_number(summary.get("passed"), "summary.passed"))
    total = int(require_number(summary.get("total"), "summary.total"))
    score = float(require_number(summary.get("score"), "summary.score"))
    if total <= 0:
        fail("summary.total must be greater than zero")
    if passed < 0 or passed > total:
        fail("summary.passed must be within total")
    if not 0.0 <= score <= 1.0:
        fail("summary.score must be between 0 and 1")
    require_object(summary.get("by_capability"), "summary.by_capability")

    if len(results) != total:
        fail("results length must match summary.total")
    for index, result in enumerate(results):
        item = require_object(result, f"results[{index}]")
        for field in ("id", "kind", "capability", "observed"):
            if field not in item:
                fail(f"results[{index}].{field} is required")
        require_bool(item.get("passed"), f"results[{index}].passed")

    if first_model_eval.get("claim_allowed") is not False:
        fail("first_model_eval.claim_allowed must remain false")
    if first_model_eval.get("agi_claim") is not False:
        fail("first_model_eval.agi_claim must remain false")
    for field in (
        "direct_memory_writes",
        "direct_objective_writes",
        "direct_tool_execution",
    ):
        if first_model_eval.get(field) is not False:
            fail(f"first_model_eval.{field} must remain false")

    model_results = require_array(first_model_eval.get("results"), "first_model_eval.results")
    model_total = int(require_number(first_model_eval.get("total"), "first_model_eval.total"))
    model_passed = int(require_number(first_model_eval.get("passed"), "first_model_eval.passed"))
    if len(model_results) != model_total:
        fail("first_model_eval.results length must match total")
    if model_passed != model_total:
        fail("first_model_eval must pass all smoke cases")


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate EDEN capability report contract.")
    parser.add_argument("--report", type=Path, default=Path("target/eden_training_smoke/capability_report.json"))
    parser.add_argument("--schema", type=Path, default=Path("contracts/v1/schemas/eden-training-capability-report-v1.json"))
    args = parser.parse_args()

    report = json.loads(args.report.read_text(encoding="utf-8"))
    schema = json.loads(args.schema.read_text(encoding="utf-8"))
    validate_report(require_object(report, "report"), require_object(schema, "schema"))
    print(f"validated {args.report} against {args.schema}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
