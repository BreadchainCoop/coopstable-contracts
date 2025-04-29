use soroban_sdk::{Env, Address, Symbol};
use crate::storage_types::{
    YieldAdapterRegistryMap,
    YIELD_REGISTRY_KEY, 
    REGISTRY_BUMP_AMOUNT, 
    REGISTRY_LIFETIME_THRESHOLD
};

pub fn read_yield_adapter_registry(e: &Env) -> YieldAdapterRegistryMap {
    if let Some(registry_map) = e
        .storage()
        .persistent().get::<Symbol, YieldAdapterRegistryMap>(&YIELD_REGISTRY_KEY) {
            e.storage()
                .persistent()
                .extend_ttl(
                    &YIELD_REGISTRY_KEY,
                    REGISTRY_BUMP_AMOUNT, 
                    REGISTRY_LIFETIME_THRESHOLD
                );
            
            registry_map
    } else {
        YieldAdapterRegistryMap::new(e)
    }
}

fn write_yield_adapter_registry(e: &Env, registry_map: YieldAdapterRegistryMap) {
    e.storage().persistent().set(&YIELD_REGISTRY_KEY, &registry_map);
    e.storage()
        .persistent()
        .extend_ttl(
            &YIELD_REGISTRY_KEY,
            REGISTRY_BUMP_AMOUNT,
            REGISTRY_LIFETIME_THRESHOLD
        );
}

pub fn register_yield_adapter(e: &Env, protocol: Symbol, adapter_id: Address) {
    let mut registry_map: YieldAdapterRegistryMap = read_yield_adapter_registry(e);
    if !registry_map.contains_value(adapter_id.clone()) {
        registry_map.set_adapter(protocol, adapter_id.clone());
        write_yield_adapter_registry(e, registry_map);
    }
}

pub fn remove_yield_adapter(e: &Env, protocol: Symbol) {
    let mut registry_map = read_yield_adapter_registry(e);
    registry_map.remove(protocol);
    write_yield_adapter_registry(e, registry_map);
}

pub fn verify_if_yield_adapter_exists(e: &Env, protocol: Symbol) -> bool {
    let registry_map = read_yield_adapter_registry(e);
    registry_map.contains_key(protocol)
}

pub fn get_yield_adapter(e: &Env, protocol: Symbol) -> Address {
    let registry_map = read_yield_adapter_registry(e);
    registry_map.get_adapter(protocol).unwrap()
}
