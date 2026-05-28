#!/usr/bin/env python3
"""Build a repo-owned EDEN capability corpus from real project artifacts.

The previous SFT/ELCP pilot used compact synthetic transitions. This builder
uses EDEN's own documentation, ADRs and runtime source as the evidence base for
larger capability-evaluation tasks. It does not read private user data and does
not use external models or network access.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter
from pathlib import Path
from typing import Any


SCHEMA = "eden.real_capability.corpus_row.v1"
MANIFEST_SCHEMA = "eden.real_capability.corpus_manifest.v1"
AUTHORITY = "global_executive_workspace_core"
DEFAULT_MAX_ROWS = 420

SOURCE_GLOBS = [
    "docs/**/*.md",
    "eden_core/src/garm/**/*.rs",
    "training/**/*.md",
    "training/**/*.json",
]

CATEGORY_RULES = [
    ("safety", ("safety", "policy", "permission", "guard", "threat", "sandbox")),
    ("checkpoint", ("checkpoint", "weights", "admission", "model_adapter", "7b")),
    ("training", ("training", "dataset", "sft", "elcp", "megatron", "rocm")),
    ("inference", ("inference", "response", "tokens", "model", "adapter")),
    ("memory", ("memory", "retrieval", "locus", "context", "episodic")),
    ("world_model", ("world", "simulation", "causal", "predict", "state")),
    ("tools", ("tool", "mcp", "action", "dry-run", "execute")),
    ("evaluation", ("eval", "benchmark", "validation", "metric", "test")),
    ("runtime", ("runtime", "gewc", "garm", "kernel", "workspace")),
    ("observability", ("audit", "trace", "log", "evidence", "provenance")),
]


def fnv64(data: bytes) -> str:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value:016x}"


def normalize(text: str) -> str:
    text = re.sub(r"\s+", " ", text)
    return text.strip()


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
    blocks = re.split(r"\n\s*\n|(?<=\.)\s+(?=[A-Z`])", body)
    rows: list[dict[str, Any]] = []
    source_hash = fnv64(body.encode("utf-8", errors="replace"))
    for block_index, block in enumerate(blocks):
        excerpt = redact_sensitive_literals(normalize(block))
        if len(excerpt) < 120:
            continue
        excerpt = excerpt[:900]
        category = category_for(excerpt)
        rows.append(
            {
                "source_path": rel,
                "source_fnv64": source_hash,
                "block_index": block_index,
                "category": category,
                "excerpt": excerpt,
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
    """Keep broad capability coverage when the repo has more rows than needed."""
    by_category: dict[str, list[dict[str, Any]]] = {}
    for row in rows:
        by_category.setdefault(row["category"], []).append(row)

    selected: list[dict[str, Any]] = []
    categories = sorted(by_category)
    cursor = 0
    while len(selected) < max_rows and any(by_category.values()):
        category = categories[cursor % len(categories)]
        if by_category[category]:
            selected.append(by_category[category].pop(0))
        cursor += 1

    selected.sort(key=lambda row: (row["source_path"], row["block_index"], row["category"]))
    return selected


def target_for(category: str) -> dict[str, Any]:
    evidence = {
        "safety": ["policy_gate", "permission_trace", "audit_log"],
        "checkpoint": ["training_evidence", "inference_report", "admission_gate"],
        "training": ["dataset_manifest", "training_report", "prepost_eval"],
        "inference": ["model_adapter", "structured_packet", "verifier"],
        "memory": ["retrieval_trace", "authority_label", "write_gate"],
        "world_model": ["state_delta", "causal_hypothesis", "simulation_gate"],
        "tools": ["tool_contract", "dry_run", "approval"],
        "evaluation": ["held_out_case", "score", "regression_delta"],
        "runtime": ["gewc_decision", "state_snapshot", "handler_route"],
        "observability": ["trace_id", "source_hash", "replay_record"],
    }.get(category, ["gewc_decision", "state_snapshot", "audit_log"])
    return {
        "authority": AUTHORITY,
        "decision": f"route_{category}_through_gewc",
        "required_evidence": evidence,
        "safety_gate": "verify_before_runtime_mutation",
        "claim_allowed": False,
        "agi_claim": False,
    }


def make_row(raw: dict[str, Any], index: int) -> dict[str, Any]:
    category = raw["category"]
    return {
        "schema": SCHEMA,
        "id": f"eden-real-capability-{index:04d}",
        "input": {
            "task": "classify_and_govern_runtime_capability_evidence",
            "category_hint": category,
            "source_path": raw["source_path"],
            "evidence_excerpt": raw["excerpt"],
            "question": "Which GEWC-controlled evidence path is required before this can affect runtime state?",
        },
        "target": target_for(category),
        "metadata": {
            "source_fnv64": raw["source_fnv64"],
            "block_index": raw["block_index"],
            "contains_private_data": False,
            "external_model_dependency": False,
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
    categories = Counter(
        row["input"]["category_hint"] for rows in splits.values() for row in rows
    )
    return {
        "schema": MANIFEST_SCHEMA,
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "contains_private_data": False,
        "external_model_dependency": False,
        "rows": {name: len(rows) for name, rows in splits.items()},
        "total_rows": sum(len(rows) for rows in splits.values()),
        "categories": dict(sorted(categories.items())),
        "paths": {name: str(path) for name, path in paths.items()},
        "accepted_for": [
            "capability_eval_suite",
            "checkpoint_admission_review",
            "runtime_governance_regression",
        ],
        "not_accepted_for": ["private_memory_training", "AGI_claim"],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--max-rows", type=int, default=DEFAULT_MAX_ROWS)
    parser.add_argument("--output-dir", type=Path, default=Path("training/data"))
    parser.add_argument("--manifest", type=Path, default=Path("target/eden_real_capability/corpus_manifest.json"))
    args = parser.parse_args()

    root = args.repo_root.resolve()
    raw_rows = collect_sources(root, args.max_rows)
    rows = [make_row(raw, index) for index, raw in enumerate(raw_rows, start=1)]
    if len(rows) < 300:
        raise SystemExit(f"not enough repo-owned capability rows: {len(rows)} < 300")
    splits = split_rows(rows)
    paths = {
        "train": args.output_dir / "eden_real_capability_train.jsonl",
        "eval": args.output_dir / "eden_real_capability_eval.jsonl",
        "challenge": args.output_dir / "eden_real_capability_challenge.jsonl",
    }
    for name, path in paths.items():
        write_jsonl(path, splits[name])
    manifest = manifest_for(paths, splits)
    args.manifest.parent.mkdir(parents=True, exist_ok=True)
    args.manifest.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "EDEN real capability corpus "
        f"train={len(splits['train'])} eval={len(splits['eval'])} "
        f"challenge={len(splits['challenge'])} -> {args.output_dir}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
