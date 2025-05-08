use soroban_sdk::{Address, Env, Symbol, Vec};
use crate::storage_types::{
    YieldAdapterRegistryMap,
    REGISTRY_BUMP_AMOUNT, 
    REGISTRY_LIFETIME_THRESHOLD
};

pub fn read_yield_adapter_registry(e: &Env, yield_type: Symbol) -> YieldAdapterRegistryMap {
    if let Some(registry_map) = e
        .storage()
        .persistent().get::<Symbol, YieldAdapterRegistryMap>(&yield_type) {
            e.storage()
                .persistent()
                .extend_ttl(
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
    e.storage().persistent().set(&registry_map.yield_type, &registry_map);
    e.storage()
        .persistent()
        .extend_ttl(
            &registry_map.yield_type,
            REGISTRY_LIFETIME_THRESHOLD,
            REGISTRY_BUMP_AMOUNT,
        );
}

pub fn register_yield_adapter(e: &Env, yield_type: Symbol, protocol: Symbol, adapter_id: Address) {
    let mut registry_map: YieldAdapterRegistryMap = read_yield_adapter_registry(e, yield_type.clone());
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

pub fn verify_if_yield_adapter_exists(e: &Env, yield_type: Symbol,protocol: Symbol) -> bool {
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
    registry_map.supports_asset(protocol,asset)
}

pub fn get_yield_adapters(e: &Env, yield_type: Symbol) -> Vec<Address> {
    let registry_map = read_yield_adapter_registry(e, yield_type.clone());
    registry_map.adapters()
}

pub fn get_yield_adapters_with_assets(e: &Env, yield_type: Symbol) -> Vec<(Address, Vec<Address>)> {
    let registry_map = read_yield_adapter_registry(e, yield_type.clone());
    registry_map.adapter_with_assets()
}