#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"

STATE_DIR="${GARM_EVIDENCE_STATE_DIR:-${GARM_BLACKBOX_STATE_DIR:-/tmp/eden_garm_blackbox}}"
OUTPUT_FILE="${GARM_EVIDENCE_OUTPUT:-${STATE_DIR}/operational_evidence_bundle.json}"
LOG_FILE="${GARM_EVIDENCE_LOG_FILE:-${GARM_BLACKBOX_LOG_FILE:-/tmp/eden_garm_blackbox.log}}"
STDOUT_FILE="${GARM_EVIDENCE_STDOUT_FILE:-${GARM_BLACKBOX_STDOUT_FILE:-/tmp/eden_garm_blackbox.stdout}}"

usage() {
    cat <<'EOF'
Usage: operational_evidence_bundle.sh [--state-dir DIR] [--output FILE] [--log-file FILE] [--stdout-file FILE]
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --state-dir)
            STATE_DIR="${2:?missing --state-dir value}"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="${2:?missing --output value}"
            shift 2
            ;;
        --log-file)
            LOG_FILE="${2:?missing --log-file value}"
            shift 2
            ;;
        --stdout-file)
            STDOUT_FILE="${2:?missing --stdout-file value}"
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            printf 'Unknown argument: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

json_escape() {
    local value="$1"
    value="${value//\\/\\\\}"
    value="${value//\"/\\\"}"
    value="${value//$'\n'/\\n}"
    value="${value//$'\r'/\\r}"
    printf '%s' "${value}"
}

file_record() {
    local -r name="$1"
    local -r path="$2"
    local present=false
    local bytes=0
    local checksum=0
    if [[ -f "${path}" ]]; then
        present=true
        bytes="$(wc -c <"${path}" | tr -d ' ')"
        checksum="$(cksum "${path}" | awk '{print $1}')"
    fi
    printf '{"name":"%s","path":"%s","present":%s,"bytes":%s,"cksum":"%s"}' \
        "$(json_escape "${name}")" \
        "$(json_escape "${path}")" \
        "${present}" \
        "${bytes}" \
        "$(json_escape "${checksum}")"
}

mkdir -p -- "$(dirname -- "${OUTPUT_FILE}")"

records=(
    "operational_contract:${STATE_DIR}/operational_contract.json"
    "operational_permissions:${STATE_DIR}/operational_permissions.json"
    "operational_permissions_audit:${STATE_DIR}/operational_permissions_audit.json"
    "operational_permissions_diff:${STATE_DIR}/operational_permissions_diff.json"
    "operational_permissions_history:${STATE_DIR}/operational_permissions_history.jsonl"
    "operational_recovery_plan:${STATE_DIR}/operational_recovery_plan.json"
    "operational_demo_suite:${STATE_DIR}/operational_demo_suite.json"
    "schema_registry:${STATE_DIR}/schema_registry.json"
    "operational_e2e_scenario:${STATE_DIR}/operational_e2e_scenario.json"
    "operational_replay_eval:${STATE_DIR}/operational_replay_eval.json"
    "eden_locus_layer:${STATE_DIR}/eden_locus_layer.json"
    "locus_authority_model:${STATE_DIR}/locus_authority_model.json"
    "locus_evidence_vault:${STATE_DIR}/locus_evidence_vault.json"
    "locus_permission_matrix:${STATE_DIR}/locus_permission_matrix.json"
    "locus_context_packet:${STATE_DIR}/locus_context_packet.json"
    "eden_operator_forge:${STATE_DIR}/eden_operator_forge.json"
    "operator_primitive_basis:${STATE_DIR}/operator_primitive_basis.json"
    "operator_expression_graphs:${STATE_DIR}/operator_expression_graphs.jsonl"
    "operator_verification_report:${STATE_DIR}/operator_verification_report.json"
    "operator_model_registry:${STATE_DIR}/operator_model_registry.json"
    "locus_operator_bridge:${STATE_DIR}/locus_operator_bridge.json"
    "action_evidence:${STATE_DIR}/action_evidence.jsonl"
    "gewc_runtime:${STATE_DIR}/global_executive_workspace_runtime.jsonl"
    "gewc_runtime_state:${STATE_DIR}/global_executive_workspace_runtime_state.json"
    "runtime_stdout:${STDOUT_FILE}"
    "runtime_log:${LOG_FILE}"
)

{
    printf '{\n'
    printf '  "schema": "eden-operational-evidence-bundle-v1",\n'
    printf '  "claim_allowed": false,\n'
    printf '  "agi_claim": false,\n'
    printf '  "generated_by": "operational_evidence_bundle.sh",\n'
    printf '  "state_dir": "%s",\n' "$(json_escape "${STATE_DIR}")"
    printf '  "repo_root": "%s",\n' "$(json_escape "${ROOT_DIR}")"
    printf '  "records": [\n'
    for index in "${!records[@]}"; do
        IFS=':' read -r name path <<<"${records[$index]}"
        printf '    '
        file_record "${name}" "${path}"
        if [[ "${index}" -lt $((${#records[@]} - 1)) ]]; then
            printf ','
        fi
        printf '\n'
    done
    printf '  ],\n'
    printf '  "verification_commands": ["make operational-blackbox", "make eden-api-conformance"],\n'
    printf '  "reexecution_policy": "bundle records evidence only and does not re-execute actions"\n'
    printf '}\n'
} >"${OUTPUT_FILE}"

printf 'EDEN operational evidence bundle written to %s\n' "${OUTPUT_FILE}"
