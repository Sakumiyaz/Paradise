#!/usr/bin/env python3
"""Train a small EDEN-owned SFT/ELCP pilot on GPU.

This is intentionally not a 7B checkpoint fine-tune. It is the first real
learned cognitive-transition module: a compact multi-head classifier trained on
repo-local EDEN SFT/ELCP fixtures. Its outputs are structured hypothesis
packets that remain subordinate to GEWC.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import time
from collections import Counter
from pathlib import Path
from typing import Any


SCHEMA = "eden.sft_elcp.gpu_pilot.v1"
PREPOST_SCHEMA = "eden.sft_elcp.prepost_eval.v1"
PACKET_SCHEMA = "eden.sft_elcp.inference_packets.v1"
ADMISSION_SCHEMA = "eden.sft_elcp.checkpoint_admission_review.v1"
AUTHORITY = "global_executive_workspace_core"
TARGET_FIELDS = [
    "next_situation",
    "next_goal_state",
    "memory_transition",
    "world_delta",
    "plan_transition",
    "action_affordance",
    "uncertainty",
    "safety_gate",
    "learning_update",
]


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            stripped = line.strip()
            if stripped:
                rows.append(json.loads(stripped))
    return rows


def fnv64(data: bytes) -> str:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value:016x}"


def text_for(row: dict[str, Any]) -> str:
    input_obj = row.get("input", {})
    parts = [
        input_obj.get("surface_text", ""),
        input_obj.get("situation", ""),
        input_obj.get("goal", ""),
        input_obj.get("plan_state", ""),
        input_obj.get("risk_context", ""),
        " ".join(str(item) for item in input_obj.get("working_memory", [])),
        " ".join(str(item) for item in input_obj.get("available_tools", [])),
        json.dumps(input_obj.get("world_state", {}), sort_keys=True),
    ]
    return " ".join(str(part) for part in parts)


def tokenize(text: str) -> list[str]:
    return re.findall(r"[a-z0-9_]+", text.lower())


def build_vocab(rows: list[dict[str, Any]], min_count: int) -> dict[str, int]:
    counts: Counter[str] = Counter()
    for row in rows:
        counts.update(tokenize(text_for(row)))
    tokens = ["<unk>"] + sorted(token for token, count in counts.items() if count >= min_count)
    return {token: index for index, token in enumerate(tokens)}


def vectorize(row: dict[str, Any], vocab: dict[str, int]) -> list[float]:
    vector = [0.0] * len(vocab)
    for token in tokenize(text_for(row)):
        vector[vocab.get(token, 0)] += 1.0
    total = sum(vector) or 1.0
    return [value / total for value in vector]


def build_label_maps(rows: list[dict[str, Any]]) -> dict[str, dict[str, int]]:
    maps: dict[str, dict[str, int]] = {}
    for field in TARGET_FIELDS:
        labels = sorted({str(row.get("target", {}).get(field, "")) for row in rows})
        maps[field] = {label: index for index, label in enumerate(labels)}
    return maps


def import_torch() -> Any:
    import torch
    from torch import nn

    return torch, nn


def evaluate(model: Any, tensors: dict[str, Any], rows: list[dict[str, Any]], label_maps: dict[str, dict[str, int]], torch: Any) -> dict[str, Any]:
    model.eval()
    with torch.no_grad():
        outputs = model(tensors["x"])
    field_results: dict[str, dict[str, int]] = {}
    row_results: list[dict[str, Any]] = []
    row_passed = 0
    total_fields = 0
    passed_fields = 0
    reverse_maps = {
        field: {index: label for label, index in mapping.items()}
        for field, mapping in label_maps.items()
    }
    for row_index, row in enumerate(rows):
        prediction: dict[str, str] = {}
        target: dict[str, str] = {}
        fields: dict[str, bool] = {}
        for field in TARGET_FIELDS:
            pred_index = int(outputs[field][row_index].argmax().item())
            pred = reverse_maps[field][pred_index]
            truth = str(row.get("target", {}).get(field, ""))
            prediction[field] = pred
            target[field] = truth
            ok = pred == truth
            fields[field] = ok
            bucket = field_results.setdefault(field, {"passed": 0, "total": 0})
            bucket["total"] += 1
            total_fields += 1
            if ok:
                bucket["passed"] += 1
                passed_fields += 1
        passed = all(fields.values())
        if passed:
            row_passed += 1
        row_results.append({
            "id": row.get("id"),
            "passed": passed,
            "prediction": prediction,
            "target": target,
            "field_results": fields,
        })
    return {
        "passed_fields": passed_fields,
        "total_fields": total_fields,
        "field_score": passed_fields / total_fields if total_fields else 0.0,
        "passed_rows": row_passed,
        "total_rows": len(rows),
        "row_score": row_passed / len(rows) if rows else 0.0,
        "by_field": field_results,
        "results": row_results,
    }


def tensors_for(rows: list[dict[str, Any]], vocab: dict[str, int], label_maps: dict[str, dict[str, int]], device: Any, torch: Any) -> dict[str, Any]:
    x = torch.tensor([vectorize(row, vocab) for row in rows], dtype=torch.float32, device=device)
    y = {
        field: torch.tensor(
            [label_maps[field][str(row.get("target", {}).get(field, ""))] for row in rows],
            dtype=torch.long,
            device=device,
        )
        for field in TARGET_FIELDS
    }
    return {"x": x, "y": y}


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Run EDEN SFT/ELCP GPU pilot.")
    parser.add_argument("--train", type=Path, default=Path("training/data/eden_cognitive_sft_elcp_train.jsonl"))
    parser.add_argument("--eval", type=Path, default=Path("training/data/eden_cognitive_sft_elcp_eval.jsonl"))
    parser.add_argument("--output-dir", type=Path, default=Path("target/eden_sft_elcp_gpu_pilot"))
    parser.add_argument("--epochs", type=int, default=int(os.environ.get("EDEN_SFT_ELCP_EPOCHS", "120")))
    parser.add_argument("--hidden-size", type=int, default=int(os.environ.get("EDEN_SFT_ELCP_HIDDEN", "96")))
    parser.add_argument("--lr", type=float, default=float(os.environ.get("EDEN_SFT_ELCP_LR", "0.05")))
    parser.add_argument("--min-count", type=int, default=1)
    parser.add_argument("--seed", type=int, default=2718)
    parser.add_argument("--require-gpu", action="store_true")
    args = parser.parse_args()

    torch, nn = import_torch()
    torch.manual_seed(args.seed)
    if torch.cuda.is_available():
        device = torch.device("cuda")
    else:
        if args.require_gpu:
            raise SystemExit("GPU required but torch.cuda.is_available() is false")
        device = torch.device("cpu")

    train_rows = load_jsonl(args.train)
    eval_rows = load_jsonl(args.eval)
    all_rows = train_rows + eval_rows
    vocab = build_vocab(train_rows, args.min_count)
    label_maps = build_label_maps(all_rows)
    train_tensors = tensors_for(train_rows, vocab, label_maps, device, torch)
    eval_tensors = tensors_for(eval_rows, vocab, label_maps, device, torch)

    class EdenSftElcpModel(nn.Module):
        def __init__(self) -> None:
            super().__init__()
            self.encoder = nn.Sequential(
                nn.Linear(len(vocab), args.hidden_size),
                nn.ReLU(),
                nn.Linear(args.hidden_size, args.hidden_size),
                nn.ReLU(),
            )
            self.heads = nn.ModuleDict({
                field: nn.Linear(args.hidden_size, len(label_maps[field]))
                for field in TARGET_FIELDS
            })

        def forward(self, x: Any) -> dict[str, Any]:
            hidden = self.encoder(x)
            return {field: head(hidden) for field, head in self.heads.items()}

    model = EdenSftElcpModel().to(device)
    optimizer = torch.optim.AdamW(model.parameters(), lr=args.lr, weight_decay=0.01)
    loss_fn = nn.CrossEntropyLoss()

    started = time.time()
    pre_eval = evaluate(model, eval_tensors, eval_rows, label_maps, torch)
    losses: list[float] = []
    for epoch in range(args.epochs):
        model.train()
        optimizer.zero_grad(set_to_none=True)
        outputs = model(train_tensors["x"])
        loss = sum(loss_fn(outputs[field], train_tensors["y"][field]) for field in TARGET_FIELDS)
        loss.backward()
        optimizer.step()
        losses.append(float(loss.detach().cpu().item()))
    post_eval = evaluate(model, eval_tensors, eval_rows, label_maps, torch)
    elapsed_ms = int((time.time() - started) * 1000)

    checkpoint_path = args.output_dir / "eden_sft_elcp_gpu_pilot.pt"
    checkpoint_path.parent.mkdir(parents=True, exist_ok=True)
    torch.save(
        {
            "model_state": model.state_dict(),
            "vocab": vocab,
            "label_maps": label_maps,
            "target_fields": TARGET_FIELDS,
            "schema": SCHEMA,
            "authority": AUTHORITY,
        },
        checkpoint_path,
    )
    checkpoint_hash = hashlib.sha256(checkpoint_path.read_bytes()).hexdigest()

    packets = []
    for result in post_eval["results"][: min(8, len(post_eval["results"]))]:
        packets.append({
            "schema": "eden.structured_inference_packet.v1",
            "packet_id": f"sft-elcp-{result['id']}",
            "source_model": "eden-sft-elcp-gpu-pilot",
            "candidate_structure": {
                "kind": "learned_cognitive_transition_hypothesis",
                "prediction": result["prediction"],
                "requires_verification": True,
                "memory_action": "audit_only",
                "objective_action": "none",
                "tool_action": "none",
            },
            "authority": {
                "GEWC_final_authority": True,
                "model_may_not_mutate_state": True,
                "accepted_as_truth": False,
            },
        })

    threshold_field_score = 0.90
    threshold_row_score = 0.70
    release_allowed = (
        post_eval["field_score"] >= threshold_field_score
        and post_eval["row_score"] >= threshold_row_score
    )
    common = {
        "authority": AUTHORITY,
        "claim_allowed": False,
        "agi_claim": False,
        "external_model_dependency": False,
        "direct_memory_writes": False,
        "direct_objective_writes": False,
        "direct_tool_execution": False,
    }
    prepost_report = {
        "schema": PREPOST_SCHEMA,
        **common,
        "artifact": "eden_sft_elcp_prepost_eval",
        "pre_eval": pre_eval,
        "post_eval": post_eval,
        "delta": {
            "field_score": post_eval["field_score"] - pre_eval["field_score"],
            "row_score": post_eval["row_score"] - pre_eval["row_score"],
        },
    }
    packet_report = {
        "schema": PACKET_SCHEMA,
        **common,
        "artifact": "eden_sft_elcp_inference_packets",
        "packet_count": len(packets),
        "packets": packets,
    }
    admission = {
        "schema": ADMISSION_SCHEMA,
        **common,
        "artifact": "eden_sft_elcp_checkpoint_admission_review",
        "checkpoint_path": str(checkpoint_path),
        "checkpoint_sha256": checkpoint_hash,
        "thresholds": {
            "field_score": threshold_field_score,
            "row_score": threshold_row_score,
        },
        "metrics": {
            "pre_field_score": pre_eval["field_score"],
            "post_field_score": post_eval["field_score"],
            "pre_row_score": pre_eval["row_score"],
            "post_row_score": post_eval["row_score"],
        },
        "checkpoint_admission_allowed": False,
        "release_candidate": release_allowed,
        "reason": "pilot may become a release candidate only after GEWC review; direct admission remains blocked",
        "required_before_admission": [
            "GEWC_review",
            "external_regression",
            "adversarial_prompt_injection_eval",
            "rollback_drill",
            "operator_approval",
        ],
    }
    training_report = {
        "schema": SCHEMA,
        **common,
        "artifact": "eden_sft_elcp_gpu_pilot",
        "training_executed": True,
        "gpu_job_submitted": device.type == "cuda",
        "device": str(device),
        "torch_version": torch.__version__,
        "epochs": args.epochs,
        "hidden_size": args.hidden_size,
        "learning_rate": args.lr,
        "elapsed_ms": elapsed_ms,
        "datasets": {
            "train": str(args.train),
            "eval": str(args.eval),
            "train_rows": len(train_rows),
            "eval_rows": len(eval_rows),
            "train_fnv64": fnv64(args.train.read_bytes()),
            "eval_fnv64": fnv64(args.eval.read_bytes()),
        },
        "loss": {
            "initial": losses[0] if losses else None,
            "final": losses[-1] if losses else None,
            "points": losses,
        },
        "checkpoint_path": str(checkpoint_path),
        "checkpoint_sha256": checkpoint_hash,
        "prepost_eval_path": str(args.output_dir / "eden_sft_elcp_prepost_eval.json"),
        "packet_report_path": str(args.output_dir / "eden_sft_elcp_inference_packets.json"),
        "admission_review_path": str(args.output_dir / "eden_sft_elcp_checkpoint_admission_review.json"),
    }

    write_json(args.output_dir / "eden_sft_elcp_prepost_eval.json", prepost_report)
    write_json(args.output_dir / "eden_sft_elcp_inference_packets.json", packet_report)
    write_json(args.output_dir / "eden_sft_elcp_checkpoint_admission_review.json", admission)
    write_json(args.output_dir / "eden_sft_elcp_training_report.json", training_report)
    print(
        "EDEN SFT/ELCP GPU pilot "
        f"device={device} pre_field={pre_eval['field_score']:.3f} "
        f"post_field={post_eval['field_score']:.3f} release_candidate={release_allowed} "
        f"-> {args.output_dir / 'eden_sft_elcp_training_report.json'}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
