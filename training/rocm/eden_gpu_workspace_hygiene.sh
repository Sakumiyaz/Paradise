#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

RUN_ROOT="${EDEN_GPU_RUN_ROOT:-/root/paradise-gpu/runs}"
OUTPUT="${EDEN_GPU_HYGIENE_OUTPUT:-${REPO_ROOT}/target/eden_v01/gpu_workspace_hygiene_report.json}"

usage() {
  cat <<'EOF'
Usage: training/rocm/eden_gpu_workspace_hygiene.sh

Writes a non-destructive GPU workspace hygiene report. It inventories the
current repo, run directory, Docker image availability and EDEN target evidence
without deleting checkpoints or training outputs. Cleanup remains an explicit
operator action outside this report.

Environment:
  EDEN_GPU_RUN_ROOT         Run directory to inspect. Default: /root/paradise-gpu/runs
  EDEN_GPU_HYGIENE_OUTPUT   Report path. Default: target/eden_v01/gpu_workspace_hygiene_report.json
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

cd "$REPO_ROOT"
mkdir -p "$(dirname -- "$OUTPUT")"

python3 - "$RUN_ROOT" "$OUTPUT" <<'PY'
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any


AUTHORITY = "global_executive_workspace_core"
SCHEMA = "eden.v01.gpu_workspace_hygiene.v1"


def run(cmd: list[str]) -> tuple[int, str]:
    try:
        completed = subprocess.run(cmd, check=False, text=True, capture_output=True)
    except OSError as exc:
        return 127, str(exc)
    return completed.returncode, (completed.stdout + completed.stderr).strip()


def dir_size(path: Path) -> int:
    total = 0
    if not path.exists():
        return total
    try:
        children = path.rglob("*")
        for child in children:
            try:
                if child.is_file():
                    total += child.stat().st_size
            except OSError:
                pass
    except OSError:
        pass
    return total


def path_exists(path: Path) -> bool:
    try:
        return path.exists()
    except OSError:
        return False


def readable_dir(path: Path) -> bool:
    try:
        return path.is_dir() and os.access(path, os.R_OK | os.X_OK)
    except OSError:
        return False


def iter_dirs(path: Path) -> tuple[list[Path], str | None]:
    if not path_exists(path):
        return [], None
    if not readable_dir(path):
        return [], "permission_denied"
    try:
        return sorted(child for child in path.iterdir() if child.is_dir()), None
    except OSError as exc:
        return [], f"{type(exc).__name__}: {exc}"


def marker_exists(path: Path) -> bool:
    try:
        return path.exists()
    except OSError:
        return False


run_root = Path(sys.argv[1])
output = Path(sys.argv[2])
repo_root = Path.cwd()
git_base = ["git", "-c", f"safe.directory={repo_root}"]
git_status_code, git_status = run([*git_base, "status", "--short"])
git_head_code, git_head = run([*git_base, "log", "-1", "--oneline"])
docker_code, docker_info = run(["docker", "image", "inspect", "rocm/megatron-lm:v25.3", "--format", "{{.Id}}"])

run_dirs: list[dict[str, Any]] = []
run_dir_error: str | None
children, run_dir_error = iter_dirs(run_root)
for child in children:
    run_dirs.append(
        {
            "path": str(child),
            "bytes": dir_size(child),
            "has_git": marker_exists(child / ".git"),
            "has_7b_evidence": marker_exists(child / "target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json"),
            "has_inference_report": marker_exists(child / "target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json"),
        }
    )

report = {
    "schema": SCHEMA,
    "authority": AUTHORITY,
    "artifact": "eden_v01_gpu_workspace_hygiene",
    "claim_allowed": False,
    "agi_claim": False,
    "destructive_apply": False,
    "repo": {
        "head": git_head if git_head_code == 0 else None,
        "dirty": bool(git_status.strip()) if git_status_code == 0 else None,
        "status": git_status.splitlines()[:50] if git_status else [],
    },
    "gpu_workspace": {
        "run_root": str(run_root),
        "run_root_exists": path_exists(run_root),
        "run_root_readable": readable_dir(run_root),
        "inspection_error": run_dir_error,
        "run_dir_count": len(run_dirs),
        "run_dirs": run_dirs,
    },
    "docker": {
        "docker_available": shutil.which("docker") is not None,
        "rocm_megatron_image_present": docker_code == 0,
        "rocm_megatron_image_id": docker_info if docker_code == 0 else None,
    },
    "cleanup_policy": {
        "delete_checkpoints_automatically": False,
        "delete_training_outputs_automatically": False,
        "recommended_layout": "/root/paradise-gpu/runs/<purpose>-<commit>",
        "operator_can_remove_old_run_dirs_after_copying_target_evidence": True,
    },
    "environment": {
        "cwd": os.getcwd(),
        "network_required": False,
    },
}
output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"EDEN GPU workspace hygiene report -> {output}")
PY
