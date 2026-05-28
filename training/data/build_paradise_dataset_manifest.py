#!/usr/bin/env python3
"""Build a public, non-confidential dataset manifest for Paradise."""

from __future__ import annotations

import collections
import hashlib
import json
from pathlib import Path
from typing import Any


DATASETS = [
    Path("training/data/eden_v04_cognitive_capability_train.jsonl"),
    Path("training/data/eden_v04_cognitive_capability_eval.jsonl"),
    Path("training/data/eden_v04_cognitive_capability_challenge.jsonl"),
    Path("training/data/eden_70b_modular_train.jsonl"),
    Path("training/data/eden_70b_modular_eval.jsonl"),
    Path("training/data/eden_70b_modular_challenge.jsonl"),
]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    value = json.loads(path.read_text(encoding="utf-8"))
    return value if isinstance(value, dict) else {}


def summarize_jsonl(path: Path) -> dict[str, Any]:
    categories: collections.Counter[str] = collections.Counter()
    task_types: collections.Counter[str] = collections.Counter()
    schemas: collections.Counter[str] = collections.Counter()
    rows = 0
    contains_private_data = False
    external_model_dependency = False
    with path.open(encoding="utf-8") as handle:
        for line in handle:
            if not line.strip():
                continue
            row = json.loads(line)
            rows += 1
            schemas[str(row.get("schema", "unknown"))] += 1
            metadata = row.get("metadata", {})
            input_body = row.get("input", {})
            categories[str(input_body.get("category_hint", "uncategorized"))] += 1
            task_types[str(input_body.get("task_type", "unknown"))] += 1
            contains_private_data = contains_private_data or bool(
                metadata.get("contains_private_data")
            )
            external_model_dependency = external_model_dependency or bool(
                metadata.get("external_model_dependency")
            )
    return {
        "path": path.as_posix(),
        "present": True,
        "rows": rows,
        "sha256": sha256(path),
        "schemas": dict(sorted(schemas.items())),
        "categories": dict(sorted(categories.items())),
        "top_task_types": dict(task_types.most_common(25)),
        "contains_private_data": contains_private_data,
        "external_model_dependency": external_model_dependency,
    }


def missing_dataset(path: Path) -> dict[str, Any]:
    return {
        "path": path.as_posix(),
        "present": False,
        "rows": 0,
        "contains_private_data": None,
        "external_model_dependency": None,
    }


def main() -> int:
    output_dir = Path("target/paradise_dataset_manifest")
    output_dir.mkdir(parents=True, exist_ok=True)
    license_manifest_path = Path("training/data/license_manifest.json")
    license_manifest = load_json(license_manifest_path)
    datasets = [
        summarize_jsonl(path) if path.exists() else missing_dataset(path)
        for path in DATASETS
    ]
    present = [dataset for dataset in datasets if dataset["present"]]
    total_rows = sum(int(dataset["rows"]) for dataset in present)
    all_public = all(dataset["contains_private_data"] is False for dataset in present)
    all_eden_owned = all(
        dataset["external_model_dependency"] is False for dataset in present
    )
    report = {
        "schema": "paradise.dataset_manifest.v1",
        "artifact": "paradise_dataset_manifest",
        "authority": "global_executive_workspace_core",
        "claim_allowed": False,
        "agi_claim": False,
        "production_model_allowed": False,
        "gpu_required": False,
        "dataset_scope": "public non-GPU training/evaluation fixtures and module capability corpora",
        "license_manifest": {
            "path": license_manifest_path.as_posix(),
            "present": license_manifest_path.exists(),
            "schema": license_manifest.get("schema"),
            "claim_allowed": license_manifest.get("claim_allowed"),
            "private_data_allowed": license_manifest.get("private_data_allowed"),
        },
        "datasets": datasets,
        "totals": {
            "files_present": len(present),
            "files_expected": len(DATASETS),
            "rows": total_rows,
        },
        "checks": [
            {
                "check": "all_expected_dataset_files_present",
                "passed": len(present) == len(DATASETS),
            },
            {"check": "license_manifest_present", "passed": license_manifest_path.exists()},
            {"check": "no_private_data_flags", "passed": all_public},
            {"check": "no_external_model_dependency_flags", "passed": all_eden_owned},
            {"check": "non_empty_corpus", "passed": total_rows > 0},
        ],
    }
    report["passed"] = all(check["passed"] for check in report["checks"])
    output = output_dir / "paradise_dataset_manifest.json"
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "[paradise-dataset-manifest] "
        f"passed={report['passed']} rows={total_rows} path={output}"
    )
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
