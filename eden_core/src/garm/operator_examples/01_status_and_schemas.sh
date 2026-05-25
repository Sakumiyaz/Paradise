#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/../../../.." && pwd -P)"
BASE_URL="${EDEN_BASE_URL:-http://127.0.0.1:8080}"

edenctl() {
    cargo run -q -p eden_core --bin edenctl -- --base-url "${BASE_URL}" "$@"
}

cd -- "${ROOT_DIR}"

edenctl status
edenctl schemas operational_status
