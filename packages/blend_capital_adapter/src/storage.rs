use crate::constants::{
    LENDING_POOL_ID, 
    USER_DEPOSITS, 
    YIELD_CONTROLLER_ID,
};
use soroban_sdk::{
    Address, 
    Env
};
use yield_adapter::constants::{
    ADAPTER_INSTANCE_BUMP_AMOUNT, 
    ADAPTER_INSTANCE_LIFETIME_THRESHOLD
};

fn get_yield_controller(e: &Env) -> Address {
    e.storage().instance().extend_ttl(
        ADAPTER_INSTANCE_LIFETIME_THRESHOLD,
        ADAPTER_INSTANCE_BUMP_AMOUNT,
    );

    e.storage().instance().get(&YIELD_CONTROLLER_ID).unwrap()
}

pub fn store_deposit(e: &Env, user: &Address, asset: &Address, amount: i128) {
    e.storage().instance().extend_ttl(
        ADAPTER_INSTANCE_LIFETIME_THRESHOLD,
        ADAPTER_INSTANCE_BUMP_AMOUNT,
    );

    let key = (USER_DEPOSITS, user.clone(), asset.clone());
    let current_amount = e.storage().instance().get(&key).unwrap_or(0_i128);

    e.storage().instance().set(&key, &(current_amount + amount));
}

pub fn remove_deposit(e: &Env, user: &Address, asset: &Address, amount: i128) {
    e.storage().instance().extend_ttl(
        ADAPTER_INSTANCE_LIFETIME_THRESHOLD,
        ADAPTER_INSTANCE_BUMP_AMOUNT,
    );

    let key = (USER_DEPOSITS, user.clone(), asset.clone());
    let current_amount = e.storage().instance().get(&key).unwrap_or(0_i128);

    if amount >= current_amount {
        e.storage().instance().remove(&key);
    } else {
        e.storage().instance().set(&key, &(current_amount - amount));
    }
}

pub fn get_deposit_amount(e: &Env, user: &Address, asset: &Address) -> i128 {
    let key = (USER_DEPOSITS, user.clone(), asset.clone());
    e.storage().instance().get(&key).unwrap_or(0_i128)
}

pub fn require_yield_controller(e: &Env) {
    let yield_controller_id: Address = get_yield_controller(e);
    yield_controller_id.require_auth()
}

pub fn read_lend_pool_id(e: &Env) -> Address {
    e.storage().instance().get(&LENDING_POOL_ID).unwrap()
}