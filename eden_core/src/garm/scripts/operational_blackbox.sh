#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"

PORT="${GARM_BLACKBOX_PORT:-8114}"
STATE_DIR="${GARM_BLACKBOX_STATE_DIR:-/tmp/eden_garm_blackbox}"
PID_FILE="${GARM_BLACKBOX_PID_FILE:-/tmp/eden_garm_blackbox.pid}"
LOG_FILE="${GARM_BLACKBOX_LOG_FILE:-/tmp/eden_garm_blackbox.log}"
STDOUT_FILE="${GARM_BLACKBOX_STDOUT_FILE:-/tmp/eden_garm_blackbox.stdout}"
BASE_URL="http://127.0.0.1:${PORT}"

cleanup() {
    if [[ -n "${GARM_PID:-}" ]] && kill -0 "${GARM_PID}" 2>/dev/null; then
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

require_file_contains() {
    local -r file="$1"
    local -r needle="$2"
    if [[ ! -f "${file}" ]]; then
        printf 'Expected file to exist: %s\n' "${file}" >&2
        exit 1
    fi
    require_contains "$(<"${file}")" "${needle}"
}

curl_text() {
    curl -fsS --max-time 20 "$1"
}

rm -rf -- "${STATE_DIR}"
mkdir -p -- "${STATE_DIR}"

cargo build --bin eden-garm -p eden_core >/dev/null

EDEN_GARM_SKIP_LEGACY_MIGRATION="${EDEN_GARM_SKIP_LEGACY_MIGRATION:-1}" \
EDEN_GARM_TTS_BACKEND="${GARM_BLACKBOX_TTS_BACKEND:-local_stub}" \
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

health="$(curl_text "${BASE_URL}/api/health")"
require_contains "${health}" '"status":"ok"'
ready="$(curl_text "${BASE_URL}/ready")"
require_contains "${ready}" "ready"

runtime_snapshot="$(curl_text "${BASE_URL}/api/runtime/snapshot")"
require_contains "${runtime_snapshot}" '"schema": "eden-runtime-state-snapshot-v1"'
require_contains "${runtime_snapshot}" '"ready": true'

operational_eval="$(curl_text "${BASE_URL}/command_sync?cmd=operational+api+eval")"
require_contains "${operational_eval}" "[OPERATIONAL-API]"
require_contains "${operational_eval}" "[OPERATIONAL-CONTRACT]"

runtime_state_eval="$(curl_text "${BASE_URL}/command_sync?cmd=runtime+state+api+eval")"
require_contains "${runtime_state_eval}" "[RUNTIME-STATE-API]"

runtime_phase="$(curl_text "${BASE_URL}/command_sync?cmd=operational+runtime+eval")"
require_contains "${runtime_phase}" "[OPERATIONAL-RUNTIME-PHASE]"
require_contains "${runtime_phase}" "passed=8/8"
require_contains "${runtime_phase}" "locus_operator_bridge=governed"

locus_eval="$(curl_text "${BASE_URL}/command_sync?cmd=locus+eval")"
require_contains "${locus_eval}" "[EDEN-LOCUS-LAYER]"
locus_ingest="$(curl_text "${BASE_URL}/command_sync?cmd=locus+ingest+operator+preference+::+blackbox+context+must+stay+governed")"
require_contains "${locus_ingest}" "[LOCUS-INGEST]"
require_contains "${locus_ingest}" "status=accepted_context"
locus_context="$(curl_text "${BASE_URL}/command_sync?cmd=locus+context+blackbox+permission+boundary")"
require_contains "${locus_context}" "[LOCUS-CONTEXT]"

forge_eval="$(curl_text "${BASE_URL}/command_sync?cmd=operator+forge+eval")"
require_contains "${forge_eval}" "[EDEN-OPERATOR-FORGE]"
forge_synth="$(curl_text "${BASE_URL}/command_sync?cmd=operator+forge+synth+causal+risk+model+for+blackbox+validation")"
require_contains "${forge_synth}" "[OPERATOR-FORGE-SYNTH]"
require_contains "${forge_synth}" "candidate_recorded"
forge_verify="$(curl_text "${BASE_URL}/command_sync?cmd=operator+forge+verify")"
require_contains "${forge_verify}" "[OPERATOR-FORGE-VERIFY]"

contract="$(curl_text "${BASE_URL}/api/operational/contract")"
require_contains "${contract}" '"schema": "eden-operational-contract-v1"'
require_contains "${contract}" '"degraded"'
require_contains "${contract}" '"dry-run endpoints must not queue or execute commands"'

status="$(curl_text "${BASE_URL}/api/operational/status")"
require_contains "${status}" '"schema": "eden-operational-status-v1"'
require_contains "${status}" '"permissions_endpoint": "/api/operational/permissions"'

permissions="$(curl_text "${BASE_URL}/api/operational/permissions")"
require_contains "${permissions}" '"schema": "eden-operational-permissions-v1"'
require_contains "${permissions}" '"id": "remote_network"'

schemas="$(curl_text "${BASE_URL}/api/operational/schemas")"
require_contains "${schemas}" '"schema": "eden-schema-registry-v1"'
require_contains "${schemas}" 'eden-operational-status-v1'

schema_record="$(curl_text "${BASE_URL}/api/operational/schema?name=operational_status")"
require_contains "${schema_record}" '"schema": "eden-schema-registry-record-v1"'
require_contains "${schema_record}" '"found": true'

recovery_plan="$(curl_text "${BASE_URL}/api/operational/recovery")"
require_contains "${recovery_plan}" '"schema": "eden-operational-recovery-plan-v1"'

permission_audit="$(curl_text "${BASE_URL}/command_sync?cmd=operational+permissions+audit")"
require_contains "${permission_audit}" "[OPERATIONAL-PERMISSIONS-AUDIT]"

permission_diff="$(curl_text "${BASE_URL}/command_sync?cmd=operational+permissions+diff")"
require_contains "${permission_diff}" "[OPERATIONAL-PERMISSIONS-DIFF]"

lifecycle_pause="$(curl_text "${BASE_URL}/command_sync?cmd=gewc+lifecycle+world_model+pause")"
require_contains "${lifecycle_pause}" "[OPERATIONAL-LIFECYCLE]"
status_degraded="$(curl_text "${BASE_URL}/api/operational/status")"
require_contains "${status_degraded}" '"state": "degraded"'
recovery_run="$(curl_text "${BASE_URL}/command_sync?cmd=operational+recovery+run")"
require_contains "${recovery_run}" "[OPERATIONAL-RECOVERY-RUN]"
status_recovered="$(curl_text "${BASE_URL}/api/operational/status")"
require_contains "${status_recovered}" '"state": "ready"'

dry_run_safe="$(curl_text "${BASE_URL}/api/actions/dry-run?cmd=status")"
require_contains "${dry_run_safe}" '"standalone_execution_allowed": true'
require_contains "${dry_run_safe}" '"would_execute": false'
require_contains "${dry_run_safe}" '"persistent_permission"'

dry_run_risk="$(curl_text "${BASE_URL}/api/actions/dry-run?cmd=evolve")"
require_contains "${dry_run_risk}" '"permission_level": "destructive_or_autonomous"'
require_contains "${dry_run_risk}" '"requires_human_approval": true'
require_contains "${dry_run_risk}" '"permission_key": "local_bounded_self_improvement"'

dry_run_locus="$(curl_text "${BASE_URL}/api/actions/dry-run?cmd=locus+ingest+operator+preference")"
require_contains "${dry_run_locus}" '"handler": "gewc_locus_context_body_handler"'
require_contains "${dry_run_locus}" '"permission_key": "local_state_mutation"'

dry_run_forge="$(curl_text "${BASE_URL}/api/actions/dry-run?cmd=operator+forge+synth+causal+risk")"
require_contains "${dry_run_forge}" '"handler": "gewc_formal_synthesis_body_handler"'
require_contains "${dry_run_forge}" '"permission_key": "local_state_mutation"'

scenario="$(curl_text "${BASE_URL}/command_sync?cmd=operational+scenario+run")"
require_contains "${scenario}" "[OPERATIONAL-E2E-SCENARIO]"

replay="$(curl_text "${BASE_URL}/command_sync?cmd=operational+replay+run")"
require_contains "${replay}" "[OPERATIONAL-REPLAY]"

action_evidence="$(curl_text "${BASE_URL}/command_sync?cmd=action+evidence")"
require_contains "${action_evidence}" "[ACTION-EVIDENCE]"

demo_suite="$(curl_text "${BASE_URL}/command_sync?cmd=operational+demo+run")"
require_contains "${demo_suite}" "[OPERATIONAL-DEMO-SUITE]"
demos="$(curl_text "${BASE_URL}/api/operational/demos")"
require_contains "${demos}" '"schema": "eden-operational-demo-suite-v1"'
require_contains "${demos}" 'tool_security_replay'
require_contains "${demos}" 'locus_operator_bridge'

replay_index="$(curl_text "${BASE_URL}/api/operational/replay")"
require_contains "${replay_index}" '"schema": "eden-gewc-replay-index-v1"'
require_contains "${replay_index}" '"latest_decisions"'

replay_missing="$(curl_text "${BASE_URL}/api/operational/replay?decision_id=missing")"
require_contains "${replay_missing}" '"schema": "eden-gewc-decision-replay-v1"'
require_contains "${replay_missing}" '"found": false'

scenario_state="$(curl_text "${BASE_URL}/api/runtime/state?name=operational_e2e_scenario")"
require_contains "${scenario_state}" '"schema": "eden-operational-e2e-scenario-v1"'
require_contains "${scenario_state}" "blocked_high_risk_action"

replay_state="$(curl_text "${BASE_URL}/api/runtime/state?name=operational_replay_eval")"
require_contains "${replay_state}" '"schema": "eden-operational-replay-eval-v1"'
require_contains "${replay_state}" '"replayable": true'

locus_state="$(curl_text "${BASE_URL}/api/runtime/state?name=eden_locus_layer")"
require_contains "${locus_state}" '"schema": "eden-locus-layer-v1"'
locus_packet_state="$(curl_text "${BASE_URL}/api/runtime/state?name=locus_context_packet")"
require_contains "${locus_packet_state}" '"schema": "eden-locus-context-packet-v1"'
forge_state="$(curl_text "${BASE_URL}/api/runtime/state?name=eden_operator_forge")"
require_contains "${forge_state}" '"schema": "eden-operator-forge-v1"'
forge_graphs_state="$(curl_text "${BASE_URL}/api/runtime/state?name=operator_expression_graphs")"
require_contains "${forge_graphs_state}" '"schema":"eden-operator-expression-graph-v1"'
bridge_state="$(curl_text "${BASE_URL}/api/runtime/state?name=locus_operator_bridge")"
require_contains "${bridge_state}" '"schema": "eden-locus-operator-bridge-v1"'
require_contains "${bridge_state}" '"direct_memory_write": false'

locus_artifact="$(curl_text "${BASE_URL}/api/artifact?name=eden_locus_layer")"
require_contains "${locus_artifact}" '"schema": "eden-locus-layer-v1"'
forge_artifact="$(curl_text "${BASE_URL}/api/artifact?name=eden_operator_forge")"
require_contains "${forge_artifact}" '"schema": "eden-operator-forge-v1"'
bridge_artifact="$(curl_text "${BASE_URL}/api/artifact?name=locus_operator_bridge")"
require_contains "${bridge_artifact}" '"schema": "eden-locus-operator-bridge-v1"'

gewc_runtime="$(curl_text "${BASE_URL}/api/gewc/runtime")"
require_contains "${gewc_runtime}" "eden-gewc-runtime-api-v1"

require_file_contains "${STATE_DIR}/operational_contract.json" "eden-operational-contract-v1"
require_file_contains "${STATE_DIR}/operational_permissions.json" "eden-operational-permissions-v1"
require_file_contains "${STATE_DIR}/schema_registry.json" "eden-schema-registry-v1"
require_file_contains "${STATE_DIR}/operational_permissions_audit.json" "eden-operational-permissions-audit-v1"
require_file_contains "${STATE_DIR}/operational_recovery_plan.json" "eden-operational-recovery-plan-v1"
require_file_contains "${STATE_DIR}/operational_demo_suite.json" "eden-operational-demo-suite-v1"
require_file_contains "${STATE_DIR}/operational_e2e_scenario.json" "blocked_high_risk_action"
require_file_contains "${STATE_DIR}/eden_locus_layer.json" "eden-locus-layer-v1"
require_file_contains "${STATE_DIR}/eden_operator_forge.json" "eden-operator-forge-v1"
require_file_contains "${STATE_DIR}/locus_operator_bridge.json" "eden-locus-operator-bridge-v1"
require_file_contains "${STATE_DIR}/action_evidence.jsonl" "garm-action-evidence-v1"
test -s "${STATE_DIR}/global_executive_workspace_runtime.jsonl"

bash "${SCRIPT_DIR}/operational_evidence_bundle.sh" \
    --state-dir "${STATE_DIR}" \
    --log-file "${LOG_FILE}" \
    --stdout-file "${STDOUT_FILE}" \
    --output "${STATE_DIR}/operational_evidence_bundle.json" >/dev/null
require_file_contains "${STATE_DIR}/operational_evidence_bundle.json" "eden-operational-evidence-bundle-v1"

curl_text "${BASE_URL}/command?cmd=quit" >/dev/null
wait "${GARM_PID}"
GARM_PID=""

printf 'EDEN operational black-box passed on %s with state dir %s\n' "${BASE_URL}" "${STATE_DIR}"
