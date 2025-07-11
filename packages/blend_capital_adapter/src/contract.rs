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

        // Initialize epoch principal if this is the first deposit
        if storage::get_asset_epoch_principal(e, &asset).is_none() {
            storage::set_asset_epoch_principal(e, &asset, 0, 0);
        }

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

    fn claim_yield(e: &Env, asset: Address, yield_amount: i128) -> i128 {
        
        storage::require_yield_controller(e);
        
        let claimed = adapter::withdraw_collateral(e, storage::get_yield_controller(e), asset.clone(), yield_amount);
        
        LendingAdapterEvents::claim_yield(
            &e,
            storage::get_yield_controller(e),
            storage::get_yield_controller(e),
            asset,
            claimed,
        );
        
        claimed
    }

    fn claim_yield_auth(e: &Env, asset: Address, amount: i128) -> Option<(Address, Symbol, Vec<Val>)> {

        Some( adapter::withdraw_collateral_auth(e, storage::get_yield_controller(e), asset.clone(), amount) )
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

    fn get_emissions(e: &Env, asset: Address) -> i128 { adapter::get_user_emissions(e, storage::get_yield_controller(e), asset.clone()) }

    fn protocol_token(e: &Env) -> Address {

        storage::read_blend_token_id(e)
    }

    fn get_total_deposited(e: &Env, asset: Address) -> i128 { storage::read_deposit(e, &storage::get_yield_controller(e), &asset) }
    
    fn get_balance(e: &Env, user: Address, asset: Address) -> i128 {
        adapter::get_balance(e, user, asset)
    }
    
    fn get_apy(env: &Env, asset: Address) -> u32 {
        adapter::get_apy(env, asset)
    }
    
    fn update_epoch_principal(env: &Env, asset: Address, epoch: u64, principal: i128) {
        storage::require_yield_controller(env);
        storage::set_asset_epoch_principal(env, &asset, epoch, principal);
        
        LendingAdapterEvents::update_epoch_principal(env, asset, epoch, principal);
    }
}
