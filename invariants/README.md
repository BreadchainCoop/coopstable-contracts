# Backing invariant property test

Random sequences of deposits, withdrawals, interest, pool losses, harvest stages, cancels, pauses, top-ups,
rescues and time jumps, with "cUSD is backed" checked after every step. Every outcome is predicted from
public state first, so an unexpected success or error code is a failure too.

The five CoopStable contracts run as wasm, read from disk when the test starts. Only the Blend pool is a mock
(`mod pool` in `src/main.rs`), written from the Blend v2 source.

## Run

```sh
make invariants                                   # from the repo root: builds the wasm, runs every stage
make invariants INVARIANT_STAGES="s3 s4" CASES=3000

# or by hand, after `stellar contract build`
cd invariants
cargo run --release -- s1          # 2,000 cases, about 5 minutes
cargo run --release -- dust        # scripted case for L-09
cargo run --release -- absorb      # scripted case for I-06
```

Use `--release`: the wasm interpreter is far too slow in a debug build. Build output, results
(`target/results/`) and saved failing cases (`target/proptest-regressions/`) all live under `target/`,
which is ignored by git.

| Stage | What it adds | Expected |
|---|---|---|
| `s0` | Pre-fix controller wasm (mainnet until 2026-11-19) | Fail at P0 |
| `s1` | Deposits, withdrawals, interest, harvest stages, members, emissions, pauses, top-ups, rescues | Pass |
| `s2` | + `cancel_harvest` in state `Recompounded` only | Pass |
| `s3` | + `cancel_harvest` in any state | Pass |
| `s4` | `s2` + pool losses | Pass |
| `s4b` | Pool losses, no harvest stages, withdrawals capped at the position | Pass |
| `s5` | `s1` with 50,000 cases | Pass |

Exit code 0 is a pass, 1 a failed property (the shrunk case and its step log are printed), 2 an aborted run.
A pass only counts if the report ends with `coverage: every action reached its minimum of 200`;
`make invariants` fails otherwise.

## Properties

| ID | Property |
|---|---|
| P0 | Every call's result and effect on balances, supply and the harvest record was predicted |
| P1 | No action except a pool loss deepens a shortfall beyond rounding |
| P2 | With no pool loss, supply never exceeds backing by more than rounding |
| P3 | Deposits and withdrawals are at par, within rounding |
| P4 | `cusd_total_supply` equals the sum of balances |
| P5 | Reported yield is real surplus |
| P6 | The controller holds USDC only while a harvest is between stage 1 and stage 2 |
| P7 | Everyone can leave: all cUSD redeems, up to rounding |
| X1 | USDC is conserved and the mock pool can pay the position |

## Environment

| Variable | Default | Meaning |
|---|---|---|
| `WASM_DIR` | `../target/wasm32v1-none/release` | Where the five `.wasm` files are read from |
| `PREFIX_WASM` | `target/controller_prefix.wasm` | Controller used by `s0`. Fetch with `stellar contract fetch --wasm-hash d8485ad34915d657a2bc5dd76c8a690277af6765d261f86bfc1088cacf9c6012 --network mainnet --out-file target/controller_prefix.wasm` |
| `SEED` | `7` | 0 to 255. Same seed, same cases |
| `SKIP` | none | Property IDs to ignore, for example `SKIP=P5,P3`, to see what fails next |
| `MAX_SECS` | none | Stop generating new work after this many seconds; the report says how many cases were skipped |

Every run prints the sha256 of each wasm it loaded and whether it equals a deployed hash.

## Limits

- The pool is a mock. It has no borrowers, utilisation limit, pool status, supply cap or emission accrual.
- Every call is authorized (`mock_all_auths_allowing_non_root_auth`). This tests accounting, not access
  control; `contracts/lending_yield_controller/src/test_e2e.rs` covers who may call what.
- The client types are generated at compile time from the wasm in `target/`. If a contract's interface
  changes, rebuild the wasm first, then the harness.
