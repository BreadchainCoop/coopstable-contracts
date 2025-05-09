use crate::artifacts::pool::{Positions, Reserve, ReserveConfig, ReserveData};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Map, Symbol, Vec};

// Define basic storage keys
const KEY_RESERVES: Symbol = symbol_short!("RES");

#[contract]
pub struct PoolContract;

#[contractimpl]
impl PoolContract {
    pub fn init(env: Env, initial_asset: Address) -> () {
        let reserves = Vec::from_array(&env, [initial_asset]);
        env.storage().instance().set(&KEY_RESERVES, &reserves);
    }

    pub fn submit_with_allowance(
        env: Env,
        user: Address,
        _spender: Address,
        _sender: Address,
        _requests: Vec<crate::contract_types::Request>,
    ) -> Positions {
        Positions {
            liabilities: Map::new(&env),
            collateral: Map::new(&env),
            supply: Map::new(&env),
        }
    }

    // Return a simple empty map for positions
    pub fn get_positions(env: Env, _user: Address) -> Positions {
        Positions {
            liabilities: Map::new(&env),
            collateral: Map::new(&env),
            supply: Map::new(&env),
        }
    }

    // Return a simple structure for reserve data with a default b_rate
    pub fn get_reserve(asset: Address) -> Reserve {
        default_reserve(asset)
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

    // Update b_rate helper does nothing in this simplified version
    pub fn update_b_rate(_env: Env, _asset: Address, _new_rate: i128) {
        // Do nothing
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
