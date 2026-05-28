#!/usr/bin/env python3
"""Review local checkpoint/inference evidence without admitting checkpoints."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


PUBLIC_REGISTRY = Path("training/models/checkpoint_registry.json")
EVIDENCE_PATHS = {
    "training_evidence": Path("target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json"),
    "inference_report": Path("target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json"),
    "v04_hard_admission": Path("target/eden_v04/hard_checkpoint_admission_report.json"),
    "v04_generative_probe": Path("target/eden_v04/generative_probe_report.json"),
}


def load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    value = json.loads(path.read_text(encoding="utf-8"))
    return value if isinstance(value, dict) else {}


def main() -> int:
    output_dir = Path("target/paradise_checkpoint_evidence_review")
    output_dir.mkdir(parents=True, exist_ok=True)
    registry = load_json(PUBLIC_REGISTRY)
    evidence = {name: load_json(path) for name, path in EVIDENCE_PATHS.items()}
    inference = evidence["inference_report"]
    hard_admission = evidence["v04_hard_admission"]
    generated = evidence["v04_generative_probe"]
    training = evidence["training_evidence"]

    checkpoint_load_probe_seen = (
        inference.get("run", {}).get("checkpoint_loaded") is True
        and inference.get("run", {}).get("checkpoint_admission") is False
    )
    token_probe_seen = int(inference.get("run", {}).get("generated_count", 0) or 0) > 0
    training_probe_seen = bool(training.get("training_summary") or training.get("run"))
    candidate_gate_seen = hard_admission.get("candidate_runtime_admission_allowed") is True
    semantic_probe_seen = generated.get("runtime_smoke_passed") is True

    checks = [
        {"check": "public_registry_present", "passed": bool(registry)},
        {
            "check": "public_registry_no_active_checkpoint",
            "passed": registry.get("active_checkpoint") is None,
        },
        {
            "check": "public_registry_blocks_production",
            "passed": registry.get("production_model_allowed") is False
            and registry.get("agi_claim") is False,
        },
        {
            "check": "checkpoint_load_probe_seen_if_evidence_exists",
            "passed": not EVIDENCE_PATHS["inference_report"].exists() or checkpoint_load_probe_seen,
        },
        {
            "check": "token_probe_seen_if_evidence_exists",
            "passed": not EVIDENCE_PATHS["inference_report"].exists() or token_probe_seen,
        },
        {
            "check": "training_probe_seen_if_evidence_exists",
            "passed": not EVIDENCE_PATHS["training_evidence"].exists() or training_probe_seen,
        },
        {
            "check": "candidate_gate_seen_if_evidence_exists",
            "passed": not EVIDENCE_PATHS["v04_hard_admission"].exists() or candidate_gate_seen,
        },
        {
            "check": "semantic_probe_seen_if_evidence_exists",
            "passed": not EVIDENCE_PATHS["v04_generative_probe"].exists() or semantic_probe_seen,
        },
    ]
    present_evidence = [
        {
            "name": name,
            "path": path.as_posix(),
            "present": path.exists(),
            "schema": evidence[name].get("schema"),
        }
        for name, path in EVIDENCE_PATHS.items()
    ]
    report = {
        "schema": "paradise.checkpoint_evidence_review.v1",
        "artifact": "paradise_checkpoint_evidence_review",
        "authority": "global_executive_workspace_core",
        "claim_allowed": False,
        "agi_claim": False,
        "production_model_allowed": False,
        "checkpoint_admission_allowed": False,
        "gpu_required": False,
        "public_registry": {
            "path": PUBLIC_REGISTRY.as_posix(),
            "present": PUBLIC_REGISTRY.exists(),
            "active_checkpoint": registry.get("active_checkpoint"),
            "entry_count": len(registry.get("entries", [])) if isinstance(registry.get("entries"), list) else 0,
        },
        "local_evidence": present_evidence,
        "candidate_probe": {
            "checkpoint_load_probe_seen": checkpoint_load_probe_seen,
            "token_probe_seen": token_probe_seen,
            "training_probe_seen": training_probe_seen,
            "candidate_gate_seen": candidate_gate_seen,
            "semantic_probe_seen": semantic_probe_seen,
            "model_id": inference.get("run", {}).get("model_id"),
            "generated_count": inference.get("run", {}).get("generated_count", 0),
        },
        "decision": "keep_public_checkpoint_registry_blocked; local probe evidence may be reviewed but not released as production",
        "required_before_real_admission": [
            "checkpoint_files_available_in_artifact_store",
            "checkpoint_hash_manifest",
            "held_out_semantic_eval",
            "safety_verifier_eval",
            "rollback_drill",
            "operator_release_approval",
        ],
        "checks": checks,
    }
    report["passed"] = all(check["passed"] for check in checks)
    output = output_dir / "checkpoint_evidence_review.json"
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(
        "[paradise-checkpoint-evidence-review] "
        f"passed={report['passed']} evidence_present="
        f"{sum(1 for item in present_evidence if item['present'])}/{len(present_evidence)} "
        f"path={output}"
    )
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
