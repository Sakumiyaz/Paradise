#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"
PORT="${GARM_HRM_REGRESSION_PORT:-8112}"
STATE_DIR="${GARM_HRM_REGRESSION_STATE_DIR:-/tmp/eden_garm_hrm_retrieval_regression}"
PID_FILE="${GARM_HRM_REGRESSION_PID_FILE:-/tmp/eden_garm_hrm_retrieval_regression.pid}"
LOG_FILE="${GARM_HRM_REGRESSION_LOG_FILE:-/tmp/eden_garm_hrm_retrieval_regression.log}"
STDOUT_FILE="${GARM_HRM_REGRESSION_STDOUT_FILE:-/tmp/eden_garm_hrm_retrieval_regression.stdout}"
BASE_URL="http://127.0.0.1:${PORT}"
GARM_PID=""

cleanup() {
    if [[ -n "${GARM_PID}" ]] && kill -0 "${GARM_PID}" 2>/dev/null; then
        curl -fsS --max-time 3 "${BASE_URL}/command?cmd=quit" >/dev/null 2>&1 || true
        wait "${GARM_PID}" 2>/dev/null || true
    fi
}
trap cleanup EXIT

require_contains() {
    local -r haystack="$1"
    local -r needle="$2"
    if [[ "${haystack}" != *"${needle}"* ]]; then
        printf 'Expected output to contain: %s\nActual output:\n%s\n' "${needle}" "${haystack}" >&2
        exit 1
    fi
}

require_file_contains() {
    local -r file="$1"
    local -r needle="$2"
    if [[ ! -f "${file}" ]]; then
        printf 'Expected file to exist: %s\n' "${file}" >&2
        exit 1
    fi
    if ! grep -F -- "${needle}" "${file}" >/dev/null; then
        printf 'Expected file %s to contain: %s\n' "${file}" "${needle}" >&2
        exit 1
    fi
}

curl_text() {
    curl -fsS --max-time 10 "$1"
}

command_sync() {
    local -r command="$1"
    curl_text "${BASE_URL}/command_sync?cmd=${command}"
}

rm -rf -- "${STATE_DIR}"
mkdir -p -- "${STATE_DIR}"

cargo build --bin eden-garm -p eden_core >/dev/null

EDEN_GARM_SKIP_LEGACY_MIGRATION="${EDEN_GARM_SKIP_LEGACY_MIGRATION:-1}" \
EDEN_GARM_TTS_BACKEND="${GARM_HRM_REGRESSION_TTS_BACKEND:-local_stub}" \
    "${ROOT_DIR}/target/debug/eden-garm" \
    --daemon \
    --pid-file "${PID_FILE}" \
    --log-file "${LOG_FILE}" \
    --api-port "${PORT}" \
    --state-dir "${STATE_DIR}" \
    >"${STDOUT_FILE}" 2>&1 &
GARM_PID=$!

for _ in $(seq 1 40); do
    if curl -fsS --max-time 1 "${BASE_URL}/ready" >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

require_contains "$(curl_text "${BASE_URL}/ready")" "ready"

CORPUS_FILE="${STATE_DIR}/hrm_retrieval_regression.en.txt"
printf '%s\n' \
    'alpha retrieval evidence anchors deterministic HRM planning' \
    'beta unrelated control segment should not be the primary hit' \
    'gamma provenance ledger records retrieval evidence for audit' \
    >"${CORPUS_FILE}"

corpus="$(command_sync "hrm+text+corpus+${CORPUS_FILE}")"
require_contains "${corpus}" "[HRM-TEXT-CORPUS]"
require_contains "${corpus}" "exists=true"

ingest="$(command_sync "hrm+text+ingest+${STATE_DIR}")"
require_contains "${ingest}" "[HRM-TEXT-INGEST]"
require_contains "${ingest}" "segments="

search="$(command_sync 'hrm+text+search+alpha+retrieval+evidence')"
require_contains "${search}" "[HRM-TEXT-SEARCH]"
require_contains "${search}" "status=hit"
require_contains "${search}" "confidence="
require_contains "${search}" "citation=doc:"
require_contains "${search}" "alpha retrieval evidence anchors deterministic HRM planning"

context_pack="$(command_sync 'rag+answer+alpha+retrieval+evidence')"
require_contains "${context_pack}" "[HRM-TEXT-CONTEXT-PACK]"
require_contains "${context_pack}" "status=sufficient"
require_contains "${context_pack}" "generation_restricted=true"

abstain="$(command_sync 'rag+answer+zzzz+missing+hallucination+bait')"
require_contains "${abstain}" "[HRM-TEXT-ABSTAIN]"

rag_eval="$(command_sync 'rag+eval')"
require_contains "${rag_eval}" "[HRM-TEXT-EVAL]"
require_contains "${rag_eval}" "hallucination_guard=active"

hrm_run="$(command_sync 'hrm+run+alpha+retrieval+evidence')"
require_contains "${hrm_run}" "[HRM-RUN]"
require_contains "${hrm_run}" "[HRM-RUN-RETRIEVAL]"
require_contains "${hrm_run}" "hits="
require_contains "${hrm_run}" "[HRM-TEXT-RETRIEVAL] status=hit"
require_contains "${hrm_run}" "alpha retrieval evidence anchors deterministic HRM planning"

eval_run="$(command_sync 'eval+run')"
require_contains "${eval_run}" "[EVAL-RUN]"

report="$(command_sync 'garm+report')"
require_contains "${report}" "[GARM-REPORT]"
export_result="$(command_sync 'garm+export')"
require_contains "${export_result}" "[GARM-EXPORT]"
artifacts="$(command_sync 'garm+artifacts')"
require_contains "${artifacts}" "[GARM-ARTIFACTS]"
save="$(command_sync 'save')"
require_contains "${save}" "Runtime state also saved"

require_file_contains "${STATE_DIR}/hrm_text_segments.jsonl" "alpha retrieval evidence anchors deterministic HRM planning"
require_file_contains "${STATE_DIR}/hrm_text_context_pack.json" '"generation_restricted":true'
require_file_contains "${STATE_DIR}/learning_ledger.json" "hrm_text_retrieval"
require_file_contains "${STATE_DIR}/provenance_ledger.json" "hrm_text_segment"
require_file_contains "${STATE_DIR}/garm_report.txt" "[GARM-REPORT]"
require_file_contains "${STATE_DIR}/garm_export.json" '"schema": "garm-export-v1"'

curl_text "${BASE_URL}/command?cmd=quit" >/dev/null
wait "${GARM_PID}"
GARM_PID=""

printf 'GARM HRM retrieval regression passed on port %s with state dir %s\n' "${PORT}" "${STATE_DIR}"
