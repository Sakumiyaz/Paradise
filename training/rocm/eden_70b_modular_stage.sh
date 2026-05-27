#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
OUT_DIR="${EDEN_70B_OUT_DIR:-"$ROOT/target/eden_70b_modular"}"
RUN_GPU="${EDEN_70B_RUN_GPU:-false}"

mkdir -p "$OUT_DIR"

cat >"$OUT_DIR/eden_70b_modular_launcher_manifest.json" <<'JSON'
{
  "schema": "eden.modular_70b.launcher_plan.v1",
  "authority": "global_executive_workspace_core",
  "claim_allowed": false,
  "agi_claim": false,
  "not_a_single_model": true,
  "network_policy": "offline_by_default",
  "training_executed": false,
  "checkpoint_admission_allowed": false,
  "modules": [
    {"id": "eden_33b_elcp_primary", "parameters": 33000000000, "launcher": "megatron_module_33b_elcp"},
    {"id": "eden_cwm_12b_causal_world_model", "parameters": 12000000000, "launcher": "megatron_module_12b_cwm"},
    {"id": "eden_multimodal_vla_12b", "parameters": 12000000000, "launcher": "megatron_module_12b_vla"},
    {"id": "eden_planner_code_tool_6b", "parameters": 6000000000, "launcher": "megatron_module_6b_planner_tool"},
    {"id": "eden_safety_verifier_4b", "parameters": 4000000000, "launcher": "megatron_module_4b_safety"},
    {"id": "eden_memory_router_retrieval_3b", "parameters": 3000000000, "launcher": "megatron_module_3b_memory_router"}
  ]
}
JSON

if [[ "$RUN_GPU" != "true" ]]; then
  printf '[EDEN-70B-LAUNCHER] mode=plan_only training_executed=false path=%s\n' \
    "$OUT_DIR/eden_70b_modular_launcher_manifest.json"
  exit 0
fi

cat >"$OUT_DIR/eden_70b_modular_gpu_blocked.json" <<'JSON'
{
  "schema": "eden.modular_70b.gpu_block.v1",
  "authority": "global_executive_workspace_core",
  "claim_allowed": false,
  "agi_claim": false,
  "training_executed": false,
  "status": "blocked_before_gpu_training",
  "reason": "70B modular training requires per-module GPU budget, dataset freeze, checkpoint retention policy and explicit operator approval per module."
}
JSON

printf '[EDEN-70B-LAUNCHER] mode=gpu_requested status=blocked training_executed=false path=%s\n' \
  "$OUT_DIR/eden_70b_modular_gpu_blocked.json"
