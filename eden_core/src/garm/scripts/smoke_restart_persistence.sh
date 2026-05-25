#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"
PORT="${GARM_RESTART_SMOKE_PORT:-8111}"
STATE_DIR="${GARM_RESTART_SMOKE_STATE_DIR:-/tmp/eden_garm_restart_smoke}"
PID_FILE="${GARM_RESTART_SMOKE_PID_FILE:-/tmp/eden_garm_restart_smoke.pid}"
LOG_FILE="${GARM_RESTART_SMOKE_LOG_FILE:-/tmp/eden_garm_restart_smoke.log}"
STDOUT_FILE="${GARM_RESTART_SMOKE_STDOUT_FILE:-/tmp/eden_garm_restart_smoke.stdout}"
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

start_garm() {
    : >"${STDOUT_FILE}"
    EDEN_GARM_SKIP_LEGACY_MIGRATION="${EDEN_GARM_SKIP_LEGACY_MIGRATION:-1}" \
        EDEN_GARM_TTS_BACKEND="${GARM_RESTART_SMOKE_TTS_BACKEND:-local_stub}" \
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
            return 0
        fi
        sleep 1
    done
    printf 'GARM did not become ready. Output:\n' >&2
    cat "${STDOUT_FILE}" >&2
    exit 1
}

stop_garm() {
    if [[ -n "${GARM_PID}" ]] && kill -0 "${GARM_PID}" 2>/dev/null; then
        curl_text "${BASE_URL}/command?cmd=quit" >/dev/null
        wait "${GARM_PID}"
        GARM_PID=""
    fi
}

rm -rf -- "${STATE_DIR}"
mkdir -p -- "${STATE_DIR}"

cargo build --bin eden-garm -p eden_core >/dev/null

start_garm
require_contains "$(curl_text "${BASE_URL}/ready")" "ready"

hrm_run="$(command_sync 'hrm+run+restart+persistence')"
require_contains "${hrm_run}" "[HRM-RUN]"
require_contains "${hrm_run}" "[HRM-RUN-METRICS]"
require_contains "${hrm_run}" "executions=1"

tts="$(command_sync 'tts+persistencia+restart')"
require_contains "${tts}" "[VOZ-TTS]"
require_contains "${tts}" "backend=local_stub"
test -f "${STATE_DIR}/voice_backend_request.txt"
bash "${SCRIPT_DIR}/local_tts_backend.sh" --state-dir "${STATE_DIR}" >/dev/null
require_file_contains "${STATE_DIR}/voice_backend_output.txt" "status=rendered_text_manifest"

audit_before="$(command_sync 'garm+audit')"
require_contains "${audit_before}" "[GARM-AUDIT]"
require_contains "${audit_before}" "executions:1"
require_contains "${audit_before}" "voice:requests:"
report_before="$(command_sync 'garm+report')"
require_contains "${report_before}" "[GARM-REPORT]"
require_contains "${report_before}" "[LastDeltas]"
second_report_before="$(command_sync 'garm+report')"
require_contains "${second_report_before}" "[GARM-REPORT]"
report_history_before="$(command_sync 'garm+report+history')"
require_contains "${report_history_before}" "[GARM-REPORT-HISTORY]"
require_contains "${report_history_before}" "entries=2"
require_file_contains "${STATE_DIR}/garm_report.txt" "[GARM-REPORT]"
require_file_contains "${STATE_DIR}/garm_report_history.jsonl" '"verdict":"ready"'
report_endpoint="$(curl_text "${BASE_URL}/report")"
require_contains "${report_endpoint}" "[GARM-REPORT]"
report_history_endpoint="$(curl_text "${BASE_URL}/report/history")"
require_contains "${report_history_endpoint}" '"tick":'
export_before="$(command_sync 'garm+export')"
require_contains "${export_before}" "[GARM-EXPORT]"
require_contains "${export_before}" "checksum_fnv64="
require_file_contains "${STATE_DIR}/garm_export.json" '"schema": "garm-export-v1"'
export_endpoint="$(curl_text "${BASE_URL}/export")"
require_contains "${export_endpoint}" '"mode": "diagnostic-read-only"'
verify_export_before="$(command_sync 'garm+verify+export')"
require_contains "${verify_export_before}" "[GARM-VERIFY-EXPORT] ok=true"
verify_export_endpoint="$(curl_text "${BASE_URL}/export/verify")"
require_contains "${verify_export_endpoint}" "ok=true"
import_before="$(command_sync 'garm+import')"
require_contains "${import_before}" "[GARM-IMPORT] valid=true"
require_contains "${import_before}" "restored=false"
artifacts_before="$(command_sync 'garm+artifacts')"
require_contains "${artifacts_before}" "[GARM-ARTIFACTS]"
require_contains "${artifacts_before}" "name=garm_export"
artifacts_endpoint="$(curl_text "${BASE_URL}/artifacts")"
require_contains "${artifacts_endpoint}" "[GARM-ARTIFACTS-SUMMARY]"

backup_before="$(command_sync 'garm+backup')"
require_contains "${backup_before}" "[GARM-BACKUP]"
compact="$(command_sync 'garm+compact')"
require_contains "${compact}" "[GARM-COMPACT]"
save="$(command_sync 'save')"
require_contains "${save}" "Runtime state also saved"
backup_after="$(command_sync 'garm+backup')"
require_contains "${backup_after}" "[GARM-BACKUP]"

require_file_contains "${STATE_DIR}/hrm_reasoner.json" '"plan_executions":1'
require_file_contains "${STATE_DIR}/voice_synthesizer.json" '"requests":'
require_file_contains "${STATE_DIR}/backup/hrm_reasoner.json" '"plan_executions":1'
require_file_contains "${STATE_DIR}/backup/voice_synthesizer.json" '"requests":'

stop_garm

start_garm
load="$(command_sync 'load')"
require_contains "${load}" "Runtime state also loaded"

audit_after="$(command_sync 'garm+audit')"
require_contains "${audit_after}" "[GARM-AUDIT]"
require_contains "${audit_after}" "executions:1"
require_contains "${audit_after}" "voice:requests:"
report_after="$(command_sync 'garm+report')"
require_contains "${report_after}" "[GARM-REPORT]"
require_contains "${report_after}" "executions:1"
report_history_after="$(command_sync 'garm+report+history')"
require_contains "${report_history_after}" "[GARM-REPORT-HISTORY]"
require_contains "${report_history_after}" "entries=3"
export_after="$(command_sync 'garm+export')"
require_contains "${export_after}" "history_count=3"
verify_export_after="$(command_sync 'garm+verify+export')"
require_contains "${verify_export_after}" "ok=true"
import_after="$(command_sync 'garm+import')"
require_contains "${import_after}" "valid=true"
artifacts_after="$(command_sync 'garm+artifacts')"
require_contains "${artifacts_after}" "name=backup kind=dir exists=true"

organs_run="$(command_sync 'organos+run')"
require_contains "${organs_run}" "[ORGANOS-RUN]"
require_contains "${organs_run}" "profiled_autonomous=32"
require_contains "${organs_run}" "delta="
if [[ "${organs_run}" == *"legacy:no_delta"* ]]; then
    printf 'Unexpected legacy:no_delta after restart:\n%s\n' "${organs_run}" >&2
    exit 1
fi

restore="$(command_sync 'garm+restore')"
require_contains "${restore}" "[GARM-RESTORE]"
require_contains "${restore}" "run_load_next=true"
load_after_restore="$(command_sync 'load')"
require_contains "${load_after_restore}" "Runtime state also loaded"
audit_after_restore="$(command_sync 'garm+audit')"
require_contains "${audit_after_restore}" "executions:1"
require_contains "${audit_after_restore}" "voice:requests:"

test -f "${STATE_DIR}/voice_backend_request.txt"
test -f "${STATE_DIR}/voice_backend_output.txt"
test -f "${STATE_DIR}/backup/hrm_reasoner.json"
test -f "${STATE_DIR}/backup/voice_synthesizer.json"

stop_garm

printf 'GARM restart persistence smoke passed on port %s with state dir %s\n' "${PORT}" "${STATE_DIR}"
