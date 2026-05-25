#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"
BASE_URL="${EDEN_BASE_URL:-http://127.0.0.1:8080}"
STATE_DIR="${GARM_EVIDENCE_STATE_DIR:-/tmp/eden_garm}"
LOG_FILE="${GARM_EVIDENCE_LOG_FILE:-/tmp/eden_garm.log}"
STDOUT_FILE="${GARM_EVIDENCE_STDOUT_FILE:-/tmp/eden_garm.stdout}"
OUTPUT_FILE="${GARM_EVIDENCE_OUTPUT:-${STATE_DIR}/operational_evidence_bundle.json}"

edenctl() {
    cargo run -q -p eden_core --bin edenctl -- --base-url "${BASE_URL}" "$@"
}

cd -- "${ROOT_DIR}"

edenctl command "operational api eval"
edenctl command "operational runtime eval"
edenctl demo run
edenctl evidence bundle \
    --state-dir "${STATE_DIR}" \
    --log-file "${LOG_FILE}" \
    --stdout-file "${STDOUT_FILE}" \
    --output "${OUTPUT_FILE}"
