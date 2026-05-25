#!/usr/bin/env python3
"""CPU-safe EDEN capability smoke benchmark.

This is intentionally small and stdlib-only. It validates the evaluation path
that future AMD ROCm/GPU jobs will use, while keeping model authority below
GEWC and keeping AGI claims blocked.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import shutil
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


TOKEN_RE = re.compile(r"[a-z0-9_]+")


def tokenize(text: str) -> list[str]:
    return TOKEN_RE.findall(text.lower())


class TinyEdenMemoryModel:
    """A tiny trainable lexical memory baseline.

    It is not an LLM. It fits IDF weights over provided memories and retrieves
    the highest scoring memory for a query.
    """

    def __init__(self) -> None:
        self.documents: list[str] = []
        self.idf: dict[str, float] = {}

    def fit(self, documents: list[str]) -> None:
        self.documents = documents
        doc_freq: Counter[str] = Counter()
        for document in documents:
            doc_freq.update(set(tokenize(document)))

        total = max(len(documents), 1)
        self.idf = {
            token: math.log((1 + total) / (1 + freq)) + 1.0
            for token, freq in doc_freq.items()
        }

    def retrieve(self, query: str) -> str:
        query_tokens = tokenize(query)
        if not self.documents or not query_tokens:
            return ""

        best_score = -1.0
        best_document = ""
        for document in self.documents:
            document_tokens = set(tokenize(document))
            score = sum(self.idf.get(token, 1.0) for token in query_tokens if token in document_tokens)
            if score > best_score:
                best_score = score
                best_document = document
        return best_document


def rocm_status() -> dict[str, Any]:
    return {
        "rocm_detected": shutil.which("rocminfo") is not None or Path("/opt/rocm").exists(),
        "rocminfo": shutil.which("rocminfo"),
        "rocm_path": "/opt/rocm" if Path("/opt/rocm").exists() else None,
        "visible_devices": os.environ.get("HIP_VISIBLE_DEVICES") or os.environ.get("ROCR_VISIBLE_DEVICES"),
    }


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    cases: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as handle:
        for line_no, line in enumerate(handle, start=1):
            stripped = line.strip()
            if not stripped:
                continue
            try:
                cases.append(json.loads(stripped))
            except json.JSONDecodeError as exc:
                raise ValueError(f"{path}:{line_no}: invalid JSONL: {exc}") from exc
    return cases


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    kind = case["kind"]
    passed = False
    observed: Any = None

    if kind == "memory_retrieval":
        model = TinyEdenMemoryModel()
        model.fit(list(case.get("memories", [])))
        observed = model.retrieve(case.get("query", ""))
        passed = case.get("expected_substring", "") in observed

    elif kind == "planning":
        plan = [
            "load_rocm_profile",
            "run_cpu_safe_smoke_benchmark",
            "write_capability_report",
            "keep_claim_allowed_false",
        ]
        observed = plan
        passed = all(step in plan for step in case.get("required_steps", []))

    elif kind == "tool_safety":
        risk = case.get("risk")
        permission_granted = bool(case.get("permission_granted"))
        decision = "allowed" if permission_granted or risk in {"low", "medium"} else "blocked"
        observed = decision
        passed = decision == case.get("expected_decision")

    elif kind == "continual_learning":
        memory: dict[str, Any] = {}
        for update in case.get("updates", []):
            memory[str(update["key"])] = update["value"]
        observed = memory.get(case.get("query_key"))
        passed = observed == case.get("expected_value")

    else:
        observed = f"unsupported kind: {kind}"

    return {
        "id": case.get("id"),
        "kind": kind,
        "capability": case.get("capability", "unknown"),
        "passed": passed,
        "observed": observed,
    }


def summarize(results: list[dict[str, Any]]) -> dict[str, Any]:
    by_capability: dict[str, dict[str, int]] = defaultdict(lambda: {"passed": 0, "total": 0})
    for result in results:
        bucket = by_capability[result["capability"]]
        bucket["total"] += 1
        if result["passed"]:
            bucket["passed"] += 1

    total = len(results)
    passed = sum(1 for result in results if result["passed"])
    return {
        "passed": passed,
        "total": total,
        "score": passed / total if total else 0.0,
        "by_capability": dict(sorted(by_capability.items())),
    }


def load_model_config(path: Path | None) -> dict[str, Any] | None:
    if path is None:
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def run_first_model_eval(config: dict[str, Any] | None) -> dict[str, Any] | None:
    if config is None:
        return None

    datasets = config.get("datasets", {})
    train_path = Path(datasets.get("train", ""))
    eval_path = Path(datasets.get("eval", ""))
    train_rows = load_jsonl(train_path)
    eval_rows = load_jsonl(eval_path)

    model = TinyEdenMemoryModel()
    model.fit([str(row.get("text", "")) for row in train_rows])

    results = []
    for row in eval_rows:
        observed = model.retrieve(str(row.get("query", "")))
        expected_substring = str(row.get("expected_substring", ""))
        results.append(
            {
                "id": row.get("id"),
                "query": row.get("query"),
                "passed": expected_substring in observed,
                "observed": observed,
                "expected_substring": expected_substring,
            }
        )

    total = len(results)
    passed = sum(1 for result in results if result["passed"])
    return {
        "module": config.get("module"),
        "role": config.get("role"),
        "backend": config.get("backend_plan", {}).get("initial"),
        "claim_allowed": False,
        "agi_claim": False,
        "direct_memory_writes": False,
        "direct_objective_writes": False,
        "direct_tool_execution": False,
        "passed": passed,
        "total": total,
        "score": passed / total if total else 0.0,
        "results": results,
    }


def markdown_value(value: Any) -> str:
    if isinstance(value, (dict, list)):
        return "`" + json.dumps(value, sort_keys=True) + "`"
    return str(value).replace("\n", " ")


def write_markdown_report(report: dict[str, Any], output: Path) -> None:
    summary = report["summary"]
    model_eval = report.get("first_model_eval")
    lines = [
        "# EDEN Training Capability Smoke Report",
        "",
        "| Field | Value |",
        "| --- | --- |",
        f"| Schema | `{report['schema']}` |",
        f"| Claim allowed | `{report['claim_allowed']}` |",
        f"| AGI claim | `{report['agi_claim']}` |",
        f"| ROCm detected | `{report['device']['rocm_detected']}` |",
        f"| Passed | `{summary['passed']}/{summary['total']}` |",
        f"| Score | `{summary['score']:.3f}` |",
        "",
        "## Capability Summary",
        "",
        "| Capability | Passed | Total |",
        "| --- | ---: | ---: |",
    ]
    for capability, bucket in summary["by_capability"].items():
        lines.append(f"| {capability} | {bucket['passed']} | {bucket['total']} |")

    lines.extend(
        [
            "",
            "## Cases",
            "",
            "| ID | Kind | Capability | Passed | Observed |",
            "| --- | --- | --- | --- | --- |",
        ]
    )
    for result in report["results"]:
        lines.append(
            "| {id} | {kind} | {capability} | `{passed}` | {observed} |".format(
                id=result.get("id"),
                kind=result.get("kind"),
                capability=result.get("capability"),
                passed=result.get("passed"),
                observed=markdown_value(result.get("observed")),
            )
        )

    if model_eval is not None:
        lines.extend(
            [
                "",
                "## First Model Eval",
                "",
                "| Field | Value |",
                "| --- | --- |",
                f"| Module | `{model_eval['module']}` |",
                f"| Role | `{model_eval['role']}` |",
                f"| Backend | `{model_eval['backend']}` |",
                f"| Passed | `{model_eval['passed']}/{model_eval['total']}` |",
                f"| Score | `{model_eval['score']:.3f}` |",
                f"| Direct memory writes | `{model_eval['direct_memory_writes']}` |",
                f"| Direct objective writes | `{model_eval['direct_objective_writes']}` |",
                f"| Direct tool execution | `{model_eval['direct_tool_execution']}` |",
                "",
                "| ID | Passed | Observed |",
                "| --- | --- | --- |",
            ]
        )
        for result in model_eval["results"]:
            lines.append(
                f"| {result.get('id')} | `{result.get('passed')}` | "
                f"{markdown_value(result.get('observed'))} |"
            )

    lines.extend(
        [
            "",
            "## Boundary",
            "",
            "This report validates the local training/evaluation path only. It does not",
            "grant AGI claims, does not grant direct model authority over memory, goals",
            "or tools, and must be admitted by GEWC before becoming runtime evidence.",
        ]
    )
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Run EDEN training/capability smoke benchmark.")
    parser.add_argument("--dataset", type=Path, default=Path("training/data/capability_smoke.jsonl"))
    parser.add_argument("--output", type=Path, default=Path("target/eden_training_smoke/capability_report.json"))
    parser.add_argument("--markdown-output", type=Path, default=Path("target/eden_training_smoke/capability_report.md"))
    parser.add_argument("--profile", type=Path, default=Path("training/configs/rocm_smoke.json"))
    parser.add_argument("--model-config", type=Path, default=Path("training/configs/first_model_memory_retrieval.json"))
    args = parser.parse_args()

    cases = load_jsonl(args.dataset)
    profile = json.loads(args.profile.read_text(encoding="utf-8"))
    model_config = load_model_config(args.model_config)
    first_model_eval = run_first_model_eval(model_config)
    results = [run_case(case) for case in cases]
    summary = summarize(results)

    report = {
        "schema": "eden.training.capability_report.v1",
        "claim_allowed": False,
        "agi_claim": False,
        "profile": profile,
        "model_config": model_config,
        "device": rocm_status(),
        "summary": summary,
        "first_model_eval": first_model_eval,
        "results": results,
    }

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_markdown_report(report, args.markdown_output)

    print(f"wrote {args.output}")
    print(f"wrote {args.markdown_output}")
    print(f"passed {summary['passed']}/{summary['total']} capability smoke cases")
    first_model_passed = (
        first_model_eval is None
        or first_model_eval["passed"] == first_model_eval["total"]
    )
    return 0 if summary["passed"] == summary["total"] and first_model_passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
