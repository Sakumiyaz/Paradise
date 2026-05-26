#!/usr/bin/env python3
"""Write an internal model card for the EDEN v0.2 7B stability candidate."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.v02.model_card_internal.v1"
STORAGE_SCHEMA = "eden.v02.checkpoint_storage.v1"
AUTHORITY = "global_executive_workspace_core"


def load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    return value if isinstance(value, dict) else {}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stability-eval", type=Path, default=Path("target/eden_v02/stability_eval_report.json"))
    parser.add_argument("--comparison", type=Path, default=Path("target/eden_v02/checkpoint_comparison_report.json"))
    parser.add_argument("--adversarial", type=Path, default=Path("target/eden_v02/adversarial_eval_report.json"))
    parser.add_argument("--rollback", type=Path, default=Path("target/eden_v02/rollback_drill_report.json"))
    parser.add_argument("--model-card", type=Path, default=Path("target/eden_v02/model_card_internal.json"))
    parser.add_argument("--storage", type=Path, default=Path("target/eden_v02/checkpoint_storage_manifest.json"))
    args = parser.parse_args()

    stability = load_json(args.stability_eval)
    comparison = load_json(args.comparison)
    adversarial = load_json(args.adversarial)
    rollback = load_json(args.rollback)
    candidate = comparison.get("candidate", {}) if isinstance(comparison.get("candidate"), dict) else {}
    passed = all(
        value.get("passed") is True
        for value in [stability, comparison, adversarial, rollback]
    )
    storage = {
        "schema": STORAGE_SCHEMA,
        "authority": AUTHORITY,
        "artifact": "eden_v02_checkpoint_storage",
        "claim_allowed": False,
        "agi_claim": False,
        "weights_committed_to_repo": False,
        "weights_retained_on_gpu_vm": False,
        "recommended_storage": "private_encrypted_object_store_or_ephemeral_volume_snapshot",
        "required_before_retention": [
            "owner_approval",
            "access_control_policy",
            "hash_manifest",
            "rollback_drill",
            "deletion_policy",
        ],
        "current_policy": "copy reports only; purge run directories after evidence capture",
    }
    model_card = {
        "schema": SCHEMA,
        "authority": AUTHORITY,
        "artifact": "eden_v02_model_card_internal",
        "claim_allowed": False,
        "agi_claim": False,
        "passed": passed,
        "model": {
            "model_id": "eden-megatron-7b-stability-250",
            "architecture": "Megatron GPT-style decoder, random init, EDEN-owned corpus/tokenizer",
            "parameters": candidate.get("model_parameters"),
            "completed_iterations": candidate.get("iterations"),
            "production_model_allowed": False,
        },
        "intended_use": [
            "GEWC-subordinate candidate generation",
            "checkpoint stability comparison",
            "runtime adapter testing",
        ],
        "not_intended_use": [
            "AGI claim",
            "production autonomous inference",
            "direct memory writes",
            "direct tool execution",
            "objective mutation",
            "external benchmark superiority claim",
        ],
        "training_data": {
            "source": "repo-owned EDEN corpus plus v0.2 stability/adversarial corpus for evaluation",
            "external_model_dependency": False,
            "private_data_allowed": False,
        },
        "known_limits": [
            "short run length",
            "tiny corpus relative to production LLM training",
            "token generation remains noisy",
            "semantic acceptance comes from GEWC gates, not model self-claims",
            "no external AGI benchmark validation",
        ],
        "required_before_production": [
            "longer pretraining",
            "held-out external eval",
            "prompt-injection and data-poisoning eval",
            "checkpoint retention approval",
            "operator release review",
        ],
        "supporting_reports": {
            "stability_eval": str(args.stability_eval),
            "comparison": str(args.comparison),
            "adversarial": str(args.adversarial),
            "rollback": str(args.rollback),
            "storage": str(args.storage),
        },
    }
    args.storage.parent.mkdir(parents=True, exist_ok=True)
    args.storage.write_text(json.dumps(storage, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    args.model_card.parent.mkdir(parents=True, exist_ok=True)
    args.model_card.write_text(json.dumps(model_card, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"EDEN v0.2 model card passed={passed} -> {args.model_card}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
