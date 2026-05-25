#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

IMAGE="${EDEN_MEGATRON_IMAGE:-rocm/megatron-lm:v25.3}"
OUTPUT_DIR="${EDEN_MEGATRON_OUTPUT_DIR:-${REPO_ROOT}/target/eden_megatron_offline_smoke}"
MASTER_PORT="${EDEN_MEGATRON_MASTER_PORT:-6004}"
TRAIN_ITERS="${EDEN_MEGATRON_TRAIN_ITERS:-1}"
SEQ_LENGTH="${EDEN_MEGATRON_SEQ_LENGTH:-128}"
VOCAB_SIZE="${EDEN_MEGATRON_VOCAB_SIZE:-32000}"
LOG_FILE="${OUTPUT_DIR}/offline_megatron_smoke.log"
SUMMARY_FILE="${OUTPUT_DIR}/offline_megatron_smoke.summary"

usage() {
  cat <<'EOF'
Usage: training/rocm/megatron_offline_smoke.sh

Runs a tiny Megatron-LM smoke train on AMD ROCm without external model
dependencies. The container is started with --network none, uses NullTokenizer,
mock data, random initialization and no Hugging Face/OpenAI/Anthropic provider.

Environment:
  EDEN_MEGATRON_IMAGE        Docker image to use. Default: rocm/megatron-lm:v25.3
  EDEN_MEGATRON_OUTPUT_DIR   Output directory. Default: target/eden_megatron_offline_smoke
  EDEN_MEGATRON_MASTER_PORT  Local torchrun port. Default: 6004
  EDEN_MEGATRON_TRAIN_ITERS  Smoke train iterations. Default: 1
  EDEN_MEGATRON_SEQ_LENGTH   Sequence length. Default: 128
  EDEN_MEGATRON_VOCAB_SIZE   NullTokenizer vocab size. Default: 32000
EOF
}

fail() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

require_command() {
  local -r command_name="$1"
  command -v "$command_name" >/dev/null 2>&1 || fail "missing required command: ${command_name}"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

require_command docker
docker image inspect "$IMAGE" >/dev/null 2>&1 || {
  fail "Docker image not found locally: ${IMAGE}. Pull it explicitly before running this offline smoke."
}

mkdir -p "$OUTPUT_DIR"
rm -f -- "$LOG_FILE" "$SUMMARY_FILE"

printf 'eden_megatron_offline_smoke_start=true\n'
printf 'image=%s\n' "$IMAGE"
printf 'output_dir=%s\n' "$OUTPUT_DIR"
printf 'network=none\n'
printf 'tokenizer=NullTokenizer\n'
printf 'mock_data=true\n'
printf 'external_model_dependency=false\n'

docker run --rm \
  --device /dev/dri \
  --device /dev/kfd \
  --network none \
  --ipc host \
  --group-add video \
  --cap-add SYS_PTRACE \
  --security-opt seccomp=unconfined \
  --privileged \
  --shm-size 64G \
  -v "${OUTPUT_DIR}:/eden-output" \
  "$IMAGE" \
  bash -lc "set -Eeuo pipefail
cd /workspace/Megatron-LM
export GPU_MAX_HW_QUEUES=2
export CUDA_DEVICE_MAX_CONNECTIONS=1
export HSA_NO_SCRATCH_RECLAIM=1
export TOKENIZERS_PARALLELISM=false
export GLOO_SOCKET_IFNAME=lo
export NCCL_SOCKET_IFNAME=lo
torchrun --nproc_per_node 1 --nnodes 1 --node_rank 0 --master_addr 127.0.0.1 --master_port '${MASTER_PORT}' pretrain_gpt.py \
  --tensor-model-parallel-size 1 \
  --pipeline-model-parallel-size 1 \
  --num-layers 2 \
  --hidden-size 128 \
  --ffn-hidden-size 512 \
  --num-attention-heads 4 \
  --seq-length '${SEQ_LENGTH}' \
  --max-position-embeddings '${SEQ_LENGTH}' \
  --micro-batch-size 1 \
  --global-batch-size 1 \
  --train-iters '${TRAIN_ITERS}' \
  --lr 1e-4 \
  --min-lr 1e-5 \
  --lr-decay-iters 10 \
  --lr-decay-style cosine \
  --weight-decay 0.01 \
  --clip-grad 1.0 \
  --optimizer adam \
  --tokenizer-type NullTokenizer \
  --vocab-size '${VOCAB_SIZE}' \
  --mock-data \
  --data-cache-path /root/cache \
  --dataloader-type cyclic \
  --log-interval 1 \
  --eval-interval 1000 \
  --eval-iters 0 \
  --save-interval 1000 \
  --bf16 \
  --no-masked-softmax-fusion \
  --disable-bias-linear \
  --attention-dropout 0.0 \
  --hidden-dropout 0.0 \
  --normalization RMSNorm \
  --position-embedding-type rope \
  --swiglu \
  --untie-embeddings-and-output-weights \
  --no-save-optim \
  --distributed-backend nccl \
  --no-gradient-accumulation-fusion \
  --no-async-tensor-model-parallel-allreduce \
  | tee /eden-output/offline_megatron_smoke.log"

{
  printf 'eden_megatron_offline_smoke_passed=true\n'
  printf 'image=%s\n' "$IMAGE"
  printf 'network=none\n'
  printf 'tokenizer=NullTokenizer\n'
  printf 'mock_data=true\n'
  printf 'external_model_dependency=false\n'
  grep -E 'tokenizer_type|tokenizer_model|mock_data|train_iters|number of parameters|iteration[[:space:]]+1/|after training is done' "$LOG_FILE" || true
} >"$SUMMARY_FILE"

cat "$SUMMARY_FILE"
