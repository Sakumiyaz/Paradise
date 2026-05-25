#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"
CYCLES="${GARM_PERF_CYCLES:-512}"
STATE_DIR="${GARM_PERF_STATE_DIR:-/tmp/eden_garm_perf_smoke}"
STDOUT_FILE="${GARM_PERF_STDOUT_FILE:-/tmp/eden_garm_perf_smoke.stdout}"

log_info() {
    printf '[perf] %s\n' "$*" >&2
}

require_contains() {
    local -r haystack="$1"
    local -r needle="$2"
    if [[ "${haystack}" != *"${needle}"* ]]; then
        printf 'Expected output to contain: %s\nActual output:\n%s\n' "${needle}" "${haystack}" >&2
        exit 1
    fi
}

cd "${ROOT_DIR}"

if [[ -z "${STATE_DIR}" || "${STATE_DIR}" == "/" ]]; then
    printf 'Refusing unsafe state dir: %s\n' "${STATE_DIR}" >&2
    exit 1
fi

rm -rf -- "${STATE_DIR}"
mkdir -p -- "${STATE_DIR}"

log_info "building release example"
cargo build --release --bin eden-garm -p eden_core >/dev/null

start_ns="$(date +%s%N)"
"${ROOT_DIR}/target/release/eden-garm" \
    --no-interactive \
    --max-cycles "${CYCLES}" \
    --api-port 0 \
    --state-dir "${STATE_DIR}" \
    >"${STDOUT_FILE}" 2>&1
end_ns="$(date +%s%N)"

output="$(<"${STDOUT_FILE}")"
require_contains "${output}" "METRICS"
require_contains "${output}" "cycles: ${CYCLES}"

elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
if [[ "${elapsed_ms}" -le 0 ]]; then
    elapsed_ms=1
fi
cycles_per_sec=$(( CYCLES * 1000 / elapsed_ms ))

printf '[GARM-PERF] cycles=%s elapsed_ms=%s cycles_per_sec=%s state_dir=%s\n' \
    "${CYCLES}" "${elapsed_ms}" "${cycles_per_sec}" "${STATE_DIR}"
