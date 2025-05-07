#![cfg(test)]
extern crate std;

use crate::contract::{BlendCapitalAdapter, BlendCapitalAdapterArgs, BlendCapitalAdapterClient};
use soroban_sdk::{
    testutils::Address as _ ,
    Address, Env, symbol_short,
};
use pretty_assertions::assert_eq;
use crate::blend_pool_mock::{PoolContractClient,PoolContract};

// Helper function to create a test environment with a deployed BlendCapitalAdapter
fn setup_test() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    
    // Create asset
    let token_admin = Address::generate(&env);
    let usdc_token = env.register_stellar_asset_contract_v2(token_admin.clone());
    let usdc_token_id = usdc_token.address();
    
    // Deploy mock Pool contract
    let pool_contract_id = env.register(PoolContract, ());
    let pool_client = PoolContractClient::new(&env, &pool_contract_id);
    
    // Initialize pool with USDC as initial asset
    pool_client.init(&usdc_token_id);
    
    // Create the lending controller address (yield controller)
    let yield_controller = Address::generate(&env);
    
    // Deploy BlendCapitalAdapter contract
    let blend_adapter_id = env.register(
        BlendCapitalAdapter,
        BlendCapitalAdapterArgs::__constructor(
            &yield_controller,
            &pool_contract_id
        )
    );
    
    (env, blend_adapter_id, yield_controller, usdc_token_id, pool_contract_id)
}

// Helper function to update b_rate in pool
fn update_b_rate(env: &Env, pool_id: &Address, asset: &Address, new_b_rate: i128) {
    let pool_client = PoolContractClient::new(env, pool_id);
    pool_client.update_b_rate(asset, &new_b_rate);
}

// Test the successful constructor and initialization
#[test]
fn test_constructor() {
    let (env, blend_adapter_id, yield_controller, _usdc_token_id, pool_id) = setup_test();
    
    // Verify the contract was initialized correctly
    env.as_contract(&blend_adapter_id, || {
        let stored_controller: Address = env.storage().instance().get(&symbol_short!("LACID")).unwrap();
        let stored_pool: Address = env.storage().instance().get(&symbol_short!("BID")).unwrap();
        
        assert_eq!(stored_controller, yield_controller);
        assert_eq!(stored_pool, pool_id);
    });
}

// Test deposit operation
#[test]
fn test_deposit() {
    let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
    let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
    let user = Address::generate(&env);
    let amount: i128 = 1000;
    
    // Mock the yield controller authorization
    env.mock_all_auths();
    
    // Deposit collateral
    let result = client.supply_collateral(&user, &usdc_token_id, &amount);
    
    // Verify the result
    assert_eq!(result, amount);
    
    // Verify deposit tracking
    env.as_contract(&blend_adapter_id, || {
        let key = (symbol_short!("UDEP"), user.clone(), usdc_token_id.clone());
        let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
        assert_eq!(stored_amount, amount);
    });
}
// Test deposit operation
#[test]
#[should_panic(expected = "Unauthorized function call for address")]
fn deposit_non_yield_controller() {
    let (env, blend_adapter_id, _, usdc_token_id, _pool_id) = setup_test();
    
    let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
    let user = Address::generate(&env);
    let amount: i128 = 1000;
    
    // Deposit collateral
    client.supply_collateral(&user, &usdc_token_id, &amount);
}

