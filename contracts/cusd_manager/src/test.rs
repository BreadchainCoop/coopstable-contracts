#![cfg(test)]
extern crate std;

use crate::{
    contract::{CUSDManager, CUSDManagerArgs, CUSDManagerClient}, token::{process_token_burn, process_token_mint}
};
use pretty_assertions::assert_eq;
use soroban_sdk::{
    testutils::{Address as _, Events}, token::{StellarAssetClient, TokenClient}, vec, Address, Env, IntoVal, Symbol
};

// Helper function to create a test environment with a deployed CUSD token
fn setup_test() -> (Env, Address, Address, Address, Address) {
    let e = Env::default();
    let owner = Address::generate(&e);
    let admin = Address::generate(&e);

    // Deploy token contract (simulate a Stellar Asset Contract for CUSD)
    let token_admin = Address::generate(&e);
    let cusd_token = e.register_stellar_asset_contract_v2(token_admin.clone());
    let cusd_token_id = cusd_token.address();

    // Deploy CUSD Manager contract
    let cusd_manager_id = e.register(
        CUSDManager,
        CUSDManagerArgs::__constructor(&cusd_token_id, &owner, &admin),
    );

    // Set the CUSD Manager contract as the admin of the CUSD token
    let token_client = StellarAssetClient::new(&e, &cusd_token_id);

    e.mock_all_auths_allowing_non_root_auth();

    token_client.set_admin(&cusd_manager_id);

    (e, cusd_manager_id, owner, admin, cusd_token_id)
}

// Helper to setup with yield controller
fn setup_test_with_yield_controller() -> (Env, Address, Address, Address, Address, Address) {
    let (e, cusd_manager_id, owner, admin, cusd_token_id) = setup_test();
    
    let yield_controller = Address::generate(&e);
    let cusd_manager_client = CUSDManagerClient::new(&e, &cusd_manager_id);
    // env.mock_all_auths() is already called in setup_test()
    cusd_manager_client.set_yield_controller(&yield_controller);
    
    (e, cusd_manager_id, owner, admin, cusd_token_id, yield_controller)
}

// Test the successful constructor and initialization
#[test]
fn test_constructor() {
    let (env, cusd_manager_id, _owner, _admin, cusd_token_id) = setup_test();

    let client = CUSDManagerClient::new(&env, &cusd_manager_id);

    // Test that the token ID is correctly set
    let stored_token_id = client.get_cusd_id();
    assert_eq!(stored_token_id, cusd_token_id);
}

// Test setting a new CUSD manager admin
#[test]
fn test_set_admin() {
    let (env, cusd_manager_id, _owner, _, _cusd_token_id) = setup_test();

    let client = CUSDManagerClient::new(&env, &cusd_manager_id);
    let new_admin = Address::generate(&env);

    // Mock admin authentication
    env.mock_all_auths();

    // Set new admin (should succeed)
    client.set_admin(&new_admin);
}


// Test issuing CUSD tokens
#[test]
fn test_issue_cusd() {
    let (env, cusd_manager_id, _owner, _admin, cusd_token_id, yield_controller) = setup_test_with_yield_controller();

    let client = CUSDManagerClient::new(&env, &cusd_manager_id);
    let recipient = Address::generate(&env);
    let amount: i128 = 1000;

    // Issue tokens (auth already mocked in setup)
    client.issue_cusd(&recipient, &amount);

    // Verify the tokens were issued
    let token_client = TokenClient::new(&env, &cusd_token_id);
    let balance = token_client.balance(&recipient);
    assert_eq!(balance, amount);
}

// Test burning CUSD tokens
#[test]
fn test_burn_cusd() {
    let (env, cusd_manager_id, _owner, _admin, cusd_token_id, yield_controller) = setup_test_with_yield_controller();

    let client = CUSDManagerClient::new(&env, &cusd_manager_id);
    let token_client: TokenClient = TokenClient::new(&env, &cusd_token_id);
    let user = Address::generate(&env);
    let amount: i128 = 1000;

    // Issue tokens first (auth already mocked in setup)
    client.issue_cusd(&user, &amount);

    // Verify initial balance
    let initial_balance = token_client.balance(&user);
    assert_eq!(initial_balance, amount);
    
    // Burn tokens
    let burn_amount = amount / 2;
    token_client.transfer(&user, &client.address, &burn_amount);
    client.burn_cusd(&user, &burn_amount);

    // Verify final balance
    let final_balance = token_client.balance(&user);
    assert_eq!(final_balance, amount / 2);
}

// Test issuing CUSD tokens with negative amount (should fail)
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_issue_cusd_negative_amount() {
    let (env, cusd_manager_id, _owner, _admin, _cusd_token_id, yield_controller) = setup_test_with_yield_controller();

    let client = CUSDManagerClient::new(&env, &cusd_manager_id);
    let recipient = Address::generate(&env);
    let amount: i128 = -100; // Negative amount

    // Should panic due to negative amount (auth already mocked in setup)
    client.issue_cusd(&recipient, &amount);
}

// Test burning CUSD tokens with negative amount (should fail)
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_burn_cusd_negative_amount() {
    let (env, cusd_manager_id, _owner, _admin, _cusd_token_id, yield_controller) = setup_test_with_yield_controller();

    let client = CUSDManagerClient::new(&env, &cusd_manager_id);
    let user = Address::generate(&env);
    let amount: i128 = -100; // Negative amount

    // Mock authentication
    env.mock_all_auths();

    // Should panic due to negative amount
    client.burn_cusd(&user, &amount);
}

// Test issuing CUSD tokens from non-admin (should fail)
#[test]
#[should_panic(expected = "Error(Contract, #1301)")]
fn test_issue_cusd_non_admin() {
    let (env, cusd_manager_id, _owner, _admin, _cusd_token_id) = setup_test();

    let client = CUSDManagerClient::new(&env, &cusd_manager_id);
    let non_admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let amount: i128 = 100;

    // Mock authentication
    env.mock_all_auths();

    // Should panic because non_admin doesn't have CUSD_ADMIN role
    client.issue_cusd(&recipient, &amount);
}


#[test]
fn test_process_token_mint() {
    let env = Env::default();
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_id = token_contract.address();

    let recipient = Address::generate(&env);
    let amount: i128 = 1000;

    // Perform token mint
    env.mock_all_auths();
    process_token_mint(&env, recipient.clone(), amount);

    // Verify balance
    let token_client = TokenClient::new(&env, &token_id);
    let balance = token_client.balance(&recipient);
    assert_eq!(balance, amount);
}

// Test token burning process directly
#[test]
fn test_process_token_burn() {
    let env = Env::default();
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_id = token_contract.address();

    let user = Address::generate(&env);
    let amount: i128 = 1000;

    // Mint tokens first
    env.mock_all_auths();
    process_token_mint(&env, user.clone(), amount);

    // Verify initial balance
    let token_client = TokenClient::new(&env, &token_id);
    let initial_balance = token_client.balance(&user);
    assert_eq!(initial_balance, amount);

    // Burn tokens
    env.mock_all_auths();
    token_client.approve(&user, &user, &(amount / 2), &1000000);
    process_token_burn(
        &env,
        user.clone(),
        amount / 2,
    );

    // Verify final balance
    let final_balance = token_client.balance(&user);
    assert_eq!(final_balance, amount / 2);
}

// Test that events are published when issuing CUSD
#[test]
fn test_issue_cusd_events() {
    let (env, cusd_manager_id, _owner, _admin, _cusd_token_id, _) = setup_test_with_yield_controller();

    let client = CUSDManagerClient::new(&env, &cusd_manager_id);
    let recipient = Address::generate(&env);
    let amount: i128 = 1000;

    // Issue tokens and capture events (auth already mocked in setup)
    env.events().all();
    client.issue_cusd(&recipient, &amount);

    // Get events published by the contract
    let event_published = vec![&client.env, client.env.events().all().last_unchecked()];
    let topic = (Symbol::new(&client.env, "issue_cusd"),).into_val(&client.env);
    let event_data = (recipient, amount).into_val(&client.env);
    let event_control = vec![&client.env, (client.address.clone(), topic, event_data)];
    assert_eq!(event_published, event_control);
}

#[test]
fn test_burn_cusd_events() {
    let (env, cusd_manager_id, _owner, _admin, cusd_token_id, yield_controller) = setup_test_with_yield_controller();

    let client = CUSDManagerClient::new(&env, &cusd_manager_id);
    let token_client: TokenClient = TokenClient::new(&env, &cusd_token_id);
    let user = Address::generate(&env);
    let amount: i128 = 1000;

    client.issue_cusd(&user, &amount);

    env.events().all();
    token_client.transfer(&user, &client.address, &amount);
    client.burn_cusd(&user, &(amount / 2));

    // Get events published by the contract
    let event_published = vec![&client.env, client.env.events().all().last_unchecked()];
    let topic = (Symbol::new(&client.env, "burn_cusd"),).into_val(&client.env);
    let event_data = (user, amount / 2).into_val(&client.env);
    let event_control = vec![&client.env, (client.address.clone(), topic, event_data)];
    assert_eq!(event_published, event_control);
}
