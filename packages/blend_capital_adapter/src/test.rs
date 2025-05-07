#![cfg(test)]
extern crate std;

use crate::{
    contract::{BlendCapitalAdapter, BlendCapitalAdapterArgs},
    contract_types::RequestType,
};
use yield_adapter::{
    lending_adapter::LendingAdapterClient,
    contract_types::SupportedAdapter,
};
use soroban_sdk::{
    testutils::Address as _ ,
    Address, Env, symbol_short, Vec, vec, Map, Val
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

// // Test deposit operation
// #[test]
// fn test_deposit() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000;
    
//     // Mock the yield controller authorization
//     env.mock_all_auths();
    
//     // Deposit collateral
//     let result = client.deposit(&yield_controller, &user, &usdc_token_id, &amount);
    
//     // Verify the result
//     assert_eq!(result, amount);
    
//     // Verify deposit tracking
//     env.as_contract(&blend_adapter_id, || {
//         let key = (symbol_short!("UDEP"), user.clone(), usdc_token_id.clone());
//         let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
//         assert_eq!(stored_amount, amount);
//     });
    
//     // Verify auth was required for yield controller
//     let auths = env.auths();
//     assert_eq!(auths.len(), 1);
    
//     // The first auth should be from the yield controller
//     assert_eq!(auths[0].0, yield_controller);
// }

// // Test withdraw operation
// #[test]
// fn test_withdraw() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let deposit_amount: i128 = 1000;
//     let withdraw_amount: i128 = 500;
    
//     // First deposit some amount
//     env.mock_all_auths();
//     client.deposit(&yield_controller, &user, &usdc_token_id, &deposit_amount);
    
//     // Clear auths
//     env.auths();
    
//     // Now withdraw part of it
//     client.withdraw(&yield_controller, &user, &usdc_token_id, &withdraw_amount);
    
//     // Verify deposit tracking after withdrawal
//     env.as_contract(&blend_adapter_id, || {
//         let key = (symbol_short!("UDEP"), user.clone(), usdc_token_id.clone());
//         let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
//         assert_eq!(stored_amount, deposit_amount - withdraw_amount);
//     });
    
//     // Verify events
//     let events = env.events().all();
//     let mut withdrawal_event_found = false;
    
//     for event in events.iter() {
//         if let (contract_id, (topics, _data)) = event {
//             if *contract_id == blend_adapter_id {
//                 // Check for BLEND_ADAPTER, withdraw event
//                 if let soroban_sdk::Vec::Vec(topics_vec) = topics {
//                     if topics_vec.len() >= 2 {
//                         let topic1: Result<Symbol, _> = Symbol::try_from_val(&env, &topics_vec.get_unchecked(0));
//                         let topic2: Result<Symbol, _> = Symbol::try_from_val(&env, &topics_vec.get_unchecked(1));
                        
//                         if let (Ok(t1), Ok(t2)) = (topic1, topic2) {
//                             if t1.to_string() == "BLEND_ADAPTER" && t2.to_string() == "withdraw" {
//                                 withdrawal_event_found = true;
//                                 break;
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
    
//     assert!(withdrawal_event_found, "Withdrawal event not found");
// }

// // Test getting yield
// #[test]
// fn test_get_yield() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let deposit_amount: i128 = 1000_000_000_000; // 1000 tokens * 10^12
    
//     // First deposit some amount
//     env.mock_all_auths();
//     client.deposit(&yield_controller, &user, &usdc_token_id, &deposit_amount);
    
//     // No yield initially (b_rate unchanged)
//     let initial_yield = client.get_yield(&user, &usdc_token_id);
//     assert_eq!(initial_yield, 0);
    
//     // Simulate yield accrual by increasing the b_rate (10% increase)
//     update_b_rate(&env, &pool_id, &usdc_token_id, 1_100_000_000_000);
    
//     // Check yield after rate change - should be 10% of deposit amount
//     let accrued_yield = client.get_yield(&user, &usdc_token_id);
//     assert_eq!(accrued_yield, 100_000_000_000); // 10% of 1000
// }

// // Test claiming yield
// #[test]
// fn test_claim_yield() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let deposit_amount: i128 = 1000_000_000_000; // 1000 tokens
    
//     // First deposit some amount
//     env.mock_all_auths();
//     client.deposit(&yield_controller, &user, &usdc_token_id, &deposit_amount);
    
//     // Simulate yield accrual by increasing the b_rate (10% increase)
//     update_b_rate(&env, &pool_id, &usdc_token_id, 1_100_000_000_000);
    
//     // Clear events
//     env.events().all();
    
//     // Claim yield
//     let claimed_yield = client.claim_yield(&yield_controller, &user, &usdc_token_id);
    
//     // Expected yield: 10% of 1000 = 100
//     assert_eq!(claimed_yield, 100_000_000_000);
    
//     // Verify yield was reset to zero after claiming
//     let remaining_yield = client.get_yield(&user, &usdc_token_id);
//     assert_eq!(remaining_yield, 0);
    
//     // Verify events were emitted
//     let events = env.events().all();
//     assert!(!events.is_empty(), "No events were emitted");
// }

// // Test authorization - only yield controller can call deposit
// #[test]
// #[should_panic(expected = "Error(Contract, ")]
// fn test_unauthorized_deposit() {
//     let (env, blend_adapter_id, _yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000;
    
//     // Call from unauthorized address
//     let unauthorized = Address::generate(&env);
//     env.mock_all_auths();
    
//     // Should panic
//     client.deposit(&unauthorized, &user, &usdc_token_id, &amount);
// }

// // Test authorization - only yield controller can call withdraw
// #[test]
// #[should_panic(expected = "Error(Contract, ")]
// fn test_unauthorized_withdraw() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000;
    
//     // First deposit some amount
//     env.mock_all_auths();
//     client.deposit(&yield_controller, &user, &usdc_token_id, &amount);
    
//     // Call withdraw from unauthorized address
//     let unauthorized = Address::generate(&env);
    
//     // Should panic
//     client.withdraw(&unauthorized, &user, &usdc_token_id, &amount);
// }

// // Test multiple deposits and withdrawals
// #[test]
// fn test_multiple_operations() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
    
//     env.mock_all_auths();
    
//     // Perform multiple deposits
//     client.deposit(&yield_controller, &user, &usdc_token_id, &1000);
//     client.deposit(&yield_controller, &user, &usdc_token_id, &2000);
//     client.deposit(&yield_controller, &user, &usdc_token_id, &3000);
    
//     // Perform partial withdrawals
//     client.withdraw(&yield_controller, &user, &usdc_token_id, &500);
//     client.withdraw(&yield_controller, &user, &usdc_token_id, &1500);
    
//     // Total deposited: 1000 + 2000 + 3000 = 6000
//     // Total withdrawn: 500 + 1500 = 2000
//     // Expected balance: 4000
    
//     // Verify deposit tracking
//     env.as_contract(&blend_adapter_id, || {
//         let key = (symbol_short!("UDEP"), user.clone(), usdc_token_id.clone());
//         let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
//         assert_eq!(stored_amount, 4000);
//     });
// }

// // Test complete withdrawal
// #[test]
// fn test_complete_withdrawal() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000;
    
//     // Deposit and then withdraw the entire amount
//     env.mock_all_auths();
//     client.deposit(&yield_controller, &user, &usdc_token_id, &amount);
//     client.withdraw(&yield_controller, &user, &usdc_token_id, &amount);
    
//     // Verify deposit tracking is removed
//     env.as_contract(&blend_adapter_id, || {
//         let key = (symbol_short!("UDEP"), user.clone(), usdc_token_id.clone());
//         let exists = env.storage().instance().has(&key);
//         assert!(!exists, "Deposit tracking should be removed after full withdrawal");
//     });
// }

// // Test using LendingAdapterClient interface
// #[test]
// fn test_lending_adapter_interface() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     // Use the adapter through the LendingAdapter interface
//     let lending_client = LendingAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000;
    
//     env.mock_all_auths();
    
//     // Deposit through interface
//     lending_client.deposit(&yield_controller, &user, &usdc_token_id, &amount);
    
//     // Verify through interface
//     let yield_amount = lending_client.get_yield(&user, &usdc_token_id);
//     assert_eq!(yield_amount, 0); // No yield yet
    
//     // Withdraw through interface
//     lending_client.withdraw(&yield_controller, &user, &usdc_token_id, &(amount / 2));
    
//     // Verify deposit tracking
//     env.as_contract(&blend_adapter_id, || {
//         let key = (symbol_short!("UDEP"), user.clone(), usdc_token_id.clone());
//         let stored_amount: i128 = env.storage().instance().get(&key).unwrap();
//         assert_eq!(stored_amount, amount / 2);
//     });
// }