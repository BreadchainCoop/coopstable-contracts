use soroban_sdk::{vec, Address, Env, IntoVal, Symbol, Val, Vec};
use crate::{
    artifacts::pool::{Client as PoolClient, Request, Reserve}, constants::SCALAR_12, contract_types::RequestType, storage
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
    
    // Update epoch principal when new deposits are made
    storage::add_epoch_deposit(e, &asset, amount);
    
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
    
    // Track withdrawal within the current epoch
    storage::add_epoch_withdrawal(e, &asset, amount);

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
    // Note: user parameter is kept for interface compatibility but will always be yield_controller
    let current_value = get_balance(e, user.clone(), asset.clone());
    
    if let Some(epoch_data) = storage::get_asset_epoch_principal(e, &asset) {
        // Calculate effective principal including withdrawals and deposits within epoch
        // Principal + deposits_in_epoch - withdrawals = total amount that should not count as yield
        let effective_principal = epoch_data.principal + epoch_data.deposits_in_epoch - epoch_data.withdrawals;
        
        if effective_principal <= 0 || current_value <= effective_principal {
            return 0;
        }
        
        current_value - effective_principal
    } else {
        // Fallback to original deposit method for first epoch
        let original_deposit = storage::get_deposit_amount(e, &user, &asset);
        if original_deposit == 0 || current_value <= original_deposit {
            return 0;
        }
        current_value - original_deposit
    }
}

pub fn get_apy(e: &Env, asset: Address) -> u32 {
    let pool_id = storage::read_lend_pool_id(e);
    let pool_client = PoolClient::new(e, &pool_id);
    
    let reserve = pool_client.get_reserve(&asset);
    let pool_config = pool_client.get_config();
    
    let utilization = calculate_utilization(&reserve);
    
    // If utilization is 0, as done in Blend SDK
    if utilization == 0 {
        return 0;
    }
    
    let borrow_rate = calculate_borrow_rate(&reserve, utilization);
    
    let supply_rate = calculate_supply_rate(borrow_rate, utilization, pool_config.bstop_rate);
    
    supply_rate_to_apy(supply_rate)
}

pub fn calculate_utilization(reserve: &Reserve) -> i128 {
    if reserve.data.b_supply == 0 {
        return 0;
    }    
    
    let total_supplied = (reserve.data.b_supply * reserve.data.b_rate) / SCALAR_12;
    
    if total_supplied == 0 {
        return 0;
    }
    
    let total_borrowed = (reserve.data.d_supply * reserve.data.d_rate) / SCALAR_12;
    
    let util_scaled = total_borrowed * SCALAR_12;
    (util_scaled + total_supplied - 1) / total_supplied
}

fn calculate_borrow_rate(reserve: &Reserve, utilization: i128) -> i128 {
    let util_7_decimals = (utilization * 10_000_000) / SCALAR_12;
    let target_util = reserve.config.util as i128;
    let fixed_95_percent = 9_500_000i128; // 95% with 7 decimals
    let fixed_5_percent = 500_000i128;    // 5% with 7 decimals
    
    let ir_mod = reserve.data.ir_mod as i128;
    
    let base_rate = if util_7_decimals <= target_util {
        // segment 1: 0% to target utilization - linear interpolation
        let r_base = reserve.config.r_base as i128;
        let r_one = reserve.config.r_one as i128;
        
        if target_util > 0 {
            // linear interpolation: r_base + (utilization / target_util) * r_one
            r_base + ((util_7_decimals * r_one) / target_util)
        } else {
            r_base
        }
    } else if util_7_decimals <= fixed_95_percent {
        let r_base = reserve.config.r_base as i128;
        let r_one = reserve.config.r_one as i128;
        let r_two = reserve.config.r_two as i128;
        
        // base rate at target utilization
        let base_at_target = r_base + r_one;
        
        if fixed_95_percent > target_util {
            // linear interpolation from (target, base_at_target) to (95%, base_at_target + r_two)
            base_at_target + (((util_7_decimals - target_util) * r_two) / (fixed_95_percent - target_util))
        } else {
            base_at_target
        }
    } else {
        let r_base = reserve.config.r_base as i128;
        let r_one = reserve.config.r_one as i128;
        let r_two = reserve.config.r_two as i128;
        let r_three = reserve.config.r_three as i128;
        
        // rate at 95% utilization
        let rate_at_95 = r_base + r_one + r_two;
        
        // additional rate based on how far above 95% we are
        let util_above_95 = util_7_decimals - fixed_95_percent;
        let additional_rate = (util_above_95 * r_three) / fixed_5_percent;
        
        rate_at_95 + additional_rate
    };
    
    // Apply interest rate modifier with correct decimal handling
    (base_rate * ir_mod) / 10_000_000i128
}

fn calculate_supply_rate(borrow_rate: i128, utilization: i128, bstop_rate: u32) -> i128 {
    let util_7_decimals = (utilization * 10_000_000) / SCALAR_12;

    let backstop_take_rate = bstop_rate as i128;
    let net_capture_rate = 10_000_000 - backstop_take_rate;

    let supply_rate_numerator = borrow_rate * net_capture_rate * util_7_decimals;
    let supply_rate = supply_rate_numerator / (10_000_000 * 10_000_000);
    
    supply_rate
}

fn supply_rate_to_apy(annual_rate: i128) -> u32 {
    let apr_bps = annual_rate / 1000;
    
    if apr_bps <= 0 {
        return 0;
    }

    let apr_squared = (apr_bps * apr_bps) / 10000; // APR^2 in basis points
    let compounding_adjustment = (apr_squared * 51) / 104; // (APR^2 * 51) / (2 * 52)
    let apy_bps = apr_bps + compounding_adjustment;
    
    if apy_bps > 10000 {
        10000
    } else if apy_bps < 0 {
        0
    } else {
        apy_bps as u32
    }
}
