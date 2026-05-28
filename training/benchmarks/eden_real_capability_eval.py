#!/usr/bin/env python3
"""Evaluate the current EDEN real-capability evidence surface.

This is an operational architecture/capability evaluation. It does not score
AGI. It verifies that the repo has real data, a 7B training run, inference
evidence, learned SFT/ELCP packets, safety gates and no-claim boundaries.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any


SCHEMA = "eden.real_capability.operational_eval.v1"
AUTHORITY = "global_executive_workspace_core"
TARGET_FIELDS = [
    "authority",
    "decision",
    "required_evidence",
    "safety_gate",
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
    if not isinstance(target, dict):
        return False
    if any(field not in target for field in TARGET_FIELDS):
        return False
    if target.get("authority") != AUTHORITY:
        return False
    if target.get("claim_allowed") is not False or target.get("agi_claim") is not False:
        return False
    evidence = target.get("required_evidence")
    return isinstance(evidence, list) and len(evidence) >= 2


def bool_at(value: dict[str, Any], *path: str) -> bool | None:
    current: Any = value
    for key in path:
        if not isinstance(current, dict) or key not in current:
            return None
        current = current[key]
    return current if isinstance(current, bool) else None


def number_at(value: dict[str, Any], *path: str) -> float:
    current: Any = value
    for key in path:
        if not isinstance(current, dict) or key not in current:
            return 0.0
        current = current[key]
    return float(current) if isinstance(current, (int, float)) else 0.0


def check(name: str, passed: bool, evidence: str, weight: int = 1) -> dict[str, Any]:
    return {"check": name, "passed": passed, "evidence": evidence, "weight": weight}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--train", type=Path, default=Path("training/data/eden_real_capability_train.jsonl"))
    parser.add_argument("--eval", type=Path, default=Path("training/data/eden_real_capability_eval.jsonl"))
    parser.add_argument("--challenge", type=Path, default=Path("training/data/eden_real_capability_challenge.jsonl"))
    parser.add_argument("--training-evidence", type=Path, default=Path("target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json"))
    parser.add_argument("--inference-report", type=Path, default=Path("target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json"))
    parser.add_argument("--sft-report", type=Path, default=Path("target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_training_report.json"))
    parser.add_argument("--sft-prepost", type=Path, default=Path("target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_prepost_eval.json"))
    parser.add_argument("--sft-packets", type=Path, default=Path("target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_inference_packets.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_real_capability/capability_eval_report.json"))
    args = parser.parse_args()

    train_rows = load_jsonl(args.train)
    eval_rows = load_jsonl(args.eval)
    challenge_rows = load_jsonl(args.challenge)
    all_rows = train_rows + eval_rows + challenge_rows
    valid_rows = sum(1 for row in all_rows if row_valid(row))
    categories = Counter(
        row.get("input", {}).get("category_hint", "unknown")
        for row in all_rows
        if isinstance(row.get("input"), dict)
    )

    training = load_json(args.training_evidence)
    inference = load_json(args.inference_report)
    sft_report = load_json(args.sft_report)
    sft_prepost = load_json(args.sft_prepost)
    sft_packets = load_json(args.sft_packets)
    packet_rows = sft_packets.get("packets", [])
    safe_packets = [
        packet
        for packet in packet_rows
        if isinstance(packet, dict)
        and packet.get("authority", {}).get("accepted_as_truth") is False
        and packet.get("candidate_structure", {}).get("requires_verification") is True
    ]

    checks = [
        check("dataset_has_300_plus_rows", len(all_rows) >= 300, "eden_real_capability_*.jsonl", 2),
        check("dataset_has_challenge_split", len(challenge_rows) >= 40, str(args.challenge), 1),
        check("dataset_targets_valid", valid_rows == len(all_rows) and len(all_rows) > 0, "target fields", 2),
        check("dataset_has_8_plus_categories", len(categories) >= 8, "category coverage", 1),
        check("seven_b_training_50_iters", number_at(training, "run", "completed_iterations") >= 50, str(args.training_evidence), 2),
        check("seven_b_checkpoint_written", bool_at(training, "checkpoint_policy", "checkpoint_written") is True, str(args.training_evidence), 1),
        check("seven_b_no_claim", training.get("claim_allowed") is False and training.get("agi_claim") is False, str(args.training_evidence), 1),
        check("inference_checkpoint_loaded", bool_at(inference, "run", "checkpoint_loaded") is True, str(args.inference_report), 2),
        check("inference_generated_two_plus", number_at(inference, "run", "generated_count") >= 2, str(args.inference_report), 1),
        check("sft_gpu_training_executed", sft_report.get("gpu_job_submitted") is True, str(args.sft_report), 1),
        check("sft_prepost_improved", number_at(sft_prepost, "post_eval", "field_score") > number_at(sft_prepost, "pre_eval", "field_score"), str(args.sft_prepost), 1),
        check("sft_packets_hypothesis_gated", bool(packet_rows) and len(packet_rows) == len(safe_packets), str(args.sft_packets), 2),
    ]
    weighted_total = sum(item["weight"] for item in checks)
    weighted_passed = sum(item["weight"] for item in checks if item["passed"])
    score = weighted_passed / weighted_total if weighted_total else 0.0

    report = {
        "schema": SCHEMA,
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "artifact": "eden_real_capability_operational_eval",
        "score": score,
        "passed": score >= 0.90 and all(item["passed"] for item in checks),
        "weighted_passed": weighted_passed,
        "weighted_total": weighted_total,
        "rows": {
            "train": len(train_rows),
            "eval": len(eval_rows),
            "challenge": len(challenge_rows),
            "total": len(all_rows),
            "valid": valid_rows,
        },
        "categories": dict(sorted(categories.items())),
        "checks": checks,
        "not_measured": [
            "AGI",
            "open_domain_reasoning",
            "human_preference_alignment",
            "external_benchmark_superiority",
        ],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "EDEN real capability eval "
        f"score={score:.3f} passed={report['passed']} rows={len(all_rows)} -> {args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
