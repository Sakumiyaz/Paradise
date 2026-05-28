#!/usr/bin/env python3
"""Build a stronger non-GPU Paradise evaluation bundle.

This gate composes existing local evidence into module families. It does not
run model inference and it does not admit checkpoints; it verifies that the
runtime has executable evidence for the public architectural claims it exposes.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


AUTHORITY = "global_executive_workspace_core"
SCHEMA = "paradise.strong_eval.v1"
OUTPUT = Path("target/paradise_strong_eval/strong_eval_report.json")


def load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    return value if isinstance(value, dict) else {}


def check(name: str, passed: bool, evidence: str) -> dict[str, Any]:
    return {"check": name, "passed": passed, "evidence": evidence}


def report_passed(value: dict[str, Any]) -> bool:
    return value.get("passed") is True


def claim_boundary(value: dict[str, Any]) -> bool:
    return (
        value.get("claim_allowed") is False
        and value.get("agi_claim") is False
        and value.get("production_model_allowed") is False
    )


def family(name: str, checks: list[dict[str, Any]]) -> dict[str, Any]:
    passed = all(item["passed"] for item in checks)
    return {
        "family": name,
        "passed": passed,
        "passed_checks": sum(1 for item in checks if item["passed"]),
        "total_checks": len(checks),
        "checks": checks,
    }


def main() -> int:
    module_eval = load_json(Path("target/paradise_module_semantic_eval/module_semantic_eval_report.json"))
    dataset_manifest = load_json(Path("target/paradise_dataset_manifest/paradise_dataset_manifest.json"))
    checkpoint_review = load_json(Path("target/paradise_checkpoint_evidence_review/checkpoint_evidence_review.json"))
    non_gpu = load_json(Path("target/paradise_non_gpu_readiness/non_gpu_readiness_report.json"))
    contracts = load_json(Path("target/public_contracts/validation_report.json"))
    registry = load_json(Path("training/models/checkpoint_registry.json"))
    target = load_json(Path("training/configs/eden_70b_modular_target.json"))

    module_results = module_eval.get("module_results", [])
    module_names = {
        item.get("module")
        for item in module_results
        if isinstance(item, dict) and item.get("passed") is True
    }
    required_modules = {
        "memory",
        "planner",
        "world_model",
        "safety",
        "model_router",
        "observability",
    }
    datasets = dataset_manifest.get("datasets", [])
    module_budget = target.get("module_budget", [])

    families = [
        family(
            "memory_planning_world_model",
            [
                check("memory_route_covered", "memory" in module_names, "paradise_module_semantic_eval.module_results"),
                check("planner_route_covered", "planner" in module_names, "paradise_module_semantic_eval.module_results"),
                check("world_model_route_covered", "world_model" in module_names, "paradise_module_semantic_eval.module_results"),
            ],
        ),
        family(
            "safety_verification_checkpoint_boundary",
            [
                check("safety_route_covered", "safety" in module_names, "paradise_module_semantic_eval.module_results"),
                check("checkpoint_review_passed", report_passed(checkpoint_review), "paradise_checkpoint_evidence_review.passed"),
                check(
                    "public_registry_blocks_admission",
                    registry.get("active_checkpoint") is None
                    and registry.get("production_model_allowed") is False
                    and checkpoint_review.get("checkpoint_admission_allowed") is False,
                    "training/models/checkpoint_registry.json",
                ),
            ],
        ),
        family(
            "model_router_inference_governance",
            [
                check("model_router_route_covered", "model_router" in module_names, "paradise_module_semantic_eval.module_results"),
                check(
                    "models_subordinate_to_runtime",
                    target.get("runtime_policy", {}).get("models_are_subordinate") is True,
                    "training/configs/eden_70b_modular_target.json",
                ),
                check(
                    "modular_budget_declared",
                    isinstance(module_budget, list) and len(module_budget) >= 6,
                    "training/configs/eden_70b_modular_target.json",
                ),
            ],
        ),
        family(
            "observability_reproducibility",
            [
                check("observability_route_covered", "observability" in module_names, "paradise_module_semantic_eval.module_results"),
                check("contracts_passed", contracts.get("passed") is True, "target/public_contracts/validation_report.json"),
                check("non_gpu_readiness_passed", report_passed(non_gpu), "target/paradise_non_gpu_readiness/non_gpu_readiness_report.json"),
            ],
        ),
        family(
            "dataset_governance",
            [
                check("dataset_manifest_passed", report_passed(dataset_manifest), "target/paradise_dataset_manifest/paradise_dataset_manifest.json"),
                check(
                    "public_rows_without_private_data",
                    bool(datasets)
                    and all(
                        isinstance(item, dict)
                        and item.get("contains_private_data") is False
                        and item.get("external_model_dependency") is False
                        for item in datasets
                    ),
                    "paradise_dataset_manifest.datasets",
                ),
                check("all_required_modules_present", required_modules.issubset(module_names), "paradise_module_semantic_eval.module_results"),
            ],
        ),
    ]

    checks = [item for record in families for item in record["checks"]]
    passed_checks = sum(1 for item in checks if item["passed"])
    report = {
        "schema": SCHEMA,
        "artifact": "paradise_strong_eval",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "production_model_allowed": False,
        "gpu_required": False,
        "score": passed_checks / len(checks) if checks else 0,
        "families": families,
        "checks": checks,
        "input_reports": {
            "module_semantic_eval": bool(module_eval),
            "dataset_manifest": bool(dataset_manifest),
            "checkpoint_evidence_review": bool(checkpoint_review),
            "non_gpu_readiness": bool(non_gpu),
            "public_contracts": bool(contracts),
        },
        "claim_boundaries": {
            "module_semantic_eval": claim_boundary(module_eval),
            "dataset_manifest": claim_boundary(dataset_manifest),
            "checkpoint_evidence_review": claim_boundary(checkpoint_review),
            "non_gpu_readiness": claim_boundary(non_gpu),
        },
        "decision": "non_gpu_architecture_evidence_passes; checkpoint_and_AGI_claims_remain_blocked",
    }
    report["passed"] = all(record["passed"] for record in families)
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "[paradise-strong-eval] "
        f"passed={report['passed']} score={report['score']:.2f} path={OUTPUT}"
    )
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
