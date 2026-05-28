#!/usr/bin/env python3
"""Build a non-confidential Paradise release package manifest."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any


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


def main() -> int:
    root = Path(".").resolve()
    output_dir = root / "target/paradise_release"
    output_dir.mkdir(parents=True, exist_ok=True)
    commit = git_value(["rev-parse", "HEAD"], "unknown")
    branch = git_value(["rev-parse", "--abbrev-ref", "HEAD"], "unknown")
    short = commit[:12] if commit != "unknown" else "unknown"
    tag = f"paradise-public-readiness-{short}"

    contract_report = root / "target/public_contracts/validation_report.json"
    readiness_report = root / "target/paradise_non_gpu_readiness/non_gpu_readiness_report.json"
    dataset_manifest = root / "target/paradise_dataset_manifest/paradise_dataset_manifest.json"
    semantic_eval = root / "target/paradise_module_semantic_eval/module_semantic_eval_report.json"
    strong_eval = root / "target/paradise_strong_eval/strong_eval_report.json"
    checkpoint_review = root / "target/paradise_checkpoint_evidence_review/checkpoint_evidence_review.json"
    public_demo = root / "target/paradise_public_demo/public_demo_manifest.json"
    public_demo_transcript = root / "target/paradise_public_demo/demo_transcript.md"
    release_note = root / "docs/releases/paradise-non-gpu-readiness.md"
    v02_release_note = root / "docs/releases/v0.2.0-public-readiness.md"
    checkpoint_registry = root / "training/models/checkpoint_registry.json"
    license_manifest = root / "training/data/license_manifest.json"

    manifest = {
        "schema": "paradise.release_package.v1",
        "claim_allowed": False,
        "agi_claim": False,
        "production_model_allowed": False,
        "gpu_required": False,
        "branch": branch,
        "commit": commit,
        "suggested_tag": tag,
        "release_scope": "public non-GPU runtime and governance readiness",
        "commands": [
            "make contracts-validate",
            "make paradise-non-gpu-readiness",
            "make paradise-dataset-manifest",
            "make paradise-module-semantic-eval",
            "make paradise-strong-eval",
            "make paradise-checkpoint-evidence-review",
            "make paradise-checkpoint-registry-smoke",
            "make paradise-checkpoint-admission-gate-smoke",
            "make paradise-public-demo",
            "make paradise-benchmark-local",
            "make check",
            "make eden-api-conformance",
            "make public-audit",
            "make paradise-external-validation-package",
        ],
        "optional_strict_commands": [
            "make install-secret-scanners",
            "make public-audit-strict",
        ],
        "artifacts": [
            {
                "name": "public_contract_validation",
                "path": str(contract_report.relative_to(root)),
                "present": contract_report.exists(),
                "passed": load_json(contract_report).get("passed"),
            },
            {
                "name": "non_gpu_readiness",
                "path": str(readiness_report.relative_to(root)),
                "present": readiness_report.exists(),
                "passed": load_json(readiness_report).get("passed"),
            },
            {
                "name": "paradise_dataset_manifest",
                "path": str(dataset_manifest.relative_to(root)),
                "present": dataset_manifest.exists(),
                "passed": load_json(dataset_manifest).get("passed"),
            },
            {
                "name": "paradise_module_semantic_eval",
                "path": str(semantic_eval.relative_to(root)),
                "present": semantic_eval.exists(),
                "passed": load_json(semantic_eval).get("passed"),
            },
            {
                "name": "paradise_strong_eval",
                "path": str(strong_eval.relative_to(root)),
                "present": strong_eval.exists(),
                "passed": load_json(strong_eval).get("passed"),
            },
            {
                "name": "paradise_checkpoint_evidence_review",
                "path": str(checkpoint_review.relative_to(root)),
                "present": checkpoint_review.exists(),
                "passed": load_json(checkpoint_review).get("passed"),
            },
            {
                "name": "paradise_public_demo",
                "path": str(public_demo.relative_to(root)),
                "present": public_demo.exists(),
                "passed": load_json(public_demo).get("passed"),
            },
            {
                "name": "paradise_public_demo_transcript",
                "path": str(public_demo_transcript.relative_to(root)),
                "present": public_demo_transcript.exists(),
            },
            {
                "name": "release_note",
                "path": str(release_note.relative_to(root)),
                "present": release_note.exists(),
            },
            {
                "name": "v0_2_public_readiness_release_note",
                "path": str(v02_release_note.relative_to(root)),
                "present": v02_release_note.exists(),
            },
            {
                "name": "checkpoint_registry",
                "path": str(checkpoint_registry.relative_to(root)),
                "present": checkpoint_registry.exists(),
            },
            {
                "name": "dataset_license_manifest",
                "path": str(license_manifest.relative_to(root)),
                "present": license_manifest.exists(),
            },
        ],
        "excluded": [
            "checkpoint weights",
            "private datasets",
            "credentials",
            "GPU VM workspaces",
            "runtime target directories",
        ],
    }

    output = output_dir / "release_package_manifest.json"
    output.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"[paradise-release-package] manifest={output} tag={tag}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
