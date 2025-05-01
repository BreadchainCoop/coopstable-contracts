use soroban_sdk::{
    contract, 
    contractimpl,  
    symbol_short, 
    Address, 
    Env, 
    Symbol, 
    vec, 
    Vec,
    token::TokenClient
};
use super::contract_types::RequestType;
use crate::artifacts::pool::{
    Client as PoolClient, 
    Request, 
    Reserve,
};
use yield_adapter::{
    lending_adapter::LendingAdapter,
    storage_types::{ ADAPTER_INSTANCE_BUMP_AMOUNT, ADAPTER_INSTANCE_LIFETIME_THRESHOLD }
};

#[contract]
pub struct BlendCapitalAdapter;

const YIELD_CONTROLLER_ID: Symbol = symbol_short!("LACID");
const BLEND_POOL_ID: Symbol = symbol_short!("BID");

fn get_yield_controller(e: &Env) -> Address {

    e.storage()
        .instance()
        .extend_ttl(ADAPTER_INSTANCE_LIFETIME_THRESHOLD, ADAPTER_INSTANCE_BUMP_AMOUNT);
    
    e.storage()
        .instance()
        .get(&YIELD_CONTROLLER_ID).unwrap()
}

fn require_yield_controller(e: &Env) { 
    let yield_controller_id: Address = get_yield_controller(e);
    yield_controller_id.require_auth()
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
        env.storage().instance().set(&YIELD_CONTROLLER_ID, &lending_adapter_controller_id);
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
        
        let pool_id: Address = read_blend_pool_id(env);
        let pool_client = PoolClient::new(env, &pool_id);
        
        let request = Self::create_request(RequestType::SupplyCollateral, asset.clone(), amount);
        let request_vec: Vec<Request> = vec![env, request];

        pool_client.submit_with_allowance(
            user, // user in this case will be the yield controller
            &env.current_contract_address(), 
            user, // user in this case will be the yield controller
            &request_vec
        );        
        
        amount
    }

    fn withdraw_collateral(
        env: &Env,
        user: &Address,
        asset: &Address,
        amount: i128
    ) -> i128 {
        
        let pool_id: Address = read_blend_pool_id(env);
        let pool_client = PoolClient::new(env, &pool_id);
        let request = Self::create_request(RequestType::WithdrawCollateral, asset.clone(), amount);
        
        let request_vec: Vec<Request> = vec![env, request];
        
        pool_client.submit_with_allowance(
            user, // user in this case will be the yield controller
            &env.current_contract_address(), 
            user, // user in this case will be the yield controller
            &request_vec
        );

        env.events().publish(
            ("BLEND_ADAPTER", "withdraw"),
            (user, asset, amount)
        );
        
        amount
    }

    fn get_balance(
        env: &Env,
        user: Address,
        asset: Address
    ) -> i128 {
        
        let pool_id: Address = read_blend_pool_id(env);
        let pool_client = PoolClient::new(env, &pool_id);
        
        // Get the user's positions from the pool
        let positions = pool_client.get_positions(&user);
        
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
            // Check collateral position (which is what we use for this adapter)
            if let Some(amount) = positions.collateral.get(idx) {
                // Get the reserve to convert bTokens to underlying asset value
                let reserve: Reserve = pool_client.get_reserve(&asset);
                
                // The Reserve struct has a method to convert bTokens to the underlying asset
                // This is a direct calculation based on the current b_rate
                // In a real-world implementation, you'd need to use the appropriate method
                // from the pool contract to perform this conversion
                
                // Calculate current value in underlying asset
                let value_in_asset = reserve.to_asset_from_b_token(env, amount);
                return value_in_asset;
            }
        }
        
        0 // No position found
    }
    

}

#[contractimpl]
impl LendingAdapter for BlendCapitalAdapter  {

    fn deposit(
        env: &Env,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {
        
        require_yield_controller(env);
                        
        Self::supply_collateral(env, &user, &asset, amount);
        
        env.events().publish(
            ("BLEND_ADAPTER", "deposit"),
            (user, asset, amount)
        );
        
        amount
    }
    
    fn withdraw(
        env: &Env,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {
        
        require_yield_controller(env);

        Self::withdraw_collateral(env, &user, &asset, amount);
        
        env.events().publish(
            ("BLEND_ADAPTER", "withdraw"),
            (user, asset, amount)
        );
        
        amount
    }
    
    
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

    fn claim_yield(env: &Env, user: Address, asset: Address) -> i128 {
        // TODO: implement claim_yield method
        0
    }
}
