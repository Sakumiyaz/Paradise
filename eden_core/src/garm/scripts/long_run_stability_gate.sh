#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"

PORT="${GARM_LONGRUN_PORT:-8115}"
STATE_DIR="${GARM_LONGRUN_STATE_DIR:-/tmp/eden_garm_long_run}"
PID_FILE="${GARM_LONGRUN_PID_FILE:-/tmp/eden_garm_long_run.pid}"
LOG_FILE="${GARM_LONGRUN_LOG_FILE:-/tmp/eden_garm_long_run.log}"
STDOUT_FILE="${GARM_LONGRUN_STDOUT_FILE:-/tmp/eden_garm_long_run.stdout}"
EVIDENCE_FILE="${GARM_LONGRUN_EVIDENCE_FILE:-${STATE_DIR}/operational_evidence_bundle.json}"
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
    require_contains "$(<"${file}")" "${needle}"
}

edenctl() {
    "${ROOT_DIR}/target/debug/edenctl" --base-url "${BASE_URL}" "$@"
}

start_runtime() {
    EDEN_GARM_SKIP_LEGACY_MIGRATION="${EDEN_GARM_SKIP_LEGACY_MIGRATION:-1}" \
        EDEN_GARM_TTS_BACKEND="${GARM_LONGRUN_TTS_BACKEND:-local_stub}" \
        "${ROOT_DIR}/target/debug/eden-garm" \
        --daemon \
        --pid-file "${PID_FILE}" \
        --log-file "${LOG_FILE}" \
        --api-port "${PORT}" \
        --state-dir "${STATE_DIR}" \
        >"${STDOUT_FILE}" 2>&1 &
    GARM_PID=$!

    for _ in $(seq 1 50); do
        if curl -fsS --max-time 1 "${BASE_URL}/ready" >/dev/null 2>&1; then
            return 0
        fi
        sleep 1
    done

    printf 'EDEN runtime did not become ready on %s\n' "${BASE_URL}" >&2
    exit 1
}

stop_runtime() {
    if [[ -n "${GARM_PID}" ]] && kill -0 "${GARM_PID}" 2>/dev/null; then
        curl -fsS --max-time 3 "${BASE_URL}/command?cmd=quit" >/dev/null
        wait "${GARM_PID}"
        GARM_PID=""
    fi
}

rm -rf -- "${STATE_DIR}"
mkdir -p -- "${STATE_DIR}"

cargo build \
    --bin eden-garm \
    --bin edenctl \
    --bin eden-garm-package-validator \
    -p eden_core >/dev/null

start_runtime

status="$(edenctl status)"
require_contains "${status}" "[EDENCTL-STATUS]"
require_contains "${status}" "state=ready"

require_contains "$(edenctl command "operational api eval")" "[OPERATIONAL-API]"
require_contains "$(edenctl command "runtime state api eval")" "[RUNTIME-STATE-API]"
require_contains "$(edenctl command "artifact api eval")" "[ARTIFACT-API]"
require_contains "$(edenctl command "operational runtime eval")" "[OPERATIONAL-RUNTIME-PHASE]"
require_contains "$(edenctl locus eval)" "[EDEN-LOCUS-LAYER]"
require_contains "$(edenctl locus ingest "operator preference :: long-run context must stay governed")" "[LOCUS-INGEST]"
require_contains "$(edenctl locus context "long-run permission boundary")" "[LOCUS-CONTEXT]"
require_contains "$(edenctl forge eval)" "[EDEN-OPERATOR-FORGE]"
require_contains "$(edenctl forge synth "causal risk model for long-run validation")" "[OPERATOR-FORGE-SYNTH]"
require_contains "$(edenctl forge verify)" "[OPERATOR-FORGE-VERIFY]"

require_contains "$(edenctl dry-run evolve)" '"requires_human_approval": true'
require_contains "$(edenctl dry-run "operator forge synth causal risk")" '"handler": "gewc_formal_synthesis_body_handler"'

require_contains "$(edenctl command "gewc lifecycle world_model pause")" "[OPERATIONAL-LIFECYCLE]"
require_contains "$(edenctl status)" "state=degraded"
require_contains "$(edenctl recovery run)" "[OPERATIONAL-RECOVERY-RUN]"
require_contains "$(edenctl status)" "state=ready"

require_contains "$(edenctl command "operational scenario run")" "[OPERATIONAL-E2E-SCENARIO]"
require_contains "$(edenctl demo run)" "[OPERATIONAL-DEMO-SUITE]"
require_contains "$(edenctl command "operational replay run")" "[OPERATIONAL-REPLAY]"
require_contains "$(edenctl command --wait-sec 180 "readiness package")" "[READINESS-PACKAGE]"

edenctl evidence bundle \
    --state-dir "${STATE_DIR}" \
    --log-file "${LOG_FILE}" \
    --stdout-file "${STDOUT_FILE}" \
    --output "${EVIDENCE_FILE}" >/dev/null
require_file_contains "${EVIDENCE_FILE}" "eden-operational-evidence-bundle-v1"

stop_runtime

start_runtime
require_contains "$(edenctl status)" "[EDENCTL-STATUS]"
require_contains "$(edenctl schemas operational_status)" '"found": true'
stop_runtime

validator_output="$("${ROOT_DIR}/target/debug/eden-garm-package-validator" --state-dir "${STATE_DIR}")"
require_contains "${validator_output}" "[INDEPENDENT-VALIDATION] status=passed"

require_file_contains "${STATE_DIR}/readiness_package.json" "garm-readiness-package-v1"
require_file_contains "${STATE_DIR}/schema_registry.json" "eden-schema-registry-v1"
require_file_contains "${STATE_DIR}/operational_demo_suite.json" "eden-operational-demo-suite-v1"
require_file_contains "${STATE_DIR}/operational_recovery_plan.json" "eden-operational-recovery-plan-v1"
require_file_contains "${STATE_DIR}/eden_locus_layer.json" "eden-locus-layer-v1"
require_file_contains "${STATE_DIR}/eden_operator_forge.json" "eden-operator-forge-v1"
require_file_contains "${STATE_DIR}/locus_operator_bridge.json" "eden-locus-operator-bridge-v1"
require_file_contains "${STATE_DIR}/independent_validation_report.json" "passed"

printf 'EDEN long-run stability gate passed on %s with state dir %s\n' "${BASE_URL}" "${STATE_DIR}"
