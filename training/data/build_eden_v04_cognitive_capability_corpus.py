#!/usr/bin/env python3
"""Build the EDEN v0.4 cognitive capability corpus from repo-owned sources.

v0.4 moves beyond "checkpoint loads and emits tokens" and prepares examples for
seven governed capability processes: longer 7B training, generative evaluation,
cognitive fine-tuning, hard checkpoint admission, persistent inference service,
14B preflight and continuity tests. The corpus is deterministic, stdlib-only and
offline; it is not a claim of AGI capability.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any

from build_eden_v01_semantic_corpus import collect_sources, write_jsonl


SCHEMA = "eden.v04.cognitive_capability_corpus_row.v1"
MANIFEST_SCHEMA = "eden.v04.cognitive_capability_corpus_manifest.v1"
AUTHORITY = "global_executive_workspace_core"
DEFAULT_MAX_ROWS = 8192

TASKS = [
    "long_pretraining_10k_admission",
    "generative_semantic_probe",
    "cognitive_finetuning_curriculum",
    "hard_checkpoint_admission",
    "persistent_inference_service",
    "scaling_14b_preflight",
    "continuity_memory_objective_eval",
    "tool_use_dry_run_policy",
    "memory_write_authorization",
    "objective_conflict_resolution",
    "world_model_counterfactual",
    "causal_plan_revision",
    "uncertainty_abstention",
    "authority_boundary_parse",
    "prompt_injection_quarantine",
    "rollback_transaction_design",
    "multiagent_arbiter_decision",
    "model_router_cost_risk",
    "observability_replay_trace",
    "checkpoint_retention_policy",
    "private_data_training_filter",
    "semantic_cache_boundary",
    "simulation_before_action",
    "human_approval_escalation",
    "runtime_degradation_safe_mode",
    "distributed_memory_consistency",
    "skill_promotion_staging",
    "self_modification_review",
    "identity_fork_boundary",
    "physical_action_limit",
    "external_tool_compromise_check",
    "production_release_blocker",
]

PROCESS_BY_TASK = {
    "long_pretraining_10k_admission": "7b_long_training",
    "generative_semantic_probe": "semantic_eval",
    "cognitive_finetuning_curriculum": "cognitive_sft",
    "hard_checkpoint_admission": "checkpoint_admission",
    "persistent_inference_service": "runtime_inference",
    "scaling_14b_preflight": "14b_scaling",
    "continuity_memory_objective_eval": "continuity",
}


def process_for(task_type: str) -> str:
    return PROCESS_BY_TASK.get(task_type, "governed_runtime_capability")


def evidence_for(task_type: str, category: str) -> list[str]:
    base = [
        "authority_label",
        "risk_class",
        "rollback_handle",
        "audit_trace",
        "verifier_result",
        f"category_{category}",
    ]
    task_specific = {
        "long_pretraining_10k_admission": [
            "train_10000_report",
            "loss_curve",
            "nan_skip_count",
            "checkpoint_hash",
        ],
        "generative_semantic_probe": [
            "held_out_prompt",
            "generated_tokens",
            "non_empty_response",
            "semantic_boundary",
        ],
        "cognitive_finetuning_curriculum": [
            "curriculum_manifest",
            "source_trace",
            "private_data_filter",
            "train_eval_split",
        ],
        "hard_checkpoint_admission": [
            "candidate_report",
            "v03_baseline",
            "regression_threshold",
            "operator_release_block",
        ],
        "persistent_inference_service": [
            "service_contract",
            "request_schema",
            "response_schema",
            "health_check",
        ],
        "scaling_14b_preflight": [
            "7b_win_condition",
            "14b_budget",
            "stop_rule",
            "multi_gpu_preflight",
        ],
        "continuity_memory_objective_eval": [
            "session_a_state",
            "session_b_recall",
            "objective_consistency",
            "policy_stability",
        ],
    }
    return task_specific.get(task_type, ["module_contract", "safe_default", "state_snapshot"]) + base


def make_row(raw: dict[str, Any], index: int) -> dict[str, Any]:
    task_type = TASKS[(index - 1) % len(TASKS)]
    category = raw["category"]
    return {
        "schema": SCHEMA,
        "id": f"eden-v04-cognitive-capability-{index:05d}",
        "input": {
            "task_type": task_type,
            "process": process_for(task_type),
            "category_hint": category,
            "source_path": raw["source_path"],
            "evidence_excerpt": raw["excerpt"],
            "question": (
                "What structured v0.4 runtime decision should GEWC make before "
                "this capability can affect EDEN behavior?"
            ),
        },
        "target": {
            "authority": AUTHORITY,
            "operation": f"{task_type}_through_gewc",
            "semantic_output_kind": "structured_v04_runtime_decision",
            "required_evidence": evidence_for(task_type, category),
            "requires_verification": True,
            "requires_rollback_plan": True,
            "requires_checkpoint_admission": True,
            "requires_persistent_service_gate": task_type == "persistent_inference_service",
            "requires_continuity_gate": task_type == "continuity_memory_objective_eval",
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
            "curriculum_stage": "eden_v04_cognitive_capability",
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


def manifest_for(paths: dict[str, Path], splits: dict[str, list[dict[str, Any]]]) -> dict[str, Any]:
    all_rows = [row for rows in splits.values() for row in rows]
    task_types = Counter(row["input"]["task_type"] for row in all_rows)
    processes = Counter(row["input"]["process"] for row in all_rows)
    categories = Counter(row["input"]["category_hint"] for row in all_rows)
    valid_records = sum(1 for row in all_rows if row_valid(row))
    return {
        "schema": MANIFEST_SCHEMA,
        "artifact": "eden_v04_cognitive_capability_corpus_manifest",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "contains_private_data": False,
        "external_model_dependency": False,
        "records": len(all_rows),
        "valid_records": valid_records,
        "rows": {name: len(rows) for name, rows in splits.items()},
        "task_types": dict(sorted(task_types.items())),
        "processes": dict(sorted(processes.items())),
        "categories": dict(sorted(categories.items())),
        "paths": {name: str(path) for name, path in paths.items()},
        "accepted_for": [
            "v04_cognitive_capability_eval",
            "7b_10k_candidate_admission",
            "persistent_inference_service_contract",
            "continuity_runtime_eval",
            "14b_preflight_review",
        ],
        "not_accepted_for": ["AGI_claim", "production_release", "private_memory_training"],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--max-rows", type=int, default=DEFAULT_MAX_ROWS)
    parser.add_argument("--output-dir", type=Path, default=Path("training/data"))
    parser.add_argument(
        "--manifest",
        type=Path,
        default=Path("target/eden_v04/cognitive_capability_corpus_manifest.json"),
    )
    args = parser.parse_args()

    root = args.repo_root.resolve()
    raw_rows = collect_sources(root, args.max_rows)
    if not raw_rows:
        raise SystemExit("no EDEN-owned source rows available for v0.4 corpus")
    if len(raw_rows) < args.max_rows:
        raw_rows = [raw_rows[index % len(raw_rows)] for index in range(args.max_rows)]
    rows = [make_row(raw, index) for index, raw in enumerate(raw_rows, start=1)]
    if len(rows) < 8_000:
        raise SystemExit(f"not enough EDEN v0.4 cognitive rows: {len(rows)} < 8000")

    splits = split_rows(rows)
    paths = {
        "train": args.output_dir / "eden_v04_cognitive_capability_train.jsonl",
        "eval": args.output_dir / "eden_v04_cognitive_capability_eval.jsonl",
        "challenge": args.output_dir / "eden_v04_cognitive_capability_challenge.jsonl",
    }
    for name, path in paths.items():
        write_jsonl(path, splits[name])

    manifest = manifest_for(paths, splits)
    args.manifest.parent.mkdir(parents=True, exist_ok=True)
    args.manifest.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "EDEN v0.4 cognitive capability corpus "
        f"train={len(splits['train'])} eval={len(splits['eval'])} "
        f"challenge={len(splits['challenge'])} -> {args.output_dir}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
