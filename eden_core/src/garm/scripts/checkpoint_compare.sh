#!/usr/bin/env bash
set -Eeuo pipefail

LEFT=""
RIGHT=""

usage() {
    printf 'Usage: %s --left DIR --right DIR\n' "$0" >&2
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --left)
            LEFT="${2:?missing value for --left}"
            shift 2
            ;;
        --right)
            RIGHT="${2:?missing value for --right}"
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

if [[ -z "${LEFT}" || -z "${RIGHT}" ]]; then
    usage
    exit 1
fi
if [[ ! -d "${LEFT}" || ! -d "${RIGHT}" ]]; then
    printf 'Both checkpoints must be directories: left=%s right=%s\n' "${LEFT}" "${RIGHT}" >&2
    exit 1
fi

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

extract_json_field() {
    local -r path="$1"
    local -r field="$2"
    if [[ -f "${path}" ]]; then
        grep -m1 -o "\"${field}\": \"[^\"]*\"" "${path}" | cut -d'"' -f4 || true
    fi
}

extract_kv() {
    local -r path="$1"
    local -r key="$2"
    if [[ -f "${path}" ]]; then
        grep -m1 -E "^${key}=" "${path}" | cut -d= -f2- || true
    fi
}

left_export="${LEFT}/garm_export.json"
right_export="${RIGHT}/garm_export.json"
left_report="${LEFT}/garm_report.txt"
right_report="${RIGHT}/garm_report.txt"
left_segments="${LEFT}/hrm_text_segments.jsonl"
right_segments="${RIGHT}/hrm_text_segments.jsonl"
left_corpus="${LEFT}/hrm_text_corpus_manifest.txt"
right_corpus="${RIGHT}/hrm_text_corpus_manifest.txt"

left_checksum="$(extract_json_field "${left_export}" checksum_fnv64)"
right_checksum="$(extract_json_field "${right_export}" checksum_fnv64)"
left_segment_count="$(file_lines "${left_segments}")"
right_segment_count="$(file_lines "${right_segments}")"
left_corpora="$(extract_kv "${left_corpus}" corpora)"
right_corpora="$(extract_kv "${right_corpus}" corpora)"
left_verdict="$(grep -m1 -o 'verdict=[^ ]*' "${left_report}" 2>/dev/null | cut -d= -f2 || true)"
right_verdict="$(grep -m1 -o 'verdict=[^ ]*' "${right_report}" 2>/dev/null | cut -d= -f2 || true)"

if [[ "${left_checksum}" == "${right_checksum}" && "${left_segment_count}" == "${right_segment_count}" && "${left_corpora}" == "${right_corpora}" ]]; then
    drift="none"
else
    drift="detected"
fi

printf '[GARM-CHECKPOINT-COMPARE] drift=%s left=%s right=%s\n' "${drift}" "${LEFT}" "${RIGHT}"
printf -- '- export_checksum left=%s right=%s\n' "${left_checksum:-missing}" "${right_checksum:-missing}"
printf -- '- report_verdict left=%s right=%s\n' "${left_verdict:-missing}" "${right_verdict:-missing}"
printf -- '- export_bytes left=%s right=%s\n' "$(file_bytes "${left_export}")" "$(file_bytes "${right_export}")"
printf -- '- hrm_text_corpora left=%s right=%s\n' "${left_corpora:-0}" "${right_corpora:-0}"
printf -- '- hrm_text_segments left=%s right=%s\n' "${left_segment_count}" "${right_segment_count}"
