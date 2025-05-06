#![cfg(test)]
extern crate std;

use crate::{
    contract::{
        BlendCapitalAdapter, 
        BlendCapitalAdapterArgs, 
        BlendCapitalAdapterClient,
    },
    contract_types::RequestType,
};
use yield_adapter::{
    lending_adapter::LendingAdapterClient,
    contract_types::SupportedAdapter,
};
use soroban_sdk::{
    testutils::{Address as _, Events},
    token::{Client as TokenClient, StellarAssetClient},  
    Address, Env, IntoVal, Symbol, vec, Vec, symbol_short
};
use pretty_assertions::assert_eq;

// Mock interface for Pool operations to simulate Blend Capital behavior
mod mock_pool {
    use super::*;
    
    pub struct PoolState {
        pub reserves: Vec<Address>,
        pub b_rates: Vec<i128>,
        pub positions: std::collections::HashMap<Address, (Vec<i128>, Vec<i128>)>, // (collateral, debt)
    }
    
    pub fn setup_mock_pool(e: &Env) -> (Address, PoolState) {
        // Setup initial pool state
        let pool_address = Address::generate(e);
        let state = PoolState {
            reserves: vec![e],
            b_rates: vec![e],
            positions: std::collections::HashMap::new(),
        };
        
        // Handle submit_with_allowance
        e.mock_all_auths();
        e.register(
            &pool_address, |e, contract_id, func, args: Vec<_>| {
            match func.to_string().as_str() {
                "submit_with_allowance" => {
                    let user: Address = args.get(0).unwrap().clone().into_val(e);
                    let _spender: Address = args.get(1).unwrap().clone().into_val(e);
                    let _sender: Address = args.get(2).unwrap().clone().into_val(e);
                    let requests: Vec<_> = args.get(3).unwrap().clone().into_val(e);
                    
                    let state = e.contract_data::<PoolState>(contract_id);
                    let mut positions = state.positions.clone();
                    
                    // Process each request
                    for req in requests.iter() {
                        let req_map: std::collections::HashMap<String, soroban_sdk::Val> = req.clone().into_val(e);
                        let req_type: u32 = req_map.get("request_type").unwrap().clone().into_val(e);
                        let asset: Address = req_map.get("address").unwrap().clone().into_val(e);
                        let amount: i128 = req_map.get("amount").unwrap().clone().into_val(e);
                        
                        // Find asset index
                        let mut asset_index = 0;
                        let reserves = state.reserves.clone();
                        for (i, reserve) in reserves.iter().enumerate() {
                            if *reserve == asset {
                                asset_index = i;
                                break;
                            }
                        }
                        
                        // Apply operation based on request type
                        if req_type == RequestType::SupplyCollateral as u32 {
                            // Add collateral
                            let user_entry = positions.entry(user.clone()).or_insert_with(|| {
                                let collateral = vec![e];
                                for _ in 0..state.reserves.len() {
                                    collateral.push_back(0);
                                }
                                let debt = vec![e];
                                for _ in 0..state.reserves.len() {
                                    debt.push_back(0);
                                }
                                (collateral, debt)
                            });
                            
                            // Convert amount to bTokens
                            let b_rate = state.b_rates.get(asset_index).unwrap_or(&1_000_000_000_000);
                            let b_tokens = (amount * 1_000_000_000_000) / b_rate;
                            
                            // Add to position
                            user_entry.0.set(asset_index, user_entry.0.get(asset_index).unwrap_or(0) + b_tokens);
                        } 
                        else if req_type == RequestType::WithdrawCollateral as u32 {
                            // Remove collateral
                            if let Some(user_entry) = positions.get_mut(&user) {
                                // Convert amount to bTokens
                                let b_rate = state.b_rates.get(asset_index).unwrap_or(&1_000_000_000_000);
                                let b_tokens = (amount * 1_000_000_000_000) / b_rate;
                                
                                // Remove from position
                                let current = user_entry.0.get(asset_index).unwrap_or(0);
                                user_entry.0.set(asset_index, if current > b_tokens { current - b_tokens } else { 0 });
                            }
                        }
                    }
                    
                    // Update state
                    let mut state = state.clone();
                    state.positions = positions;
                    e.set_contract_data::<PoolState>(contract_id, state);
                    
                    Ok(().into_val(e))
                },
                "get_positions" => {
                    let user: Address = args.get(0).unwrap().clone().into_val(e);
                    let state = e.contract_data::<PoolState>(contract_id);
                    
                    if let Some(position) = state.positions.get(&user) {
                        // Return user positions
                        Ok(std::collections::HashMap::from([
                            ("collateral".to_string(), position.0.clone().into_val(e)),
                            ("debt".to_string(), position.1.clone().into_val(e)),
                        ]).into_val(e))
                    } else {
                        // Return empty positions
                        let empty_vec = vec![e];
                        Ok(std::collections::HashMap::from([
                            ("collateral".to_string(), empty_vec.clone().into_val(e)),
                            ("debt".to_string(), empty_vec.into_val(e)),
                        ]).into_val(e))
                    }
                },
                "get_reserve" => {
                    let asset: Address = args.get(0).unwrap().clone().into_val(e);
                    let state = e.contract_data::<PoolState>(contract_id);
                    
                    // Find asset index
                    let mut asset_index = 0;
                    for (i, reserve) in state.reserves.iter().enumerate() {
                        if *reserve == asset {
                            asset_index = i;
                            break;
                        }
                    }
                    
                    // Return reserve data
                    let b_rate = state.b_rates.get(asset_index).unwrap_or(&1_000_000_000_000);
                    
                    Ok(std::collections::HashMap::from([
                        ("data".to_string(), std::collections::HashMap::from([
                            ("b_rate".to_string(), (*b_rate).into_val(e))
                        ]).into_val(e))
                    ]).into_val(e))
                },
                "get_reserve_list" => {
                    let state = e.contract_data::<PoolState>(contract_id);
                    Ok(state.reserves.clone().into_val(e))
                },
                "claim" => {
                    // Mock claiming emissions
                    let _user: Address = args.get(0).unwrap().clone().into_val(e);
                    let _token_ids: Vec<u32> = args.get(1).unwrap().clone().into_val(e);
                    let _to: Address = args.get(2).unwrap().clone().into_val(e);
                    
                    // Return claimed amount (always return 100 for testing)
                    Ok(100_i128.into_val(e))
                },
                _ => panic!("Unexpected function call: {}", func),
            }
        });
        
        // Initialize pool state
        e.set_contract_data(&pool_address, state.clone());
        
        (pool_address, state)
    }
    
    pub fn add_reserve(e: &Env, pool_address: &Address, asset: Address, b_rate: i128) {
        let mut state = e.contract_data::<PoolState>(pool_address);
        state.reserves.push_back(asset);
        state.b_rates.push_back(b_rate);
        e.set_contract_data(pool_address, state);
    }
    
    pub fn update_b_rate(e: &Env, pool_address: &Address, asset_index: usize, new_b_rate: i128) {
        let mut state = e.contract_data::<PoolState>(pool_address);
        state.b_rates.set(asset_index, new_b_rate);
        e.set_contract_data(pool_address, state);
    }
}

// Helper function to create a test environment with a deployed BlendCapitalAdapter
fn setup_test() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    
    // Deploy token contract (simulate a Stellar Asset Contract for USDC)
    let token_admin = Address::generate(&env);
    let usdc_token = env.register_stellar_asset_contract_v2(token_admin.clone());
    let usdc_token_id = usdc_token.address();
    
    // Setup mock Blend Capital pool
    let (pool_id, _) = mock_pool::setup_mock_pool(&env);
    mock_pool::add_reserve(&env, &pool_id, usdc_token_id.clone(), 1_000_000_000_000); // Initial 1:1 rate
    
    // Create the lending controller address
    let yield_controller = Address::generate(&env);
    
    // Deploy BlendCapitalAdapter contract
    env.mock_all_auths();
    let blend_adapter_id = env.register(
        BlendCapitalAdapter {},
        BlendCapitalAdapterArgs::__constructor(
            &yield_controller.clone(), 
            &pool_id.clone()
        )
    );
    
    (env, blend_adapter_id, yield_controller, usdc_token_id, pool_id)
}

// Test the successful constructor and initialization
#[test]
fn test_constructor() {
    let (e, blend_adapter_id, _yield_controller, _usdc_token_id, _pool_id) = setup_test();
    
    // Verify the contract was deployed
    assert!(e.storage().instance().has(&blend_adapter_id));
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
    
//     // Verify events
//     let events = env.events().all();
//     let mut deposit_event_found = false;
    
//     for event in events.iter() {
//         if let (contract_id, (topics, data)) = event {
//             if *contract_id == blend_adapter_id {
//                 if let soroban_sdk::Vec::Vec(topics_vec) = topics {
//                     if topics_vec.len() == 2 {
//                         let topic1: String = topics_vec[0].try_into_val(&env).unwrap_or_default();
//                         let topic2: String = topics_vec[1].try_into_val(&env).unwrap_or_default();
                        
//                         if topic1 == "BLEND_ADAPTER" && topic2 == "deposit" {
//                             let event_data: soroban_sdk::Val = data.clone();
                            
//                             // Tuple of (user, asset, amount)
//                             if let soroban_sdk::Val::Vec(event_data_vec) = event_data {
//                                 if event_data_vec.len() == 3 {
//                                     let event_user: Address = event_data_vec[0].clone().try_into_val(&env).unwrap();
//                                     let event_asset: Address = event_data_vec[1].clone().try_into_val(&env).unwrap();
//                                     let event_amount: i128 = event_data_vec[2].clone().try_into_val(&env).unwrap();
                                    
//                                     assert_eq!(event_user, user);
//                                     assert_eq!(event_asset, usdc_token_id);
//                                     assert_eq!(event_amount, amount);
                                    
//                                     deposit_event_found = true;
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
    
//     assert!(deposit_event_found, "Deposit event not found");
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
    
//     // Clear events
//     env.events().all();
    
//     // Now withdraw part of it
//     client.withdraw(&yield_controller, &user, &usdc_token_id, &withdraw_amount);
    
//     // Verify events
//     let events = env.events().all();
//     let mut withdraw_event_found = false;
    
//     for event in events.iter() {
//         if let (contract_id, (topics, data)) = event {
//             if *contract_id == blend_adapter_id {
//                 if let soroban_sdk::Vec::Vec(topics_vec) = topics {
//                     if topics_vec.len() == 2 {
//                         let topic1: String = topics_vec[0].try_into_val(&env).unwrap_or_default();
//                         let topic2: String = topics_vec[1].try_into_val(&env).unwrap_or_default();
                        
//                         if topic1 == "BLEND_ADAPTER" && topic2 == "withdraw" {
//                             let event_data: soroban_sdk::Val = data.clone();
                            
//                             // Tuple of (user, asset, amount)
//                             if let soroban_sdk::Val::Vec(event_data_vec) = event_data {
//                                 if event_data_vec.len() == 3 {
//                                     let event_user: Address = event_data_vec[0].clone().try_into_val(&env).unwrap();
//                                     let event_asset: Address = event_data_vec[1].clone().try_into_val(&env).unwrap();
//                                     let event_amount: i128 = event_data_vec[2].clone().try_into_val(&env).unwrap();
                                    
//                                     assert_eq!(event_user, user);
//                                     assert_eq!(event_asset, usdc_token_id);
//                                     assert_eq!(event_amount, withdraw_amount);
                                    
//                                     withdraw_event_found = true;
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
    
//     assert!(withdraw_event_found, "Withdraw event not found");
// }

// // Test getting yield
// #[test]
// fn test_get_yield() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let deposit_amount: i128 = 1000_000_000_000; // 1000 tokens
    
//     // First deposit some amount
//     env.mock_all_auths();
//     client.deposit(&yield_controller, &user, &usdc_token_id, &deposit_amount);
    
//     // No yield initially
//     let initial_yield = client.get_yield(&user, &usdc_token_id);
//     assert_eq!(initial_yield, 0);
    
//     // Simulate yield accrual by increasing the b_rate (10% increase)
//     mock_pool::update_b_rate(&env, &pool_id, 0, 1_100_000_000_000);
    
//     // Check yield after rate change
//     let accrued_yield = client.get_yield(&user, &usdc_token_id);
    
//     // Expected yield: 10% of 1000 = 100
//     assert_eq!(accrued_yield, 100_000_000_000);
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
//     mock_pool::update_b_rate(&env, &pool_id, 0, 1_100_000_000_000);
    
//     // Clear events
//     env.events().all();
    
//     // Claim yield
//     let claimed_yield = client.claim_yield(&yield_controller, &user, &usdc_token_id);
    
//     // Expected yield: 10% of 1000 = 100
//     assert_eq!(claimed_yield, 100_000_000_000);
    
//     // Verify events
//     let events = env.events().all();
//     let mut yield_claimed_event_found = false;
//     let mut emissions_claimed_event_found = false;
    
//     for event in events.iter() {
//         if let (contract_id, (topics, data)) = event {
//             if *contract_id == blend_adapter_id {
//                 if let soroban_sdk::Vec::Vec(topics_vec) = topics {
//                     if topics_vec.len() == 2 {
//                         let topic1: String = topics_vec[0].try_into_val(&env).unwrap_or_default();
//                         let topic2: String = topics_vec[1].try_into_val(&env).unwrap_or_default();
                        
//                         if topic1 == "BLEND_ADAPTER" {
//                             if topic2 == "yield_claimed" {
//                                 let event_data: soroban_sdk::Val = data.clone();
                                
//                                 // Tuple of (user, asset, amount)
//                                 if let soroban_sdk::Val::Vec(event_data_vec) = event_data {
//                                     if event_data_vec.len() == 3 {
//                                         let event_user: Address = event_data_vec[0].clone().try_into_val(&env).unwrap();
//                                         let event_asset: Address = event_data_vec[1].clone().try_into_val(&env).unwrap();
//                                         let event_amount: i128 = event_data_vec[2].clone().try_into_val(&env).unwrap();
                                        
//                                         assert_eq!(event_user, user);
//                                         assert_eq!(event_asset, usdc_token_id);
//                                         assert_eq!(event_amount, 100_000_000_000);
                                        
//                                         yield_claimed_event_found = true;
//                                     }
//                                 }
//                             } else if topic2 == "emissions_claimed" {
//                                 let event_data: soroban_sdk::Val = data.clone();
                                
//                                 // Tuple of (user, asset, amount)
//                                 if let soroban_sdk::Val::Vec(event_data_vec) = event_data {
//                                     if event_data_vec.len() == 3 {
//                                         let event_user: Address = event_data_vec[0].clone().try_into_val(&env).unwrap();
//                                         let event_asset: Address = event_data_vec[1].clone().try_into_val(&env).unwrap();
//                                         let event_amount: i128 = event_data_vec[2].clone().try_into_val(&env).unwrap();
                                        
//                                         assert_eq!(event_user, user);
//                                         assert_eq!(event_asset, usdc_token_id);
//                                         assert_eq!(event_amount, 100); // Mocked emission amount
                                        
//                                         emissions_claimed_event_found = true;
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
    
//     assert!(yield_claimed_event_found, "Yield claimed event not found");
//     assert!(emissions_claimed_event_found, "Emissions claimed event not found");
    
//     // Verify that yield has been reset
//     let after_claim_yield = client.get_yield(&user, &usdc_token_id);
//     assert_eq!(after_claim_yield, 0);
// }

// // Test authorization - only yield controller can call deposit
// #[test]
// #[should_panic(expected = "Unauthorized")]
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
// #[should_panic(expected = "Unauthorized")]
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
//     client.withdraw(&unauthorized, &user, &usdc_token_id, &amount);
// }

// // Test authorization - only yield controller can call claim_yield
// #[test]
// #[should_panic(expected = "Unauthorized")]
// fn test_unauthorized_claim_yield() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, pool_id) = setup_test();
    
//     let client = BlendCapitalAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000_000_000_000;
    
//     // First deposit some amount
//     env.mock_all_auths();
//     client.deposit(&yield_controller, &user, &usdc_token_id, &amount);
    
//     // Simulate yield accrual
//     mock_pool::update_b_rate(&env, &pool_id, 0, 1_100_000_000_000);
    
//     // Call claim_yield from unauthorized address
//     let unauthorized = Address::generate(&env);
//     client.claim_yield(&unauthorized, &user, &usdc_token_id);
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
    
//     // The stored deposit should track the net amount
//     let key = (symbol_short!("UDEP"), user.clone(), usdc_token_id.clone());
//     let stored_amount: i128 = env.storage().instance().get(&key).unwrap_or(0);
    
//     assert_eq!(stored_amount, 4000);
// }

// // Test LendingAdapter interface implementation
// #[test]
// fn test_lending_adapter_interface() {
//     let (env, blend_adapter_id, yield_controller, usdc_token_id, _pool_id) = setup_test();
    
//     // Use the LendingAdapterClient to verify the interface
//     let client = LendingAdapterClient::new(&env, &blend_adapter_id);
//     let user = Address::generate(&env);
//     let amount: i128 = 1000;
    
//     env.mock_all_auths();
    
//     // Test deposit through the interface
//     let deposit_result = client.deposit(&yield_controller, &user, &usdc_token_id, &amount);
//     assert_eq!(deposit_result, amount);
    
//     // Test withdraw through the interface
//     let withdraw_result = client.withdraw(&yield_controller, &user, &usdc_token_id, &amount);
//     assert_eq!(withdraw_result, amount);
    
//     // The interface is correctly implemented
//     assert_eq!(client.address(), blend_adapter_id);
// }