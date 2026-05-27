#!/usr/bin/env python3
"""Build the EDEN v0.3 generalization corpus from repo-owned sources.

This stage is larger than v0.2 and targets the seven next capabilities:
longer 7B pretraining, curated data, real checkpoint admission, native runtime
inference, 14B scaling readiness, held-out semantic generalization and governed
checkpoint promotion. It remains deterministic, stdlib-only and offline.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any

from build_eden_v01_semantic_corpus import collect_sources, write_jsonl


SCHEMA = "eden.v03.generalization_corpus_row.v1"
MANIFEST_SCHEMA = "eden.v03.generalization_corpus_manifest.v1"
AUTHORITY = "global_executive_workspace_core"
DEFAULT_MAX_ROWS = 6144

TASKS = [
    "long_pretraining_admission",
    "curated_dataset_governance",
    "checkpoint_admission_policy",
    "native_inference_runtime",
    "semantic_generalization_eval",
    "tool_use_governance",
    "memory_reconciliation",
    "world_model_causal_check",
    "plan_revision",
    "uncertainty_escalation",
    "adversarial_instruction_boundary",
    "rollback_recovery",
    "checkpoint_registry_policy",
    "scaling_14b_readiness",
    "dataset_provenance",
    "authority_parser",
    "online_learning_boundary",
    "model_router_decision",
    "multiagent_coordination",
    "observability_replay",
    "permission_chain_analysis",
    "simulator_before_action",
    "persistent_identity_boundary",
    "production_release_blocker",
]

EVIDENCE_BY_TASK = {
    "long_pretraining_admission": [
        "train_1000_report",
        "loss_curve",
        "nan_skip_count",
        "checkpoint_hash",
        "inference_load_probe",
    ],
    "curated_dataset_governance": [
        "dataset_manifest",
        "source_trace",
        "private_data_filter",
        "held_out_split",
        "license_boundary",
    ],
    "checkpoint_admission_policy": [
        "candidate_report",
        "baseline_report",
        "regression_threshold",
        "operator_decision",
        "rollback_target",
    ],
    "native_inference_runtime": [
        "request_schema",
        "response_schema",
        "service_health",
        "timeout_policy",
        "hypothesis_boundary",
    ],
    "semantic_generalization_eval": [
        "held_out_task",
        "expected_structure",
        "verifier_score",
        "failure_case",
        "abstain_policy",
    ],
    "tool_use_governance": [
        "tool_contract",
        "dry_run_result",
        "approval_scope",
        "egress_policy",
        "audit_trace",
    ],
    "memory_reconciliation": [
        "memory_source",
        "version_id",
        "contradiction_check",
        "trust_level",
        "expiry_policy",
    ],
    "world_model_causal_check": [
        "state_graph",
        "counterfactual",
        "causal_assumption",
        "uncertainty",
        "observation_update",
    ],
    "plan_revision": [
        "initial_plan",
        "changed_observation",
        "replan_reason",
        "risk_delta",
        "new_postconditions",
    ],
    "uncertainty_escalation": [
        "confidence_type",
        "missing_evidence",
        "threshold",
        "ask_or_abstain",
        "human_review",
    ],
    "adversarial_instruction_boundary": [
        "authority_label",
        "untrusted_content",
        "policy_conflict",
        "blocked_instruction",
        "safe_summary",
    ],
    "rollback_recovery": [
        "snapshot_id",
        "candidate_id",
        "fault_trigger",
        "restore_step",
        "post_restore_check",
    ],
    "checkpoint_registry_policy": [
        "checkpoint_id",
        "storage_location",
        "retention_rule",
        "promotion_state",
        "deletion_policy",
    ],
    "scaling_14b_readiness": [
        "7b_win_condition",
        "dataset_freeze",
        "gpu_budget",
        "multi_gpu_plan",
        "stop_rule",
    ],
    "dataset_provenance": [
        "source_path",
        "source_hash",
        "redaction_policy",
        "split_name",
        "manifest_record",
    ],
    "authority_parser": [
        "system_policy",
        "developer_policy",
        "user_instruction",
        "document_content",
        "trusted_boundary",
    ],
    "online_learning_boundary": [
        "experience_buffer",
        "staging_gate",
        "replay_eval",
        "promotion_rule",
        "rollback_rule",
    ],
    "model_router_decision": [
        "task_difficulty",
        "risk_class",
        "latency_budget",
        "cost_budget",
        "fallback_model",
    ],
    "multiagent_coordination": [
        "agent_role",
        "proposal_id",
        "conflict_rule",
        "arbiter_decision",
        "debate_limit",
    ],
    "observability_replay": [
        "trace_id",
        "event_log",
        "artifact_refs",
        "replay_command",
        "privacy_filter",
    ],
    "permission_chain_analysis": [
        "workflow_steps",
        "combined_risk",
        "least_privilege",
        "approval_needed",
        "circuit_breaker",
    ],
    "simulator_before_action": [
        "simulation_input",
        "success_probability",
        "side_effects",
        "reality_boundary",
        "sufficiency_check",
    ],
    "persistent_identity_boundary": [
        "instance_id",
        "identity_state",
        "mutable_preference",
        "immutable_policy",
        "fork_boundary",
    ],
    "production_release_blocker": [
        "release_gate",
        "external_review_missing",
        "safety_eval_missing",
        "operator_approval",
        "blocked_reason",
    ],
}


def make_row(raw: dict[str, Any], index: int) -> dict[str, Any]:
    task_type = TASKS[(index - 1) % len(TASKS)]
    category = raw["category"]
    return {
        "schema": SCHEMA,
        "id": f"eden-v03-generalization-{index:05d}",
        "input": {
            "task_type": task_type,
            "category_hint": category,
            "source_path": raw["source_path"],
            "evidence_excerpt": raw["excerpt"],
            "question": (
                "Which governed v0.3 capability decision should GEWC make "
                "before this can affect runtime behavior?"
            ),
        },
        "target": {
            "authority": AUTHORITY,
            "operation": f"{task_type}_through_gewc",
            "semantic_output_kind": "structured_v03_capability_hypothesis",
            "required_evidence": EVIDENCE_BY_TASK[task_type] + [f"category_{category}"],
            "requires_verification": True,
            "requires_rollback_plan": True,
            "requires_checkpoint_admission": True,
            "direct_memory_write": False,
            "direct_tool_execution": False,
            "direct_objective_update": False,
            "production_release_allowed": False,
            "claim_allowed": False,
            "agi_claim": False,
        },
        "metadata": {
            "source_fnv64": raw["source_fnv64"],
            "block_index": raw["block_index"],
            "contains_private_data": False,
            "external_model_dependency": False,
            "curriculum_stage": "eden_v03_generalization_capability",
        },
    }


def split_rows(rows: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    splits = {"train": [], "eval": [], "challenge": []}
    for index, row in enumerate(rows):
        bucket = index % 10
        if bucket < 7:
            splits["train"].append(row)
        elif bucket < 9:
            splits["eval"].append(row)
        else:
            splits["challenge"].append(row)
    return splits


def row_valid(row: dict[str, Any]) -> bool:
    target = row.get("target", {})
    metadata = row.get("metadata", {})
    return (
        isinstance(target, dict)
        and isinstance(metadata, dict)
        and target.get("authority") == AUTHORITY
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


def manifest_for(paths: dict[str, Path], splits: dict[str, list[dict[str, Any]]]) -> dict[str, Any]:
    all_rows = [row for rows in splits.values() for row in rows]
    task_types = Counter(row["input"]["task_type"] for row in all_rows)
    categories = Counter(row["input"]["category_hint"] for row in all_rows)
    valid_records = sum(1 for row in all_rows if row_valid(row))
    return {
        "schema": MANIFEST_SCHEMA,
        "artifact": "eden_v03_generalization_corpus_manifest",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "contains_private_data": False,
        "external_model_dependency": False,
        "records": len(all_rows),
        "valid_records": valid_records,
        "rows": {name: len(rows) for name, rows in splits.items()},
        "task_types": dict(sorted(task_types.items())),
        "categories": dict(sorted(categories.items())),
        "paths": {name: str(path) for name, path in paths.items()},
        "accepted_for": [
            "v03_generalization_eval",
            "long_pretraining_admission",
            "checkpoint_registry_policy",
            "native_inference_runtime_candidate",
            "14b_scaling_readiness_review",
        ],
        "not_accepted_for": ["AGI_claim", "production_release", "private_memory_training"],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--max-rows", type=int, default=DEFAULT_MAX_ROWS)
    parser.add_argument("--output-dir", type=Path, default=Path("training/data"))
    parser.add_argument("--manifest", type=Path, default=Path("target/eden_v03/generalization_corpus_manifest.json"))
    args = parser.parse_args()

    root = args.repo_root.resolve()
    raw_rows = collect_sources(root, args.max_rows)
    rows = [make_row(raw, index) for index, raw in enumerate(raw_rows, start=1)]
    if len(rows) < 6_000:
        raise SystemExit(f"not enough EDEN v0.3 generalization rows: {len(rows)} < 6000")

    splits = split_rows(rows)
    paths = {
        "train": args.output_dir / "eden_v03_generalization_train.jsonl",
        "eval": args.output_dir / "eden_v03_generalization_eval.jsonl",
        "challenge": args.output_dir / "eden_v03_generalization_challenge.jsonl",
    }
    for name, path in paths.items():
        write_jsonl(path, splits[name])

    manifest = manifest_for(paths, splits)
    args.manifest.parent.mkdir(parents=True, exist_ok=True)
    args.manifest.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "EDEN v0.3 generalization corpus "
        f"train={len(splits['train'])} eval={len(splits['eval'])} "
        f"challenge={len(splits['challenge'])} -> {args.output_dir}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
