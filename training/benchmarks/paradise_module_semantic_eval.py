#!/usr/bin/env python3
"""Evaluate public module-level semantic coverage without model inference."""

from __future__ import annotations

import collections
import json
from pathlib import Path
from typing import Any


EVAL_FILES = [
    Path("training/data/eden_v04_cognitive_capability_eval.jsonl"),
    Path("training/data/eden_v04_cognitive_capability_challenge.jsonl"),
]

REQUIRED_CATEGORIES = {
    "runtime",
    "world_model",
    "tools",
    "training",
    "inference",
    "observability",
    "evaluation",
    "safety",
    "memory",
    "scaling",
}

REQUIRED_TASKS = {
    "tool_use_dry_run_policy",
    "memory_write_authorization",
    "model_router_cost_risk",
    "observability_replay_trace",
    "self_modification_review",
    "identity_fork_boundary",
    "hard_checkpoint_admission",
    "persistent_inference_service",
    "authority_boundary_parse",
    "prompt_injection_quarantine",
    "human_approval_escalation",
    "runtime_degradation_safe_mode",
    "generative_semantic_probe",
    "causal_plan_revision",
    "production_release_blocker",
}

MODULES = {
    "memory": {"categories": {"memory"}, "task_terms": {"memory"}},
    "planner": {"categories": {"runtime", "tools"}, "task_terms": {"plan", "tool_use"}},
    "world_model": {
        "categories": {"world_model"},
        "task_terms": {"causal", "simulation", "world"},
    },
    "safety": {
        "categories": {"safety"},
        "task_terms": {"injection", "approval", "production", "authority"},
    },
    "model_router": {
        "categories": {"inference", "scaling"},
        "task_terms": {"router", "inference", "checkpoint"},
    },
    "observability": {
        "categories": {"observability", "evaluation"},
        "task_terms": {"observability", "replay", "eval"},
    },
}


def rows(path: Path) -> list[dict[str, Any]]:
    result: list[dict[str, Any]] = []
    with path.open(encoding="utf-8") as handle:
        for line in handle:
            if line.strip():
                result.append(json.loads(line))
    return result


def main() -> int:
    output_dir = Path("target/paradise_module_semantic_eval")
    output_dir.mkdir(parents=True, exist_ok=True)
    loaded: list[dict[str, Any]] = []
    file_summaries = []
    for path in EVAL_FILES:
        file_rows = rows(path) if path.exists() else []
        loaded.extend(file_rows)
        file_summaries.append({"path": path.as_posix(), "present": path.exists(), "rows": len(file_rows)})

    categories: collections.Counter[str] = collections.Counter()
    task_types: collections.Counter[str] = collections.Counter()
    boundary_failures: list[str] = []
    for row in loaded:
        row_id = str(row.get("id", "unknown"))
        input_body = row.get("input", {})
        target = row.get("target", {})
        categories[str(input_body.get("category_hint", "uncategorized"))] += 1
        task_type = str(input_body.get("task_type", "unknown"))
        task_types[task_type] += 1
        if target.get("claim_allowed") is not False:
            boundary_failures.append(f"{row_id}: claim_allowed")
        if target.get("agi_claim") is not False:
            boundary_failures.append(f"{row_id}: agi_claim")
        if target.get("direct_memory_write") is not False:
            boundary_failures.append(f"{row_id}: direct_memory_write")
        if target.get("direct_tool_execution") is not False:
            boundary_failures.append(f"{row_id}: direct_tool_execution")

    module_results = []
    for module, rule in MODULES.items():
        category_hits = sum(categories[category] for category in rule["categories"])
        task_hits = sum(
            count
            for task, count in task_types.items()
            if any(term in task for term in rule["task_terms"])
        )
        module_results.append(
            {
                "module": module,
                "category_hits": category_hits,
                "task_hits": task_hits,
                "passed": category_hits > 0 and task_hits > 0,
            }
        )

    missing_categories = sorted(REQUIRED_CATEGORIES.difference(categories))
    missing_tasks = sorted(REQUIRED_TASKS.difference(task_types))
    checks = [
        {"check": "eval_files_present", "passed": all(item["present"] for item in file_summaries)},
        {"check": "non_empty_eval_rows", "passed": len(loaded) >= 100},
        {"check": "required_categories_present", "passed": not missing_categories},
        {"check": "required_tasks_present", "passed": not missing_tasks},
        {"check": "module_semantic_routes_covered", "passed": all(item["passed"] for item in module_results)},
        {"check": "claim_boundaries_preserved", "passed": not boundary_failures},
    ]
    report = {
        "schema": "paradise.module_semantic_eval.v1",
        "artifact": "paradise_module_semantic_eval",
        "authority": "global_executive_workspace_core",
        "claim_allowed": False,
        "agi_claim": False,
        "production_model_allowed": False,
        "gpu_required": False,
        "files": file_summaries,
        "rows": len(loaded),
        "categories": dict(sorted(categories.items())),
        "top_task_types": dict(task_types.most_common(30)),
        "missing_categories": missing_categories,
        "missing_tasks": missing_tasks,
        "boundary_failures": boundary_failures[:50],
        "module_results": module_results,
        "checks": checks,
    }
    report["passed"] = all(check["passed"] for check in checks)
    output = output_dir / "module_semantic_eval_report.json"
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "[paradise-module-semantic-eval] "
        f"passed={report['passed']} rows={len(loaded)} modules={len(module_results)} path={output}"
    )
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
