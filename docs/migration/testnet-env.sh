# Testnet rehearsal environment for docs/migration/bundle-2-testnet.md.
# Source it from the Bundle 2 checkout in EVERY new shell (functions and variables do not survive a new terminal):
#     source docs/migration/testnet-env.sh
# Works in bash and zsh.

if [ ! -f mainnet.contracts.json ] || [ ! -f Makefile ]; then
  echo "source this from the coopstable-contracts checkout (cd there first)" >&2
  return 1 2>/dev/null || exit 1
fi

export MAIN="$(pwd)"
export RPC=https://soroban-testnet.stellar.org
export USDC=CAQCFVLOBK5GIULPNZRGATJJMIZL5BSP7X5YJVMGCPTUEPFM4AVSRCJU
export USDC_ISSUER=GATALTGTWIOT6BUDBCZM3Q4OQ4BO2COLOAZ7IYSKPLC2PMSOPPGF5V56
export BLND=CB22KRA3YZVCNCQI64JQ5WE7UY2VAV7WFLK6A2JN3HEX56T2EDAFO7QF
export BLEND_POOL=CCEBVDYM32YNYCVNRXQKDFFPISJJCV557CDZEIRBEE4NCV4KHPQ44HGF

# make with every mainnet default of the Makefile overridden for testnet
tmake() {
  make NETWORK=testnet OWNER_KEY=owner ADMIN_KEY=admin \
    TREASURY="$(stellar keys public-key treasury)" \
    BLEND_POOL_ID="$BLEND_POOL" BLEND_TOKEN_ID="$BLND" USDC_ID="$USDC" \
    DISTRIBUTION_PERIOD=300 "$@"
}

# ttl <contract-id>: prints "<latest ledger> <liveUntilLedgerSeq> <id>" for the contract instance entry.
# 17,280 ledgers is one day; 2,073,600 is 120 days.
ttl() {
  local key
  key=$(echo "{\"contract_data\":{\"contract\":\"$1\",\"key\":\"ledger_key_contract_instance\",\"durability\":\"persistent\"}}" \
        | stellar xdr encode --type LedgerKey)
  curl -s -X POST "$RPC" -H 'Content-Type: application/json' -H 'User-Agent: coopstable' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getLedgerEntries\",\"params\":{\"keys\":[\"$key\"]}}" \
    | jq -r --arg id "$1" '"\(.result.latestLedger) \(.result.entries[0].liveUntilLedgerSeq // "not found")  \($id)"'
}

# ids of the deployment currently in deployed_addresses.mk (re-source after any deploy to refresh them)
if [ -f deployed_addresses.mk ]; then
  _addr() { grep "^$1" deployed_addresses.mk | cut -d= -f2 | tr -d ' '; }
  export CUSD="$(_addr CUSD_ID)"
  export MANAGER="$(_addr CUSD_MANAGER_ID)"
  export REGISTRY="$(_addr YIELD_ADAPTER_REGISTRY_ID)"
  export DISTRIBUTOR="$(_addr YIELD_DISTRIBUTOR_ID)"
  export CONTROLLER="$(_addr LENDING_YIELD_CONTROLLER_ID)"
  export ADAPTER="$(_addr BLEND_CAPITAL_ADAPTER_ID)"
  unset -f _addr
  echo "MAIN=$MAIN"
  echo "CONTROLLER=$CONTROLLER"
  echo "ADAPTER=$ADAPTER"
else
  echo "MAIN=$MAIN (no deployed_addresses.mk yet)"
fi
