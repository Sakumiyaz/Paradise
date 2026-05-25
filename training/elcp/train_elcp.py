#!/usr/bin/env python3
"""Dry-run training interface for Eden Latent Cognitive Prediction.

This script deliberately refuses to train unless future work adds a separate
explicit 4B execution path. Today it validates inputs, estimates schedule
shape and writes a no-weights dry-run report.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.elcp.training_dry_run.v1"
AUTHORITY = "global_executive_workspace_core"


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def count_jsonl(path: Path) -> int:
    with path.open("r", encoding="utf-8") as handle:
        return sum(1 for line in handle if line.strip())


def build_report(config: dict[str, Any], validation: dict[str, Any], baseline: dict[str, Any]) -> dict[str, Any]:
    datasets = config.get("datasets", {})
    train_path = Path(str(datasets.get("train", "")))
    eval_path = Path(str(datasets.get("eval", "")))
    train_rows = count_jsonl(train_path) if train_path.exists() else 0
    eval_rows = count_jsonl(eval_path) if eval_path.exists() else 0
    target_count = len(config.get("prediction_targets", []))
    batch_size = max(min(train_rows, 8), 1)
    estimated_steps = max(train_rows, 1)
    return {
        "schema": SCHEMA,
        "claim_allowed": False,
        "agi_claim": False,
        "authority": AUTHORITY,
        "objective": config.get("objective"),
        "mode": "dry_run_only",
        "training_executed": False,
        "weights_present": False,
        "weights_written": False,
        "gpu_job_submitted": False,
        "checkpoint_admitted": False,
        "datasets": {
            "train": str(train_path),
            "eval": str(eval_path),
            "train_rows": train_rows,
            "eval_rows": eval_rows,
        },
        "schedule_shape": {
            "backend": config.get("backend_plan", {}).get("future"),
            "batch_size": batch_size,
            "estimated_steps": estimated_steps,
            "target_count": target_count,
            "loss_terms": [
                "L_token",
                "L_situation_state",
                "L_goal_state",
                "L_memory_transition",
                "L_world_delta",
                "L_plan_transition",
                "L_action_affordance",
                "L_risk_calibration",
                "L_uncertainty",
                "L_safety_gate",
            ],
        },
        "input_evidence": {
            "validation_report_schema": validation.get("schema"),
            "validation_errors": len(validation.get("errors", [])),
            "baseline_report_schema": baseline.get("schema"),
            "baseline_field_score": baseline.get("summary", {}).get("field_score"),
        },
        "blocked_until_4b": [
            "explicit_operator_approval_required",
            "gpu_budget_approval_required",
            "dataset_freeze_required",
            "checkpoint_output_dir_required",
            "post_train_eval_required",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Prepare ELCP training without executing training.")
    parser.add_argument("--dry-run", action="store_true", help="Required. Writes a dry-run report only.")
    parser.add_argument("--config", type=Path, default=Path("training/configs/elcp_latent_cognitive_prediction.json"))
    parser.add_argument("--validation-report", type=Path, default=Path("target/eden_elcp/validation_report.json"))
    parser.add_argument("--baseline-report", type=Path, default=Path("target/eden_elcp/baseline_report.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_elcp/training_dry_run.json"))
    args = parser.parse_args()

    if not args.dry_run:
        raise SystemExit("ELCP training is blocked: rerun with --dry-run for 4B-prep evidence only")

    config = load_json(args.config)
    validation = load_json(args.validation_report)
    baseline = load_json(args.baseline_report)
    report = build_report(config, validation, baseline)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"ELCP dry-run prepared; training_executed=false weights_written=false -> {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
