use crate::contract_types::RequestType;

use soroban_sdk::{
    contract,
    contractimpl,
    log,
    Address, Env, TryIntoVal, Symbol, symbol_short, Vec, vec, Map, Val
};

// Define storage keys
const KEY_RESERVES: Symbol = symbol_short!("RES");
const KEY_BRATE: Symbol = symbol_short!("BRATE");
const KEY_POSITION: Symbol = symbol_short!("POS");

#[contract]
pub struct PoolContract;

#[contractimpl]
impl PoolContract {
    // Constructor to setup initial state
    pub fn init(env: Env, initial_asset: Address) -> () {
        // Initialize the reserves list with the initial asset
        let reserves = vec![&env, initial_asset.clone()];
        env.storage().instance().set(&KEY_RESERVES, &reserves);
        
        // Set initial b_rate (1:1)
        // Use a Map with asset address as the key within the b_rate storage
        let mut b_rates = Map::new(&env);
        b_rates.set(initial_asset, 1_000_000_000_000i128);
        env.storage().instance().set(&KEY_BRATE, &b_rates);
    }
    
    // Submit operations with allowance
    pub fn submit_with_allowance(
        env: Env,
        user: Address,
        _spender: Address,
        _sender: Address,
        requests: Vec<Map<Symbol, Val>>
    ) -> () {
        log!(&env, "submitting_with_allowance: {:?}", requests);
        for req in requests.iter() {
            let req_type: u32 = req.get(Symbol::new(&env,"request_type")).unwrap().try_into_val(&env).unwrap();
            let asset: Address = req.get(Symbol::new(&env,"address")).unwrap().try_into_val(&env).unwrap();
            let amount: i128 = req.get(Symbol::new(&env,"amount")).unwrap().try_into_val(&env).unwrap();
            
            // Get reserves list
            let reserves: Vec<Address> = env.storage().instance().get(&KEY_RESERVES).unwrap();
            
            // Find asset index
            let mut asset_index = None;
            for (i, addr) in reserves.iter().enumerate() {
                if addr == asset {
                    asset_index = Some(i);
                    break;
                }
            }
            
            if let Some(idx) = asset_index {
                // Get current positions map
                let positions: Option<Map<Address, Map<Symbol, Map<u32, i128>>>> = 
                    env.storage().instance().get(&KEY_POSITION);
                
                // Get or create positions map
                let mut positions_map = match positions {
                    Some(pos) => pos,
                    None => Map::new(&env)
                };
                
                // Get or create user position
                let position = match positions_map.get(user.clone()) {
                    Some(pos) => pos,
                    None => {
                        let empty_collateral = Map::<u32, i128>::new(&env);
                        let empty_debt = Map::<u32, i128>::new(&env);
                        
                        let mut result = Map::new(&env);
                        result.set(symbol_short!("collat"), empty_collateral);
                        result.set(symbol_short!("debt"), empty_debt);
                        
                        result
                    }
                };
                
                // Get b_rate map
                let b_rates: Map<Address, i128> = env.storage().instance().get(&KEY_BRATE).unwrap_or_else(|| {
                    Map::new(&env)
                });
                
                // Get b_rate for asset
                let b_rate = b_rates.get(asset.clone()).unwrap_or(1_000_000_000_000);
                
                // Process request based on type
                if req_type == RequestType::SupplyCollateral as u32 {
                    // Clone the position to create a mutable version
                    let mut position_clone = position.clone();
                    
                    // Get existing collateral map
                    let mut collateral_map = position_clone.get(symbol_short!("collat")).unwrap_or_else(|| {
                        Map::<u32, i128>::new(&env)
                    });
                    
                    // Convert amount to bTokens (divide by b_rate)
                    let b_tokens = (amount * 1_000_000_000_000) / b_rate;
                    
                    // Add to position
                    let current = collateral_map.get(idx as u32).unwrap_or(0);
                    collateral_map.set(idx as u32, current + b_tokens);
                    
                    // Update in position map
                    position_clone.set(symbol_short!("collat"), collateral_map);
                    
                    // Update user position in positions map
                    positions_map.set(user.clone(), position_clone);
                } 
                else if req_type == RequestType::WithdrawCollateral as u32 {
                    // Clone the position to create a mutable version
                    let mut position_clone = position.clone();
                    
                    // Get existing collateral map
                    let mut collateral_map = position_clone.get(symbol_short!("collat")).unwrap_or_else(|| {
                        Map::<u32, i128>::new(&env)
                    });
                    
                    // Convert amount to bTokens
                    let b_tokens = (amount * 1_000_000_000_000) / b_rate;
                    
                    // Remove from position
                    let current = collateral_map.get(idx as u32).unwrap_or(0);
                    if current >= b_tokens {
                        collateral_map.set(idx as u32, current - b_tokens);
                    } else {
                        collateral_map.set(idx as u32, 0);
                    }
                    
                    // Update in position map
                    position_clone.set(symbol_short!("collat"), collateral_map);
                    
                    // Update user position in positions map
                    positions_map.set(user.clone(), position_clone);
                }
                
                // Save updated positions
                env.storage().instance().set(&KEY_POSITION, &positions_map);
            }
        }
    }
    
    // Get user's positions
    pub fn get_positions(env: Env, user: Address) -> Map<Symbol, Map<u32, i128>> {
        // Get positions map
        let positions: Option<Map<Address, Map<Symbol, Map<u32, i128>>>> = 
            env.storage().instance().get(&KEY_POSITION);
        
        match positions {
            Some(pos_map) => match pos_map.get(user) {
                Some(user_pos) => user_pos,
                None => {
                    let empty_collateral = Map::<u32, i128>::new(&env);
                    let empty_debt = Map::<u32, i128>::new(&env);
                    
                    let mut result = Map::new(&env);
                    result.set(symbol_short!("collat"), empty_collateral);
                    result.set(symbol_short!("debt"), empty_debt);
                    
                    result
                }
            },
            None => {
                let empty_collateral = Map::<u32, i128>::new(&env);
                let empty_debt = Map::<u32, i128>::new(&env);
                
                let mut result = Map::new(&env);
                result.set(symbol_short!("collat"), empty_collateral);
                result.set(symbol_short!("debt"), empty_debt);
                
                result
            }
        }
    }
    
    // Get reserve data
    pub fn get_reserve(env: Env, asset: Address) -> Map<Symbol, Map<Symbol, i128>> {
        // Get b_rate map
        let b_rates: Map<Address, i128> = env.storage().instance().get(&KEY_BRATE).unwrap_or_else(|| {
            Map::new(&env)
        });
        
        // Get b_rate for asset
        let b_rate = b_rates.get(asset).unwrap_or(1_000_000_000_000);
        
        // Create reserve data structure
        let mut data_map = Map::new(&env);
        data_map.set(symbol_short!("b_rate"), b_rate);
        
        let mut result = Map::new(&env);
        result.set(symbol_short!("data"), data_map);
        
        result
    }
    
    // Get list of supported reserves
    pub fn get_reserve_list(env: Env) -> Vec<Address> {
        env.storage().instance().get(&KEY_RESERVES).unwrap()
    }
    
    // Claim emissions
    pub fn claim(_env: Env, _user: Address, _token_ids: Vec<u32>, _to: Address) -> i128 {
        // Return fixed emission amount for testing
        100_i128
    }
    
    // Update b_rate (test helper)
    pub fn update_b_rate(env: Env, asset: Address, new_rate: i128) {
        // Get b_rate map
        let mut b_rates: Map<Address, i128> = env.storage().instance().get(&KEY_BRATE).unwrap_or_else(|| {
            Map::new(&env)
        });
        
        // Update b_rate for asset
        b_rates.set(asset, new_rate);
        
        // Save updated b_rates
        env.storage().instance().set(&KEY_BRATE, &b_rates);
    }
}