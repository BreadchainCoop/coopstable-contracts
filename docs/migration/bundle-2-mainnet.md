# Bundle 2 on mainnet: migrate and deploy

The production run of what `bundle-2-testnet.md` rehearsed on 2026-10-09/10. Same commands, no overrides: the
Makefile defaults are mainnet (`owner-mainnet`, `admin-mainnet`, the live Blend pool and USDC). Read sections 0
and 1 completely before typing anything; the order and the timing decision in section 1 matter.

> **2026-10-14: the claimer already ran one cycle on the old mainnet contracts.** The new yield-claimer was
> started against mainnet by mistake (13:30 local, 08:00 UTC) and ran harvest → recompound → finalize, then
> keep-alive, all signed and paid by `admin-mainnet`:
>
> | TX | Step | Fee (XLM) |
> |---|---|---:|
> | `ee00e154` | `harvest_yield` (restored the old adapter, controller and distributor) | 46.71 |
> | `d44a75c9` | `recompound_yield` | 0.00 |
> | `4037d783` | `finalize_distribution` (3,628,078 stroops = 3.63 USDC distributed, epoch 2 → 3) | 25.80 |
> | `c21b61fe` | keep-alive: every code, instance and data entry extended to 120 days | 7.66 |
> | `a30fa13a` | keep-alive top-up (the per-tick bug fixed in the claimer since) | 0.00 |
> | | **Total** | **80.17** |
>
> This is option B of the old Decision 1 (harvest first, then swap), paid by the admin instead of the owner. It
> changes this runbook in three ways: nothing has to be restored any more (all five code entries, including the
> old adapter, live until ledger 67,009,429, about 2027-02-11), the owner only pays uploads and the adapter
> deploy, and the unclaimed-yield decision is gone. Sections 0, 1 and 4 below are updated accordingly; the
> 2026-10-10 figures they replace are in git history.

## 0. What happens, what it costs, who pays

| Contract | Action | Signer |
|---|---|---|
| cUSD manager, adapter registry, yield distributor, yield controller | `upgrade` in place: same address, all storage kept | owner |
| Blend adapter | new deployment, then `register_adapter` + `add_support_for_asset` | owner, then admin |
| cUSD token, Blend pool, USDC | untouched | |

Calling `upgrade` executes the *old* code, so it needs the old code and instance live. Since 2026-10-14 they
are (see the note above): the 120 days of rent on code that the upgrade replaces were already paid by the
accidental claimer run, so the migration itself restores nothing. Keeping addresses and state this way is still
the point; the alternative, a full redeploy as in April, would lose members, epochs and the SAC admin wiring.
Charges at the floor rent rate (declared fees run ~15% higher and must be covered by the balance):

| Step | XLM | Paid by |
|---|---:|---|
| ~~Restore old controller, distributor, manager, registry~~ | already paid 2026-10-14 (admin) | |
| Upload new manager, registry, distributor, controller (4,236 / 5,792 / 13,709 / 13,825 B) | ~51.4 | owner |
| Deploy new adapter (17,088 B) | ~23.9 | owner |
| `upgrade` x4, `register_adapter`, `add_support_for_asset` | <0.5 | owner / admin |
| **Owner total** | **~76** | |
| Claimer's next keep-alive: extend the new code to 120 days again | ~9 | admin |

If the migration slips past ~2027-02-11, the old entries archive again and the restore rows come back
(~52 XLM for controller, distributor, manager and registry). After the migration, rent on the five code entries
is about 74 XLM per 120 days instead of 80, paid by the claimer's keep-alive on the 1st and 16th of each month
(~9 XLM per run) instead of in lumps by whoever touches a cold contract. The old code entries, the old adapter
included, keep their TTL until 2027-02-11 and then expire; nothing pays for them again.

Live state on mainnet read 1.76 GB on 2026-10-09 and 1.81 GB on 2026-10-10; the rent rate leaves its floor at
2.0 GB and roughly doubles by 2.1 GB. `make code-rent` prints both numbers; check it the day you migrate, and
if state is within reach of 2.0 GB start the claimer with `KEEP_ALIVE_LEDGERS=3110400` right away to prepay
180 days at the floor rate.

## 1. Pre-flight (read-only, nothing is sent)

**Wallets.** On 2026-10-14 (after the claimer run and a 150 XLM transfer admin → owner) `owner-mainnet` holds
**156.9 XLM** and `admin-mainnet` **282.8 XLM**. The owner needs ~90 XLM (the ~76 above plus the declared-fee
margin); the admin needs ~50 XLM free for the claimer's first months. Both are covered. Check:

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
make check-distribution-status                # false until ~2026-11-13 08:00 UTC (epoch 3 started 2026-10-14)
stellar contract invoke --send=no --network mainnet --source admin-mainnet --id $LENDING_YIELD_CONTROLLER_ID -- get_yield --protocol BC_LA --asset $USDC
stellar contract invoke --send=no --network mainnet --source admin-mainnet --id $BLEND_CAPITAL_ADAPTER_ID -- get_balance --user $LENDING_YIELD_CONTROLLER_ID --asset $USDC
make test-total-supply
```

On 2026-10-14 (after the claimer run): no pending harvest, distribution not available, yield 1,473 stroops,
controller position 156,161,299 (156.16 USDC), one member, epoch 3 started 08:00:22 UTC.

**Unclaimed yield: settled.** The new adapter adopts the controller's position as its principal at deploy time,
so yield accrued but not harvested *before* the swap stays in the pool as surplus collateral and is never
distributed. The 2026-10-14 run harvested and distributed the 3.63 USDC that had built up, so only what accrues
from then until the swap is left behind (1,473 stroops in the first ~70 minutes after the harvest): migrate
early and it is negligible. Do not run another manual harvest cycle first; it would only spend admin XLM.

**Timing.** Migrate **before ~2026-11-13 08:00 UTC**, when the next distribution opens; after that the claimer
would harvest on the old adapter again (cheap now, everything is warm) and the left-behind yield grows. The
adapter swap takes two transactions (`deploy`, then `register_adapter`), a few seconds apart; a member deposit
between them goes to the old adapter and is simply fine. **Stop the mainnet claimer for the migration window**
(it should not harvest between the pre-flight `get-pending-harvest` check and `register_adapter`), and pick a
quiet hour so the front-end and claimer can be redeployed in the same window.

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
```

`upgrade-all` prints one wasm hash per contract and stops at the first failure (nothing after a failed step is
sent). **Do not rerun it after a failure**: it starts again from the first contract and would re-upload and
re-upgrade the ones already done. Check which code each contract runs (`code_rent.py`, section 3) and finish
only the failed one by hand (a timed-out upload can simply be resent; on 2026-10-14 the controller's did exactly
that and landed with the same hash):

```bash
stellar contract upload --source owner-mainnet --network mainnet --wasm target/wasm32v1-none/release/<contract>.wasm
stellar contract invoke --source owner-mainnet --network mainnet --id <contract id> -- upgrade --new_wasm_hash <printed hash>
```

A simulation error mentioning the fee or `txInsufficientBalance` means the owner balance is short; top it up
first. `deploy-blend-adapter-full` prints the new adapter id and rewrites `deployed_addresses.mk` only;
`deployed_addresses.sh` keeps the old adapter until it is regenerated from the JSON below.

Record and commit:

```bash
# mainnet.contracts.json: BLEND_CAPITAL_ADAPTER = new id, keep the old one as BLEND_CAPITAL_ADAPTER_LEGACY_2026_04,
# hashes = the five printed hashes (adapter: `python3 -I docs/gas-analysis/scripts/code_rent.py $BLEND_CAPITAL_ADAPTER_ID`)
git add mainnet.contracts.json && git commit -m "chore(mainnet): bundle 2 upgrades and adapter swap"
make load-addresses NETWORK=mainnet && source deployed_addresses.sh   # BLEND_CAPITAL_ADAPTER_ID is now the new one
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

A member path pays no rent. Simulate a mint from any address holding USDC and a cUSD trustline; simulation
needs no secret key, so a public address works (the distributor's member list has one). The resource fee must
be a warm one (hundreds of thousands of stroops, not hundreds of millions; 292,439 on 2026-10-14):

```bash
M=GAPRPZYCIV3QPMCTWSRDNY64EJMZNCJFUCTJHQDQNW6RJ66TEVEH5UDU   # get_distribution_info .members
stellar contract invoke --source $M --network mainnet --id $C --build-only -- deposit_collateral \
  --protocol BC_LA --user $M --asset $USDC --amount 10000000 \
  | stellar tx simulate --network mainnet --source-account $M \
  | stellar xdr decode --type TransactionEnvelope --output json | jq '.tx.tx.ext.v1.resource_fee'
```

## 4. Deploy the yield-claimer

`PUBLIC` config already carries the mainnet ids; the adapter is read from the registry, so nothing in the
claimer references the new address. The claimer now lives at `RasenGUY/yield-claimer`; every push to `main`
builds `ghcr.io/rasenguy/yield-claimer:mainnet` (and `:mainnet-<sha>`), with the `mainnet` GitHub environment's
variables baked in (`NETWORK=PUBLIC`). Secrets are never in the image; pass them at runtime:

```
WALLET_SECRET_KEY / WALLET_PUBLIC_KEY   admin-mainnet (mainnet environment secrets)
MAX_FEE_XLM=100                         default
KEEP_ALIVE_LEDGERS=2073600              default, 120 days; raise to 3110400 (180 days) once no release is planned
ALERT_WEBHOOK_URL=                      optional secret, see the claimer README
```

Schedules are fixed per network in the claimer's `src/config/config.ts` (there is no `CRON_EXPRESSION` any
more), in the container's UTC:

- claim check daily at 00:00: claims once per 30-day distribution, within a day of it opening;
- keep-alive at 01:00 on the 1st and 16th: reads every TTL and submits only when the shortest one has drifted
  more than a day below 120 days, or an entry is missing or archived (then it restores first).

Locally, `pnpm start:mainnet` in the claimer runs the same thing from `.env.mainnet`. The startup log must say
`PUBLIC: claim-yield "0 0 * * *", keep-alive "0 1 1,16 * *"`. After the migration, the first keep-alive run
extends the new code entries back to 120 days; expect:

```
keep-alive: fee ~9 XLM                keep-alive: TX ...      (a `restore:` line first if anything was archived)
```

Later runs log about the same (15 days of rent on ~74 XLM per 120 days). A run that is more than about twice
the previous one means the rent rate has left the floor; a fee above `MAX_FEE_XLM` is refused and alerted, never
paid. The first claim after the migration is the ~2026-11-13 distribution, if the migration lands before it.

## 5. Deploy the front-end

Merge and deploy the client as usual. The cold guard needs no configuration; the mainnet config is unchanged.
Once live, a mint prompts the wallet for ~0.015 XLM and shows "The protocol is warming up" instead of a prompt
if the simulated fee ever exceeds 0.5 XLM.

## 6. Afterwards

- `make keep-alive` is the manual backstop if the claimer is down across a 1st or 16th.
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
