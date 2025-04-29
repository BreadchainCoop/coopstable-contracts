use soroban_sdk::{Address, Env};

pub trait LendingPoolAdapter {
    fn deposit(env: &Env, user: Address, asset: Address, amount: i128) -> i128; 
    
    fn withdraw(env: &Env, user: Address, asset: Address, amount: i128) -> i128;
    
    fn get_balance(env: &Env, user: Address, asset: Address) -> i128;
    
    fn get_yield(env: &Env, user: Address, asset: Address) -> i128;
}

// fn get_pool_id(env: &Env) -> Address;

// fn get_funding_controller_id(env: &Env) -> Address;
