#!/usr/bin/env bash
# simulate.sh <label> <contract_id> <fn> [args...]
# Simulates a contract call on mainnet through RPC and prints resources, fee and the call chain.
# Never submits: builds with --build-only and calls simulateTransaction directly.
set -euo pipefail
OUT=${OUT:-$(dirname "$0")/out}; mkdir -p "$OUT"
LABEL=$1; CID=$2; FN=$3; shift 3
RPC=${RPC:-https://mainnet.sorobanrpc.com/}
SRC=${SRC:-admin-mainnet}

XDR=$(stellar contract invoke --id "$CID" --source "$SRC" --network mainnet --build-only -- "$FN" "$@" 2>"$OUT/$LABEL.build.err")
curl -s -X POST "$RPC" -H 'Content-Type: application/json' \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"simulateTransaction\",\"params\":{\"transaction\":\"$XDR\",\"resourceConfig\":{\"instructionLeeway\":0}}}" \
  > "$OUT/$LABEL.sim.json"

if jq -e '.result.error' "$OUT/$LABEL.sim.json" >/dev/null 2>&1; then
  echo "== $LABEL: SIM ERROR"; jq -r '.result.error' "$OUT/$LABEL.sim.json" | head -c 1500; echo
fi

TD=$(jq -r '.result.transactionData // empty' "$OUT/$LABEL.sim.json")
[ -n "$TD" ] && echo "$TD" | stellar xdr decode --type SorobanTransactionData --output json > "$OUT/$LABEL.txdata.json"

: > "$OUT/$LABEL.events.json"
jq -r '.result.events[]? // empty' "$OUT/$LABEL.sim.json" | while read -r ev; do
  [ -n "$ev" ] && echo "$ev" | stellar xdr decode --type DiagnosticEvent --output json >> "$OUT/$LABEL.events.json" 2>/dev/null || true
done

echo "== $LABEL"
jq -r '.result | "minResourceFee=\(.minResourceFee // "?") stroops (\((.minResourceFee // 0 | tonumber) / 10000000) XLM)"' "$OUT/$LABEL.sim.json"
if [ -f "$OUT/$LABEL.txdata.json" ]; then
  jq -r '.resources | "instructions=\(.instructions) diskReadBytes=\(.disk_read_bytes // .read_bytes) writeBytes=\(.write_bytes) readOnly=\(.footprint.read_only|length) readWrite=\(.footprint.read_write|length)"' "$OUT/$LABEL.txdata.json"
  jq -r '"archived entries auto-restored (rw index): \((.ext | if type=="object" then .v1.archived_soroban_entries else [] end) // [])"' "$OUT/$LABEL.txdata.json"
fi
jq -r 'select(.event.body.v0.topics[0].symbol? == "fn_call") | "  call \(.event.body.v0.topics[1].bytes[0:8]) \(.event.body.v0.topics[2].symbol)"' "$OUT/$LABEL.events.json" 2>/dev/null | head -60 || true
