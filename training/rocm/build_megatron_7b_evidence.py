#!/usr/bin/env python3
"""Build stdlib-only evidence for the EDEN-owned Megatron 7B pilot."""

from __future__ import annotations

import argparse
import json
import math
import re
from pathlib import Path
from typing import Any


SCHEMA = "eden.megatron.7b.training_evidence.v1"
DEFAULT_OUTPUT_DIR = Path("target/eden_megatron_7b_base_pilot")
DEFAULT_SCHEMA_PATH = Path("contracts/v1/schemas/eden-megatron-7b-training-evidence-v1.json")


def fail(message: str) -> None:
    raise SystemExit(f"Megatron 7B evidence validation failed: {message}")


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


def latest_iteration(log_text: str) -> dict[str, Any]:
    pattern = re.compile(
        r"iteration\s+(?P<iteration>\d+)/\s*(?P<total>\d+).*?"
        r"elapsed time per iteration \(ms\):\s*(?P<elapsed>[0-9.E+-]+).*?"
        r"lm loss:\s*(?P<loss>[0-9.E+-]+).*?"
        r"grad norm:\s*(?P<grad>[0-9.E+-]+).*?"
        r"number of skipped iterations:\s*(?P<skipped>\d+).*?"
        r"number of nan iterations:\s*(?P<nan>\d+)",
        re.IGNORECASE,
    )
    matches = list(pattern.finditer(log_text))
    if not matches:
        fail("no Megatron iteration line found in log")
    match = matches[-1]
    loss = float(match.group("loss"))
    grad_norm = float(match.group("grad"))
    elapsed_ms = float(match.group("elapsed"))
    if not math.isfinite(loss) or loss <= 0:
        fail("final loss must be finite and positive")
    if not math.isfinite(grad_norm):
        fail("grad norm must be finite")
    return {
        "iteration": int(match.group("iteration")),
        "total": int(match.group("total")),
        "elapsed_ms": elapsed_ms,
        "loss": loss,
        "grad_norm": grad_norm,
        "skipped": int(match.group("skipped")),
        "nan": int(match.group("nan")),
    }


def parse_train_iters(log_text: str, fallback: int) -> int:
    match = re.search(r"train_iters\s+\.+\s+(\d+)", log_text)
    return int(match.group(1)) if match else fallback


def parse_parameters(log_text: str) -> tuple[int, float]:
    parameter_match = re.search(r"number of parameters.*?:\s*(\d+)", log_text)
    billion_match = re.search(r"Total number of parameters in billions:\s*([0-9.]+)", log_text)
    if not parameter_match:
        fail("model parameter count missing from log")
    parameters = int(parameter_match.group(1))
    billions = float(billion_match.group(1)) if billion_match else parameters / 1_000_000_000
    if parameters < 6_900_000_000:
        fail("model parameter count is below 7B-scale threshold")
    return parameters, billions


def parse_memory(log_text: str) -> dict[str, float]:
    match = re.search(
        r"memory \(MB\).*?allocated:\s*([0-9.]+).*?"
        r"max allocated:\s*([0-9.]+).*?"
        r"reserved:\s*([0-9.]+).*?"
        r"max reserved:\s*([0-9.]+)",
        log_text,
    )
    if not match:
        return {}
    return {
        "allocated_mb": float(match.group(1)),
        "max_allocated_mb": float(match.group(2)),
        "reserved_mb": float(match.group(3)),
        "max_reserved_mb": float(match.group(4)),
    }


def corpus_metrics(output_dir: Path, repo_root: Path) -> dict[str, int]:
    jsonl_path = output_dir / "data" / "eden_corpus.jsonl"
    corpus_root = repo_root / "eden_core" / "corpus"
    file_count = len([path for path in corpus_root.rglob("*.txt") if path.is_file()])
    line_count = 0
    char_count = 0
    if jsonl_path.exists():
        for raw_line in jsonl_path.read_text(encoding="utf-8", errors="replace").splitlines():
            if not raw_line.strip():
                continue
            line_count += 1
            try:
                char_count += len(json.loads(raw_line).get("text", ""))
            except json.JSONDecodeError:
                fail(f"invalid JSONL corpus line in {jsonl_path}")
    return {
        "files": file_count,
        "jsonl_lines": line_count,
        "jsonl_chars": char_count,
    }


def has_files(path: Path) -> bool:
    return path.exists() and any(child.is_file() for child in path.rglob("*"))


def build_evidence(output_dir: Path, schema_path: Path, repo_root: Path) -> dict[str, Any]:
    summary_path = output_dir / "eden_7b_base_pilot.summary"
    log_path = output_dir / "eden_7b_base_pilot.log"
    if not summary_path.exists():
        fail(f"missing summary file: {summary_path}")
    if not log_path.exists():
        fail(f"missing log file: {log_path}")

    summary_text = summary_path.read_text(encoding="utf-8", errors="replace")
    log_text = log_path.read_text(encoding="utf-8", errors="replace")
    values = key_values(summary_text)
    iteration = latest_iteration(log_text)
    parameters, billions = parse_parameters(log_text)
    train_iters = parse_train_iters(log_text, iteration["total"])
    checkpoint_dir = output_dir / "checkpoints"
    checkpoint_written = has_files(checkpoint_dir)

    evidence = {
        "schema": SCHEMA,
        "authority": "global_executive_workspace_core",
        "claim_allowed": False,
        "agi_claim": False,
        "accepted_as": "7b_training_path_evidence",
        "source": {
            "summary_path": str(summary_path),
            "log_path": str(log_path),
            "summary_fnv64": fnv64(summary_path.read_bytes()),
            "log_fnv64": fnv64(log_path.read_bytes()),
            "schema_path": str(schema_path),
        },
        "run": {
            "passed": parse_bool(values.get("eden_megatron_7b_base_pilot_passed", "false")),
            "image": values.get("image", "unknown"),
            "network": values.get("network", "unknown"),
            "model_scale": values.get("model_scale", "unknown"),
            "tokenizer": values.get("tokenizer", "unknown"),
            "dataset": values.get("dataset", "unknown"),
            "mock_data": parse_bool(values.get("mock_data", "true")),
            "external_model_dependency": parse_bool(values.get("external_model_dependency", "true")),
            "train_iters": train_iters,
            "completed_iterations": iteration["iteration"],
            "model_parameters": parameters,
            "total_parameters_billion": billions,
            "final_loss": iteration["loss"],
            "final_grad_norm": iteration["grad_norm"],
            "elapsed_ms_last_iteration": iteration["elapsed_ms"],
            "skipped_iterations": iteration["skipped"],
            "nan_iterations": iteration["nan"],
            "memory": parse_memory(log_text),
        },
        "model_config": {
            "architecture": "gpt_megatron_random_init",
            "layers": 32,
            "hidden_size": 4096,
            "ffn_hidden_size": 12288,
            "attention_heads": 32,
            "sequence_length": 128,
            "parallelism": {
                "tensor_model_parallel_size": 1,
                "pipeline_model_parallel_size": 1,
                "data_parallel_size": 1,
            },
        },
        "corpus": corpus_metrics(output_dir, repo_root),
        "checkpoint_policy": {
            "checkpoint_written": checkpoint_written,
            "checkpoint_path": str(checkpoint_dir),
            "checkpoint_admission": False,
            "weights_admitted": False,
            "production_model": False,
            "reason": "pilot evidence may prove training execution only; checkpoint promotion requires separate evaluation and approval",
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
                "7b_shape_training_path_evidence",
                "loss_curve_observation",
                "rocm_megatron_integration_status",
            ],
            "not_accepted_for": [
                "agi_claim",
                "external_validation",
                "checkpoint_release",
                "autonomous_runtime_authority",
                "production_inference",
            ],
        },
    }
    validate_evidence(evidence, schema_path)
    return evidence


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(f"{name} must be an object")
    return value


def validate_evidence(evidence: dict[str, Any], schema_path: Path) -> None:
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    if evidence.get("schema") != schema["properties"]["schema"]["const"]:
        fail("schema mismatch")
    if evidence.get("authority") != "global_executive_workspace_core":
        fail("authority must be global_executive_workspace_core")
    if evidence.get("claim_allowed") is not False or evidence.get("agi_claim") is not False:
        fail("claims must remain false")

    run = require_object(evidence.get("run"), "run")
    if run.get("passed") is not True:
        fail("run.passed must be true")
    if run.get("network") != "none":
        fail("run.network must be none")
    if run.get("mock_data") is not False:
        fail("run.mock_data must be false")
    if run.get("external_model_dependency") is not False:
        fail("run.external_model_dependency must be false")
    if int(run.get("train_iters", 0)) < 1:
        fail("run.train_iters must be positive")
    if int(run.get("completed_iterations", 0)) < 1:
        fail("run.completed_iterations must be positive")
    if int(run.get("model_parameters", 0)) < 6_900_000_000:
        fail("run.model_parameters must be 7B-scale")
    if int(run.get("nan_iterations", 1)) != 0:
        fail("run.nan_iterations must be zero")

    checkpoint_policy = require_object(evidence.get("checkpoint_policy"), "checkpoint_policy")
    if checkpoint_policy.get("checkpoint_admission") is not False:
        fail("checkpoint admission must remain false")
    if checkpoint_policy.get("weights_admitted") is not False:
        fail("weights_admitted must remain false")
    if checkpoint_policy.get("production_model") is not False:
        fail("production_model must remain false")

    safety_boundary = require_object(evidence.get("safety_boundary"), "safety_boundary")
    for field in ("direct_memory_writes", "direct_objective_writes", "direct_tool_execution"):
        if safety_boundary.get(field) is not False:
            fail(f"safety_boundary.{field} must remain false")


def main() -> int:
    parser = argparse.ArgumentParser(description="Build EDEN Megatron 7B training evidence.")
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--schema", type=Path, default=DEFAULT_SCHEMA_PATH)
    parser.add_argument("--repo-root", type=Path, default=Path("."))
    parser.add_argument("--evidence", type=Path)
    args = parser.parse_args()

    output_dir = args.output_dir
    evidence_path = args.evidence or output_dir / "eden_7b_training_evidence.json"
    evidence = build_evidence(output_dir, args.schema, args.repo_root)
    evidence_path.parent.mkdir(parents=True, exist_ok=True)
    evidence_path.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {evidence_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
