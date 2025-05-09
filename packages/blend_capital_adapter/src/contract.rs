use crate::artifacts::pool::{Client as PoolClient, Request};
use crate::contract_types::RequestType;
use crate::storage::{get_deposit_amount, remove_deposit, store_deposit};
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, Env, Symbol, Vec};
use yield_adapter::{
    constants::{ADAPTER_INSTANCE_BUMP_AMOUNT, ADAPTER_INSTANCE_LIFETIME_THRESHOLD},
    events::LendingAdapterEvents,
    lending_adapter::LendingAdapter,
};
const YIELD_CONTROLLER_ID: Symbol = symbol_short!("LACID");
const LENDING_POOL_ID: Symbol = symbol_short!("LID");

fn get_yield_controller(e: &Env) -> Address {
    e.storage().instance().extend_ttl(
        ADAPTER_INSTANCE_LIFETIME_THRESHOLD,
        ADAPTER_INSTANCE_BUMP_AMOUNT,
    );

    e.storage().instance().get(&YIELD_CONTROLLER_ID).unwrap()
}

fn require_yield_controller(e: &Env) {
    let yield_controller_id: Address = get_yield_controller(e);
    yield_controller_id.require_auth()
}

fn read_lend_pool_id(e: &Env) -> Address {
    e.storage().instance().get(&LENDING_POOL_ID).unwrap()
}

trait BlendCapitalAdapterTrait {
    fn create_request(request_type: RequestType, asset: Address, amount: i128) -> Request;

    fn supply_collateral(e: &Env, user: Address, asset: Address, amount: i128) -> i128;

    fn withdraw_collateral(e: &Env, user: Address, asset: Address, amount: i128) -> i128;

    fn get_balance(e: &Env, user: Address, asset: Address) -> i128;

    fn get_reserve_token_id(e: &Env, asset: Address) -> Option<u32>;
}

impl BlendCapitalAdapterTrait for BlendCapitalAdapter {
    fn create_request(request_type: RequestType, asset: Address, amount: i128) -> Request {
        Request {
            request_type: request_type as u32,
            address: asset,
            amount,
        }
    }

    fn supply_collateral(e: &Env, user: Address, asset: Address, amount: i128) -> i128 {
        let pool_id: Address = read_lend_pool_id(e);
        let pool_client = PoolClient::new(e, &pool_id);

        let request = Self::create_request(RequestType::SupplyCollateral, asset.clone(), amount);
        let request_vec: Vec<Request> = vec![e, request];
        pool_client.submit_with_allowance(
            &user, // user in this case will be the yield controller
            &e.current_contract_address(),
            &user, // user in this case will be the yield controller
            &request_vec,
        );

        store_deposit(e, &user, &asset, amount);

        amount
    }

    fn withdraw_collateral(e: &Env, user: Address, asset: Address, amount: i128) -> i128 {
        let pool_id: Address = read_lend_pool_id(e);
        let pool_client = PoolClient::new(e, &pool_id);
        let request = Self::create_request(RequestType::WithdrawCollateral, asset.clone(), amount);

        let request_vec: Vec<Request> = vec![e, request];

        pool_client.submit_with_allowance(
            &user, // user in this case will be the yield controller
            &e.current_contract_address(),
            &user, // user in this case will be the yield controller
            &request_vec,
        );

        // Remove the withdrawn amount from tracking
        remove_deposit(e, &user, &asset, amount);

        amount
    }

    fn get_balance(e: &Env, user: Address, asset: Address) -> i128 {
        let pool_id: Address = read_lend_pool_id(e);
        let pool_client = PoolClient::new(e, &pool_id);

        // Get the user's positions and the reserve from the pool
        let positions = pool_client.get_positions(&user);
        let reserve = pool_client.get_reserve(&asset);

        // Find the reserve index for the asset
        let reserve_list = pool_client.get_reserve_list();
        let mut reserve_index = None;

        for (i, addr) in reserve_list.iter().enumerate() {
            if addr == asset {
                reserve_index = Some(i as u32);
                break;
            }
        }

        if let Some(idx) = reserve_index {
            // Check if user has a collateral position for this asset
            if let Some(b_token_amount) = positions.collateral.get(idx) {
                // The b_rate represents exchange rate between bTokens and the underlying asset
                // Calculate underlying asset value: b_tokens * b_rate / 10^12
                // Note: SCALAR_12 (10^12) is the fixed-point scalar used in the contract
                let scalar_12: i128 = 1_000_000_000_000;
                return (b_token_amount * reserve.data.b_rate) / scalar_12;
            }
        }

        0 // No position found
    }

    fn get_reserve_token_id(e: &Env, asset: Address) -> Option<u32> {
        let pool_id: Address = read_lend_pool_id(e);
        let pool_client = PoolClient::new(e, &pool_id);

        let reserve_list = pool_client.get_reserve_list();

        for (i, addr) in reserve_list.iter().enumerate() {
            if addr == *&asset {
                // For collateral (bTokens), reserve_token_id = reserve_index * 2 + 1
                return Some((i as u32) * 2 + 1);
            }
        }

        None
    }
}

#[contract]
pub struct BlendCapitalAdapter;

#[contractimpl]
impl LendingAdapter for BlendCapitalAdapter {
    fn __constructor(e: Env, lending_adapter_controller_id: Address, lending_pool_id: Address) {
        e.storage()
            .instance()
            .set(&YIELD_CONTROLLER_ID, &lending_adapter_controller_id);
        e.storage()
            .instance()
            .set(&LENDING_POOL_ID, &lending_pool_id);
    }

    fn deposit(e: &Env, user: Address, asset: Address, amount: i128) -> i128 {
        require_yield_controller(e);

        Self::supply_collateral(e, user.clone(), asset.clone(), amount);

        LendingAdapterEvents::deposit(&e, e.current_contract_address(), user, asset, amount);

        amount
    }

    fn withdraw(e: &Env, user: Address, asset: Address, amount: i128) -> i128 {
        require_yield_controller(e);

        Self::withdraw_collateral(e, user.clone(), asset.clone(), amount);

        LendingAdapterEvents::withdraw(&e, e.current_contract_address(), user, asset, amount);

        amount
    }

    fn get_yield(e: &Env, user: Address, asset: Address) -> i128 {
        // Get the current value of user's supplied collateral
        let current_value = Self::get_balance(e, user.clone(), asset.clone());

        // Get the original deposit amount
        let original_deposit = get_deposit_amount(e, &user, &asset);

        // The yield is the difference between current value and original deposit
        // If there is no deposit or current value is less, return 0
        if original_deposit == 0 || current_value <= original_deposit {
            return 0;
        }

        current_value - original_deposit
    }

    fn claim_yield(e: &Env, user: Address, asset: Address) -> i128 {
        require_yield_controller(e);

        let yield_amount = Self::get_yield(e, user.clone(), asset.clone());
        if yield_amount <= 0 {
            return 0;
        }

        if let Some(reserve_token_id) = Self::get_reserve_token_id(e, asset.clone()) {
            let pool_id: Address = read_lend_pool_id(e);
            let pool_client = PoolClient::new(e, &pool_id);

            let reserve_token_ids = vec![e, reserve_token_id];

            // Claim emissions - this sends BLND tokens to the user
            let emission_amount = pool_client.claim(&user, &reserve_token_ids, &user);

            if emission_amount > 0 {
                e.events().publish(
                    ("emissions_claimed", user.clone()),
                    (asset.clone(), emission_amount),
                );
            }
        }

        Self::withdraw_collateral(e, user.clone(), asset.clone(), yield_amount);

        LendingAdapterEvents::claim_yield(
            &e,
            e.current_contract_address(),
            user,
            asset,
            yield_amount,
        );

        yield_amount
    }
}
