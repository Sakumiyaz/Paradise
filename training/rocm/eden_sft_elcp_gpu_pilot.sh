#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

IMAGE="${EDEN_MEGATRON_IMAGE:-rocm/megatron-lm:v25.3}"
OUTPUT_DIR="${EDEN_SFT_ELCP_OUTPUT_DIR:-${REPO_ROOT}/target/eden_sft_elcp_gpu_pilot}"
EPOCHS="${EDEN_SFT_ELCP_EPOCHS:-120}"
HIDDEN="${EDEN_SFT_ELCP_HIDDEN:-96}"
LR="${EDEN_SFT_ELCP_LR:-0.05}"
TRAIN_DATA="${EDEN_SFT_ELCP_TRAIN_DATA:-${REPO_ROOT}/training/data/eden_cognitive_sft_elcp_train.jsonl}"
EVAL_DATA="${EDEN_SFT_ELCP_EVAL_DATA:-${REPO_ROOT}/training/data/eden_cognitive_sft_elcp_eval.jsonl}"

usage() {
  cat <<'EOF'
Usage: training/rocm/eden_sft_elcp_gpu_pilot.sh

Runs EDEN's first real SFT/ELCP GPU pilot on AMD ROCm. This trains a compact
EDEN-owned cognitive-transition module over repo-local synthetic data. It does
not use external models, does not admit a production checkpoint and does not
claim AGI.

Environment:
  EDEN_MEGATRON_IMAGE       Docker image. Default: rocm/megatron-lm:v25.3
  EDEN_SFT_ELCP_OUTPUT_DIR  Output dir. Default: target/eden_sft_elcp_gpu_pilot
  EDEN_SFT_ELCP_EPOCHS      Training epochs. Default: 120
  EDEN_SFT_ELCP_HIDDEN      Hidden size. Default: 96
  EDEN_SFT_ELCP_LR          Learning rate. Default: 0.05
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

command -v docker >/dev/null 2>&1 || fail "missing required command: docker"
docker image inspect "$IMAGE" >/dev/null 2>&1 || {
  fail "Docker image not found locally: ${IMAGE}. Pull it explicitly before running this offline pilot."
}
[[ -f "$TRAIN_DATA" ]] || fail "missing train dataset: ${TRAIN_DATA}"
[[ -f "$EVAL_DATA" ]] || fail "missing eval dataset: ${EVAL_DATA}"

mkdir -p "$OUTPUT_DIR"
OUTPUT_DIR="$(cd -- "$OUTPUT_DIR" && pwd -P)"
rm -f -- \
  "${OUTPUT_DIR}/eden_sft_elcp_training_report.json" \
  "${OUTPUT_DIR}/eden_sft_elcp_prepost_eval.json" \
  "${OUTPUT_DIR}/eden_sft_elcp_inference_packets.json" \
  "${OUTPUT_DIR}/eden_sft_elcp_checkpoint_admission_review.json" \
  "${OUTPUT_DIR}/eden_sft_elcp_gpu_pilot.pt"

printf 'eden_sft_elcp_gpu_pilot_start=true\n'
printf 'image=%s\n' "$IMAGE"
printf 'output_dir=%s\n' "$OUTPUT_DIR"
printf 'network=none\n'
printf 'external_model_dependency=false\n'
printf 'train_data=%s\n' "$TRAIN_DATA"
printf 'eval_data=%s\n' "$EVAL_DATA"

docker run --rm \
  --device /dev/dri \
  --device /dev/kfd \
  --network none \
  --ipc host \
  --group-add video \
  --cap-add SYS_PTRACE \
  --security-opt seccomp=unconfined \
  --privileged \
  --shm-size 16G \
  -v "${REPO_ROOT}:/workspace/Paradise:ro" \
  -v "${OUTPUT_DIR}:/eden-output" \
  "$IMAGE" \
  bash -lc "set -Eeuo pipefail
export GPU_MAX_HW_QUEUES=2
export CUDA_DEVICE_MAX_CONNECTIONS=1
export HSA_NO_SCRATCH_RECLAIM=1
cd /workspace/Paradise
python3 training/rocm/eden_sft_elcp_gpu_pilot.py \
  --train training/data/eden_cognitive_sft_elcp_train.jsonl \
  --eval training/data/eden_cognitive_sft_elcp_eval.jsonl \
  --output-dir /eden-output \
  --epochs '${EPOCHS}' \
  --hidden-size '${HIDDEN}' \
  --lr '${LR}' \
  --require-gpu"

printf 'training_report=%s\n' "${OUTPUT_DIR}/eden_sft_elcp_training_report.json"
