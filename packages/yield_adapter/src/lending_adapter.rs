use soroban_sdk::{contractclient, Address, Env, Symbol, Vec, Val};

#[contractclient(name = "LendingAdapterClient")]
pub trait LendingAdapter {
    fn deposit(env: &Env, user: Address, asset: Address, amount: i128) -> i128;

    fn deposit_auth(env: &Env, user: Address, asset: Address, amount: i128) -> Option<(Address, Symbol, Vec<Val>)>;
    
    fn withdraw(env: &Env, user: Address, asset: Address, amount: i128) -> i128;

    fn withdraw_auth(env: &Env, user: Address, asset: Address, amount: i128) -> Option<(Address, Symbol, Vec<Val>)>;

    fn get_yield(env: &Env, asset: Address) -> i128;

    fn claim_yield(env: &Env, asset: Address, amount: i128) -> i128;

    fn claim_yield_auth(env: &Env, asset: Address, amount: i128) -> Option<(Address, Symbol, Vec<Val>)>;

    fn claim_emissions(e: &Env, to: Address, asset: Address) -> i128;

    fn claim_emissions_auth(e: &Env, to: Address, asset: Address) -> Option<(Address, Symbol, Vec<Val>)>;

    fn get_apy(env: &Env, asset: Address) -> u32;
    
    fn get_emissions(e: &Env, asset: Address) -> i128;

    fn get_total_deposited(e: &Env, asset: Address) -> i128;
    
    fn protocol_token(e: &Env) -> Address;

    fn __constructor(e: Env, yield_controller: Address, lending_pool_id: Address, protocol_token_id: Address); 
}
