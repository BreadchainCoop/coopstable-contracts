//! Property test of the cUSD backing invariant.
//!
//! All five CoopStable contracts are loaded as wasm at run time (WASM_DIR), so the
//! test runs on the deployed bytes. Only the Blend pool is a mock (native, below).
//!
//!   cargo run --release -- <s0|s1|s2|s3|s4|s4b|s5> [cases]      (or `make invariants` from the repo root)
//!   cargo run --release -- <dust|absorb>      scripted cases
//!   env: WASM_DIR, PREFIX_WASM, SEED, SKIP=P3,P5, MAX_SECS

use proptest::prelude::*;
use proptest::strategy::Union;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestError, TestRng, TestRunner,
};
use soroban_sdk::{
    testutils::{Address as _, EnvTestConfig, Ledger},
    token::{StellarAssetClient, TokenClient},
    xdr::ScErrorType,
    Address, Bytes, Env, InvokeError, Symbol,
};
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Instant;

// Clients only. The bytes that run are read from disk in `wasm()`.
mod controller {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/lending_yield_controller.wasm");
}
mod adapter {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/blend_capital_adapter.wasm");
}
mod manager {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/cusd_manager.wasm");
}
mod registry {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/yield_adapter_registry.wasm");
}
mod distributor {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/yield_distributor.wasm");
}
mod blend {
    soroban_sdk::contractimport!(file = "../packages/blend_capital_adapter/artifacts/pool.wasm");
}

const S12: i128 = 1_000_000_000_000;
const UNIT: i128 = 10_000_000;
const PERIOD: u64 = 86_400;
const RATE_CAP: i128 = 2 * S12;
const RATE_FLOOR: i128 = S12 / 10;
/// The controller treats a smaller yield as none (L-09).
const MIN_HARVEST: i128 = 1_000;
const HOST: u32 = u32::MAX - 1; // a failure that is not a contract error (budget, auth, trap)

/// Mock Blend pool. Rules copied from blend-contracts-v2 `pool/src/pool/actions.rs` and `user.rs`:
/// `from` and (if different) `spender` must authorize; negative amounts are rejected; supply mints
/// bTokens rounded down, withdraw burns rounded up; an oversized withdrawal is clamped to the
/// position; a mint or burn of zero bTokens is rejected. Not modelled: borrowers, utilisation,
/// pool status, supply caps, emission accrual.
mod pool {
    use super::blend::{Positions, Request, Reserve, ReserveConfig, ReserveData, UserEmissionData};
    use soroban_sdk::{
        contract, contracterror, contractimpl, panic_with_error, symbol_short, token::TokenClient, Address, Env, Map,
        Symbol, Vec,
    };

    const RES: Symbol = symbol_short!("RES");
    const POS: Symbol = symbol_short!("POS");
    const RATE: Symbol = symbol_short!("RATE");
    const BLND: Symbol = symbol_short!("BLND");
    const S12: i128 = 1_000_000_000_000;

    #[contracterror]
    #[derive(Copy, Clone, Debug, PartialEq)]
    #[repr(u32)]
    pub enum PoolError {
        NegativeAmount = 8,
        BadRequest = 1200,
        InvalidBTokenMint = 1216,
        InvalidBTokenBurn = 1217,
    }

    #[contract]
    pub struct MockPool;

    #[contractimpl]
    impl MockPool {
        pub fn __constructor(e: Env, reserves: Vec<Address>, blnd: Address) {
            e.storage().instance().set(&RES, &reserves);
            e.storage().instance().set(&BLND, &blnd);
        }

        pub fn submit(e: Env, from: Address, spender: Address, to: Address, requests: Vec<Request>) -> Positions {
            from.require_auth();
            if spender != from {
                spender.require_auth();
            }
            let mut positions = Self::get_positions(e.clone(), from.clone());
            let reserves = Self::get_reserve_list(e.clone());
            for r in requests.iter() {
                let Some(idx) = reserves.iter().position(|a| a == r.address) else {
                    panic_with_error!(&e, PoolError::BadRequest)
                };
                let idx = idx as u32;
                if r.amount < 0 {
                    panic_with_error!(&e, PoolError::NegativeAmount);
                }
                let rate = Self::get_rate(e.clone(), r.address.clone());
                let cur = positions.collateral.get(idx).unwrap_or(0);
                let token = TokenClient::new(&e, &r.address);
                match r.request_type {
                    2 => {
                        let mint = r.amount * S12 / rate;
                        if mint <= 0 {
                            panic_with_error!(&e, PoolError::InvalidBTokenMint);
                        }
                        token.transfer(&spender, &e.current_contract_address(), &r.amount);
                        positions.collateral.set(idx, cur + mint);
                    }
                    3 => {
                        let mut burn = (r.amount * S12 + rate - 1) / rate;
                        let mut out = r.amount;
                        if burn > cur {
                            burn = cur;
                            out = cur * rate / S12;
                        }
                        if burn <= 0 {
                            panic_with_error!(&e, PoolError::InvalidBTokenBurn);
                        }
                        positions.collateral.set(idx, cur - burn);
                        token.transfer(&e.current_contract_address(), &to, &out);
                    }
                    _ => panic_with_error!(&e, PoolError::BadRequest),
                }
            }
            e.storage().instance().set(&(POS, from), &positions);
            positions
        }

        pub fn get_positions(e: Env, user: Address) -> Positions {
            e.storage().instance().get(&(POS, user)).unwrap_or(Positions {
                liabilities: Map::new(&e),
                collateral: Map::new(&e),
                supply: Map::new(&e),
            })
        }

        pub fn get_reserve_list(e: Env) -> Vec<Address> {
            e.storage().instance().get(&RES).unwrap()
        }

        pub fn get_reserve(e: Env, asset: Address) -> Reserve {
            Reserve {
                asset: asset.clone(),
                config: ReserveConfig {
                    decimals: 7, c_factor: 0, l_factor: 0, util: 0, max_util: 0, r_base: 0, r_one: 0, r_two: 0,
                    r_three: 0, reactivity: 0, index: 0, supply_cap: i128::MAX, enabled: true,
                },
                data: ReserveData {
                    b_rate: Self::get_rate(e, asset), d_rate: S12, ir_mod: 1_0000000, b_supply: 0, d_supply: 0,
                    last_time: 0, backstop_credit: 0,
                },
                scalar: 1_0000000,
            }
        }

        pub fn claim(e: Env, from: Address, _ids: Vec<u32>, to: Address) -> i128 {
            from.require_auth();
            let blnd: Address = e.storage().instance().get(&BLND).unwrap();
            TokenClient::new(&e, &blnd).transfer(&e.current_contract_address(), &to, &100);
            100
        }

        pub fn get_user_emissions(_e: Env, _user: Address, _id: u32) -> Option<UserEmissionData> {
            Some(UserEmissionData { accrued: 100, index: 0 })
        }

        pub fn set_rate(e: Env, asset: Address, rate: i128) {
            e.storage().instance().set(&(RATE, asset), &rate);
        }

        pub fn get_rate(e: Env, asset: Address) -> i128 {
            e.storage().instance().get(&(RATE, asset)).unwrap_or(S12)
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Stage configuration
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Cancel {
    Off,
    RecompoundedOnly,
    Any,
}

#[derive(Clone, Debug)]
struct Cfg {
    name: &'static str,
    prefix_controller: bool,
    cancel: Cancel,
    loss: bool,
    harvest: bool,
    cap_withdraw: bool,
    exit_check: bool,
    cases: u32,
    skip: Vec<String>,
}

fn stage(name: &str) -> Cfg {
    let base = Cfg {
        name: "s1", prefix_controller: false, cancel: Cancel::Off, loss: false, harvest: true,
        cap_withdraw: false, exit_check: true, cases: 2000, skip: vec![],
    };
    match name {
        "s0" => Cfg { name: "s0", prefix_controller: true, ..base },
        "s1" => base,
        "s2" => Cfg { name: "s2", cancel: Cancel::RecompoundedOnly, ..base },
        "s3" => Cfg { name: "s3", cancel: Cancel::Any, ..base },
        "s4" => Cfg { name: "s4", cancel: Cancel::RecompoundedOnly, loss: true, exit_check: false, ..base },
        "s4b" => Cfg { name: "s4b", loss: true, harvest: false, cap_withdraw: true, exit_check: false, ..base },
        "s5" => Cfg { name: "s5", cases: 50_000, ..base },
        _ => panic!("unknown stage {name}"),
    }
}

// ---------------------------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Amt {
    /// Basis points of the actor's balance when the step runs.
    Bps(u16),
    /// 0 to 3 stroops.
    Dust(u8),
    Neg,
    /// Balance + 1.
    Over,
}

#[derive(Clone, Debug)]
enum Act {
    Deposit(u8, Amt),
    Withdraw(u8, Amt),
    Interest(u16),
    Time(u8),
    Cycle(bool),
    Harvest,
    Recompound,
    Finalize,
    DepositOther(u8),
    WithdrawOther(u8),
    Claim,
    AddMember(u8),
    RemoveMember(u8),
    Cancel,
    Loss(u16),
    /// A user tops up the backing; no cUSD is minted.
    Recollateralize(u8, Amt),
    /// (mint, redeem): the admin switches pauses on.
    Pause(bool, bool),
    /// (mint, redeem): the owner clears pauses.
    Unpause(bool, bool),
    /// Owner sweep of USDC held by the controller itself. Must never succeed: it holds none, or a harvest.
    Rescue(Amt),
    /// Owner withdrawal from the pool position: (of USDC?). USDC is backing, so it must be refused.
    RescueCollateral(bool),
}

fn amt() -> impl Strategy<Value = Amt> {
    prop_oneof![
        6 => (1u16..=10_000).prop_map(Amt::Bps),
        2 => (0u8..=3).prop_map(Amt::Dust),
        1 => Just(Amt::Neg),
        1 => Just(Amt::Over),
    ]
}

fn act(cfg: &Cfg) -> impl Strategy<Value = Act> {
    let mut v: Vec<(u32, BoxedStrategy<Act>)> = vec![
        (6, (0u8..3, amt()).prop_map(|(u, a)| Act::Deposit(u, a)).boxed()),
        (5, (0u8..7, amt()).prop_map(|(w, a)| Act::Withdraw(w, a)).boxed()),
        (4, (0u16..=500).prop_map(Act::Interest).boxed()),
        (3, (0u8..=4).prop_map(Act::Time).boxed()),
        (1, (0u8..3).prop_map(Act::DepositOther).boxed()),
        (1, (0u8..7).prop_map(Act::WithdrawOther).boxed()),
        (1, Just(Act::Claim).boxed()),
        (2, (0u8..3).prop_map(Act::AddMember).boxed()),
        (2, (0u8..3).prop_map(Act::RemoveMember).boxed()),
    ];
    // The pre-fix controller has none of these entry points.
    if !cfg.prefix_controller {
        v.push((2, (0u8..3, amt()).prop_map(|(u, a)| Act::Recollateralize(u, a)).boxed()));
        v.push((1, (any::<bool>(), any::<bool>()).prop_map(|(m, r)| Act::Pause(m, r)).boxed()));
        v.push((2, (any::<bool>(), any::<bool>()).prop_map(|(m, r)| Act::Unpause(m, r)).boxed()));
        v.push((1, amt().prop_map(Act::Rescue).boxed()));
        v.push((1, any::<bool>().prop_map(Act::RescueCollateral).boxed()));
    }
    if cfg.harvest {
        v.push((4, any::<bool>().prop_map(Act::Cycle).boxed()));
        v.push((2, Just(Act::Harvest).boxed()));
        v.push((2, Just(Act::Recompound).boxed()));
        v.push((2, Just(Act::Finalize).boxed()));
    }
    if cfg.cancel != Cancel::Off {
        // Extra lone harvests and recompounds so there is a pending harvest, in either state, to cancel often enough.
        v.push((4, Just(Act::Harvest).boxed()));
        v.push((4, Just(Act::Recompound).boxed()));
        v.push((6, Just(Act::Cancel).boxed()));
    }
    if cfg.loss {
        v.push((2, (0u16..=2000).prop_map(Act::Loss).boxed()));
    }
    Union::new_weighted(v)
}

// ---------------------------------------------------------------------------------------------
// Coverage and measurements (process-wide; printed at the end of a stage)
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
struct Stats {
    /// kind -> [generated, succeeded, failed as expected]
    cov: BTreeMap<&'static str, [u64; 3]>,
    /// (kind, error code) -> count
    errs: BTreeMap<(&'static str, u32), u64>,
    cases: u64,
    skipped_for_time: u64,
    max_deposit_shortfall: i128,
    max_withdraw_excess: i128,
    max_hole: i128,
    tau_at_max_hole: i128,
    blocked_recompound: u64,
    max_distributor_dust: i128,
    max_stranded_after_exit: i128,
    max_exit_shortfall: i128,
}

static STATS: Mutex<Option<Stats>> = Mutex::new(None);

fn stats<R>(f: impl FnOnce(&mut Stats) -> R) -> R {
    f(STATS.lock().unwrap().get_or_insert_with(Stats::default))
}

// ---------------------------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------------------------

// Holder indices into Snap.usdc / Snap.cusd
const TREASURY: usize = 6;
const DIST: usize = 7;
const CTL: usize = 8;
const POOL: usize = 9;
const ADP: usize = 10;
const MGR: usize = 11;

#[derive(Clone, PartialEq, Debug)]
struct Snap {
    /// Value of the controller's USDC collateral, as the adapter reports it.
    p: i128,
    /// cusd_total_supply
    s: i128,
    /// get_yield
    y: i128,
    btok: i128,
    rate: i128,
    usdc: Vec<i128>,
    cusd: Vec<i128>,
    /// (amount, is Harvested)
    pending: Option<(i128, bool)>,
    avail: bool,
    epoch: u64,
    active: [bool; 3],
    /// BLND emissions land in the treasury.
    blnd_treasury: i128,
    /// (mint paused, redeem paused)
    paused: (bool, bool),
}

impl Snap {
    fn c(&self) -> i128 {
        self.usdc[CTL]
    }
    /// USDC that backs cUSD: the position, plus yield in transit between harvest and recompound.
    fn backing(&self) -> i128 {
        self.p + match self.pending {
            Some((amt, true)) => amt,
            _ => 0,
        }
    }
    fn hole(&self) -> i128 {
        (self.s - self.backing()).max(0)
    }
}

struct W {
    env: Env,
    cfg: Cfg,
    ctl: controller::Client<'static>,
    adp: adapter::Client<'static>,
    mgr: manager::Client<'static>,
    dist: distributor::Client<'static>,
    pool: pool::MockPoolClient<'static>,
    usdc: TokenClient<'static>,
    usdc_admin: StellarAssetClient<'static>,
    other: Address,
    cusd: TokenClient<'static>,
    blnd: TokenClient<'static>,
    holders: Vec<Address>,
    usdc_minted: i128,
    ops: i128,
    rmax: i128,
    had_loss: bool,
    exiting: bool,
    ever_member: [bool; 3],
    snap: Snap,
    log: Vec<String>,
}

fn wasm_dir() -> String {
    std::env::var("WASM_DIR")
        .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../target/wasm32v1-none/release").to_string())
}

fn wasm(cfg: &Cfg, name: &str) -> std::vec::Vec<u8> {
    let path = if cfg.prefix_controller && name == "lending_yield_controller" {
        std::env::var("PREFIX_WASM")
            .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/target/controller_prefix.wasm").into())
    } else {
        format!("{}/{name}.wasm", wasm_dir())
    };
    std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

fn protocol(e: &Env) -> Symbol {
    Symbol::new(e, "BC_LA")
}

/// Ok(value) or Err(contract error code). Anything that is not a contract error becomes HOST.
fn oc<T, C, E: Into<soroban_sdk::Error>>(r: Result<Result<T, C>, Result<E, InvokeError>>) -> Result<T, u32> {
    match r {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(_)) => Err(u32::MAX),
        Err(Ok(e)) => {
            let e: soroban_sdk::Error = e.into();
            if e.is_type(ScErrorType::Contract) { Err(e.get_code()) } else { Err(HOST) }
        }
        Err(Err(InvokeError::Contract(c))) => Err(c),
        Err(Err(InvokeError::Abort)) => Err(HOST),
    }
}

impl W {
    fn new(cfg: &Cfg) -> W {
        let env = Env::new_with_config(EnvTestConfig { capture_snapshot_at_drop: false });
        // Timestamp moves, sequence number does not, so no entry reaches its TTL.
        env.ledger().set_sequence_number(100);
        env.ledger().set_timestamp(1_000_000_000);
        env.mock_all_auths_allowing_non_root_auth();

        let g = || Address::generate(&env);
        let (owner, admin, issuer, treasury) = (g(), g(), g(), g());
        let users = [g(), g(), g()];
        let members = [g(), g(), g()];

        let sac = |env: &Env| env.register_stellar_asset_contract_v2(issuer.clone()).address();
        let (usdc_id, other_id, cusd_id, blnd_id) = (sac(&env), sac(&env), sac(&env), sac(&env));

        let registry_id = env.register(wasm(cfg, "yield_adapter_registry").as_slice(), (admin.clone(), owner.clone()));
        let dist_id = env.register(
            wasm(cfg, "yield_distributor").as_slice(),
            (treasury.clone(), 1000u32, admin.clone(), PERIOD, owner.clone(), admin.clone()),
        );
        let mgr_id = env.register(wasm(cfg, "cusd_manager").as_slice(), (cusd_id.clone(), owner.clone(), admin.clone()));
        let ctl_id = env.register(
            wasm(cfg, "lending_yield_controller").as_slice(),
            (dist_id.clone(), registry_id.clone(), mgr_id.clone(), admin.clone(), owner.clone()),
        );
        let pool_id =
            env.register(pool::MockPool, (soroban_sdk::vec![&env, usdc_id.clone(), other_id.clone()], blnd_id.clone()));
        let adp_id = env.register(
            wasm(cfg, "blend_capital_adapter").as_slice(),
            (ctl_id.clone(), pool_id.clone(), blnd_id.clone(), owner.clone(), usdc_id.clone(), 0i128),
        );

        let reg = registry::Client::new(&env, &registry_id);
        let dist = distributor::Client::new(&env, &dist_id);
        let mgr = manager::Client::new(&env, &mgr_id);
        StellarAssetClient::new(&env, &cusd_id).set_admin(&mgr_id);
        mgr.set_yield_controller(&ctl_id);
        dist.set_yield_controller(&ctl_id);
        let lend = Symbol::new(&env, "LEND");
        reg.register_adapter(&lend, &protocol(&env), &adp_id);
        reg.add_support_for_asset(&lend, &protocol(&env), &usdc_id);

        let usdc_admin = StellarAssetClient::new(&env, &usdc_id);
        for u in &users {
            usdc_admin.mint(u, &(10_000 * UNIT));
            StellarAssetClient::new(&env, &other_id).mint(u, &(1_000 * UNIT));
        }
        StellarAssetClient::new(&env, &blnd_id).mint(&pool_id, &(1_000_000 * UNIT));

        let mut holders: Vec<Address> = users.to_vec();
        holders.extend(members.iter().cloned());
        holders.extend([treasury, dist_id.clone(), ctl_id.clone(), pool_id.clone(), adp_id.clone(), mgr_id.clone()]);

        let mut w = W {
            cfg: cfg.clone(),
            ctl: controller::Client::new(&env, &ctl_id),
            adp: adapter::Client::new(&env, &adp_id),
            mgr,
            dist,
            pool: pool::MockPoolClient::new(&env, &pool_id),
            usdc: TokenClient::new(&env, &usdc_id),
            usdc_admin,
            other: other_id,
            cusd: TokenClient::new(&env, &cusd_id),
            blnd: TokenClient::new(&env, &blnd_id),
            holders,
            usdc_minted: 30_000 * UNIT,
            ops: 0,
            rmax: S12,
            had_loss: false,
            exiting: false,
            ever_member: [false; 3],
            snap: Snap {
                p: 0, s: 0, y: 0, btok: 0, rate: S12, usdc: vec![], cusd: vec![], pending: None, avail: false,
                epoch: 0, active: [false; 3], blnd_treasury: 0, paused: (false, false),
            },
            log: vec![],
            env,
        };
        w.snap = w.read();
        w
    }

    fn read(&self) -> Snap {
        let ctl = &self.holders[CTL];
        let p = protocol(&self.env);
        let listed = self.dist.list_members();
        let mut active = [false; 3];
        for (i, a) in active.iter_mut().enumerate() {
            *a = listed.iter().any(|m| m == self.holders[3 + i]);
        }
        Snap {
            p: self.adp.get_balance(ctl, &self.usdc.address),
            s: self.mgr.cusd_total_supply(),
            y: self.ctl.get_yield(&p, &self.usdc.address),
            btok: self.pool.get_positions(ctl).collateral.get(0).unwrap_or(0),
            rate: self.pool.get_rate(&self.usdc.address),
            usdc: self.holders.iter().map(|h| self.usdc.balance(h)).collect(),
            cusd: self.holders.iter().map(|h| self.cusd.balance(h)).collect(),
            pending: self
                .ctl
                .get_pending_harvest(&p, &self.usdc.address)
                .map(|h| (h.amount, h.state == controller::HarvestState::Harvested)),
            avail: self.dist.is_distribution_available(),
            epoch: self.dist.get_current_epoch(),
            active,
            blnd_treasury: self.blnd.balance(&self.holders[TREASURY]),
            paused: if self.cfg.prefix_controller { (false, false) } else { self.ctl.get_paused() },
        }
    }

    /// Rounding one pool operation can cost, at the highest rate seen so far (finding L-07).
    fn eps(&self) -> i128 {
        (self.rmax + S12 - 1) / S12 + 1
    }
    /// Rounding the whole run can have cost so far.
    fn tau(&self) -> i128 {
        self.ops * self.eps()
    }

    fn fail(&self, id: &str, msg: String) -> Result<(), String> {
        if self.cfg.skip.iter().any(|s| s == id) {
            return Ok(());
        }
        Err(format!("{id}: {msg}"))
    }

    fn step(&mut self, a: &Act) -> Result<(), String> {
        self.step_as(a, None)
    }

    /// One action. `kind_override` relabels it in the coverage table (used by the exit check).
    fn step_as(&mut self, a: &Act, kind_override: Option<&'static str>) -> Result<(), String> {
        if let Act::Cycle(warp) = a {
            if *warp {
                self.step(&Act::Time(2))?;
            }
            self.step(&Act::Harvest)?;
            self.step(&Act::Recompound)?;
            return self.step(&Act::Finalize);
        }

        let pre = self.snap.clone();
        let proto = protocol(&self.env);
        let usdc = self.usdc.address.clone();
        let rate = pre.rate;
        // Predicted post-state, built from `pre`. `p` is filled in from btok and rate below.
        let mut want = pre.clone();
        let mut kind: &'static str;
        // Ok(()) or Err(expected contract error code)
        let exp: Result<(), u32>;
        let res: Result<i128, u32>;
        let mut step_ops = 0i128;
        let mut is_loss = false;
        // (requested, paid out) for withdrawals; requested for deposits
        let mut dep: Option<i128> = None;
        let mut wd: Option<(i128, i128)> = None;
        let mut desc = format!("{a:?}");

        let resolve = |amt: &Amt, bal: i128| match amt {
            Amt::Bps(b) => bal * *b as i128 / 10_000,
            Amt::Dust(d) => *d as i128,
            Amt::Neg => -1 - bal / 2,
            Amt::Over => bal + 1,
        };

        match a {
            Act::Deposit(u, amt) => {
                kind = "deposit";
                let u = *u as usize;
                let amount = resolve(amt, pre.usdc[u]);
                desc = format!("Deposit(user{u}, {amount})");
                let mint = if amount > 0 { amount * S12 / rate } else { 0 };
                exp = if pre.paused.0 {
                    Err(1006)
                } else if amount <= 0 {
                    Err(8)
                } else if mint == 0 {
                    Err(1216)
                } else if amount > pre.usdc[u] {
                    Err(10)
                } else {
                    want.usdc[u] -= amount;
                    want.usdc[POOL] += amount;
                    want.cusd[u] += amount;
                    want.s += amount;
                    want.btok += mint;
                    step_ops = 1;
                    dep = Some(amount);
                    Ok(())
                };
                res = oc(self.ctl.try_deposit_collateral(&proto, &self.holders[u], &usdc, &amount));
            }
            Act::Withdraw(w, amt) => {
                kind = "withdraw";
                let w = *w as usize;
                let mut amount = resolve(amt, pre.cusd[w]);
                // A redemption above the position reverts (M-04), so the last holders out ask for what is there.
                if (self.cfg.cap_withdraw || self.exiting) && amount > pre.p {
                    amount = pre.p;
                }
                desc = format!("Withdraw(holder{w}, {amount})");
                let (mut burn, mut out) = (if amount > 0 { (amount * S12 + rate - 1) / rate } else { 0 }, amount);
                if burn > pre.btok {
                    burn = pre.btok;
                    out = pre.btok * rate / S12;
                }
                exp = if pre.paused.1 {
                    Err(1007)
                } else if amount <= 0 {
                    Err(8)
                } else if burn == 0 {
                    Err(1217)
                } else if out != amount {
                    // M-04: the pool would clamp to the position and pay less, so the controller reverts.
                    Err(10)
                } else if amount > pre.cusd[w] {
                    Err(10)
                } else {
                    want.cusd[w] -= amount;
                    want.s -= amount;
                    want.usdc[w] += out;
                    want.usdc[POOL] -= out;
                    want.btok -= burn;
                    step_ops = 1;
                    wd = Some((amount, out));
                    Ok(())
                };
                res = oc(self.ctl.try_withdraw_collateral(&proto, &self.holders[w], &usdc, &amount));
            }
            Act::DepositOther(u) => {
                kind = "deposit-other";
                exp = Err(if pre.paused.0 { 1006 } else { 1000 });
                res = oc(self.ctl.try_deposit_collateral(&proto, &self.holders[*u as usize], &self.other, &UNIT));
            }
            Act::WithdrawOther(w) => {
                kind = "withdraw-other";
                exp = Err(if pre.paused.1 { 1007 } else { 1000 });
                res = oc(self.ctl.try_withdraw_collateral(&proto, &self.holders[*w as usize], &self.other, &UNIT));
            }
            Act::Interest(bps) => {
                kind = "interest";
                let new = (rate + rate * *bps as i128 / 10_000).min(RATE_CAP).max(rate);
                self.pool.set_rate(&usdc, &new);
                // Borrowers pay the interest, so the pool can always honour the position.
                let owed = pre.btok * new / S12 - pre.btok * rate / S12;
                if owed > 0 {
                    self.usdc_admin.mint(&self.holders[POOL], &owed);
                    self.usdc_minted += owed;
                    want.usdc[POOL] += owed;
                }
                self.rmax = self.rmax.max(new);
                want.rate = new;
                exp = Ok(());
                res = Ok(0);
            }
            Act::Loss(bps) => {
                kind = "loss";
                let new = (rate - rate * *bps as i128 / 10_000).max(RATE_FLOOR).min(rate);
                self.pool.set_rate(&usdc, &new);
                self.had_loss |= new < rate;
                is_loss = true;
                want.rate = new;
                exp = Ok(());
                res = Ok(0);
            }
            Act::Time(halves) => {
                kind = "time";
                let t = self.env.ledger().timestamp() + *halves as u64 * PERIOD / 2;
                self.env.ledger().set_timestamp(t);
                exp = Ok(());
                res = Ok(0);
            }
            Act::Harvest => {
                kind = "harvest";
                if pre.pending.is_some() {
                    exp = Err(1003);
                } else if pre.y >= MIN_HARVEST {
                    want.btok -= (pre.y * S12 + rate - 1) / rate;
                    want.usdc[POOL] -= pre.y;
                    want.usdc[CTL] += pre.y;
                    want.pending = Some((pre.y, true));
                    step_ops = 1;
                    exp = Ok(());
                } else if pre.avail {
                    kind = "harvest-zero-reset";
                    want.epoch += 1;
                    exp = Ok(());
                } else {
                    kind = "harvest-noop";
                    exp = Ok(());
                }
                res = oc(self.ctl.try_harvest_yield(&proto, &usdc));
            }
            Act::Recompound => {
                kind = "recompound";
                exp = match pre.pending {
                    None => Err(1002),
                    Some((_, false)) => Err(1004),
                    Some((amount, true)) => {
                        let mint = amount * S12 / rate;
                        if mint == 0 {
                            stats(|s| s.blocked_recompound += 1);
                            Err(1216)
                        } else if amount > pre.c() {
                            Err(10)
                        } else {
                            want.btok += mint;
                            want.usdc[CTL] -= amount;
                            want.usdc[POOL] += amount;
                            want.pending = Some((amount, false));
                            step_ops = 1;
                            Ok(())
                        }
                    }
                };
                res = oc(self.ctl.try_recompound_yield(&proto, &usdc));
            }
            Act::Finalize => {
                kind = "finalize";
                exp = if !pre.avail {
                    Err(1001)
                } else {
                    match pre.pending {
                        None => Err(1002),
                        Some((_, true)) => Err(1004),
                        // Only what is still surplus is minted: a loss after stage 1 is not paid out.
                        Some((amount, false)) if amount.min(pre.y) <= 0 => Err(1001),
                        Some((amount, false)) => {
                            let amount = amount.min(pre.y);
                            let n = pre.active.iter().filter(|x| **x).count() as i128;
                            let per = if n > 0 { (amount - amount * 1000 / 10_000) / n } else { 0 };
                            // The treasury takes its share plus the division remainder (L-02).
                            want.cusd[TREASURY] += amount - per * n;
                            for i in 0..3 {
                                if pre.active[i] {
                                    want.cusd[3 + i] += per;
                                }
                            }
                            want.s += amount;
                            want.pending = None;
                            want.epoch += 1;
                            Ok(())
                        }
                    }
                };
                res = oc(self.ctl.try_finalize_distribution(&proto, &usdc));
            }
            Act::Cancel => {
                kind = "cancel";
                if self.cfg.cancel == Cancel::RecompoundedOnly && matches!(pre.pending, Some((_, true))) {
                    // This stage only cancels after stage 2, so run stage 2 first.
                    self.step(&Act::Recompound)?;
                    if matches!(self.snap.pending, Some((_, true))) {
                        stats(|s| s.cov.entry("cancel-skipped").or_default()[0] += 1);
                        return Ok(());
                    }
                    return self.step_as(a, kind_override);
                }
                exp = match pre.pending {
                    None => Err(1002),
                    // M-02: before stage 2 the harvested yield goes back into the pool.
                    Some((amount, true)) => {
                        want.btok += amount * S12 / rate;
                        want.usdc[CTL] -= amount;
                        want.usdc[POOL] += amount;
                        want.pending = None;
                        step_ops = 1;
                        Ok(())
                    }
                    Some((_, false)) => {
                        want.pending = None;
                        Ok(())
                    }
                };
                res = oc(self.ctl.try_cancel_harvest(&proto, &usdc)).map(|_| 0);
            }
            Act::Claim => {
                kind = "claim";
                want.blnd_treasury += 100;
                exp = Ok(());
                res = oc(self.ctl.try_claim_emissions(&proto, &usdc));
            }
            Act::AddMember(i) => {
                kind = "add-member";
                let i = *i as usize;
                exp = if pre.active[i] {
                    Err(1200)
                } else {
                    want.active[i] = true;
                    Ok(())
                };
                res = oc(self.dist.try_add_member(&self.holders[3 + i])).map(|_| 0);
                if res.is_ok() {
                    self.ever_member[i] = true;
                }
            }
            Act::RemoveMember(i) => {
                kind = "remove-member";
                let i = *i as usize;
                exp = if !self.ever_member[i] {
                    Err(1201)
                } else {
                    want.active[i] = false;
                    Ok(())
                };
                res = oc(self.dist.try_remove_member(&self.holders[3 + i])).map(|_| 0);
            }
            Act::Recollateralize(u, amt) => {
                kind = "recollateralize";
                let u = *u as usize;
                let amount = resolve(amt, pre.usdc[u]);
                desc = format!("Recollateralize(user{u}, {amount})");
                let mint = if amount > 0 { amount * S12 / rate } else { 0 };
                // Not a mint, so the mint pause does not apply.
                exp = if amount <= 0 {
                    Err(8)
                } else if mint == 0 {
                    Err(1216)
                } else if amount > pre.usdc[u] {
                    Err(10)
                } else {
                    want.usdc[u] -= amount;
                    want.usdc[POOL] += amount;
                    want.btok += mint;
                    step_ops = 1;
                    Ok(())
                };
                res = oc(self.ctl.try_recollateralize(&proto, &usdc, &self.holders[u], &amount)).map(|_| 0);
            }
            Act::Pause(mint, redeem) => {
                kind = "pause";
                want.paused = (pre.paused.0 || *mint, pre.paused.1 || *redeem);
                exp = Ok(());
                res = oc(self.ctl.try_pause(mint, redeem)).map(|_| 0);
            }
            Act::Unpause(mint, redeem) => {
                kind = "unpause";
                want.paused = (pre.paused.0 && !*mint, pre.paused.1 && !*redeem);
                exp = Ok(());
                res = oc(self.ctl.try_unpause(mint, redeem)).map(|_| 0);
            }
            Act::Rescue(amt) => {
                kind = "rescue";
                let amount = resolve(amt, pre.c());
                desc = format!("Rescue(USDC, {amount})");
                // The controller holds USDC only as a stage-1 harvest, and that cannot be swept.
                exp = if amount <= 0 {
                    Err(8)
                } else if matches!(pre.pending, Some((_, true))) {
                    Err(1003)
                } else {
                    Err(10)
                };
                res = oc(self.ctl.try_rescue(&usdc, &self.holders[TREASURY], &amount)).map(|_| 0);
            }
            Act::RescueCollateral(of_usdc) => {
                kind = "rescue-collateral";
                // USDC is backing: refused. The other reserve has no position: the pool rejects a zero burn.
                let (asset, code) = if *of_usdc { (usdc.clone(), 1008) } else { (self.other.clone(), 1217) };
                exp = Err(code);
                res = oc(self.ctl.try_rescue_collateral(&proto, &asset, &self.holders[TREASURY], &UNIT)).map(|_| 0);
            }
            Act::Cycle(_) => unreachable!(),
        }
        if let Some(k) = kind_override {
            kind = k;
        }

        let post = self.read();
        self.snap = post.clone();
        self.log.push(format!(
            "{:<34} -> {:<12} P={} S={} C={} yield={} rate={} pending={:?} epoch={} due={}",
            desc,
            match res {
                Ok(v) => format!("ok {v}"),
                Err(c) => format!("err #{c}"),
            },
            post.p, post.s, post.c(), post.y, post.rate, post.pending, post.epoch, post.avail
        ));
        stats(|s| {
            let c = s.cov.entry(kind).or_default();
            c[0] += 1;
            match res {
                Ok(_) => c[1] += 1,
                Err(code) => {
                    c[2] += (exp == Err(code)) as u64;
                    *s.errs.entry((kind, code)).or_default() += 1;
                }
            }
        });

        // P0: the outcome and its effects were predicted from public state.
        if res.map(|_| ()) != exp {
            self.fail("P0", format!("{desc}: expected {exp:?}, got {res:?}"))?;
        }
        if res.is_err() {
            want = pre.clone();
        } else {
            want.p = want.btok * want.rate / S12;
            want.y = post.y;
            want.avail = post.avail;
            self.ops += step_ops;
        }
        if post != want {
            self.fail("P0", format!("{desc}: state differs from prediction\n   want {want:?}\n   got  {post:?}"))?;
        }

        let (eps, tau) = (self.eps(), self.tau());
        if res.is_ok() {
            // P3: par.
            if let Some(amount) = dep {
                let gained = post.p - pre.p;
                stats(|s| s.max_deposit_shortfall = s.max_deposit_shortfall.max(amount - gained));
                if gained > amount || gained < amount - eps {
                    self.fail("P3", format!("deposit of {amount} added {gained} to the position"))?;
                }
            }
            if let Some((amount, out)) = wd {
                stats(|s| s.max_withdraw_excess = s.max_withdraw_excess.max((pre.p - post.p) - out));
                if out > amount || out < amount - tau {
                    self.fail("P3", format!("withdrawal burned {amount} cUSD and paid {out} USDC (tolerance {tau})"))?;
                }
            }
        }
        // P4: supply adds up.
        let held: i128 = post.cusd.iter().sum();
        if post.s != held {
            self.fail("P4", format!("cusd_total_supply {} != sum of balances {held}", post.s))?;
        }
        // P6: the controller holds USDC only while a harvest is in state Harvested.
        let should_hold = match post.pending {
            Some((amount, true)) => amount,
            _ => 0,
        };
        if post.c() != should_hold {
            self.fail("P6", format!("controller holds {} USDC, pending harvest is {:?}", post.c(), post.pending))?;
        }
        // P1: no action except a pool loss deepens the hole beyond rounding.
        if !is_loss && post.hole() > pre.hole().max(tau) + eps * step_ops {
            self.fail("P1", format!("hole grew from {} to {} (rounding allowance {tau})", pre.hole(), post.hole()))?;
        }
        // P2: backed, when the pool never lost money.
        stats(|s| {
            if !self.had_loss && post.hole() > s.max_hole {
                s.max_hole = post.hole();
                s.tau_at_max_hole = tau;
            }
        });
        if !self.had_loss && post.hole() > tau {
            self.fail("P2", format!("supply {} exceeds backing {} by more than {tau}", post.s, post.backing()))?;
        }
        // P5: reported yield is real surplus.
        if post.y > (post.p + post.c() - post.s).max(0) + tau {
            self.fail(
                "P5",
                format!("get_yield={} but position {} + controller {} - supply {} leaves less", post.y, post.p, post.c(), post.s),
            )?;
        }
        // X1: USDC is conserved and the pool can pay the position.
        let total: i128 = post.usdc.iter().sum();
        if total != self.usdc_minted || post.usdc[ADP] != 0 || post.usdc[MGR] != 0 || post.usdc[DIST] != 0 {
            self.fail("X1", format!("USDC not conserved: {total} held, {} minted, balances {:?}", self.usdc_minted, post.usdc))?;
        }
        if !self.had_loss && post.usdc[POOL] < post.p {
            self.fail("X1", format!("mock pool holds {} USDC for a position worth {}", post.usdc[POOL], post.p))?;
        }
        Ok(())
    }

    /// P7: everyone can leave. Every holder redeems its whole balance, in an order set by `seed`.
    fn exit(&mut self, mut seed: u64) -> Result<(), String> {
        let mut order: Vec<u8> = (0..7).collect();
        for i in (1..order.len()).rev() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            order.swap(i, (seed >> 33) as usize % (i + 1));
        }
        self.log.push("-- exit: every holder redeems everything".into());
        if !self.cfg.prefix_controller {
            self.step(&Act::Unpause(true, true))?;
        }
        self.exiting = true;
        for w in order {
            if self.snap.cusd[w as usize] > 0 {
                self.step_as(&Act::Withdraw(w, Amt::Bps(10_000)), Some("exit-withdraw"))?;
            }
        }
        let left: i128 = self.snap.cusd[..7].iter().sum();
        stats(|s| {
            s.max_exit_shortfall = s.max_exit_shortfall.max(left);
            s.max_stranded_after_exit = s.max_stranded_after_exit.max(self.snap.p);
        });
        if left > self.tau() {
            self.fail("P7", format!("{left} cUSD could not be redeemed (rounding allowance {})", self.tau()))?;
        }
        Ok(())
    }
}

fn run_case(cfg: &Cfg, acts: &[Act], seed: u64) -> (Result<(), String>, Vec<String>) {
    let mut w = W::new(cfg);
    let mut r = Ok(());
    for a in acts {
        r = w.step(a);
        if r.is_err() {
            break;
        }
    }
    if r.is_ok() && cfg.exit_check {
        r = w.exit(seed);
    }
    stats(|s| {
        s.cases += 1;
        s.max_distributor_dust = s.max_distributor_dust.max(w.snap.cusd[DIST]);
    });
    (r, w.log)
}

// ---------------------------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------------------------

const MAINNET: [(&str, &str); 5] = [
    ("cusd_manager", "97a1072f714b529e4e63b20a58bc1a4335c5c6351c34f751209b21316a96e8cd"),
    ("yield_adapter_registry", "2e6c63c89c8ad6e2197808a04aae0b555af7df1c4b126fb04792be6a3a6bfc01"),
    ("yield_distributor", "d02d86d813fe9850294c36d39bbffa993179e59c695d68e461deba05c8b998d0"),
    ("lending_yield_controller", "4590302c494aff7adf94f89fc83ae3b171e19c2b53dc310341d5d64a4c1fd7d1"),
    ("blend_capital_adapter", "439dbc048bc3aca6bcf2678505aecd1f642efc2d22b89dfaf2c7b874d865f9f8"),
];
const PREFIX: &str = "d8485ad34915d657a2bc5dd76c8a690277af6765d261f86bfc1088cacf9c6012";

fn print_hashes(cfg: &Cfg) {
    let env = Env::new_with_config(EnvTestConfig { capture_snapshot_at_drop: false });
    println!("wasm under test:");
    for (name, mainnet) in MAINNET {
        let bytes = wasm(cfg, name);
        let hash: String =
            env.crypto().sha256(&Bytes::from_slice(&env, &bytes)).to_array().iter().map(|b| format!("{b:02x}")).collect();
        let label = if hash == mainnet {
            "= mainnet (mainnet.contracts.json)"
        } else if hash == PREFIX {
            "= pre-fix controller (mainnet until 2026-11-19)"
        } else {
            "not a deployed hash (a local build)"
        };
        println!("  {name:<26} {hash} {label}");
    }
}

/// Scripted cases the random generator almost never reaches, run through the same executor.
///   dust:   a harvest of one stroop. Blend rejects a supply that mints zero bTokens, so the
///           harvest cannot be recompounded.
///   absorb: interest that accrues between harvest and finalize becomes principal.
fn script(name: &str) {
    let cfg = Cfg { exit_check: false, ..stage("s3") };
    print_hashes(&cfg);
    let acts = match name {
        "dust" => {
            let mut a = vec![Act::Deposit(0, Amt::Dust(3)); 7];
            a.extend([Act::Interest(500), Act::Harvest, Act::Recompound, Act::Harvest, Act::Finalize, Act::Cancel]);
            a
        }
        _ => vec![
            Act::Deposit(0, Amt::Bps(1000)),
            Act::Interest(100),
            Act::Harvest,
            Act::Interest(100),
            Act::Recompound,
            Act::Finalize,
            Act::Time(2),
            Act::Harvest,
        ],
    };
    let (r, log) = run_case(&cfg, &acts, 0);
    for (i, l) in log.iter().enumerate() {
        println!("  {:>2} {l}", i + 1);
    }
    println!("  => {}", r.err().unwrap_or_else(|| "all properties held".into()));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(name @ ("dust" | "absorb")) = args.get(1).map(String::as_str) {
        return script(name);
    }
    let mut cfg = stage(args.get(1).map(String::as_str).unwrap_or("s1"));
    if let Some(n) = args.get(2) {
        cfg.cases = n.parse().expect("cases");
    }
    if let Ok(s) = std::env::var("SKIP") {
        cfg.skip = s.split(',').map(str::to_string).collect();
    }
    let seed: u8 = std::env::var("SEED").ok().map(|s| s.parse().expect("SEED 0-255")).unwrap_or(7);
    let max_secs: u64 = std::env::var("MAX_SECS").ok().map(|s| s.parse().expect("MAX_SECS")).unwrap_or(u64::MAX);

    println!("stage {} cases={} seed={seed} skip={:?}\n{cfg:?}", cfg.name, cfg.cases, cfg.skip);
    print_hashes(&cfg);

    let persist: &'static str = Box::leak(
        format!("{}/target/proptest-regressions/{}.txt", env!("CARGO_MANIFEST_DIR"), cfg.name).into_boxed_str(),
    );
    let config = Config {
        cases: cfg.cases,
        max_shrink_iters: 4000,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(persist))),
        ..Config::default()
    };
    let mut runner = TestRunner::new_with_rng(config, TestRng::from_seed(RngAlgorithm::ChaCha, &[seed; 32]));
    let strat = (proptest::collection::vec(act(&cfg), 1..=40), any::<u64>());
    let start = Instant::now();
    let result = runner.run(&strat, |(acts, exit_seed)| {
        if start.elapsed().as_secs() > max_secs {
            stats(|s| s.skipped_for_time += 1);
            return Ok(());
        }
        run_case(&cfg, &acts, exit_seed).0.map_err(TestCaseError::fail)
    });

    // Coverage is taken before the replay below so the replay does not count.
    let report = stats(|s| {
        let mut out = format!(
            "\ncases run: {} (skipped after the time cap: {}), {:.0}s\n\n{:<20} {:>9} {:>9} {:>12}\n",
            s.cases, s.skipped_for_time, start.elapsed().as_secs_f64(), "action", "generated", "succeeded", "failed as expected"
        );
        for (k, c) in &s.cov {
            out += &format!("{k:<20} {:>9} {:>9} {:>12}\n", c[0], c[1], c[2]);
        }
        out += "\nerror codes seen (action, code, count):\n";
        for ((k, code), n) in &s.errs {
            out += &format!("  {k:<20} #{code:<11} {n}\n");
        }
        out += &format!(
            "\nmeasured:\n  largest rounding loss on a deposit: {} stroops\n  largest rounding loss on a withdrawal: {} stroops\n  \
             largest supply - backing with no pool loss: {} stroops (allowance at that point {})\n  \
             recompound blocked because the harvest would mint zero bTokens: {} times\n  \
             largest cUSD left in the distributor: {} stroops\n  \
             largest cUSD not redeemable at exit: {} stroops\n  largest position left after everyone exits: {} stroops\n",
            s.max_deposit_shortfall, s.max_withdraw_excess, s.max_hole, s.tau_at_max_hole, s.blocked_recompound,
            s.max_distributor_dust, s.max_exit_shortfall, s.max_stranded_after_exit
        );
        // A pass only counts if every action was exercised.
        let mut need_ok = vec!["deposit", "withdraw", "interest", "time", "claim", "add-member", "remove-member"];
        let mut need_err = vec!["deposit-other", "withdraw-other"];
        if !cfg.prefix_controller {
            need_ok.extend(["recollateralize", "pause", "unpause"]);
            need_err.extend(["rescue", "rescue-collateral"]);
        }
        if cfg.harvest {
            need_ok.extend(["harvest", "harvest-zero-reset", "recompound", "finalize"]);
        }
        if cfg.cancel != Cancel::Off {
            need_ok.push("cancel");
        }
        if cfg.loss {
            need_ok.push("loss");
        }
        let mut short: Vec<String> = vec![];
        for k in need_ok {
            let n = s.cov.get(k).map(|c| c[1]).unwrap_or(0);
            if n < 200 {
                short.push(format!("{k} succeeded {n}"));
            }
        }
        for k in need_err {
            let n = s.cov.get(k).map(|c| c[2]).unwrap_or(0);
            if n < 200 {
                short.push(format!("{k} failed as expected {n}"));
            }
        }
        out += &if short.is_empty() {
            "coverage: every action reached its minimum of 200\n".to_string()
        } else {
            format!("coverage SHORT (minimum 200): {}\n", short.join(", "))
        };
        out
    });

    match result {
        Ok(()) => {
            println!("{report}\nRESULT {}: PASS", cfg.name);
        }
        Err(TestError::Fail(reason, (acts, exit_seed))) => {
            println!("{report}\nRESULT {}: FAIL\n{reason}\n\nshrunk case ({} actions, exit seed {exit_seed}):", cfg.name, acts.len());
            for a in &acts {
                println!("  {a:?}");
            }
            println!("\nstep log:");
            let (r, log) = run_case(&cfg, &acts, exit_seed);
            for (i, l) in log.iter().enumerate() {
                println!("  {:>2} {l}", i + 1);
            }
            println!("  => {}", r.err().unwrap_or_else(|| "did not fail on replay".into()));
            std::process::exit(1);
        }
        Err(TestError::Abort(reason)) => {
            println!("{report}\nRESULT {}: ABORTED {reason}", cfg.name);
            std::process::exit(2);
        }
    }
}
