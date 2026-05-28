#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"

RUN_GPU="${EDEN_V02_RUN_GPU:-false}"
BASELINE_ITERS="${EDEN_V02_BASELINE_ITERS:-100}"
CANDIDATE_ITERS="${EDEN_V02_CANDIDATE_ITERS:-250}"
BASELINE_DIR="${EDEN_V02_BASELINE_OUTPUT_DIR:-${REPO_ROOT}/target/eden_megatron_7b_stability_100}"
CANDIDATE_DIR="${EDEN_V02_CANDIDATE_OUTPUT_DIR:-${REPO_ROOT}/target/eden_megatron_7b_stability_250}"
V02_DIR="${EDEN_V02_OUTPUT_DIR:-${REPO_ROOT}/target/eden_v02}"
PURGE_CHECKPOINTS="${EDEN_V02_PURGE_LOCAL_CHECKPOINTS:-false}"

usage() {
  cat <<'EOF'
Usage: training/rocm/eden_v02_stability_stage.sh

Builds the EDEN v0.2 stability stage:
1. larger v0.2 stability/adversarial corpus;
2. optional 100-iteration baseline checkpoint and inference probe;
3. optional 250-iteration candidate checkpoint and inference probe;
4. checkpoint comparison and semantic stability eval;
5. adversarial architecture eval;
6. rollback drill;
7. model card and checkpoint storage policy;
8. non-mutating operational demo trace;
9. GPU workspace hygiene report.

Set EDEN_V02_RUN_GPU=true on a ROCm/Megatron host to run the bounded GPU jobs.
Generated reports are evidence only. Weights are not committed to git.
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
  *) fail "EDEN_V02_RUN_GPU must be true or false" ;;
esac
case "$PURGE_CHECKPOINTS" in
  true | false) ;;
  *) fail "EDEN_V02_PURGE_LOCAL_CHECKPOINTS must be true or false" ;;
esac

cd "$REPO_ROOT"
mkdir -p "$V02_DIR"
python3 training/data/build_eden_v02_stability_corpus.py

if [[ "$RUN_GPU" == "true" ]]; then
  EDEN_MEGATRON_7B_OUTPUT_DIR="$BASELINE_DIR" \
  EDEN_MEGATRON_7B_MASTER_PORT="${EDEN_V02_BASELINE_MASTER_PORT:-6306}" \
  EDEN_MEGATRON_7B_TRAIN_ITERS="$BASELINE_ITERS" \
  EDEN_MEGATRON_7B_SAVE_CHECKPOINT=true \
  EDEN_MEGATRON_7B_SAVE_INTERVAL="$BASELINE_ITERS" \
    bash training/rocm/megatron_eden_7b_base_pilot.sh

  EDEN_MEGATRON_7B_OUTPUT_DIR="$BASELINE_DIR" \
  EDEN_MEGATRON_7B_INFER_MASTER_PORT="${EDEN_V02_BASELINE_INFER_MASTER_PORT:-6316}" \
    bash training/rocm/megatron_eden_7b_inference_probe.sh

  EDEN_MEGATRON_7B_OUTPUT_DIR="$CANDIDATE_DIR" \
  EDEN_MEGATRON_7B_MASTER_PORT="${EDEN_V02_CANDIDATE_MASTER_PORT:-6406}" \
  EDEN_MEGATRON_7B_TRAIN_ITERS="$CANDIDATE_ITERS" \
  EDEN_MEGATRON_7B_SAVE_CHECKPOINT=true \
  EDEN_MEGATRON_7B_SAVE_INTERVAL="$CANDIDATE_ITERS" \
    bash training/rocm/megatron_eden_7b_base_pilot.sh

  EDEN_MEGATRON_7B_OUTPUT_DIR="$CANDIDATE_DIR" \
  EDEN_MEGATRON_7B_INFER_MASTER_PORT="${EDEN_V02_CANDIDATE_INFER_MASTER_PORT:-6416}" \
    bash training/rocm/megatron_eden_7b_inference_probe.sh

  copy_report "$BASELINE_DIR/eden_7b_training_evidence.json" "$V02_DIR/eden_7b_baseline_100_training_evidence.json"
  copy_report "$BASELINE_DIR/eden_7b_inference_report.json" "$V02_DIR/eden_7b_baseline_100_inference_report.json"
  copy_report "$CANDIDATE_DIR/eden_7b_training_evidence.json" "$V02_DIR/eden_7b_stability_250_training_evidence.json"
  copy_report "$CANDIDATE_DIR/eden_7b_inference_report.json" "$V02_DIR/eden_7b_stability_250_inference_report.json"
fi

python3 training/benchmarks/eden_v02_stability_eval.py
python3 training/benchmarks/eden_v02_adversarial_eval.py
python3 training/benchmarks/eden_v02_rollback_drill.py
python3 training/benchmarks/eden_v02_model_card.py
python3 training/demos/eden_v02_stability_demo.py
bash training/rocm/eden_gpu_workspace_hygiene.sh

if [[ "$PURGE_CHECKPOINTS" == "true" ]]; then
  rm -rf -- "$BASELINE_DIR/checkpoints" "$CANDIDATE_DIR/checkpoints"
fi

printf 'eden_v02_stability_stage_ready=true\n'
