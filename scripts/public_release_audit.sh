#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd -P)"

declare -a FAILURES=()
declare -a WARNINGS=()
STRICT_SCANNERS=false
SCANNER_SOURCE_DIR=""

cleanup() {
    if [[ -n "${SCANNER_SOURCE_DIR}" && -d "${SCANNER_SOURCE_DIR}" ]]; then
        rm -rf -- "${SCANNER_SOURCE_DIR}"
    fi
}

trap cleanup EXIT

for arg in "$@"; do
    case "${arg}" in
        --strict-scanners)
            STRICT_SCANNERS=true
            ;;
        *)
            printf '[public-audit] unknown argument: %s\n' "${arg}" >&2
            exit 2
            ;;
    esac
done

record_failure() {
    FAILURES+=("$1")
    printf '[public-audit] FAIL: %s\n' "$1" >&2
}

record_warning() {
    WARNINGS+=("$1")
    printf '[public-audit] WARN: %s\n' "$1" >&2
}

run_required() {
    local -r label="$1"
    shift
    printf '[public-audit] check: %s\n' "${label}"
    if ! "$@"; then
        record_failure "${label}"
    fi
}

check_tracked_forbidden_paths() {
    local matches
    matches="$(
        git -C "${ROOT_DIR}" ls-files |
            grep -E '(^|/)\.git($|/)|(^|/)\.git\.bak($|/)|\.bundle$|(__pycache__|\.pyc$|\.bak$|\.debug$|\.tmp$|\.env|secret|token|credential|private|\.pem$|\.p12$|\.pfx$|Cargo\.toml\.v|/reports/|language_reports)' || true
    )"
    if [[ -n "${matches}" ]]; then
        printf '%s\n' "${matches}" >&2
        return 1
    fi
}

check_forbidden_filesystem_artifacts() {
    local matches
    matches="$(
        find "${ROOT_DIR}" \
            -path "${ROOT_DIR}/.git" -prune -o \
            -path "${ROOT_DIR}/target" -prune -o \
            -path "${ROOT_DIR}/eden_core/target" -prune -o \
            -path "${ROOT_DIR}/mnemosyne/target" -prune -o \
            \( -name '.git.bak' -o -name '*.bundle' -o -name '*.git.backup' \) \
            -print
    )"
    if [[ -n "${matches}" ]]; then
        printf '%s\n' "${matches}" >&2
        return 1
    fi
}

check_untracked_secret_files() {
    local matches
    matches="$(
        find "${ROOT_DIR}" \
            -path "${ROOT_DIR}/.git" -prune -o \
            -path "${ROOT_DIR}/target" -prune -o \
            -path "${ROOT_DIR}/eden_core/target" -prune -o \
            -path "${ROOT_DIR}/mnemosyne/target" -prune -o \
            -type f \( -name '.env' -o -name '.env.*' -o -name '*.pem' -o -name '*.key' -o -name '*.p12' -o -name '*.pfx' \) \
            -print
    )"
    if [[ -n "${matches}" ]]; then
        printf '%s\n' "${matches}" >&2
        return 1
    fi
}

check_secret_patterns() {
    local -r pattern='(AKIA[0-9A-Z]{16}|sk-[A-Za-z0-9]{20,}|xox[baprs]-|ghp_[A-Za-z0-9_]{20,}|-----BEGIN (RSA |OPENSSH |EC |DSA |PRIVATE )?PRIVATE KEY-----|OPENAI_API_KEY=|ANTHROPIC_API_KEY=|GITHUB_TOKEN=|AWS_SECRET_ACCESS_KEY=)'
    if command -v rg >/dev/null 2>&1; then
        ! rg -n --hidden \
            -g '!.git' \
            -g '!target' \
            -g '!eden_core/target' \
            -g '!mnemosyne/target' \
            -g '!Cargo.lock' \
            -g '!scripts/public_release_audit.sh' \
            "${pattern}" \
            "${ROOT_DIR}"
        return
    fi

    ! grep -RInE \
        --exclude-dir='.git' \
        --exclude-dir='target' \
        --exclude='Cargo.lock' \
        --exclude='public_release_audit.sh' \
        "${pattern}" \
        "${ROOT_DIR}"
}

prepare_scanner_source() {
    if [[ -n "${SCANNER_SOURCE_DIR}" ]]; then
        printf '%s\n' "${SCANNER_SOURCE_DIR}"
        return
    fi

    SCANNER_SOURCE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/paradise-public-audit.XXXXXX")"
    while IFS= read -r -d '' tracked_path; do
        mkdir -p -- "${SCANNER_SOURCE_DIR}/$(dirname -- "${tracked_path}")"
        cp -Pp -- "${ROOT_DIR}/${tracked_path}" "${SCANNER_SOURCE_DIR}/${tracked_path}"
    done < <(git -C "${ROOT_DIR}" ls-files -z)

    printf '%s\n' "${SCANNER_SOURCE_DIR}"
}

check_public_docs() {
    local -a required=(
        "LICENSE"
        "SECURITY.md"
        "CONTRIBUTING.md"
        "CHANGELOG.md"
        "CODE_OF_CONDUCT.md"
        "PUBLIC_RELEASE.md"
        "docs/CLAIMS_AND_LIMITATIONS.md"
        "docs/THREAT_MODEL.md"
        "docs/PROJECT_STRUCTURE.md"
        "docs/HISTORY_REWRITE_PLAYBOOK.md"
        "docs/releases/v0.1.0-public-draft.md"
    )
    local missing=false
    for path in "${required[@]}"; do
        if [[ ! -s "${ROOT_DIR}/${path}" ]]; then
            printf 'missing required document: %s\n' "${path}" >&2
            missing=true
        fi
    done
    [[ "${missing}" == false ]]
}

check_optional_scanners() {
    local scanner_source
    scanner_source="$(prepare_scanner_source)"

    if command -v gitleaks >/dev/null 2>&1; then
        printf '[public-audit] check: gitleaks dir\n'
        gitleaks dir --redact --exit-code 1 "${scanner_source}"
    elif [[ "${STRICT_SCANNERS}" == true ]]; then
        record_failure "gitleaks not installed; strict scanner mode requires it"
    else
        record_warning "gitleaks not installed; fallback regex scan was used"
    fi

    if command -v trufflehog >/dev/null 2>&1; then
        printf '[public-audit] check: trufflehog filesystem\n'
        trufflehog filesystem "${scanner_source}" --no-update --fail
    elif [[ "${STRICT_SCANNERS}" == true ]]; then
        record_failure "trufflehog not installed; strict scanner mode requires it"
    else
        record_warning "trufflehog not installed; fallback regex scan was used"
    fi
}

main() {
    cd "${ROOT_DIR}"
    run_required "tracked forbidden path scan" check_tracked_forbidden_paths
    run_required "forbidden filesystem artifact scan" check_forbidden_filesystem_artifacts
    run_required "untracked secret file scan" check_untracked_secret_files
    run_required "secret pattern scan" check_secret_patterns
    run_required "public-ready document presence" check_public_docs
    run_required "JavaScript package-manager policy" make js-policy
    check_optional_scanners

    printf '[public-audit] warnings=%s failures=%s\n' "${#WARNINGS[@]}" "${#FAILURES[@]}"
    if ((${#FAILURES[@]} > 0)); then
        printf '[public-audit] status=failed\n' >&2
        exit 1
    fi
    printf '[public-audit] status=passed\n'
}

main
