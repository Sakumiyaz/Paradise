#!/usr/bin/env python3
"""Evaluate EDEN v0.3 long-pretraining and checkpoint admission readiness."""

from __future__ import annotations

import argparse
import json
import math
from collections import Counter
from pathlib import Path
from typing import Any


AUTHORITY = "global_executive_workspace_core"
EVAL_SCHEMA = "eden.v03.generalization_eval.v1"
ADMISSION_SCHEMA = "eden.v03.checkpoint_admission.v1"
RUNTIME_SCHEMA = "eden.v03.live_inference_runtime.v1"
REGISTRY_SCHEMA = "eden.v03.checkpoint_registry.v1"
SCALING_SCHEMA = "eden.v03.scaling_14b_plan.v1"


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


def finite_positive(value: float) -> bool:
    return math.isfinite(value) and value > 0.0


def check(name: str, passed: bool, evidence: str, weight: int = 1) -> dict[str, Any]:
    return {"check": name, "passed": passed, "evidence": evidence, "weight": weight}


def row_valid(row: dict[str, Any]) -> bool:
    target = row.get("target", {})
    metadata = row.get("metadata", {})
    if not isinstance(target, dict) or not isinstance(metadata, dict):
        return False
    required = [
        "authority",
        "operation",
        "semantic_output_kind",
        "required_evidence",
        "requires_verification",
        "requires_rollback_plan",
        "requires_checkpoint_admission",
        "direct_memory_write",
        "direct_tool_execution",
        "direct_objective_update",
        "production_release_allowed",
        "claim_allowed",
        "agi_claim",
    ]
    if any(field not in target for field in required):
        return False
    return (
        target.get("authority") == AUTHORITY
        and target.get("semantic_output_kind") == "structured_v03_capability_hypothesis"
        and target.get("requires_verification") is True
        and target.get("requires_rollback_plan") is True
        and target.get("requires_checkpoint_admission") is True
        and target.get("direct_memory_write") is False
        and target.get("direct_tool_execution") is False
        and target.get("direct_objective_update") is False
        and target.get("production_release_allowed") is False
        and target.get("claim_allowed") is False
        and target.get("agi_claim") is False
        and metadata.get("contains_private_data") is False
        and metadata.get("external_model_dependency") is False
        and isinstance(target.get("required_evidence"), list)
        and len(target["required_evidence"]) >= 6
    )


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--train", type=Path, default=Path("training/data/eden_v03_generalization_train.jsonl"))
    parser.add_argument("--eval", type=Path, default=Path("training/data/eden_v03_generalization_eval.jsonl"))
    parser.add_argument("--challenge", type=Path, default=Path("training/data/eden_v03_generalization_challenge.jsonl"))
    parser.add_argument("--manifest", type=Path, default=Path("target/eden_v03/generalization_corpus_manifest.json"))
    parser.add_argument("--v02-comparison", type=Path, default=Path("target/eden_v02/checkpoint_comparison_report.json"))
    parser.add_argument("--v02-adversarial", type=Path, default=Path("target/eden_v02/adversarial_eval_report.json"))
    parser.add_argument("--v02-rollback", type=Path, default=Path("target/eden_v02/rollback_drill_report.json"))
    parser.add_argument("--v02-model-card", type=Path, default=Path("target/eden_v02/model_card_internal.json"))
    parser.add_argument("--long-training", type=Path, default=Path("target/eden_v03/eden_7b_long_1000_training_evidence.json"))
    parser.add_argument("--long-inference", type=Path, default=Path("target/eden_v03/eden_7b_long_1000_inference_report.json"))
    parser.add_argument("--min-rows", type=int, default=6144)
    parser.add_argument("--min-iters", type=int, default=1000)
    parser.add_argument("--output", type=Path, default=Path("target/eden_v03/generalization_eval_report.json"))
    parser.add_argument("--admission-output", type=Path, default=Path("target/eden_v03/checkpoint_admission_report.json"))
    parser.add_argument("--runtime-output", type=Path, default=Path("target/eden_v03/live_inference_runtime_report.json"))
    parser.add_argument("--registry-output", type=Path, default=Path("target/eden_v03/checkpoint_registry.json"))
    parser.add_argument("--scaling-output", type=Path, default=Path("target/eden_v03/scaling_14b_plan.json"))
    args = parser.parse_args()

    rows = load_jsonl(args.train) + load_jsonl(args.eval) + load_jsonl(args.challenge)
    valid_rows = sum(1 for row in rows if row_valid(row))
    task_types = Counter(row.get("input", {}).get("task_type", "unknown") for row in rows if isinstance(row.get("input"), dict))
    categories = Counter(row.get("input", {}).get("category_hint", "unknown") for row in rows if isinstance(row.get("input"), dict))
    challenge_rows = load_jsonl(args.challenge)

    manifest = load_json(args.manifest)
    v02_comparison = load_json(args.v02_comparison)
    v02_adversarial = load_json(args.v02_adversarial)
    v02_rollback = load_json(args.v02_rollback)
    v02_model_card = load_json(args.v02_model_card)
    long_training = load_json(args.long_training)
    long_inference = load_json(args.long_inference)

    prior_loss = number_at(v02_comparison, "candidate", "loss")
    long_iters = int(number_at(long_training, "run", "completed_iterations"))
    long_loss = number_at(long_training, "run", "final_loss")
    long_params = int(number_at(long_training, "run", "model_parameters"))
    loss_delta = long_loss - prior_loss if finite_positive(prior_loss) and finite_positive(long_loss) else None
    loss_ratio = long_loss / prior_loss if finite_positive(prior_loss) and finite_positive(long_loss) else None
    no_nan_or_skips = (
        number_at(long_training, "run", "nan_iterations") == 0
        and number_at(long_training, "run", "skipped_iterations") == 0
    )

    checks = [
        check("dataset_has_6144_plus_rows", len(rows) >= args.min_rows, "eden_v03_generalization_*.jsonl", 2),
        check("dataset_rows_are_valid", valid_rows == len(rows) and len(rows) > 0, "target fields", 2),
        check("manifest_records_match_dataset", int(number_at(manifest, "records")) == len(rows), str(args.manifest), 1),
        check("challenge_has_600_plus_rows", len(challenge_rows) >= 600, str(args.challenge), 1),
        check("task_type_coverage_24", len(task_types) >= 24, "task type coverage", 1),
        check("category_coverage_8_plus", len(categories) >= 8, "category coverage", 1),
        check("v02_candidate_is_available", bool_at(v02_comparison, "passed") is True, str(args.v02_comparison), 1),
        check("v02_safety_chain_passed", bool_at(v02_adversarial, "passed") is True and bool_at(v02_rollback, "passed") is True, "v02 safety artifacts", 2),
        check("v02_limits_disclosed", bool_at(v02_model_card, "passed") is True, str(args.v02_model_card), 1),
        check("long_run_completed_1000_iters", long_iters >= args.min_iters, str(args.long_training), 3),
        check("long_run_same_7b_shape", 6_900_000_000 <= long_params <= 7_100_000_000, "model_parameters", 1),
        check("long_loss_finite", finite_positive(long_loss) and long_loss < 10.0, str(args.long_training), 1),
        check("long_loss_not_regressed_against_v02", loss_delta is not None and long_loss <= prior_loss + 0.25, "long_loss <= v02_loss + 0.25", 3),
        check("long_run_no_nan_or_skips", no_nan_or_skips, str(args.long_training), 1),
        check("long_checkpoint_written", bool_at(long_training, "checkpoint_policy", "checkpoint_written") is True, str(args.long_training), 2),
        check("long_inference_loaded", bool_at(long_inference, "run", "checkpoint_loaded") is True, str(args.long_inference), 2),
        check("long_inference_generated", number_at(long_inference, "run", "generated_count") >= 2, str(args.long_inference), 1),
        check("no_external_model_dependency", bool_at(long_training, "run", "external_model_dependency") is False and text_at(long_training, "run", "network") == "none", "network=none", 2),
        check("production_still_blocked", bool_at(long_training, "checkpoint_policy", "production_model") is False and bool_at(long_inference, "run", "production_model") is False, "production_model=false", 2),
    ]
    weighted_total = sum(item["weight"] for item in checks)
    weighted_passed = sum(item["weight"] for item in checks if item["passed"])
    score = weighted_passed / weighted_total if weighted_total else 0.0
    passed = score >= 0.985 and all(item["passed"] for item in checks)

    checkpoint_id = f"eden-7b-v03-long-{long_iters}" if long_iters else "eden-7b-v03-long-missing"
    admission_allowed = passed
    admission = {
        "schema": ADMISSION_SCHEMA,
        "artifact": "eden_v03_checkpoint_admission",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "checkpoint_id": checkpoint_id,
        "candidate_runtime_admission_allowed": admission_allowed,
        "production_model_allowed": False,
        "autonomous_authority_allowed": False,
        "decision": "admit_as_gewc_subordinate_candidate" if admission_allowed else "blocked_until_v03_evidence_passes",
        "evidence": {
            "long_training": str(args.long_training),
            "long_inference": str(args.long_inference),
            "generalization_eval": str(args.output),
            "v02_safety_chain": [str(args.v02_adversarial), str(args.v02_rollback), str(args.v02_model_card)],
        },
        "loss_delta_against_v02_candidate": loss_delta,
        "loss_ratio_against_v02_candidate": loss_ratio,
        "required_before_production_release": [
            "external_review",
            "larger_non_synthetic_corpus",
            "red-team_eval",
            "privacy_review",
            "operator_release_approval",
        ],
    }

    runtime = {
        "schema": RUNTIME_SCHEMA,
        "artifact": "eden_v03_live_inference_runtime",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "status": "candidate_ready" if admission_allowed else "blocked",
        "checkpoint_id": checkpoint_id,
        "persistent_service_candidate": admission_allowed,
        "service_scope": "local_candidate_runtime_only",
        "request_contract": {
            "schema": "eden.v03.inference_request.v1",
            "fields": [
                "task_id",
                "goal",
                "situation_model",
                "memory_refs",
                "risk_class",
                "checkpoint_id",
                "rollback_target",
            ],
        },
        "response_contract": {
            "schema": "eden.v03.inference_response.v1",
            "fields": [
                "candidate_text",
                "structured_hypothesis",
                "uncertainty",
                "verifier_required",
                "audit_trace",
                "rollback_handle",
            ],
        },
        "guards": {
            "direct_memory_write": False,
            "direct_objective_update": False,
            "direct_tool_execution": False,
            "production_release_allowed": False,
            "outputs_are_hypotheses": True,
        },
    }

    registry = {
        "schema": REGISTRY_SCHEMA,
        "artifact": "eden_v03_checkpoint_registry",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "registry_policy": {
            "weights_committed_to_repo": False,
            "promotion_requires_gewc_gate": True,
            "production_release_requires_external_review": True,
            "purge_from_ephemeral_gpu_vm_allowed_after_evidence_copy": True,
        },
        "checkpoints": [
            {
                "id": "eden-7b-v02-stability-250",
                "source": str(args.v02_comparison),
                "role": "previous_candidate",
                "production_model_allowed": False,
            },
            {
                "id": checkpoint_id,
                "source": str(args.long_training),
                "role": "v03_long_candidate",
                "iterations": long_iters,
                "loss": long_loss if finite_positive(long_loss) else None,
                "runtime_admitted": admission_allowed,
                "production_model_allowed": False,
            },
        ],
    }

    scaling = {
        "schema": SCALING_SCHEMA,
        "artifact": "eden_v03_scaling_14b_plan",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "current_dense_ceiling": 14_000_000_000,
        "current_checkpoint": checkpoint_id,
        "may_start_14b_prototype": admission_allowed,
        "must_not_exceed_dense_parameters": 14_000_000_000,
        "required_before_14b": [
            "v03_generalization_eval_passed",
            "checkpoint_registry_clean",
            "dataset_freeze",
            "budget_approval",
            "multi_gpu_megatron_plan",
        ],
        "stop_rules": [
            "loss_nan_or_skip_detected",
            "semantic_regression",
            "safety_chain_failure",
            "operator_budget_limit",
        ],
    }

    report = {
        "schema": EVAL_SCHEMA,
        "artifact": "eden_v03_generalization_eval",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "score": score,
        "passed": passed,
        "weighted_passed": weighted_passed,
        "weighted_total": weighted_total,
        "rows": {
            "train": len(load_jsonl(args.train)),
            "eval": len(load_jsonl(args.eval)),
            "challenge": len(challenge_rows),
            "total": len(rows),
            "valid": valid_rows,
        },
        "task_types": dict(sorted(task_types.items())),
        "categories": dict(sorted(categories.items())),
        "long_run": {
            "iterations": long_iters,
            "loss": long_loss if finite_positive(long_loss) else None,
            "prior_v02_loss": prior_loss if finite_positive(prior_loss) else None,
            "loss_delta": loss_delta,
            "loss_ratio": loss_ratio,
            "model_parameters": long_params,
        },
        "checks": checks,
        "admission_report": str(args.admission_output),
        "runtime_report": str(args.runtime_output),
        "checkpoint_registry": str(args.registry_output),
        "scaling_14b_plan": str(args.scaling_output),
        "not_measured": [
            "AGI",
            "open-ended autonomy",
            "external benchmark superiority",
            "production model safety",
        ],
    }

    write_json(args.output, report)
    write_json(args.admission_output, admission)
    write_json(args.runtime_output, runtime)
    write_json(args.registry_output, registry)
    write_json(args.scaling_output, scaling)
    print(
        "EDEN v0.3 generalization eval "
        f"score={score:.3f} passed={passed} rows={len(rows)} "
        f"iters={long_iters} loss={long_loss:.6f} -> {args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
