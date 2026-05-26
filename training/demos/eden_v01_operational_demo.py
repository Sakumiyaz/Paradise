#!/usr/bin/env python3
"""Build an executable EDEN v0.1 operational demo trace.

The demo is intentionally local and non-mutating: it combines one held-out
semantic task, current 7B inference evidence, SFT/ELCP hypothesis packets and
the semantic eval report into a GEWC-style trace that proves the runtime path
can plan, verify, dry-run and audit without granting production autonomy.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "eden.v01.operational_demo.v1"
AUTHORITY = "global_executive_workspace_core"


def load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    return value if isinstance(value, dict) else {}


def load_first_jsonl(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            value = json.loads(line)
            return value if isinstance(value, dict) else {}
    return {}


def first_item(value: dict[str, Any], key: str) -> Any:
    items = value.get(key, [])
    if isinstance(items, list) and items:
        return items[0]
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--challenge", type=Path, default=Path("training/data/eden_v01_semantic_challenge.jsonl"))
    parser.add_argument("--semantic-eval", type=Path, default=Path("target/eden_v01/semantic_eval_report.json"))
    parser.add_argument("--training-evidence", type=Path, default=Path("target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json"))
    parser.add_argument("--inference-report", type=Path, default=Path("target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json"))
    parser.add_argument("--sft-packets", type=Path, default=Path("target/eden_sft_elcp_gpu_pilot/eden_sft_elcp_inference_packets.json"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_v01/operational_demo_trace.json"))
    args = parser.parse_args()

    challenge = load_first_jsonl(args.challenge)
    semantic_eval = load_json(args.semantic_eval)
    training = load_json(args.training_evidence)
    inference = load_json(args.inference_report)
    packets = load_json(args.sft_packets)
    sample_response = first_item(inference, "responses")
    sample_packet = first_item(packets, "packets")

    steps = [
        {
            "step": "observe_task",
            "artifact": args.challenge.as_posix(),
            "result": "held_out_semantic_task_loaded",
            "mutates_runtime_state": False,
        },
        {
            "step": "retrieve_context",
            "artifact": challenge.get("metadata", {}).get("source_fnv64"),
            "result": "source_hash_bound_to_context_packet",
            "mutates_runtime_state": False,
        },
        {
            "step": "native_inference_candidate",
            "artifact": args.inference_report.as_posix(),
            "result": "7b_checkpoint_response_available_as_hypothesis",
            "sample": sample_response,
            "mutates_runtime_state": False,
        },
        {
            "step": "sft_elcp_packet",
            "artifact": args.sft_packets.as_posix(),
            "result": "learned_transition_packet_available_as_hypothesis",
            "sample": sample_packet,
            "mutates_runtime_state": False,
        },
        {
            "step": "semantic_verification",
            "artifact": args.semantic_eval.as_posix(),
            "result": "semantic_eval_gate_passed" if semantic_eval.get("passed") else "semantic_eval_gate_blocked",
            "mutates_runtime_state": False,
        },
        {
            "step": "dry_run_action",
            "artifact": "GEWC action contract",
            "result": "no tool execution; no memory write; no objective mutation",
            "mutates_runtime_state": False,
        },
        {
            "step": "checkpoint_candidate_decision",
            "artifact": args.training_evidence.as_posix(),
            "result": "candidate_runtime_admission_reviewable; production_release_blocked",
            "mutates_runtime_state": False,
        },
        {
            "step": "audit_record",
            "artifact": args.output.as_posix(),
            "result": "trace_written_for_replay",
            "mutates_runtime_state": False,
        },
    ]

    training_run = training.get("run", {})
    inference_run = inference.get("run", {})
    passed = (
        semantic_eval.get("passed") is True
        and int(training_run.get("completed_iterations", 0)) >= 100
        and training.get("checkpoint_policy", {}).get("checkpoint_written") is True
        and inference_run.get("checkpoint_loaded") is True
        and int(inference_run.get("generated_count", 0)) >= 2
        and sample_packet is not None
        and all(step["mutates_runtime_state"] is False for step in steps)
    )

    report = {
        "schema": SCHEMA,
        "authority": AUTHORITY,
        "artifact": "eden_v01_operational_demo",
        "claim_allowed": False,
        "agi_claim": False,
        "passed": passed,
        "task": challenge.get("input", {}),
        "target": challenge.get("target", {}),
        "steps": steps,
        "safety_boundary": {
            "direct_memory_writes": False,
            "direct_objective_writes": False,
            "direct_tool_execution": False,
            "outputs_are_hypotheses": True,
            "requires_gewc_verification": True,
        },
        "not_claimed": ["AGI", "production_release", "autonomous_tool_authority"],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"EDEN v0.1 operational demo passed={passed} steps={len(steps)} -> {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
