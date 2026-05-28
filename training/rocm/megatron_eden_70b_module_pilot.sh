#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

IMAGE="${EDEN_MEGATRON_IMAGE:-rocm/megatron-lm:v25.3}"
MODULE_ID="${EDEN_70B_MODULE_ID:-eden_memory_router_retrieval_3b}"
OUTPUT_ROOT="${EDEN_70B_MODULE_OUTPUT_ROOT:-${REPO_ROOT}/target/eden_70b_modular_training}"
OUTPUT_DIR="${EDEN_70B_MODULE_OUTPUT_DIR:-${OUTPUT_ROOT}/${MODULE_ID}}"
CACHE_DIR="${EDEN_MEGATRON_CACHE_DIR:-${REPO_ROOT}/target/rocm_megatron_cache}"
MASTER_PORT="${EDEN_70B_MODULE_MASTER_PORT:-6606}"
TRAIN_ITERS="${EDEN_70B_MODULE_TRAIN_ITERS:-1}"
SEQ_LENGTH="${EDEN_70B_MODULE_SEQ_LENGTH:-64}"
VOCAB_SIZE="${EDEN_70B_MODULE_VOCAB_SIZE:-512}"
SAVE_CHECKPOINT="${EDEN_70B_MODULE_SAVE_CHECKPOINT:-false}"
SAVE_INTERVAL="${EDEN_70B_MODULE_SAVE_INTERVAL:-100000}"
AITER_ROPE="${EDEN_MEGATRON_AITER_ROPE:-false}"
ALLOW_OVERSIZE="${EDEN_70B_MODULE_ALLOW_OVERSIZE:-false}"

LOG_FILE="${OUTPUT_DIR}/module_training.log"
SUMMARY_FILE="${OUTPUT_DIR}/module_training.summary"
EVIDENCE_FILE="${OUTPUT_DIR}/module_training_evidence.json"
BLOCK_FILE="${OUTPUT_DIR}/module_training_blocked.json"

usage() {
  cat <<'EOF'
Usage: training/rocm/megatron_eden_70b_module_pilot.sh

Starts a bounded ROCm/Megatron pilot for one module of the EDEN-70B modular
family. EDEN-70B is not one dense model; this launcher trains one GEWC-routed
subordinate module at a time and blocks modules that do not fit a single MI300X.

Environment:
  EDEN_70B_MODULE_ID              Module id. Default: eden_memory_router_retrieval_3b
  EDEN_70B_MODULE_OUTPUT_ROOT     Output root. Default: target/eden_70b_modular_training
  EDEN_70B_MODULE_OUTPUT_DIR      Exact output dir. Default: output root/module id
  EDEN_70B_MODULE_TRAIN_ITERS     Train iterations. Default: 1
  EDEN_70B_MODULE_SEQ_LENGTH      Sequence length. Default: 64
  EDEN_70B_MODULE_VOCAB_SIZE      SentencePiece vocab size. Default: 512
  EDEN_70B_MODULE_SAVE_CHECKPOINT Write checkpoint. Default: false
  EDEN_MEGATRON_CACHE_DIR         Persistent ROCm/Megatron cache.
  EDEN_MEGATRON_AITER_ROPE        Use ROCm AITER RoPE backend. Default: false
  EDEN_70B_MODULE_ALLOW_OVERSIZE  Bypass single-GPU memory block. Default: false
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

case "$SAVE_CHECKPOINT" in
  true | false) ;;
  *) fail "EDEN_70B_MODULE_SAVE_CHECKPOINT must be true or false" ;;
esac
case "$AITER_ROPE" in
  true | false) ;;
  *) fail "EDEN_MEGATRON_AITER_ROPE must be true or false" ;;
esac
case "$ALLOW_OVERSIZE" in
  true | false) ;;
  *) fail "EDEN_70B_MODULE_ALLOW_OVERSIZE must be true or false" ;;
esac
[[ "$TRAIN_ITERS" =~ ^[0-9]+$ ]] || fail "EDEN_70B_MODULE_TRAIN_ITERS must be an integer"
(( TRAIN_ITERS >= 1 )) || fail "EDEN_70B_MODULE_TRAIN_ITERS must be >= 1"
[[ "$SEQ_LENGTH" =~ ^[0-9]+$ ]] || fail "EDEN_70B_MODULE_SEQ_LENGTH must be an integer"
(( SEQ_LENGTH >= 16 )) || fail "EDEN_70B_MODULE_SEQ_LENGTH must be >= 16"
[[ "$VOCAB_SIZE" =~ ^[0-9]+$ ]] || fail "EDEN_70B_MODULE_VOCAB_SIZE must be an integer"
(( VOCAB_SIZE >= 128 )) || fail "EDEN_70B_MODULE_VOCAB_SIZE must be >= 128"

ROLE=""
TARGET_PARAMS=""
NUM_LAYERS=""
HIDDEN_SIZE=""
FFN_HIDDEN_SIZE=""
NUM_HEADS=""
ESTIMATED_VRAM_GB=""
EXECUTABLE_ON_SINGLE_MI300X="true"

case "$MODULE_ID" in
  eden_memory_router_retrieval_3b)
    ROLE="memory_router_retrieval"
    TARGET_PARAMS=3000000000
    NUM_LAYERS=28
    HIDDEN_SIZE=2816
    FFN_HIDDEN_SIZE=8448
    NUM_HEADS=22
    ESTIMATED_VRAM_GB=56
    ;;
  eden_safety_verifier_4b)
    ROLE="safety_verifier_critic"
    TARGET_PARAMS=4000000000
    NUM_LAYERS=32
    HIDDEN_SIZE=3072
    FFN_HIDDEN_SIZE=9216
    NUM_HEADS=24
    ESTIMATED_VRAM_GB=76
    ;;
  eden_planner_code_tool_6b)
    ROLE="planning_code_tools"
    TARGET_PARAMS=6000000000
    NUM_LAYERS=32
    HIDDEN_SIZE=3840
    FFN_HIDDEN_SIZE=11520
    NUM_HEADS=30
    ESTIMATED_VRAM_GB=118
    ;;
  eden_cwm_12b_causal_world_model)
    ROLE="world_model"
    TARGET_PARAMS=12000000000
    NUM_LAYERS=40
    HIDDEN_SIZE=5120
    FFN_HIDDEN_SIZE=15360
    NUM_HEADS=40
    ESTIMATED_VRAM_GB=228
    EXECUTABLE_ON_SINGLE_MI300X="false"
    ;;
  eden_multimodal_vla_12b)
    ROLE="multimodal_grounding"
    TARGET_PARAMS=12000000000
    NUM_LAYERS=40
    HIDDEN_SIZE=5120
    FFN_HIDDEN_SIZE=15360
    NUM_HEADS=40
    ESTIMATED_VRAM_GB=228
    EXECUTABLE_ON_SINGLE_MI300X="false"
    ;;
  eden_33b_elcp_primary)
    ROLE="primary_cognitive_model"
    TARGET_PARAMS=33000000000
    NUM_LAYERS=48
    HIDDEN_SIZE=7168
    FFN_HIDDEN_SIZE=21504
    NUM_HEADS=56
    ESTIMATED_VRAM_GB=620
    EXECUTABLE_ON_SINGLE_MI300X="false"
    ;;
  *)
    fail "unknown EDEN_70B_MODULE_ID: ${MODULE_ID}"
    ;;
esac

mkdir -p "$OUTPUT_DIR" "$CACHE_DIR/aiter-jit" "$CACHE_DIR/megatron-data" "$CACHE_DIR/torch"
OUTPUT_DIR="$(cd -- "$OUTPUT_DIR" && pwd -P)"
CACHE_DIR="$(cd -- "$CACHE_DIR" && pwd -P)"
LOG_FILE="${OUTPUT_DIR}/module_training.log"
SUMMARY_FILE="${OUTPUT_DIR}/module_training.summary"
EVIDENCE_FILE="${OUTPUT_DIR}/module_training_evidence.json"
BLOCK_FILE="${OUTPUT_DIR}/module_training_blocked.json"
rm -f -- "$LOG_FILE" "$SUMMARY_FILE" "$EVIDENCE_FILE" "$BLOCK_FILE"
if [[ "$SAVE_CHECKPOINT" == "true" ]]; then
  rm -rf -- "${OUTPUT_DIR}/checkpoints"
fi

if [[ "$EXECUTABLE_ON_SINGLE_MI300X" != "true" && "$ALLOW_OVERSIZE" != "true" ]]; then
  python3 - "$BLOCK_FILE" "$MODULE_ID" "$ROLE" "$TARGET_PARAMS" "$ESTIMATED_VRAM_GB" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = {
    "schema": "eden.modular_70b.module_training_blocked.v1",
    "authority": "global_executive_workspace_core",
    "claim_allowed": False,
    "agi_claim": False,
    "training_executed": False,
    "module_id": sys.argv[2],
    "role": sys.argv[3],
    "target_parameters": int(sys.argv[4]),
    "estimated_single_gpu_vram_gb": int(sys.argv[5]),
    "status": "blocked_before_gpu_training",
    "reason": "module target exceeds safe single-MI300X pilot budget; requires multi-GPU tensor/pipeline parallel training or a smaller proxy run",
    "required_before_training": [
        "multi_gpu_topology",
        "tensor_parallel_plan",
        "checkpoint_retention_policy",
        "dataset_freeze",
        "operator_approval_for_this_module",
    ],
}
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
  cat "$BLOCK_FILE"
  exit 0
fi

require_command docker
docker image inspect "$IMAGE" >/dev/null 2>&1 || {
  fail "Docker image not found locally: ${IMAGE}. Pull it explicitly before running this offline pilot."
}

printf 'eden_70b_module_pilot_start=true\n'
printf 'image=%s\n' "$IMAGE"
printf 'module_id=%s\n' "$MODULE_ID"
printf 'role=%s\n' "$ROLE"
printf 'target_parameters=%s\n' "$TARGET_PARAMS"
printf 'output_dir=%s\n' "$OUTPUT_DIR"
printf 'cache_dir=%s\n' "$CACHE_DIR"
printf 'network=none\n'
printf 'external_model_dependency=false\n'
printf 'aiter_rope=%s\n' "$AITER_ROPE"
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

python3 - '${MODULE_ID}' <<'PY'
from pathlib import Path
import json
import sys

module_id = sys.argv[1]
repo = Path('/workspace/Paradise')
splits = [
    repo / 'training/data/eden_70b_modular_train.jsonl',
    repo / 'training/data/eden_70b_modular_eval.jsonl',
    repo / 'training/data/eden_70b_modular_challenge.jsonl',
]
rows = []
for path in splits:
    with path.open('r', encoding='utf-8') as handle:
        for line in handle:
            item = json.loads(line)
            if item.get('module_id') == module_id:
                rows.append(item)
if not rows:
    raise SystemExit(f'no module rows found for {module_id}')

jsonl_path = Path('/eden-output/data/module_corpus.jsonl')
txt_path = Path('/eden-output/data/module_corpus.txt')
line_count = 0
char_count = 0
with jsonl_path.open('w', encoding='utf-8') as jsonl_out, txt_path.open('w', encoding='utf-8') as txt_out:
    for repeat in range(128):
        for row in rows:
            input_obj = row.get('input', {})
            target_obj = row.get('target', {})
            text = ' '.join([
                f'module_id={row.get(\"module_id\", \"\")}',
                f'role={row.get(\"role\", \"\")}',
                f'task={input_obj.get(\"task\", \"\")}',
                f'context={input_obj.get(\"context\", \"\")}',
                f'authority={input_obj.get(\"authority\", \"\")}',
                f'risk={input_obj.get(\"risk\", \"\")}',
                f'output_kind={target_obj.get(\"output_kind\", \"\")}',
                f'outputs_are_hypotheses={target_obj.get(\"outputs_are_hypotheses\", False)}',
                f'direct_memory_write={target_obj.get(\"direct_memory_write\", False)}',
                f'direct_objective_update={target_obj.get(\"direct_objective_update\", False)}',
                f'direct_tool_execution={target_obj.get(\"direct_tool_execution\", False)}',
                f'repeat={repeat}',
            ])
            jsonl_out.write(json.dumps({'text': text}, ensure_ascii=False) + '\n')
            txt_out.write(text + '\n')
            line_count += 1
            char_count += len(text)
print(f'module_dataset_rows={len(rows)}')
print(f'module_corpus_lines={line_count}')
print(f'module_corpus_chars={char_count}')
PY

python3 - <<'PY'
import sentencepiece as spm

spm.SentencePieceTrainer.Train(
    input='/eden-output/data/module_corpus.txt',
    model_prefix='/eden-output/tokenizer/module_sp',
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
  --input /eden-output/data/module_corpus.jsonl \
  --json-keys text \
  --tokenizer-type GPTSentencePieceTokenizer \
  --tokenizer-model /eden-output/tokenizer/module_sp.model \
  --output-prefix /eden-output/data/module_corpus \
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
  --num-layers '${NUM_LAYERS}' \
  --hidden-size '${HIDDEN_SIZE}' \
  --ffn-hidden-size '${FFN_HIDDEN_SIZE}' \
  --num-attention-heads '${NUM_HEADS}' \
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
  --tokenizer-model /eden-output/tokenizer/module_sp.model \
  --data-path /eden-output/data/module_corpus_text_document \
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
  | tee /eden-output/module_training.log"

CHECKPOINT_WRITTEN=false
if [[ -d "${OUTPUT_DIR}/checkpoints" ]] && find "${OUTPUT_DIR}/checkpoints" -type f -print -quit | grep -q .; then
  CHECKPOINT_WRITTEN=true
fi

{
  printf 'eden_70b_module_pilot_passed=true\n'
  printf 'image=%s\n' "$IMAGE"
  printf 'module_id=%s\n' "$MODULE_ID"
  printf 'role=%s\n' "$ROLE"
  printf 'target_parameters=%s\n' "$TARGET_PARAMS"
  printf 'network=none\n'
  printf 'mock_data=false\n'
  printf 'external_model_dependency=false\n'
  printf 'agi_evidence=false\n'
  printf 'train_iters=%s\n' "$TRAIN_ITERS"
  printf 'save_checkpoint=%s\n' "$SAVE_CHECKPOINT"
  printf 'checkpoint_written=%s\n' "$CHECKPOINT_WRITTEN"
  printf 'checkpoint_admission=false\n'
  grep -E 'module_dataset_rows|module_corpus_|tokenizer_type|tokenizer_model|mock_data|data_path|train_iters|number of parameters|Total number of parameters|iteration[[:space:]]+1/|after training is done|memory \(MB\)' "$LOG_FILE" || true
} >"$SUMMARY_FILE"

python3 - "$EVIDENCE_FILE" "$SUMMARY_FILE" "$MODULE_ID" "$ROLE" "$TARGET_PARAMS" "$TRAIN_ITERS" "$SEQ_LENGTH" "$CHECKPOINT_WRITTEN" <<'PY'
import json
import re
import sys
from pathlib import Path

evidence_path = Path(sys.argv[1])
summary_path = Path(sys.argv[2])
summary = summary_path.read_text(encoding='utf-8')

def find(pattern: str, cast=str, default=None):
    match = re.search(pattern, summary)
    if not match:
        return default
    return cast(match.group(1))

payload = {
    "schema": "eden.modular_70b.module_training_pilot.v1",
    "artifact": "eden_70b_module_training_pilot",
    "authority": "global_executive_workspace_core",
    "claim_allowed": False,
    "agi_claim": False,
    "training_executed": True,
    "checkpoint_admission_allowed": False,
    "production_release_allowed": False,
    "external_model_dependency": False,
    "network": "none",
    "module": {
        "id": sys.argv[3],
        "role": sys.argv[4],
        "target_parameters": int(sys.argv[5]),
        "observed_total_parameters_b": find(r"Total number of parameters in billions: ([0-9.]+)", float),
    },
    "training": {
        "train_iters": int(sys.argv[6]),
        "seq_length": int(sys.argv[7]),
        "mock_data": False,
        "checkpoint_written": sys.argv[8] == "true",
        "loss": find(r"lm loss: ([0-9.E+-]+)", float),
        "iteration_ms": find(r"elapsed time per iteration \(ms\): ([0-9.]+)", float),
        "allocated_mb": find(r"allocated: ([0-9.]+)", float),
    },
    "governance": {
        "model_outputs_are_hypotheses": True,
        "direct_memory_writes": False,
        "direct_objective_writes": False,
        "direct_tool_execution": False,
        "requires_gewc_review_before_admission": True,
    },
}
evidence_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

cat "$SUMMARY_FILE"
printf 'evidence_file=%s\n' "$EVIDENCE_FILE"
