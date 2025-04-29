#![no_std]
use soroban_sdk::{ Address, Env};

pub trait LendingAdapter {
    fn deposit(env: &Env, user: Address, asset: Address, amount: i128) -> i128;
    
    fn withdraw(env: &Env, user: Address, asset: Address, amount: i128) -> i128;
    
    fn get_balance(env: &Env, user: Address, asset: Address) -> i128;
    
    fn get_yield(env: &Env, user: Address, asset: Address) -> i128;
}