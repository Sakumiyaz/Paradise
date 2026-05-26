#!/usr/bin/env python3
"""Evaluate EDEN v0.2 checkpoint stability and comparison readiness."""

from __future__ import annotations

import argparse
import json
import math
from collections import Counter
from pathlib import Path
from typing import Any


SCHEMA = "eden.v02.stability_eval.v1"
COMPARISON_SCHEMA = "eden.v02.checkpoint_comparison.v1"
AUTHORITY = "global_executive_workspace_core"
TARGET_FIELDS = [
    "authority",
    "operation",
    "semantic_output_kind",
    "required_evidence",
    "requires_verification",
    "requires_rollback_plan",
    "direct_memory_write",
    "direct_tool_execution",
    "direct_objective_update",
    "production_release_allowed",
    "claim_allowed",
    "agi_claim",
]


def load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    return value if isinstance(value, dict) else {}


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    rows: list[dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            value = json.loads(line)
            if isinstance(value, dict):
                rows.append(value)
    return rows


def at(value: dict[str, Any], *path: str) -> Any:
    current: Any = value
    for key in path:
        if not isinstance(current, dict) or key not in current:
            return None
        current = current[key]
    return current


def number_at(value: dict[str, Any], *path: str) -> float:
    current = at(value, *path)
    return float(current) if isinstance(current, (int, float)) else 0.0


def bool_at(value: dict[str, Any], *path: str) -> bool | None:
    current = at(value, *path)
    return current if isinstance(current, bool) else None


def text_at(value: dict[str, Any], *path: str) -> str:
    current = at(value, *path)
    return current if isinstance(current, str) else ""


def row_valid(row: dict[str, Any]) -> bool:
    target = row.get("target", {})
    metadata = row.get("metadata", {})
    if not isinstance(target, dict) or not isinstance(metadata, dict):
        return False
    if any(field not in target for field in TARGET_FIELDS):
        return False
    if target.get("authority") != AUTHORITY:
        return False
    if target.get("semantic_output_kind") != "structured_stability_hypothesis":
        return False
    if target.get("requires_verification") is not True:
        return False
    if target.get("requires_rollback_plan") is not True:
        return False
    if target.get("direct_memory_write") is not False:
        return False
    if target.get("direct_tool_execution") is not False:
        return False
    if target.get("direct_objective_update") is not False:
        return False
    if target.get("production_release_allowed") is not False:
        return False
    if target.get("claim_allowed") is not False or target.get("agi_claim") is not False:
        return False
    if metadata.get("contains_private_data") is not False:
        return False
    if metadata.get("external_model_dependency") is not False:
        return False
    evidence = target.get("required_evidence")
    return isinstance(evidence, list) and len(evidence) >= 5


def check(name: str, passed: bool, evidence: str, weight: int = 1) -> dict[str, Any]:
    return {"check": name, "passed": passed, "evidence": evidence, "weight": weight}


def finite_positive(value: float) -> bool:
    return math.isfinite(value) and value > 0.0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--train", type=Path, default=Path("training/data/eden_v02_stability_train.jsonl"))
    parser.add_argument("--eval", type=Path, default=Path("training/data/eden_v02_stability_eval.jsonl"))
    parser.add_argument("--challenge", type=Path, default=Path("training/data/eden_v02_stability_challenge.jsonl"))
    parser.add_argument("--baseline-training", type=Path, default=Path("target/eden_v02/eden_7b_baseline_100_training_evidence.json"))
    parser.add_argument("--baseline-inference", type=Path, default=Path("target/eden_v02/eden_7b_baseline_100_inference_report.json"))
    parser.add_argument("--candidate-training", type=Path, default=Path("target/eden_v02/eden_7b_stability_250_training_evidence.json"))
    parser.add_argument("--candidate-inference", type=Path, default=Path("target/eden_v02/eden_7b_stability_250_inference_report.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_v02/stability_eval_report.json"))
    parser.add_argument("--comparison-output", type=Path, default=Path("target/eden_v02/checkpoint_comparison_report.json"))
    args = parser.parse_args()

    rows = load_jsonl(args.train) + load_jsonl(args.eval) + load_jsonl(args.challenge)
    valid_rows = sum(1 for row in rows if row_valid(row))
    task_types = Counter(row.get("input", {}).get("task_type", "unknown") for row in rows if isinstance(row.get("input"), dict))
    categories = Counter(row.get("input", {}).get("category_hint", "unknown") for row in rows if isinstance(row.get("input"), dict))

    baseline_training = load_json(args.baseline_training)
    baseline_inference = load_json(args.baseline_inference)
    candidate_training = load_json(args.candidate_training)
    candidate_inference = load_json(args.candidate_inference)

    baseline_iters = int(number_at(baseline_training, "run", "completed_iterations"))
    candidate_iters = int(number_at(candidate_training, "run", "completed_iterations"))
    baseline_loss = number_at(baseline_training, "run", "final_loss")
    candidate_loss = number_at(candidate_training, "run", "final_loss")
    baseline_params = int(number_at(baseline_training, "run", "model_parameters"))
    candidate_params = int(number_at(candidate_training, "run", "model_parameters"))
    loss_delta = candidate_loss - baseline_loss if finite_positive(baseline_loss) and finite_positive(candidate_loss) else None
    loss_ratio = candidate_loss / baseline_loss if finite_positive(baseline_loss) and finite_positive(candidate_loss) else None

    comparison_checks = [
        check("baseline_completed_100_iters", baseline_iters >= 100, str(args.baseline_training), 2),
        check("candidate_completed_250_iters", candidate_iters >= 250, str(args.candidate_training), 3),
        check("same_7b_parameter_shape", baseline_params == candidate_params and 6_900_000_000 <= candidate_params <= 14_000_000_000, "model_parameters", 1),
        check("candidate_loss_finite", finite_positive(candidate_loss) and candidate_loss < 10.0, str(args.candidate_training), 1),
        check("candidate_loss_not_regressed", loss_delta is not None and candidate_loss <= baseline_loss + 0.25, "loss_delta <= 0.25", 2),
        check("candidate_no_nan_or_skips", number_at(candidate_training, "run", "nan_iterations") == 0 and number_at(candidate_training, "run", "skipped_iterations") == 0, str(args.candidate_training), 1),
        check("candidate_checkpoint_written", bool_at(candidate_training, "checkpoint_policy", "checkpoint_written") is True, str(args.candidate_training), 1),
        check("candidate_inference_loaded", bool_at(candidate_inference, "run", "checkpoint_loaded") is True, str(args.candidate_inference), 2),
        check("candidate_inference_generated", number_at(candidate_inference, "run", "generated_count") >= 2, str(args.candidate_inference), 1),
        check("production_still_blocked", bool_at(candidate_inference, "run", "production_model") is False and bool_at(candidate_training, "checkpoint_policy", "production_model") is False, "production_model=false", 2),
        check("no_external_model_dependency", bool_at(candidate_training, "run", "external_model_dependency") is False and text_at(candidate_training, "run", "network") == "none", "network=none", 2),
    ]
    dataset_checks = [
        check("dataset_has_4096_rows", len(rows) >= 4096, "eden_v02_stability_*.jsonl", 2),
        check("dataset_targets_valid", valid_rows == len(rows) and len(rows) > 0, "target fields", 2),
        check("dataset_has_16_task_types", len(task_types) >= 16, "task type coverage", 1),
        check("dataset_has_8_plus_categories", len(categories) >= 8, "category coverage", 1),
        check("challenge_has_400_plus_rows", len(load_jsonl(args.challenge)) >= 400, str(args.challenge), 1),
    ]
    checks = dataset_checks + comparison_checks
    weighted_total = sum(item["weight"] for item in checks)
    weighted_passed = sum(item["weight"] for item in checks if item["passed"])
    score = weighted_passed / weighted_total if weighted_total else 0.0
    passed = score >= 0.98 and all(item["passed"] for item in checks)

    comparison = {
        "schema": COMPARISON_SCHEMA,
        "authority": AUTHORITY,
        "artifact": "eden_v02_checkpoint_comparison",
        "claim_allowed": False,
        "agi_claim": False,
        "passed": all(item["passed"] for item in comparison_checks),
        "baseline": {
            "iterations": baseline_iters,
            "loss": baseline_loss,
            "model_parameters": baseline_params,
            "checkpoint_loaded": bool_at(baseline_inference, "run", "checkpoint_loaded"),
            "generated_count": int(number_at(baseline_inference, "run", "generated_count")),
            "training_log_hash": text_at(baseline_training, "source", "log_fnv64"),
        },
        "candidate": {
            "iterations": candidate_iters,
            "loss": candidate_loss,
            "model_parameters": candidate_params,
            "checkpoint_loaded": bool_at(candidate_inference, "run", "checkpoint_loaded"),
            "generated_count": int(number_at(candidate_inference, "run", "generated_count")),
            "training_log_hash": text_at(candidate_training, "source", "log_fnv64"),
        },
        "loss_delta": loss_delta,
        "loss_ratio": loss_ratio,
        "checks": comparison_checks,
        "admission_scope": "candidate_runtime_only",
        "production_model_allowed": False,
    }
    args.comparison_output.parent.mkdir(parents=True, exist_ok=True)
    args.comparison_output.write_text(json.dumps(comparison, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    report = {
        "schema": SCHEMA,
        "authority": AUTHORITY,
        "artifact": "eden_v02_stability_eval",
        "claim_allowed": False,
        "agi_claim": False,
        "score": score,
        "passed": passed,
        "weighted_passed": weighted_passed,
        "weighted_total": weighted_total,
        "rows": {
            "train": len(load_jsonl(args.train)),
            "eval": len(load_jsonl(args.eval)),
            "challenge": len(load_jsonl(args.challenge)),
            "total": len(rows),
            "valid": valid_rows,
        },
        "task_types": dict(sorted(task_types.items())),
        "categories": dict(sorted(categories.items())),
        "comparison_report": str(args.comparison_output),
        "checks": checks,
        "not_measured": ["AGI", "production_release_safety", "external_benchmark_superiority"],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "EDEN v0.2 stability eval "
        f"score={score:.3f} passed={passed} rows={len(rows)} "
        f"baseline_iters={baseline_iters} candidate_iters={candidate_iters} -> {args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
