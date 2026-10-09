# Bundle 2 on testnet: deploy, migrate, verify

A rehearsal of exactly what mainnet will get. Testnet has no Coopstable deployment (the December 2025 testnet
reset wiped it), so you first deploy **today's mainnet code** as a baseline, seed it with a member deposit and
one yield cycle, then run the migration against it and check every promise Bundle 2 makes. Everything here is
testnet XLM. Section 8 is the mainnet run; it is the same commands without the overrides.

## 0. What changes and why

| Contract | On chain | Why |
|---|---|---|
| cUSD manager, adapter registry, yield distributor, yield controller | `upgrade` in place (same address, storage kept) | TTL bump 7/30 days to 120 days, extension only on admin paths, so no member transaction ever pays rent |
| Blend adapter | **new deployment** + `register_adapter` | The deployed version has no `upgrade` entrypoint and ships the test mock pool, whose unauthenticated `submit` can grow the adapter's instance storage until every adapter write fails. The new one has no mock, an owner, `upgrade`, and a constructor that adopts the controller's existing Blend position as its principal |
| cUSD token, Blend pool, USDC | untouched | |

Off chain: the yield-claimer gets a fee ceiling, a 24-hour "available but unclaimed" alert, and a keep-alive
transaction every tick (the backstop for everything the contracts no longer extend themselves: manager, registry,
and the data entries member transactions read). The front-end refuses to prompt a wallet when the simulated fee
says the contracts are cold. Background: `docs/gas-analysis/README.md`, `SOLUTIONS.md` section 7.

Mainnet cost of the migration, at the floor rent rate: four uploads of about 51 XLM in total, one adapter deploy
of about 23 XLM, plus a few tenths of an XLM of invokes. The adapter alone was 28.7 XLM per 120 days and is
now about 23; the rest is unchanged in size.

## 1. Prerequisites

```bash
stellar --version            # 26.1.0 works on testnet (it warns about protocol 29; harmless)
rustup target list --installed | grep wasm32v1-none
command -v jq curl node pnpm
stellar keys ls              # must list: owner admin treasury member_1 member_2 member_3
```

Fund the six testnet identities (the reset emptied them; friendbot gives 10,000 XLM each):

```bash
for k in owner admin treasury member_1 member_2 member_3; do stellar keys fund $k --network testnet; done
```

The Makefile's header is hard-wired to mainnet (`NETWORK`, key names, Blend pool, BLND, USDC, treasury,
30-day distribution period), and the checks below need a few helpers. They live in
`docs/migration/testnet-env.sh`; **source it from this checkout in every new terminal** (functions and variables
do not survive a new shell):

```bash
cd ~/work/coopstable/coopstable-contracts        # the Bundle 2 checkout
source docs/migration/testnet-env.sh
```

It defines:

- `tmake <target>`: `make` with every mainnet default overridden for testnet (`NETWORK=testnet`, `owner`/`admin`
  keys, `treasury`, Blend TestnetV2 pool, BLND, USDC, `DISTRIBUTION_PERIOD=300`).
- `ttl <contract-id>`: prints `<latest ledger> <liveUntilLedgerSeq> <id>` for the contract instance entry. This is
  the single number every check below looks at; 17,280 ledgers is one day, 2,073,600 is 120 days.
- `$MAIN`, `$RPC`, `$USDC`, `$BLEND_POOL`, and, once `deployed_addresses.mk` exists, `$CUSD`, `$MANAGER`,
  `$REGISTRY`, `$DISTRIBUTOR`, `$CONTROLLER`, `$ADAPTER` from it. Re-source after any deploy to refresh them.

`deployed_addresses.mk` and `deployed_addresses.sh` are **gitignored**, hold whichever network you last deployed
or loaded, and the Makefile `-include`s the first one; every deploy target rewrites them. The tracked source of
truth is `mainnet.contracts.json` / `testnet.contracts.json`, and `make load-addresses NETWORK=<net>` regenerates
both files from the json. So there is nothing to back up: before any mainnet command run
`make load-addresses NETWORK=mainnet`; to come back to the testnet deployment, `make load-addresses NETWORK=testnet`
(after you have recorded its ids in `testnet.contracts.json`, step 3).

## 2. Blend testnet assets and test USDC

Blend's TestnetV2 pool, its mock USDC and BLND are live (checked by RPC on 2026-10-09); confirm before spending
time, and confirm USDC is a reserve of the pool:

```bash
ttl CCEBVDYM32YNYCVNRXQKDFFPISJJCV557CDZEIRBEE4NCV4KHPQ44HGF     # pool: liveUntil must be > latest
stellar contract invoke --send=no --network testnet --source admin \
  --id CCEBVDYM32YNYCVNRXQKDFFPISJJCV557CDZEIRBEE4NCV4KHPQ44HGF -- get_reserve_list   # must list $USDC
```

`member_1` needs test USDC (and `admin` too if you want `tmake test-deposit`, which deposits from admin). Blend's
testnet faucet hands out 1,000 USDC, 5,000 BLND and some wETH/wBTC per request, **but only for assets whose
trustline it creates itself**: it returns a transaction already signed by the issuer that adds the trustlines
and pays, and it silently skips any asset the account already trusts. So do **not** create the USDC trustline
by hand; if you already did (with a zero balance), drop it first:

```bash
# only if `curl -s https://horizon-testnet.stellar.org/accounts/$(stellar keys public-key member_1) | jq '.balances'`
# already shows a USDC line with balance 0:
stellar tx new change-trust --source-account member_1 --network testnet \
  --line USDC:GATALTGTWIOT6BUDBCZM3Q4OQ4BO2COLOAZ7IYSKPLC2PMSOPPGF5V56 --limit 0
```

Then fetch, co-sign and send the faucet transaction (the account signs for its own trustlines):

```bash
for k in member_1 admin; do
  curl -s "https://ewqw4hx7oa.execute-api.us-east-1.amazonaws.com/getAssets?userId=$(stellar keys public-key $k)" \
    | jq -r . \
    | stellar tx sign --sign-with-key $k --network testnet \
    | stellar tx send --network testnet
  stellar contract invoke --send=no --network testnet --source $k --id $USDC -- balance --id "$(stellar keys public-key $k)"
done
# expect "10000000000" (1,000 USDC) per account
```

The same faucet sits behind the "Faucet" button of https://testnet.blend.capital if you prefer a wallet.

## 3. Baseline: deploy today's mainnet code

The migration must start from the contracts mainnet runs today, so build and deploy them from a second checkout
at the commit **before** the Bundle 2 changes (while they are uncommitted on `develop`, that is `develop` itself;
after they are committed, use the parent commit):

```bash
MAIN=$(pwd)                                               # the Bundle 2 checkout, used again below
git worktree add ../coopstable-contracts-base develop     # some tooling relocates worktrees, so resolve the path:
BASE=$(git worktree list | awk '/coopstable-contracts-base/ {print $1}')
cd "$BASE"
rm -f deployed_addresses.mk deployed_addresses.sh        # gitignored, so a fresh worktree normally has none
tmake quick-deploy                                        # build + deploy-protocol, about 10 minutes
```

`deploy-protocol` deploys the cUSD asset (issuer = `owner`) and its SAC, the manager, registry, distributor,
controller and adapter, hands SAC admin to the manager, wires controller and distributor, registers the adapter
for `LEND`/`BC_LA` with USDC supported, and (testnet only) adds `member_1..3` as members with cUSD trustlines.
Expected tail of the output: `show-addresses` printing six ids and `Addresses saved`.

Bring the ids into the Bundle 2 checkout and record the baseline:

```bash
cp deployed_addresses.mk deployed_addresses.sh "$MAIN/"
cd "$MAIN"
source docs/migration/testnet-env.sh          # picks up the ids you just deployed
export ADAPTER_OLD=$ADAPTER                   # the adapter the migration will replace
for id in $CUSD $MANAGER $REGISTRY $DISTRIBUTOR $CONTROLLER $ADAPTER; do ttl $id; done
python3 -I docs/gas-analysis/scripts/code_rent.py --rpc $RPC $MANAGER $REGISTRY $DISTRIBUTOR $CONTROLLER $ADAPTER
```

Expected TTLs: everything, cUSD token included, shows `liveUntil` about **7 days** past the latest ledger
(120,960 ledgers), not 120. Testnet's minimum TTL for a new entry is 7 days (mainnet's is 120), and the old
contracts never extend beyond that: their bump is 7 days with a 6-day threshold, a no-op until the last day.
The new contracts extend to 120 days on admin paths, which is what section 6(c) checks. If you pause the rehearsal
for more than a week the baseline entries expire; the CLI restores them automatically on the next invoke, for a
few testnet stroops.

The last command prints, per contract, the wasm hash it runs and its rent; keep that output, it is the rollback
list (section 7). Put the ids and hashes into `testnet.contracts.json` now: from then on
`make load-addresses NETWORK=testnet` can always recreate `deployed_addresses.mk`.

## 4. Seed state

A member deposit (mints cUSD), then one yield cycle as admin once the 5-minute period has passed:

```bash
stellar contract invoke --source member_1 --network testnet --id $CONTROLLER -- deposit_collateral \
  --protocol BC_LA --user "$(stellar keys public-key member_1)" --asset $USDC --amount 1000000000   # 100 USDC
stellar contract invoke --send=no --network testnet --source admin --id $ADAPTER_OLD -- get_balance --user $CONTROLLER --asset $USDC
# expect 1000000000 (or a hair more once the pool accrues)
sleep 300
tmake test-claim-yield
```

Testnet pools rarely yield anything in five minutes, so the expected result is the zero-yield path: `harvest_yield`
returns `0` and advances the epoch; `recompound_yield` and `finalize_distribution` then fail with
"No pending harvest exists". That is correct. If the pool did yield, all three stages succeed and `member_1`
receives cUSD. Either way `ttl $CONTROLLER` now shows the baseline behaviour you are about to change: still about 7 days, never
120 (the old contracts only ever extend by 7 days, and only when under 6).

## 5. Migrate

From the Bundle 2 checkout (`cd "$MAIN"`):

```bash
make build && make test
stellar contract info interface --wasm target/wasm32v1-none/release/blend_capital_adapter.wasm \
  | grep -cE 'fn (submit|add_yield|update_b_rate|init)\b'                     # must print 0
stat -c %s target/wasm32v1-none/release/blend_capital_adapter.wasm             # about 17,100 bytes

tmake upgrade-all                 # uploads + upgrades manager, registry, distributor, controller; owner signs
tmake deploy-blend-adapter-full   # deploys the new adapter (owner, adopts the position), registers it (admin)
source docs/migration/testnet-env.sh   # $ADAPTER is now the new one; $ADAPTER_OLD still holds the previous
```

`upgrade-all` prints one wasm hash per contract; `deploy-blend-adapter-full` prints the new adapter id and
rewrites `deployed_addresses.mk`. Record all of them in `testnet.contracts.json` (the adapter's hash:
`python3 -I docs/gas-analysis/scripts/code_rent.py --rpc $RPC $ADAPTER`).

Point the other two repositories at this deployment:

- `../yield-claimer/src/config/config.ts`, `TESTNET.contracts`: distributor, controller, manager, registry and
  cUSD ids. The adapter id is not configured; the claimer asks the registry.
- `../coopstable-client/src/app/config.ts`, `TESTNET`: `yieldController`, `yieldDistributor`, `cusdManager`
  contract ids, `cusd.contractId` and `cusd.issuer` (= `stellar keys public-key owner`).

## 6. Verify

**(a) The swap did not invent yield.** Right after step 5:

```bash
stellar contract invoke --send=no --network testnet --source admin --id $ADAPTER -- get_yield --asset $USDC
stellar contract invoke --send=no --network testnet --source admin --id $ADAPTER -- get_balance --user $CONTROLLER --asset $USDC
```

`get_balance` is the position from step 4 plus whatever accrued since. `get_yield` is only the accrual since the
adapter was deployed: a few hundred stroops on testnet, never anything near the position size. (The constructor
adopted the position as principal at deploy time; without that, the next member deposit would have made
`get_yield` report the whole position.)

**(b) A member transaction pays no rent.** Snapshot TTLs, mint again as `member_1`, snapshot again: the
`liveUntilLedgerSeq` of controller, adapter, manager and registry must be identical before and after.

```bash
for id in $CONTROLLER $ADAPTER $MANAGER $REGISTRY; do ttl $id; done
stellar contract invoke --source member_1 --network testnet --id $CONTROLLER -- deposit_collateral \
  --protocol BC_LA --user "$(stellar keys public-key member_1)" --asset $USDC --amount 100000000      # 10 USDC
for id in $CONTROLLER $ADAPTER $MANAGER $REGISTRY; do ttl $id; done
```

To see the fee itself (not just the TTLs): build the same call without sending, simulate it, and read the
resource fee out of the assembled envelope. It is in stroops and should be well under 1,000,000 (0.1 XLM),
because nothing is restored or extended:

```bash
stellar contract invoke --source member_1 --network testnet --id $CONTROLLER --build-only -- deposit_collateral \
  --protocol BC_LA --user "$(stellar keys public-key member_1)" --asset $USDC --amount 100000000 \
  | stellar tx simulate --network testnet --source-account member_1 \
  | stellar xdr decode --type TransactionEnvelope --output json | jq '.tx.tx.ext.v1.resource_fee'
```

**(c) Admin paths extend to 120 days.** Wait for the period (5 minutes since the last epoch advance), then run
the whole cycle:

```bash
tmake harvest-yield
ttl $CONTROLLER; ttl $ADAPTER                      # both now latest + 2,073,600
tmake recompound-yield
tmake finalize-distribution
ttl $DISTRIBUTOR                                   # now latest + 2,073,600 as well
```

With yield to claim, the harvest extends the controller (admin) and the adapter (`claim_yield`), and the
finalize extends the distributor (`distribute_yield`). With zero yield, the harvest alone extends all three
(`advance_epoch`, `update_epoch_principal`) and the two later stages fail with "No pending harvest", which is
fine. Do not leave a harvest pending: the next harvest would refuse with error 1003. The manager and registry
are **not** extended by the cycle, by design; that is the claimer's job, next.

**(d) The claimer keeps the rest alive and refuses absurd fees.** In `../yield-claimer`:

```bash
pnpm install
NETWORK=TESTNET \
WALLET_SECRET_KEY="$(stellar keys secret admin)" WALLET_PUBLIC_KEY="$(stellar keys public-key admin)" \
CRON_EXPRESSION="*/2 * * * *" \
pnpm start:dev
```

(Inline variables override whatever `.env` holds, so the mainnet `.env` stays untouched.)

Within two minutes the log shows `keep-alive: fee 0.0xxx XLM` and `keep-alive: TX <hash>` (on the very first run,
if anything was archived, a `restore:` line first). Then `ttl` on the manager, registry and cUSD token ids shows
`latest + 2,073,600` (`ttl $MANAGER; ttl $REGISTRY; ttl $CUSD`). Stop it, run once with `MAX_FEE_XLM=0.001`, and the same tick logs
`... exceeds MAX_FEE_XLM=0.001, not submitting` and posts to `ALERT_WEBHOOK_URL` if you set one. Remove the
override. `curl localhost:3001/yield-claimer/health` returns `{"status":"ok"}` (the Docker healthcheck path).

**(e) The front-end blocks cold transactions.** In `../coopstable-client` with `NEXT_PUBLIC_DEFAULT_NETWORK=TESTNET`
in `.env`, `pnpm dev`, Freighter on testnet with `member_1` imported: mint 1 USDC succeeds. To see the guard
without waiting 120 days, set `COLD_FEE_STROOPS = 1` in `src/app/services/ContractService/yieldController.ts`,
reload, try to mint: the dialog shows "The protocol is warming up, please try again in a few minutes." and the
wallet is never prompted. Revert the constant.

**(f) Rent of the new adapter:**

```bash
python3 -I docs/gas-analysis/scripts/code_rent.py --rpc $RPC $ADAPTER       # about 23 XLM / 120 days
```

## 7. Rollback (testnet or mainnet)

The previous code entries stay on chain for the rest of their TTL, so the four upgraded contracts can be put
back with the hashes recorded in step 3:

```bash
tmake upgrade-contract CONTRACT_ID=<id> WASM_HASH=<previous hash>
```

The adapter swap is undone by registering the previous adapter again (its state, including the epoch principal,
is still there):

```bash
stellar contract invoke --source admin --network testnet --id $REGISTRY \
  -- register_adapter --yield_type LEND --protocol BC_LA --adapter_address $ADAPTER_OLD
```

## 8. Mainnet

The production run is its own document, `bundle-2-mainnet.md`: same commands without the overrides, plus the
pre-flight checks, the cost table (the owner pays about 120 XLM, mostly to restore the old code that `upgrade`
has to execute), the decision about the 3.5 USDC of unclaimed yield, and the claimer and front-end deployment.
