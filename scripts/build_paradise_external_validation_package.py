#!/usr/bin/env python3
"""Build a non-confidential external validation package manifest."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any


AUTHORITY = "global_executive_workspace_core"
SCHEMA = "paradise.external_validation_package.v1"
OUTPUT = Path("target/paradise_external_validation/external_validation_manifest.json")


def git_value(args: list[str], default: str) -> str:
    try:
        result = subprocess.run(
            ["git", *args],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    except (FileNotFoundError, subprocess.CalledProcessError):
        return default
    value = result.stdout.strip()
    return value or default


def load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    return value if isinstance(value, dict) else {}


def artifact(name: str, path: Path, required: bool = True) -> dict[str, Any]:
    value = load_json(path)
    return {
        "name": name,
        "path": path.as_posix(),
        "present": path.exists(),
        "required": required,
        "schema": value.get("schema"),
        "passed": value.get("passed"),
    }


def main() -> int:
    commit = git_value(["rev-parse", "HEAD"], "unknown")
    artifacts = [
        artifact("public_contract_validation", Path("target/public_contracts/validation_report.json")),
        artifact("non_gpu_readiness", Path("target/paradise_non_gpu_readiness/non_gpu_readiness_report.json")),
        artifact("dataset_manifest", Path("target/paradise_dataset_manifest/paradise_dataset_manifest.json")),
        artifact("module_semantic_eval", Path("target/paradise_module_semantic_eval/module_semantic_eval_report.json")),
        artifact("strong_eval", Path("target/paradise_strong_eval/strong_eval_report.json")),
        artifact("checkpoint_evidence_review", Path("target/paradise_checkpoint_evidence_review/checkpoint_evidence_review.json")),
        artifact("public_demo", Path("target/paradise_public_demo/public_demo_manifest.json")),
        artifact("release_package", Path("target/paradise_release/release_package_manifest.json")),
        artifact("checkpoint_registry", Path("training/models/checkpoint_registry.json")),
        artifact("license_manifest", Path("training/data/license_manifest.json")),
    ]
    missing_required = [item["name"] for item in artifacts if item["required"] and not item["present"]]
    failed_required = [
        item["name"]
        for item in artifacts
        if item["required"] and item["passed"] is False
    ]
    report = {
        "schema": SCHEMA,
        "artifact": "paradise_external_validation_package",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "production_model_allowed": False,
        "gpu_required": False,
        "public_only": True,
        "commit": commit,
        "scope": "non-confidential Paradise runtime, contracts, dataset governance, and no-claim evaluation evidence",
        "commands": [
            "make contracts-validate",
            "make paradise-non-gpu-readiness",
            "make paradise-dataset-manifest",
            "make paradise-module-semantic-eval",
            "make paradise-checkpoint-evidence-review",
            "make paradise-checkpoint-admission-gate-smoke",
            "make paradise-strong-eval",
            "make paradise-release-package",
            "make public-audit",
        ],
        "artifacts": artifacts,
        "excluded": [
            "checkpoint weights",
            "private datasets",
            "credentials",
            "GPU VM filesystem snapshots",
            "raw private logs",
        ],
        "missing_required": missing_required,
        "failed_required": failed_required,
        "passed": not missing_required and not failed_required,
        "reviewer_notes": [
            "This package validates public evidence shape and reproducibility.",
            "It is not an AGI benchmark result and does not include checkpoint admission.",
            "All model checkpoints remain outside git and outside this package.",
        ],
    }
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "[paradise-external-validation-package] "
        f"passed={report['passed']} artifacts={len(artifacts)} path={OUTPUT}"
    )
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
