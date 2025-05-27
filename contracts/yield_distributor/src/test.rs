#![cfg(test)]
extern crate std;

use pretty_assertions::assert_eq;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger as _},
    vec, Address, Env, IntoVal, Symbol,
};

use crate::contract::{YieldDistributor, YieldDistributorArgs, YieldDistributorClient};

// Helper function to create a test environment
fn setup_test() -> (
    Env,
    YieldDistributorClient<'static>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let yield_controller = Address::generate(&env);

    // Initial configuration
    let treasury_share_bps: u32 = 1000; // 10%
    let distribution_period: u64 = 2592000; // 30 days in seconds

    // Deploy contract
    let contract_id = env.register(
        YieldDistributor,
        YieldDistributorArgs::__constructor(
            &treasury,
            &treasury_share_bps,
            &yield_controller,
            &distribution_period,
            &admin,
            &admin,
        ),
    );

    let client = YieldDistributorClient::new(&env, &contract_id);

    (env, client, admin, treasury, yield_controller)
}

#[test]
fn test_constructor() {
    let (_, client, _admin, treasury, _yield_controller) = setup_test();

    // Verify the contract was initialized correctly
    let stored_treasury = client.get_treasury();
    assert_eq!(stored_treasury, treasury);

    let treasury_share = client.get_treasury_share();
    assert_eq!(treasury_share, 1000);

    let distribution_period = client.get_distribution_period();
    assert_eq!(distribution_period, 2592000);
}

#[test]
fn test_member_management() {
    let (env, client, admin, _treasury, _yield_controller) = setup_test();

    // Create test members
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);

    // Mock admin auth
    env.mock_all_auths();

    // Add members
    client.add_member(&admin, &member1);
    client.add_member(&admin, &member2);

    // Verify members were added
    let members = client.list_members();
    assert_eq!(members.len(), 2);
    assert!(members.iter().any(|m| m == member1));
    assert!(members.iter().any(|m| m == member2));

    // Remove a member
    client.remove_member(&admin, &member1);

    // Verify member was removed
    let members = client.list_members();
    assert_eq!(members.len(), 1);
    assert!(members.iter().any(|m| m == member2));
    assert!(!members.iter().any(|m| m == member1));
}

#[test]
fn test_configuration_updates() {
    let (env, client, admin, _treasury, _yield_controller) = setup_test();

    // Mock admin auth
    env.mock_all_auths();

    // Update treasury
    let new_treasury = Address::generate(&env);
    client.set_treasury(&admin, &new_treasury);

    // Verify treasury was updated
    let stored_treasury = client.get_treasury();
    assert_eq!(stored_treasury, new_treasury);

    // Update treasury share
    let new_share_bps: u32 = 2000; // 20%
    client.set_treasury_share(&admin, &new_share_bps);

    // Verify treasury share was updated
    let stored_share = client.get_treasury_share();
    assert_eq!(stored_share, new_share_bps);

    // Update distribution period
    let new_period: u64 = 1296000; // 15 days in seconds
    client.set_distribution_period(&admin, &new_period);

    // Verify distribution period was updated
    let stored_period = client.get_distribution_period();
    assert_eq!(stored_period, new_period);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_distribute_unauthorized() {
    let (env, client, admin, _treasury, _yield_controller) = setup_test();

    // Try to distribute from non-yield-controller account
    let token = Address::generate(&env);

    env.mock_all_auths();
    client.distribute_yield(&admin, &token, &1000);
}

// Token contract for testing
fn create_token(env: &Env) -> (Address, Address) {
    // Token admin address
    let admin = Address::generate(env);
    // Create stellar asset contract
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    (admin.clone(), token_id.address())
}

#[test]
fn test_distribution_timing() {
    let (env, client, admin, _, _) = setup_test();

    // Create token for distribution
    let (token_admin, token_address) = create_token(&env);

    // Add members
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);

    env.mock_all_auths();
    client.add_member(&admin, &member1);
    client.add_member(&admin, &member2);

    // Set distribution period to a shorter interval for testing
    let period: u64 = 600; // 10 minutes in seconds
    client.set_distribution_period(&admin, &period);

    // Distribution should not be available initially
    assert!(!client.is_distribution_available());

    // Check next distribution time is in the future
    let next_time = client.get_next_distribution_time();
    let current_time = env.ledger().timestamp();
    assert!(next_time > current_time);

    // Advance time past the distribution period
    env.ledger().set_timestamp(current_time + period + 10);

    // Now distribution should be available
    assert!(client.is_distribution_available());
}

#[test]
fn test_yield_distribution() {
    let (env, client, admin, treasury, yield_controller) = setup_test();

    // Create token for distribution
    let (_, token_address) = create_token(&env);

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);

    env.mock_all_auths();
    client.add_member(&admin, &member1);
    client.add_member(&admin, &member2);

    let period: u64 = 60;
    client.set_distribution_period(&admin, &period);

    let current_time = env.ledger().timestamp();
    env.ledger().set_timestamp(current_time + period + 10);

    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    let token_client = soroban_sdk::token::TokenClient::new(&env, &token_address);
    let amount = 10000_i128;
    token_admin_client.mint(&yield_controller, &amount);

    token_client.approve(&yield_controller, &client.address, &amount, &100);

    env.mock_all_auths();
    let _ = &client.env.events().all().last_unchecked();
    let result = client.distribute_yield(&yield_controller, &token_address, &amount);
    let treasury_amount = (amount * 1000) / 10000;
    let member_amount = (amount - treasury_amount) / 2;

    let published_event = vec![&client.env, client.env.events().all().last_unchecked()];
    let control_event = vec![
        &client.env,
        (
            client.address.clone(),
            (
                Symbol::new(&client.env, "distribute_yield"),
                token_client.address.clone(),
            )
                .into_val(&client.env),
            (
                amount,
                treasury_amount,
                vec![&client.env, member1.clone(), member2.clone()],
                member_amount,
            )
                .into_val(&client.env),
        ),
    ];
    assert_eq!(published_event, control_event);
    assert!(result);

    let treasury_balance = token_client.balance(&treasury);
    let member1_balance = token_client.balance(&member1);
    let member2_balance = token_client.balance(&member2);

    assert_eq!(treasury_balance, treasury_amount);
    assert_eq!(member1_balance, member_amount);
    assert_eq!(member2_balance, member_amount);
}

#[test]
fn test_distribution_no_members() {
    let (env, client, admin, treasury, yield_controller) = setup_test();

    // Create token for distribution
    let (_, token_address) = create_token(&env);

    // Set distribution period to a short interval
    let period: u64 = 60; // 1 minute in seconds
    env.mock_all_auths();
    client.set_distribution_period(&admin, &period);

    // Advance time past the distribution period
    let current_time = env.ledger().timestamp();
    env.ledger().set_timestamp(current_time + period + 10);

    // Mint tokens to the yield controller
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    let amount = 10000_i128;
    let token_client = soroban_sdk::token::TokenClient::new(&env, &token_address);
    token_admin_client.mint(&yield_controller, &amount);

    // Perform the distribution with no members
    env.mock_all_auths();
    token_client.approve(&yield_controller, &client.address, &amount, &100);
    let result = client.distribute_yield(&yield_controller, &token_address, &amount);

    // Distribution should fail (return false) because there are no members
    assert!(!result);

    // Treasury should not receive anything
    let treasury_balance = token_client.balance(&treasury);
    assert_eq!(treasury_balance, 0);
}

#[test]
fn test_distribution_after_member_removal() {
    let (env, client, admin, treasury, yield_controller) = setup_test();

    // Create token for distribution
    let (token_admin, token_address) = create_token(&env);

    // Add members
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);

    env.mock_all_auths();
    client.add_member(&admin, &member1);
    client.add_member(&admin, &member2);

    // Remove one member
    client.remove_member(&admin, &member1);

    // Set distribution period to a short interval
    let period: u64 = 60; // 1 minute in seconds
    client.set_distribution_period(&admin, &period);

    // Advance time past the distribution period
    let current_time = env.ledger().timestamp();
    env.ledger().set_timestamp(current_time + period + 10);

    // Mint tokens to the yield controller
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    let token_client = soroban_sdk::token::TokenClient::new(&env, &token_address);
    let amount = 10000_i128;
    token_admin_client.mint(&yield_controller, &amount);

    // Approve the distributor to spend the tokens
    token_client.approve(&yield_controller, &client.address, &amount, &100);

    // Perform the distribution
    env.mock_all_auths();
    let result = client.distribute_yield(&yield_controller, &token_address, &amount);

    // Distribution should succeed
    assert!(result);

    // Check balances - only member2 and treasury should receive tokens
    let treasury_share_bps = client.get_treasury_share();
    let treasury_amount = (amount * treasury_share_bps as i128) / 10000;
    let member_amount = amount - treasury_amount; // All to member2

    let treasury_balance = token_client.balance(&treasury);
    let member1_balance = token_client.balance(&member1);
    let member2_balance = token_client.balance(&member2);

    assert_eq!(treasury_balance, treasury_amount);
    assert_eq!(member1_balance, 0);
    assert_eq!(member2_balance, member_amount);
}

#[test]
fn test_sequential_distributions() {
    let (env, client, admin, treasury, yield_controller) = setup_test();

    // Create token for distribution
    let (token_admin, token_address) = create_token(&env);

    // Add members
    let member1 = Address::generate(&env);
    env.mock_all_auths();
    client.add_member(&admin, &member1);

    // Set distribution period to a short interval
    let period: u64 = 600; // 10 minutes
    client.set_distribution_period(&admin, &period);

    // First distribution
    let current_time = env.ledger().timestamp();
    env.ledger().set_timestamp(current_time + period + 10);

    let token_client = soroban_sdk::token::TokenClient::new(&env, &token_address);
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    let amount1 = 5000_i128;
    token_admin_client.mint(&yield_controller, &amount1);
    token_client.approve(&yield_controller, &client.address, &amount1, &100);

    env.mock_all_auths();
    let result1 = client.distribute_yield(&yield_controller, &token_address, &amount1);
    assert!(result1);

    // Try to distribute again immediately - should fail
    let amount2 = 3000_i128;
    token_admin_client.mint(&yield_controller, &amount2);
    token_client.approve(&yield_controller, &client.address, &amount2, &100);

    env.mock_all_auths();
    let result2 = client.distribute_yield(&yield_controller, &token_address, &amount2);
    assert!(!result2);

    // Advance time past another distribution period
    let new_time = env.ledger().timestamp();
    env.ledger().set_timestamp(new_time + period + 10);

    // Now distribution should succeed
    env.mock_all_auths();
    let result3 = client.distribute_yield(&yield_controller, &token_address, &amount2);
    assert!(result3);

    // Check total balances after both distributions
    let treasury_share_bps = client.get_treasury_share();
    let treasury_amount1 = (amount1 * treasury_share_bps as i128) / 10000;
    let treasury_amount2 = (amount2 * treasury_share_bps as i128) / 10000;
    let member_amount1 = amount1 - treasury_amount1;
    let member_amount2 = amount2 - treasury_amount2;

    let treasury_balance = token_client.balance(&treasury);
    let member1_balance = token_client.balance(&member1);

    assert_eq!(treasury_balance, treasury_amount1 + treasury_amount2);
    assert_eq!(member1_balance, member_amount1 + member_amount2);
}

#[test]
fn test_set_admin() {
    let (env, client, admin, treasury, yield_controller) = setup_test();

    // Mock admin auth
    env.mock_all_auths();
    let new_admin = Address::generate(&env);

    // Set new admin
    client.set_admin(&admin, &new_admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #1300)")]
fn test_set_admin_unauthorized() {
    let (env, client, admin, treasury, yield_controller) = setup_test();

    // Mock admin auth
    env.mock_all_auths();
    let new_admin = Address::generate(&env);

    // Set new admin
    client.set_admin(&new_admin, &new_admin);
}