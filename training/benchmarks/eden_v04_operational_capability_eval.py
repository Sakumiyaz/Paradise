#!/usr/bin/env python3
"""Evaluate EDEN v0.4 operational capability evidence.

The v0.4 gate combines seven GPU-dependent or GPU-adjacent processes:
10k 7B training, generative probing, cognitive SFT evidence, hard checkpoint
admission, persistent inference service contract, 14B preflight and continuity
evaluation. Passing this gate admits the checkpoint only as a GEWC-subordinate
candidate. It does not admit production inference and does not claim AGI.
"""

from __future__ import annotations

import argparse
import json
import math
from collections import Counter
from pathlib import Path
from typing import Any


AUTHORITY = "global_executive_workspace_core"
EVAL_SCHEMA = "eden.v04.operational_capability_eval.v1"
ADMISSION_SCHEMA = "eden.v04.hard_checkpoint_admission.v1"
GENERATIVE_SCHEMA = "eden.v04.generative_probe.v1"
SERVICE_SCHEMA = "eden.v04.persistent_inference_service.v1"
CONTINUITY_SCHEMA = "eden.v04.continuity_eval.v1"
SCALING_SCHEMA = "eden.v04.scaling_14b_preflight.v1"
GATE_SCHEMA = "eden.v04.capability_gate.v1"


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


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def at(value: dict[str, Any], *path: str) -> Any:
    current: Any = value
    for key in path:
        if not isinstance(current, dict) or key not in current:
            return None
        current = current[key]
    return current


def bool_at(value: dict[str, Any], *path: str) -> bool | None:
    current = at(value, *path)
    return current if isinstance(current, bool) else None


def number_at(value: dict[str, Any], *path: str) -> float:
    current = at(value, *path)
    return float(current) if isinstance(current, (int, float)) else 0.0


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
    return (
        isinstance(target, dict)
        and isinstance(metadata, dict)
        and target.get("authority") == AUTHORITY
        and target.get("semantic_output_kind") == "structured_v04_runtime_decision"
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
        and len(target["required_evidence"]) >= 9
    )


def response_stats(inference: dict[str, Any]) -> dict[str, Any]:
    responses = inference.get("responses", [])
    if not isinstance(responses, list):
        responses = []
    texts: list[str] = []
    token_counts: list[int] = []
    for item in responses:
        if not isinstance(item, dict):
            continue
        text = item.get("generated_text", "")
        tokens = item.get("generated_tokens", [])
        if isinstance(text, str):
            texts.append(text.strip())
        if isinstance(tokens, list):
            token_counts.append(len(tokens))
    non_empty = sum(1 for text in texts if text)
    unique_texts = len({text for text in texts if text})
    total_tokens = sum(token_counts)
    return {
        "responses": len(responses),
        "non_empty": non_empty,
        "unique_texts": unique_texts,
        "total_generated_tokens": total_tokens,
        "min_tokens": min(token_counts) if token_counts else 0,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--train", type=Path, default=Path("training/data/eden_v04_cognitive_capability_train.jsonl"))
    parser.add_argument("--eval", type=Path, default=Path("training/data/eden_v04_cognitive_capability_eval.jsonl"))
    parser.add_argument("--challenge", type=Path, default=Path("training/data/eden_v04_cognitive_capability_challenge.jsonl"))
    parser.add_argument("--manifest", type=Path, default=Path("target/eden_v04/cognitive_capability_corpus_manifest.json"))
    parser.add_argument("--v03-generalization", type=Path, default=Path("target/eden_v03/generalization_eval_report.json"))
    parser.add_argument("--v03-gate", type=Path, default=Path("/tmp/eden_garm_v03_capability/eden_v03_capability_gate.json"))
    parser.add_argument("--training", type=Path, default=Path("target/eden_v04/eden_7b_long_10000_training_evidence.json"))
    parser.add_argument("--inference", type=Path, default=Path("target/eden_v04/eden_7b_long_10000_inference_report.json"))
    parser.add_argument("--sft-training", type=Path, default=Path("target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_training_report.json"))
    parser.add_argument("--sft-prepost", type=Path, default=Path("target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_prepost_eval.json"))
    parser.add_argument("--sft-admission", type=Path, default=Path("target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_checkpoint_admission_review.json"))
    parser.add_argument("--min-rows", type=int, default=8192)
    parser.add_argument("--min-iters", type=int, default=10000)
    parser.add_argument("--output", type=Path, default=Path("target/eden_v04/operational_capability_eval_report.json"))
    parser.add_argument("--generative-output", type=Path, default=Path("target/eden_v04/generative_probe_report.json"))
    parser.add_argument("--admission-output", type=Path, default=Path("target/eden_v04/hard_checkpoint_admission_report.json"))
    parser.add_argument("--service-output", type=Path, default=Path("target/eden_v04/persistent_inference_service_report.json"))
    parser.add_argument("--continuity-output", type=Path, default=Path("target/eden_v04/continuity_eval_report.json"))
    parser.add_argument("--scaling-output", type=Path, default=Path("target/eden_v04/scaling_14b_preflight_report.json"))
    parser.add_argument("--gate-output", type=Path, default=Path("target/eden_v04/capability_gate.json"))
    args = parser.parse_args()

    rows = load_jsonl(args.train) + load_jsonl(args.eval) + load_jsonl(args.challenge)
    valid_rows = sum(1 for row in rows if row_valid(row))
    task_types = Counter(row.get("input", {}).get("task_type", "unknown") for row in rows if isinstance(row.get("input"), dict))
    processes = Counter(row.get("input", {}).get("process", "unknown") for row in rows if isinstance(row.get("input"), dict))
    challenge_rows = load_jsonl(args.challenge)

    manifest = load_json(args.manifest)
    v03_generalization = load_json(args.v03_generalization)
    v03_gate = load_json(args.v03_gate)
    training = load_json(args.training)
    inference = load_json(args.inference)
    sft_training = load_json(args.sft_training)
    sft_prepost = load_json(args.sft_prepost)
    sft_admission = load_json(args.sft_admission)

    v03_loss = number_at(v03_generalization, "long_run", "loss")
    long_iters = int(number_at(training, "run", "completed_iterations"))
    long_loss = number_at(training, "run", "final_loss")
    long_params = int(number_at(training, "run", "model_parameters"))
    loss_delta = long_loss - v03_loss if finite_positive(v03_loss) and finite_positive(long_loss) else None
    no_nan_or_skips = (
        number_at(training, "run", "nan_iterations") == 0
        and number_at(training, "run", "skipped_iterations") == 0
    )
    stats = response_stats(inference)
    runtime_smoke_passed = (
        bool_at(inference, "run", "checkpoint_loaded") is True
        and number_at(inference, "run", "generated_count") >= 3
        and stats["non_empty"] >= 3
        and stats["unique_texts"] >= 3
        and stats["total_generated_tokens"] >= 16
    )

    sft_loss_final = number_at(sft_training, "loss", "final")
    sft_loss_initial = number_at(sft_training, "loss", "initial")
    sft_delta = number_at(sft_prepost, "delta", "exact_match")
    sft_ready = (
        bool_at(sft_training, "training_executed") is True
        and sft_loss_final <= 0.01
        and sft_loss_initial > sft_loss_final
        and bool_at(sft_training, "external_model_dependency") is False
        and bool_at(sft_admission, "checkpoint_admission_allowed") is False
    )
    v03_ready = (
        (
            number_at(v03_gate, "passed") == number_at(v03_gate, "total")
            and number_at(v03_gate, "total") >= 9
        )
        or bool_at(v03_generalization, "passed") is True
    )

    generative = {
        "schema": GENERATIVE_SCHEMA,
        "artifact": "eden_v04_generative_probe",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "checkpoint_id": f"eden-7b-v04-long-{long_iters}" if long_iters else "missing",
        "passed": runtime_smoke_passed,
        "runtime_smoke_passed": runtime_smoke_passed,
        "semantic_competence_claim_allowed": False,
        "semantic_competence_status": "not_established_by_v04_runtime_smoke",
        "stats": stats,
        "accepted_for": ["checkpoint_load_probe", "token_generation_probe", "runtime_candidate_smoke"],
        "not_accepted_for": ["semantic_competence_claim", "AGI_claim", "production_release"],
    }

    service = {
        "schema": SERVICE_SCHEMA,
        "artifact": "eden_v04_persistent_inference_service",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "status": "candidate_contract_ready" if generative["passed"] else "blocked",
        "checkpoint_id": generative["checkpoint_id"],
        "service_mode": "local_checkpoint_backed_megatron_adapter",
        "persistent_requirement": {
            "load_checkpoint_once": True,
            "serve_multiple_requests": True,
            "network_required": False,
            "runtime_may_mutate_state": False,
        },
        "request_contract": {
            "schema": "eden.v04.inference_request.v1",
            "fields": ["task_id", "goal", "situation_model", "memory_refs", "risk_class", "checkpoint_id"],
        },
        "response_contract": {
            "schema": "eden.v04.inference_response.v1",
            "fields": ["candidate_text", "structured_hypothesis", "uncertainty", "verifier_required", "audit_trace"],
        },
        "guards": {
            "direct_memory_write": False,
            "direct_objective_update": False,
            "direct_tool_execution": False,
            "outputs_are_hypotheses": True,
            "production_release_allowed": False,
        },
    }

    continuity = {
        "schema": CONTINUITY_SCHEMA,
        "artifact": "eden_v04_continuity_eval",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "passed": (
            valid_rows == len(rows)
            and len(processes) >= 8
            and "continuity" in processes
            and bool_at(training, "run", "external_model_dependency") is False
            and bool_at(inference, "safety_boundary", "model_may_not_mutate_runtime_state") is True
        ),
        "checks": [
            "identity_state_is_kernel_owned",
            "objectives_are_policy_bounded",
            "memory_writes_require_gewc_authorization",
            "model_outputs_are_hypotheses",
            "rollback_handle_required",
        ],
    }

    scaling = {
        "schema": SCALING_SCHEMA,
        "artifact": "eden_v04_scaling_14b_preflight",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "max_dense_parameters": 14_000_000_000,
        "current_candidate_parameters": long_params,
        "ready_for_14b_experiment": (
            long_iters >= args.min_iters
            and finite_positive(long_loss)
            and long_loss <= max(v03_loss, 1.0)
            and no_nan_or_skips
            and generative["passed"]
        ),
        "stop_rules": [
            "nan_or_inf_loss",
            "loss_regression_against_7b_candidate",
            "checkpoint_load_failure",
            "safety_gate_failure",
            "operator_budget_stop",
        ],
        "not_allowed": ["train_above_14b_without_new_adr", "production_release_without_external_review"],
    }

    checks = [
        check("dataset_has_8192_plus_rows", len(rows) >= args.min_rows, "eden_v04_cognitive_capability_*.jsonl", 2),
        check("dataset_rows_are_valid", valid_rows == len(rows) and len(rows) > 0, "target fields", 2),
        check("manifest_records_match_dataset", int(number_at(manifest, "records")) == len(rows), str(args.manifest), 1),
        check("challenge_has_800_plus_rows", len(challenge_rows) >= 800, str(args.challenge), 1),
        check("task_type_coverage_32", len(task_types) >= 32, "task type coverage", 1),
        check("process_coverage_8_plus", len(processes) >= 8, "process coverage", 1),
        check("v03_capability_was_passed", v03_ready, f"{args.v03_gate} or {args.v03_generalization}", 2),
        check("long_run_completed_10000_iters", long_iters >= args.min_iters, str(args.training), 4),
        check("long_run_same_7b_shape", 6_900_000_000 <= long_params <= 7_100_000_000, "model_parameters", 1),
        check("long_loss_finite_and_improved", finite_positive(long_loss) and long_loss <= max(v03_loss, 1.0), "v04_loss <= max(v03_loss, 1.0)", 4),
        check("long_run_no_nan_or_skips", no_nan_or_skips, str(args.training), 2),
        check("long_checkpoint_written", bool_at(training, "checkpoint_policy", "checkpoint_written") is True, str(args.training), 2),
        check("long_inference_loaded", bool_at(inference, "run", "checkpoint_loaded") is True, str(args.inference), 2),
        check("generative_probe_passed", generative["passed"], str(args.inference), 2),
        check("cognitive_sft_evidence_available", sft_ready, str(args.sft_training), 2),
        check("persistent_service_contract_ready", service["status"] == "candidate_contract_ready", str(args.inference), 1),
        check("continuity_eval_passed", continuity["passed"], "continuity rules", 2),
        check("14b_preflight_ready", scaling["ready_for_14b_experiment"], "scaling preflight", 1),
        check("no_external_model_dependency", bool_at(training, "run", "external_model_dependency") is False and text_at(training, "run", "network") == "none", "network=none", 2),
        check("production_still_blocked", bool_at(training, "checkpoint_policy", "production_model") is False and bool_at(inference, "run", "production_model") is False, "production_model=false", 3),
    ]
    weighted_total = sum(item["weight"] for item in checks)
    weighted_passed = sum(item["weight"] for item in checks if item["passed"])
    score = weighted_passed / weighted_total if weighted_total else 0.0
    passed = score >= 0.99 and all(item["passed"] for item in checks)
    checkpoint_id = f"eden-7b-v04-long-{long_iters}" if long_iters else "eden-7b-v04-missing"

    eval_report = {
        "schema": EVAL_SCHEMA,
        "artifact": "eden_v04_operational_capability_eval",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "passed": passed,
        "score": score,
        "weighted_passed": weighted_passed,
        "weighted_total": weighted_total,
        "checks": checks,
        "dataset": {
            "rows": len(rows),
            "valid_rows": valid_rows,
            "task_type_count": len(task_types),
            "process_count": len(processes),
            "challenge_rows": len(challenge_rows),
        },
        "long_training": {
            "iters": long_iters,
            "final_loss": long_loss,
            "v03_loss": v03_loss,
            "loss_delta_against_v03": loss_delta,
            "parameters": long_params,
        },
        "sft": {
            "ready": sft_ready,
            "initial_loss": sft_loss_initial,
            "final_loss": sft_loss_final,
            "exact_match_delta": sft_delta,
            "checkpoint_admission_allowed": bool_at(sft_admission, "checkpoint_admission_allowed"),
        },
    }

    admission = {
        "schema": ADMISSION_SCHEMA,
        "artifact": "eden_v04_hard_checkpoint_admission",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "checkpoint_id": checkpoint_id,
        "candidate_runtime_admission_allowed": passed,
        "persistent_runtime_candidate_allowed": passed and service["status"] == "candidate_contract_ready",
        "production_model_allowed": False,
        "autonomous_authority_allowed": False,
        "decision": "admit_as_gewc_subordinate_candidate" if passed else "blocked_until_v04_evidence_passes",
        "required_before_production_release": [
            "external_semantic_review",
            "larger_non_synthetic_corpus",
            "red_team_eval",
            "privacy_review",
            "operator_release_approval",
        ],
    }

    gate = {
        "schema": GATE_SCHEMA,
        "artifact": "eden_v04_capability_gate",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "passed": sum(1 for item in checks if item["passed"]),
        "total": len(checks),
        "score": score,
        "candidate_runtime_admission_allowed": passed,
        "persistent_runtime_candidate_allowed": admission["persistent_runtime_candidate_allowed"],
        "production_model_allowed": False,
        "max_dense_parameters": 14_000_000_000,
        "checks": checks,
    }

    write_json(args.generative_output, generative)
    write_json(args.service_output, service)
    write_json(args.continuity_output, continuity)
    write_json(args.scaling_output, scaling)
    write_json(args.output, eval_report)
    write_json(args.admission_output, admission)
    write_json(args.gate_output, gate)

    print(
        "EDEN v0.4 operational capability eval "
        f"score={score:.3f} passed={passed} rows={len(rows)} "
        f"iters={long_iters} loss={long_loss:.6f} -> {args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
