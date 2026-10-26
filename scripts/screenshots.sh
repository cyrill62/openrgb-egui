#!/usr/bin/env bash
# Capture the GUI pages against the local mock OpenRGB server.
# Usage: scripts/screenshots.sh [output dir]
set -euo pipefail

cd "$(dirname "$0")/.."
OUT=${1:-docs/screenshots}
[[ "$OUT" = /* ]] || OUT="$PWD/$OUT"
PORT=26742

cargo build --features screenshot --bin pimpbox-rgb --example mock_server
mkdir -p "$OUT"
SERVER_LOG=$(mktemp)
./target/debug/examples/mock_server "$PORT" >"$SERVER_LOG" 2>&1 &
MOCK_PID=$!
cleanup() {
  kill "$MOCK_PID" 2>/dev/null || true
  wait "$MOCK_PID" 2>/dev/null || true
  rm -f "$SERVER_LOG"
}
trap cleanup EXIT

ready=0
for _ in {1..50}; do
  if ! kill -0 "$MOCK_PID" 2>/dev/null; then
    cat "$SERVER_LOG" >&2
    echo "error: mock server exited" >&2
    exit 1
  fi
  if grep -Fq "mock OpenRGB server listening on 127.0.0.1:$PORT" "$SERVER_LOG"; then
    ready=1
    break
  fi
  sleep 0.1
done
if [[ $ready -ne 1 ]]; then
  cat "$SERVER_LOG" >&2
  echo "error: mock server did not start" >&2
  exit 1
fi

PIMPBOX_RGB_SHOTS="$OUT" ./target/debug/pimpbox-rgb "127.0.0.1:$PORT"
shopt -s nullglob
SHOTS=("$OUT"/*.png)
(( ${#SHOTS[@]} > 0 )) || { echo "error: no PNG files were written to $OUT" >&2; exit 1; }
printf '%s\n' "${SHOTS[@]}"
