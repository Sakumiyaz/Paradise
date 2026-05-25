#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"
STATE_DIR="${GARM_RELEASE_STATE_DIR:-/tmp/eden_garm_release_checkpoint}"
HANDOFF_DIR="${GARM_RELEASE_HANDOFF_DIR:-/tmp/eden_garm_release_handoff}"
PORT="${GARM_RELEASE_PORT:-8120}"
TAG=""

usage() {
    printf 'Usage: %s [--state-dir DIR] [--handoff-dir DIR] [--port PORT] [--tag TAG]\n' "$0" >&2
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --state-dir)
            STATE_DIR="${2:?missing value for --state-dir}"
            shift 2
            ;;
        --handoff-dir)
            HANDOFF_DIR="${2:?missing value for --handoff-dir}"
            shift 2
            ;;
        --port)
            PORT="${2:?missing value for --port}"
            shift 2
            ;;
        --tag)
            TAG="${2:?missing value for --tag}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'Unknown option: %s\n' "$1" >&2
            usage
            exit 1
            ;;
    esac
done

BASE_URL="http://127.0.0.1:${PORT}"
PID_FILE="${STATE_DIR}.pid"
LOG_FILE="${STATE_DIR}.log"
STDOUT_FILE="${STATE_DIR}.stdout"
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

command_sync() {
    local -r command="$1"
    curl -fsS --max-time 20 --get --data-urlencode "cmd=${command}" "${BASE_URL}/command_sync"
}

rm -rf -- "${STATE_DIR}" "${HANDOFF_DIR}"
mkdir -p -- "${STATE_DIR}" "${HANDOFF_DIR}"

cat >"${STATE_DIR}/release_corpus.en.txt" <<'EOF'
GARM HRM-text release checkpoint corpus.
Hybrid voice manifest connects text, intent, prosody, acoustic planning and verification.
Local-only evidence feeds HRM runtime, learning, provenance, policy and maturity ledgers.
EOF

cargo build --bin eden-garm -p eden_core >/dev/null

EDEN_GARM_SKIP_LEGACY_MIGRATION="${EDEN_GARM_SKIP_LEGACY_MIGRATION:-1}" \
EDEN_GARM_TTS_BACKEND="local_hybrid_manifest_backend" \
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

require_contains "$(curl -fsS --max-time 5 "${BASE_URL}/ready")" "ready"

require_contains "$(command_sync "hrm text corpus ${STATE_DIR}/release_corpus.en.txt")" "exists=true"
require_contains "$(command_sync "hrm text ingest ${STATE_DIR}")" "[HRM-TEXT-INGEST]"
require_contains "$(command_sync "hrm text search release checkpoint")" "[HRM-TEXT-SEARCH]"
require_contains "$(command_sync "hrm text objective release checkpoint text priors")" "[HRM-TEXT-OBJECTIVE]"
require_contains "$(command_sync "hrm text plan")" "[HYBRID-VOICE-PLAN]"
require_contains "$(command_sync "hrm text run")" "weights_present=false"
require_contains "$(command_sync "hybrid voice synth release checkpoint voice")" "[HYBRID-VOICE-SYNTH]"
bash "${SCRIPT_DIR}/local_tts_backend.sh" --state-dir "${STATE_DIR}" >/dev/null
require_contains "$(command_sync "garm report")" "[GARM-REPORT]"
require_contains "$(command_sync "garm export")" "schema=garm-export-v1"
require_contains "$(command_sync "garm verify export")" "ok=true"
require_contains "$(command_sync "garm artifacts")" "hrm_text_corpus_manifest"
require_contains "$(command_sync "garm artifacts")" "hrm_text_segments"
require_contains "$(command_sync "garm backup")" "[GARM-BACKUP]"
require_contains "$(command_sync "save")" "HRM-text pretraining state also saved."

bash "${SCRIPT_DIR}/operator_summary.sh" --state-dir "${STATE_DIR}" >"${HANDOFF_DIR}/operator_summary.txt"
cp -R -- "${STATE_DIR}" "${HANDOFF_DIR}/state"
tar -C "$(dirname -- "${HANDOFF_DIR}")" -czf "${HANDOFF_DIR}.tar.gz" "$(basename -- "${HANDOFF_DIR}")"

if [[ -n "${TAG}" ]]; then
    git -C "${ROOT_DIR}" tag "${TAG}"
fi

printf '[GARM-RELEASE-CHECKPOINT] state_dir=%s handoff_dir=%s archive=%s tag=%s\n' \
    "${STATE_DIR}" "${HANDOFF_DIR}" "${HANDOFF_DIR}.tar.gz" "${TAG:-none}"
