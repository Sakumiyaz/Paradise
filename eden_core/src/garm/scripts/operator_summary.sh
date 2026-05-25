#!/usr/bin/env bash
set -Eeuo pipefail

STATE_DIR="${GARM_OPERATOR_STATE_DIR:-/tmp/eden_garm}"

usage() {
    printf 'Usage: %s [--state-dir DIR]\n' "$0" >&2
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --state-dir)
            if [[ $# -lt 2 ]]; then
                printf 'Missing value for --state-dir\n' >&2
                usage
                exit 1
            fi
            STATE_DIR="${2:?missing value for --state-dir}"
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

file_bytes() {
    local -r path="$1"
    if [[ -f "${path}" ]]; then
        wc -c <"${path}"
    else
        printf '0\n'
    fi
}

file_lines() {
    local -r path="$1"
    if [[ -f "${path}" ]]; then
        wc -l <"${path}"
    else
        printf '0\n'
    fi
}

contains_text() {
    local -r path="$1"
    local -r needle="$2"
    [[ -f "${path}" ]] && grep -F -- "${needle}" "${path}" >/dev/null 2>&1
}

report="${STATE_DIR}/garm_report.txt"
history="${STATE_DIR}/garm_report_history.jsonl"
export_file="${STATE_DIR}/garm_export.json"
artifacts=(
    "${report}"
    "${history}"
    "${export_file}"
    "${STATE_DIR}/voice_last.txt"
    "${STATE_DIR}/voice_backend_request.txt"
    "${STATE_DIR}/voice_backend_output.txt"
    "${STATE_DIR}/hrm_reasoner.json"
    "${STATE_DIR}/hrm_text_pretraining.json"
    "${STATE_DIR}/hrm_text_checkpoint_manifest.txt"
    "${STATE_DIR}/hrm_text_corpus_manifest.txt"
    "${STATE_DIR}/hrm_text_segments.jsonl"
    "${STATE_DIR}/voice_synthesizer.json"
    "${STATE_DIR}/organ_autonomy.json"
)

present=0
for artifact in "${artifacts[@]}"; do
    if [[ -f "${artifact}" ]]; then
        present=$((present + 1))
    fi
done

printf '[GARM-OPERATOR-SUMMARY] state_dir=%s files_present=%s expected_files=%s\n' \
    "${STATE_DIR}" "${present}" "${#artifacts[@]}"
printf -- '- report_bytes=%s history_entries=%s export_bytes=%s\n' \
    "$(file_bytes "${report}")" "$(file_lines "${history}")" "$(file_bytes "${export_file}")"

if contains_text "${report}" '[GARM-REPORT]'; then
    printf -- '- report=present\n'
else
    printf -- '- report=missing_or_unrecognized\n'
fi

if contains_text "${export_file}" '"schema": "garm-export-v1"'; then
    printf -- '- export_schema=garm-export-v1\n'
else
    printf -- '- export_schema=missing_or_unrecognized\n'
fi

if contains_text "${export_file}" '"checksum_fnv64"'; then
    printf -- '- export_integrity=present\n'
else
    printf -- '- export_integrity=missing\n'
fi

if [[ -d "${STATE_DIR}/backup" ]]; then
    printf -- '- backup_dir=present\n'
else
    printf -- '- backup_dir=missing\n'
fi
