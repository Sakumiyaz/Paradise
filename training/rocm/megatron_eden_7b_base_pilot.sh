#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

IMAGE="${EDEN_MEGATRON_IMAGE:-rocm/megatron-lm:v25.3}"
OUTPUT_DIR="${EDEN_MEGATRON_7B_OUTPUT_DIR:-${REPO_ROOT}/target/eden_megatron_7b_base_pilot}"
MASTER_PORT="${EDEN_MEGATRON_7B_MASTER_PORT:-6006}"
TRAIN_ITERS="${EDEN_MEGATRON_7B_TRAIN_ITERS:-1}"
SEQ_LENGTH="${EDEN_MEGATRON_7B_SEQ_LENGTH:-128}"
VOCAB_SIZE="${EDEN_MEGATRON_7B_VOCAB_SIZE:-2048}"
SAVE_CHECKPOINT="${EDEN_MEGATRON_7B_SAVE_CHECKPOINT:-false}"
SAVE_INTERVAL="${EDEN_MEGATRON_7B_SAVE_INTERVAL:-100000}"
CACHE_DIR="${EDEN_MEGATRON_CACHE_DIR:-${REPO_ROOT}/target/rocm_megatron_cache}"
AITER_ROPE="${EDEN_MEGATRON_AITER_ROPE:-false}"
LOG_FILE="${OUTPUT_DIR}/eden_7b_base_pilot.log"
SUMMARY_FILE="${OUTPUT_DIR}/eden_7b_base_pilot.summary"
EVIDENCE_FILE="${OUTPUT_DIR}/eden_7b_training_evidence.json"

usage() {
  cat <<'EOF'
Usage: training/rocm/megatron_eden_7b_base_pilot.sh

Builds an EDEN-owned 7B-shape Megatron base-model pilot on AMD ROCm.
The script trains a repo-local SentencePiece tokenizer from eden_core/corpus,
preprocesses EDEN-owned text into Megatron indexed format, initializes a large
GPT-style model from random weights and runs a very short pilot train.

No external model, tokenizer, dataset or provider is used. Docker runs with
--network none. The output is evidence for the 7B training path only; it is not
AGI evidence, checkpoint admission or a production model release.

Environment:
  EDEN_MEGATRON_IMAGE            Docker image. Default: rocm/megatron-lm:v25.3
  EDEN_MEGATRON_7B_OUTPUT_DIR    Output dir. Default: target/eden_megatron_7b_base_pilot
  EDEN_MEGATRON_7B_MASTER_PORT   Local torchrun port. Default: 6006
  EDEN_MEGATRON_7B_TRAIN_ITERS   Pilot train iterations. Default: 1
  EDEN_MEGATRON_7B_SEQ_LENGTH    Sequence length. Default: 128
  EDEN_MEGATRON_7B_VOCAB_SIZE    SentencePiece vocab size. Default: 2048
  EDEN_MEGATRON_7B_SAVE_CHECKPOINT
                                  Write a pilot checkpoint. Default: false
  EDEN_MEGATRON_7B_SAVE_INTERVAL  Checkpoint interval when saving. Default: 100000
  EDEN_MEGATRON_CACHE_DIR         Persistent ROCm/Megatron cache. Default: target/rocm_megatron_cache
  EDEN_MEGATRON_AITER_ROPE        Use ROCm AITER RoPE backend. Default: false for fast pilots
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
case "$SAVE_CHECKPOINT" in
  true | false) ;;
  *) fail "EDEN_MEGATRON_7B_SAVE_CHECKPOINT must be true or false" ;;
esac
case "$AITER_ROPE" in
  true | false) ;;
  *) fail "EDEN_MEGATRON_AITER_ROPE must be true or false" ;;
esac

mkdir -p "$OUTPUT_DIR" "$CACHE_DIR/aiter-jit" "$CACHE_DIR/megatron-data" "$CACHE_DIR/torch"
OUTPUT_DIR="$(cd -- "$OUTPUT_DIR" && pwd -P)"
CACHE_DIR="$(cd -- "$CACHE_DIR" && pwd -P)"
rm -f -- "$LOG_FILE" "$SUMMARY_FILE" "$EVIDENCE_FILE"
LOG_FILE="${OUTPUT_DIR}/eden_7b_base_pilot.log"
SUMMARY_FILE="${OUTPUT_DIR}/eden_7b_base_pilot.summary"
EVIDENCE_FILE="${OUTPUT_DIR}/eden_7b_training_evidence.json"
if [[ "$SAVE_CHECKPOINT" == "true" ]]; then
  rm -rf -- "${OUTPUT_DIR}/checkpoints"
fi

printf 'eden_megatron_7b_base_pilot_start=true\n'
printf 'image=%s\n' "$IMAGE"
printf 'output_dir=%s\n' "$OUTPUT_DIR"
printf 'cache_dir=%s\n' "$CACHE_DIR"
printf 'aiter_rope=%s\n' "$AITER_ROPE"
printf 'network=none\n'
printf 'model_scale=7b_shape\n'
printf 'tokenizer=eden_sentencepiece\n'
printf 'dataset=eden_core/corpus\n'
printf 'external_model_dependency=false\n'
printf 'save_checkpoint=%s\n' "$SAVE_CHECKPOINT"

SAVE_ARGS="--save-interval ${SAVE_INTERVAL}"
if [[ "$SAVE_CHECKPOINT" == "true" ]]; then
  SAVE_ARGS="--save /eden-output/checkpoints --ckpt-format torch ${SAVE_ARGS}"
fi

docker run --rm \
  --device /dev/dri \
  --device /dev/kfd \
  --network none \
  --ipc host \
  --group-add video \
  --cap-add SYS_PTRACE \
  --security-opt seccomp=unconfined \
  --privileged \
  --shm-size 128G \
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
  --log-interval 1000

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
  --num-layers 32 \
  --hidden-size 4096 \
  --ffn-hidden-size 12288 \
  --num-attention-heads 32 \
  --seq-length '${SEQ_LENGTH}' \
  --max-position-embeddings '${SEQ_LENGTH}' \
  --micro-batch-size 1 \
  --global-batch-size 1 \
  --train-iters '${TRAIN_ITERS}' \
  --lr 1e-5 \
  --min-lr 1e-6 \
  --lr-decay-iters 100 \
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
  ${SAVE_ARGS} \
  --bf16 \
  --no-masked-softmax-fusion \
  --disable-bias-linear \
  --attention-dropout 0.0 \
  --hidden-dropout 0.0 \
  --normalization RMSNorm \
  --position-embedding-type rope \
  --swiglu \
  --untie-embeddings-and-output-weights \
  --recompute-granularity full \
  --recompute-method uniform \
  --recompute-num-layers 1 \
  --no-save-optim \
  --distributed-backend nccl \
  --no-gradient-accumulation-fusion \
  --no-async-tensor-model-parallel-allreduce \
  | tee /eden-output/eden_7b_base_pilot.log"

{
  printf 'eden_megatron_7b_base_pilot_passed=true\n'
  printf 'image=%s\n' "$IMAGE"
  printf 'network=none\n'
  printf 'model_scale=7b_shape\n'
  printf 'tokenizer=eden_sentencepiece\n'
  printf 'dataset=eden_core/corpus\n'
  printf 'mock_data=false\n'
  printf 'external_model_dependency=false\n'
  printf 'agi_evidence=false\n'
  printf 'train_iters=%s\n' "$TRAIN_ITERS"
  printf 'save_checkpoint=%s\n' "$SAVE_CHECKPOINT"
  printf 'checkpoint_written=%s\n' "$([[ -d "${OUTPUT_DIR}/checkpoints" ]] && find "${OUTPUT_DIR}/checkpoints" -type f -print -quit | grep -q . && printf true || printf false)"
  printf 'checkpoint_admission=false\n'
  grep -E 'eden_corpus_|tokenizer_type|tokenizer_model|mock_data|data_path|train_iters|number of parameters|Total number of parameters|iteration[[:space:]]+1/|after training is done|memory \\(MB\\)' "$LOG_FILE" || true
} >"$SUMMARY_FILE"

python3 "${SCRIPT_DIR}/build_megatron_7b_evidence.py" \
  --repo-root "$REPO_ROOT" \
  --output-dir "$OUTPUT_DIR" \
  --schema "${REPO_ROOT}/contracts/v1/schemas/eden-megatron-7b-training-evidence-v1.json" \
  --evidence "$EVIDENCE_FILE"

cat "$SUMMARY_FILE"
printf 'evidence_file=%s\n' "$EVIDENCE_FILE"
