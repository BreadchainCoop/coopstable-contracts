# Bundle 2 on mainnet: migrate and deploy

The production run of what `bundle-2-testnet.md` rehearsed on 2026-10-09/10. Same commands, no overrides: the
Makefile defaults are mainnet (`owner-mainnet`, `admin-mainnet`, the live Blend pool and USDC). Read sections 0
and 1 completely before typing anything; the order and the two decisions in section 1 matter.

## 0. What happens, what it costs, who pays

| Contract | Action | Signer |
|---|---|---|
| cUSD manager, adapter registry, yield distributor, yield controller | `upgrade` in place: same address, all storage kept | owner |
| Blend adapter | new deployment, then `register_adapter` + `add_support_for_asset` | owner, then admin |
| cUSD token, Blend pool, USDC | untouched | |

Calling `upgrade` executes the *old* code, so every archived contract is restored first (the CLI folds the
restore into the invoke) and 120 days of rent are paid on code that is replaced in the same transaction. That is
the price of keeping addresses and state; the alternative, a full redeploy as in April, would lose members,
epochs and the SAC admin wiring. Actual charges at the floor rent rate (declared fees run ~15% higher and must be
covered by the balance):

| Step | XLM | Paid by |
|---|---:|---|
| Restore old controller code + instance (archived) | ~17.8 | owner |
| Restore old distributor code (instance still live until ~2026-10-26) | ~19.0 | owner |
| Restore old manager code + instance (archived) | ~6.4 | owner |
| Restore registry code, only if migrating after ~2026-10-26 | ~8.5 | owner |
| Upload new manager, registry, distributor, controller (4,236 / 5,792 / 13,709 / 13,825 B) | ~51.4 | owner |
| Deploy new adapter (17,088 B) | ~23.9 | owner |
| `upgrade` x4, `register_adapter`, `add_support_for_asset` | <0.5 | owner / admin |
| **Owner total** | **~120 (~130 after Oct 26)** | |
| Claimer's first tick: restore the remaining archived data entries, extend everything | <1 | admin |

The old adapter is never restored (it is replaced, which is why its 28.7 XLM does not appear). After the
migration, rent on the five code entries is about 74 XLM per 120 days instead of 80, and it is paid by the
claimer in ~0.34 XLM ticks every 12 hours instead of in lumps by whoever touches a cold contract.

Live state on mainnet read 1.76 GB on 2026-10-09 and 1.81 GB on 2026-10-10; the rent rate leaves its floor at
2.0 GB and roughly doubles by 2.1 GB. `make code-rent` prints both numbers; check it the day you migrate, and
if state is within reach of 2.0 GB start the claimer with `KEEP_ALIVE_LEDGERS=3110400` right away to prepay
180 days at the floor rate.

## 1. Pre-flight (read-only, nothing is sent)

**Wallets.** On 2026-10-10 `owner-mainnet` holds **6.9 XLM** and `admin-mainnet` 512.9 XLM. Fund the owner to
at least **150 XLM** before starting; the admin needs ~150 XLM free for the claimer's first weeks. Check:

```bash
for k in owner-mainnet admin-mainnet; do
  echo "$k $(curl -s https://horizon.stellar.org/accounts/$(stellar keys public-key $k) | jq -r '.balances[] | select(.asset_type=="native") | .balance') XLM"
done
```

**Protocol state.** All by simulation (`--send=no` submits nothing even when the contracts are cold):

```bash
make load-addresses NETWORK=mainnet          # deployed_addresses.mk/.sh from mainnet.contracts.json
export USDC=CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75
source deployed_addresses.sh                  # CUSD_ID, CUSD_MANAGER_ID, ..., BLEND_CAPITAL_ADAPTER_ID
make get-pending-harvest                      # must be "null": never swap the adapter with a harvest pending
make check-distribution-status                # true since July 2026
stellar contract invoke --send=no --network mainnet --source admin-mainnet --id $LENDING_YIELD_CONTROLLER_ID -- get_yield --protocol BC_LA --asset $USDC
stellar contract invoke --send=no --network mainnet --source admin-mainnet --id $BLEND_CAPITAL_ADAPTER_ID -- get_balance --user $LENDING_YIELD_CONTROLLER_ID --asset $USDC
make test-total-supply
```

On 2026-10-10: no pending harvest, distribution available, yield 3,497,364 stroops (3.50 USDC), controller
position 156,029,109 (156.03 USDC), cUSD supply 152,531,735 (152.53 USDC), one member, epoch 2.

**Decision 1: the 3.5 USDC of unclaimed yield.** The new adapter adopts the controller's position as its
principal at deploy time, so whatever has accrued but not been harvested *before* the swap stays in the pool as
surplus collateral and is never distributed. Two options:

- **A. Swap first (cheaper, default).** The 3.5 USDC remains in the pool backing cUSD; the claimer's first
  harvest after the swap takes the zero-yield path and simply advances the epoch. Cost as in the table above.
- **B. Harvest first, then swap.** Run one cycle on the old adapter, which distributes 3.15 USDC to the member
  and 0.35 to treasury. It restores the old adapter code (+28.7 XLM) and the old controller and distributor
  (which the upgrade needs anyway), all paid by `admin-mainnet`. Net extra: about 29 XLM. The owner then pays
  only uploads and the adapter deploy (~75 XLM).

If B: `make harvest-yield && make recompound-yield && make finalize-distribution`, then confirm
`make get-pending-harvest` is `null` again before going on. If the harvest reports no yield, the cycle still
advances the epoch and the three commands complete with the last two failing on "No pending harvest".

**Decision 2: timing.** The adapter swap takes two transactions (`deploy`, then `register_adapter`), a few
seconds apart; a member deposit between them goes to the old adapter and is simply fine. Pick a quiet hour
anyway so the front-end and claimer changes can be deployed in the same window.

**Build and check:**

```bash
git status                         # on develop with the Bundle 2 changes
make build && make test            # 134 tests
stellar contract info interface --wasm target/wasm32v1-none/release/blend_capital_adapter.wasm \
  | grep -cE 'fn (submit|add_yield|update_b_rate|init)\b'      # 0
stat -c %s target/wasm32v1-none/release/*.wasm                  # adapter 17088, controller 13825, distributor 13709, registry 5792, manager 4236
make code-rent                                                  # what the five live code entries cost today
```

## 2. Migrate

```bash
make upgrade-all                   # owner: upload + upgrade manager, registry, distributor, controller
make deploy-blend-adapter-full     # owner: deploy the new adapter (adopts the position); admin: register + support USDC
source deployed_addresses.sh       # BLEND_CAPITAL_ADAPTER_ID is now the new one
```

`upgrade-all` prints one wasm hash per contract and stops at the first failure (nothing after a failed step is
sent). A simulation error mentioning the fee or `txInsufficientBalance` means the owner balance is short; top
it up and rerun, it continues with the contract that failed. `deploy-blend-adapter-full` prints the new adapter
id and rewrites `deployed_addresses.mk`.

Record and commit:

```bash
# mainnet.contracts.json: BLEND_CAPITAL_ADAPTER = new id, keep the old one as BLEND_CAPITAL_ADAPTER_LEGACY_2026_04,
# hashes = the five printed hashes (adapter: `python3 -I docs/gas-analysis/scripts/code_rent.py $BLEND_CAPITAL_ADAPTER_ID`)
git add mainnet.contracts.json && git commit -m "chore(mainnet): bundle 2 upgrades and adapter swap"
```

## 3. Verify on chain

```bash
A=$BLEND_CAPITAL_ADAPTER_ID; C=$LENDING_YIELD_CONTROLLER_ID
stellar contract info interface --network mainnet --id $A | grep -cE 'fn (submit|add_yield)'   # 0
stellar contract invoke --send=no --network mainnet --source admin-mainnet --id $A -- get_yield --asset $USDC
stellar contract invoke --send=no --network mainnet --source admin-mainnet --id $A -- get_balance --user $C --asset $USDC
python3 -I docs/gas-analysis/scripts/code_rent.py $CUSD_MANAGER_ID $YIELD_ADAPTER_REGISTRY_ID $YIELD_DISTRIBUTOR_ID $C $A
```

Expected: `get_balance` equals the pre-flight position plus accrual; `get_yield` is only the accrual since the
deploy (stroops, never anything near the position); `code_rent.py` shows the five new hashes, each
`liveUntil` about 2,073,600 ledgers ahead, and a total near 74 XLM per 120 days.

A member path pays no rent. Simulate a mint from any wallet holding USDC and a cUSD trustline; the resource fee
must be a warm one (hundreds of thousands of stroops, not hundreds of millions):

```bash
stellar contract invoke --source <member-identity> --network mainnet --id $C --build-only -- deposit_collateral \
  --protocol BC_LA --user <member G address> --asset $USDC --amount 10000000 \
  | stellar tx simulate --network mainnet --source-account <member-identity> \
  | stellar xdr decode --type TransactionEnvelope --output json | jq '.tx.tx.ext.v1.resource_fee'
```

## 4. Deploy the yield-claimer

`PUBLIC` config already carries the mainnet ids; the adapter is read from the registry, so nothing in the
claimer references the new address. Merge the claimer branch, build the image (the repository's workflow does
it on push), and run it with:

```
NETWORK=PUBLIC
WALLET_SECRET_KEY / WALLET_PUBLIC_KEY   admin-mainnet
CRON_EXPRESSION="0 0-23/12 * * *"       every 12 hours
MAX_FEE_XLM=100
KEEP_ALIVE_LEDGERS=2073600              120 days; raise to 3110400 (180 days) once no release is planned
ALERT_WEBHOOK_URL=                      optional
```

`docker-compose.yml` passes all of these through and health-checks `/yield-claimer/health`. The first tick:
harvest (zero-yield path under option A: advances the epoch), then keep-alive, which restores the archived data
entries (cUSD token instance, `CusdSupply`, the controller's USDC balance, the distributor's member and
distribution records: under 1 XLM in total) and extends everything to 120 days. Expect in the log:

```
harvest_yield: fee 0.x XLM            Harvest complete. TX ...
keep-alive: fee 0.x XLM               keep-alive: TX ...      (a `restore:` line first if anything was archived)
```

Every later tick logs `keep-alive: fee 0.3x XLM`. A tick that is more than about twice the previous one means
the rent rate has left the floor; a fee above `MAX_FEE_XLM` is refused and alerted, never paid.

## 5. Deploy the front-end

Merge and deploy the client as usual. The cold guard needs no configuration; the mainnet config is unchanged.
Once live, a mint prompts the wallet for ~0.015 XLM and shows "The protocol is warming up" instead of a prompt
if the simulated fee ever exceeds 0.5 XLM.

## 6. Afterwards

- `make keep-alive` is the manual backstop if the claimer is down for more than a few days.
- `make code-rent` once a month: rent per contract, live state, distance to the 2.0 GB knee.
- When no further contract release is planned for a while, set `KEEP_ALIVE_LEDGERS=3110400` on the claimer to
  prepay 180 days at today's rate.
- The old adapter stays registered nowhere and expires on its own; nothing to do.

## 7. Rollback

The four previous code entries stay on chain for the rest of their 120-day TTL (they were restored by the
upgrade), so each contract can be put back with its previous hash from `mainnet.contracts.json` history:

```bash
make upgrade-contract CONTRACT_ID=<id> WASM_HASH=<previous hash>
```

The adapter swap is undone by registering the previous adapter again (its state is intact; it is archived, and
the registry call does not touch it, so the next harvest restores it for ~29 XLM):

```bash
stellar contract invoke --source admin-mainnet --network mainnet --id $YIELD_ADAPTER_REGISTRY_ID \
  -- register_adapter --yield_type LEND --protocol BC_LA --adapter_address <previous adapter id>
```

Roll the claimer and front-end back to their previous images; the old contracts are compatible with both.
