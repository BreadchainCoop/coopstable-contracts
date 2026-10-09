#!/usr/bin/env bash
# sdk-probe.sh <path-to-yield-claimer-repo>
# Replays the yield-claimer's exact SDK calls (generated clients, stellar-sdk 14) against mainnet.
# Simulation only: nothing is signed or sent. Prints the fee the SDK would declare and whether the RPC
# folded an archived-entry restore into it (simType SUCCESS + minResourceFee, no restorePreamble).
set -euo pipefail
CLAIMER=${1:?path to yield-claimer repo}
OUT=${OUT:-$(dirname "$0")/out}/sdk; mkdir -p "$OUT"
ln -sfn "$CLAIMER/node_modules" "$OUT/node_modules"
for p in yield_distributor lending_yield_controller; do
  # the generated clients import type-only names as values; node's type stripping needs them marked `type`
  awk 'BEGIN{skip=0} /^import \{$/ && NR<=4 {skip=1;
      print "import { AssembledTransaction, Client as ContractClient, Spec as ContractSpec } from \x27@stellar/stellar-sdk/contract\x27;";
      print "import type { ClientOptions as ContractClientOptions, MethodOptions, Result } from \x27@stellar/stellar-sdk/contract\x27;"; next}
    skip==1 { if ($0 ~ /from .@stellar\/stellar-sdk\/contract.;/) {skip=0}; next } {print}' \
    "$CLAIMER/packages/$p/src/index.ts" > "$OUT/$p.ts"
done
cp "$(dirname "$0")/sdk-probe.mts" "$OUT/probe.mts"
node --experimental-transform-types --no-warnings "$OUT/probe.mts"
