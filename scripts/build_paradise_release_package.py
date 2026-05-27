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
    release_note = root / "docs/releases/paradise-non-gpu-readiness.md"
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
            "make paradise-checkpoint-registry-smoke",
            "make check",
            "make eden-api-conformance",
            "make public-audit",
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
                "name": "release_note",
                "path": str(release_note.relative_to(root)),
                "present": release_note.exists(),
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
