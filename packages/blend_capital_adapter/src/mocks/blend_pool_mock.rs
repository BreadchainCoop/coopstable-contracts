use crate::artifacts::pool::{Positions, Reserve, ReserveConfig, ReserveData, Request, UserEmissionData};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Map, Symbol, Vec};

// Define basic storage keys
const KEY_RESERVES: Symbol = symbol_short!("RES");
const KEY_USER_POS: Symbol = symbol_short!("USERPOS");
const KEY_B_RATE: Symbol = symbol_short!("BRATE");

#[contract]
pub struct PoolContract;

#[contractimpl]
impl PoolContract {
    pub fn init(env: Env, initial_asset: Address) -> () {
        let reserves = Vec::from_array(&env, [initial_asset]);
        env.storage().instance().set(&KEY_RESERVES, &reserves);
    }

    pub fn submit(
        env: Env,
        user: Address,
        _spender: Address,
        _to: Address,
        requests: Vec<Request>,
    ) -> Positions {
        let mut positions = Self::get_positions(env.clone(), user.clone());
        
        for request in requests.iter() {
            match request.request_type {
                2 => { // SupplyCollateral
                    // Find the reserve index for the asset
                    let reserve_list = Self::get_reserve_list(env.clone());
                    if let Some(idx) = reserve_list.iter().position(|a| a == request.address) {
                        let current = positions.collateral.get(idx as u32).unwrap_or(0);
                        positions.collateral.set(idx as u32, current + request.amount);
                    }
                }
                3 => { // WithdrawCollateral
                    // Find the reserve index for the asset
                    let reserve_list = Self::get_reserve_list(env.clone());
                    if let Some(idx) = reserve_list.iter().position(|a| a == request.address) {
                        let current = positions.collateral.get(idx as u32).unwrap_or(0);
                        positions.collateral.set(idx as u32, current - request.amount);
                    }
                }
                _ => {}
            }
        }
        
        // Store updated positions
        env.storage().instance().set(&(KEY_USER_POS, user.clone()), &positions);
        
        positions
    }

    pub fn submit_with_allowance(
        env: Env,
        user: Address,
        _spender: Address,
        _sender: Address,
        _requests: Vec<Request>,
    ) -> Positions {
        Positions {
            liabilities: Map::new(&env),
            collateral: Map::new(&env),
            supply: Map::new(&env),
        }
    }

    // Return user positions from storage
    pub fn get_positions(env: Env, user: Address) -> Positions {
        env.storage()
            .instance()
            .get(&(KEY_USER_POS, user))
            .unwrap_or_else(|| Positions {
                liabilities: Map::new(&env),
                collateral: Map::new(&env),
                supply: Map::new(&env),
            })
    }

    // Return a simple structure for reserve data with stored or default b_rate
    pub fn get_reserve(env: Env, asset: Address) -> Reserve {
        let b_rate = env.storage()
            .instance()
            .get(&(KEY_B_RATE, asset.clone()))
            .unwrap_or(1_000_000_000_000);
        
        let mut reserve = default_reserve(asset);
        reserve.data.b_rate = b_rate;
        reserve
    }

    // Return the simple reserves list
    pub fn get_reserve_list(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&KEY_RESERVES)
            .unwrap_or_else(|| Vec::new(&env))
    }

    // Return a fixed value for emissions
    pub fn claim(_env: Env, _user: Address, _token_ids: Vec<u32>, _to: Address) -> i128 {
        100
    }

    // Update b_rate helper to store the new rate
    pub fn update_b_rate(env: Env, asset: Address, new_rate: i128) {
        env.storage().instance().set(&(KEY_B_RATE, asset), &new_rate);
    }

    // Return mock user emission data
    pub fn get_user_emissions(_env: Env, _user: Address, _reserve_token_id: u32) -> Option<UserEmissionData> {
        Some(UserEmissionData {
            accrued: 100,
            index: 0,
        })
    }
}

pub(crate) fn default_reserve(asset: Address) -> Reserve {
    Reserve {
        asset: asset,
        config: ReserveConfig {
            decimals: 7,
            c_factor: 0_7500000,
            l_factor: 0_7500000,
            util: 0_7500000,
            max_util: 0_9500000,
            r_base: 0_0100000,
            r_one: 0_0500000,
            r_two: 0_5000000,
            r_three: 1_5000000,
            reactivity: 0_0000020,
            index: 0,
            supply_cap: 1000000000000000000,
            enabled: true,
        },
        data: ReserveData {
            b_rate: 1_000_000_000_000,
            d_rate: 1_000_000_000_000,
            ir_mod: 1_0000000,
            b_supply: 100_0000000,
            d_supply: 75_0000000,
            last_time: 0,
            backstop_credit: 0,
        },
        scalar: 1_0000000,
    }
}
