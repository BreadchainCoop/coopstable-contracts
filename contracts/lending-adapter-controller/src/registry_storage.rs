use soroban_sdk::{Env, Address, Map};
use crate::storage_types::{
    DataKey,
    LendingAdapterRegistry, 
    REGISTRY_BUMP_AMOUNT, 
    REGISTRY_LIFETIME_THRESHOLD
};

pub fn read_lending_adapter_registry(e: &Env) -> LendingAdapterRegistry {
    let key = DataKey::LendingAdapterRegistry;
    if let Some(registry_map) = e.storage()
        .persistent().get::<DataKey, LendingAdapterRegistryMap>(&key) {
            e.storage()
                .persistent()
                .extend_ttl(&key, REGISTRY_BUMP_AMOUNT, REGISTRY_LIFETIME_THRESHOLD);
            
            registry_map
    } else {
        LendingAdapterRegistryMap::new(e)
    }
}

fn write_lending_adapter_registry(e: &Env, registry_map: LendingAdapterRegistryMap) {
    let key = DataKey::LendingAdapterRegistry;
    e.storage().persistent().set(&key, &registry_map);
    e.storage()
        .persistent()
        .extend_ttl(&key, REGISTRY_BUMP_AMOUNT, REGISTRY_LIFETIME_THRESHOLD);
}

pub fn register_lending_adapter(e: &Env, protocol: LendingAdapter, adapter_id: Address) {
    let mut registry_map: LendingAdapterRegistryMap = read_lending_adapter_registry(e);
    if !registry_map.contains_value(adapter_id) {
        registry_map.set(protocol, adapter_id);
        write_lending_adapter_registry(e, registry_map);
    }
}

pub fn remove_lending_adapter(e: &Env, protocol: LendingAdapter) {
    let mut registry_map = read_lending_adapter_registry(e);
    registry_map.remove(protocol);
    write_lending_adapter_registry(e, registry_map);
}

pub fn verify_if_lending_adapter_exists(e: &Env, protocol: LendgingAdapter) -> bool {
    let registry_map = read_lending_adapter_registry(e);
    registry_map.contains_key(protocol)
}

