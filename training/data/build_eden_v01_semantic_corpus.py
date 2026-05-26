#!/usr/bin/env python3
"""Build the EDEN v0.1 semantic capability corpus from repo-owned sources.

The corpus is larger than the real-capability gate corpus and is aimed at
semantic runtime behavior: situation modeling, planning, memory, uncertainty,
tool policy, causal/world-model reasoning, checkpoint admission and rollback.
It is deterministic, stdlib-only, offline and redacts credential-shaped
literals before writing examples.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter
from pathlib import Path
from typing import Any


SCHEMA = "eden.v01.semantic_corpus_row.v1"
MANIFEST_SCHEMA = "eden.v01.semantic_corpus_manifest.v1"
AUTHORITY = "global_executive_workspace_core"
DEFAULT_MAX_ROWS = 2048

SOURCE_GLOBS = [
    "docs/**/*.md",
    "eden_core/corpus/**/*.txt",
    "eden_core/src/garm/**/*.rs",
    "training/**/*.md",
    "training/**/*.json",
    "training/**/*.py",
    "training/**/*.sh",
]

TASKS = [
    "situation_model",
    "hierarchical_plan",
    "memory_retrieval",
    "causal_world_model",
    "tool_policy",
    "uncertainty_calibration",
    "semantic_verification",
    "checkpoint_admission",
    "rollback_recovery",
    "runtime_observability",
    "safety_authority",
    "scaling_decision",
]

CATEGORY_RULES = [
    ("safety", ("safety", "policy", "permission", "guard", "sandbox", "risk")),
    ("training", ("training", "dataset", "sft", "elcp", "megatron", "rocm", "loss")),
    ("inference", ("inference", "token", "response", "model", "adapter", "checkpoint")),
    ("memory", ("memory", "retrieval", "locus", "context", "episodic", "semantic")),
    ("world_model", ("world", "simulation", "causal", "predict", "state", "counterfactual")),
    ("tools", ("tool", "mcp", "action", "dry-run", "execute", "api")),
    ("evaluation", ("eval", "benchmark", "validation", "metric", "test", "gate")),
    ("runtime", ("runtime", "gewc", "garm", "kernel", "workspace", "router")),
    ("observability", ("audit", "trace", "log", "evidence", "provenance", "replay")),
    ("scaling", ("scale", "scaling", "14b", "7b", "parameters", "gpu")),
]


def fnv64(data: bytes) -> str:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value:016x}"


def normalize(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def redact_sensitive_literals(text: str) -> str:
    text = re.sub(
        r"\b([A-Z0-9_]*(?:API_KEY|TOKEN|PASSWORD|SECRET))=(['\"])[^'\"]+(['\"])",
        r"\1=\2<redacted-demo-value>\3",
        text,
    )
    text = re.sub(
        r"\b([A-Z0-9_]*(?:API_KEY|TOKEN|PASSWORD|SECRET))=[^\s`]+",
        r"\1=<redacted-demo-value>",
        text,
    )
    return re.sub(
        r"-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
        "<redacted-private-key>",
        text,
        flags=re.DOTALL,
    )


def category_for(text: str) -> str:
    lowered = text.lower()
    scores = {
        category: sum(1 for token in tokens if token in lowered)
        for category, tokens in CATEGORY_RULES
    }
    category, score = max(scores.items(), key=lambda item: (item[1], item[0]))
    return category if score > 0 else "runtime"


def chunks_for(path: Path, root: Path) -> list[dict[str, Any]]:
    try:
        body = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []
    rel = path.relative_to(root).as_posix()
    source_hash = fnv64(body.encode("utf-8", errors="replace"))
    blocks = re.split(r"\n\s*\n|(?<=\.)\s+(?=[A-Z`#])", body)
    rows: list[dict[str, Any]] = []
    for block_index, block in enumerate(blocks):
        excerpt = redact_sensitive_literals(normalize(block))
        if len(excerpt) < 80:
            continue
        rows.append(
            {
                "source_path": rel,
                "source_fnv64": source_hash,
                "block_index": block_index,
                "category": category_for(excerpt),
                "excerpt": excerpt[:700],
            }
        )
    return rows


def collect_sources(root: Path, max_rows: int) -> list[dict[str, Any]]:
    candidates: list[Path] = []
    for pattern in SOURCE_GLOBS:
        candidates.extend(path for path in root.glob(pattern) if path.is_file())
    candidates = sorted(set(candidates), key=lambda path: path.relative_to(root).as_posix())
    rows: list[dict[str, Any]] = []
    for path in candidates:
        rows.extend(chunks_for(path, root))
    rows.sort(key=lambda row: (row["category"], row["source_path"], row["block_index"]))
    return balanced_sample(rows, max_rows)


def balanced_sample(rows: list[dict[str, Any]], max_rows: int) -> list[dict[str, Any]]:
    by_category: dict[str, list[dict[str, Any]]] = {}
    for row in rows:
        by_category.setdefault(row["category"], []).append(row)
    categories = sorted(by_category)
    selected: list[dict[str, Any]] = []
    cursor = 0
    while len(selected) < max_rows and any(by_category.values()):
        category = categories[cursor % len(categories)]
        if by_category[category]:
            selected.append(by_category[category].pop(0))
        cursor += 1
    selected.sort(key=lambda row: (row["source_path"], row["block_index"], row["category"]))
    return selected


def evidence_for(task_type: str, category: str) -> list[str]:
    base = {
        "situation_model": ["state_snapshot", "authority_labels", "active_goal"],
        "hierarchical_plan": ["goal_contract", "preconditions", "risk_budget"],
        "memory_retrieval": ["retrieval_trace", "source_hash", "permission_filter"],
        "causal_world_model": ["state_delta", "counterfactual", "uncertainty"],
        "tool_policy": ["tool_contract", "dry_run", "approval_scope"],
        "uncertainty_calibration": ["confidence_interval", "missing_evidence", "abstain_rule"],
        "semantic_verification": ["source_trace", "verifier_result", "contradiction_check"],
        "checkpoint_admission": ["training_evidence", "semantic_eval", "rollback_plan"],
        "rollback_recovery": ["snapshot_id", "undo_plan", "postcondition_check"],
        "runtime_observability": ["trace_id", "event_log", "replay_record"],
        "safety_authority": ["authority_parser", "policy_gate", "circuit_breaker"],
        "scaling_decision": ["loss_curve", "eval_delta", "gpu_budget"],
    }[task_type]
    return base + [f"category_{category}"]


def make_row(raw: dict[str, Any], index: int) -> dict[str, Any]:
    task_type = TASKS[(index - 1) % len(TASKS)]
    category = raw["category"]
    return {
        "schema": SCHEMA,
        "id": f"eden-v01-semantic-{index:05d}",
        "input": {
            "task_type": task_type,
            "category_hint": category,
            "source_path": raw["source_path"],
            "evidence_excerpt": raw["excerpt"],
            "question": "What governed semantic operation should GEWC perform before runtime state can change?",
        },
        "target": {
            "authority": AUTHORITY,
            "operation": f"{task_type}_through_gewc",
            "semantic_output_kind": "structured_hypothesis",
            "required_evidence": evidence_for(task_type, category),
            "requires_verification": True,
            "direct_memory_write": False,
            "direct_tool_execution": False,
            "direct_objective_update": False,
            "claim_allowed": False,
            "agi_claim": False,
        },
        "metadata": {
            "source_fnv64": raw["source_fnv64"],
            "block_index": raw["block_index"],
            "contains_private_data": False,
            "external_model_dependency": False,
            "curriculum_stage": "eden_v01_semantic_capability",
        },
    }


def split_rows(rows: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    splits = {"train": [], "eval": [], "challenge": []}
    for index, row in enumerate(rows):
        bucket = index % 10
        if bucket < 7:
            splits["train"].append(row)
        elif bucket < 9:
            splits["eval"].append(row)
        else:
            splits["challenge"].append(row)
    return splits


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(row, sort_keys=True) + "\n")


def manifest_for(paths: dict[str, Path], splits: dict[str, list[dict[str, Any]]]) -> dict[str, Any]:
    all_rows = [row for rows in splits.values() for row in rows]
    task_types = Counter(row["input"]["task_type"] for row in all_rows)
    categories = Counter(row["input"]["category_hint"] for row in all_rows)
    return {
        "schema": MANIFEST_SCHEMA,
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "contains_private_data": False,
        "external_model_dependency": False,
        "rows": {name: len(rows) for name, rows in splits.items()},
        "total_rows": len(all_rows),
        "task_types": dict(sorted(task_types.items())),
        "categories": dict(sorted(categories.items())),
        "paths": {name: str(path) for name, path in paths.items()},
        "accepted_for": [
            "semantic_capability_eval",
            "native_inference_runtime_admission",
            "checkpoint_candidate_review",
        ],
        "not_accepted_for": ["AGI_claim", "private_memory_training", "production_release"],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--max-rows", type=int, default=DEFAULT_MAX_ROWS)
    parser.add_argument("--output-dir", type=Path, default=Path("training/data"))
    parser.add_argument("--manifest", type=Path, default=Path("target/eden_v01/semantic_corpus_manifest.json"))
    args = parser.parse_args()

    root = args.repo_root.resolve()
    raw_rows = collect_sources(root, args.max_rows)
    rows = [make_row(raw, index) for index, raw in enumerate(raw_rows, start=1)]
    if len(rows) < 2_000:
        raise SystemExit(f"not enough EDEN v0.1 semantic rows: {len(rows)} < 2000")
    splits = split_rows(rows)
    paths = {
        "train": args.output_dir / "eden_v01_semantic_train.jsonl",
        "eval": args.output_dir / "eden_v01_semantic_eval.jsonl",
        "challenge": args.output_dir / "eden_v01_semantic_challenge.jsonl",
    }
    for name, path in paths.items():
        write_jsonl(path, splits[name])
    manifest = manifest_for(paths, splits)
    args.manifest.parent.mkdir(parents=True, exist_ok=True)
    args.manifest.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "EDEN v0.1 semantic corpus "
        f"train={len(splits['train'])} eval={len(splits['eval'])} "
        f"challenge={len(splits['challenge'])} -> {args.output_dir}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
