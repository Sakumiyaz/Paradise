#!/usr/bin/env python3
"""Build stdlib-only evidence for an EDEN 7B Megatron checkpoint inference probe."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.megatron.7b.inference_report.v1"
TRAINING_EVIDENCE_SCHEMA = "eden.megatron.7b.training_evidence.v1"
DEFAULT_OUTPUT_DIR = Path("target/eden_megatron_7b_base_pilot")
DEFAULT_SCHEMA_PATH = Path("contracts/v1/schemas/eden-megatron-7b-inference-report-v1.json")


def fail(message: str) -> None:
    raise SystemExit(f"Megatron 7B inference report validation failed: {message}")


def fnv64(data: bytes) -> str:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value:016x}"


def parse_bool(value: str) -> bool:
    if value == "true":
        return True
    if value == "false":
        return False
    fail(f"invalid boolean value {value!r}")


def key_values(text: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()
    return values


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(f"{name} must be an object")
    return value


def require_list(value: Any, name: str) -> list[Any]:
    if not isinstance(value, list):
        fail(f"{name} must be an array")
    return value


def read_json(path: Path, name: str) -> dict[str, Any]:
    if not path.exists():
        fail(f"missing {name}: {path}")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"invalid {name} JSON: {exc}")
    return require_object(value, name)


def validate_training_evidence(evidence: dict[str, Any]) -> None:
    if evidence.get("schema") != TRAINING_EVIDENCE_SCHEMA:
        fail("training evidence schema mismatch")
    if evidence.get("authority") != "global_executive_workspace_core":
        fail("training evidence authority mismatch")
    if evidence.get("claim_allowed") is not False or evidence.get("agi_claim") is not False:
        fail("training evidence claims must remain false")
    run = require_object(evidence.get("run"), "training_evidence.run")
    if run.get("passed") is not True:
        fail("training evidence run.passed must be true")
    if run.get("network") != "none":
        fail("training evidence run.network must be none")
    if run.get("external_model_dependency") is not False:
        fail("training evidence must not depend on an external model")
    if int(run.get("model_parameters", 0)) < 6_900_000_000:
        fail("training evidence is not 7B-scale")
    checkpoint_policy = require_object(
        evidence.get("checkpoint_policy"), "training_evidence.checkpoint_policy"
    )
    if checkpoint_policy.get("checkpoint_written") is not True:
        fail("training evidence must show a written checkpoint")
    for field in ("checkpoint_admission", "weights_admitted", "production_model"):
        if checkpoint_policy.get(field) is not False:
            fail(f"training evidence {field} must remain false")


def normalize_response(raw_response: dict[str, Any]) -> list[dict[str, Any]]:
    responses = require_list(raw_response.get("responses"), "response.responses")
    normalized: list[dict[str, Any]] = []
    for idx, response in enumerate(responses):
        record = require_object(response, f"response.responses[{idx}]")
        prompt = str(record.get("prompt", ""))
        generated_text = str(record.get("generated_text", ""))
        generated_tokens = record.get("generated_tokens", [])
        if not prompt:
            fail(f"response {idx} prompt is empty")
        if not generated_text:
            fail(f"response {idx} generated_text is empty")
        if not isinstance(generated_tokens, list):
            generated_tokens = [str(generated_tokens)]
        normalized.append(
            {
                "prompt": prompt,
                "generated_text": generated_text,
                "generated_tokens": generated_tokens,
                "prompt_in_output": generated_text.startswith(prompt),
            }
        )
    if not normalized:
        fail("at least one generated response is required")
    return normalized


def build_report(output_dir: Path, schema_path: Path) -> dict[str, Any]:
    summary_path = output_dir / "eden_7b_inference.summary"
    log_path = output_dir / "eden_7b_inference_probe.log"
    response_path = output_dir / "eden_7b_inference_response.json"
    training_evidence_path = output_dir / "eden_7b_training_evidence.json"
    checkpoint_dir = output_dir / "checkpoints"
    checkpoint_latest = checkpoint_dir / "latest_checkpointed_iteration.txt"

    if not summary_path.exists():
        fail(f"missing summary file: {summary_path}")
    if not log_path.exists():
        fail(f"missing inference log: {log_path}")
    if not checkpoint_latest.exists():
        fail(f"missing checkpoint latest iteration marker: {checkpoint_latest}")

    summary = key_values(summary_path.read_text(encoding="utf-8", errors="replace"))
    response = read_json(response_path, "inference response")
    training_evidence = read_json(training_evidence_path, "training evidence")
    validate_training_evidence(training_evidence)
    normalized_responses = normalize_response(response)

    tokens_to_generate = int(summary.get("tokens_to_generate", "0"))
    if tokens_to_generate <= 0:
        fail("tokens_to_generate must be positive")
    checkpoint_files = [path for path in checkpoint_dir.rglob("*") if path.is_file()]
    if not checkpoint_files:
        fail("checkpoint directory has no files")

    report = {
        "schema": SCHEMA,
        "authority": "global_executive_workspace_core",
        "claim_allowed": False,
        "agi_claim": False,
        "accepted_as": "7b_checkpoint_inference_probe",
        "source": {
            "summary_path": str(summary_path),
            "log_path": str(log_path),
            "response_path": str(response_path),
            "training_evidence_path": str(training_evidence_path),
            "checkpoint_path": str(checkpoint_dir),
            "checkpoint_latest_iteration": checkpoint_latest.read_text(
                encoding="utf-8", errors="replace"
            ).strip(),
            "summary_fnv64": fnv64(summary_path.read_bytes()),
            "log_fnv64": fnv64(log_path.read_bytes()),
            "response_fnv64": fnv64(response_path.read_bytes()),
            "training_evidence_fnv64": fnv64(training_evidence_path.read_bytes()),
            "schema_path": str(schema_path),
        },
        "run": {
            "passed": parse_bool(summary.get("eden_7b_inference_probe_passed", "false")),
            "image": summary.get("image", "unknown"),
            "network": summary.get("network", "unknown"),
            "model_id": summary.get("model_id", "unknown"),
            "model_scale": summary.get("model_scale", "unknown"),
            "tokenizer": summary.get("tokenizer", "unknown"),
            "external_model_dependency": parse_bool(
                summary.get("external_model_dependency", "true")
            ),
            "checkpoint_loaded": parse_bool(summary.get("checkpoint_loaded", "false")),
            "checkpoint_admission": parse_bool(summary.get("checkpoint_admission", "true")),
            "production_model": parse_bool(summary.get("production_model", "true")),
            "generated_count": len(normalized_responses),
            "tokens_to_generate": tokens_to_generate,
        },
        "responses": normalized_responses,
        "training_summary": {
            "model_parameters": training_evidence["run"]["model_parameters"],
            "completed_iterations": training_evidence["run"]["completed_iterations"],
            "final_loss": training_evidence["run"]["final_loss"],
            "checkpoint_written": training_evidence["checkpoint_policy"]["checkpoint_written"],
            "checkpoint_admission": False,
        },
        "safety_boundary": {
            "direct_memory_writes": False,
            "direct_objective_writes": False,
            "direct_tool_execution": False,
            "requires_gewc_admission": True,
            "outputs_are_hypotheses": True,
            "model_may_not_mutate_runtime_state": True,
        },
        "runtime_use": {
            "accepted_for": [
                "checkpoint_load_probe",
                "token_generation_probe",
                "governed_model_adapter_status",
            ],
            "not_accepted_for": [
                "agi_claim",
                "semantic_competence_claim",
                "external_validation",
                "checkpoint_release",
                "autonomous_runtime_authority",
                "production_inference",
            ],
        },
    }
    validate_report(report, schema_path)
    return report


def validate_report(report: dict[str, Any], schema_path: Path) -> None:
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    if report.get("schema") != schema["properties"]["schema"]["const"]:
        fail("schema mismatch")
    if report.get("authority") != "global_executive_workspace_core":
        fail("authority must be global_executive_workspace_core")
    if report.get("claim_allowed") is not False or report.get("agi_claim") is not False:
        fail("claims must remain false")
    run = require_object(report.get("run"), "run")
    if run.get("passed") is not True:
        fail("run.passed must be true")
    if run.get("network") != "none":
        fail("run.network must be none")
    if run.get("external_model_dependency") is not False:
        fail("run.external_model_dependency must be false")
    if run.get("checkpoint_loaded") is not True:
        fail("run.checkpoint_loaded must be true")
    if run.get("checkpoint_admission") is not False:
        fail("run.checkpoint_admission must remain false")
    if run.get("production_model") is not False:
        fail("run.production_model must remain false")
    if int(run.get("generated_count", 0)) < 1:
        fail("run.generated_count must be positive")
    if int(run.get("tokens_to_generate", 0)) < 1:
        fail("run.tokens_to_generate must be positive")
    safety_boundary = require_object(report.get("safety_boundary"), "safety_boundary")
    for field in ("direct_memory_writes", "direct_objective_writes", "direct_tool_execution"):
        if safety_boundary.get(field) is not False:
            fail(f"safety_boundary.{field} must remain false")


def main() -> int:
    parser = argparse.ArgumentParser(description="Build EDEN Megatron 7B inference report.")
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--schema", type=Path, default=DEFAULT_SCHEMA_PATH)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()

    report_path = args.report or args.output_dir / "eden_7b_inference_report.json"
    report = build_report(args.output_dir, args.schema)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {report_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
