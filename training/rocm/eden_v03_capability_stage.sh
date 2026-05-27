#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

RUN_GPU="${EDEN_V03_RUN_GPU:-false}"
LONG_ITERS="${EDEN_V03_LONG_ITERS:-1000}"
LONG_DIR="${EDEN_V03_LONG_OUTPUT_DIR:-${REPO_ROOT}/target/eden_megatron_7b_v03_long_1000}"
V03_DIR="${EDEN_V03_OUTPUT_DIR:-${REPO_ROOT}/target/eden_v03}"
PURGE_CHECKPOINTS="${EDEN_V03_PURGE_LOCAL_CHECKPOINTS:-false}"

usage() {
  cat <<'EOF'
Usage: training/rocm/eden_v03_capability_stage.sh

Builds the EDEN v0.3 capability stage:
1. curated v0.3 generalization corpus;
2. optional 1000-iteration 7B Megatron continuation on ROCm;
3. optional checkpoint-load inference probe;
4. generalization and checkpoint-admission report;
5. persistent native inference runtime contract;
6. checkpoint registry and 14B scaling plan;
7. non-mutating operational demo and GPU hygiene report.

Set EDEN_V03_RUN_GPU=true on a ROCm/Megatron host to run the long 7B job.
Generated reports are evidence only. Weights are never committed to git.
EOF
}

fail() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

copy_report() {
  local -r src="$1"
  local -r dst="$2"
  [[ -f "$src" ]] || fail "missing report: $src"
  cp "$src" "$dst"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

case "$RUN_GPU" in
  true | false) ;;
  *) fail "EDEN_V03_RUN_GPU must be true or false" ;;
esac
case "$PURGE_CHECKPOINTS" in
  true | false) ;;
  *) fail "EDEN_V03_PURGE_LOCAL_CHECKPOINTS must be true or false" ;;
esac
[[ "$LONG_ITERS" =~ ^[0-9]+$ ]] || fail "EDEN_V03_LONG_ITERS must be an integer"
(( LONG_ITERS >= 1 )) || fail "EDEN_V03_LONG_ITERS must be >= 1"

cd "$REPO_ROOT"
mkdir -p "$V03_DIR"
python3 training/data/build_eden_v03_generalization_corpus.py

if [[ "$RUN_GPU" == "true" ]]; then
  EDEN_MEGATRON_7B_OUTPUT_DIR="$LONG_DIR" \
  EDEN_MEGATRON_7B_MASTER_PORT="${EDEN_V03_LONG_MASTER_PORT:-6506}" \
  EDEN_MEGATRON_7B_TRAIN_ITERS="$LONG_ITERS" \
  EDEN_MEGATRON_7B_SAVE_CHECKPOINT=true \
  EDEN_MEGATRON_7B_SAVE_INTERVAL="$LONG_ITERS" \
    bash training/rocm/megatron_eden_7b_base_pilot.sh

  EDEN_MEGATRON_7B_OUTPUT_DIR="$LONG_DIR" \
  EDEN_MEGATRON_7B_INFER_MASTER_PORT="${EDEN_V03_LONG_INFER_MASTER_PORT:-6516}" \
  EDEN_MEGATRON_7B_PROMPTS_JSON='["EDEN v0.3 checkpoint state:","GEWC runtime admission plan:","Memory safety and rollback:"]' \
    bash training/rocm/megatron_eden_7b_inference_probe.sh

  copy_report "$LONG_DIR/eden_7b_training_evidence.json" "$V03_DIR/eden_7b_long_${LONG_ITERS}_training_evidence.json"
  copy_report "$LONG_DIR/eden_7b_inference_report.json" "$V03_DIR/eden_7b_long_${LONG_ITERS}_inference_report.json"

  if [[ "$LONG_ITERS" != "1000" ]]; then
    copy_report "$LONG_DIR/eden_7b_training_evidence.json" "$V03_DIR/eden_7b_long_1000_training_evidence.json"
    copy_report "$LONG_DIR/eden_7b_inference_report.json" "$V03_DIR/eden_7b_long_1000_inference_report.json"
  fi
fi

python3 training/benchmarks/eden_v03_generalization_eval.py --min-iters "$LONG_ITERS"
python3 training/demos/eden_v03_operational_demo.py
bash training/rocm/eden_gpu_workspace_hygiene.sh

if [[ "$PURGE_CHECKPOINTS" == "true" ]]; then
  rm -rf -- "$LONG_DIR/checkpoints"
fi

printf 'eden_v03_capability_stage_ready=true\n'
