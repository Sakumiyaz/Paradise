#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

IMAGE="${EDEN_MEGATRON_IMAGE:-rocm/megatron-lm:v25.3}"
OUTPUT_DIR="${EDEN_MEGATRON_7B_OUTPUT_DIR:-${REPO_ROOT}/target/eden_megatron_7b_base_pilot}"
MASTER_PORT="${EDEN_MEGATRON_7B_INFER_MASTER_PORT:-6016}"
SEQ_LENGTH="${EDEN_MEGATRON_7B_SEQ_LENGTH:-128}"
TOKENS_TO_GENERATE="${EDEN_MEGATRON_7B_TOKENS:-8}"
TEMPERATURE="${EDEN_MEGATRON_7B_TEMPERATURE:-0.8}"
TOP_K="${EDEN_MEGATRON_7B_TOP_K:-10}"
TOP_P="${EDEN_MEGATRON_7B_TOP_P:-0.0}"
PROMPTS_JSON="${EDEN_MEGATRON_7B_PROMPTS_JSON:-[\"EDEN runtime state:\",\"Memory safety plan:\"]}"
LOG_FILE="${OUTPUT_DIR}/eden_7b_inference_probe.log"
SUMMARY_FILE="${OUTPUT_DIR}/eden_7b_inference.summary"
RESPONSE_FILE="${OUTPUT_DIR}/eden_7b_inference_response.json"
REPORT_FILE="${OUTPUT_DIR}/eden_7b_inference_report.json"

usage() {
  cat <<'EOF'
Usage: training/rocm/megatron_eden_7b_inference_probe.sh

Loads the EDEN-owned Megatron 7B pilot checkpoint and runs a local token
generation probe through Megatron Core inference. This proves only the
checkpoint-load and token-generation path. It is not AGI evidence, semantic
competence admission, checkpoint release or production inference.

Environment:
  EDEN_MEGATRON_IMAGE                 Docker image. Default: rocm/megatron-lm:v25.3
  EDEN_MEGATRON_7B_OUTPUT_DIR         Output dir. Default: target/eden_megatron_7b_base_pilot
  EDEN_MEGATRON_7B_INFER_MASTER_PORT  Local torchrun port. Default: 6016
  EDEN_MEGATRON_7B_SEQ_LENGTH         Sequence length. Default: 128
  EDEN_MEGATRON_7B_TOKENS             Tokens to generate. Default: 8
  EDEN_MEGATRON_7B_TEMPERATURE        Sampling temperature. Default: 0.8
  EDEN_MEGATRON_7B_TOP_K              Top-k sampling. Default: 10
  EDEN_MEGATRON_7B_TOP_P              Top-p sampling. Default: 0.0
  EDEN_MEGATRON_7B_PROMPTS_JSON       JSON array of prompts.
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
require_command python3
docker image inspect "$IMAGE" >/dev/null 2>&1 || {
  fail "Docker image not found locally: ${IMAGE}. Pull it explicitly before running this offline probe."
}

[[ -d "${OUTPUT_DIR}/checkpoints" ]] || fail "missing checkpoint directory: ${OUTPUT_DIR}/checkpoints"
[[ -f "${OUTPUT_DIR}/checkpoints/latest_checkpointed_iteration.txt" ]] || fail "missing latest checkpoint marker"
[[ -f "${OUTPUT_DIR}/tokenizer/eden_sp.model" ]] || fail "missing EDEN SentencePiece tokenizer"
[[ -f "${OUTPUT_DIR}/eden_7b_training_evidence.json" ]] || fail "missing training evidence JSON"

python3 - "$PROMPTS_JSON" <<'PY'
import json
import sys

prompts = json.loads(sys.argv[1])
if not isinstance(prompts, list) or not prompts:
    raise SystemExit("EDEN_MEGATRON_7B_PROMPTS_JSON must be a non-empty JSON array")
if not all(isinstance(prompt, str) and prompt.strip() for prompt in prompts):
    raise SystemExit("all prompts must be non-empty strings")
PY

mkdir -p "$OUTPUT_DIR"
rm -f -- "$LOG_FILE" "$SUMMARY_FILE" "$RESPONSE_FILE" "$REPORT_FILE"

printf 'eden_7b_inference_probe_start=true\n'
printf 'image=%s\n' "$IMAGE"
printf 'output_dir=%s\n' "$OUTPUT_DIR"
printf 'network=none\n'
printf 'model_id=eden-megatron-7b-base-pilot\n'
printf 'model_scale=7b_shape\n'
printf 'tokenizer=eden_sentencepiece\n'
printf 'external_model_dependency=false\n'
printf 'tokens_to_generate=%s\n' "$TOKENS_TO_GENERATE"

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
  -v "${REPO_ROOT}:/workspace/Paradise:ro" \
  -v "${OUTPUT_DIR}:/eden-output" \
  -e EDEN_INFERENCE_RESPONSE_JSON=/eden-output/eden_7b_inference_response.json \
  -e EDEN_INFERENCE_PROMPTS_JSON="${PROMPTS_JSON}" \
  "$IMAGE" \
  bash -lc "set -Eeuo pipefail
cd /workspace/Megatron-LM

cat >/tmp/eden_megatron_7b_infer.py <<'PY'
import json
import os
import sys
from argparse import Namespace
from typing import Any

import torch

sys.path.append('/workspace/Megatron-LM')

from megatron.core.inference.common_inference_params import CommonInferenceParams
from megatron.core.inference.engines.abstract_engine import AbstractEngine
from megatron.core.inference.engines.mcore_engine import MCoreEngine
from megatron.core.inference.model_inference_wrappers.gpt.gpt_inference_wrapper import (
    GPTInferenceWrapper,
)
from megatron.core.inference.model_inference_wrappers.inference_wrapper_config import (
    InferenceWrapperConfig,
)
from megatron.core.inference.text_generation_controllers.simple_text_generation_controller import (
    SimpleTextGenerationController,
)
from megatron.core.transformer.module import MegatronModule
from megatron.training import get_args, get_model, get_tokenizer
from megatron.training.checkpointing import load_checkpoint
from megatron.training.initialize import initialize_megatron
from pretrain_gpt import model_provider


def add_text_generate_args(parser):
    group = parser.add_argument_group(title='EDEN text generation probe')
    group.add_argument('--temperature', type=float, default=1.0)
    group.add_argument('--top_k', type=int, default=1)
    group.add_argument('--top_p', type=float, default=0.0)
    group.add_argument('--return-log-probs', action='store_true', default=False)
    group.add_argument('--num-tokens-to-generate', type=int, default=8)
    group.add_argument('--max-batch-size', type=int, default=1)
    return parser


def jsonable(value: Any) -> Any:
    if hasattr(value, 'tolist'):
        return value.tolist()
    if isinstance(value, (list, tuple)):
        return [jsonable(item) for item in value]
    if isinstance(value, (str, int, float, bool)) or value is None:
        return value
    return str(value)


def get_inference_engine(args: Namespace, model: MegatronModule) -> AbstractEngine:
    tokenizer = get_tokenizer()
    inference_wrapper_config = InferenceWrapperConfig(
        hidden_size=args.hidden_size,
        inference_batch_times_seqlen_threshold=args.inference_batch_times_seqlen_threshold,
        fp32_residual_connection=args.fp32_residual_connection,
        params_dtype=args.params_dtype,
        padded_vocab_size=args.padded_vocab_size,
    )
    inference_wrapped_model = GPTInferenceWrapper(model, inference_wrapper_config)
    controller = SimpleTextGenerationController(
        inference_wrapped_model=inference_wrapped_model,
        tokenizer=tokenizer,
    )
    return MCoreEngine(text_generation_controller=controller, max_batch_size=args.max_batch_size)


def main() -> None:
    initialize_megatron(
        extra_args_provider=add_text_generate_args,
        args_defaults={
            'no_load_rng': True,
            'no_load_optim': True,
            'micro_batch_size': 1,
            'exit_on_missing_checkpoint': True,
        },
    )

    model = get_model(model_provider, wrap_with_ddp=False)
    load_checkpoint(model, None, None)
    model = model[0]
    args = get_args()
    prompts = json.loads(os.environ['EDEN_INFERENCE_PROMPTS_JSON'])
    engine = get_inference_engine(args, model)
    params = CommonInferenceParams(
        temperature=args.temperature,
        top_k=args.top_k,
        top_p=args.top_p,
        return_log_probs=args.return_log_probs,
        num_tokens_to_generate=args.num_tokens_to_generate,
    )
    results = engine.generate(prompts=prompts, common_inference_params=params)
    if torch.distributed.get_rank() == 0:
        payload = {
            'model_id': 'eden-megatron-7b-base-pilot',
            'prompts': prompts,
            'responses': [
                {
                    'id': jsonable(result.request_id),
                    'prompt': result.prompt,
                    'generated_text': result.generated_text,
                    'generated_tokens': jsonable(result.generated_tokens),
                }
                for result in results
            ],
        }
        response_path = os.environ['EDEN_INFERENCE_RESPONSE_JSON']
        with open(response_path, 'w', encoding='utf-8') as handle:
            json.dump(payload, handle, indent=2, sort_keys=True)
            handle.write('\n')
        print('EDEN_INFERENCE_RESPONSE_JSON=' + response_path)
        print(json.dumps(payload, ensure_ascii=False, sort_keys=True))


if __name__ == '__main__':
    main()
PY

export GPU_MAX_HW_QUEUES=2
export CUDA_DEVICE_MAX_CONNECTIONS=1
export HSA_NO_SCRATCH_RECLAIM=1
export TOKENIZERS_PARALLELISM=false
export GLOO_SOCKET_IFNAME=lo
export NCCL_SOCKET_IFNAME=lo

torchrun --nproc_per_node 1 --nnodes 1 --node_rank 0 --master_addr 127.0.0.1 --master_port '${MASTER_PORT}' /tmp/eden_megatron_7b_infer.py \
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
  --tokenizer-type GPTSentencePieceTokenizer \
  --tokenizer-model /eden-output/tokenizer/eden_sp.model \
  --load /eden-output/checkpoints \
  --ckpt-format torch \
  --bf16 \
  --no-masked-softmax-fusion \
  --disable-bias-linear \
  --attention-dropout 0.0 \
  --hidden-dropout 0.0 \
  --normalization RMSNorm \
  --position-embedding-type rope \
  --swiglu \
  --untie-embeddings-and-output-weights \
  --distributed-backend nccl \
  --no-gradient-accumulation-fusion \
  --no-async-tensor-model-parallel-allreduce \
  --temperature '${TEMPERATURE}' \
  --top_k '${TOP_K}' \
  --top_p '${TOP_P}' \
  --num-tokens-to-generate '${TOKENS_TO_GENERATE}' \
  --max-batch-size 1 \
  | tee /eden-output/eden_7b_inference_probe.log"

[[ -s "$RESPONSE_FILE" ]] || fail "inference response file was not written: ${RESPONSE_FILE}"

{
  printf 'eden_7b_inference_probe_passed=true\n'
  printf 'image=%s\n' "$IMAGE"
  printf 'network=none\n'
  printf 'model_id=eden-megatron-7b-base-pilot\n'
  printf 'model_scale=7b_shape\n'
  printf 'tokenizer=eden_sentencepiece\n'
  printf 'external_model_dependency=false\n'
  printf 'checkpoint_loaded=true\n'
  printf 'checkpoint_admission=false\n'
  printf 'production_model=false\n'
  printf 'tokens_to_generate=%s\n' "$TOKENS_TO_GENERATE"
} >"$SUMMARY_FILE"

python3 "${SCRIPT_DIR}/build_megatron_7b_inference_report.py" \
  --output-dir "$OUTPUT_DIR" \
  --schema "${REPO_ROOT}/contracts/v1/schemas/eden-megatron-7b-inference-report-v1.json" \
  --report "$REPORT_FILE"

cat "$SUMMARY_FILE"
printf 'response_file=%s\n' "$RESPONSE_FILE"
printf 'report_file=%s\n' "$REPORT_FILE"
