use crate::constants::{
    ASSET, ASSET_EPOCH_PRINCIPAL, BLEND_TOKEN_ID, LENDING_POOL_ID, OWNER, USER_DEPOSITS, YIELD_CONTROLLER_ID
};
use crate::storage_types::AssetEpochPrincipal;
use crate::contract_types::AdapterError;
use soroban_sdk::{panic_with_error, Address, Env};
use yield_adapter::constants::{
    ADAPTER_INSTANCE_BUMP_AMOUNT, 
    ADAPTER_INSTANCE_LIFETIME_THRESHOLD
};

/// Only the yield cycle (claim, emissions, epoch update) and upgrades extend the instance TTL, so a member's
/// deposit or withdrawal never pays rent for the protocol. Code and instance share this TTL.
pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(
        ADAPTER_INSTANCE_LIFETIME_THRESHOLD,
        ADAPTER_INSTANCE_BUMP_AMOUNT,
    );
}

pub fn get_yield_controller(e: &Env) -> Address {
    e.storage().instance().get(&YIELD_CONTROLLER_ID).unwrap()
}

pub fn read_owner(e: &Env) -> Address {
    e.storage().instance().get(&OWNER).unwrap()
}

pub fn write_owner(e: &Env, owner: &Address) {
    e.storage().instance().set(&OWNER, owner);
}

pub fn store_deposit(e: &Env, user: &Address, asset: &Address, amount: i128) {
    let key = (USER_DEPOSITS, user.clone(), asset.clone());
    let current_amount = e.storage().instance().get(&key).unwrap_or(0_i128);

    e.storage().instance().set(&key, &(current_amount + amount));
}

pub fn read_deposit(e: &Env, user: &Address, asset: &Address) -> i128 {
    let key = (USER_DEPOSITS, user.clone(), asset.clone());
    e.storage().instance().get(&key).unwrap_or(0_i128)
}

pub fn remove_deposit(e: &Env, user: &Address, asset: &Address, amount: i128) {
    let key = (USER_DEPOSITS, user.clone(), asset.clone());
    let current_amount = e.storage().instance().get(&key).unwrap_or(0_i128);

    if amount >= current_amount {
        e.storage().instance().remove(&key);
    } else {
        e.storage().instance().set(&key, &(current_amount - amount));
    }
}

pub fn require_yield_controller(e: &Env) {
    let yield_controller_id: Address = get_yield_controller(e);
    yield_controller_id.require_auth()
}

pub fn read_lend_pool_id(e: &Env) -> Address {
    e.storage().instance().get(&LENDING_POOL_ID).unwrap()
}

pub fn read_blend_token_id(e: &Env) -> Address {
    e.storage().instance().get(&BLEND_TOKEN_ID).unwrap()
}

pub fn write_asset(e: &Env, asset: &Address) {
    e.storage().instance().set(&ASSET, asset);
}

pub fn read_asset(e: &Env) -> Option<Address> {
    e.storage().instance().get(&ASSET)
}

/// Deposits fail closed for anything but the bound asset, including on an adapter deployed before the
/// binding existed, until the owner calls `set_asset`.
pub fn require_asset(e: &Env, asset: &Address) {
    if read_asset(e).as_ref() != Some(asset) {
        panic_with_error!(e, AdapterError::UnsupportedAsset);
    }
}

/// What is owed to cUSD holders for `asset`: epoch principal plus deposits minus withdrawals since.
pub fn read_principal(e: &Env, asset: &Address) -> i128 {
    get_asset_epoch_principal(e, asset)
        .map(|r| r.principal + r.deposits_in_epoch - r.withdrawals)
        .unwrap_or(0)
}

pub fn get_asset_epoch_principal(e: &Env, asset: &Address) -> Option<AssetEpochPrincipal> {
    let key = (ASSET_EPOCH_PRINCIPAL, asset.clone());
    e.storage().instance().get(&key)
}

pub fn set_asset_epoch_principal(e: &Env, asset: &Address, epoch: u64, principal: i128) {
    let key = (ASSET_EPOCH_PRINCIPAL, asset.clone());
    let epoch_data = AssetEpochPrincipal {
        epoch,
        principal,
        withdrawals: 0,
        deposits_in_epoch: 0, // Reset deposits_in_epoch for new epoch
        last_updated: e.ledger().timestamp(),
    };
    
    e.storage().instance().set(&key, &epoch_data);
}

pub fn add_epoch_withdrawal(e: &Env, asset: &Address, amount: i128) {
    let key = (ASSET_EPOCH_PRINCIPAL, asset.clone());
    if let Some(mut epoch_data) = get_asset_epoch_principal(e, asset) {
        epoch_data.withdrawals += amount;
        epoch_data.last_updated = e.ledger().timestamp();
        e.storage().instance().set(&key, &epoch_data);
    }
}

pub fn add_epoch_deposit(e: &Env, asset: &Address, amount: i128) {
    let key = (ASSET_EPOCH_PRINCIPAL, asset.clone());
    if let Some(mut epoch_data) = get_asset_epoch_principal(e, asset) {
        // Track deposits made within the current epoch
        // Principal should only be updated during yield claims to include compounded yield
        epoch_data.deposits_in_epoch += amount;
        epoch_data.last_updated = e.ledger().timestamp();
        e.storage().instance().set(&key, &epoch_data);
    } else {
        // If no epoch data exists, create it with the deposit amount as initial principal
        let epoch_data = AssetEpochPrincipal {
            epoch: 0,
            principal: amount,
            withdrawals: 0,
            deposits_in_epoch: 0, // First deposit becomes the principal, not a within-epoch deposit
            last_updated: e.ledger().timestamp(),
        };
        e.storage().instance().set(&key, &epoch_data);
    }
}
