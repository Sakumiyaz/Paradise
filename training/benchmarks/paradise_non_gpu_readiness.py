#!/usr/bin/env python3
"""Write a Paradise non-GPU readiness report.

This gate checks product/runtime readiness surfaces that do not require GPU,
network, hardware devices or checkpoints. It is intentionally a no-claim gate:
passing it means the repo has coherent public contracts and policies, not that
Eden has learned model capability.
"""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


AUTHORITY = "global_executive_workspace_core"
SCHEMA = "paradise.non_gpu_readiness.v1"


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
        if not line.strip():
            continue
        value = json.loads(line)
        if isinstance(value, dict):
            rows.append(value)
    return rows


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def check(area: str, passed: bool, evidence: str) -> dict[str, Any]:
    return {"area": area, "passed": passed, "evidence": evidence}


def git_ls_files(root: Path) -> list[str]:
    try:
        result = subprocess.run(
            ["git", "ls-files"],
            cwd=root,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        return []
    return [line for line in result.stdout.splitlines() if line.strip()]


def dataset_summary(paths: list[Path]) -> dict[str, Any]:
    rows: list[dict[str, Any]] = []
    for path in paths:
        rows.extend(load_jsonl(path))
    private_rows = 0
    external_dependency_rows = 0
    schemas: set[str] = set()
    for row in rows:
        metadata = row.get("metadata", {})
        if not isinstance(metadata, dict):
            metadata = row.get("governance", {})
        if isinstance(metadata, dict):
            if metadata.get("contains_private_data") is not False:
                private_rows += 1
            if metadata.get("external_model_dependency") is not False:
                external_dependency_rows += 1
        else:
            private_rows += 1
            external_dependency_rows += 1
        schema = row.get("schema")
        if isinstance(schema, str):
            schemas.add(schema)
    return {
        "rows": len(rows),
        "private_rows": private_rows,
        "external_dependency_rows": external_dependency_rows,
        "schemas": sorted(schemas),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path("."))
    parser.add_argument("--output", type=Path, default=Path("target/paradise_non_gpu_readiness/non_gpu_readiness_report.json"))
    args = parser.parse_args()

    root = args.repo_root.resolve()
    makefile = read_text(root / "Makefile")
    readme = read_text(root / "README.md")
    docs_index = read_text(root / "docs/README.md")
    console = read_text(root / "docs/EDEN_OPERATOR_CONSOLE.html")
    gitignore = read_text(root / ".gitignore")
    manifest = load_json(root / "contracts/v1/manifest.json")
    model_target = load_json(root / "training/configs/eden_70b_modular_target.json")
    license_manifest = load_json(root / "training/data/license_manifest.json")
    checkpoint_registry = load_json(root / "training/models/checkpoint_registry.json")
    tracked = git_ls_files(root)

    required_docs = [
        "docs/PARADISE_PRODUCT_SPEC.md",
        "docs/PARADISE_MODEL_INTERFACE.md",
        "docs/PARADISE_DATASET_GOVERNANCE.md",
        "docs/PARADISE_EVALUATION_AND_ADMISSION.md",
        "docs/PARADISE_USABLE_TODAY.md",
        "docs/PARADISE_EXTERNAL_BRIEF.md",
        "docs/PARADISE_TECHNICAL_DEBT_REGISTER.md",
        "docs/PARADISE_ROADMAP.md",
        "docs/releases/paradise-non-gpu-readiness.md",
        "docs/releases/v0.2.0-public-readiness.md",
        "docs/CLAIMS_AND_LIMITATIONS.md",
        "docs/THREAT_MODEL.md",
    ]
    doc_paths = [root / item for item in required_docs]
    missing_docs = [item for item, path in zip(required_docs, doc_paths) if not path.exists()]

    dataset = dataset_summary([
        root / "training/data/eden_70b_modular_train.jsonl",
        root / "training/data/eden_70b_modular_eval.jsonl",
        root / "training/data/eden_70b_modular_challenge.jsonl",
        root / "training/data/eden_v04_cognitive_capability_train.jsonl",
        root / "training/data/eden_v04_cognitive_capability_eval.jsonl",
        root / "training/data/eden_v04_cognitive_capability_challenge.jsonl",
    ])

    module_budget = model_target.get("module_budget", [])
    modules = module_budget if isinstance(module_budget, list) else []
    runtime_policy = model_target.get("runtime_policy", {})
    manifest_schemas = manifest.get("schemas", [])
    manifest_schema_set = set(manifest_schemas if isinstance(manifest_schemas, list) else [])

    sensitive_tracked = [
        item for item in tracked
        if item.endswith(".env") or item.endswith(".key") or item.endswith(".pem") or "/.ssh/" in item
    ]

    checks = [
        check(
            "repo_state_sensitive_files",
            not sensitive_tracked and "*.env" in gitignore and "*.key" in gitignore and "/target" in gitignore,
            f"sensitive_tracked={len(sensitive_tracked)} gitignore_target={'/target' in gitignore}",
        ),
        check(
            "public_product_spec",
            not missing_docs and "Paradise is not a completed AGI" in readme and "PARADISE_PRODUCT_SPEC.md" in docs_index,
            f"missing_docs={missing_docs}",
        ),
        check(
            "model_interface_authority",
            model_target.get("total_parameters") == 70000000000
            and len(modules) == 6
            and all(isinstance(module, dict) and module.get("routed_by_gewc") is True for module in modules)
            and runtime_policy.get("models_are_subordinate") is True
            and runtime_policy.get("direct_memory_writes") is False
            and runtime_policy.get("direct_tool_execution") is False,
            f"modules={len(modules)} total={model_target.get('total_parameters')}",
        ),
        check(
            "dataset_governance",
            dataset["rows"] > 0 and dataset["private_rows"] == 0 and dataset["external_dependency_rows"] == 0,
            f"rows={dataset['rows']} private={dataset['private_rows']} external={dataset['external_dependency_rows']}",
        ),
        check(
            "dataset_license_manifest",
            license_manifest.get("schema") == "paradise.dataset_license_manifest.v1"
            and license_manifest.get("contains_private_data") is False
            and license_manifest.get("external_model_dependency") is False
            and len(license_manifest.get("datasets", [])) >= 3,
            f"datasets={len(license_manifest.get('datasets', []))} private={license_manifest.get('contains_private_data')}",
        ),
        check(
            "evaluation_suite",
            all(target in makefile for target in [
                "training-eden-v01-semantic-eval",
                "training-eden-v02-adversarial-eval",
                "training-eden-v03-generalization-eval",
                "training-eden-v04-operational-eval",
                "eden-api-conformance",
                "paradise-module-semantic-eval",
                "paradise-strong-eval",
                "paradise-checkpoint-evidence-review",
            ]),
            "v01/v02/v03/v04/API and Paradise module evidence targets present",
        ),
        check(
            "checkpoint_admission_policy",
            "schemas/paradise-non-gpu-readiness-v1.json" in manifest_schema_set
            and "schemas/paradise-checkpoint-registry-v1.json" in manifest_schema_set
            and "schemas/paradise-checkpoint-registry-admission-v1.json" in manifest_schema_set
            and "schemas/paradise-checkpoint-admission-dry-run-v1.json" in manifest_schema_set
            and "schemas/paradise-checkpoint-admission-gate-v1.json" in manifest_schema_set
            and "schemas/eden-70b-checkpoint-admission-v1.json" in manifest_schema_set
            and checkpoint_registry.get("schema") == "paradise.checkpoint_registry.v1"
            and checkpoint_registry.get("production_model_allowed") is False
            and checkpoint_registry.get("entries") == []
            and "paradise-checkpoint-registry-smoke:" in makefile
            and "paradise-checkpoint-admission-gate-smoke:" in makefile
            and "production_model_allowed" in read_text(root / "docs/PARADISE_EVALUATION_AND_ADMISSION.md"),
            "checkpoint admission remains contract-gated and no-claim",
        ),
        check(
            "public_contract_validation",
            "schemas/paradise-public-contract-validation-v1.json" in manifest_schema_set
            and "make contracts-validate" in manifest.get("gates", [])
            and "contracts-validate:" in makefile,
            "contract validation is executable and listed in the public manifest",
        ),
        check(
            "operator_console_visibility",
            "paradise-non-gpu-readiness" in console
            and "Model checkpoints" in console
            and "Approval queue" in console,
            "operator console exposes non-GPU gate and checkpoint boundary",
        ),
        check(
            "external_deliverable_boundary",
            "will not share" in read_text(root / "docs/PARADISE_EXTERNAL_BRIEF.md")
            and "checkpoints" in read_text(root / "docs/PARADISE_EXTERNAL_BRIEF.md"),
            "external brief separates public metrics from private data/checkpoints",
        ),
        check(
            "hardware_network_tests_isolated",
            "external-tests:" in makefile and "api-socket-test:" in makefile and "make external-tests" in read_text(root / "docs/PARADISE_EVALUATION_AND_ADMISSION.md"),
            "hardware/network tests are opt-in",
        ),
        check(
            "technical_debt_registered",
            "Benchmark local tick harness" in read_text(root / "docs/PARADISE_TECHNICAL_DEBT_REGISTER.md")
            and "GPU model checkpoints" in read_text(root / "docs/PARADISE_TECHNICAL_DEBT_REGISTER.md"),
            "known non-GPU follow-ups are registered",
        ),
        check(
            "roadmap_and_release_package",
            "Paradise Roadmap" in read_text(root / "docs/PARADISE_ROADMAP.md")
            and "Paradise Non-GPU Readiness Release Package" in read_text(root / "docs/releases/paradise-non-gpu-readiness.md")
            and "Paradise v0.2.0 Public Readiness" in read_text(root / "docs/releases/v0.2.0-public-readiness.md")
            and "Paradise Usable Today" in read_text(root / "docs/PARADISE_USABLE_TODAY.md")
            and "paradise-public-demo:" in makefile
            and "paradise-dataset-manifest:" in makefile
            and "paradise-release-package:" in makefile,
            "roadmap and release package are documented and executable",
        ),
    ]

    passed_checks = sum(1 for item in checks if item["passed"])
    score = passed_checks / len(checks)
    report = {
        "schema": SCHEMA,
        "artifact": "paradise_non_gpu_readiness",
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "gpu_required": False,
        "production_model_allowed": False,
        "passed": passed_checks == len(checks),
        "score": score,
        "checks": checks,
        "dataset_summary": dataset,
        "model_target": {
            "total_parameters": model_target.get("total_parameters"),
            "module_count": len(modules),
            "active_default_parameters": model_target.get("active_default_parameters"),
        },
        "license_manifest": {
            "schema": license_manifest.get("schema"),
            "dataset_count": len(license_manifest.get("datasets", [])),
            "contains_private_data": license_manifest.get("contains_private_data"),
        },
        "checkpoint_registry": {
            "schema": checkpoint_registry.get("schema"),
            "entry_count": len(checkpoint_registry.get("entries", [])),
            "production_model_allowed": checkpoint_registry.get("production_model_allowed"),
        },
        "output": str(args.output),
    }
    write_json(args.output, report)
    print(
        f"[PARADISE-NON-GPU-READINESS] passed={report['passed']} "
        f"checks={passed_checks}/{len(checks)} score={score:.2f} path={args.output}"
    )
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
