use soroban_sdk::{
    contract, 
    contractimpl, 
    Address, 
    Env, 
};
use crate::{
    constants::{YIELD_CONTROLLER_ID, LENDING_POOL_ID},
    storage,
    adapter
};
use yield_adapter::{
    events::LendingAdapterEvents,
    lending_adapter::LendingAdapter,
};

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
        storage::require_yield_controller(e);

        adapter::supply_collateral(e, user.clone(), asset.clone(), amount);

        LendingAdapterEvents::deposit(&e, e.current_contract_address(), user, asset, amount);

        amount
    }

    fn withdraw(e: &Env, user: Address, asset: Address, amount: i128) -> i128 {
        storage::require_yield_controller(e);

        adapter::withdraw_collateral(e, user.clone(), asset.clone(), amount);

        LendingAdapterEvents::withdraw(&e, e.current_contract_address(), user, asset, amount);

        amount
    }

    fn get_yield(e: &Env, user: Address, asset: Address) -> i128 {

        let current_value = adapter::get_balance(e, user.clone(), asset.clone());

        let original_deposit = storage::get_deposit_amount(e, &user, &asset);

        if original_deposit == 0 || current_value <= original_deposit {
            return 0;
        }

        current_value - original_deposit
    }

    fn claim_yield(e: &Env, user: Address, asset: Address) -> i128 {
        storage::require_yield_controller(e);

        let yield_amount = Self::get_yield(e, user.clone(), asset.clone());
        if yield_amount <= 0 {
            return 0;
        }

        adapter::withdraw_collateral(e, user.clone(), asset.clone(), yield_amount);

        LendingAdapterEvents::claim_yield(
            &e,
            e.current_contract_address(),
            user,
            asset,
            yield_amount,
        );

        yield_amount
    }

    fn claim_emissions(e: &Env, to: Address, asset: Address) -> i128 {
        storage::require_yield_controller(e);
        let from = e.current_contract_address();

        let emissions = adapter::claim(e, from.clone(), to.clone(), asset.clone());
        
        LendingAdapterEvents::claim_emissions(e, from, to, asset, emissions);

        emissions
    }
}
