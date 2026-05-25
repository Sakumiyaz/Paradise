#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
PORT="${GARM_SMOKE_PORT:-8110}"
STATE_DIR="${GARM_SMOKE_STATE_DIR:-/tmp/eden_garm_smoke}"
PID_FILE="${GARM_SMOKE_PID_FILE:-/tmp/eden_garm_smoke.pid}"
LOG_FILE="${GARM_SMOKE_LOG_FILE:-/tmp/eden_garm_smoke.log}"
STDOUT_FILE="${GARM_SMOKE_STDOUT_FILE:-/tmp/eden_garm_smoke.stdout}"
BASE_URL="http://127.0.0.1:${PORT}"

cleanup() {
    if [[ -n "${GARM_PID:-}" ]] && kill -0 "${GARM_PID}" 2>/dev/null; then
        curl -fsS --max-time 3 "${BASE_URL}/command?cmd=quit" >/dev/null 2>&1 || true
        wait "${GARM_PID}" 2>/dev/null || true
    fi
}
trap cleanup EXIT

require_contains() {
    local haystack="$1"
    local needle="$2"
    if [[ "${haystack}" != *"${needle}"* ]]; then
        printf 'Expected output to contain: %s\nActual output:\n%s\n' "${needle}" "${haystack}" >&2
        exit 1
    fi
}

curl_text() {
    curl -fsS --max-time 10 "$1"
}

rm -rf "${STATE_DIR}"
mkdir -p "${STATE_DIR}"

cargo build --bin eden-garm -p eden_core >/dev/null

EDEN_GARM_SKIP_LEGACY_MIGRATION="${EDEN_GARM_SKIP_LEGACY_MIGRATION:-1}" \
EDEN_GARM_TTS_BACKEND="${GARM_SMOKE_TTS_BACKEND:-local_stub}" \
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

ready="$(curl_text "${BASE_URL}/ready")"
require_contains "${ready}" "ready"

state="$(curl_text "${BASE_URL}/state")"
require_contains "${state}" "${STATE_DIR}"
require_contains "${state}" "graph.json"

metrics="$(curl_text "${BASE_URL}/metrics")"
require_contains "${metrics}" '"ready":true'
require_contains "${metrics}" '"autonomous":true'
require_contains "${metrics}" '"cag_cache_entries":'
require_contains "${metrics}" '"cag_hits":'
require_contains "${metrics}" '"cag_misses":'
require_contains "${metrics}" '"cag_pending_actions":'
require_contains "${metrics}" '"cag_actions_executed":'
require_contains "${metrics}" '"cag_actions_blocked":'
require_contains "${metrics}" '"cag_autonomous_runs":'
require_contains "${metrics}" '"organ_pending_actions":'
require_contains "${metrics}" '"organ_actions_executed":'
require_contains "${metrics}" '"organ_actions_blocked":'
require_contains "${metrics}" '"organ_autonomous_runs":'

remember="$(curl_text "${BASE_URL}/command_sync?cmd=aprende+smoke+api+ok")"
require_contains "${remember}" "Recordado: smoke api ok"

queued="$(curl_text "${BASE_URL}/command?cmd=historial")"
require_contains "${queued}" "queued:"
cmd_id="$(printf '%s' "${queued}" | awk '{print $2}')"
sleep 1

result_once="$(curl_text "${BASE_URL}/command_result?id=${cmd_id}")"
require_contains "${result_once}" "cmd: historial"
result_twice="$(curl_text "${BASE_URL}/command_result?id=${cmd_id}")"
require_contains "${result_twice}" "cmd: historial"
forgot="$(curl_text "${BASE_URL}/command_forget?id=${cmd_id}")"
require_contains "${forgot}" "forgot: ${cmd_id}"

stop="$(curl_text "${BASE_URL}/command_sync?cmd=stop")"
require_contains "${stop}" "Autonomia pausada"
save="$(curl_text "${BASE_URL}/command_sync?cmd=save")"
require_contains "${save}" "Runtime state also saved"
start="$(curl_text "${BASE_URL}/command_sync?cmd=start")"
require_contains "${start}" "Autonomia reanudada"
load="$(curl_text "${BASE_URL}/command_sync?cmd=load")"
require_contains "${load}" "Runtime state also loaded"
metrics_after_load="$(curl_text "${BASE_URL}/metrics")"
require_contains "${metrics_after_load}" '"autonomous":false'

observatory="$(curl_text "${BASE_URL}/command_sync?cmd=observatorio")"
require_contains "${observatory}" "OBSERVATORIO GARM"
require_contains "${observatory}" "autonomous=false"

evolve="$(curl_text "${BASE_URL}/command_sync?cmd=evolve")"
require_contains "${evolve}" "Evolucion acotada completada"

reason="$(curl_text "${BASE_URL}/command_sync?cmd=que+es+smoke")"
require_contains "${reason}" "Definicion migrada"
audit="$(curl_text "${BASE_URL}/command_sync?cmd=garm+audit")"
require_contains "${audit}" "[GARM-AUDIT]"
require_contains "${audit}" "[ORGANOS-AUDIT] total=32"
report="$(curl_text "${BASE_URL}/command_sync?cmd=garm+report")"
require_contains "${report}" "[GARM-REPORT]"
require_contains "${report}" "[LastDeltas]"
report_second="$(curl_text "${BASE_URL}/command_sync?cmd=garm+report")"
require_contains "${report_second}" "[GARM-REPORT]"
report_history="$(curl_text "${BASE_URL}/command_sync?cmd=garm+report+history")"
require_contains "${report_history}" "[GARM-REPORT-HISTORY]"
require_contains "${report_history}" "entries=2"
report_api="$(curl_text "${BASE_URL}/report")"
require_contains "${report_api}" "[GARM-REPORT]"
report_api_compat="$(curl_text "${BASE_URL}/api/report")"
require_contains "${report_api_compat}" "[GARM-REPORT]"
report_history_api="$(curl_text "${BASE_URL}/report/history")"
require_contains "${report_history_api}" '"verdict":"ready"'
report_history_api_compat="$(curl_text "${BASE_URL}/api/report/history")"
require_contains "${report_history_api_compat}" '"tick":'
export_result="$(curl_text "${BASE_URL}/command_sync?cmd=garm+export")"
require_contains "${export_result}" "[GARM-EXPORT]"
require_contains "${export_result}" "schema=garm-export-v1"
require_contains "${export_result}" "checksum_fnv64="
verify_export_result="$(curl_text "${BASE_URL}/command_sync?cmd=garm+verify+export")"
require_contains "${verify_export_result}" "[GARM-VERIFY-EXPORT] ok=true"
require_contains "${verify_export_result}" "cryptographic=false"
export_api="$(curl_text "${BASE_URL}/export")"
require_contains "${export_api}" '"schema": "garm-export-v1"'
require_contains "${export_api}" '"integrity"'
verify_export_api="$(curl_text "${BASE_URL}/export/verify")"
require_contains "${verify_export_api}" "ok=true"
verify_export_api_compat="$(curl_text "${BASE_URL}/api/export/verify")"
require_contains "${verify_export_api_compat}" "algorithm=fnv64"
export_api_compat="$(curl_text "${BASE_URL}/api/export")"
require_contains "${export_api_compat}" '"mode": "diagnostic-read-only"'
import_result="$(curl_text "${BASE_URL}/command_sync?cmd=garm+import")"
require_contains "${import_result}" "[GARM-IMPORT] valid=true"
require_contains "${import_result}" "restored=false"
artifacts="$(curl_text "${BASE_URL}/command_sync?cmd=garm+artifacts")"
require_contains "${artifacts}" "[GARM-ARTIFACTS]"
require_contains "${artifacts}" "name=garm_report"
require_contains "${artifacts}" "name=garm_export"
require_contains "${artifacts}" "fnv64="
artifacts_api="$(curl_text "${BASE_URL}/artifacts")"
require_contains "${artifacts_api}" "[GARM-ARTIFACTS]"
artifacts_api_compat="$(curl_text "${BASE_URL}/api/artifacts")"
require_contains "${artifacts_api_compat}" "[GARM-ARTIFACTS-SUMMARY]"
hrm_run="$(curl_text "${BASE_URL}/command_sync?cmd=hrm+run+smoke")"
require_contains "${hrm_run}" "[HRM-RUN]"
require_contains "${hrm_run}" "[HRM-RUN-METRICS]"
printf 'smoke corpus\n' >"${STATE_DIR}/hrm_text_smoke.en.txt"
hrm_text_corpus="$(curl_text "${BASE_URL}/command_sync?cmd=hrm+text+corpus+${STATE_DIR}/hrm_text_smoke.en.txt")"
require_contains "${hrm_text_corpus}" "[HRM-TEXT-CORPUS]"
require_contains "${hrm_text_corpus}" "exists=true"
hrm_text_ingest="$(curl_text "${BASE_URL}/command_sync?cmd=hrm+text+ingest+${STATE_DIR}")"
require_contains "${hrm_text_ingest}" "[HRM-TEXT-INGEST]"
require_contains "${hrm_text_ingest}" "segments="
hrm_text_search="$(curl_text "${BASE_URL}/command_sync?cmd=hrm+text+search+smoke+corpus")"
require_contains "${hrm_text_search}" "[HRM-TEXT-SEARCH]"
require_contains "${hrm_text_search}" "status=hit"
require_contains "${hrm_text_search}" "confidence="
require_contains "${hrm_text_search}" "citation=doc:"
hrm_text_context="$(curl_text "${BASE_URL}/command_sync?cmd=rag+answer+smoke+corpus")"
require_contains "${hrm_text_context}" "[HRM-TEXT-CONTEXT-PACK]"
require_contains "${hrm_text_context}" "status=sufficient"
require_contains "${hrm_text_context}" "generation_restricted=true"
hrm_text_abstain="$(curl_text "${BASE_URL}/command_sync?cmd=rag+answer+zzzz+missing+hallucination+bait")"
require_contains "${hrm_text_abstain}" "[HRM-TEXT-ABSTAIN]"
hrm_text_eval="$(curl_text "${BASE_URL}/command_sync?cmd=rag+eval")"
require_contains "${hrm_text_eval}" "[HRM-TEXT-EVAL]"
require_contains "${hrm_text_eval}" "hallucination_guard=active"
readiness="$(curl_text "${BASE_URL}/command_sync?cmd=readiness")"
require_contains "${readiness}" "READINESS"
require_contains "${readiness}" "[READINESS-ARCHITECTURE]"
require_contains "${readiness}" "no_claim_until_all_gates_pass"
hrm_text_objective="$(curl_text "${BASE_URL}/command_sync?cmd=hrm+text+objective+smoke+text+priors")"
require_contains "${hrm_text_objective}" "[HRM-TEXT-OBJECTIVE]"
hrm_text_plan="$(curl_text "${BASE_URL}/command_sync?cmd=hrm+text+plan")"
require_contains "${hrm_text_plan}" "[HRM-TEXT-PLAN]"
hrm_text_run="$(curl_text "${BASE_URL}/command_sync?cmd=hrm+text+run")"
require_contains "${hrm_text_run}" "[HRM-TEXT-RUN]"
require_contains "${hrm_text_run}" "training_executed=false"
test -f "${STATE_DIR}/hrm_text_corpus_manifest.txt"
test -f "${STATE_DIR}/hrm_text_checkpoint_manifest.txt"
test -f "${STATE_DIR}/hrm_text_segments.jsonl"
test -f "${STATE_DIR}/hrm_text_context_pack.json"
tts="$(curl_text "${BASE_URL}/command_sync?cmd=tts+smoke+voz")"
require_contains "${tts}" "[VOZ-TTS]"
require_contains "${tts}" "backend=local_stub"
test -f "${STATE_DIR}/voice_backend_request.txt"
bash "${ROOT_DIR}/eden_core/src/garm/scripts/local_tts_backend.sh" --state-dir "${STATE_DIR}" >/dev/null
test -f "${STATE_DIR}/voice_backend_output.txt"
hybrid_voice="$(curl_text "${BASE_URL}/command_sync?cmd=hybrid+voice+synth+smoke+hybrid+voz")"
require_contains "${hybrid_voice}" "[HYBRID-VOICE-SYNTH]"
bash "${ROOT_DIR}/eden_core/src/garm/scripts/local_tts_backend.sh" --state-dir "${STATE_DIR}" --hybrid-manifest "${STATE_DIR}/hybrid_voice_manifest.txt" >/dev/null
require_contains "$(cat "${STATE_DIR}/voice_backend_output.txt")" "waveform_generated=false"
organs="$(curl_text "${BASE_URL}/command_sync?cmd=organos")"
require_contains "${organs}" "[ORGANOS] total=32"
organs_audit="$(curl_text "${BASE_URL}/command_sync?cmd=organos+audit")"
require_contains "${organs_audit}" "[ORGANOS-AUDIT] total=32"
organs_health="$(curl_text "${BASE_URL}/command_sync?cmd=organos+health")"
require_contains "${organs_health}" "[ORGANOS-HEALTH] total=32"
organs_actions="$(curl_text "${BASE_URL}/command_sync?cmd=organos+actions")"
require_contains "${organs_actions}" "[ORGANOS-ACTIONS]"
require_contains "${organs_actions}" "individual_organs=32"
metrics_after_cag="$(curl_text "${BASE_URL}/metrics")"
require_contains "${metrics_after_cag}" '"cag_cache_entries":'
require_contains "${metrics_after_cag}" '"cag_hits":'
require_contains "${metrics_after_cag}" '"cag_misses":'
require_contains "${metrics_after_cag}" '"cag_pending_actions":'
require_contains "${metrics_after_cag}" '"cag_actions_executed":'
require_contains "${metrics_after_cag}" '"cag_actions_blocked":'
require_contains "${metrics_after_cag}" '"cag_autonomous_runs":'
require_contains "${metrics_after_cag}" '"organ_pending_actions":'
require_contains "${metrics_after_cag}" '"organ_actions_executed":'
require_contains "${metrics_after_cag}" '"organ_actions_blocked":'
require_contains "${metrics_after_cag}" '"organ_autonomous_runs":'

organs_api="$(curl_text "${BASE_URL}/api/organs")"
require_contains "${organs_api}" '"organs":32'
require_contains "${organs_api}" '"pending":'
organs_api_actions="$(curl_text "${BASE_URL}/organs/actions")"
require_contains "${organs_api_actions}" "[ORGANOS-ACTIONS]"
organs_api_audit="$(curl_text "${BASE_URL}/organs/audit")"
require_contains "${organs_api_audit}" "[ORGANOS-AUTONOMY-AUDIT]"
organs_api_recovery="$(curl_text "${BASE_URL}/organs/recovery")"
require_contains "${organs_api_recovery}" "[ORGANOS-RECOVERY]"
backup="$(curl_text "${BASE_URL}/command_sync?cmd=garm+backup")"
require_contains "${backup}" "[GARM-BACKUP]"
compact="$(curl_text "${BASE_URL}/command_sync?cmd=garm+compact")"
require_contains "${compact}" "[GARM-COMPACT]"
restore="$(curl_text "${BASE_URL}/command_sync?cmd=garm+restore")"
require_contains "${restore}" "[GARM-RESTORE]"

test -f "${STATE_DIR}/graph.json"
test -f "${STATE_DIR}/capabilities.json"
test -f "${STATE_DIR}/runtime.json"
test -f "${STATE_DIR}/organ_autonomy.json"
test -f "${STATE_DIR}/garm_report.txt"
test -f "${STATE_DIR}/garm_report_history.jsonl"
test -f "${STATE_DIR}/garm_export.json"

curl_text "${BASE_URL}/command?cmd=quit" >/dev/null
wait "${GARM_PID}"
GARM_PID=""

printf 'GARM API smoke test passed on port %s with state dir %s\n' "${PORT}" "${STATE_DIR}"
