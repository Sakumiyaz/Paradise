#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"

log_info() {
    printf '[verify] %s\n' "$*" >&2
}

run_step() {
    local -r label="$1"
    shift
    log_info "starting: ${label}"
    "$@"
    log_info "ok: ${label}"
}

cd "${ROOT_DIR}"

run_step "cargo fmt" cargo fmt --check -p eden_core
run_step "eden_garm tests" cargo test -p eden_core eden_garm --lib
run_step "eden_core examples and bins" cargo check -p eden_core --examples --bins
run_step "GARM API smoke" bash "${SCRIPT_DIR}/smoke_api.sh"
run_step "GARM restart persistence smoke" bash "${SCRIPT_DIR}/smoke_restart_persistence.sh"
run_step "GARM HRM retrieval regression" bash "${SCRIPT_DIR}/hrm_retrieval_regression.sh"
run_step "cargo-deny advisories" cargo deny check advisories
run_step "cargo audit" cargo audit

log_info "verification complete"
