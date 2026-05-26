#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

RUN_GPU="${EDEN_V01_RUN_GPU:-false}"
TRAIN_ITERS="${EDEN_V01_7B_TRAIN_ITERS:-100}"
SAVE_INTERVAL="${EDEN_V01_7B_SAVE_INTERVAL:-100}"
MIN_ITERS="${EDEN_V01_MIN_ITERS:-100}"

usage() {
  cat <<'EOF'
Usage: training/rocm/eden_v01_capability_stage.sh

Builds the EDEN v0.1 capability stage:
1. larger curated semantic corpus;
2. semantic capability evaluation;
3. optional 7B training beyond the 50-step pilot;
4. checkpoint inference probe;
5. operational demo trace;
6. GPU workspace hygiene report.

By default it consumes existing evidence. Set EDEN_V01_RUN_GPU=true to run the
bounded 7B continuation and inference probe first.
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
  *) fail "EDEN_V01_RUN_GPU must be true or false" ;;
esac

cd "$REPO_ROOT"
python3 training/data/build_eden_v01_semantic_corpus.py

if [[ "$RUN_GPU" == "true" ]]; then
  python3 training/data/build_eden_cognitive_sft_elcp.py
  bash training/rocm/eden_sft_elcp_gpu_pilot.sh
  EDEN_MEGATRON_7B_TRAIN_ITERS="$TRAIN_ITERS" \
  EDEN_MEGATRON_7B_SAVE_CHECKPOINT=true \
  EDEN_MEGATRON_7B_SAVE_INTERVAL="$SAVE_INTERVAL" \
    bash training/rocm/megatron_eden_7b_base_pilot.sh
  bash training/rocm/megatron_eden_7b_inference_probe.sh
fi

python3 training/benchmarks/eden_v01_semantic_eval.py --min-iters "$MIN_ITERS"
python3 training/demos/eden_v01_operational_demo.py
bash training/rocm/eden_gpu_workspace_hygiene.sh
printf 'eden_v01_capability_stage_ready=true\n'
