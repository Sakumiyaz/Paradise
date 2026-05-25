#!/usr/bin/env bash
set -Eeuo pipefail

STATE_DIR="/tmp/eden_garm"
REQUEST_FILE=""
OUTPUT_FILE=""
HYBRID_MANIFEST=""

usage() {
    printf 'Usage: %s [--state-dir DIR] [--request FILE] [--hybrid-manifest FILE] [--output FILE]\n' "$0" >&2
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --state-dir)
            STATE_DIR="${2:?missing value for --state-dir}"
            shift 2
            ;;
        --request)
            REQUEST_FILE="${2:?missing value for --request}"
            shift 2
            ;;
        --hybrid-manifest)
            HYBRID_MANIFEST="${2:?missing value for --hybrid-manifest}"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="${2:?missing value for --output}"
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

REQUEST_FILE="${REQUEST_FILE:-${STATE_DIR}/voice_backend_request.txt}"
HYBRID_MANIFEST="${HYBRID_MANIFEST:-${STATE_DIR}/hybrid_voice_manifest.txt}"
OUTPUT_FILE="${OUTPUT_FILE:-${STATE_DIR}/voice_backend_output.txt}"

if [[ ! -f "${REQUEST_FILE}" && ! -f "${HYBRID_MANIFEST}" ]]; then
    printf 'No TTS request or hybrid manifest found: %s %s\n' "${REQUEST_FILE}" "${HYBRID_MANIFEST}" >&2
    exit 1
fi

backend="unknown"
input_artifact=""
text=""
hybrid_id=""
hybrid_tick=""
hybrid_backbone=""
hybrid_hierarchy=""
hybrid_prosody=""

if [[ -f "${REQUEST_FILE}" ]]; then
    while IFS='=' read -r key value; do
        case "${key}" in
            backend) backend="${value}" ;;
            input_artifact) input_artifact="${value}" ;;
            text) text="${value}" ;;
        esac
    done <"${REQUEST_FILE}"
fi

if [[ -f "${HYBRID_MANIFEST}" ]]; then
    while IFS='=' read -r key value; do
        case "${key}" in
            id) hybrid_id="${value}" ;;
            tick) hybrid_tick="${value}" ;;
            backbone) hybrid_backbone="${value}" ;;
            hierarchy) hybrid_hierarchy="${value}" ;;
            prosody) hybrid_prosody="${value}" ;;
            text)
                if [[ -z "${text}" ]]; then
                    text="${value}"
                fi
                ;;
        esac
    done <"${HYBRID_MANIFEST}"
fi

if [[ -z "${backend}" || "${backend}" == "unknown" ]]; then
    backend="local_hybrid_manifest_backend"
fi
if [[ -z "${input_artifact}" && -f "${HYBRID_MANIFEST}" ]]; then
    input_artifact="${HYBRID_MANIFEST}"
fi

mkdir -p "$(dirname -- "${OUTPUT_FILE}")"
tmp_file="${OUTPUT_FILE}.$$"
{
    printf 'backend=%s\n' "${backend}"
    printf 'status=rendered_text_manifest\n'
    printf 'input_artifact=%s\n' "${input_artifact}"
    printf 'hybrid_manifest=%s\n' "${HYBRID_MANIFEST}"
    printf 'hybrid_id=%s\n' "${hybrid_id}"
    printf 'hybrid_tick=%s\n' "${hybrid_tick}"
    printf 'hybrid_backbone=%s\n' "${hybrid_backbone}"
    printf 'hybrid_hierarchy=%s\n' "${hybrid_hierarchy}"
    printf 'hybrid_prosody=%s\n' "${hybrid_prosody}"
    printf 'audio_backend=absent\n'
    printf 'waveform_generated=false\n'
    printf 'text=%s\n' "${text}"
} >"${tmp_file}"
mv -- "${tmp_file}" "${OUTPUT_FILE}"

printf 'Wrote local TTS backend output: %s\n' "${OUTPUT_FILE}"
