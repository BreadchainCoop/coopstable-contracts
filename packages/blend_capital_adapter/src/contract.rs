use soroban_sdk::{ contract, contractimpl, Address, Env, Symbol, Val, Vec };
use crate::{ adapter, constants::{BLEND_TOKEN_ID, LENDING_POOL_ID, YIELD_CONTROLLER_ID}, storage };
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

    fn deposit_auth(e: &Env, user: Address, asset: Address, amount: i128) -> Option<(Address, Symbol, Vec<Val>)> {
        Some( adapter::supply_collateral_auth(e, user.clone(), asset.clone(), amount) )
    }

    fn withdraw(e: &Env, user: Address, asset: Address, amount: i128) -> i128 {
        
        storage::require_yield_controller(e);

        adapter::withdraw_collateral(e, user.clone(), asset.clone(), amount);

        LendingAdapterEvents::withdraw(&e, e.current_contract_address(), user, asset, amount);

        amount
    }

    fn withdraw_auth(e: &Env, user: Address, asset: Address, amount: i128) -> Option<(Address, Symbol, Vec<Val>)> {
        Some( adapter::withdraw_collateral_auth(e, user.clone(), asset.clone(), amount) )
    }

    fn get_yield(e: &Env, asset: Address) -> i128 { adapter::read_yield(e, storage::get_yield_controller(e), asset) }    

    fn claim_yield(e: &Env, asset: Address, recipient: Address) -> i128 {
        
        storage::require_yield_controller(e);
        
        let yield_amount = adapter::read_yield(e, storage::get_yield_controller(e), asset.clone());
        
        if yield_amount <= 0 {
            return 0;
        }
        
        adapter::withdraw_collateral(e, recipient.clone(), asset.clone(), yield_amount);
        
        LendingAdapterEvents::claim_yield(
            &e,
            storage::get_yield_controller(e),
            recipient,
            asset,
            yield_amount,
        );
        
        yield_amount
    }

    fn claim_yield_auth(e: &Env, asset: Address, recipient: Address) -> Option<(Address, Symbol, Vec<Val>)> {
        
        let yield_amount = adapter::read_yield(e, storage::get_yield_controller(e), asset.clone());
        if yield_amount <= 0 {
            return None;
        }
        Some( adapter::withdraw_collateral_auth(e, recipient.clone(), asset.clone(), yield_amount) )
    }
    
    fn claim_emissions(e: &Env, to: Address, asset: Address) -> i128 {
        storage::require_yield_controller(e);
        
        let emissions = adapter::claim(e, storage::get_yield_controller(e), to.clone(), asset.clone());

        LendingAdapterEvents::claim_emissions(e, storage::get_yield_controller(e), to, asset, emissions);

        emissions
    }

    fn claim_emissions_auth(e: &Env, to: Address, asset: Address) -> Option<(Address, Symbol, Vec<Val>)> {
        if let Some(auth_args) = adapter::claim_auth(e, storage::get_yield_controller(e), to.clone(), asset.clone()) {
            return Some(auth_args);
        }
        None
    }

    fn get_emissions(e: &Env, asset: Address) -> i128 {
        
        storage::require_yield_controller(e);
        
        adapter::get_user_emissions(e, storage::get_yield_controller(e), asset.clone())
    }

    fn protocol_token(e: &Env) -> Address {

        storage::read_blend_token_id(e)
    }
}
