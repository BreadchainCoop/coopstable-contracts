use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "LendingAdapterClient")]
pub trait LendingAdapter {
    fn deposit(env: &Env, user: Address, asset: Address, amount: i128) -> i128;

    fn withdraw(env: &Env, user: Address, asset: Address, amount: i128) -> i128;

    fn get_yield(env: &Env, user: Address, asset: Address) -> i128;

    fn claim_yield(env: &Env, user: Address, asset: Address) -> i128;

    fn claim_emissions(e: &Env, to: Address, asset: Address) -> i128;

    fn __constructor(e: Env, lending_adapter_controller_id: Address, lending_pool_id: Address);
}
