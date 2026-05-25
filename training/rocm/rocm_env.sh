#!/usr/bin/env bash
set -Eeuo pipefail

if command -v rocminfo >/dev/null 2>&1; then
  printf 'rocm_detected=true\n'
  printf 'rocminfo=%s\n' "$(command -v rocminfo)"
else
  printf 'rocm_detected=false\n'
fi

if [ -d /opt/rocm ]; then
  printf 'rocm_path=/opt/rocm\n'
else
  printf 'rocm_path=unavailable\n'
fi

printf 'cpu_fallback=true\n'
