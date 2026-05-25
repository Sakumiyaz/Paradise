#!/bin/bash
set -e

SOCKET="/tmp/eden_mnemosyne.sock"
TIMEOUT=5

case "${1:-run}" in
    start)
        cargo build --release 2>/dev/null || cargo build
        nohup ./target/release/mnemosyne > /tmp/mnemosyne.log 2>&1 &
        sleep 1
        if [[ -S "$SOCKET" ]]; then
            echo "[MNEMOSYNE] started pid=$!"
        else
            echo "[MNEMOSYNE] start failed"
            exit 1
        fi
        ;;
    stop)
        pkill -f mnemosyne || true
        rm -f "$SOCKET"
        echo "[MNEMOSYNE] stopped"
        ;;
    status)
        if [[ -S "$SOCKET" ]]; then
            echo "[MNEMOSYNE] running socket=$SOCKET"
        else
            echo "[MNEMOSYNE] not running"
            exit 1
        fi
        ;;
    test)
        if [[ ! -S "$SOCKET" ]]; then
            echo "[MNEMOSYNE] not running"
            exit 1
        fi

        echo '{"op":"create_node","label":"test_node","properties":{"key":"value"},"ttl":3600,"embeddings":[0.1,0.2,0.3]}' | timeout "$TIMEOUT" socat - "$SOCKET"

        echo '{"op":"get_stale_nodes","days":7}' | timeout "$TIMEOUT" socat - "$SOCKET"

        echo '{"op":"search_similar","embedding":[0.1,0.2,0.3],"top_k":5}' | timeout "$TIMEOUT" socat - "$SOCKET"
        ;;
    log)
        tail -f /tmp/mnemosyne.log
        ;;
    *)
        echo "Usage: $0 {start|stop|status|test|log}"
        exit 1
        ;;
esac
