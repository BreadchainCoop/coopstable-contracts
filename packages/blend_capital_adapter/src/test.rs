#![cfg(test)]
extern crate std;

use crate::contract::{BlendCapitalAdapter, BlendCapitalAdapterArgs, BlendCapitalAdapterClient};
use crate::contract_types::RequestType;

use soroban_sdk::{log, IntoVal};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    Address, Env, Symbol, vec, symbol_short,
};
use pretty_assertions::assert_eq;
use crate::blend_pool_mock::{PoolContractClient, PoolContract};
use yield_adapter::lending_adapter::LendingAdapterClient;

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

// Helper function to update b_rate in pool (to simulate yield accumulation)
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
    let (env, blend_adapter_id, _, usdc_token_id, _pool_id) = setup_test();
    
    let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
    let user = Address::generate(&env);
    let amount: i128 = 1000;
    
    // Mock the yield controller authorization
    env.mock_all_auths();

    // Deposit collateral
    let result = client.deposit(&user, &usdc_token_id, &amount);
    
    // Verify the result
    assert_eq!(result, amount);
    
    // Verify deposit tracking
    env.as_contract(&blend_adapter_id, || {
        let key = (symbol_short!("UDEP"), user.clone(), usdc_token_id.clone());
        let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
        assert_eq!(stored_amount, amount);
    });
    
    // Verify the event was emitted
    let topics = ( 
        Symbol::new(&client.env, "deposit"), 
        blend_adapter_id.clone(),
        user.clone() 
    );
    let event_data = (usdc_token_id.clone(), amount);
    let event = vec![
        &client.env, 
        (
            blend_adapter_id.clone(),
            topics.into_val(&client.env),
            event_data.into_val(&client.env)
        )
    ];
    log!(&client.env, "events for current e: {:?}", vec![&client.env, client.env.events().all()]);

    // let published_event = vec![&client.env, client.env.events().all().last_unchecked()];
    assert_eq!(
        vec![&env, env.events().all().last_unchecked()], 
        event
    );
}

// // Test deposit operation with unauthorized user
// #[test]
// #[should_panic(expected = "Unauthorized function call for address")]
// fn deposit_non_yield_controller() {
//     let (env, blend_adapter_id, _, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000;
    
//     // Do not mock authorizations
//     // Deposit collateral - should fail without yield controller auth
//     client.deposit(&user, &usdc_token_id, &amount);
// }

// // Test withdrawal operation
// #[test]
// fn test_withdraw() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let deposit_amount: i128 = 1000;
//     let withdraw_amount: i128 = 500;
    
//     // Mock the yield controller authorization
//     env.mock_all_auths();
    
//     // First deposit
//     client.deposit(&user, &usdc_token_id, &deposit_amount);
    
//     // Then withdraw part of it
//     let result = client.withdraw(&user, &usdc_token_id, &withdraw_amount);
    
//     // Verify the result
//     assert_eq!(result, withdraw_amount);
    
//     // Verify deposit tracking is updated
//     env.as_contract(&blend_adapter_id, || {
//         let key = (symbol_short!("UDEP"), user.clone(), usdc_token_id.clone());
//         let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
//         assert_eq!(stored_amount, deposit_amount - withdraw_amount);
//     });
    
//     // Verify the event was emitted
//     let events = env.events().all();
//     let expected_topic = (Symbol::new(&env, "BLEND_ADAPTER"), Symbol::new(&env, "withdraw"));
//     let expected_data = (user.clone(), usdc_token_id.clone(), withdraw_amount);
    
//     assert!(events.iter().any(|e| 
//         e.0 == blend_adapter_id && 
//         e.1 == expected_topic.into_val(&env) && 
//         e.2 == expected_data.into_val(&env)
//     ));
// }

// // Test full withdrawal operation
// #[test]
// fn test_full_withdraw() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000;
    
//     // Mock the yield controller authorization
//     env.mock_all_auths();
    
//     // First deposit
//     client.deposit(&user, &usdc_token_id, &amount);
    
//     // Then withdraw everything
//     let result = client.withdraw(&user, &usdc_token_id, &amount);
    
//     // Verify the result
//     assert_eq!(result, amount);
    
//     // Verify deposit tracking is removed
//     env.as_contract(&blend_adapter_id, || {
//         let key = (symbol_short!("UDEP"), user.clone(), usdc_token_id.clone());
//         assert!(!env.storage().instance().has(&key));
//     });
// }

// // Test withdraw operation with unauthorized user
// #[test]
// #[should_panic(expected = "Unauthorized function call for address")]
// fn withdraw_non_yield_controller() {
//     let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000;
    
//     // Do not mock authorizations
//     // Withdraw collateral - should fail without yield controller auth
//     client.withdraw(&user, &usdc_token_id, &amount);
// }

// // Test get balance functionality
// #[test]
// fn test_get_balance() {
//     let (env, blend_adapter_id, _yield_controller, usdc_token_id, pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000_0000000; // Using 7 decimal places
    
//     // Mock the yield controller authorization
//     env.mock_all_auths();
    
//     // Deposit
//     client.supply_collateral(&user, &usdc_token_id, &amount);
    
//     // Get balance
//     let balance = client.get_balance(&user, &usdc_token_id);
    
//     // Balance should equal deposit since b_rate is 1e12 initially
//     let expected_balance = amount;
//     assert_eq!(balance, expected_balance);
    
//     // Now simulate yield accrual by updating b_rate
//     // Increase b_rate by 10% (1.1e12)
//     let new_b_rate: i128 = 1_100_000_000_000;
//     update_b_rate(&env, &pool_id, &usdc_token_id, new_b_rate);
    
//     // Get balance again
//     let updated_balance = client.get_balance(&user, &usdc_token_id);
    
//     // Expected balance should be 10% higher
//     let expected_updated_balance = (amount * new_b_rate) / 1_000_000_000_000;
//     assert_eq!(updated_balance, expected_updated_balance);
// }

// // Test get yield functionality
// #[test]
// fn test_get_yield() {
//     let (env, blend_adapter_id, _yield_controller, usdc_token_id, pool_id) = setup_test();
    
//     let client = LendingAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000_0000000; // Using 7 decimal places
    
//     // Mock the yield controller authorization
//     env.mock_all_auths();
    
//     // Deposit
//     client.deposit(&user, &usdc_token_id, &amount);
    
//     // Initially there should be no yield
//     let initial_yield = client.get_yield(&user, &usdc_token_id);
//     assert_eq!(initial_yield, 0);
    
//     // Now simulate yield accrual by updating b_rate
//     // Increase b_rate by 10% (1.1e12)
//     let new_b_rate: i128 = 1_100_000_000_000;
//     update_b_rate(&env, &pool_id, &usdc_token_id, new_b_rate);
    
//     // Get yield
//     let accrued_yield = client.get_yield(&user, &usdc_token_id);
    
//     // Expected yield is 10% of deposit
//     let expected_yield = amount / 10; // 10% of amount
//     assert_eq!(accrued_yield, expected_yield);
// }

// // Test claim yield functionality
// #[test]
// fn test_claim_yield() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, pool_id) = setup_test();
    
//     let client = LendingAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000_0000000; // Using 7 decimal places
    
//     // Mock the yield controller authorization
//     env.mock_all_auths();
    
//     // Deposit
//     client.deposit(&user, &usdc_token_id, &amount);
    
//     // Now simulate yield accrual by updating b_rate
//     // Increase b_rate by 10% (1.1e12)
//     let new_b_rate: i128 = 1_100_000_000_000;
//     update_b_rate(&env, &pool_id, &usdc_token_id, new_b_rate);
    
//     // Claim yield
//     let claimed_yield = client.claim_yield(&user, &usdc_token_id);
    
//     // Expected yield is 10% of deposit
//     let expected_yield = amount / 10; // 10% of amount
//     assert_eq!(claimed_yield, expected_yield);
    
//     // Verify the event was emitted
//     let events = env.events().all();
//     let expected_topic = (Symbol::new(&env, "BLEND_ADAPTER"), Symbol::new(&env, "yield_claimed"));
//     let expected_data = (user.clone(), usdc_token_id.clone(), expected_yield);
    
//     assert!(events.iter().any(|e| 
//         e.0 == blend_adapter_id && 
//         e.1 == expected_topic.into_val(&env) && 
//         e.2 == expected_data.into_val(&env)
//     ));
    
//     // After claiming, get_yield should return 0
//     let post_claim_yield = client.get_yield(&user, &usdc_token_id);
//     assert_eq!(post_claim_yield, 0);
// }

// // Test claim yield with no yield accrued
// #[test]
// fn test_claim_yield_no_yield() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = LendingAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000_0000000; // Using 7 decimal places
    
//     // Mock the yield controller authorization
//     env.mock_all_auths();
    
//     // Deposit
//     client.deposit(&user, &usdc_token_id, &amount);
    
//     // Try to claim yield (should be 0)
//     let claimed_yield = client.claim_yield(&user, &usdc_token_id);
//     assert_eq!(claimed_yield, 0);
// }

// // Test claim yield with unauthorized user
// #[test]
// #[should_panic(expected = "Unauthorized function call for address")]
// fn claim_yield_non_yield_controller() {
//     let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = LendingAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
    
//     // Do not mock authorizations
//     // Claim yield - should fail without yield controller auth
//     client.claim_yield(&user, &usdc_token_id);
// }

// // Test get reserve token ID
// #[test]
// fn test_get_reserve_token_id() {
//     let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
    
//     // Get token ID for the USDC token (should be 1 since it's the first asset)
//     let token_id = client.get_reserve_token_id(&usdc_token_id);
//     assert_eq!(token_id, Some(1)); // index 0 * 2 + 1 = 1
    
//     // Get token ID for a non-existent asset
//     let non_existent = Address::generate(&env);
//     let invalid_token_id = client.get_reserve_token_id(&non_existent);
//     assert_eq!(invalid_token_id, None);
// }

// // Test authorization with explicit requirements
// #[test]
// fn test_authorization_requirements() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = LendingAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000;
    
//     // Mock authorization
//     env.mock_all_auths();
    
//     // Perform deposit
//     client.deposit(&user, &usdc_token_id, &amount);
    
//     // Verify that the yield controller was required to authorize this call
//     let auth = env.auths().get(0);
//     assert!(auth.is_some());
    
//     let (addr, _) = auth.unwrap();
//     assert_eq!(addr, yield_controller);
// }

// // Test create_request helper function
// #[test]
// fn test_create_request() {
//     let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let amount: i128 = 1000;
    
//     // Create a request
//     let request = client.create_request(&RequestType::SupplyCollateral, &usdc_token_id, &amount);
    
//     // Verify request properties
//     assert_eq!(request.request_type, RequestType::SupplyCollateral as u32);
//     assert_eq!(request.address, usdc_token_id);
//     assert_eq!(request.amount, amount);
// }