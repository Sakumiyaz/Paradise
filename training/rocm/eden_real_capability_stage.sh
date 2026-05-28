#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

RUN_GPU="${EDEN_REAL_CAPABILITY_RUN_GPU:-false}"
TRAIN_ITERS="${EDEN_REAL_CAPABILITY_7B_TRAIN_ITERS:-50}"
SAVE_INTERVAL="${EDEN_REAL_CAPABILITY_7B_SAVE_INTERVAL:-50}"

usage() {
  cat <<'EOF'
Usage: training/rocm/eden_real_capability_stage.sh

Builds the seven-part EDEN real-capability stage:
1. repo-owned capability corpus;
2. governed 7B training evidence;
3. integrated inference evidence;
4. operational capability eval;
5. checkpoint admission decision;
6. operational demo;
7. scaling ladder.

By default it consumes existing 7B evidence. Set EDEN_REAL_CAPABILITY_RUN_GPU=true
to launch the bounded 7B ROCm job and inference probe first.
EOF
}

fail() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

case "$RUN_GPU" in
  true | false) ;;
  *) fail "EDEN_REAL_CAPABILITY_RUN_GPU must be true or false" ;;
esac

cd "$REPO_ROOT"
python3 training/data/build_eden_real_capability_corpus.py

if [[ "$RUN_GPU" == "true" ]]; then
  EDEN_MEGATRON_7B_TRAIN_ITERS="$TRAIN_ITERS" \
  EDEN_MEGATRON_7B_SAVE_CHECKPOINT=true \
  EDEN_MEGATRON_7B_SAVE_INTERVAL="$SAVE_INTERVAL" \
    bash training/rocm/megatron_eden_7b_base_pilot.sh
  bash training/rocm/megatron_eden_7b_inference_probe.sh
fi

python3 training/benchmarks/eden_real_capability_eval.py
printf 'eden_real_capability_stage_ready=true\n'
