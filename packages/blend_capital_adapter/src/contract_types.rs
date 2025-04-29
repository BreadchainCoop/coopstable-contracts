#![no_std]
use soroban_sdk::{Address, contracttype};

#[derive(Clone)]
#[repr(u32)]
#[contracttype]
pub enum RequestType {
    Deposit = 0,
    Withdraw = 1,
    SupplyCollateral = 2,
    WithdrawCollateral = 3,
    Borrow = 4,
    Repay = 5,
}


#[derive(Clone)]
#[contracttype]
pub struct Request {
    pub request_type: u32,
    pub address: Address,
    pub amount: i128,
}