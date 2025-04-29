use soroban_sdk::{
    contract, 
    contracterror, 
    contractimpl, 
    contractmeta, 
    contracttype, 
    panic_with_error, 
    vec,
    symbol_short, 
    Symbol,
    Address, 
    Env, 
    Map, 
    Vec
};

use crate::lending_adapter_trait::LendingPoolAdapter;

// Define error codes for the adapter
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BlendAdapterError {
    AssetNotSupported = 1,
    InsufficientBalance = 2,
    PoolOperationFailed = 3,
    Unauthorized = 4,
}

// According to Blend docs, RequestType is an enum that we'll use to construct requests
#[derive(Clone)]
#[repr(u32)]
pub enum RequestType {
    Deposit = 0,          // Supply without collateralizing
    Withdraw = 1,         // Withdraw uncollateralized funds
    SupplyCollateral = 2, // Supply and mark as collateral
    WithdrawCollateral = 3, // Withdraw from collateral
    Borrow = 4,           // Borrow funds (not used in our adapter)
    Repay = 5,            // Repay debt (not used in our adapter)
}

// Represents a request to the Blend pool
#[contracttype]
#[derive(Clone)]
pub struct Request {
    pub request_type: u32,
    pub address: Address,  // Asset address
    pub amount: i128,      // Amount to deposit/withdraw
}

contractmeta!(
    key = "Description",
    val = "Blend Capital Lending Pool Adapter for the Coopstable cUSD system"
);

const ADMIN_ID: Symbol = symbol_short!("AID");

fn require_admin_auth(admin_id: Address) {
    admin_id.require_auth();
}

trait LendingAdapterControllerTrait {
    fn __constructor(env: Env, admin_id: Address);
    fn register_adapter(env: &Env, protocol: Symbol, address: Address);
    fn remove_lending_protocol(env: &Env, protocol: Symbol, address: Address);
    fn get_lending_adapter(env: &Env, protocol: Symbol) -> Address;
    fn deploy_adapter(env: &Env, protocol: Symbol, address: Address);
}

#[contract]
pub struct LendingAdapterController;

#[contractimpl]
impl LendingPoolAdapter {
    pub fn __constructor(
        env: Env,
        admin_id: Address,
    ) {
        env.storage()
            .persistent()
            .set(&FUNDING_CONTROLLER_ID, &funding_controller_id);
        
        env.storage()
            .persistent()
            .set(&BLEND_POOL_ID, &blend_pool_id); 
    }

    fn get_pool_id(env: &Env) -> Address {
        env.storage().persistent().get(&BLEND_POOL_ID).unwrap()
    }
    
    fn get_funding_controller_id(env: &Env) -> Address {
        env.storage().persistent().get(&FUNDING_CONTROLLER_ID).unwrap()
    }
}

impl BlendLendingAdapter {
    fn create_request(     
        request_type: RequestType, 
        asset: Address, 
        amount: i128
    ) -> Request {
        Request {
            request_type: request_type as u32,
            address: asset,
            amount,
        }
    }
    
     // Helper to call the Blend pool's submit method
     fn call_blend_pool(
        env: &Env,
        pool_id: &Address,
        spender: &Address,
        from: &Address,
        to: &Address,
        requests: Vec<Request>
    ) -> Result<(), BlendAdapterError> {
        // Convert requests to the format expected by Blend (assuming they use Symbol/Val pairs)
        // let requests_val = requests.iter().map(|req| {
        //         let mut map = Map::new(env);
        //         map.set(Symbol::new(env, "request_type"), req.request_type.into_val(env));
        //         map.set(Symbol::new(env, "address"), req.address.to_val());
        //         map.set(Symbol::new(env, "amount"), req.amount.into_val(env));
        //         map
        //     })
        //     .collect();
        
        // Call the pool contract's submit function
        // Note: In a real implementation, you would need to handle error cases properly
        // let _result = pool_id.try_call::<_, ()>(
        //     env,
        //     Symbol::new(env, "submit"),
        //     (spender, from, to, requests_val)
        // ).map_err(|_| BlendAdapterError::PoolOperationFailed)?;
        // TODO: implement the deposit function
        
        Ok(())
    }

}

#[contractimpl]
impl LendingPoolAdapter for LendingPoolAdapter  {

    
    // Implementation of deposit method - deposits assets into Blend pool
    fn deposit(
        env: &Env,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {
        // Check if asset is supported
        require_funding_controller_auth(Self::get_funding_controller_id(env));
        
        // Require user authorization
        user.require_auth();
        
        // Get the Blend pool ID
        let pool_id: Address = Self::get_pool_id(env);
        
        // We'll use SupplyCollateral (Type 2) to ensure we're depositing as collateral
        let request = Self::create_request(RequestType::SupplyCollateral, asset.clone(), amount);
        
        // Call the Blend pool contract to submit the deposit
        Self::call_blend_pool(
            &env,
            &pool_id,
            &user,  // spender (the user pays)
            &user,  // from (user's funds)
            &user,  // to (positions credited to user)
            vec![&env, request]
        ).unwrap_or_else(|e| panic_with_error!(&env, e));
        
        // Emit deposit event
        env.events().publish(
            ("BLEND_ADAPTER", "deposit"),
            (user, asset, amount)
        );
        
        // Return the deposited amount
        amount
    }
    
    // Implementation of withdraw method - withdraws assets from Blend pool
    fn withdraw(
        env: &Env,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {
        
        require_funding_controller_auth(Self::get_funding_controller_id(env));

        // Require user authorization
        user.require_auth();
        
        // Get the Blend pool ID
        let pool_id: Address = Self::get_pool_id(env);
        
        // Check user balance first to prevent unnecessary calls
        let balance = Self::get_balance(&env, user.clone(), asset.clone());
        if balance < amount {
            panic_with_error!(&env, BlendAdapterError::InsufficientBalance);
        }
        
        // We'll use WithdrawCollateral (Type 3) since we deposited as collateral
        let request = Self::create_request(RequestType::WithdrawCollateral, asset.clone(), amount);
        
        // Call the Blend pool contract to submit the withdrawal
        Self::call_blend_pool(
            &env,
            &pool_id,
            &user,  // spender 
            &user,  // from
            &user,  // to
            vec![&env, request]
        ).unwrap_or_else(|e| panic_with_error!(&env, e));
        
        // Emit withdrawal event
        env.events().publish(
            ("BLEND_ADAPTER", "withdraw"),
            (user, asset, amount)
        );
        
        // Return the withdrawn amount
        amount
    }
    
    // Get user's balance in the Blend pool
    fn get_balance(
        env: &Env,
        user: Address,d
        
        // Get the Blend pool ID
        let pool_id: Address = Self::get_pool_id(env);
        
        // Call the Blend pool to get positions (simulated, actual implementation would vary)
        let positions: Map<Address, Map<Symbol, i128>> = pool_id
            .try_call(
                env,
                Symbol::new(env, "get_positions"),
                user.to_val()
            )
            .unwrap_or_else(|_| Map::new(env));
        
        // Check if user has a bToken balance for this asset (lender position)
        // In real implementation, you'd need to look for the asset in the positions map
        // and convert the bToken amount to the underlying asset amount using the bRate
        if let Some(asset_position) = positions.get(asset) {
            // Look for bToken balance (represents supply/collateral)
            if let Some(btoken_bal) = asset_position.get(Symbol::new(env, "btoken")) {
                // Get the bRate to convert to underlying tokens
                // This is a simplification - in reality, you'd query the bRate from the pool
                let b_rate: i128 = pool_id
                    .try_call(
                        env,
                        Symbol::new(env, "get_b_rate"),
                        asset.to_val()
                    )
                    .unwrap_or(1_000_000); // Default to 1.0 (scaled by 1e6)
                
                // Convert bTokens to underlying tokens: bTokens * bRate
                // Note: Actual calculation might need to handle decimals properly
                return btoken_bal * b_rate / 1_000_000;
            }
        }
        
        0 // No position found
    }
    
    // Get user's accrued yield in the Blend pool
    fn get_yield(
        env: &Env,
        user: Address,
        asset: Address
    ) -> i128 {

        // Get the Blend pool ID
        let pool_id: Address = Self::get_pool_id(env);
        
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
