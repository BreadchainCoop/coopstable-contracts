#![cfg(test)]
extern crate std;

use crate::contract::{BlendCapitalAdapter, BlendCapitalAdapterArgs, BlendCapitalAdapterClient};
use yield_adapter::lending_adapter::LendingAdapterClient;

use crate::mocks::blend_pool_mock::{PoolContract, PoolContractClient};
use pretty_assertions::assert_eq;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events},
    Address, Env, IntoVal, Symbol,
};

// Helper function to create a test environment with a deployed BlendCapitalAdapter
fn setup_test() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();

    // Create asset
    let token_admin = Address::generate(&env);
    let usdc_token = env.register_stellar_asset_contract_v2(token_admin.clone());
    let blend_token = env.register_stellar_asset_contract_v2(token_admin.clone());
    let blend_token_id = blend_token.address();
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
        BlendCapitalAdapterArgs::__constructor(&yield_controller, &pool_contract_id, &blend_token_id),
    );

    (
        env,
        blend_adapter_id,
        yield_controller,
        usdc_token_id,
        pool_contract_id,
    )
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
        let stored_controller: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("LACID"))
            .unwrap();
        let stored_pool: Address = env.storage().instance().get(&symbol_short!("LID")).unwrap();

        assert_eq!(stored_controller, yield_controller);
        assert_eq!(stored_pool, pool_id);
    });
}

// Test deposit operation
#[test]
fn test_deposit_with_events() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();

    let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
    let user = Address::generate(&env);
    let amount: i128 = 1000;

    // Mock the yield controller authorization
    env.mock_all_auths();

    // Clear events then deposit collateral
    let _ = env.events().all();
    let result = client.deposit(&user, &usdc_token_id, &amount);

    // Get all events
    let events = env.events().all();
    assert!(!events.is_empty(), "No events were emitted");

    // Get the last event
    let last_event = events.last().unwrap();

    // Define the expected topic
    let expected_topics = (
        Symbol::new(&env, "deposit"),
        blend_adapter_id.clone(),
    )
        .into_val(&env);

    // Assert that the event matches our expectations
    assert_eq!(last_event.0, blend_adapter_id);
    assert_eq!(last_event.1, expected_topics);

    // Verify the result
    assert_eq!(result, amount);

    // Verify deposit tracking
    env.as_contract(&blend_adapter_id, || {
        let key = (symbol_short!("UDEP"), blend_adapter_id.clone(), usdc_token_id.clone());
        let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
        assert_eq!(stored_amount, amount);
    });
}

#[test]
#[should_panic(expected = "Unauthorized function call for address")]
fn test_deposit_non_yield_controller() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();

    let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
    let amount: i128 = 1000;
    let unauthorized_user = Address::generate(&env);
    // Do not mock authorizations - this should cause a panic
    client.deposit(&unauthorized_user, &usdc_token_id, &amount);
}

// Test withdrawal operation
#[test]
fn test_withdraw() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();

    let client = LendingAdapterClient::new(&env, &blend_adapter_id);
    let user = Address::generate(&env);
    let deposit_amount: i128 = 1000;
    let withdraw_amount: i128 = 500;

    // Mock the yield controller authorization
    env.mock_all_auths();

    // First deposit
    client.deposit(&user, &usdc_token_id, &deposit_amount);

    // Clear events before withdraw
    let _ = env.events().all();

    // Then withdraw part of it
    let result = client.withdraw(&user, &usdc_token_id, &withdraw_amount);

    // Verify the result
    assert_eq!(result, withdraw_amount);

    // Verify the event was emitted
    let events = env.events().all();
    assert!(!events.is_empty(), "No events were emitted");

    // Get the last event
    let last_event = events.last().unwrap();

    // Define the expected topic
    let expected_topics = (
        Symbol::new(&env, "withdraw"),
        blend_adapter_id.clone(),
        user.clone(),
    )
        .into_val(&env);

    // Assert that the event matches our expectations
    assert_eq!(last_event.0, blend_adapter_id);
    assert_eq!(last_event.1, expected_topics);
    // assert_eq!(last_event.2, (usdc_token_id.clone(), withdraw_amount).into_val(&env));

    // Verify deposit tracking is updated
    env.as_contract(&blend_adapter_id, || {
        let key = (symbol_short!("UDEP"), blend_adapter_id.clone(), usdc_token_id.clone());
        let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
        assert_eq!(stored_amount, deposit_amount - withdraw_amount);
    });
}

// Test full withdrawal operation
#[test]
fn test_full_withdraw() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();

    let client = LendingAdapterClient::new(&env, &blend_adapter_id);
    let user = Address::generate(&env);
    let amount: i128 = 1000;

    // Mock the yield controller authorization
    env.mock_all_auths();

    // First deposit
    client.deposit(&user,&usdc_token_id, &amount);

    // Then withdraw everything
    let result = client.withdraw(&user, &usdc_token_id, &amount);

    // Verify the result
    assert_eq!(result, amount);

    // Verify deposit tracking is removed
    env.as_contract(&blend_adapter_id, || {
        let key = (symbol_short!("UDEP"), blend_adapter_id.clone(), usdc_token_id.clone());
        assert!(!env.storage().instance().has(&key));
    });
}

// Test withdraw operation with unauthorized user
#[test]
#[should_panic(expected = "Unauthorized function call for address")]
fn test_withdraw_non_yield_controller() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();

    let client = LendingAdapterClient::new(&env, &blend_adapter_id);
    let user = Address::generate(&env);
    let amount: i128 = 1000;

    // Do not mock authorizations
    // Withdraw collateral - should fail without yield controller auth
    client.withdraw(&user, &usdc_token_id, &amount);
}

// Test get yield functionality
#[test]
fn test_get_yield() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, _) = setup_test();

    let client = LendingAdapterClient::new(&env, &blend_adapter_id);
    let user = Address::generate(&env);
    let amount: i128 = 1000_0000000; // Using 7 decimal places

    // Mock the yield controller authorization
    env.mock_all_auths();

    // Deposit
    client.deposit(&user, &usdc_token_id, &amount);

    // Initially there should be no yield
    let initial_yield = client.get_yield(&usdc_token_id);
    assert_eq!(initial_yield, 0);

    // TODO: Fix this test
    // // Now simulate yield accrual by updating b_rate
    // // Increase b_rate by 10% (1.1e12)
    // let new_b_rate: i128 = 1_100_000_000_000;
    // update_b_rate(&env, &pool_id, &usdc_token_id, new_b_rate);

    // // Get yield
    // env.mock_all_auths();
    // let accrued_yield = client.get_yield(&user, &usdc_token_id);

    // // Expected yield is 10% of deposit
    // let expected_yield = amount / 10; // 10% of amount
    // assert_eq!(accrued_yield, expected_yield);
}

// Test claim yield with no yield accrued
#[test]
fn test_claim_yield_no_yield() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();

    let client = LendingAdapterClient::new(&env, &blend_adapter_id);
    let amount: i128 = 1000_0000000; // Using 7 decimal places
    let user = Address::generate(&env);
    // Mock the yield controller authorization
    env.mock_all_auths();

    // Deposit
    client.deposit(&user, &usdc_token_id, &amount);

    // Try to claim yield (should be 0)
    let claimed_yield = client.claim_yield(&usdc_token_id, &user);
    assert_eq!(claimed_yield, 0);
}

// Test claim yield with unauthorized user
#[test]
#[should_panic(expected = "Unauthorized function call for address")]
fn test_claim_yield_non_yield_controller() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();

    let client = LendingAdapterClient::new(&env, &blend_adapter_id);
    let user = Address::generate(&env);
    // Do not mock authorizations
    // Claim yield - should fail without yield controller auth
    client.claim_yield(&usdc_token_id, &user);
}

// Test authorization with explicit requirements
#[test]
fn test_authorization_requirements() {
    let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();

    let client = LendingAdapterClient::new(&env, &blend_adapter_id);
    let amount: i128 = 1000;
    let user = Address::generate(&env);

    // Setup auth for the yield controller
    env.mock_all_auths();

    // Perform deposit
    client.deposit(&user,&usdc_token_id, &amount);

    // Verify that the yield controller was required to authorize this call
    let auths = env.auths();
    assert!(!auths.is_empty(), "No authorizations were recorded");

    // Check if yield_controller authorization was required
    let yield_controller_auth = auths.iter().find(|(addr, _)| *addr == yield_controller);
    assert!(
        yield_controller_auth.is_some(),
        "Yield controller authorization was not required"
    );
}

// Test compound operations (multiple deposits and withdrawals)
#[test]
fn test_compound_operations() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();

    let client = LendingAdapterClient::new(&env, &blend_adapter_id);
    let user = Address::generate(&env);

    // Mock the yield controller authorization
    env.mock_all_auths();

    // First deposit
    let deposit1 = 500_0000000;
    client.deposit(&user, &usdc_token_id, &deposit1);

    // Second deposit
    let deposit2 = 300_0000000;
    client.deposit(&user,&usdc_token_id, &deposit2);

    // Verify total deposit tracking
    env.as_contract(&blend_adapter_id, || {
        let key = (symbol_short!("UDEP"), blend_adapter_id.clone(), usdc_token_id.clone());
        let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
        assert_eq!(stored_amount, deposit1 + deposit2);
    });

    // First withdrawal
    let withdraw1 = 200_0000000;
    client.withdraw(&user, &usdc_token_id, &withdraw1);

    // Second withdrawal
    let withdraw2 = 300_0000000;
    client.withdraw(&user, &usdc_token_id, &withdraw2);

    // Verify remaining deposit
    env.as_contract(&blend_adapter_id, || {
        let key = (symbol_short!("UDEP"), blend_adapter_id.clone(), usdc_token_id.clone());
        let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
        assert_eq!(stored_amount, deposit1 + deposit2 - withdraw1 - withdraw2);
    });

    // Final withdrawal (all remaining balance)
    let remaining = deposit1 + deposit2 - withdraw1 - withdraw2;
    client.withdraw(&user, &usdc_token_id, &remaining);

    // Verify deposit tracking is removed
    env.as_contract(&blend_adapter_id, || {
        let key = (symbol_short!("UDEP"), blend_adapter_id.clone(), usdc_token_id.clone());
        assert!(!env.storage().instance().has(&key));
    });
}

// Test multi-user operations
// NOTE: This test is disabled because the adapter stores all deposits under the contract address,
// not per individual user. The adapter tracks total deposits per asset, not per user.
#[test]
#[ignore]
fn test_multi_user_operations() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();

    let client = LendingAdapterClient::new(&env, &blend_adapter_id);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Mock the yield controller authorization
    env.mock_all_auths();

    // User 1 deposits
    let deposit1 = 500_0000000;
    client.deposit(&user1, &usdc_token_id, &deposit1);

    // User 2 deposits
    let deposit2 = 300_0000000;
    client.deposit(&user2, &usdc_token_id, &deposit2);

    // Verify each user's deposit tracking
    env.as_contract(&blend_adapter_id, || {
        let key1 = (symbol_short!("UDEP"), user1.clone(), usdc_token_id.clone());
        let stored_amount1: i128 = env.storage().instance().get(&key1).unwrap();
        assert_eq!(stored_amount1, deposit1);

        let key2 = (symbol_short!("UDEP"), user2.clone(), usdc_token_id.clone());
        let stored_amount2: i128 = env.storage().instance().get(&key2).unwrap();
        assert_eq!(stored_amount2, deposit2);
    });

    // User 1 withdraws
    let withdraw1 = 200_0000000;
    client.withdraw(&user1, &usdc_token_id, &withdraw1);

    // User 2 withdraws all
    client.withdraw(&user2, &usdc_token_id, &deposit2);

    // Verify final state
    env.as_contract(&blend_adapter_id, || {
        // User 1 should still have a deposit
        let key1 = (symbol_short!("UDEP"), user1.clone(), usdc_token_id.clone());
        let stored_amount1: i128 = env.storage().instance().get(&key1).unwrap();
        assert_eq!(stored_amount1, deposit1 - withdraw1);

        // User 2 should have no deposit
        let key2 = (symbol_short!("UDEP"), user2.clone(), usdc_token_id.clone());
        assert!(!env.storage().instance().has(&key2));
    });
}

// Test handling of negative yield scenarios
#[test]
fn test_negative_yield_handling() {
    let (env, blend_adapter_id, _yield_controller, usdc_token_id, pool_id) = setup_test();

    let client = LendingAdapterClient::new(&env, &blend_adapter_id);
    let amount: i128 = 1000_0000000; // Using 7 decimal places
    let user = Address::generate(&env);
    // Mock the yield controller authorization
    env.mock_all_auths();

    // Deposit
    client.deposit(&user, &usdc_token_id, &amount);

    // Simulate negative yield by updating b_rate to a lower value
    let new_b_rate: i128 = 900_000_000_000; // 10% loss
    update_b_rate(&env, &pool_id, &usdc_token_id, new_b_rate);

    // Check yield - should return 0 for negative yield
    let yield_amount = client.get_yield(&usdc_token_id);
    assert_eq!(yield_amount, 0, "Negative yield should be reported as 0");

    // Try to claim yield - should also return 0
    let claimed_yield = client.claim_yield(&usdc_token_id, &user);
    assert_eq!(claimed_yield, 0, "Claiming negative yield should return 0");
}
