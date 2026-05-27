#!/usr/bin/env python3
"""Build tiny EDEN-70B modular seed splits from repo-owned records only."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "training" / "data"

MODULES = [
    (
        "eden_33b_elcp_primary",
        "primary_cognitive_model",
        "Convert a task into a governed cognitive hypothesis packet.",
        "surface_response + situation_state + risk_calibration",
    ),
    (
        "eden_cwm_12b_causal_world_model",
        "world_model",
        "Predict causal consequences before an action is accepted.",
        "world_delta + counterfactual + uncertainty",
    ),
    (
        "eden_multimodal_vla_12b",
        "multimodal_grounding",
        "Ground multimodal context into concepts and action affordances.",
        "perception_concepts + affordances + modality_confidence",
    ),
    (
        "eden_planner_code_tool_6b",
        "planning_code_tools",
        "Draft a hierarchical plan and tool contract without executing it.",
        "plan_graph + tool_contract + rollback_requirements",
    ),
    (
        "eden_safety_verifier_4b",
        "safety_verifier_critic",
        "Review a proposed state change for risk, policy and corrigibility.",
        "risk_score + policy_verdict + approval_requirement",
    ),
    (
        "eden_memory_router_retrieval_3b",
        "memory_router_retrieval",
        "Retrieve and rank memory evidence without writing memory directly.",
        "retrieval_candidates + source_confidence + conflict_flags",
    ),
]

SPLITS = {
    "train": "module_training_contract",
    "eval": "module_eval_contract",
    "challenge": "module_challenge_contract",
}


def row(split: str, index: int, module: tuple[str, str, str, str]) -> dict:
    module_id, role, prompt, target = module
    return {
        "schema": "eden.modular_70b.dataset_row.v1",
        "id": f"eden-70b-{split}-{index:02d}",
        "module_id": module_id,
        "role": role,
        "input": {
            "task": prompt,
            "context": f"{SPLITS[split]} for {module_id}",
            "authority": "global_executive_workspace_core",
            "risk": "bounded",
        },
        "target": {
            "output_kind": target,
            "direct_memory_write": False,
            "direct_objective_update": False,
            "direct_tool_execution": False,
            "outputs_are_hypotheses": True,
        },
        "governance": {
            "claim_allowed": False,
            "agi_claim": False,
            "checkpoint_admission": False,
            "external_model_dependency": False,
            "contains_private_data": False,
        },
        "metadata": {
            "contains_private_data": False,
            "external_model_dependency": False,
            "source_family": "repo_owned_seed_contract",
            "license_review_required_before_scale": True,
            "curriculum_stage": "eden_70b_modular_seed",
        },
        "evidence": {
            "source": "repo_owned_seed_contract",
            "license_review_required_before_scale": True,
        },
    }


def write_split(split: str) -> None:
    path = OUT / f"eden_70b_modular_{split}.jsonl"
    rows = [row(split, idx + 1, module) for idx, module in enumerate(MODULES)]
    path.write_text("\n".join(json.dumps(item, sort_keys=True) for item in rows) + "\n")
    print(f"wrote {path} rows={len(rows)}")


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for split in SPLITS:
        write_split(split)


if __name__ == "__main__":
    main()
