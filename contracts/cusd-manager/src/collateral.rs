use crate::storage_types::{CollateralBalanceDataKey, DataKey, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{Address, Env};

pub fn read_collateral_balance(e: &Env, addr: Address, asset_address: Address) -> i128 {
    let key = DataKey::CollateralBalance(CollateralBalanceDataKey{
        asset: asset_address,
        owner: addr,
    });
    if let Some(balance) = e.storage().persistent().get::<DataKey, i128>(&key) {
        e.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        balance
    } else {
        0
    }
}

fn write_collateral_balance(e: &Env, addr: Address, asset_address: Address, amount: i128) {
    let key = DataKey::CollateralBalance(CollateralBalanceDataKey { asset: asset_address, owner: addr });
    e.storage().persistent().set(&key, &amount);
    e.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn receive_collateral_balance(e: &Env, owner: Address, asset_address: Address, amount: i128) {
    let balance = read_collateral_balance(e, owner.clone(), asset_address.clone());
    write_collateral_balance(e, owner.clone(), asset_address.clone(), balance + amount);
}

pub fn spend_balance(e: &Env, addr: Address, asset_address: Address, amount: i128) {
    let balance = read_collateral_balance(e, addr.clone(), asset_address.clone());
    if balance < amount {
        panic!("insufficient balance");
    }
    write_collateral_balance(e, addr.clone(), asset_address.clone(), balance - amount);
}
