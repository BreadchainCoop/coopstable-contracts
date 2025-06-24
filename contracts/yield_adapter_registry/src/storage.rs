use crate::storage_types::{
    YieldAdapterRegistryMap, 
    REGISTRY_BUMP_AMOUNT, 
    REGISTRY_LIFETIME_THRESHOLD,
    DataKey,
};
use soroban_sdk::{Address, Env, Symbol, Vec};

fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(REGISTRY_LIFETIME_THRESHOLD, REGISTRY_BUMP_AMOUNT);
}

pub fn read_admin(e: &Env) -> Address { read_address(e, &DataKey::Admin)}
pub fn read_owner(e: &Env) -> Address { read_address(e, &DataKey::Owner)}
pub fn write_admin(e: &Env, new_admin: Address) { write_address(e, &DataKey::Admin, &new_admin);}
pub fn write_owner(e: &Env, new_owner: Address) { write_address(e, &DataKey::Owner, &new_owner);}

fn read_address(e: &Env, key: &DataKey) -> Address {
    extend_instance(e);
    e.storage().instance().get(key).unwrap()  
}

fn write_address(e: &Env, key: &DataKey, address: &Address) {
    extend_instance(e);
    e.storage().instance().set(key, address); 
}

pub fn read_yield_adapter_registry(e: &Env, yield_type: Symbol) -> YieldAdapterRegistryMap {
    if let Some(registry_map) = e
        .storage()
        .persistent()
        .get::<Symbol, YieldAdapterRegistryMap>(&yield_type)
    {
        e.storage().persistent().extend_ttl(
            &yield_type,
            REGISTRY_LIFETIME_THRESHOLD,
            REGISTRY_BUMP_AMOUNT,
        );

        registry_map
    } else {
        YieldAdapterRegistryMap::new(e, yield_type)
    }
}

fn write_yield_adapter_registry(e: &Env, registry_map: YieldAdapterRegistryMap) {
    e.storage()
        .persistent()
        .set(&registry_map.yield_type, &registry_map);
    e.storage().persistent().extend_ttl(
        &registry_map.yield_type,
        REGISTRY_LIFETIME_THRESHOLD,
        REGISTRY_BUMP_AMOUNT,
    );
}

pub fn register_yield_adapter(e: &Env, yield_type: Symbol, protocol: Symbol, adapter_id: Address) {
    let mut registry_map: YieldAdapterRegistryMap =
        read_yield_adapter_registry(e, yield_type.clone());
    if !registry_map.contains_value(adapter_id.clone()) {
        registry_map.set_adapter(protocol, adapter_id.clone());
        write_yield_adapter_registry(e, registry_map);
    }
}

pub fn remove_yield_adapter(e: &Env, yield_type: Symbol, protocol: Symbol) {
    let mut registry_map = read_yield_adapter_registry(e, yield_type.clone());
    registry_map.remove(protocol);
    write_yield_adapter_registry(e, registry_map);
}

pub fn verify_if_yield_adapter_exists(e: &Env, yield_type: Symbol, protocol: Symbol) -> bool {
    let registry_map = read_yield_adapter_registry(e, yield_type.clone());
    registry_map.contains_key(protocol)
}

pub fn get_yield_adapter(e: &Env, yield_type: Symbol, protocol: Symbol) -> Address {
    let registry_map = read_yield_adapter_registry(e, yield_type.clone());
    registry_map.get_adapter(protocol)
}

pub fn support_asset(e: &Env, yield_type: Symbol, protocol: Symbol, asset: Address) {
    let mut registry_map = read_yield_adapter_registry(e, yield_type.clone());
    registry_map.support_asset(protocol, asset);
    write_yield_adapter_registry(e, registry_map);
}

pub fn remove_asset_support(e: &Env, yield_type: Symbol, protocol: Symbol, asset: Address) {
    let mut registry_map = read_yield_adapter_registry(e, yield_type.clone());
    registry_map.remove_asset_support(protocol, asset);
    write_yield_adapter_registry(e, registry_map);
}

pub fn is_asset_supported(e: &Env, yield_type: Symbol, protocol: Symbol, asset: Address) -> bool {
    let registry_map = read_yield_adapter_registry(e, yield_type.clone());
    registry_map.supports_asset(protocol, asset)
}

pub fn get_yield_adapters(e: &Env, yield_type: Symbol) -> Vec<Address> {
    let registry_map = read_yield_adapter_registry(e, yield_type.clone());
    registry_map.adapters()
}

pub fn get_yield_adapters_with_assets(e: &Env, yield_type: Symbol) -> Vec<(Address, Vec<Address>)> {
    let registry_map = read_yield_adapter_registry(e, yield_type.clone());
    registry_map.adapter_with_assets()
}

