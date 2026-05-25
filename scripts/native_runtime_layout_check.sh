#!/usr/bin/env bash
set -euo pipefail

fail() {
    printf 'native runtime layout check failed: %s\n' "$1" >&2
    exit 1
}

require_file() {
    local -r path="$1"
    [[ -f "${path}" ]] || fail "missing required file ${path}"
}

require_absent() {
    local -r path="$1"
    [[ ! -e "${path}" ]] || fail "legacy implementation path still exists: ${path}"
}

find_refs() {
    local -r pattern="$1"
    shift
    grep -RInE \
        --exclude='native_runtime_layout_check.sh' \
        --exclude-dir='.git' \
        --exclude-dir='target' \
        --exclude-dir='decisions' \
        "${pattern}" \
        "$@" 2>/dev/null || true
}

require_file eden_core/src/garm/mod.rs
require_file eden_core/src/bin/eden_garm.rs
require_file eden_core/src/paradigms/mod.rs
require_file eden_core/src/garm_api_conformance.rs
require_file eden_core/src/garm_package_validator.rs
require_file eden_core/src/bin/eden_garm_api_conformance.rs
require_file eden_core/src/bin/eden_garm_package_validator.rs
require_file eden_core/examples/eden_garm.rs
require_file eden_core/examples/eden_garm_api_conformance.rs
require_file eden_core/examples/eden_garm_package_validator.rs

require_absent eden_core/examples/eden_garm
require_absent eden_core/examples/paradigms

for wrapper in \
    eden_core/examples/eden_garm.rs \
    eden_core/examples/eden_garm_api_conformance.rs \
    eden_core/examples/eden_garm_package_validator.rs
do
    lines="$(wc -l <"${wrapper}")"
    if (( lines > 12 )); then
        fail "${wrapper} must remain a thin compatibility wrapper, got ${lines} lines"
    fi
done

if ! grep -Eq 'eden_core::garm::main_entry\(\)' eden_core/examples/eden_garm.rs; then
    fail "eden_core/examples/eden_garm.rs must delegate to eden_core::garm::main_entry()"
fi
if ! grep -Eq 'eden_core::garm_api_conformance::main_entry\(\)' eden_core/examples/eden_garm_api_conformance.rs; then
    fail "eden_core/examples/eden_garm_api_conformance.rs must delegate to the native conformance module"
fi
if ! grep -Eq 'eden_core::garm_package_validator::main_entry\(\)' eden_core/examples/eden_garm_package_validator.rs; then
    fail "eden_core/examples/eden_garm_package_validator.rs must delegate to the native package validator module"
fi

forbidden_runtime_refs="$(
    find_refs \
        'target/debug/examples/eden_garm_(api_conformance|package_validator)|--example eden_garm_(api_conformance|package_validator)' \
        Makefile .github eden_core/src/garm/scripts scripts docs README.md \
)"
if [[ -n "${forbidden_runtime_refs}" ]]; then
    printf '%s\n' "${forbidden_runtime_refs}" >&2
    fail "operational runners must use native bins, not example targets"
fi

legacy_impl_refs="$(
    find_refs \
        'eden_core/examples/eden_garm/|eden_core/examples/paradigms/' \
        Makefile .github eden_core/src scripts docs README.md \
)"
if [[ -n "${legacy_impl_refs}" ]]; then
    printf '%s\n' "${legacy_impl_refs}" >&2
    fail "current docs/scripts must point at native src/garm and src/paradigms paths"
fi

printf 'native runtime layout check passed\n'
