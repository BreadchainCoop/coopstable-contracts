use soroban_sdk::{Address, Env, Vec, vec, token};
use crate::{
    artifacts::pool::{Client as PoolClient, Request},
    storage,
    contract_types::RequestType,
    constants::SCALAR_12
};

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

fn approve_asset(e: &Env, spender: Address, asset: Address, amount: i128) {
    let token_client = token::TokenClient::new(e, &asset);
    let ledger_sequence = e.ledger().sequence();
    e.authorize_as_current_contract(vec![&e]);
    token_client.approve(&e.current_contract_address(), &spender, &amount, &(ledger_sequence+5*12));
}

pub fn supply_collateral(
    e: &Env, 
    asset: Address, 
    amount: i128
) -> i128 {
    
    // Get the lending pool
    let pool_id: Address = storage::read_lend_pool_id(e);
    let pool_client = PoolClient::new(e, &pool_id);
    let request = create_request(RequestType::SupplyCollateral, asset.clone(), amount);
    let request_vec: Vec<Request> = vec![e, request];
    
    approve_asset(e, pool_id, asset.clone(), amount);
    
    pool_client.submit_with_allowance(
        &e.current_contract_address(), // from - the adapter
        &e.current_contract_address(), // spender - the adapter
        &e.current_contract_address(), // to - the adapter (receives any returns)
        &request_vec,
    );

    storage::store_deposit(e, &e.current_contract_address(), &asset, amount);

    amount
}

pub fn withdraw_collateral(
    e: &Env, 
    user: Address, 
    asset: Address, 
    amount: i128
) -> i128 {
    let pool_id: Address = storage::read_lend_pool_id(e);
    let pool_client = PoolClient::new(e, &pool_id);
    let request = create_request(RequestType::WithdrawCollateral, asset.clone(), amount);

    let request_vec: Vec<Request> = vec![e, request];

    e.authorize_as_current_contract(vec![&e]);
    pool_client.submit_with_allowance(
        &e.current_contract_address(),
        &e.current_contract_address(),
        &user,
        &request_vec,
    );

    storage::remove_deposit(e, &e.current_contract_address(), &asset, amount);

    amount
}

pub fn get_balance(e: &Env, user: Address, asset: Address) -> i128 {
    let pool_id: Address = storage::read_lend_pool_id(e);
    let pool_client = PoolClient::new(e, &pool_id);

    // Get the user's positions and the reserve from the pool
    let positions = pool_client.get_positions(&user);
    let reserve = pool_client.get_reserve(&asset);

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

        if let Some(b_token_amount) = positions.collateral.get(idx) {

            return (b_token_amount * reserve.data.b_rate) / SCALAR_12;
        }
    }

    0
}

fn get_reserve_token_id(e: &Env, asset: Address) -> Option<u32> {

    let pool_id: Address = storage::read_lend_pool_id(e);
    let pool_client = PoolClient::new(e, &pool_id);
    
    let reserve_list = pool_client.get_reserve_list();
    for (i, addr) in reserve_list.iter().enumerate() {

        if addr == *&asset {
            return Some((i as u32) * 2 + 1);
        }
    }

    None
}

pub fn get_user_emissions(e: &Env, user: Address, asset: Address)
-> i128 {
    let pool_id: Address = storage::read_lend_pool_id(e);
    let pool_client = PoolClient::new(e, &pool_id);

    if let Some(reserve_token_id) = get_reserve_token_id(e, asset) {
        
        if let Some(user_emission_data) = pool_client.get_user_emissions(&user, &reserve_token_id) {

            return user_emission_data.accrued;
        }

        return 0;
    }
    0
}

pub fn claim(e: &Env, from: Address, to: Address, asset: Address) -> i128 {

    if let Some(reserve_token_id) = get_reserve_token_id(e, asset.clone()) {
        let pool_id: Address = storage::read_lend_pool_id(e);
        let pool_client = PoolClient::new(e, &pool_id);

        let reserve_token_ids = vec![e, reserve_token_id];
        
        let emission_amount = pool_client.claim(&from, &reserve_token_ids, &to);
        
        return emission_amount
    }
    
    0
}

pub fn read_yield(e: &Env, user: Address, asset: Address) -> i128 {
    let current_value = get_balance(e, user.clone(), asset.clone());

    let original_deposit = storage::get_deposit_amount(e, &user, &asset);

    if original_deposit == 0 || current_value <= original_deposit {
        return 0;
    }

    current_value - original_deposit
}