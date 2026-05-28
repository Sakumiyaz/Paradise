#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

IMAGE="${EDEN_MEGATRON_IMAGE:-rocm/megatron-lm:v25.3}"
OUTPUT_DIR="${EDEN_MEGATRON_CORPUS_OUTPUT_DIR:-${REPO_ROOT}/target/eden_megatron_corpus_pilot}"
MASTER_PORT="${EDEN_MEGATRON_CORPUS_MASTER_PORT:-6005}"
TRAIN_ITERS="${EDEN_MEGATRON_CORPUS_TRAIN_ITERS:-1}"
SEQ_LENGTH="${EDEN_MEGATRON_CORPUS_SEQ_LENGTH:-128}"
VOCAB_SIZE="${EDEN_MEGATRON_CORPUS_VOCAB_SIZE:-2048}"
CACHE_DIR="${EDEN_MEGATRON_CACHE_DIR:-${REPO_ROOT}/target/rocm_megatron_cache}"
AITER_ROPE="${EDEN_MEGATRON_AITER_ROPE:-false}"
LOG_FILE="${OUTPUT_DIR}/eden_corpus_pilot.log"
SUMMARY_FILE="${OUTPUT_DIR}/eden_corpus_pilot.summary"

usage() {
  cat <<'EOF'
Usage: training/rocm/megatron_eden_corpus_pilot.sh

Builds an EDEN-owned Megatron pilot dataset from eden_core/corpus, trains a
repo-local SentencePiece tokenizer, preprocesses the data into Megatron indexed
format and runs a tiny randomly initialized GPT pilot on AMD ROCm.

No external model, tokenizer, dataset or provider is used. Docker runs with
--network none. The output is pilot evidence only; it is not checkpoint
admission and not a production model release.

Environment:
  EDEN_MEGATRON_IMAGE                Docker image. Default: rocm/megatron-lm:v25.3
  EDEN_MEGATRON_CORPUS_OUTPUT_DIR    Output dir. Default: target/eden_megatron_corpus_pilot
  EDEN_MEGATRON_CORPUS_MASTER_PORT   Local torchrun port. Default: 6005
  EDEN_MEGATRON_CORPUS_TRAIN_ITERS   Pilot train iterations. Default: 1
  EDEN_MEGATRON_CORPUS_SEQ_LENGTH    Sequence length. Default: 128
  EDEN_MEGATRON_CORPUS_VOCAB_SIZE    SentencePiece vocab size. Default: 2048
  EDEN_MEGATRON_CACHE_DIR            Persistent ROCm/Megatron cache. Default: target/rocm_megatron_cache
  EDEN_MEGATRON_AITER_ROPE           Use ROCm AITER RoPE backend. Default: false for fast pilots
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
  fail "Docker image not found locally: ${IMAGE}. Pull it explicitly before running this offline pilot."
}
case "$AITER_ROPE" in
  true | false) ;;
  *) fail "EDEN_MEGATRON_AITER_ROPE must be true or false" ;;
esac

mkdir -p "$OUTPUT_DIR" "$CACHE_DIR/aiter-jit" "$CACHE_DIR/megatron-data" "$CACHE_DIR/torch"
OUTPUT_DIR="$(cd -- "$OUTPUT_DIR" && pwd -P)"
CACHE_DIR="$(cd -- "$CACHE_DIR" && pwd -P)"
rm -f -- "$LOG_FILE" "$SUMMARY_FILE"
LOG_FILE="${OUTPUT_DIR}/eden_corpus_pilot.log"
SUMMARY_FILE="${OUTPUT_DIR}/eden_corpus_pilot.summary"

printf 'eden_megatron_corpus_pilot_start=true\n'
printf 'image=%s\n' "$IMAGE"
printf 'output_dir=%s\n' "$OUTPUT_DIR"
printf 'cache_dir=%s\n' "$CACHE_DIR"
printf 'aiter_rope=%s\n' "$AITER_ROPE"
printf 'network=none\n'
printf 'tokenizer=eden_sentencepiece\n'
printf 'dataset=eden_core/corpus\n'
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
  -v "${CACHE_DIR}/aiter-jit:/workspace/aiter/aiter/jit/build" \
  -v "${CACHE_DIR}/megatron-data:/root/cache" \
  -v "${CACHE_DIR}/torch:/root/.cache/torch" \
  -v "${REPO_ROOT}:/workspace/Paradise:ro" \
  -v "${OUTPUT_DIR}:/eden-output" \
  "$IMAGE" \
  bash -lc "set -Eeuo pipefail
cd /workspace/Megatron-LM
mkdir -p /eden-output/data /eden-output/tokenizer

python3 - <<'PY'
from pathlib import Path
import json

corpus_root = Path('/workspace/Paradise/eden_core/corpus')
text_files = sorted(path for path in corpus_root.rglob('*.txt') if path.is_file())
if not text_files:
    raise SystemExit('no EDEN corpus text files found')

combined_path = Path('/eden-output/data/eden_corpus.txt')
jsonl_path = Path('/eden-output/data/eden_corpus.jsonl')
line_count = 0
char_count = 0

with combined_path.open('w', encoding='utf-8') as combined, jsonl_path.open('w', encoding='utf-8') as jsonl:
    for path in text_files:
        text = path.read_text(encoding='utf-8', errors='replace')
        combined.write(text)
        if not text.endswith('\n'):
            combined.write('\n')
        for raw_line in text.splitlines():
            line = ' '.join(raw_line.split())
            if len(line) < 3:
                continue
            jsonl.write(json.dumps({'text': line}, ensure_ascii=False) + '\n')
            line_count += 1
            char_count += len(line)

print(f'eden_corpus_files={len(text_files)}')
print(f'eden_corpus_lines={line_count}')
print(f'eden_corpus_chars={char_count}')
PY

python3 - <<'PY'
import sentencepiece as spm

spm.SentencePieceTrainer.Train(
    input='/eden-output/data/eden_corpus.txt',
    model_prefix='/eden-output/tokenizer/eden_sp',
    vocab_size=${VOCAB_SIZE},
    model_type='bpe',
    character_coverage=1.0,
    bos_id=1,
    eos_id=2,
    unk_id=0,
    pad_id=3,
    hard_vocab_limit=False,
)
PY

python3 tools/preprocess_data.py \
  --input /eden-output/data/eden_corpus.jsonl \
  --json-keys text \
  --tokenizer-type GPTSentencePieceTokenizer \
  --tokenizer-model /eden-output/tokenizer/eden_sp.model \
  --output-prefix /eden-output/data/eden_corpus \
  --workers 2 \
  --append-eod \
  --log-interval 100

export GPU_MAX_HW_QUEUES=2
export TORCH_HOME=/root/.cache/torch
export TORCHINDUCTOR_CACHE_DIR=/root/.cache/torch/inductor
if [[ '${AITER_ROPE}' != 'true' ]]; then
  export USE_ROCM_AITER_ROPE_BACKEND=0
fi
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
  --tokenizer-type GPTSentencePieceTokenizer \
  --tokenizer-model /eden-output/tokenizer/eden_sp.model \
  --data-path /eden-output/data/eden_corpus_text_document \
  --dataloader-type cyclic \
  --split 90,10,0 \
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
  | tee /eden-output/eden_corpus_pilot.log"

{
  printf 'eden_megatron_corpus_pilot_passed=true\n'
  printf 'image=%s\n' "$IMAGE"
  printf 'network=none\n'
  printf 'tokenizer=eden_sentencepiece\n'
  printf 'dataset=eden_core/corpus\n'
  printf 'mock_data=false\n'
  printf 'external_model_dependency=false\n'
  printf 'checkpoint_admission=false\n'
  grep -E 'eden_corpus_|tokenizer_type|tokenizer_model|mock_data|data_path|train_iters|number of parameters|iteration[[:space:]]+1/|after training is done' "$LOG_FILE" || true
} >"$SUMMARY_FILE"

cat "$SUMMARY_FILE"
