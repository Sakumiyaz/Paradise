#!/usr/bin/env python3
"""Validate the public Paradise contract surface without external packages."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def validate_schema(path: Path, errors: list[str]) -> None:
    try:
        schema = load_json(path)
    except json.JSONDecodeError as exc:
        fail(errors, f"{path}: invalid JSON: {exc}")
        return

    if not isinstance(schema, dict):
        fail(errors, f"{path}: schema must be a JSON object")
        return
    if schema.get("type") != "object":
        fail(errors, f"{path}: top-level type must be object")
    if not schema.get("$schema"):
        fail(errors, f"{path}: missing $schema")
    if not schema.get("$id"):
        fail(errors, f"{path}: missing $id")
    if "properties" not in schema:
        fail(errors, f"{path}: missing properties")


def validate_openapi(path: Path, errors: list[str]) -> None:
    try:
        spec = load_json(path)
    except json.JSONDecodeError as exc:
        fail(errors, f"{path}: invalid JSON: {exc}")
        return

    if not isinstance(spec, dict):
        fail(errors, f"{path}: OpenAPI spec must be a JSON object")
        return
    if not str(spec.get("openapi", "")).startswith("3."):
        fail(errors, f"{path}: expected OpenAPI 3.x")
    if not isinstance(spec.get("paths"), dict) or not spec["paths"]:
        fail(errors, f"{path}: paths must be a non-empty object")
    info = spec.get("info")
    if not isinstance(info, dict) or not info.get("title") or not info.get("version"):
        fail(errors, f"{path}: missing info.title or info.version")


def validate_license_manifest(path: Path, errors: list[str]) -> None:
    try:
        manifest = load_json(path)
    except FileNotFoundError:
        fail(errors, f"{path}: missing dataset license manifest")
        return
    except json.JSONDecodeError as exc:
        fail(errors, f"{path}: invalid JSON: {exc}")
        return

    if not isinstance(manifest, dict):
        fail(errors, f"{path}: manifest must be a JSON object")
        return
    if manifest.get("schema") != "paradise.dataset_license_manifest.v1":
        fail(errors, f"{path}: unexpected schema")
    if manifest.get("contains_private_data") is not False:
        fail(errors, f"{path}: contains_private_data must be false")
    if manifest.get("external_model_dependency") is not False:
        fail(errors, f"{path}: external_model_dependency must be false")

    datasets = manifest.get("datasets")
    if not isinstance(datasets, list) or not datasets:
        fail(errors, f"{path}: datasets must be a non-empty array")
        return
    for index, dataset in enumerate(datasets):
        if not isinstance(dataset, dict):
            fail(errors, f"{path}: datasets[{index}] must be an object")
            continue
        for field in ("id", "path", "license", "origin", "contains_private_data"):
            if field not in dataset:
                fail(errors, f"{path}: datasets[{index}] missing {field}")
        if dataset.get("contains_private_data") is not False:
            fail(errors, f"{path}: datasets[{index}] must not contain private data")
        if dataset.get("license") in {"unknown", "", None}:
            fail(errors, f"{path}: datasets[{index}] has unknown license")


def validate_checkpoint_registry(path: Path, errors: list[str]) -> None:
    try:
        registry = load_json(path)
    except FileNotFoundError:
        fail(errors, f"{path}: missing checkpoint registry")
        return
    except json.JSONDecodeError as exc:
        fail(errors, f"{path}: invalid JSON: {exc}")
        return

    if not isinstance(registry, dict):
        fail(errors, f"{path}: registry must be a JSON object")
        return
    if registry.get("schema") != "paradise.checkpoint_registry.v1":
        fail(errors, f"{path}: unexpected schema")
    if registry.get("production_model_allowed") is not False:
        fail(errors, f"{path}: production_model_allowed must remain false")
    if registry.get("claim_allowed") is not False or registry.get("agi_claim") is not False:
        fail(errors, f"{path}: claim flags must remain false")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path("."))
    parser.add_argument("--output", type=Path, default=Path("target/public_contracts/validation_report.json"))
    args = parser.parse_args()

    root = args.repo_root.resolve()
    manifest_path = root / "contracts/v1/manifest.json"
    errors: list[str] = []

    try:
        manifest = load_json(manifest_path)
    except json.JSONDecodeError as exc:
        errors.append(f"{manifest_path}: invalid JSON: {exc}")
        manifest = {}

    schemas = manifest.get("schemas") if isinstance(manifest, dict) else None
    openapi_specs = manifest.get("openapi") if isinstance(manifest, dict) else None
    gates = manifest.get("gates") if isinstance(manifest, dict) else None

    if manifest.get("schema") != "eden-public-contract-manifest-v1":
        fail(errors, f"{manifest_path}: unexpected schema")
    if not isinstance(schemas, list) or not schemas:
        fail(errors, f"{manifest_path}: schemas must be a non-empty array")
        schemas = []
    if len(schemas) != len(set(schemas)):
        fail(errors, f"{manifest_path}: duplicate schema entries found")
    if not isinstance(openapi_specs, list) or not openapi_specs:
        fail(errors, f"{manifest_path}: openapi must be a non-empty array")
        openapi_specs = []
    if not isinstance(gates, list) or "make contracts-validate" not in gates:
        fail(errors, f"{manifest_path}: gates must include make contracts-validate")

    for rel_path in schemas:
        schema_path = root / "contracts/v1" / rel_path
        if not schema_path.exists():
            fail(errors, f"{schema_path}: missing schema file")
            continue
        validate_schema(schema_path, errors)

    for rel_path in openapi_specs:
        spec_path = root / "contracts/v1" / rel_path
        if not spec_path.exists():
            fail(errors, f"{spec_path}: missing OpenAPI file")
            continue
        validate_openapi(spec_path, errors)

    validate_license_manifest(root / "training/data/license_manifest.json", errors)
    validate_checkpoint_registry(root / "training/models/checkpoint_registry.json", errors)

    report = {
        "schema": "paradise.public_contract_validation.v1",
        "passed": not errors,
        "checked_schema_count": len(schemas),
        "checked_openapi_count": len(openapi_specs),
        "errors": errors,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if errors:
        for error in errors:
            print(f"[contracts] {error}")
        return 1

    print(
        "[contracts] passed "
        f"schemas={report['checked_schema_count']} openapi={report['checked_openapi_count']} "
        f"output={args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
