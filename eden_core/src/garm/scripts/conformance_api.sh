#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
PORT="${EDEN_CONFORMANCE_PORT:-8113}"
STATE_DIR="${EDEN_CONFORMANCE_STATE_DIR:-/tmp/eden_garm_api_conformance}"
PID_FILE="${EDEN_CONFORMANCE_PID_FILE:-/tmp/eden_garm_api_conformance.pid}"
LOG_FILE="${EDEN_CONFORMANCE_LOG_FILE:-/tmp/eden_garm_api_conformance.log}"
STDOUT_FILE="${EDEN_CONFORMANCE_STDOUT_FILE:-/tmp/eden_garm_api_conformance.stdout}"
REPORT_FILE="${EDEN_CONFORMANCE_REPORT:-${STATE_DIR}/sdk_conformance_report.json}"
BASE_URL="http://127.0.0.1:${PORT}"

cleanup() {
    if [[ -n "${GARM_PID:-}" ]] && kill -0 "${GARM_PID}" 2>/dev/null; then
        curl -fsS --max-time 3 "${BASE_URL}/command?cmd=quit" >/dev/null 2>&1 || true
        wait "${GARM_PID}" 2>/dev/null || true
    fi
}
trap cleanup EXIT

require_contains() {
    local haystack="$1"
    local needle="$2"
    if [[ "${haystack}" != *"${needle}"* ]]; then
        printf 'Expected output to contain: %s\nActual output:\n%s\n' "${needle}" "${haystack}" >&2
        exit 1
    fi
}

curl_text() {
    curl -fsS --max-time 20 "$1"
}

rm -rf "${STATE_DIR}"
mkdir -p "${STATE_DIR}"
mkdir -p "$(dirname "${REPORT_FILE}")"

cargo build --bin eden-garm --bin eden-garm-api-conformance -p eden_core >/dev/null

EDEN_GARM_SKIP_LEGACY_MIGRATION="${EDEN_GARM_SKIP_LEGACY_MIGRATION:-1}" \
EDEN_GARM_TTS_BACKEND="${EDEN_CONFORMANCE_TTS_BACKEND:-local_stub}" \
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

ready="$(curl_text "${BASE_URL}/ready")"
require_contains "${ready}" "ready"

operational_api="$(curl_text "${BASE_URL}/command_sync?cmd=operational+api+eval")"
require_contains "${operational_api}" "[OPERATIONAL-API]"
runtime_state="$(curl_text "${BASE_URL}/command_sync?cmd=runtime+state+api+eval")"
require_contains "${runtime_state}" "[RUNTIME-STATE-API]"
artifact_api="$(curl_text "${BASE_URL}/command_sync?cmd=artifact+api+eval")"
require_contains "${artifact_api}" "[ARTIFACT-API]"

"${ROOT_DIR}/target/debug/eden-garm-api-conformance" \
    --base-url "${BASE_URL}" \
    --report "${REPORT_FILE}" \
    --timeout-sec 10

test -f "${REPORT_FILE}"
require_contains "$(cat "${REPORT_FILE}")" '"schema": "eden-sdk-conformance-report-v1"'
require_contains "$(cat "${REPORT_FILE}")" '"status": "passed"'
require_contains "$(cat "${REPORT_FILE}")" '"claim_allowed": false'

curl_text "${BASE_URL}/command?cmd=quit" >/dev/null
wait "${GARM_PID}"
GARM_PID=""

printf 'EDEN API conformance passed on %s with report %s\n' "${BASE_URL}" "${REPORT_FILE}"
