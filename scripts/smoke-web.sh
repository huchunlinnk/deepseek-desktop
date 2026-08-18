#!/usr/bin/env bash
# Boot the desktop's host (`dsh web`) and verify the web surface serves.
# Exits 0 only when http://127.0.0.1:3080/ answers 200 within the timeout.
# This is the "identical surface, serving" half of the fitness function;
# rsi_parity covers the "identical surface, composed" half.
set -euo pipefail

HOST_CMD="${DSH_DESKTOP_HOST_CMD:-npx @deepseek-ai/dsh web}"
URL="${DSH_WEB_URL:-http://127.0.0.1:3080/}"
LOG="$(mktemp)"

echo "[smoke-web] starting host: ${HOST_CMD}"
# shellcheck disable=SC2086
${HOST_CMD} >"${LOG}" 2>&1 &
HOST_PID=$!
trap 'kill "${HOST_PID}" 2>/dev/null || true; rm -f "${LOG}"' EXIT

for _ in $(seq 1 60); do
  if curl -fsS "${URL}" >/dev/null 2>&1; then
    echo "[smoke-web] OK: ${URL} served"
    exit 0
  fi
  sleep 1
done

echo "[smoke-web] FAIL: ${URL} did not answer within 60s" >&2
cat "${LOG}" >&2 || true
exit 1
