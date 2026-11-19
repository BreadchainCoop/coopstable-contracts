#![cfg(test)]
//! End-to-end tests of the money paths: the real adapter (loaded as wasm), the real manager, registry and
//! distributor, and a Blend pool mock that moves tokens and enforces auth. Amounts are in stroops.
extern crate std;

use soroban_sdk::{
    contracttype, symbol_short,
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env, IntoVal, Symbol, Val, Vec,
};

use crate::contract::{LendingYieldController, LendingYieldControllerClient};
use blend_capital_adapter::artifacts::pool::Request;
use cusd_manager::contract::{CUSDManager, CUSDManagerClient};
use yield_adapter::contract_types::{SupportedAdapter, SupportedYieldType};
use yield_adapter_registry::contract::{YieldAdapterRegistry, YieldAdapterRegistryClient};
use yield_distributor::contract::{YieldDistributor, YieldDistributorClient};

mod adapter_wasm {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/blend_capital_adapter.wasm");
}
use adapter_wasm::Client as AdapterClient;

const SCALAR_12: i128 = 1_000_000_000_000;
const UNIT: i128 = 1_0000000;
const DAY: u64 = 86_400;
const T0: u64 = 1_000_000_000;
/// Blend mints bTokens rounded down and burns them rounded up, so each pool operation can leave the position
/// a stroop short of the books (L-07). The next yield absorbs it.
const DUST: i128 = 4;

mod pool {
    use blend_capital_adapter::artifacts::pool::{
        Positions, Request, Reserve, ReserveConfig, ReserveData, UserEmissionData,
    };
    use soroban_sdk::{
        contract, contracterror, contractimpl, panic_with_error, symbol_short, token::TokenClient, Address, Env,
        Map, Symbol, Vec,
    };

    const RES: Symbol = symbol_short!("RES");
    const POS: Symbol = symbol_short!("POS");
    const RATE: Symbol = symbol_short!("RATE");
    const BLND: Symbol = symbol_short!("BLND");
    const SHORT: Symbol = symbol_short!("SHORT");
    const S12: i128 = 1_000_000_000_000;

    #[contracterror]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum PoolError {
        /// Blend's rejection of a supply that mints zero bTokens.
        InvalidBTokenMintAmount = 1216,
    }

    /// Mock Blend pool, written from the Blend v2 source: moves real tokens, requires `from` and `spender`
    /// auth, rounds bTokens against the user, clamps an over-withdrawal to the position.
    #[contract]
    pub struct MockPool;

    #[contractimpl]
    impl MockPool {
        pub fn init(e: Env, reserves: Vec<Address>, blnd: Address) {
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
                let idx = reserves.iter().position(|a| a == r.address).unwrap() as u32;
                let rate = Self::rate(&e, &r.address);
                let cur = positions.collateral.get(idx).unwrap_or(0);
                let token = TokenClient::new(&e, &r.address);
                match r.request_type {
                    2 => {
                        let minted = r.amount * S12 / rate;
                        if minted <= 0 {
                            panic_with_error!(&e, PoolError::InvalidBTokenMintAmount);
                        }
                        // SHORT: a pool that takes one stroop less than asked, to test the caller's check.
                        let short: bool = e.storage().instance().get(&SHORT).unwrap_or(false);
                        token.transfer(&spender, &e.current_contract_address(), &(r.amount - short as i128));
                        positions.collateral.set(idx, cur + minted);
                    }
                    3 => {
                        let mut burn = (r.amount * S12 + rate - 1) / rate;
                        let mut out = r.amount;
                        if burn > cur {
                            burn = cur;
                            out = cur * rate / S12;
                        }
                        positions.collateral.set(idx, cur - burn);
                        token.transfer(&e.current_contract_address(), &to, &out);
                    }
                    _ => panic!("unsupported"),
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
                    decimals: 7, c_factor: 0, l_factor: 0, util: 0, max_util: 0, r_base: 0, r_one: 0,
                    r_two: 0, r_three: 0, reactivity: 0, index: 0, supply_cap: i128::MAX, enabled: true,
                },
                data: ReserveData {
                    b_rate: Self::rate(&e, &asset), d_rate: S12, ir_mod: 1_0000000, b_supply: 0,
                    d_supply: 0, last_time: 0, backstop_credit: 0,
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

        /// Test helper: interest (rate up) or socialised bad debt (rate down).
        pub fn set_rate(e: Env, asset: Address, rate: i128) {
            e.storage().instance().set(&(RATE, asset), &rate);
        }

        /// Test helper: make every supply pull one stroop less than requested.
        pub fn set_short(e: Env, short: bool) {
            e.storage().instance().set(&SHORT, &short);
        }

        /// Test helper: give `user` a collateral position without moving tokens.
        pub fn set_collateral(e: Env, user: Address, asset: Address, b_tokens: i128) {
            let mut positions = Self::get_positions(e.clone(), user.clone());
            let idx = Self::get_reserve_list(e.clone()).iter().position(|a| a == asset).unwrap() as u32;
            positions.collateral.set(idx, b_tokens);
            e.storage().instance().set(&(POS, user), &positions);
        }

        fn rate(e: &Env, asset: &Address) -> i128 {
            e.storage().instance().get(&(RATE, asset.clone())).unwrap_or(S12)
        }
    }
}

/// Mirror of the adapter's epoch record, to read and seed it in the adapter's storage.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
struct AssetEpochPrincipal {
    epoch: u64,
    principal: i128,
    withdrawals: i128,
    deposits_in_epoch: i128,
    last_updated: u64,
}

/// Mirror of the manager's supply key, to seed the counter.
#[contracttype]
enum ManagerKey {
    CusdSupply,
}

struct Fx {
    env: Env,
    controller: LendingYieldControllerClient<'static>,
    registry: YieldAdapterRegistryClient<'static>,
    distributor: YieldDistributorClient<'static>,
    manager: CUSDManagerClient<'static>,
    adapter: AdapterClient<'static>,
    pool: pool::MockPoolClient<'static>,
    usdc: TokenClient<'static>,
    xlm: TokenClient<'static>,
    cusd: TokenClient<'static>,
    blnd: TokenClient<'static>,
    owner: Address,
    admin: Address,
    user: Address,
    treasury: Address,
}

fn lend() -> Symbol {
    SupportedYieldType::Lending.id()
}

fn protocol() -> Symbol {
    SupportedAdapter::BlendCapital.id()
}

fn setup() -> Fx {
    let env = Env::default();
    env.ledger().set_sequence_number(100);
    env.ledger().set_timestamp(T0);
    env.mock_all_auths_allowing_non_root_auth();

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let treasury = Address::generate(&env);
    let issuer = Address::generate(&env);

    let usdc_id = env.register_stellar_asset_contract_v2(issuer.clone()).address();
    let xlm_id = env.register_stellar_asset_contract_v2(issuer.clone()).address();
    let cusd_id = env.register_stellar_asset_contract_v2(issuer.clone()).address();
    let blnd_id = env.register_stellar_asset_contract_v2(issuer.clone()).address();

    let registry_id = env.register(YieldAdapterRegistry, (admin.clone(), owner.clone()));
    let distributor_id = env.register(
        YieldDistributor,
        (treasury.clone(), 1000u32, admin.clone(), DAY, owner.clone(), admin.clone()),
    );
    let manager_id = env.register(CUSDManager, (cusd_id.clone(), owner.clone(), admin.clone()));
    let controller_id = env.register(
        LendingYieldController,
        (distributor_id.clone(), registry_id.clone(), manager_id.clone(), admin.clone(), owner.clone()),
    );
    let pool_id = env.register(pool::MockPool, ());
    let pool = pool::MockPoolClient::new(&env, &pool_id);
    pool.init(&vec![&env, usdc_id.clone(), xlm_id.clone()], &blnd_id);
    let adapter_id = env.register(
        adapter_wasm::WASM,
        (controller_id.clone(), pool_id.clone(), blnd_id.clone(), owner.clone(), usdc_id.clone(), 0i128),
    );

    let registry = YieldAdapterRegistryClient::new(&env, &registry_id);
    let distributor = YieldDistributorClient::new(&env, &distributor_id);
    let manager = CUSDManagerClient::new(&env, &manager_id);

    StellarAssetClient::new(&env, &cusd_id).set_admin(&manager_id);
    manager.set_yield_controller(&controller_id);
    distributor.set_yield_controller(&controller_id);
    registry.register_adapter(&lend(), &protocol(), &adapter_id);
    registry.add_support_for_asset(&lend(), &protocol(), &usdc_id);

    StellarAssetClient::new(&env, &usdc_id).mint(&user, &(10_000 * UNIT));
    // Liquidity so the pool can honour a higher b_rate, and the BLND it pays as emissions.
    StellarAssetClient::new(&env, &usdc_id).mint(&pool_id, &(10_000 * UNIT));
    StellarAssetClient::new(&env, &xlm_id).mint(&pool_id, &(10_000 * UNIT));
    StellarAssetClient::new(&env, &blnd_id).mint(&pool_id, &(10_000 * UNIT));

    Fx {
        controller: LendingYieldControllerClient::new(&env, &controller_id),
        registry,
        distributor,
        manager,
        adapter: AdapterClient::new(&env, &adapter_id),
        pool,
        usdc: TokenClient::new(&env, &usdc_id),
        xlm: TokenClient::new(&env, &xlm_id),
        cusd: TokenClient::new(&env, &cusd_id),
        blnd: TokenClient::new(&env, &blnd_id),
        owner,
        admin,
        user,
        treasury,
        env,
    }
}

impl Fx {
    fn deposit(&self, amount: i128) {
        self.controller.deposit_collateral(&protocol(), &self.user, &self.usdc.address, &amount);
    }
    fn position(&self) -> i128 {
        self.adapter.get_balance(&self.controller.address, &self.usdc.address)
    }
    fn supply(&self) -> i128 {
        self.manager.cusd_total_supply()
    }
    fn yield_(&self) -> i128 {
        self.controller.get_yield(&protocol(), &self.usdc.address)
    }
    fn set_rate_pct(&self, pct_x100: i128) {
        self.pool.set_rate(&self.usdc.address, &(SCALAR_12 * pct_x100 / 10_000));
    }
    fn harvest(&self) -> i128 {
        self.controller.harvest_yield(&protocol(), &self.usdc.address)
    }
    fn recompound(&self) -> i128 {
        self.controller.recompound_yield(&protocol(), &self.usdc.address)
    }
    fn finalize(&self) -> i128 {
        self.controller.finalize_distribution(&protocol(), &self.usdc.address)
    }
    fn record(&self, asset: &Address) -> AssetEpochPrincipal {
        self.env.as_contract(&self.adapter.address, || {
            self.env.storage().instance().get(&(symbol_short!("AEP"), asset.clone())).unwrap()
        })
    }
    fn seed_record(&self, asset: &Address, principal: i128, deposits: i128, withdrawals: i128) {
        self.env.as_contract(&self.adapter.address, || {
            self.env.storage().instance().set(
                &(symbol_short!("AEP"), asset.clone()),
                &AssetEpochPrincipal { epoch: 4, principal, withdrawals, deposits_in_epoch: deposits, last_updated: T0 },
            )
        });
    }
    /// Only `who` signs, and only the given call tree.
    fn sign(&self, who: &Address, fn_name: &str, args: Vec<Val>, sub_invokes: &[MockAuthInvoke]) {
        self.env.mock_auths(&[MockAuth {
            address: who,
            invoke: &MockAuthInvoke { contract: &self.controller.address, fn_name, args, sub_invokes },
        }]);
    }
    /// The backing invariant, with the rounding tolerance of L-07.
    fn assert_backed(&self) {
        assert!(self.position() + DUST >= self.supply(), "position {} < supply {}", self.position(), self.supply());
    }
}

/// Asserts a `try_` call failed with the given contract error code.
macro_rules! assert_code {
    ($call:expr, $code:expr) => {
        assert_eq!($call.err().expect("call succeeded"), Ok(soroban_sdk::Error::from_contract_error($code)))
    };
}

// ---------------------------------------------------------------------------------------------------------
// Deposit and withdrawal
// ---------------------------------------------------------------------------------------------------------

#[test]
fn deposit_and_withdraw_move_exact_amounts() {
    let f = setup();
    f.deposit(1000 * UNIT);
    assert_eq!(f.cusd.balance(&f.user), 1000 * UNIT);
    assert_eq!(f.usdc.balance(&f.user), 9_000 * UNIT);
    assert_eq!(f.position(), 1000 * UNIT);
    assert_eq!(f.supply(), 1000 * UNIT);
    assert_eq!(f.adapter.get_total_deposited(&f.usdc.address), 1000 * UNIT);

    f.controller.withdraw_collateral(&protocol(), &f.user, &f.usdc.address, &(400 * UNIT));
    assert_eq!(f.cusd.balance(&f.user), 600 * UNIT);
    assert_eq!(f.usdc.balance(&f.user), 9_400 * UNIT);
    assert_eq!(f.position(), 600 * UNIT);
    assert_eq!(f.supply(), 600 * UNIT);
    assert_eq!(f.usdc.balance(&f.controller.address), 0);
    f.assert_backed();
}

/// M-04: a redemption larger than the position must fail, not burn the full amount for a smaller payout.
#[test]
fn withdraw_more_than_the_position_reverts() {
    let f = setup();
    f.deposit(1000 * UNIT);
    f.set_rate_pct(9_000); // position now worth 900

    assert_code!(f.controller.try_withdraw_collateral(&protocol(), &f.user, &f.usdc.address, &(1000 * UNIT)), 10);
    assert_eq!(f.cusd.balance(&f.user), 1000 * UNIT);
    assert_eq!(f.supply(), 1000 * UNIT);

    // What the position can pay still redeems.
    f.controller.withdraw_collateral(&protocol(), &f.user, &f.usdc.address, &(900 * UNIT));
    assert_eq!(f.cusd.balance(&f.user), 100 * UNIT);
    assert_eq!(f.usdc.balance(&f.user), 9_900 * UNIT);
}

/// M-04: a deposit must cost the user exactly `amount`, whatever the pool and adapter report.
#[test]
fn deposit_reverts_when_the_pool_takes_a_different_amount() {
    let f = setup();
    f.pool.set_short(&true);
    assert_code!(f.controller.try_deposit_collateral(&protocol(), &f.user, &f.usdc.address, &(1000 * UNIT)), 10);
    assert_eq!(f.supply(), 0);
    assert_eq!(f.usdc.balance(&f.user), 10_000 * UNIT);
}

/// I-02: the controller only pre-authorizes nested calls. With nobody but the user signing, and only the
/// user's own call tree, a deposit and a withdrawal still go through.
#[test]
fn deposit_and_withdraw_with_only_the_user_signing() {
    let f = setup();
    let amount = 1000 * UNIT;
    let args: Vec<Val> = (protocol(), f.user.clone(), f.usdc.address.clone(), amount).into_val(&f.env);

    let requests: Vec<Request> =
        vec![&f.env, Request { request_type: 2, address: f.usdc.address.clone(), amount }];
    f.sign(&f.user, "deposit_collateral", args.clone(), &[MockAuthInvoke {
        contract: &f.pool.address,
        fn_name: "submit",
        args: (f.controller.address.clone(), f.user.clone(), f.controller.address.clone(), requests).into_val(&f.env),
        sub_invokes: &[MockAuthInvoke {
            contract: &f.usdc.address,
            fn_name: "transfer",
            args: (f.user.clone(), f.pool.address.clone(), amount).into_val(&f.env),
            sub_invokes: &[],
        }],
    }]);
    f.deposit(amount);
    assert_eq!(f.cusd.balance(&f.user), amount);
    assert_eq!(f.position(), amount);

    f.sign(&f.user, "withdraw_collateral", args, &[MockAuthInvoke {
        contract: &f.cusd.address,
        fn_name: "burn",
        args: (f.user.clone(), amount).into_val(&f.env),
        sub_invokes: &[],
    }]);
    f.controller.withdraw_collateral(&protocol(), &f.user, &f.usdc.address, &amount);
    assert_eq!(f.cusd.balance(&f.user), 0);
    assert_eq!(f.usdc.balance(&f.user), 10_000 * UNIT);
    assert_eq!(f.supply(), 0);
}

/// I-05: interest left behind after every holder redeems is still yield.
#[test]
fn yield_is_claimable_after_a_full_exit() {
    let f = setup();
    f.deposit(1000 * UNIT);
    f.set_rate_pct(10_500);
    f.controller.withdraw_collateral(&protocol(), &f.user, &f.usdc.address, &(1000 * UNIT));
    assert_eq!(f.supply(), 0);
    assert!(f.yield_() >= 50 * UNIT - DUST, "yield reads {}", f.yield_());
}

// ---------------------------------------------------------------------------------------------------------
// Harvest cycle
// ---------------------------------------------------------------------------------------------------------

/// Full cycle with seven members: balances, supply, adapter record, and the division remainder (L-02).
#[test]
fn harvest_cycle_distributes_yield_and_keeps_backing() {
    let f = setup();
    let members: std::vec::Vec<Address> = (0..7).map(|_| Address::generate(&f.env)).collect();
    for m in members.iter() {
        f.distributor.add_member(m);
    }
    f.deposit(1000 * UNIT);
    f.set_rate_pct(10_500);
    let y = f.yield_();
    assert_eq!(y, 50 * UNIT);

    assert_eq!(f.harvest(), y);
    assert_eq!(f.usdc.balance(&f.controller.address), y);
    assert_eq!(f.recompound(), y);
    assert_eq!(f.usdc.balance(&f.controller.address), 0);
    // Stage 3 mints what is still surplus: the harvest less the pool's rounding on the way out and back.
    let y = f.finalize();
    assert!(50 * UNIT - y <= DUST, "minted {y}");

    assert_eq!(f.supply(), 1000 * UNIT + y);
    assert_eq!(f.controller.get_pending_harvest(&protocol(), &f.usdc.address).is_none(), true);
    assert!(f.position() >= f.supply());
    assert_eq!(f.yield_(), 0);

    // 10% to the treasury, the rest split seven ways, the remainder to the treasury.
    let per_member = (y - y / 10) / 7;
    assert!((y - y / 10) % 7 != 0);
    for m in members.iter() {
        assert_eq!(f.cusd.balance(m), per_member);
    }
    assert_eq!(f.cusd.balance(&f.treasury), y - 7 * per_member);
    assert_eq!(f.cusd.balance(&f.distributor.address), 0);
    let last = f.distributor.get_distribution_history().last().unwrap();
    assert_eq!(last.distribution_member, 7 * per_member);
    assert_eq!(last.distribution_treasury, y - 7 * per_member);

    // The record now owes principal plus the minted yield, at the distributor's epoch (I-01).
    let record = f.record(&f.usdc.address);
    assert_eq!(record.principal, 1000 * UNIT + y);
    assert_eq!((record.deposits_in_epoch, record.withdrawals), (0, 0));
    assert_eq!(record.epoch, f.distributor.get_current_epoch());
}

/// I-02: the three stages with only the admin signing each root call.
#[test]
fn harvest_cycle_with_only_the_admin_signing() {
    let f = setup();
    f.deposit(1000 * UNIT);
    f.set_rate_pct(10_500);
    let args: Vec<Val> = (protocol(), f.usdc.address.clone()).into_val(&f.env);

    f.sign(&f.admin, "harvest_yield", args.clone(), &[]);
    assert_eq!(f.harvest(), 50 * UNIT);
    f.sign(&f.admin, "recompound_yield", args.clone(), &[]);
    f.recompound();
    f.sign(&f.admin, "finalize_distribution", args, &[]);
    let y = f.finalize();

    assert!(50 * UNIT - y <= DUST);
    assert_eq!(f.cusd.balance(&f.treasury), y);
    assert_eq!(f.supply(), 1000 * UNIT + y);
    f.assert_backed();
}

/// M-02: cancelling after stage 1 puts the harvested yield back; it can never be withdrawn twice.
#[test]
fn cancel_after_harvest_returns_the_yield_to_the_pool() {
    let f = setup();
    f.deposit(1000 * UNIT);
    f.set_rate_pct(10_500);

    for _ in 0..5 {
        f.harvest();
        f.controller.cancel_harvest(&protocol(), &f.usdc.address);
        assert_eq!(f.usdc.balance(&f.controller.address), 0);
        f.assert_backed();
    }
    assert!(f.position() + 5 * DUST >= 1050 * UNIT, "position {}", f.position());
    assert_eq!(f.supply(), 1000 * UNIT);

    // The yield is still there to harvest properly.
    let y = f.harvest();
    assert!(y + 5 * DUST >= 50 * UNIT);
    f.recompound();
    f.finalize();
    f.assert_backed();
}

/// Cancelling after stage 2 only drops the record.
#[test]
fn cancel_after_recompound_keeps_the_yield() {
    let f = setup();
    f.deposit(1000 * UNIT);
    f.set_rate_pct(10_500);
    f.harvest();
    f.recompound();
    f.controller.cancel_harvest(&protocol(), &f.usdc.address);
    assert_eq!(f.usdc.balance(&f.controller.address), 0);
    assert_eq!(f.supply(), 1000 * UNIT);
    assert!(f.yield_() + DUST >= 50 * UNIT);
}

/// M-03: a loss is not written off at the epoch reset. No yield is paid until the position is worth more
/// than what is owed.
#[test]
fn a_loss_is_not_written_off_by_a_zero_yield_epoch() {
    let f = setup();
    f.deposit(1000 * UNIT);

    f.set_rate_pct(9_000);
    f.env.ledger().set_timestamp(T0 + DAY + 1);
    assert_eq!(f.harvest(), 0); // zero-yield path, epoch advances
    assert_eq!(f.distributor.get_current_epoch(), 1);
    assert_eq!(f.record(&f.usdc.address).principal, 1000 * UNIT);

    f.set_rate_pct(9_900); // still under water
    f.env.ledger().set_timestamp(T0 + 2 * DAY + 2);
    assert_eq!(f.yield_(), 0);
    assert_eq!(f.harvest(), 0);
    assert_eq!(f.supply(), 1000 * UNIT);
    assert_eq!(f.cusd.balance(&f.treasury), 0);

    f.set_rate_pct(10_100); // above water: only the surplus is yield
    assert_eq!(f.yield_(), 10 * UNIT);
}

/// A loss between stage 1 and stage 3 is not paid out: stage 3 mints only what is still surplus, and
/// refuses when nothing is.
#[test]
fn finalize_mints_only_what_is_still_surplus() {
    let f = setup();
    f.deposit(1000 * UNIT);
    f.set_rate_pct(10_500);
    assert_eq!(f.harvest(), 50 * UNIT);
    f.set_rate_pct(10_000); // the pool loses most of the interest again
    f.recompound();
    let surplus = f.yield_();
    assert!(surplus > 0 && surplus < 3 * UNIT, "surplus {surplus}");
    assert_eq!(f.finalize(), surplus);
    assert_eq!(f.supply(), 1000 * UNIT + surplus);
    assert_eq!(f.cusd.balance(&f.treasury), surplus);
    assert!(f.position() >= f.supply());

    f.set_rate_pct(10_500);
    f.env.ledger().set_timestamp(T0 + DAY + 1);
    f.harvest();
    f.set_rate_pct(9_000); // under water before stage 3
    f.recompound();
    assert_code!(f.controller.try_finalize_distribution(&protocol(), &f.usdc.address), 1001);
    f.controller.cancel_harvest(&protocol(), &f.usdc.address);
    assert_eq!(f.supply(), 1000 * UNIT + surplus);
}

/// The stages only run in order, and stage 3 only when a distribution is due.
#[test]
fn harvest_stages_enforce_their_order() {
    let f = setup();
    f.deposit(1000 * UNIT);
    f.set_rate_pct(10_500);
    let (p, a) = (protocol(), f.usdc.address.clone());

    assert_code!(f.controller.try_recompound_yield(&p, &a), 1002);
    f.harvest();
    assert_code!(f.controller.try_harvest_yield(&p, &a), 1003);
    assert_code!(f.controller.try_finalize_distribution(&p, &a), 1004);
    f.recompound();
    assert_code!(f.controller.try_recompound_yield(&p, &a), 1004);
    f.finalize();

    f.set_rate_pct(11_000);
    f.harvest();
    f.recompound();
    assert_code!(f.controller.try_finalize_distribution(&p, &a), 1001); // next distribution is a day away
    f.env.ledger().set_timestamp(T0 + DAY);
    f.finalize();
}

/// I-06: interest earned between stage 1 and stage 3 stays yield for the next epoch.
#[test]
fn interest_between_harvest_and_finalize_stays_yield() {
    let f = setup();
    f.deposit(1000 * UNIT);
    f.set_rate_pct(10_100);
    f.harvest();
    f.set_rate_pct(10_201);
    f.recompound();
    f.finalize();
    assert_eq!(f.supply(), 1010 * UNIT);
    assert!(f.yield_() > 9 * UNIT, "yield reads {}", f.yield_());
}

/// L-09: a yield too small to mint one bToken is not harvested, so the cycle cannot get stuck.
#[test]
fn dust_yield_takes_the_no_yield_path() {
    let f = setup();
    f.deposit(1000 * UNIT);
    f.pool.set_rate(&f.usdc.address, &(SCALAR_12 + 500)); // 5 stroops of interest
    assert_eq!(f.yield_(), 5);
    assert_eq!(f.harvest(), 0);
    assert!(f.controller.get_pending_harvest(&protocol(), &f.usdc.address).is_none());
    assert_eq!(f.usdc.balance(&f.controller.address), 0);
    assert_eq!(f.record(&f.usdc.address).principal, 1000 * UNIT); // epoch 0 was due: reset, nothing lost
    assert_eq!(f.yield_(), 5);
}

/// M-05: the harvest stages reject an asset the registry does not support.
#[test]
fn harvest_stages_reject_an_unsupported_asset() {
    let f = setup();
    f.deposit(1000 * UNIT);
    f.pool.set_collateral(&f.controller.address, &f.xlm.address, &(1000 * UNIT));
    f.seed_record(&f.xlm.address, 0, 1000 * UNIT, 0);
    f.pool.set_rate(&f.xlm.address, &(SCALAR_12 * 105 / 100));

    assert_code!(f.controller.try_harvest_yield(&protocol(), &f.xlm.address), 1000);
    assert_code!(f.controller.try_recompound_yield(&protocol(), &f.xlm.address), 1000);
    assert_code!(f.controller.try_finalize_distribution(&protocol(), &f.xlm.address), 1000);
    assert_code!(f.controller.try_claim_emissions(&protocol(), &f.xlm.address), 1000);
    assert_eq!(f.supply(), 1000 * UNIT);
}

/// L-01: emissions need the admin, go to the treasury, and the amount is returned.
#[test]
fn claim_emissions_needs_the_admin_and_pays_the_treasury() {
    let f = setup();
    f.deposit(1000 * UNIT);

    f.env.mock_auths(&[]); // nobody signs
    assert!(f.controller.try_claim_emissions(&protocol(), &f.usdc.address).is_err());

    f.sign(&f.admin, "claim_emissions", (protocol(), f.usdc.address.clone()).into_val(&f.env), &[]);
    assert_eq!(f.controller.claim_emissions(&protocol(), &f.usdc.address), 100);
    assert_eq!(f.blnd.balance(&f.treasury), 100);
    assert_eq!(f.blnd.balance(&f.distributor.address), 0);
}

// ---------------------------------------------------------------------------------------------------------
// Recollateralize, rescue, pause
// ---------------------------------------------------------------------------------------------------------

/// A top-up raises the position and the amount owed by the same amount, mints nothing, and is not yield.
#[test]
fn recollateralize_tops_up_backing_without_minting() {
    let f = setup();
    let funder = Address::generate(&f.env);
    StellarAssetClient::new(&f.env, &f.usdc.address).mint(&funder, &(500 * UNIT));
    f.deposit(1000 * UNIT);
    f.set_rate_pct(9_000);
    let owed = f.adapter.get_principal(&f.usdc.address);

    f.controller.pause(&true, &false); // works while mint is paused
    f.controller.recollateralize(&protocol(), &f.usdc.address, &funder, &(100 * UNIT));

    assert_eq!(f.usdc.balance(&funder), 400 * UNIT);
    assert_eq!(f.cusd.balance(&funder), 0);
    assert_eq!(f.supply(), 1000 * UNIT);
    assert!(f.position() + DUST >= 1000 * UNIT);
    assert_eq!(f.adapter.get_principal(&f.usdc.address), owed + 100 * UNIT);
    assert_eq!(f.yield_(), 0);
}

#[test]
fn recollateralize_needs_the_admin_and_the_funder_and_a_supported_asset() {
    let f = setup();
    let funder = Address::generate(&f.env);
    StellarAssetClient::new(&f.env, &f.usdc.address).mint(&funder, &(500 * UNIT));
    StellarAssetClient::new(&f.env, &f.xlm.address).mint(&funder, &(500 * UNIT));
    let amount = 100 * UNIT;
    let args: Vec<Val> = (protocol(), f.usdc.address.clone(), funder.clone(), amount).into_val(&f.env);

    assert_code!(f.controller.try_recollateralize(&protocol(), &f.xlm.address, &funder, &amount), 1000);
    assert_code!(f.controller.try_recollateralize(&protocol(), &f.usdc.address, &funder, &0), 8);

    // The funder alone, the admin alone, or the owner: none is enough.
    for who in [&funder, &f.admin, &f.owner] {
        f.sign(who, "recollateralize", args.clone(), &[]);
        assert!(f.controller.try_recollateralize(&protocol(), &f.usdc.address, &funder, &amount).is_err());
    }
    assert_eq!(f.usdc.balance(&funder), 500 * UNIT);

    // Admin and funder together, each signing only their own part.
    let requests: Vec<Request> = vec![&f.env, Request { request_type: 2, address: f.usdc.address.clone(), amount }];
    let transfer = MockAuthInvoke {
        contract: &f.usdc.address,
        fn_name: "transfer",
        args: (funder.clone(), f.pool.address.clone(), amount).into_val(&f.env),
        sub_invokes: &[],
    };
    let submit = MockAuthInvoke {
        contract: &f.pool.address,
        fn_name: "submit",
        args: (f.controller.address.clone(), funder.clone(), f.controller.address.clone(), requests).into_val(&f.env),
        sub_invokes: &[transfer],
    };
    let funder_auth = MockAuth {
        address: &funder,
        invoke: &MockAuthInvoke {
            contract: &f.controller.address, fn_name: "recollateralize", args: args.clone(), sub_invokes: &[submit],
        },
    };
    let admin_auth = MockAuth {
        address: &f.admin,
        invoke: &MockAuthInvoke {
            contract: &f.controller.address, fn_name: "recollateralize", args: args.clone(), sub_invokes: &[],
        },
    };
    // The funder's complete signature without the admin's is refused: anyone could otherwise top up.
    f.env.mock_auths(&[funder_auth.clone()]);
    assert!(f.controller.try_recollateralize(&protocol(), &f.usdc.address, &funder, &amount).is_err());
    assert_eq!(f.usdc.balance(&funder), 500 * UNIT);

    f.env.mock_auths(&[admin_auth, funder_auth]);
    f.controller.recollateralize(&protocol(), &f.usdc.address, &funder, &amount);
    assert_eq!(f.usdc.balance(&funder), 400 * UNIT);
    assert_eq!(f.supply(), 0);
}

/// Replay of the mainnet state after the 2026-11-19 exploit, from `misc/recovery-plan.md`.
#[test]
fn recovery_replay_of_the_mainnet_numbers() {
    let f = setup();
    let funder = Address::generate(&f.env);
    let taken = 1_052_0174658;
    StellarAssetClient::new(&f.env, &f.usdc.address).mint(&funder, &taken);

    // USDC: 0.0586882 left behind 1,051.7788481 cUSD; the epoch record is over-withdrawn.
    f.pool.set_collateral(&f.controller.address, &f.usdc.address, &586_882);
    f.seed_record(&f.usdc.address, 51_7790283, 1_015_0000000, 1_067_0174658);
    // XLM: the attacker's deposit, never a supported asset.
    f.pool.set_collateral(&f.controller.address, &f.xlm.address, &1_052_0174660);
    f.seed_record(&f.xlm.address, 0, 1_052_0174658, 0);
    f.env.as_contract(&f.manager.address, || {
        f.env.storage().persistent().set(&ManagerKey::CusdSupply, &1_051_7788481i128)
    });

    f.controller.pause(&true, &true);
    f.controller.recollateralize(&protocol(), &f.usdc.address, &funder, &taken);

    assert_eq!(f.supply(), 1_051_7788481);
    assert_eq!(f.adapter.get_principal(&f.usdc.address), 1_051_7790283);
    assert_eq!(f.position(), 1_052_0761540);
    assert_eq!(f.yield_(), 2_971_257);
    assert!(f.position() >= f.supply());

    // The XLM comes out; the USDC backing cannot be rescued.
    assert_code!(f.controller.try_rescue_collateral(&protocol(), &f.usdc.address, &funder, &UNIT), 1008);
    f.controller.rescue_collateral(&protocol(), &f.xlm.address, &funder, &taken);
    assert_eq!(f.xlm.balance(&funder), taken);
    assert_eq!(f.adapter.get_balance(&f.controller.address, &f.xlm.address), 2);
    assert_eq!(f.supply(), 1_051_7788481);
    assert_eq!(f.position(), 1_052_0761540);

    // Still paused in both directions until the owner clears it.
    assert_code!(f.controller.try_deposit_collateral(&protocol(), &f.user, &f.usdc.address, &UNIT), 1006);
    assert_code!(f.controller.try_withdraw_collateral(&protocol(), &f.user, &f.usdc.address, &UNIT), 1007);
}

#[test]
fn rescue_collateral_is_owner_only_and_exact() {
    let f = setup();
    let to = Address::generate(&f.env);
    f.pool.set_collateral(&f.controller.address, &f.xlm.address, &(100 * UNIT));
    let args: Vec<Val> = (protocol(), f.xlm.address.clone(), to.clone(), 40 * UNIT).into_val(&f.env);

    f.sign(&f.admin, "rescue_collateral", args.clone(), &[]);
    assert!(f.controller.try_rescue_collateral(&protocol(), &f.xlm.address, &to, &(40 * UNIT)).is_err());

    f.sign(&f.owner, "rescue_collateral", args, &[]);
    f.controller.rescue_collateral(&protocol(), &f.xlm.address, &to, &(40 * UNIT));
    assert_eq!(f.xlm.balance(&to), 40 * UNIT);
    assert_eq!(f.adapter.get_balance(&f.controller.address, &f.xlm.address), 60 * UNIT);

    // More than the position: the pool would clamp, so it reverts.
    f.env.mock_all_auths_allowing_non_root_auth();
    assert_code!(f.controller.try_rescue_collateral(&protocol(), &f.xlm.address, &to, &(61 * UNIT)), 10);
}

#[test]
fn rescue_moves_stray_tokens_but_never_a_pending_harvest() {
    let f = setup();
    let to = Address::generate(&f.env);
    StellarAssetClient::new(&f.env, &f.blnd.address).mint(&f.controller.address, &(7 * UNIT));
    let args: Vec<Val> = (f.blnd.address.clone(), to.clone(), 7 * UNIT).into_val(&f.env);

    f.sign(&f.admin, "rescue", args.clone(), &[]);
    assert!(f.controller.try_rescue(&f.blnd.address, &to, &(7 * UNIT)).is_err());
    f.sign(&f.owner, "rescue", args, &[]);
    f.controller.rescue(&f.blnd.address, &to, &(7 * UNIT));
    assert_eq!(f.blnd.balance(&to), 7 * UNIT);

    f.env.mock_all_auths_allowing_non_root_auth();
    f.deposit(1000 * UNIT);
    f.set_rate_pct(10_500);
    let y = f.harvest();
    assert_code!(f.controller.try_rescue(&f.usdc.address, &to, &y), 1003);
    f.recompound();
    assert_eq!(f.usdc.balance(&f.controller.address), 0);
}

/// Admin can only switch pauses on; only the owner clears them. Each direction is independent.
#[test]
fn pause_is_set_by_the_admin_and_cleared_by_the_owner() {
    let f = setup();
    f.deposit(1000 * UNIT);

    f.sign(&f.user, "pause", (true, true).into_val(&f.env), &[]);
    assert!(f.controller.try_pause(&true, &true).is_err());

    f.sign(&f.admin, "pause", (true, false).into_val(&f.env), &[]);
    f.controller.pause(&true, &false);
    assert_eq!(f.controller.get_paused(), (true, false));
    f.sign(&f.admin, "pause", (false, true).into_val(&f.env), &[]);
    f.controller.pause(&false, &true); // `false` does not clear the mint pause
    assert_eq!(f.controller.get_paused(), (true, true));

    f.sign(&f.admin, "unpause", (true, true).into_val(&f.env), &[]);
    assert!(f.controller.try_unpause(&true, &true).is_err());
    f.sign(&f.owner, "unpause", (false, true).into_val(&f.env), &[]);
    f.controller.unpause(&false, &true);
    assert_eq!(f.controller.get_paused(), (true, false));

    f.env.mock_all_auths_allowing_non_root_auth();
    assert_code!(f.controller.try_deposit_collateral(&protocol(), &f.user, &f.usdc.address, &UNIT), 1006);
    f.controller.withdraw_collateral(&protocol(), &f.user, &f.usdc.address, &(100 * UNIT));
    f.controller.unpause(&true, &false);
    f.deposit(UNIT);
    assert_eq!(f.supply(), 901 * UNIT);
}

/// Zero and negative amounts are refused before anything moves.
#[test]
fn non_positive_amounts_are_rejected() {
    let f = setup();
    f.deposit(1000 * UNIT);
    for amount in [0i128, -1] {
        assert_code!(f.controller.try_deposit_collateral(&protocol(), &f.user, &f.usdc.address, &amount), 8);
        assert_code!(f.controller.try_withdraw_collateral(&protocol(), &f.user, &f.usdc.address, &amount), 8);
        assert_code!(f.controller.try_rescue(&f.usdc.address, &f.user, &amount), 8);
        assert_code!(f.controller.try_rescue_collateral(&protocol(), &f.xlm.address, &f.user, &amount), 8);
    }
}
