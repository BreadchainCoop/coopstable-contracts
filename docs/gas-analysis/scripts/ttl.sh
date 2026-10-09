#!/usr/bin/env bash
# ttl.sh <label>  -- for every footprint key of a simulate.sh run, print entry size and TTL status.
set -uo pipefail
OUT=${OUT:-$(dirname "$0")/out}
RPC=${RPC:-https://mainnet.sorobanrpc.com/}
TX=$OUT/$1.txdata.json
LATEST=$(curl -s -X POST "$RPC" -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"getLatestLedger"}' | jq -r .result.sequence)
echo "latest ledger: $LATEST"
for side in read_only read_write; do
  n=$(jq ".resources.footprint.$side|length" "$TX")
  for ((i=0;i<n;i++)); do
    KJ=$(jq -c ".resources.footprint.$side[$i]" "$TX")
    KX=$(echo "$KJ" | stellar xdr encode --type LedgerKey 2>/dev/null)
    RES=$(curl -s -X POST "$RPC" -H 'Content-Type: application/json' -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getLedgerEntries\",\"params\":{\"keys\":[\"$KX\"]}}")
    LIVE=$(echo "$RES" | jq -r '.result.entries[0].liveUntilLedgerSeq // "none"')
    BYTES=$(echo "$RES" | jq -r '.result.entries[0].xdr // ""' | base64 -d 2>/dev/null | wc -c)
    DESC=$(echo "$KJ" | jq -r 'if .contract_code then "code:"+.contract_code.hash[0:10] elif .contract_data then "data:"+(.contract_data.contract|tostring)[0:10]+" key="+((.contract_data.key|tostring)[0:60]) else (.|tostring)[0:50] end')
    if [ "$LIVE" = "none" ]; then ST="n/a"; elif [ "$LIVE" -lt "$LATEST" ]; then ST="EXPIRED"; else ST="live(+$(( (LIVE-LATEST)/17280 ))d)"; fi
    printf "%-10s %6s B  liveUntil=%-10s %-14s %s\n" "$side" "$BYTES" "$LIVE" "$ST" "$DESC"
  done
done
