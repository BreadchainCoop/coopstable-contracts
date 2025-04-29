use soroban_sdk::{contract, contractimpl,  symbol_short, Address, Env, Symbol};
use super::contract_types::{RequestType, Request};
use yield_adapter::{
    lending_adapter::LendingAdapter,
    storage_types::{ ADAPTER_INSTANCE_BUMP_AMOUNT, ADAPTER_INSTANCE_LIFETIME_THRESHOLD }
};

#[contract]
pub struct BlendCapitalAdapter;

const LENDING_ADAPTER_CONTROLLER_ID: Symbol = symbol_short!("LACID");
const BLEND_POOL_ID: Symbol = symbol_short!("BID");

fn require_lending_adapter_controller_auth(e: &Env) { 
    
    e.storage()
        .instance()
        .extend_ttl(ADAPTER_INSTANCE_LIFETIME_THRESHOLD, ADAPTER_INSTANCE_BUMP_AMOUNT);
    
    let lending_adapter_controller_id: Address = e
        .storage()
        .instance()
        .get(&LENDING_ADAPTER_CONTROLLER_ID).unwrap();
    
    lending_adapter_controller_id.require_auth()
}

fn read_blend_pool_id(e: &Env) -> Address {
    e.storage().instance().get(&BLEND_POOL_ID).unwrap()
}

#[contractimpl]
impl BlendCapitalAdapter {
    fn __constructor(
        env: Env, 
        lending_adapter_controller_id: Address,
        blend_pool_id: Address
    ) {
        env.storage().instance().set(&LENDING_ADAPTER_CONTROLLER_ID, &lending_adapter_controller_id);
        env.storage().instance().set(&BLEND_POOL_ID, &blend_pool_id);   
    }

    fn create_request(request_type: RequestType, asset: Address, amount: i128) -> Request {
        Request {
            request_type: request_type as u32,
            address: asset,
            amount,
        }
    }

    fn supply_collateral(
        env: &Env,
        user: &Address,
        asset: &Address,
        amount: i128
    ) -> i128 {
        require_lending_adapter_controller_auth(env);
        
        // Get the Blend pool ID
        let pool_id: Address = read_blend_pool_id(env);
        
        // We'll use SupplyCollateral (Type 2) to ensure we're depositing as collateral
        let request = Self::create_request(RequestType::SupplyCollateral, asset.clone(), amount);
        
        // TODO: implement supply collateral method
        
        // Emit deposit event
        env.events().publish(
            ("BLEND_ADAPTER", "deposit"),
            (user, asset, amount)
        );
        
        // Return the deposited amount
        amount
    }

    fn withdraw_collateral(
        env: &Env,
        user: &Address,
        asset: &Address,
        amount: i128
    ) -> i128 {
        require_lending_adapter_controller_auth(env);
        
        // Get the Blend pool ID
        let pool_id: Address = read_blend_pool_id(env);
        
        // We'll use WithdrawCollateral (Type 3) since we deposited as collateral
        let request = Self::create_request(RequestType::WithdrawCollateral, asset.clone(), amount);
        
        // TODO: implement withdraw collateral method
        
        // Emit withdrawal event
        env.events().publish(
            ("BLEND_ADAPTER", "withdraw"),
            (user, asset, amount)
        );
        
        // Return the withdrawn amount
        amount
    }
}

#[contractimpl]
impl LendingAdapter for BlendCapitalAdapter  {

    // Implementation of deposit method - deposits assets into Blend pool
    fn deposit(
        env: &Env,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {
        // Check if asset is supported
        require_lending_adapter_controller_auth(env);
                        
        Self::supply_collateral(env, &user, &asset, amount);
        
        // Emit deposit event
        env.events().publish(
            ("BLEND_ADAPTER", "deposit"),
            (user, asset, amount)
        );
        
        amount
    }
    
    // Implementation of withdraw method - withdraws assets from Blend pool
    fn withdraw(
        env: &Env,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {
        
        require_lending_adapter_controller_auth(env);

        Self::withdraw_collateral(env, &user, &asset, amount);
        
        // Emit withdrawal event
        env.events().publish(
            ("BLEND_ADAPTER", "withdraw"),
            (user, asset, amount)
        );
        
        amount
    }
    
    // Get user's balance in the Blend pool
    fn get_balance(
        env: &Env,
        user: Address,
        asset: Address
    ) -> i128 {
        
        // Get the Blend pool ID
        let pool_id: Address = read_blend_pool_id(env);
        
        // TODO: implement get balance method

        0 // No position found
    }
    
    // Get user's accrued yield in the Blend pool
    fn get_yield(
        env: &Env,
        user: Address,
        asset: Address
    ) -> i128 {

        // Get the Blend pool ID
        let pool_id: Address = read_blend_pool_id(env);
        
        // This is a simplified approach - in a real implementation, you'd:
        // TODO: implement the yield method
        // 1. Get the user's bToken balance 
        // 2. Calculate the current value using the current bRate
        // 3. Calculate the original deposit value
        // 4. Return the difference (current value - original deposit)
        
        // For now, we'll return 0 as a placeholder
        // In reality, you might store the original deposit amount separately
        // and compare it with the current value calculated in get_balance
        0
    }
}
