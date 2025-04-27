use crate::storage_types::{DataKey, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{Address, Env, Vec};

fn read_supported_assets(e: &Env) -> Vec<Address> {
    let key = DataKey::SupportedAssets;
    if let Some(assets) = e.storage().persistent().get::<DataKey, Vec<Address>>(&key) {
        e.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        assets
    } else {
        Vec::new(e)
    }
}

fn write_supported_assets(e: &Env, assets: Vec<Address>) {
    let key = DataKey::SupportedAssets;
    e.storage().persistent().set(&key, &assets);
    e.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn add_asset(e: &Env, asset_address: &Address) {
    let mut assets: Vec<Address> = read_supported_assets(e);
    if !assets.contains(asset_address) {
        assets.push_back(asset_address.clone());
        write_supported_assets(e, assets);
    }
}

pub fn remove_asset(e: &Env, asset_address: &Address) {
    let assets = read_supported_assets(e);
    let mut new_assets = Vec::new(e);
    
    for asset in assets.iter() {
        if &asset != asset_address {
            new_assets.push_back(asset);
        }
    }
    
    write_supported_assets(e, new_assets);
}

pub fn verify_if_supported_asset(e: &Env, asset_address: &Address) -> bool {
    let assets = read_supported_assets(e);
    assets.contains(asset_address)
}