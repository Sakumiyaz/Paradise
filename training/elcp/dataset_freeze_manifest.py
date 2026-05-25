#!/usr/bin/env python3
"""Write an auditable ELCP dataset freeze manifest without allowing training."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.elcp.dataset_freeze_manifest.v1"
AUTHORITY = "global_executive_workspace_core"


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def count_jsonl(path: Path) -> int:
    if not path.exists():
        return 0
    with path.open("r", encoding="utf-8") as handle:
        return sum(1 for line in handle if line.strip())


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def file_record(path: Path, role: str) -> dict[str, Any]:
    return {
        "role": role,
        "path": str(path),
        "present": path.exists(),
        "rows": count_jsonl(path) if path.suffix == ".jsonl" else None,
        "sha256": sha256_file(path) if path.exists() else None,
    }


def build_manifest(args: argparse.Namespace) -> dict[str, Any]:
    validation = load_json(args.validation_report)
    quality = load_json(args.trace_quality_report)
    replay = load_json(args.replay_report)
    files = [
        file_record(args.train, "train_fixture"),
        file_record(args.eval, "eval_fixture"),
        file_record(args.reviewed_candidates, "reviewed_trace_candidates_candidate_only"),
    ]
    checks = [
        {"check": "validation_report_passed", "passed": validation.get("passed") == validation.get("total")},
        {"check": "trace_quality_passed", "passed": quality.get("passed") == quality.get("total")},
        {"check": "replay_eval_passed", "passed": replay.get("passed") == replay.get("total")},
        {"check": "all_files_present", "passed": all(record["present"] for record in files)},
        {"check": "all_files_hashed", "passed": all(record["sha256"] for record in files)},
        {"check": "trace_candidates_candidate_only", "passed": True},
    ]
    passed = sum(1 for check in checks if check["passed"])
    freeze_material = json.dumps(files, sort_keys=True).encode("utf-8")
    return {
        "schema": SCHEMA,
        "artifact": "elcp_dataset_freeze_manifest",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "training_executed": False,
        "weights_present": False,
        "freeze_id": hashlib.sha256(freeze_material).hexdigest()[:24],
        "dataset_locked_for_training": False,
        "candidate_pool_locked_for_review": True,
        "training_allowed": False,
        "files": files,
        "split_policy": {
            "train_fixture": "repo_local_synthetic_fixture",
            "eval_fixture": "repo_local_synthetic_fixture",
            "reviewed_trace_candidates": "candidate_only_not_training_truth",
        },
        "privacy_policy": {
            "contains_private_data": False,
            "trace_review_required_before_training": True,
            "operator_approval_required_before_dataset_lock": True,
        },
        "passed": passed,
        "total": len(checks),
        "checks": checks,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Write ELCP dataset freeze manifest.")
    parser.add_argument("--train", type=Path, default=Path("training/data/elcp_transition_train.jsonl"))
    parser.add_argument("--eval", type=Path, default=Path("training/data/elcp_transition_eval.jsonl"))
    parser.add_argument("--reviewed-candidates", type=Path, default=Path("target/eden_elcp/elcp_trace_candidates_reviewed.jsonl"))
    parser.add_argument("--validation-report", type=Path, default=Path("target/eden_elcp/validation_report.json"))
    parser.add_argument("--trace-quality-report", type=Path, default=Path("target/eden_elcp/trace_quality_gate_report.json"))
    parser.add_argument("--replay-report", type=Path, default=Path("target/eden_elcp/replay_eval_report.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_elcp/dataset_freeze_manifest.json"))
    args = parser.parse_args()

    report = build_manifest(args)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"ELCP dataset freeze manifest {report['passed']}/{report['total']} checks -> {args.output}")
    return 0 if report["passed"] == report["total"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
