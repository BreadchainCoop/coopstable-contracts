use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "LendingAdapterClient")]
pub trait LendingAdapter {
    fn deposit(env: &Env, user: Address, asset: Address, amount: i128) -> i128;

    fn withdraw(env: &Env, user: Address, asset: Address, amount: i128) -> i128;

    fn get_yield(env: &Env, asset: Address) -> i128;

    fn claim_yield(env: &Env, asset: Address, recipient: Address) -> i128;

    fn claim_emissions(e: &Env, to: Address, asset: Address) -> i128;

    fn get_emissions(e: &Env, from: Address, asset: Address) -> i128;

    fn protocol_token(e: &Env) -> Address;

    fn __constructor(e: Env, yield_controller: Address, lending_pool_id: Address, protocol_token_id: Address); 
}
