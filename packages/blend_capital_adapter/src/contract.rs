use soroban_sdk::{
    contract, 
    contractimpl, 
    Address, 
    Env,
};
use crate::{
    constants::{YIELD_CONTROLLER_ID, LENDING_POOL_ID, BLEND_TOKEN_ID},
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
    fn __constructor(e: Env, yield_controller: Address, blend_pool_id: Address, blend_token_id: Address) {
        e.storage()
            .instance()
            .set(&YIELD_CONTROLLER_ID, &yield_controller);
        e.storage()
            .instance()
            .set(&LENDING_POOL_ID, &blend_pool_id);
        e.storage()
            .instance()
            .set(&BLEND_TOKEN_ID, &blend_token_id);
    }

    fn deposit(e: &Env, user: Address, asset: Address, amount: i128) -> i128 {
        storage::require_yield_controller(e);

        adapter::supply_collateral(e, user, asset.clone(), amount);
        
        LendingAdapterEvents::deposit(&e, e.current_contract_address(), asset, amount);

        amount
    }

    fn withdraw(e: &Env, user: Address, asset: Address, amount: i128) -> i128 {
        storage::require_yield_controller(e);

        adapter::withdraw_collateral(e, user.clone(), asset.clone(), amount);

        LendingAdapterEvents::withdraw(&e, e.current_contract_address(), user, asset, amount);

        amount
    }

    fn get_yield(e: &Env, asset: Address) -> i128 {

        adapter::read_yield(e, e.current_contract_address(), asset)
    }

    fn claim_yield(e: &Env, asset: Address, recipient: Address) -> i128 {
        storage::require_yield_controller(e);

        let yield_amount = adapter::read_yield(e, e.current_contract_address(), asset.clone());
        if yield_amount <= 0 {
            return 0;
        }

        adapter::withdraw_collateral(e, recipient.clone(), asset.clone(), yield_amount);

        LendingAdapterEvents::claim_yield(
            &e,
            e.current_contract_address(),
            recipient,
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

    fn get_emissions(e: &Env, from: Address, asset: Address) -> i128 {
        
        storage::require_yield_controller(e);
        
        adapter::get_user_emissions(e, from.clone(), asset.clone())
    }

    fn protocol_token(e: &Env) -> Address {

        storage::read_blend_token_id(e)
    }
}
