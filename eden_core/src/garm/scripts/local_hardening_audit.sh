#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"

log_info() {
    printf '[hardening] %s\n' "$*" >&2
}

run_step() {
    local -r label="$1"
    shift
    log_info "starting: ${label}"
    "$@"
    log_info "ok: ${label}"
}

require_file() {
    local -r path="$1"
    if [[ ! -f "${path}" ]]; then
        printf 'Required file missing: %s\n' "${path}" >&2
        exit 1
    fi
}

cd "${ROOT_DIR}"

require_file "deny.toml"
require_file "Cargo.lock"
require_file "eden_core/src/garm/RELEASE_AUDIT.md"
require_file "eden_core/src/garm/scripts/verify.sh"

run_step "locked dependency fetch" cargo fetch --locked
run_step "cargo-deny advisories" cargo deny check advisories
run_step "cargo audit" cargo audit
run_step "duplicate dependency inventory" cargo tree -d -p eden_core

log_info "hardening audit complete"
