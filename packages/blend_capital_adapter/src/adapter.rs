use soroban_sdk::{vec, Address, Env, IntoVal, Symbol, Val, Vec};
use crate::{
    artifacts::pool::{Client as PoolClient, ReserveData, ReserveConfig, Request, Reserve, PoolConfig}, constants::SCALAR_12, contract_types::RequestType, storage
};

pub fn create_request(
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

pub fn supply_collateral(
    e: &Env, 
    user: Address,
    asset: Address,
    amount: i128
) -> i128 {
    let yield_controller = storage::get_yield_controller(e);
    let pool_id: Address = storage::read_lend_pool_id(e);
    let pool_client = PoolClient::new(e, &pool_id);
    let request = create_request(RequestType::SupplyCollateral, asset.clone(), amount);
    let request_vec: Vec<Request> = vec![e, request];

    pool_client.submit( 
        &yield_controller.clone(), 
        &user,
        &yield_controller.clone(),
        &request_vec,
    );
    storage::store_deposit(e, &yield_controller, &asset, amount);
    amount
}

pub fn supply_collateral_auth(e: &Env, user: Address, asset: Address, amount: i128) -> (Address, Symbol, Vec<Val>) {
    let yield_controller = storage::get_yield_controller(e);
    let pool_id: Address = storage::read_lend_pool_id(e);
    let request = create_request(RequestType::SupplyCollateral, asset.clone(), amount);
    let request_vec: Vec<Request> = vec![e, request];
    (
        pool_id, 
        Symbol::new(&e, "submit"), 
        vec![
            e,
            (&yield_controller).into_val(e),
            (&user).into_val(e),
            (&yield_controller).into_val(e),
            (&request_vec).into_val(e),
        ]
    )
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
    let yield_controller = storage::get_yield_controller(e);
    let request_vec: Vec<Request> = vec![e, request];
    
    pool_client.submit(
        &yield_controller.clone(), 
        &yield_controller.clone(),
        &user,
        &request_vec,
    );

    storage::remove_deposit(e, &yield_controller, &asset, amount);

    amount
}

pub fn withdraw_collateral_auth(e: &Env, user: Address, asset: Address, amount: i128) -> (Address, Symbol, Vec<Val>) {
    let pool_id: Address = storage::read_lend_pool_id(e);
    let request = create_request(RequestType::WithdrawCollateral, asset.clone(), amount);
    let yield_controller = storage::get_yield_controller(e);
    let request_vec: Vec<Request> = vec![e, request];
    (
        pool_id, 
        Symbol::new(&e, "submit"), 
        vec![
            e,
            (&yield_controller).into_val(e),
            (&yield_controller).into_val(e),
            (&user).into_val(e),
            (&request_vec).into_val(e),
        ]
    )
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

pub fn claim_auth(e: &Env, from: Address, to: Address, asset: Address) -> Option<(Address, Symbol, Vec<Val>)> {
    let pool_id: Address = storage::read_lend_pool_id(e);
    
    if let Some(reserve_token_id) = get_reserve_token_id(e, asset.clone()) {
        return Some(
            (
                pool_id, 
                Symbol::new(&e, "claim"), 
                vec![
                    e,
                    (&from).into_val(e),
                    (vec![e, reserve_token_id]).into_val(e), 
                    (&to).into_val(e),
                ]
            )
        )
    }
    None
}

pub fn read_yield(e: &Env, user: Address, asset: Address) -> i128 {
    let current_value = get_balance(e, user.clone(), asset.clone());
    let original_deposit = storage::get_deposit_amount(e, &user, &asset);
    if original_deposit == 0 || current_value <= original_deposit {
        return 0;
    }
    current_value - original_deposit
}

pub fn get_apy(e: &Env, asset: Address) -> u32 {
    let pool_id = storage::read_lend_pool_id(e);
    let pool_client = PoolClient::new(e, &pool_id);
    
    let reserve = pool_client.get_reserve(&asset);
    
    // Calculate utilization rate
    let utilization = calculate_utilization(&reserve);
    
    // Calculate borrow rate using Blend's kinked rate model
    let borrow_rate = calculate_borrow_rate(&reserve, utilization);
    
    // Calculate supply rate (what lenders earn)
    let supply_rate = calculate_supply_rate(borrow_rate, utilization);
    
    // Convert annual rate to APY (basis points)
    rate_to_apy(supply_rate)
}

fn calculate_utilization(reserve: &Reserve) -> i128 {
    if reserve.data.b_supply == 0 {
        return 0;
    }
    
    // Calculate actual supplied amount using b_rate
    let total_supplied = (reserve.data.b_supply * reserve.data.b_rate) / SCALAR_12;
    
    if total_supplied == 0 {
        return 0;
    }
    
    // Calculate actual borrowed amount using d_rate
    let total_borrowed = (reserve.data.d_supply * reserve.data.d_rate) / SCALAR_12;
    
    // Utilization as a fraction with 12 decimal places
    (total_borrowed * SCALAR_12) / total_supplied
}

fn calculate_borrow_rate(reserve: &Reserve, utilization: i128) -> i128 {
    // Blend's kinked rate model with 4 segments
    let util_threshold = (reserve.config.util as i128) * SCALAR_12 / 10_000_000; // Convert 7 decimals to 12
    let max_util_threshold = (reserve.config.max_util as i128) * SCALAR_12 / 10_000_000;
    
    // Apply interest rate modifier
    let ir_mod = reserve.data.ir_mod as i128;
    
    let base_rate = if utilization <= util_threshold {
        // Segment 1: 0 to target utilization
        let r_base = (reserve.config.r_base as i128) * SCALAR_12 / 10_000_000;
        let r_one = (reserve.config.r_one as i128) * SCALAR_12 / 10_000_000;
        
        if util_threshold > 0 {
            r_base + ((utilization * (r_one - r_base)) / util_threshold)
        } else {
            r_base
        }
        
    } else if utilization <= max_util_threshold {
        // Segment 2: target to max utilization
        let r_one = (reserve.config.r_one as i128) * SCALAR_12 / 10_000_000;
        let r_two = (reserve.config.r_two as i128) * SCALAR_12 / 10_000_000;
        
        if max_util_threshold > util_threshold {
            r_one + (((utilization - util_threshold) * (r_two - r_one)) / (max_util_threshold - util_threshold))
        } else {
            r_one
        }
        
    } else if utilization < SCALAR_12 {
        // Segment 3: above max utilization - steep increase
        let r_two = (reserve.config.r_two as i128) * SCALAR_12 / 10_000_000;
        let r_three = (reserve.config.r_three as i128) * SCALAR_12 / 10_000_000;
        
        r_two + (((utilization - max_util_threshold) * (r_three - r_two)) / (SCALAR_12 - max_util_threshold))
        
    } else {
        // Segment 4: 100% utilization
        (reserve.config.r_three as i128) * SCALAR_12 / 10_000_000
    };
    
    // Apply interest rate modifier
    (base_rate * ir_mod) / (10_000_000i128) // ir_mod is in 7 decimals
}

fn calculate_supply_rate(borrow_rate: i128, utilization: i128) -> i128 {
    // Supply rate = borrow_rate * utilization * (1 - reserve_factor)
    // For Blend, we assume no reserve factor (100% goes to suppliers)
    (borrow_rate * utilization) / SCALAR_12
}

fn rate_to_apy(annual_rate: i128) -> u32 {
    // Convert from 12-decimal rate to basis points (0.01%)
    // annual_rate is already annualized, so we just need to convert units
    
    // Convert to percentage (multiply by 100) then to basis points (multiply by 100 again)
    let apy_bps = (annual_rate * 10000) / SCALAR_12;
    
    // Cap at reasonable maximum (10000 basis points = 100%)
    if apy_bps > 10000 {
        10000
    } else if apy_bps < 0 {
        0
    } else {
        apy_bps as u32
    }
}
