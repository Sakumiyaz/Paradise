#!/usr/bin/env python3
"""Build a visible non-GPU Paradise demo transcript and evidence manifest."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any


COMMANDS = [
    {
        "name": "status",
        "argv": ["cargo", "run", "-p", "eden_core", "--bin", "paradise", "--", "status"],
    },
    {
        "name": "worldcell",
        "argv": ["cargo", "run", "-p", "eden_core", "--bin", "paradise", "--", "worldcell"],
    },
    {
        "name": "dry_run",
        "argv": [
            "cargo",
            "run",
            "-p",
            "eden_core",
            "--bin",
            "paradise",
            "--",
            "run",
            "--dry-run",
            "inspect runtime status safely",
        ],
    },
]


def run_command(argv: list[str], state_dir: Path) -> dict[str, Any]:
    command = [*argv[:7], "--state-dir", state_dir.as_posix(), *argv[7:]]
    env = os.environ.copy()
    env["EDEN_GARM_SKIP_LEGACY_MIGRATION"] = "1"
    result = subprocess.run(
        command,
        cwd=Path("."),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=120,
    )
    return {
        "command": " ".join(command),
        "exit_code": result.returncode,
        "output": result.stdout[-6000:],
    }


def main() -> int:
    output_dir = Path("target/paradise_public_demo")
    state_dir = output_dir / "state"
    output_dir.mkdir(parents=True, exist_ok=True)
    state_dir.mkdir(parents=True, exist_ok=True)

    results = [
        {"name": spec["name"], **run_command(spec["argv"], state_dir)}
        for spec in COMMANDS
    ]
    passed = all(result["exit_code"] == 0 for result in results)
    transcript_lines = [
        "# Paradise Public Demo",
        "",
        "Non-GPU demo transcript. No model weights, network, sockets, training or",
        "checkpoint admission are required.",
        "",
    ]
    for result in results:
        transcript_lines.extend(
            [
                f"## {result['name']}",
                "",
                "```bash",
                result["command"],
                "```",
                "",
                "```text",
                result["output"].strip(),
                "```",
                "",
            ]
        )
    transcript = output_dir / "demo_transcript.md"
    transcript.write_text("\n".join(transcript_lines), encoding="utf-8")
    manifest = {
        "schema": "paradise.public_demo.v1",
        "artifact": "paradise_public_demo",
        "authority": "global_executive_workspace_core",
        "claim_allowed": False,
        "agi_claim": False,
        "production_model_allowed": False,
        "checkpoint_admission_allowed": False,
        "gpu_required": False,
        "transcript": transcript.as_posix(),
        "state_dir": state_dir.as_posix(),
        "commands": results,
        "passed": passed,
    }
    output = output_dir / "public_demo_manifest.json"
    output.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"[paradise-public-demo] passed={passed} transcript={transcript} manifest={output}")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
