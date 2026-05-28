#!/usr/bin/env python3
"""Evaluate EDEN v0.1 semantic capability readiness.

This is not an AGI benchmark. It checks whether the repo has enough governed
semantic training/eval material, whether the 7B run moved beyond the 50-step
pilot, whether checkpoint inference still works, and whether all outputs remain
subordinate to GEWC verification.
"""

from __future__ import annotations

import argparse
import json
import math
from collections import Counter
from pathlib import Path
from typing import Any


SCHEMA = "eden.v01.semantic_eval.v1"
AUTHORITY = "global_executive_workspace_core"
TARGET_FIELDS = [
    "authority",
    "operation",
    "semantic_output_kind",
    "required_evidence",
    "requires_verification",
    "direct_memory_write",
    "direct_tool_execution",
    "direct_objective_update",
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


def row_valid(row: dict[str, Any]) -> bool:
    target = row.get("target", {})
    metadata = row.get("metadata", {})
    if not isinstance(target, dict) or not isinstance(metadata, dict):
        return False
    if any(field not in target for field in TARGET_FIELDS):
        return False
    if target.get("authority") != AUTHORITY:
        return False
    if target.get("semantic_output_kind") != "structured_hypothesis":
        return False
    if target.get("requires_verification") is not True:
        return False
    if target.get("direct_memory_write") is not False:
        return False
    if target.get("direct_tool_execution") is not False:
        return False
    if target.get("direct_objective_update") is not False:
        return False
    if target.get("claim_allowed") is not False or target.get("agi_claim") is not False:
        return False
    if metadata.get("contains_private_data") is not False:
        return False
    if metadata.get("external_model_dependency") is not False:
        return False
    evidence = target.get("required_evidence")
    return isinstance(evidence, list) and len(evidence) >= 4


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


def check(name: str, passed: bool, evidence: str, weight: int = 1) -> dict[str, Any]:
    return {"check": name, "passed": passed, "evidence": evidence, "weight": weight}


def loss_is_reasonable(training: dict[str, Any]) -> bool:
    final_loss = number_at(training, "run", "final_loss")
    return math.isfinite(final_loss) and 0.0 < final_loss < 10.0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--train", type=Path, default=Path("training/data/eden_v01_semantic_train.jsonl"))
    parser.add_argument("--eval", type=Path, default=Path("training/data/eden_v01_semantic_eval.jsonl"))
    parser.add_argument("--challenge", type=Path, default=Path("training/data/eden_v01_semantic_challenge.jsonl"))
    parser.add_argument("--min-rows", type=int, default=2048)
    parser.add_argument("--min-iters", type=int, default=100)
    parser.add_argument("--max-dense-params", type=int, default=14_000_000_000)
    parser.add_argument("--training-evidence", type=Path, default=Path("target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json"))
    parser.add_argument("--inference-report", type=Path, default=Path("target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json"))
    parser.add_argument("--sft-packets", type=Path, default=Path("target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_inference_packets.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_v01/semantic_eval_report.json"))
    args = parser.parse_args()

    train_rows = load_jsonl(args.train)
    eval_rows = load_jsonl(args.eval)
    challenge_rows = load_jsonl(args.challenge)
    all_rows = train_rows + eval_rows + challenge_rows
    valid_rows = sum(1 for row in all_rows if row_valid(row))
    task_types = Counter(
        row.get("input", {}).get("task_type", "unknown")
        for row in all_rows
        if isinstance(row.get("input"), dict)
    )
    categories = Counter(
        row.get("input", {}).get("category_hint", "unknown")
        for row in all_rows
        if isinstance(row.get("input"), dict)
    )

    training = load_json(args.training_evidence)
    inference = load_json(args.inference_report)
    sft_packets = load_json(args.sft_packets)
    packets = sft_packets.get("packets", [])
    safe_packets = [
        packet
        for packet in packets
        if isinstance(packet, dict)
        and packet.get("authority", {}).get("accepted_as_truth") is False
        and packet.get("candidate_structure", {}).get("requires_verification") is True
    ]
    completed_iterations = int(number_at(training, "run", "completed_iterations"))
    model_parameters = int(number_at(training, "run", "model_parameters"))

    checks = [
        check("dataset_has_2048_plus_rows", len(all_rows) >= args.min_rows, "eden_v01_semantic_*.jsonl", 2),
        check("dataset_has_challenge_split", len(challenge_rows) >= 200, str(args.challenge), 1),
        check("dataset_targets_valid", valid_rows == len(all_rows) and len(all_rows) > 0, "target fields", 2),
        check("dataset_has_12_task_types", len(task_types) >= 12, "task type coverage", 1),
        check("dataset_has_8_plus_categories", len(categories) >= 8, "category coverage", 1),
        check("training_beyond_pilot_iters", completed_iterations >= args.min_iters, str(args.training_evidence), 3),
        check("training_checkpoint_written", bool_at(training, "checkpoint_policy", "checkpoint_written") is True, str(args.training_evidence), 1),
        check("training_no_external_model", bool_at(training, "run", "external_model_dependency") is False, str(args.training_evidence), 1),
        check("training_network_none", at(training, "run", "network") == "none", str(args.training_evidence), 1),
        check("training_loss_reasonable", loss_is_reasonable(training), str(args.training_evidence), 1),
        check("model_within_14b_dense_ceiling", 6_900_000_000 <= model_parameters <= args.max_dense_params, str(args.training_evidence), 1),
        check("inference_checkpoint_loaded", bool_at(inference, "run", "checkpoint_loaded") is True, str(args.inference_report), 2),
        check("inference_generated_two_plus", number_at(inference, "run", "generated_count") >= 2, str(args.inference_report), 1),
        check("sft_packets_hypothesis_gated", bool(packets) and len(packets) == len(safe_packets), str(args.sft_packets), 1),
        check("claim_boundary_preserved", training.get("claim_allowed") is False and inference.get("claim_allowed") is False, "claim_allowed=false", 1),
    ]
    weighted_total = sum(item["weight"] for item in checks)
    weighted_passed = sum(item["weight"] for item in checks if item["passed"])
    score = weighted_passed / weighted_total if weighted_total else 0.0
    passed = score >= 0.95 and all(item["passed"] for item in checks)

    report = {
        "schema": SCHEMA,
        "authority": AUTHORITY,
        "artifact": "eden_v01_semantic_eval",
        "claim_allowed": False,
        "agi_claim": False,
        "score": score,
        "passed": passed,
        "weighted_passed": weighted_passed,
        "weighted_total": weighted_total,
        "rows": {
            "train": len(train_rows),
            "eval": len(eval_rows),
            "challenge": len(challenge_rows),
            "total": len(all_rows),
            "valid": valid_rows,
        },
        "task_types": dict(sorted(task_types.items())),
        "categories": dict(sorted(categories.items())),
        "training": {
            "completed_iterations": completed_iterations,
            "model_parameters": model_parameters,
            "final_loss": number_at(training, "run", "final_loss"),
            "checkpoint_written": bool_at(training, "checkpoint_policy", "checkpoint_written"),
        },
        "inference": {
            "checkpoint_loaded": bool_at(inference, "run", "checkpoint_loaded"),
            "generated_count": int(number_at(inference, "run", "generated_count")),
        },
        "checks": checks,
        "not_measured": [
            "AGI",
            "human-level autonomy",
            "external benchmark superiority",
            "production release safety",
        ],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "EDEN v0.1 semantic eval "
        f"score={score:.3f} passed={passed} rows={len(all_rows)} "
        f"iters={completed_iterations} -> {args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
