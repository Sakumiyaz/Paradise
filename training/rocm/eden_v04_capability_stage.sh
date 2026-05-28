#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

RUN_GPU="${EDEN_V04_RUN_GPU:-false}"
RUN_SFT="${EDEN_V04_RUN_SFT:-true}"
LONG_ITERS="${EDEN_V04_LONG_ITERS:-10000}"
LONG_DIR="${EDEN_V04_LONG_OUTPUT_DIR:-${REPO_ROOT}/target/eden_megatron_7b_v04_long_10000}"
V04_DIR="${EDEN_V04_OUTPUT_DIR:-${REPO_ROOT}/target/eden_v04}"
PURGE_CHECKPOINTS="${EDEN_V04_PURGE_LOCAL_CHECKPOINTS:-false}"
REUSE_EXISTING="${EDEN_V04_REUSE_EXISTING_GPU_EVIDENCE:-true}"

usage() {
  cat <<'EOF'
Usage: training/rocm/eden_v04_capability_stage.sh

Builds the EDEN v0.4 operational capability stage:
1. v0.4 cognitive capability corpus;
2. optional 10k-iteration 7B Megatron training on ROCm;
3. optional checkpoint-load generative probe;
4. optional compact cognitive SFT/ELCP GPU pilot;
5. hard checkpoint admission report;
6. persistent inference service contract;
7. continuity evaluation and 14B preflight report.

Set EDEN_V04_RUN_GPU=true on a ROCm/Megatron host to run missing GPU work.
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
  *) fail "EDEN_V04_RUN_GPU must be true or false" ;;
esac
case "$RUN_SFT" in
  true | false) ;;
  *) fail "EDEN_V04_RUN_SFT must be true or false" ;;
esac
case "$PURGE_CHECKPOINTS" in
  true | false) ;;
  *) fail "EDEN_V04_PURGE_LOCAL_CHECKPOINTS must be true or false" ;;
esac
case "$REUSE_EXISTING" in
  true | false) ;;
  *) fail "EDEN_V04_REUSE_EXISTING_GPU_EVIDENCE must be true or false" ;;
esac
[[ "$LONG_ITERS" =~ ^[0-9]+$ ]] || fail "EDEN_V04_LONG_ITERS must be an integer"
(( LONG_ITERS >= 10000 )) || fail "EDEN_V04_LONG_ITERS must be >= 10000"

cd "$REPO_ROOT"
mkdir -p "$V04_DIR"
python3 training/data/build_eden_v04_cognitive_capability_corpus.py

if [[ "$RUN_GPU" == "true" ]]; then
  if [[ "$REUSE_EXISTING" == "true" && -f "$LONG_DIR/eden_7b_training_evidence.json" ]]; then
    printf 'eden_v04_reuse_training_evidence=true\n'
  else
    EDEN_MEGATRON_7B_OUTPUT_DIR="$LONG_DIR" \
    EDEN_MEGATRON_7B_MASTER_PORT="${EDEN_V04_LONG_MASTER_PORT:-6526}" \
    EDEN_MEGATRON_7B_TRAIN_ITERS="$LONG_ITERS" \
    EDEN_MEGATRON_7B_SAVE_CHECKPOINT=true \
    EDEN_MEGATRON_7B_SAVE_INTERVAL="$LONG_ITERS" \
      bash training/rocm/megatron_eden_7b_base_pilot.sh
  fi

  if [[ "$REUSE_EXISTING" == "true" && -f "$LONG_DIR/eden_7b_inference_report.json" ]]; then
    printf 'eden_v04_reuse_inference_report=true\n'
  else
    EDEN_MEGATRON_7B_OUTPUT_DIR="$LONG_DIR" \
    EDEN_MEGATRON_7B_INFER_MASTER_PORT="${EDEN_V04_LONG_INFER_MASTER_PORT:-6536}" \
    EDEN_MEGATRON_7B_TOKENS="${EDEN_V04_INFER_TOKENS:-16}" \
    EDEN_MEGATRON_7B_PROMPTS_JSON='["EDEN v0.4 continuity state:","GEWC hard checkpoint admission:","Persistent inference service contract:","14B scaling preflight:"]' \
      bash training/rocm/megatron_eden_7b_inference_probe.sh
  fi

  copy_report "$LONG_DIR/eden_7b_training_evidence.json" "$V04_DIR/eden_7b_long_${LONG_ITERS}_training_evidence.json"
  copy_report "$LONG_DIR/eden_7b_inference_report.json" "$V04_DIR/eden_7b_long_${LONG_ITERS}_inference_report.json"

  if [[ "$RUN_SFT" == "true" ]]; then
    python3 training/data/build_eden_cognitive_sft_elcp.py
    EDEN_SFT_ELCP_OUTPUT_DIR="${EDEN_V04_SFT_OUTPUT_DIR:-${REPO_ROOT}/target/eden_sft_elcp_gpu_pilot}" \
      bash training/rocm/eden_sft_elcp_gpu_pilot.sh
  fi
else
  if [[ -f "$LONG_DIR/eden_7b_training_evidence.json" ]]; then
    copy_report "$LONG_DIR/eden_7b_training_evidence.json" "$V04_DIR/eden_7b_long_${LONG_ITERS}_training_evidence.json"
  fi
  if [[ -f "$LONG_DIR/eden_7b_inference_report.json" ]]; then
    copy_report "$LONG_DIR/eden_7b_inference_report.json" "$V04_DIR/eden_7b_long_${LONG_ITERS}_inference_report.json"
  fi
fi

python3 training/benchmarks/eden_v04_operational_capability_eval.py --min-iters "$LONG_ITERS"
bash training/rocm/eden_gpu_workspace_hygiene.sh

if [[ "$PURGE_CHECKPOINTS" == "true" ]]; then
  rm -rf -- "$LONG_DIR/checkpoints"
fi

printf 'eden_v04_capability_stage_ready=true\n'
